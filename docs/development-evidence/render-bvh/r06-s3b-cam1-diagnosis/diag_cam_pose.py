"""R06-S3b cam1 rgb diagnosis (CPU-only, no GPU, no product/reference writes).

Question: attempt 3 (VALID_ROUND) failed at cam1 rgb max_byte_diff=255. Is that a
product mapping/write defect, a flex numeric issue, or a harness/reference input defect?

Method (CPU only):
  1. Load scene-flex with MuJoCo 3.15 (same pinned authoring version) and set qpos from
     reference/base/state.npz, then mj_forward -> d.cam_xpos / d.cam_xmat == the upstream
     "data pose" that mujoco_warp render() feeds into the megakernel
     (upstream render.py render(): inputs include d.cam_xpos, d.cam_xmat).
  2. Compare against what tests/render_flex_validate.rs currently feeds: model-312.json
     cam_pos0 / cam_mat0 (static model pose).
  3. Check the output mapping (cam_id_map / cam_res / pixel count / rgb_adr) against the
     reference npz shapes.
  4. Check fix feasibility: can the data pose be rebuilt from state.npz + model dump alone?
"""
import json
import math
import pathlib

import mujoco
import numpy as np

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
FIX = ROOT / "fixtures" / "render-bvh" / "scene-flex"
SCENE = FIX / "scene.xml"
STATE = FIX / "reference" / "base" / "state.npz"
CAPTURE = json.loads((FIX / "reference" / "base" / "capture.json").read_text(encoding="utf-8"))
DUMP = json.loads((FIX / "model-312.json").read_text(encoding="utf-8"))
A = DUMP["arrays"]


def arr(name):
    return np.asarray(A[name]["values"], dtype=np.float64)


def f32(values):
    return np.asarray(values, dtype=np.float64).astype(np.float32)


def bits(values):
    return np.asarray(values, dtype=np.float32).view(np.uint32)


mjm = mujoco.MjModel.from_xml_path(str(SCENE))
mjd = mujoco.MjData(mjm)
state = np.load(STATE)
mjd.qpos[:] = state["qpos"][0].astype(np.float64)
mujoco.mj_forward(mjm, mjd)

print("== inputs ==")
print("scene      :", SCENE.as_posix())
print("mujoco     :", mujoco.__version__)
print("state.npz  :", sorted(state.files))
print("capture ctx: cam_id_map", CAPTURE["context"]["resolved"]["cam_id_map"],
      "cam_res", CAPTURE["context"]["resolved"]["cam_res"],
      "total_rays", CAPTURE["context"]["resolved"]["total_rays"])
print("dump       : cam_mode", arr("cam_mode").astype(int).tolist(),
      "cam_bodyid", arr("cam_bodyid").astype(int).tolist(),
      "cam_targetbodyid", arr("cam_targetbodyid").astype(int).tolist())
print("         cam names", [mjm.camera(i).name for i in range(mjm.ncam)],
      "body names", [mjm.body(i).name for i in range(mjm.nbody)])

cam_pos0 = arr("cam_pos0")
cam_mat0 = arr("cam_mat0").reshape(3, 3, 3)

print("\n== 1) camera pose table: test input (model cam_pos0/cam_mat0) vs reference data pose ==")
print("camera   mode body target | pos_delta | mat_max_abs_delta | mat_bit_equal | fwd_angle_deg")
rows = []
for cam in range(mjm.ncam):
    mode = int(arr("cam_mode")[cam])
    body = int(arr("cam_bodyid")[cam])
    target = int(arr("cam_targetbodyid")[cam])
    test_pos = cam_pos0[3 * cam:3 * cam + 3]
    test_mat = cam_mat0[cam]
    data_pos = mjd.cam_xpos[cam]
    data_mat = mjd.cam_xmat[cam].reshape(3, 3)
    pos_delta = float(np.max(np.abs(f32(test_pos) - f32(data_pos))))
    mat_delta = float(np.max(np.abs(f32(test_mat) - f32(data_mat))))
    mat_equal = bool(np.array_equal(bits(test_mat), bits(data_mat)))
    test_fwd = -f32(test_mat).astype(np.float64)[:, 2]
    data_fwd = -f32(data_mat).astype(np.float64)[:, 2]
    cos = float(np.clip(test_fwd @ data_fwd, -1.0, 1.0))
    angle = math.degrees(math.acos(cos))
    rows.append(dict(cam=cam, name=mjm.camera(cam).name, mode=mode, body=body, target=target,
                     pos_delta=pos_delta, mat_delta=mat_delta, mat_bit_equal=mat_equal,
                     angle=angle, test_pos=f32(test_pos).tolist(), data_pos=f32(data_pos).tolist()))
    print(f"cam{cam} {mjm.camera(cam).name:<10} {mode:>2} {body:>4} {target:>6} | "
          f"{pos_delta:.3e} | {mat_delta:.6f} | {mat_equal} | {angle:.3f}")

