"""G27@P2 离线 authoring：固定常量 set_const_fixed 的冻结参考。

离线专用：本脚本只被 scripts/authoring/author-g27-set-const.ps1 调用，
产品构建/测试/门禁/验收脚本均不启动 Python。

产出：
  fixtures/set-const/g27-fixed.json
  docs/development-evidence/backlog-batch1/2026-10-11-roadmap-g27-p2/authoring.json
"""

from __future__ import annotations

import hashlib
import json
import platform
import subprocess
import sys
from pathlib import Path

import numpy as np

ROOT = Path(__file__).resolve().parents[2]
FIXTURE_DIR = ROOT / "fixtures" / "set-const"
MODEL_XML = FIXTURE_DIR / "model.xml"
FIXTURE_JSON = FIXTURE_DIR / "g27-fixed.json"
EVIDENCE_DIR = (
    ROOT
    / "docs"
    / "development-evidence"
    / "backlog-batch1"
    / "2026-10-11-roadmap-g27-p2"
)
EVIDENCE_JSON = EVIDENCE_DIR / "authoring.json"
UPSTREAM = ROOT / "target" / "mjwarp-parallel" / "upstream"
PINNED_REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
RUNS = 5


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    digest.update(path.read_bytes())
    return digest.hexdigest()


def f32_list(values) -> list[float]:
    """量化到 f32，并以可被 Rust 精确 round-trip 的十进制写出。"""
    return [float(np.float32(value)) for value in values]


def main() -> int:
    import mujoco
    import mujoco_warp
    import warp

    mjm = mujoco.MjModel.from_xml_path(str(MODEL_XML))
    mjd = mujoco.MjData(mjm)

    # 输入量化规则：原生 f64 -> f32（产品输入为 f32）。
    body_mass_f32 = f32_list(mjm.body_mass)
    mjm.body_mass[:] = body_mass_f32
    mujoco.mj_setConst(mjm, mjd)
    native_subtree = [float(value) for value in mjm.body_subtreemass]
    parent = [int(value) for value in mjm.body_parentid]

    # 独立复算：子节点质量向父节点累加（f64，仅作诊断，不反馈产品）。
    check = np.array(body_mass_f32, dtype=np.float64)
    depth = compute_depth(parent)
    for body in sorted(range(len(parent)), key=lambda b: -depth[b]):
        if body != 0:
            check[parent[body]] += check[body]
    native_check_max_abs = float(np.max(np.abs(check - np.array(native_subtree))))

    # 冻结上游 MJWarp：设备侧 set_const_fixed，多次运行观察原子累加位型。
    wp_model = mujoco_warp.put_model(mjm)
    wp_data = mujoco_warp.make_data(mjm)
    runs: list[list[float]] = []
    for _ in range(RUNS):
        mujoco_warp.set_const_fixed(wp_model, wp_data)
        values = wp_model.body_subtreemass.numpy()[0]
        runs.append([float(value) for value in values])

    run_bits = {tuple(np.asarray(run, dtype=np.float32).view(np.uint32)) for run in runs}
    spread = float(
        np.max(np.abs(np.asarray(runs, dtype=np.float64) - np.asarray(runs[0], dtype=np.float64)))
    )
    upstream_vs_native = float(
        np.max(np.abs(np.asarray(runs[0], dtype=np.float64) - np.asarray(native_subtree)))
    )

    upstream_head = subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=UPSTREAM, check=True, capture_output=True, text=True
    ).stdout.strip()
    if upstream_head != PINNED_REVISION:
        raise SystemExit(f"upstream HEAD {upstream_head} != pinned {PINNED_REVISION}")
    gpu = subprocess.run(
        ["nvidia-smi", "--query-gpu=name,driver_version", "--format=csv,noheader"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()

    fixture = {
        "task_id": "2026-10-11-roadmap-g27-p2",
        "matrix": "G27",
        "phase": "P2",
        "upstream_function": "set_const_fixed",
        "model": {
            "path": "fixtures/set-const/model.xml",
            "sha256": sha256(MODEL_XML),
            "nbody": len(parent),
            "nq": int(mjm.nq),
            "nv": int(mjm.nv),
        },
        "input": {
            "conversion": "native f64 -> f32",
            "body_mass": body_mass_f32,
            "body_parentid": parent,
        },
        "native_reference": {
            "mujoco": mujoco.__version__,
            "body_subtreemass_f64": native_subtree,
            "independent_f64_recompute_max_abs": native_check_max_abs,
        },
        "upstream_reference": {
            "revision": PINNED_REVISION,
            "warp": warp.__version__,
            "mujoco_warp": getattr(mujoco_warp, "__version__", "3.15.0"),
            "function": "mujoco_warp.set_const_fixed",
            "runs": runs,
            "distinct_bit_patterns": len(run_bits),
            "max_abs_run_spread": spread,
            "max_abs_vs_native": upstream_vs_native,
        },
        "environment": {
            "python": platform.python_version(),
            "gpu": gpu,
        },
        "tolerance": {
            "native_abs": 1e-06,
            "upstream_abs": 1e-06,
            "note": "固定常量规模 O(1e1)；f32 累加顺序差异只允许 1e-6 绝对误差，不放宽。",
        },
    }

    FIXTURE_JSON.parent.mkdir(parents=True, exist_ok=True)
    FIXTURE_JSON.write_text(
        json.dumps(fixture, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )

    EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)
    evidence = {
        "task_id": fixture["task_id"],
        "generated_by": "scripts/authoring/author-g27-set-const.ps1",
        "model_sha256": fixture["model"]["sha256"],
        "nbody": fixture["model"]["nbody"],
        "runs": RUNS,
        "distinct_bit_patterns": len(run_bits),
        "max_abs_run_spread": spread,
        "max_abs_vs_native": upstream_vs_native,
        "independent_f64_recompute_max_abs": native_check_max_abs,
        "environment": fixture["environment"],
        "tolerances": fixture["tolerance"],
        "outputs": ["fixtures/set-const/g27-fixed.json"],
    }
    EVIDENCE_JSON.write_text(
        json.dumps(evidence, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )

    print(
        "g27 set_const_fixed authoring OK:"
        f" nbody={len(parent)}"
        f" runs={RUNS}"
        f" distinct_bit_patterns={len(run_bits)}"
        f" max_run_spread={spread:.3e}"
        f" max_vs_native={upstream_vs_native:.3e}"
        f" native_recompute={native_check_max_abs:.3e}"
    )
    return 0


def compute_depth(parent: list[int]) -> list[int]:
    depth: list[int | None] = [None] * len(parent)
    depth[0] = 0
    for _ in range(len(parent)):
        changed = False
        for body, value in enumerate(depth):
            if value is not None:
                continue
            ancestor = parent[body]
            if depth[ancestor] is not None:
                depth[body] = depth[ancestor] + 1
                changed = True
        if not changed:
            break
    if any(value is None for value in depth):
        raise SystemExit("model body tree is not rooted at body 0")
    return [int(value) for value in depth]


if __name__ == "__main__":
    sys.exit(main())
