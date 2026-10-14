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
CACHE_ROOT = (
  ROOT
  / "target"
  / "mjwarp-parallel"
  / "warp-cache"
  / "1.15.0"
)
CACHE = (
  CACHE_ROOT
  / "1.15.0"
  / "wp_mujoco_warp._src.smooth_ca013d6"
)
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
nv = config["nv"]
body_mass_np = np.asarray(config["body_mass"], dtype=np.float32)
body_subtreemass_np = np.asarray(config["body_subtreemass"], dtype=np.float32)
xipos_row = np.asarray(config["xipos_row"], dtype=np.float32).reshape(nbody, 3)
xmat_row = np.asarray(config["xmat_row"], dtype=np.float32).reshape(nbody, 3, 3)
xanchor_row = np.asarray(config["xanchor_row"], dtype=np.float32).reshape(njnt, 3)
xaxis_row = np.asarray(config["xaxis_row"], dtype=np.float32).reshape(njnt, 3)
xipos_np = np.repeat(xipos_row[None, :, :], worlds, axis=0)
xmat_np = np.repeat(xmat_row[None, :, :, :], worlds, axis=0)
xanchor_np = np.repeat(xanchor_row[None, :, :], worlds, axis=0)
xaxis_np = np.repeat(xaxis_row[None, :, :], worlds, axis=0)

body_mass = wp.array(body_mass_np, dtype=float, device=device)
body_subtreemass = wp.array(body_subtreemass_np, dtype=float, device=device)
body_parentid = wp.array(config["body_parentid"], dtype=int, device=device)
body_rootid = wp.array(config["body_rootid"], dtype=int, device=device)
jnt_type = wp.array(config["jnt_type"], dtype=int, device=device)
jnt_dofadr = wp.array(config["jnt_dofadr"], dtype=int, device=device)
jnt_bodyid = wp.array(config["jnt_bodyid"], dtype=int, device=device)
xipos = wp.array(xipos_np, dtype=wp.vec3, device=device)
xmat = wp.array(xmat_np, dtype=wp.mat33, device=device)
xanchor = wp.array(xanchor_np, dtype=wp.vec3, device=device)
xaxis = wp.array(xaxis_np, dtype=wp.vec3, device=device)
subtree_com = wp.zeros((worlds, nbody), dtype=wp.vec3, device=device)
cdof = wp.zeros((worlds, nv), dtype=wp.spatial_vector, device=device)

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

subtree_values = np.asarray(subtree_com.numpy(), dtype=np.float32)
cdof_values = np.asarray(cdof.numpy(), dtype=np.float32)
output = {
  "schema": 1,
  "evidence_kind": "frozen-upstream-generated-com-cdof",
  "upstream_revision": revision,
  "warp_version": wp.__version__,
  "input_sha256": digest(HERE / "input.json"),
  "worlds": worlds,
  "nbody": nbody,
  "njnt": njnt,
  "nv": nv,
  "subtree_com": [float(value) for value in subtree_values.reshape(-1)],
  "cdof": [float(value) for value in cdof_values.reshape(-1)],
  "candidate_intermediate_inputs": False,
}
(HERE / "output.json").write_text(
  json.dumps(output, indent=2) + "\n", encoding="utf-8", newline="\n"
)
raw = {
  "schema": 1,
  "provenance": "direct_upstream_device_readback",
  "upstream_revision": revision,
  "fields": [
    float_payload("subtree_com", subtree_values),
    float_payload("cdof", cdof_values),
  ],
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
  "output_sha256": digest(HERE / "output.json"),
  "raw_bits_sha256": digest(HERE / "raw-bits.json"),
  "smooth_cu_sha256": digest(HERE / "smooth.cu"),
  "smooth_meta_sha256": digest(HERE / "smooth.meta"),
  "smooth_ptx_sha256": digest(HERE / "smooth.ptx"),
  "kernels": [
    "_subtree_com_init_002b02fd_cuda_kernel_forward",
    "_subtree_com_acc_350f0560_cuda_kernel_forward",
    "_subtree_div_0ef57c26_cuda_kernel_forward",
    "_cdof_3e73eb78_cuda_kernel_forward",
  ],
  "candidate_intermediate_inputs": False,
  "product_python_dependency": False,
}
(HERE / "authoring.json").write_text(
  json.dumps(authoring, indent=2) + "\n", encoding="utf-8", newline="\n"
)
print(json.dumps(authoring, indent=2))
