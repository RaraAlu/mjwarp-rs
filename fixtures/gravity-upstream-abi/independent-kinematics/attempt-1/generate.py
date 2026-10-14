import hashlib
import json
import pathlib
import shutil
import subprocess
import sys

import numpy as np

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parents[2]
UPSTREAM = ROOT / "target" / "mjwarp-parallel" / "upstream"
CACHE_ROOT = ROOT / "target" / "mjwarp-parallel" / "warp-cache" / "1.15.0"
CACHE = CACHE_ROOT / "1.15.0" / "wp_mujoco_warp._src.smooth_ca013d6"
REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
CU_SHA256 = "844dca14a5932827729313a2500f343bdff950b8cdb6170aed1c5973e0ff290e"
PTX_SHA256 = "4179acd01fe8e7064fa97566cfe4158dd9ad4b89761dcab0da88001d9b521f57"
META_SHA256 = "2bb1bcdb12cbcce6143af26da5798c047ed628d4817c4b76358fc646db0e3cc8"
SMOOTH_SHA256 = "d0952a647a55b4c5f3e89f7224c4842d15c504fe40bafb2fd8bd9b9bd8ef8f4b"
TYPES_SHA256 = "7970e04d40dd15b33171dd1558c369cf45478b6ffdb31d8a805aec93702ec717"


def digest(path):
  return hashlib.sha256(path.read_bytes()).hexdigest()


def refuse_existing_outputs():
  for name in [
    "authoring.json",
    "output.json",
    "raw-bits.json",
    "smooth.cu",
    "smooth.meta",
    "smooth.ptx",
  ]:
    if (HERE / name).exists():
      raise RuntimeError(f"authoring output already exists: {name}")


def float_payload(name, values):
  flat = np.asarray(values, dtype=np.float32).reshape(-1)
  raw = flat.astype("<f4", copy=False).tobytes(order="C")
  return {
    "field": name,
    "count": int(flat.size),
    "little_endian_sha256": hashlib.sha256(raw).hexdigest(),
    "bits": [f"{int(value):08x}" for value in flat.view(np.uint32)],
  }


def exact_bits(actual, expected, field):
  actual_bits = np.asarray(actual, dtype=np.float32).reshape(-1).view(np.uint32)
  expected_bits = np.asarray(expected, dtype=np.float32).reshape(-1).view(np.uint32)
  if not np.array_equal(actual_bits, expected_bits):
    mismatch = int(np.flatnonzero(actual_bits != expected_bits)[0])
    raise RuntimeError(f"{field} bit mismatch at {mismatch}")


refuse_existing_outputs()
config = json.loads((HERE / "input.json").read_text(encoding="utf-8"))
if config["upstream_revision"] != REVISION or config["warp_version"] != "1.15.0":
  raise RuntimeError("unexpected frozen input identity")
revision = subprocess.check_output(
  ["git", "-C", str(UPSTREAM), "rev-parse", "HEAD"], text=True
).strip()
if revision != REVISION:
  raise RuntimeError(f"unexpected upstream revision: {revision}")
if digest(UPSTREAM / "mujoco_warp" / "_src" / "smooth.py") != SMOOTH_SHA256:
  raise RuntimeError("smooth.py hash mismatch")
if digest(UPSTREAM / "mujoco_warp" / "_src" / "types.py") != TYPES_SHA256:
  raise RuntimeError("types.py hash mismatch")

legacy_path = (HERE / config["legacy_static_input"]).resolve()
if digest(legacy_path) != config["legacy_static_input_sha256"]:
  raise RuntimeError("legacy static input hash mismatch")
legacy = json.loads(legacy_path.read_text(encoding="utf-8"))

sys.path.insert(0, str(UPSTREAM))
import warp as wp

if wp.__version__ != "1.15.0":
  raise RuntimeError(f"unexpected Warp version: {wp.__version__}")
wp.config.kernel_cache_dir = str(CACHE_ROOT)
wp.init()
import mujoco_warp._src.smooth as smooth

