#!/usr/bin/env python
"""S8 端到端：从冻结 scene-a mjb 导出产品 camlight 输入模型字段（authoring only）。

只使用 pinned mujoco 读取静态模型字段，不初始化 warp/CUDA、不占 GPU、不跑产品代码。
输出 `fixtures/render-bvh/scene-e2e/camlight-model.json`，供产品侧
`KinematicsPlan::with_camlight` 构造（`src/physics/camlight.rs` 的 GPU 段）。

数据来源与口径：
- 输入 mjb：`fixtures/render-bvh/scene-a/scene-315.mjb`（S0b 冻结夹具，sha 记录）；
- 字段取 MuJoCo 3.15 的模型数组，double -> f32 一次转换（与 product 上传 f32 口径相同）；
- qpos0 是模型默认姿态（pinned 3.15 forward 用的初始状态）；
- 参考 data 位姿不在本脚本产生：直接复用 S7b1 已冻结的同配方相机 data 位姿夹具，
  只在本文件的 provenance 里记来源（避免二次派生漂移）。
"""

import hashlib
import json
import sys
from pathlib import Path

import mujoco

ROOT = Path(__file__).resolve().parents[3]
MJB = ROOT / "fixtures/render-bvh/scene-a/scene-315.mjb"
OUT = ROOT / "fixtures/render-bvh/scene-e2e/camlight-model.json"

TASK = "2026-10-25-render-bvh-s8-e2e"

# 产品输入字段（`KinematicFields` / `InertialFields` / `AttachedFields` / `MocapModelInput`
# / `CamLightFields`），按 (json 名字, mjModel 属性) 取。
INT_FIELDS = [
    "body_parentid",
    "body_jntadr",
    "body_jntnum",
    "jnt_type",
    "jnt_bodyid",
    "jnt_qposadr",
    "jnt_dofadr",
    "dof_bodyid",
    "dof_jntid",
    "dof_parentid",
    "geom_bodyid",
    "site_bodyid",
    "body_mocapid",
    "cam_mode",
    "cam_bodyid",
    "cam_targetbodyid",
    "light_mode",
    "light_bodyid",
    "light_targetbodyid",
]

FLOAT_FIELDS = [
    "qpos0",
    "body_pos",
    "body_quat",
    "jnt_pos",
    "jnt_axis",
    "body_ipos",
    "body_iquat",
    "body_mass",
    "body_inertia",
    "dof_armature",
    "dof_damping",
    "geom_pos",
    "geom_quat",
    "site_pos",
    "site_quat",
    "cam_pos",
    "cam_quat",
    "cam_poscom0",
    "cam_pos0",
    "cam_mat0",
    "light_pos",
    "light_dir",
    "light_poscom0",
    "light_pos0",
    "light_dir0",
]


def f32(value: float) -> float:
    """double -> f32 一次舍入后仍以 JSON 数值写出（Rust 侧 `as f32` 再取一次）。"""
    import struct

    return struct.unpack("<f", struct.pack("<f", value))[0]


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def flat(name: str, values) -> list:
    """展平多维数组；f32 字段逐元素转 f32。"""
    out: list = []

    def walk(item) -> None:
        if isinstance(item, (list, tuple)) or type(item).__name__ == "ndarray":
            for sub in item:
                walk(sub)
        else:
            out.append(f32(float(item)) if name in FLOAT_FIELDS else int(item))

    walk(values)
    return out


def main() -> int:
    model = mujoco.MjModel.from_binary_path(str(MJB))
    ints = {name: flat(name, getattr(model, name)) for name in INT_FIELDS}
    floats = {name: flat(name, getattr(model, name)) for name in FLOAT_FIELDS}

    # 长度自检：拓扑字段必须等于原生维度，否则产品侧会在构造时拒绝。
    counts = {
        "nq": int(model.nq),
        "nv": int(model.nv),
        "nbody": int(model.nbody),
        "njnt": int(model.njnt),
        "ngeom": int(model.ngeom),
        "nsite": int(model.nsite),
        "nmocap": int(model.nmocap),
        "ncam": int(model.ncam),
        "nlight": int(model.nlight),
    }
    expect = {
        "qpos0": counts["nq"],
        "body_parentid": counts["nbody"],
        "body_pos": 3 * counts["nbody"],
        "body_quat": 4 * counts["nbody"],
        "jnt_type": counts["njnt"],
        "jnt_pos": 3 * counts["njnt"],
        "body_ipos": 3 * counts["nbody"],
        "body_inertia": 3 * counts["nbody"],
        "dof_bodyid": counts["nv"],
        "geom_bodyid": counts["ngeom"],
        "geom_pos": 3 * counts["ngeom"],
        "site_bodyid": counts["nsite"],
        "body_mocapid": counts["nbody"],
        "cam_mode": counts["ncam"],
        "cam_pos": 3 * counts["ncam"],
        "cam_quat": 4 * counts["ncam"],
        "cam_poscom0": 3 * counts["ncam"],
        "cam_pos0": 3 * counts["ncam"],
        "cam_mat0": 9 * counts["ncam"],
        "light_mode": counts["nlight"],
        "light_pos": 3 * counts["nlight"],
        "light_dir": 3 * counts["nlight"],
        "light_poscom0": 3 * counts["nlight"],
        "light_pos0": 3 * counts["nlight"],
        "light_dir0": 3 * counts["nlight"],
    }
    merged = {**ints, **floats}
    for name, expected in expect.items():
        actual = len(merged[name])
        if actual != expected:
            raise SystemExit(f"{name} 长度 {actual} != {expected}")

    document = {
        "schema": 1,
        "task_id": TASK,
        "kind": "s8-camlight-model-input",
        "native_version": mujoco.__version__,
        "counts": counts,
        "model": merged,
        "provenance": {
            "mjb": {
                "path": "fixtures/render-bvh/scene-a/scene-315.mjb",
                "sha256": sha256_file(MJB),
            },
            "script": {
                "path": "scripts/authoring/render-bvh-s7b/capture_e2e_camlight.py",
                "sha256": sha256_file(Path(__file__)),
            },
            "python": sys.version.split()[0],
            "mujoco": mujoco.__version__,
            "float_contract": "mjModel double -> f32 once per element (product upload dtype)",
            "camera_data_reference": {
                "note": "参考 data 位姿不在此重派生；S8 直接读 S7b1 已冻结的同配方夹具",
                "sources": [
                    "fixtures/render-bvh/scene-a/s7b/depth_seg_fast_math_false.json",
                ],
            },
            "authoring_only": True,
            "reads_product_tree": False,
            "product_python_dependency": False,
        },
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(document, ensure_ascii=False, indent=1), encoding="utf-8")
    print(f"wrote {OUT.relative_to(ROOT)} bytes={OUT.stat().st_size}")
    print(f"sha256 {sha256_file(OUT)}")
    print(f"counts {counts}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
