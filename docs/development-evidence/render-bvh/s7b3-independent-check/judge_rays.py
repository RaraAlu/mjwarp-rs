"""QA judge: AA ray layout independent re-derivation (subsample order / offsets).

Reconstructs each ray's sub-pixel coordinate from its dumped f32 direction
(`dir = normalize(x, y, -znear)` => x = -znear * dir.x/dir.z) and inverts the camera mapping to
recover (px, py). Then checks the recovered coordinates against the mapping QA derives itself
from the S5/`build_rays.cu` contract:

  base (spp=1): index = [camera][row][column], px = x, py = y, img = (w, h)
  aa2  (spp=2): index = [sample][camera][row][column],
                px = x*spp + sample%spp, py = y*spp + sample//spp, img = (w*spp, h*spp)

This does not depend on the delivered tests or on warp transcendentals: it only needs the
camera intrinsics and znear, both frozen fixture data.
"""

from __future__ import annotations

import json
import pathlib

import numpy as np

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
OUT = ROOT / "target/qa-s7b3/out"
MODEL = ROOT / "fixtures/render-bvh/scene-a/model-312.json"

ZNEAR = np.float32(0.024999999441206455)


def camera_boxes(model: dict):
    arrays = model["arrays"]
    fovy = [np.float32(v) for v in arrays["cam_fovy"]["values"]]
    sensorsize = arrays["cam_sensorsize"]["values"]
    intrinsic = arrays["cam_intrinsic"]["values"]
    projection = [int(v) for v in arrays["cam_projection"]["values"]]
    boxes = []
    for index in range(len(fovy)):
        sensor_x = np.float32(sensorsize[2 * index])
        sensor_y = np.float32(sensorsize[2 * index + 1])
        boxes.append(
            {
                "projection": projection[index],
                "fovy": fovy[index],
                "sensor_x": sensor_x,
                "sensor_y": sensor_y,
                "intrinsic": [np.float32(v) for v in intrinsic[4 * index : 4 * index + 4]],
            }
        )
    return boxes


def recover_pixels(directions: np.ndarray, box: dict, img_w: int, img_h: int) -> tuple[np.ndarray, np.ndarray]:
    dirs = np.asarray(directions, dtype=np.float64)
    znear = float(ZNEAR)
    aspect = img_w / img_h
    if box["projection"] == 1:
        raise SystemExit("orthographic camera not used in scene-a")
    sensor_h = float(box["sensor_y"])
    if sensor_h != 0.0:
        fx, fy, cx, cy = (float(v) for v in box["intrinsic"])
        sensor_w = float(box["sensor_x"])
        sensor_aspect = np.float64(np.float32(sensor_w / sensor_h))
        if aspect > sensor_aspect:
            sensor_h = np.float64(np.float32(sensor_w / aspect))
        elif aspect < sensor_aspect:
            sensor_w = np.float64(np.float32(sensor_h * aspect))
        inv_fx_znear = np.float64(np.float32(znear / fx))
        inv_fy_znear = np.float64(np.float32(znear / fy))
        half_sensor_w = 0.5 * sensor_w
        half_sensor_h = 0.5 * sensor_h
        left = -inv_fx_znear * (half_sensor_w - cx)
        right = inv_fx_znear * (half_sensor_w + cx)
        top = inv_fy_znear * (half_sensor_h - cy)
        bottom = -inv_fy_znear * (half_sensor_h + cy)
    else:
        # fovy branch: half_height = znear * tan(0.5 * fovy * DEG_TO_RAD)
        fovy_rad = float(np.float32(float(box["fovy"]) * 0.017453292519943295))
        half_height = float(np.float32(znear * np.tan(0.5 * fovy_rad)))
        half_width = float(np.float32(half_height * aspect))
        left = -half_width
        right = half_width
        top = half_height
        bottom = -half_height
    x = -znear * dirs[:, 0] / dirs[:, 2]
    y = -znear * dirs[:, 1] / dirs[:, 2]
    u = (x - left) / (right - left)
    v = (y - top) / (bottom - top)
    px = u * img_w - 0.5
    py = v * img_h - 0.5
    return px, py


def read_rays(name: str) -> tuple[np.ndarray, np.ndarray]:
    data = np.frombuffer((OUT / f"rays_{name}.bin").read_bytes(), dtype=np.float32).reshape(-1, 6)
    return data[:, 0:3], data[:, 3:6]


def check(tag: str, spp: int) -> dict:
    model = json.loads(MODEL.read_text(encoding="utf-8"))
    boxes = camera_boxes(model)
    report = json.loads((OUT / f"harness_{tag}.json").read_text(encoding="utf-8"))
    layout = report["layout"]
    cam_res = layout["cam_res"]
    directions, offsets = read_rays(tag)
    total = layout["total_rays"]
    nsamples = layout["nsamples"]
    result = {
        "tag": tag,
        "spp": spp,
        "nsamples": nsamples,
        "rays": int(directions.shape[0]),
        "expected_rays": total * nsamples,
        "offsets_all_zero": bool(np.all(offsets == 0.0)),
        "cameras": [],
    }
    camera_offset = 0
    for position, (width, height) in enumerate(cam_res):
        width, height = int(width), int(height)
        pixels = width * height
        img_w, img_h = width * spp, height * spp
        for sample in range(nsamples):
            start = sample * total + camera_offset
            camera_dirs = directions[start : start + pixels]
            px, py = recover_pixels(camera_dirs, boxes[position], img_w, img_h)
            yid = np.repeat(np.arange(height), width)
            xid = np.tile(np.arange(width), height)
            expected_px = xid * spp + (sample % spp)
            expected_py = yid * spp + (sample // spp)
            px_bad = int(np.count_nonzero(np.abs(px - expected_px) > 1.0e-2))
            py_bad = int(np.count_nonzero(np.abs(py - expected_py) > 1.0e-2))
            result["cameras"].append(
                {
                    "camera": position,
                    "sample": sample,
                    "pixels": pixels,
                    "img": [img_w, img_h],
                    "px_bad": px_bad,
                    "py_bad": py_bad,
                    "px_max_err": float(np.abs(px - expected_px).max()),
                    "py_max_err": float(np.abs(py - expected_py).max()),
                }
            )
        camera_offset += pixels
    result["layout_ok"] = all(cam["px_bad"] == 0 and cam["py_bad"] == 0 for cam in result["cameras"])
    return result


def main() -> int:
    report = {
        "base": check("base", 1),
        "aa2": check("aa2", 2),
    }
    # AA ray count equals nsamples * total_rays
    report["aa_ray_count_ok"] = (
        report["aa2"]["rays"] == report["aa2"]["expected_rays"] == 480000
        and report["aa2"]["nsamples"] == 4
    )
    out = OUT / "judge_rays.json"
    out.write_text(json.dumps(report, indent=1), encoding="utf-8")
    print(json.dumps(report, indent=1))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