device = "cuda:0"
wp.load_module(module=wp.get_module(smooth.__name__), device=device)
cache_files = {
  "smooth.cu": CACHE / "wp_mujoco_warp._src.smooth_ca013d6.cu",
  "smooth.meta": CACHE / "wp_mujoco_warp._src.smooth_ca013d6.meta",
  "smooth.ptx": CACHE / "wp_mujoco_warp._src.smooth_ca013d6.sm89.ptx",
}
for name, expected in [
  ("smooth.cu", CU_SHA256),
  ("smooth.meta", META_SHA256),
  ("smooth.ptx", PTX_SHA256),
]:
  if digest(cache_files[name]) != expected:
    raise RuntimeError(f"{name} hash mismatch")

worlds = config["worlds"]
nbody = config["nbody"]
njnt = config["njnt"]
nq = config["nq"]
nv = config["nv"]
nbranch = config["nbranch"]
nmocap = config["nmocap"]

qpos0_np = np.asarray(config["qpos0"], dtype=np.float32).reshape(-1, nq)
body_pos_np = np.asarray(config["body_pos"], dtype=np.float32).reshape(-1, nbody, 3)
body_quat_np = np.asarray(config["body_quat_xyzw"], dtype=np.float32).reshape(-1, nbody, 4)
jnt_pos_np = np.asarray(config["jnt_pos"], dtype=np.float32).reshape(-1, njnt, 3)
jnt_axis_np = np.asarray(config["jnt_axis"], dtype=np.float32).reshape(-1, njnt, 3)
qpos_np = np.asarray(config["qpos"], dtype=np.float32).reshape(worlds, nq)
mocap_pos_np = np.repeat(
  np.asarray(config["mocap_pos"], dtype=np.float32).reshape(1, nmocap, 3),
  worlds,
  axis=0,
)
mocap_quat_np = np.repeat(
  np.asarray(config["mocap_quat_xyzw"], dtype=np.float32).reshape(1, nmocap, 4),
  worlds,
  axis=0,
)
body_ipos_np = np.asarray(config["body_ipos"], dtype=np.float32).reshape(-1, nbody, 3)
body_iquat_np = np.asarray(config["body_iquat_xyzw"], dtype=np.float32).reshape(-1, nbody, 4)
body_mass_np = np.asarray(config["body_mass"], dtype=np.float32).reshape(-1, nbody)
body_subtreemass_np = np.asarray(config["body_subtreemass"], dtype=np.float32).reshape(-1, nbody)

qpos0 = wp.array(qpos0_np, dtype=float, device=device)
body_parentid = wp.array(config["body_parentid"], dtype=int, device=device)
body_rootid = wp.array(config["body_rootid"], dtype=int, device=device)
body_mocapid = wp.array(config["body_mocapid"], dtype=int, device=device)
body_jntnum = wp.array(config["body_jntnum"], dtype=int, device=device)
body_jntadr = wp.array(config["body_jntadr"], dtype=int, device=device)
body_pos = wp.array(body_pos_np, dtype=wp.vec3, device=device)
body_quat = wp.array(body_quat_np, dtype=wp.quat, device=device)
jnt_type = wp.array(config["jnt_type"], dtype=int, device=device)
jnt_qposadr = wp.array(config["jnt_qposadr"], dtype=int, device=device)
jnt_dofadr = wp.array(config["jnt_dofadr"], dtype=int, device=device)
jnt_bodyid = wp.array(config["jnt_bodyid"], dtype=int, device=device)
jnt_pos = wp.array(jnt_pos_np, dtype=wp.vec3, device=device)
jnt_axis = wp.array(jnt_axis_np, dtype=wp.vec3, device=device)
body_branches = wp.array(config["body_branches"], dtype=int, device=device)
body_branch_start = wp.array(config["body_branch_start"], dtype=int, device=device)
qpos = wp.array(qpos_np, dtype=float, device=device)
mocap_pos = wp.array(mocap_pos_np, dtype=wp.vec3, device=device)
mocap_quat = wp.array(mocap_quat_np, dtype=wp.quat, device=device)
body_ipos = wp.array(body_ipos_np, dtype=wp.vec3, device=device)
body_iquat = wp.array(body_iquat_np, dtype=wp.quat, device=device)
body_mass = wp.array(body_mass_np, dtype=float, device=device)
body_subtreemass = wp.array(body_subtreemass_np, dtype=float, device=device)

