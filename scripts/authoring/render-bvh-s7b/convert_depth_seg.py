"""S7b1 authoring-only converter: frozen S0b depth/seg npz -> product-readable JSON.

OFFLINE ONLY. Reads `fixtures/render-bvh/scene-a/reference/{depth,seg}.npz`
(MJWarp 3.15 reference, upstream 71da24d) and writes
`fixtures/render-bvh/scene-a/s7b/depth_seg.json` with the full-image f32 depth bits
and int32 (object_id, object_type) pairs. CPU only for depth/seg; the camera data poses
are read from the same 3.15 environment with one `mjwarp.forward` (GPU, seconds, no
rendering and no re-capture): `d.cam_xpos`/`d.cam_xmat` f32 bits per camera are the
data poses the frozen render used. Camera 1 is a TARGETBODY camera, so its xmat is a
forward product (tracking look-at), not `quat_to_mat(cam_quat)`.

Encoding (also written into the JSON `encoding` block):
- depth: IEEE-754 binary32 bits as unsigned decimal u32, per camera, row-major
  (row = y from 0..height-1, x fastest), camera flat size width*height.
- seg: [object_id, object_type] int32 pairs in the same pixel order.
- camera data pose: cam_xpos/cam_xmat as f32 bit patterns (u32 decimal), xmat row-major.

Self-checks before writing (any failure aborts):
- origins in `rays.npz` camera_pixels equal f32(model cam_pos) and data `cam_xpos` bits;
- fixed camera xmat vs f64 `quat_to_mat(cam_quat)` within 1e-6;
- TARGETBODY camera xmat vs f64 look-at (`z=normalize(pos-target)`, up=world z) within 1e-6;
- depth/seg lengths/aggregates/round-trip (see `verify_document`).

Run (pinned venv, repo root):
  target/mjwarp-parallel/venv/Scripts/python.exe scripts/authoring/render-bvh-s7b/convert_depth_seg.py
"""

from __future__ import annotations

import hashlib
import json
import pathlib
import struct
import sys
import time

import numpy as np

ROOT = pathlib.Path(__file__).resolve().parents[3]
REFERENCE = ROOT / "fixtures/render-bvh/scene-a/reference"
OUTPUT = ROOT / "fixtures/render-bvh/scene-a/s7b/depth_seg.json"
CONTEXT = REFERENCE / "context.json"
DEPTH_NPZ = REFERENCE / "depth.npz"
SEG_NPZ = REFERENCE / "seg.npz"
RAYS_NPZ = REFERENCE / "rays.npz"
MODEL_312 = ROOT / "fixtures/render-bvh/scene-a/model-312.json"
MJB_315 = ROOT / "fixtures/render-bvh/scene-a/scene-315.mjb"
UPSTREAM = "71da24d956378a87a703b6e1442b13aec0c4ac29"
EXPECTED_NPZ_SHA256 = {
    "depth.npz": "0fa5582170a2b2a117e46868930fa2b4270dd993b0f5d86dc603981e0a2fd7d3",
    "seg.npz": "2153fa728d634c032cf38847761803c4faa806aa554e108a2e0bd7b8ac8e6a97",
    "rays.npz": "83146ae50946537401be817a44bfe3af58320df475dabc49206aa47eec9d50ab",
}
EXPECTED_CAM_RES = [(320, 240), (240, 180)]
# mjCAMLIGHT_* camera modes (MJCF `mode`): 0 fixed, 3 target body.
CAM_MODE_FIXED = 0
CAM_MODE_TARGETBODY = 3