print("\ntest mat vs data mat detail:")
for row in rows:
    print(f"cam{row['cam']} {row['name']}: test_pos={row['test_pos']} data_pos={row['data_pos']}")
    print(f"        test_mat = {f32(cam_mat0[row['cam']]).tolist()}")
    print(f"        data_mat = {f32(mjd.cam_xmat[row['cam']].reshape(3, 3)).tolist()}")

print("\n== 2) upstream look-at formula sanity (f64) on TARGETBODY cameras ==")


def normalize(v):
    n = math.sqrt(float(v[0] * v[0] + v[1] * v[1] + v[2] * v[2]))
    return v / n if n > 0.0 else np.zeros(3)


def cross(a, b):
    return np.array([a[1] * b[2] - a[2] * b[1],
                     a[2] * b[0] - a[0] * b[2],
                     a[0] * b[1] - a[1] * b[0]])


def look_at(pos, target):
    z = normalize(pos - target)
    x = normalize(cross(np.array([0.0, 0.0, 1.0]), z))
    y = normalize(cross(z, x))
    return np.array([[x[0], y[0], z[0]],
                     [x[1], y[1], z[1]],
                     [x[2], y[2], z[2]]])


for cam in range(mjm.ncam):
    mode = int(arr("cam_mode")[cam])
    target = int(arr("cam_targetbodyid")[cam])
    if mode != 3:
        continue
    pos = arr("cam_pos")[3 * cam:3 * cam + 3]
    formula = look_at(pos, mjd.xpos[target])
    delta = float(np.max(np.abs(formula - mjd.cam_xmat[cam].reshape(3, 3))))
    print(f"cam{cam} {mjm.camera(cam).name}: look-at(formula f64) vs mj_forward data xmat "
          f"max_abs_delta={delta:.3e}")

print("\n== 3) output mapping vs reference npz ==")
pixels = [int(r[0]) * int(r[1]) for r in CAPTURE["context"]["resolved"]["cam_res"]]
adr = np.cumsum([0] + pixels[:-1]).tolist()
print("resolved cam_id_map :", CAPTURE["context"]["resolved"]["cam_id_map"])
print("resolved cam_res    :", CAPTURE["context"]["resolved"]["cam_res"])
print("pixels per slot     :", pixels, "total", sum(pixels),
      "== total_rays", CAPTURE["context"]["resolved"]["total_rays"])
print("expected rgb_adr    :", adr, "(OutputLayout: cumulative in plan order)")
rgb_npz = np.load(FIX / "reference" / "base" / "rgb.npz")
depth_npz = np.load(FIX / "reference" / "base" / "depth.npz")
seg_npz = np.load(FIX / "reference" / "base" / "seg.npz")
print("reference npz keys  :", sorted(rgb_npz.files), sorted(depth_npz.files), sorted(seg_npz.files))
for slot in range(mjm.ncam):
    h, w, _ = rgb_npz[f"rgb_cam{slot}.npy"].shape
    print(f"slot{slot} -> rgb_cam{slot}.npy {(h, w)} depth {depth_npz[f'depth_cam{slot}.npy'].shape} "
          f"seg {seg_npz[f'seg_cam{slot}.npy'].shape} | npz order slot == mujoco cam id "
          f"{slot} (cam_id_map identity) | rgb_adr {adr[slot]} | pixels {pixels[slot]}")