xpos_seed = np.full((worlds, nbody, 3), np.nan, dtype=np.float32)
xpos_seed[:, 0, :] = 0.0
xquat_seed = np.full((worlds, nbody, 4), np.nan, dtype=np.float32)
xquat_seed[:, 0, :] = np.asarray([0.0, 0.0, 0.0, 1.0], dtype=np.float32)
xpos = wp.array(xpos_seed, dtype=wp.vec3, device=device)
xquat = wp.array(xquat_seed, dtype=wp.quat, device=device)
xanchor = wp.full((worlds, njnt), value=np.nan, dtype=wp.vec3, device=device)
xaxis = wp.full((worlds, njnt), value=np.nan, dtype=wp.vec3, device=device)
xmat = wp.full((worlds, nbody), value=np.nan, dtype=wp.mat33, device=device)
xipos = wp.full((worlds, nbody), value=np.nan, dtype=wp.vec3, device=device)
ximat = wp.full((worlds, nbody), value=np.nan, dtype=wp.mat33, device=device)
subtree_com = wp.zeros((worlds, nbody), dtype=wp.vec3, device=device)
cdof = wp.zeros((worlds, nv), dtype=wp.spatial_vector, device=device)

wp.launch(
  smooth._kinematics_branch,
  dim=(worlds, nbranch),
  inputs=[
    qpos0,
    body_parentid,
    body_mocapid,
    body_jntnum,
    body_jntadr,
    body_pos,
    body_quat,
    jnt_type,
    jnt_qposadr,
    jnt_pos,
    jnt_axis,
    body_branches,
    body_branch_start,
    qpos,
    mocap_pos,
    mocap_quat,
  ],
  outputs=[xpos, xquat, xanchor, xaxis],
  device=device,
)
wp.launch(
  smooth._compute_body_matrices,
  dim=(worlds, nbody),
  inputs=[xquat],
  outputs=[xmat],
  device=device,
)
wp.launch(
  smooth._compute_body_inertial_frames,
  dim=(worlds, nbody),
  inputs=[body_ipos, body_iquat, xpos, xquat],
  outputs=[xipos, ximat],
  device=device,
)
wp.launch(
  smooth._subtree_com_init,
  dim=(worlds, nbody),
  inputs=[body_mass, xipos],
  outputs=[subtree_com],
  device=device,
)
for level in reversed(config["body_tree_levels"]):
  body_tree = wp.array(level, dtype=int, device=device)
  wp.launch(
    smooth._subtree_com_acc,
    dim=(worlds, len(level)),
    inputs=[body_parentid, subtree_com, body_tree],
    outputs=[subtree_com],
    device=device,
  )
wp.launch(
  smooth._subtree_div,
  dim=(worlds, nbody),
  inputs=[body_subtreemass, subtree_com],
  outputs=[subtree_com],
  device=device,
)
wp.launch(
  smooth._cdof,
  dim=(worlds, njnt),
  inputs=[
    body_rootid,
    jnt_type,
    jnt_dofadr,
    jnt_bodyid,
    xmat,
    xanchor,
    xaxis,
    subtree_com,
  ],
  outputs=[cdof],
  device=device,
)
wp.synchronize_device(device)

values = {
  "xpos": np.asarray(xpos.numpy(), dtype=np.float32),
  "xquat_xyzw": np.asarray(xquat.numpy(), dtype=np.float32),
  "xipos": np.asarray(xipos.numpy(), dtype=np.float32),
  "xmat": np.asarray(xmat.numpy(), dtype=np.float32),
  "xanchor": np.asarray(xanchor.numpy(), dtype=np.float32),
  "xaxis": np.asarray(xaxis.numpy(), dtype=np.float32),
  "subtree_com": np.asarray(subtree_com.numpy(), dtype=np.float32),
  "cdof": np.asarray(cdof.numpy(), dtype=np.float32),
}