def sha256(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def script_sha256() -> str:
    raw = pathlib.Path(__file__).read_bytes().replace(b"\r\n", b"\n")
    return hashlib.sha256(raw).hexdigest()


def depth_bits(array: np.ndarray) -> list[int]:
    return np.ascontiguousarray(array, dtype=np.float32).view(np.uint32).reshape(-1).tolist()


def seg_pairs(array: np.ndarray) -> list[int]:
    return np.ascontiguousarray(array, dtype=np.int32).reshape(-1).tolist()


def quat_to_mat_f64(quat) -> np.ndarray:
    """MuJoCo `mju_quat2Mat` in f64 (w, x, y, z) -> row-major 3x3."""
    w, x, y, z = (float(v) for v in quat)
    return np.asarray(
        [
            [1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
            [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
            [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)],
        ],
        dtype=np.float64,
    )


def look_at_f64(position, target) -> np.ndarray:
    """MJWarp `_cam_local_to_global` TARGETBODY rows: x=up×z, y=z×x, z=normalize(pos-target)."""
    zaxis = np.asarray(position, dtype=np.float64) - np.asarray(target, dtype=np.float64)
    zaxis = zaxis / np.linalg.norm(zaxis)
    xaxis = np.cross(np.asarray([0.0, 0.0, 1.0]), zaxis)
    xaxis = xaxis / np.linalg.norm(xaxis)
    yaxis = np.cross(zaxis, xaxis)
    yaxis = yaxis / np.linalg.norm(yaxis)
    # MJWarp stores x/y/z axes as matrix columns; row-major flattening is the transpose.
    return np.asarray([xaxis, yaxis, zaxis], dtype=np.float64).T


def camera_data_checks(rays: "np.lib.npyio.NpzFile", model_312: dict) -> list[dict]:
    """Camera data poses from the frozen 3.15 environment + independent pose checks.

    One `mjwarp.forward` on `scene-315.mjb` (same pinned env / model as the S0b capture)
    produces the f32 data `cam_xpos`/`cam_xmat` the render used. `rays.npz`
    camera_pixels origins must match these bit for bit; the fixed camera xmat must match
    f64 `quat_to_mat(cam_quat)` and the TARGETBODY camera xmat must match an f64 look-at
    against the target body position (both within 1e-6).
    """
    import mujoco  # noqa: PLC0415 - pinned authoring environment only
    import mujoco_warp as mjw  # noqa: PLC0415
    import warp as wp  # noqa: PLC0415

    mjm = mujoco.MjModel.from_binary_path(str(MJB_315))
    mjd = mujoco.MjData(mjm)
    mujoco.mj_forward(mjm, mjd)
    model = mjw.put_model(mjm)
    data = mjw.put_data(mjm, mjd, nworld=1)
    mjw.forward(model, data)
    wp.synchronize()
    xpos = np.ascontiguousarray(np.asarray(data.cam_xpos.numpy())[0], dtype=np.float32)
    xmat = np.ascontiguousarray(np.asarray(data.cam_xmat.numpy())[0].reshape(-1, 9), dtype=np.float32)
    native_xpos = np.asarray(mjd.cam_xpos, dtype=np.float64)
    cam_pos = np.asarray(model_312["arrays"]["cam_pos"]["values"], dtype=np.float64)
    cam_quat = np.asarray(model_312["arrays"]["cam_quat"]["values"], dtype=np.float64)
    category = np.asarray(rays["ray_category"], dtype=np.int8)
    origin = np.asarray(rays["ray_origin"], dtype=np.float32)
    camera_pixels = np.flatnonzero(category == 0)
    if camera_pixels.size != 128:
        raise SystemExit(f"camera_pixels ray count {camera_pixels.size} != 128")
    checks = []
    for index in range(int(mjm.ncam)):
        block = camera_pixels[index * 64 : (index + 1) * 64]
        unique = np.unique(origin[block], axis=0)
        if unique.shape[0] != 1:
            raise SystemExit(f"camera {index}: camera_pixel origins not unique: {unique.shape[0]}")
        origin_bits = np.ascontiguousarray(unique[0], dtype=np.float32).view(np.uint32).tolist()
        expected_bits = np.ascontiguousarray(cam_pos[3 * index : 3 * index + 3], dtype=np.float32)
        if origin_bits != expected_bits.view(np.uint32).tolist():
            raise SystemExit(f"camera {index}: ray origin bits {origin_bits} != f32(cam_pos)")
        data_pos_bits = np.ascontiguousarray(xpos[index], dtype=np.float32).view(np.uint32).tolist()
        if origin_bits != data_pos_bits:
            raise SystemExit(f"camera {index}: data cam_xpos bits {data_pos_bits} != ray origins")
        mode = int(mjm.cam_mode[index])
        if mode == CAM_MODE_FIXED:
            reference = quat_to_mat_f64(cam_quat[4 * index : 4 * index + 4])
        elif mode == CAM_MODE_TARGETBODY:
            target = int(mjm.cam_targetbodyid[index])
            if target < 0:
                raise SystemExit(f"camera {index}: TARGETBODY with no target")
            reference = look_at_f64(native_xpos[index], np.asarray(mjd.xpos[target], dtype=np.float64))
        else:
            raise SystemExit(f"camera {index}: unsupported mode {mode} for this fixture")
        error = float(np.max(np.abs(xmat[index].astype(np.float64).reshape(3, 3) - reference)))
        if error > 1.0e-6:
            raise SystemExit(f"camera {index}: data cam_xmat vs f64 reference error {error} > 1e-6")
        checks.append(
            {
                "index": index,
                "ray_count": int(block.size),
                "mode": mode,
                "target_bodyid": int(mjm.cam_targetbodyid[index]),
                "origin_bits": [int(value) for value in origin_bits],
                "xmat_bits": [int(value) for value in np.ascontiguousarray(xmat[index], dtype=np.float32).view(np.uint32)],
                "xmat_vs_f64_max_abs_error": error,
                "source": "mjwarp.forward(scene-315.mjb) data cam_xpos/cam_xmat f32 bits; "
                "origins cross-checked against rays.npz camera_pixels",
            }
        )
    return checks


def verify_document(document: dict, depth: "np.lib.npyio.NpzFile", seg: "np.lib.npyio.NpzFile") -> list[str]:
    """Length/aggregate self-checks before writing; any problem aborts the run."""
    problems: list[str] = []
    total = 0
    for camera in document["cameras"]:
        index = camera["index"]
        pixels = int(camera["width"]) * int(camera["height"])
        total += pixels
        if len(camera["depth_bits"]) != pixels:
            problems.append(f"cam{index}: depth_bits {len(camera['depth_bits'])} != {pixels}")
        if len(camera["seg_pairs"]) != 2 * pixels:
            problems.append(f"cam{index}: seg_pairs {len(camera['seg_pairs'])} != {2 * pixels}")
        zeros = sum(1 for value in camera["depth_bits"] if value == 0)
        if zeros != camera["depth_zero_pixels"]:
            problems.append(f"cam{index}: depth_zero_pixels {camera['depth_zero_pixels']} != {zeros}")
        positives = [value for value in camera["depth_bits"] if value != 0]
        expected_min = f"0x{min(positives):08x}" if positives else None
        if expected_min != camera["depth_min_positive_hex"]:
            problems.append(f"cam{index}: depth_min_positive_hex mismatch")
        expected_max = f"0x{max(camera['depth_bits']):08x}"
        if expected_max != camera["depth_max_hex"]:
            problems.append(f"cam{index}: depth_max_hex mismatch")
        # npz decode must equal the recorded arrays bit for bit.
        depth_value = np.asarray(depth[f"depth_cam{index}"], dtype=np.float32)
        seg_value = np.asarray(seg[f"seg_cam{index}"], dtype=np.int32)
        if depth_bits(depth_value) != camera["depth_bits"]:
            problems.append(f"cam{index}: depth_bits != decoded npz")
        if seg_pairs(seg_value) != camera["seg_pairs"]:
            problems.append(f"cam{index}: seg_pairs != decoded npz")
    if total != document["resolved"]["total_rays"]:
        problems.append(f"total pixels {total} != total_rays {document['resolved']['total_rays']}")
    return problems


def main() -> int:
    for name, expected in EXPECTED_NPZ_SHA256.items():
        path = {"depth.npz": DEPTH_NPZ, "seg.npz": SEG_NPZ, "rays.npz": RAYS_NPZ}[name]
        actual = sha256(path)
        if actual != expected:
            print(f"FAIL: {name} sha256 {actual} != {expected}", file=sys.stderr)
            return 1

    context = json.loads(CONTEXT.read_text(encoding="utf-8"))
    resolved = context["resolved"]
    cam_res = [tuple(int(v) for v in item) for item in resolved["cam_res"]]
    depth_adr = [int(v) for v in resolved["depth_adr"]]
    seg_adr = [int(v) for v in resolved["seg_adr"]]
    total_rays = int(resolved["total_rays"])
    if cam_res != EXPECTED_CAM_RES:
        print(f"FAIL: cam_res {cam_res} != {EXPECTED_CAM_RES}", file=sys.stderr)
        return 1

    depth = np.load(DEPTH_NPZ)
    seg = np.load(SEG_NPZ)
    rays = np.load(RAYS_NPZ)
    model_312 = json.loads(MODEL_312.read_text(encoding="utf-8"))
    origin_checks = camera_data_checks(rays, model_312)
    cameras = []
    flat_total = 0
    for index, (width, height) in enumerate(cam_res):
        depth_value = np.asarray(depth[f"depth_cam{index}"], dtype=np.float32)
        seg_value = np.asarray(seg[f"seg_cam{index}"], dtype=np.int32)
        if depth_value.shape != (height, width) or seg_value.shape != (height, width, 2):
            print(f"FAIL: camera {index} shapes {depth_value.shape} {seg_value.shape}", file=sys.stderr)
            return 1
        pixels = width * height
        flat_total += pixels
        bits = depth_bits(depth_value)
        pairs = seg_pairs(seg_value)
        positives = [value for value in bits if value != 0]
        pairs_2d = np.reshape(pairs, (-1, 2))
        pairs_unique = np.unique(pairs_2d, axis=0)
        cameras.append(
            {
                "index": index,
                "width": width,
                "height": height,
                "pixels": pixels,
                "depth_adr": depth_adr[index],
                "seg_adr": seg_adr[index],
                "depth_zero_pixels": int(pixels - len(positives)),
                "depth_min_positive_hex": f"0x{min(positives):08x}" if positives else None,
                "depth_max_hex": f"0x{max(bits):08x}",
                "depth_bits": bits,
                "seg_pairs": pairs,
                "seg_unique": [[int(a), int(b)] for a, b in pairs_unique],
            }
        )
    if flat_total != total_rays:
        print(f"FAIL: sum camera pixels {flat_total} != total_rays {total_rays}", file=sys.stderr)
        return 1

    document = {
        "schema": 1,
        "task_id": "2026-10-25-render-bvh-s7b1-depth-seg",
        "kind": "render-bvh-s7b1-depth-seg-reference",
        "authoring_only": True,
        "product_python_dependency": False,
        "encoding": {
            "depth": "IEEE-754 binary32 bits as unsigned decimal u32, row-major (y outer, x fastest)",
            "seg": "int32 [object_id, object_type] pairs in the same pixel order (2 * pixels values)",
        },
        "source": {
            "upstream": UPSTREAM,
            "depth_npz_sha256": EXPECTED_NPZ_SHA256["depth.npz"],
            "seg_npz_sha256": EXPECTED_NPZ_SHA256["seg.npz"],
            "rays_npz_sha256": EXPECTED_NPZ_SHA256["rays.npz"],
            "context_json_sha256": sha256(CONTEXT),
            "model_312_sha256": sha256(MODEL_312),
            "mjb_315_sha256": sha256(MJB_315),
            "converter_sha256": script_sha256(),
            "generated_at_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "semantics": "raw rc.depth_data (planar depth in meters, 0 = miss, not normalized/clamped) "
            "and raw rc.seg_data ((geom_id, 5) or (-1,-1) background) captured by S0b",
            "camera_data": "camera data poses are the f32 cam_xpos/cam_xmat mjwarp.forward produced in the "
            "same pinned 3.15 environment (camera 1 is a TARGETBODY camera: xmat is a tracking look-at, "
            "not quat_to_mat(cam_quat)); only /quat_to_mat|look_at/ f64 checks are computed here",
        },
        "resolved": {
            "total_rays": total_rays,
            "cam_res": [list(item) for item in cam_res],
            "depth_adr": depth_adr,
            "seg_adr": seg_adr,
        },
        "camera_data_check": origin_checks,
        "cameras": cameras,
    }
    problems = verify_document(document, depth, seg)
    if problems:
        print("FAIL: " + "; ".join(problems), file=sys.stderr)
        return 1
    text = json.dumps(document, ensure_ascii=False, separators=(",", ":")) + "\n"
    # Round-trip: parse the exact bytes we are about to write and compare arrays.
    parsed = json.loads(text)
    for camera in parsed["cameras"]:
        index = camera["index"]
        if camera["depth_bits"] != depth_bits(np.asarray(depth[f"depth_cam{index}"], dtype=np.float32)):
            print(f"FAIL: round-trip depth cam{index}", file=sys.stderr)
            return 1
        if camera["seg_pairs"] != seg_pairs(np.asarray(seg[f"seg_cam{index}"], dtype=np.int32)):
            print(f"FAIL: round-trip seg cam{index}", file=sys.stderr)
            return 1
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_bytes(text.encode("utf-8"))
    written = OUTPUT.read_bytes()
    if hashlib.sha256(written).hexdigest() != hashlib.sha256(text.encode("utf-8")).hexdigest():
        print("FAIL: written bytes differ from serialized text", file=sys.stderr)
        return 1
    print(
        f"wrote {OUTPUT.relative_to(ROOT)}: {len(written)} bytes sha256={hashlib.sha256(written).hexdigest()}, "
        f"cameras={[(c['width'], c['height']) for c in parsed['cameras']]}, total_rays={total_rays}, "
        f"depth_bits_total={sum(len(c['depth_bits']) for c in parsed['cameras'])}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