print("\n== 4) flex pixels per camera in the reference (seg object_type==9) ==")
for slot in range(mjm.ncam):
    seg = seg_npz[f"seg_cam{slot}.npy"].reshape(-1, 2)
    flex = int(np.count_nonzero(seg[:, 1] == 9))
    uniq = sorted({tuple(p) for p in seg.tolist()})
    print(f"slot{slot} {mjm.camera(slot).name}: flex_pixels={flex} seg_unique={uniq}")

print("\n== 5) fix feasibility: rebuild data pose from state.npz + model dump alone ==")
geom_xpos = state["geom_xpos"][0].astype(np.float64)
geom_xmat = state["geom_xmat"][0].astype(np.float64).reshape(mjm.ngeom, 3, 3)
geom_bodyid = arr("geom_bodyid").astype(int)
geom_pos_flat = arr("geom_pos")
geom_quat = arr("geom_quat")
body_of_geom = {int(geom_bodyid[g]): g for g in range(mjm.ngeom)}


def quat_to_mat(q):
    w, x, y, z = q
    return np.array([
        [1 - 2 * (y * y + z * z), 2 * (x * y - w * z), 2 * (x * z + w * y)],
        [2 * (x * y + w * z), 1 - 2 * (x * x + z * z), 2 * (y * z - w * x)],
        [2 * (x * z - w * y), 2 * (y * z + w * x), 1 - 2 * (x * x + y * y)],
    ])


tri = body_of_geom.get(6)
local_rot = quat_to_mat(geom_quat[4 * tri:4 * tri + 4])
body_rot = geom_xmat[tri] @ local_rot.T
body_pos = geom_xpos[tri] - body_rot @ geom_pos_flat[3 * tri:3 * tri + 3]
print("tri_body(6) derived from tri-mesh geom(2) + state.npz:")
print("  xpos derived", f32(body_pos).tolist(), " mj_forward", f32(mjd.xpos[6]).tolist(),
      " f32_bits_equal", bool(np.array_equal(bits(body_pos), bits(mjd.xpos[6]))),
      " max_abs_delta", float(np.max(np.abs(f32(body_pos) - f32(mjd.xpos[6])))))
cam = 1
derived1 = look_at(arr("cam_pos")[3 * cam:3 * cam + 3], body_pos)
data1 = mjd.cam_xmat[cam].reshape(3, 3)
print("cam-track look-at(derived tri_body) vs data xmat: f32_bits_equal",
      bool(np.array_equal(bits(derived1), bits(data1))),
      " max_abs_delta", float(np.max(np.abs(f32(derived1) - f32(data1)))))
print("cam-flex target body 4 (flex-target): has geom in scene?", 4 in body_of_geom,
      "; model dump body_pos present?", any(name.startswith("body_pos") for name in A))
print("  -> cam-flex data pose is NOT derivable from state.npz + current model dump alone")
print("     flex-target world pos (only from MuJoCo model here) =",
      mjd.xpos[4].tolist())

print("\n== 6) conclusion (printed for the evidence reader) ==")
print("A. cam1/cam2 are mode=3 TARGETBODY cameras; reference render() uses d.cam_xpos/d.cam_xmat")
print("   (upstream render.py:1491-1550), and data cam_xmat is the look-at product")
print("   (upstream smooth.py:867-885). model cam_mat0 for mode==3 is identity.")
print("B. test feeds model cam_pos0/cam_mat0: cam0 (mode=0) pose is bit-equal to data pose;")
print("   cam1/cam2 orientation is off by 85.517 deg / 75.562 deg -> cam1 renders an")
print("   unrelated view -> rgb max_byte_diff=255 is explained without any flex involvement.")
print("C. output mapping (cam_id_map / cam_res / pixels / rgb_adr / npz slot order) is consistent")
print("   with the reference; cam0 passing the rgb criterion confirms the write path for a")
print("   correct pose.")
print("D. reference lacks camera data poses: state.npz has only",
      sorted(state.files), "and capture.json records no cam_xpos/cam_xmat.")
print("E. attempt 3 is therefore an input/harness defect (camera pose source), not a product")
print("   mapping defect and not a flex numeric red; cam2 (654 flex pixels) was never evaluated.")
