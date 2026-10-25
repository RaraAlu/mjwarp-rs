#!/usr/bin/env python
"""G0b：审计 d04 run-5 既有多接触/GJK 直达参考（纯 CPU，只审计不重采）。

输入：docs/evidence/d04-complex-physics/authoring-runs/run-5/generated/cases/*.json
输出：fixtures/g10-p5/existing-reference-audit.json

审计内容：
  - 每个 collision case 的 count / 每 world 接触数 / geom 对 / dist 范围 / fields 完整性；
  - 判定它覆盖 G2（GJK/EPA）的哪些能力（映射表写死，见 COVERAGE）；
  - 给出「可复用 / 仅回归 / 不可作 G2 判据」结论与真实缺口。
"""

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
CASES_DIR = ROOT / "docs/evidence/d04-complex-physics/authoring-runs/run-5/generated/cases"
OUT = ROOT / "fixtures/g10-p5/existing-reference-audit.json"

# 既有 case -> G2/G4 能力映射（人工判据，写进证据供 G2 验收引用）
COVERAGE = {
    "box-box-multiccd": {
        "covers": ["support(box)", "gjk_lerp", "epa", "multicontact(box-box)", "is_discrete"],
        "reusable": "yes",
        "role": "G2 主参考（box support/EPA/multicontact/离散分支）",
    },
    "convex-mesh": {
        "covers": ["support(mesh)", "support(box)", "gjk", "epa", "多接触(box-mesh)"],
        "reusable": "yes",
        "role": "G2 参考（mesh 支撑函数 graph/穷举路径）",
    },
    "degenerate-mesh": {
        "covers": ["mesh-mesh 退化输入", "epa 退化/重复支持点"],
        "reusable": "yes",
        "role": "G2 退化路径参考",
    },
    "sphere-sphere": {
        "covers": ["sphere-sphere 解析路径（G08 primitive，非 GJK）"],
        "reusable": "regression",
        "role": "仅 G08 回归，不作 GJK 支撑函数判据",
    },
    "hfield-box": {
        "covers": ["hfield prism-CCD", "support(polyvert)", "ccd() 两阶段", "hfield 四接触后处理"],
        "reusable": "yes",
        "role": "G4 主参考（hfield 接触/四接触选择）；G2 的间接覆盖",
    },
    "sdf-cylinder": {
        "covers": ["sdf_narrowphase", "Halton initpoints", "gradient_descent", "clearance/compute_grad"],
        "reusable": "yes",
        "role": "G3 主参考（SDF 接触）",
    },
}

GAPS = [
    "sphere-box（GJK 球 shrink + `_inflate` 浅穿透）：无既有参考",
    "capsule-box（capsule shrink/support）：无",
    "cylinder-box（cylinder support + 多接触限制）：无",
    "ellipsoid-box（ellipsoid support/normalize）：无",
    "hfield-sphere / hfield-capsule（hfield 与第二类型配对）：无（只有 hfield-box）",
    "sdf-box（SDF 与 box 的 clearance 组合）：无（只有 sdf-cylinder）",
    "非退化 mesh-mesh：无（只有 degenerate-mesh）",
    "box-box margin≠0（非 is_discrete 路径）：无",
    "SDF 距离/梯度越界探针：仅 probe-volume-gradient-outside（volume 采样层）",
]


def load_case(path: Path) -> dict:
    document = json.loads(path.read_text(encoding="utf-8"))
    output = document["output"]
    fields = output["fields"]
    world_ids = fields["worldid"]["values"]
    geom_flat = fields["geom"]["values"]
    geom_pairs = [geom_flat[i:i + 2] for i in range(0, len(geom_flat), 2)]
    dist_values = fields["dist"]["values"]
    per_world = {}
    for world in sorted(set(world_ids)):
        per_world[str(world)] = world_ids.count(world)
    unique_pairs = sorted({tuple(pair) for pair in geom_pairs})
    return {
        "case": document["name"],
        "families": document.get("families", []),
        "path": str(path.relative_to(ROOT)).replace("\\", "/"),
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "worlds": document["input"]["worlds"],
        "count": output["count"],
        "contacts_per_world": per_world,
        "geom_pairs": [list(pair) for pair in unique_pairs],
        "dist_min": round(min(dist_values), 6) if dist_values else None,
        "dist_max": round(max(dist_values), 6) if dist_values else None,
        "field_names": sorted(fields.keys()),
        "field_count": len(fields),
        "boundary_evidence": document.get("boundary_evidence"),
    }


def main() -> int:
    if not CASES_DIR.is_dir():
        raise SystemExit("缺少 %s" % CASES_DIR)
    cases = {}
    for path in sorted(CASES_DIR.glob("collision-*.json")):
        row = load_case(path)
        name = row["case"]
        coverage = COVERAGE.get(name, {"covers": [], "reusable": "no", "role": "未映射"})
        row.update(coverage)
        cases[name] = row
    report = {
        "schema": 1,
        "task_id": "2026-10-25-g10-p5-fixtures",
        "kind": "g0b-existing-reference-audit",
        "note": "只审计 d04 run-5 既有参考，不重采；映射表为 G2/G3/G4 验收引用。",
        "cases": cases,
        "reusable_for_g2": sorted(k for k, v in cases.items() if v.get("reusable") == "yes"),
        "regression_only": sorted(k for k, v in cases.items() if v.get("reusable") == "regression"),
        "gaps": GAPS,
        "conclusion": "既有 box-box/convex-mesh/degenerate-mesh/hfield-box/sdf-cylinder 可复用；"
                      "真实缺口为上列 9 项，由 G0b 8-9 个新 XML + worlds=2 + offset 矩阵补采。",
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(report, ensure_ascii=False, indent=1), encoding="utf-8")
    for name, row in cases.items():
        print("%-34s count=%-3d per_world=%s pairs=%s reusable=%s"
              % (name, row["count"], row["contacts_per_world"], row["geom_pairs"], row.get("reusable")))
    print("gaps:", len(GAPS))
    print("wrote", OUT.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    sys.exit(main())