legacy_xipos = np.repeat(
  np.asarray(legacy["xipos_row"], dtype=np.float32).reshape(1, nbody, 3),
  worlds,
  axis=0,
)
legacy_xmat = np.repeat(
  np.asarray(legacy["xmat_row"], dtype=np.float32).reshape(1, nbody, 3, 3),
  worlds,
  axis=0,
)
legacy_xanchor = np.repeat(
  np.asarray(legacy["xanchor_row"], dtype=np.float32).reshape(1, njnt, 3),
  worlds,
  axis=0,
)
legacy_xaxis = np.repeat(
  np.asarray(legacy["xaxis_row"], dtype=np.float32).reshape(1, njnt, 3),
  worlds,
  axis=0,
)
legacy_xquat = np.tile(
  np.asarray([0.0, 0.0, 0.0, 1.0], dtype=np.float32),
  (worlds, nbody, 1),
)
exact_bits(values["xpos"], legacy_xipos, "xpos")
exact_bits(values["xquat_xyzw"], legacy_xquat, "xquat")
exact_bits(values["xipos"], legacy_xipos, "xipos")
exact_bits(values["xmat"], legacy_xmat, "xmat")
exact_bits(values["xanchor"], legacy_xanchor, "xanchor")
exact_bits(values["xaxis"], legacy_xaxis, "xaxis")

legacy_output_path = legacy_path.parent / "output.json"
legacy_output = json.loads(legacy_output_path.read_text(encoding="utf-8"))
exact_bits(values["subtree_com"], legacy_output["subtree_com"], "subtree_com")
exact_bits(values["cdof"], legacy_output["cdof"], "cdof")

output = {
  "schema": 1,
  "evidence_kind": "frozen-upstream-generated-qpos-kinematics-com-cdof",
  "upstream_revision": revision,
  "warp_version": wp.__version__,
  "input_sha256": digest(HERE / "input.json"),
  "worlds": worlds,
  "nbody": nbody,
  "njnt": njnt,
  "nq": nq,
  "nv": nv,
  **{name: [float(value) for value in field.reshape(-1)] for name, field in values.items()},
  "legacy_static_fields_bit_identical": True,
  "legacy_com_cdof_bit_identical": True,
  "candidate_kinematics_inputs": False,
}
(HERE / "output.json").write_text(
  json.dumps(output, indent=2) + "\n", encoding="utf-8", newline="\n"
)
raw = {
  "schema": 1,
  "provenance": "direct_upstream_device_readback",
  "upstream_revision": revision,
  "fields": [float_payload(name, field) for name, field in values.items()],
}
(HERE / "raw-bits.json").write_text(
  json.dumps(raw, indent=2) + "\n", encoding="utf-8", newline="\n"
)
for name, source in cache_files.items():
  shutil.copyfile(source, HERE / name)

authoring = {
  "schema": 1,
  "upstream_revision": revision,
  "warp_version": wp.__version__,
  "python_version": sys.version,
  "device": str(wp.get_device(device)),
  "smooth_source_sha256": SMOOTH_SHA256,
  "types_source_sha256": TYPES_SHA256,
  "input_sha256": digest(HERE / "input.json"),
  "legacy_static_input_sha256": digest(legacy_path),
  "legacy_com_output_sha256": digest(legacy_output_path),
  "output_sha256": digest(HERE / "output.json"),
  "raw_bits_sha256": digest(HERE / "raw-bits.json"),
  "smooth_cu_sha256": digest(HERE / "smooth.cu"),
  "smooth_meta_sha256": digest(HERE / "smooth.meta"),
  "smooth_ptx_sha256": digest(HERE / "smooth.ptx"),
  "kernels": [
    "_kinematics_branch_95f8028e_cuda_kernel_forward",
    "_compute_body_matrices_27760750_cuda_kernel_forward",
    "_compute_body_inertial_frames_d3bdd81a_cuda_kernel_forward",
    "_subtree_com_init_002b02fd_cuda_kernel_forward",
    "_subtree_com_acc_350f0560_cuda_kernel_forward",
    "_subtree_div_0ef57c26_cuda_kernel_forward",
    "_cdof_3e73eb78_cuda_kernel_forward",
  ],
  "qpos_generated_kinematics": True,
  "legacy_static_fields_only_as_expected_assertions": True,
  "candidate_kinematics_inputs": False,
  "product_python_dependency": False,
}
(HERE / "authoring.json").write_text(
  json.dumps(authoring, indent=2) + "\n", encoding="utf-8", newline="\n"
)
print(json.dumps(authoring, indent=2))
