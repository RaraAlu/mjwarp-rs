"""G07@P5 离线 authoring：冻结上游宽相（NXN / SAP_TILE / SAP_SEGMENTED）参考。

离线专用：本脚本只被 scripts/authoring/author-g07-broadphase.ps1 调用。
产品构建 / 测试 / 门禁 / 验收脚本都不启动 Python。

产出：
  fixtures/broadphase/broadphase-cases.json
  docs/development-evidence/backlog-batch2/2026-10-11-roadmap-g07-p5/authoring.json
"""

from __future__ import annotations

import hashlib
import json
import platform
import subprocess
import sys
from pathlib import Path

import numpy as np
import warp as wp
from mujoco_warp._src import collision_driver

ROOT = Path(__file__).resolve().parents[2]
FIXTURE_DIR = ROOT / "fixtures" / "broadphase"
FIXTURE_JSON = FIXTURE_DIR / "broadphase-cases.json"
EVIDENCE_DIR = (
    ROOT
    / "docs"
    / "development-evidence"
    / "backlog-batch2"
    / "2026-10-11-roadmap-g07-p5"
)
EVIDENCE_JSON = EVIDENCE_DIR / "authoring.json"
UPSTREAM = ROOT / "target" / "mjwarp-parallel" / "upstream"
PINNED_REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
TASK_ID = "2026-10-11-roadmap-g07-p5"

NXN = 0
SAP_TILE = 1
SAP_SEGMENTED = 2
PLANE = 1
SPHERE = 2
AABB = 4
OBB = 8
ALL_FILTERS = PLANE | SPHERE | AABB | OBB
CAPACITY = 4096
NJMAX = 20000

MIXED_KEYS = ["rest", "near", "cluster", "order"]
FILTER_KEYS = ["apart", "one_touch", "near", "rotated"]
MARGIN_CASES = [
    ("margin-0-0", 0.0, 0.0),
    ("margin-0-0011", 0.0, 0.011),
    ("margin-0011-0", 0.011, 0.0),
    ("margin-000999-0", 0.00999, 0.0),
    ("margin-0-000999", 0.0, 0.00999),
    ("margin-000999-000999", 0.00999, 0.00999),
]


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    digest.update(path.read_bytes())
    return digest.hexdigest()


def f32_list(values) -> list[float]:
    return [float(np.float32(value)) for value in np.asarray(values).reshape(-1)]


def host_arrays(host) -> dict:
    return {"ngeom": int(host.ngeom), "nbody": int(host.nbody), "nq": int(host.nq)}


def model_arrays(model) -> dict:
    dataid = np.asarray(model.geom_dataid.numpy())
    aabb = np.asarray(model.geom_aabb.numpy())
    rbound = np.asarray(model.geom_rbound.numpy())
    margin = np.asarray(model.geom_margin.numpy())
    gap = np.asarray(model.geom_gap.numpy())
    pairs = np.asarray(model.nxn_geom_pair_filtered.numpy())
    pairids = np.asarray(model.nxn_pairid_filtered.numpy())
    full_pairid = np.asarray(model.nxn_pairid.numpy())
    centers = []
    sizes = []
    for batch in range(aabb.shape[0]):
        for geom in range(aabb.shape[1]):
            centers.append(f32_list(aabb[batch, geom, 0]))
            sizes.append(f32_list(aabb[batch, geom, 1]))
    return {
        "ngeom": int(model.ngeom),
        "geom_type": [int(value) for value in np.asarray(model.geom_type.numpy()).reshape(-1)],
        "geom_bodyid": [
            int(value) for value in np.asarray(model.geom_bodyid.numpy()).reshape(-1)
        ],
        "dataid_batches": int(dataid.shape[0]),
        "geom_dataid": [int(value) for value in dataid.reshape(-1)],
        "aabb_batches": int(aabb.shape[0]),
        "geom_aabb_center": centers,
        "geom_aabb_size": sizes,
        "rbound_batches": int(rbound.shape[0]),
        "geom_rbound": f32_list(rbound),
        "margin_batches": int(margin.shape[0]),
        "geom_margin": f32_list(margin),
        "gap_batches": int(gap.shape[0]),
        "geom_gap": f32_list(gap),
        "pairs": [[int(v) for v in row] for row in pairs],
        "pairids": [[int(v) for v in row] for row in pairids],
        "full_pairid": [[int(v) for v in row] for row in full_pairid],
    }


def run_broadphase(host, model, worlds, mode, filter_bits, naconmax):
    import mujoco_warp as mjw

    data = mjw.make_data(host, nworld=len(worlds), naconmax=naconmax, njmax=NJMAX)
    qpos = np.asarray(worlds, dtype=np.float32)
    data.qpos.assign(qpos)
    data.qvel.assign(np.zeros((qpos.shape[0], host.nv), dtype=np.float32))
    mjw.kinematics(model, data)
    wp.synchronize_device()
    xpos = np.asarray(data.geom_xpos.numpy(), dtype=np.float32)
    xmat = np.asarray(data.geom_xmat.numpy(), dtype=np.float32)
    ctx = collision_driver.create_collision_context(data.naconmax)
    model.opt.broadphase = mode
    model.opt.broadphase_filter = filter_bits
    ctx.collision_pair.fill_(wp.vec2i(-1, -1))
    ctx.collision_pairid.fill_(wp.vec2i(-1, -1))
    ctx.collision_worldid.fill_(-1)
    data.ncollision.zero_()
    if mode == NXN:
        collision_driver.nxn_broadphase(model, data, ctx)
    else:
        collision_driver.sap_broadphase(model, data, ctx)
    wp.synchronize_device()
    ncollision = int(np.asarray(data.ncollision.numpy()).reshape(-1)[0])
    written = min(ncollision, int(data.naconmax))
    pairs = np.asarray(ctx.collision_pair.numpy())[:written]
    pairids = np.asarray(ctx.collision_pairid.numpy())[:written]
    worldids = np.asarray(ctx.collision_worldid.numpy())[:written]
    output = {
        "ncollision": ncollision,
        "capacity": int(data.naconmax),
        "pairs": [[int(v) for v in row] for row in pairs],
        "pairids": [[int(v) for v in row] for row in pairids],
        "worldids": [int(v) for v in worldids],
    }
    return output, xpos, xmat


def native_contacts(host, worlds):
    import mujoco

    per_world = []
    counts = []
    for world in worlds:
        data = mujoco.MjData(host)
        data.qpos[:] = np.asarray(world, dtype=np.float64)
        mujoco.mj_kinematics(host, data)
        mujoco.mj_collision(host, data)
        pairs = [
            [int(data.contact[i].geom1), int(data.contact[i].geom2)]
            for i in range(int(data.ncon))
        ]
        per_world.append(pairs)
        counts.append(int(data.ncon))
    return per_world, counts


def contact_subset(candidates, worldids, native_pairs) -> dict:
    by_world: dict[int, set[tuple[int, int]]] = {}
    for pair, world in zip(candidates, worldids):
        by_world.setdefault(world, set()).add((min(pair), max(pair)))
    missing = []
    for world, pairs in enumerate(native_pairs):
        known = by_world.get(world, set())
        for pair in pairs:
            key = (min(pair), max(pair))
            if key not in known:
                missing.append([world, list(key)])
    total = sum(len(pairs) for pairs in native_pairs)
    return {"native_contacts": total, "contacts_missing_from_candidates": missing}


def main() -> int:
    import mujoco
    import mujoco_warp as mjw

    if not (UPSTREAM / "mujoco_warp").is_dir():
        raise SystemExit(f"missing pinned upstream at {UPSTREAM}")
    head = subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=UPSTREAM, check=True, capture_output=True, text=True
    ).stdout.strip()
    if head != PINNED_REVISION:
        raise SystemExit(f"upstream HEAD {head} != pinned {PINNED_REVISION}")

    sources = {
        "mixed": FIXTURE_DIR / "mixed.xml",
        "filters": FIXTURE_DIR / "filters.xml",
        "margin": FIXTURE_DIR / "margin.xml",
        "mesh": FIXTURE_DIR / "mesh.xml",
    }
    host_models: dict[str, object] = {}
    models: dict[str, dict] = {}
    model_info: dict[str, dict] = {}
    keyframes: dict[str, list[np.ndarray]] = {}
    for name, path in sources.items():
        host = mujoco.MjModel.from_xml_path(str(path))
        host_models[name] = host
        if name != "margin":
            keyframes[name] = [
                np.asarray(host.key_qpos[index], dtype=np.float32)
                for index in range(host.nkey)
            ]
        model = mjw.put_model(host)
        models[name] = {**model_arrays(model), "host": host_arrays(host)}
        model_info[name] = {
            "name": name,
            "path": path.relative_to(ROOT).as_posix(),
            "sha256": sha256(path),
        }
        print(f"model {name}: ngeom={host.ngeom} nkey={host.nkey}")

    for label, margin_left, margin_right in MARGIN_CASES:
        host = mujoco.MjModel.from_xml_path(str(sources["margin"]))
        host.geom_margin[0] = np.float64(margin_left)
        host.geom_margin[1] = np.float64(margin_right)
        keyframes[label] = [np.asarray(host.key_qpos[0], dtype=np.float32)]
        model = mjw.put_model(host)
        models[label] = {
            **model_arrays(model),
            "host": host_arrays(host),
            "margins": f32_list(host.geom_margin),
        }
        model_info[label] = {
            "name": label,
            "path": sources["margin"].relative_to(ROOT).as_posix(),
            "sha256": sha256(sources["margin"]),
            "geom_margin": [float(margin_left), float(margin_right)],
        }
        host_models[label] = host

    cases: list[dict] = []

    def add_case(
        name,
        model_name,
        worlds,
        mode,
        filter_bits,
        naconmax=CAPACITY,
        dataid_override=None,
        full_reference=None,
    ):
        host = host_models[model_name]
        model = mjw.put_model(host)
        if dataid_override is not None:
            dataid = np.asarray(dataid_override, dtype=np.int32)
            model.geom_dataid = wp.array(dataid, dtype=int)
        output, xpos, xmat = run_broadphase(
            host, model, worlds, mode, filter_bits, naconmax
        )
        case = {
            "name": name,
            "model": model_name,
            "nworld": len(worlds),
            "worlds": [
                {
                    "qpos": f32_list(world),
                    "geom_xpos": f32_list(xpos[index]),
                    "geom_xmat": f32_list(xmat[index]),
                }
                for index, world in enumerate(worlds)
            ],
            "options": {
                "broadphase": mode,
                "broadphase_filter": filter_bits,
                "naconmax": naconmax,
            },
            "output": output,
        }
        if dataid_override is not None:
            dataid = np.asarray(dataid_override)
            case["options"]["dataid_batches"] = int(dataid.shape[0])
            case["options"]["geom_dataid"] = [
                int(value) for value in dataid.reshape(-1)
            ]
        elif filter_bits == ALL_FILTERS and naconmax != CAPACITY:
            # 容量截断案例：只写出 naconmax 条候选，无法用原生接触对做包含性判定。
            pass
        elif filter_bits == ALL_FILTERS:
            native_pairs, native_counts = native_contacts(host, worlds)
            check = contact_subset(output["pairs"], output["worldids"], native_pairs)
            check["native_ncon"] = native_counts
            check["native_pairs"] = native_pairs
            case["native"] = check
        if full_reference is not None:
            case["full_reference"] = full_reference
        cases.append(case)
        native_counts = case.get("native", {}).get("native_ncon")
        print(
            f"case {name}: ncollision={output['ncollision']}"
            f" written={len(output['pairs'])} native_ncon={native_counts}"
        )
        return case

    for key_index, key_name in enumerate(MIXED_KEYS):
        for mode_name, mode in (
            ("nxn", NXN),
            ("sap-tile", SAP_TILE),
            ("sap-segmented", SAP_SEGMENTED),
        ):
            add_case(
                f"mixed-{key_name}-{mode_name}",
                "mixed",
                [keyframes["mixed"][key_index]],
                mode,
                ALL_FILTERS,
            )

    for mode_name, mode in (("nxn", NXN), ("sap-tile", SAP_TILE)):
        add_case(
            f"mixed-two-world-{mode_name}",
            "mixed",
            [keyframes["mixed"][0], keyframes["mixed"][2]],
            mode,
            ALL_FILTERS,
        )

    filter_sets = (
        ("plane", PLANE),
        ("plane-sphere", PLANE | SPHERE),
        ("plane-aabb", PLANE | AABB),
        ("plane-obb", PLANE | OBB),
        ("all", ALL_FILTERS),
    )
    for key_name in FILTER_KEYS:
        key_index = FILTER_KEYS.index(key_name)
        for filter_name, filter_bits in filter_sets:
            add_case(
                f"filters-{key_name}-{filter_name}",
                "filters",
                [keyframes["filters"][key_index]],
                NXN,
                filter_bits,
            )
    for mode_name, mode in (("sap-tile", SAP_TILE), ("sap-segmented", SAP_SEGMENTED)):
        add_case(
            f"filters-near-{mode_name}",
            "filters",
            [keyframes["filters"][2]],
            mode,
            ALL_FILTERS,
        )

    add_case(
        "mixed-cluster-capacity-full",
        "mixed",
        [keyframes["mixed"][2]],
        NXN,
        ALL_FILTERS,
    )
    add_case(
        "mixed-cluster-capacity-one-nxn",
        "mixed",
        [keyframes["mixed"][2]],
        NXN,
        ALL_FILTERS,
        naconmax=1,
        full_reference="mixed-cluster-capacity-full",
    )
    add_case(
        "mixed-cluster-capacity-one-sap-tile",
        "mixed",
        [keyframes["mixed"][2]],
        SAP_TILE,
        ALL_FILTERS,
        naconmax=1,
        full_reference="mixed-cluster-capacity-full",
    )

    mesh_host = host_models["mesh"]
    mesh_qpos0 = np.asarray(mesh_host.qpos0, dtype=np.float32)
    mesh_worlds = [mesh_qpos0.copy(), mesh_qpos0.copy()]
    mesh_worlds[1][2] = -2.0
    base_dataid = np.asarray(mjw.put_model(mesh_host).geom_dataid.numpy()[0], dtype=np.int32)
    dataid = np.tile(base_dataid, (2, 1))
    dataid[0, 1] = -1
    for mode_name, mode in (("nxn", NXN), ("sap-segmented", SAP_SEGMENTED)):
        add_case(
            f"mesh-missing-{mode_name}",
            "mesh",
            mesh_worlds,
            mode,
            ALL_FILTERS,
            dataid_override=dataid,
        )

    for label, _margin_left, _margin_right in MARGIN_CASES:
        add_case(
            f"margin-{label}",
            label,
            [keyframes[label][0]],
            NXN,
            ALL_FILTERS,
        )

    gpu = subprocess.run(
        ["nvidia-smi", "--query-gpu=name,driver_version", "--format=csv,noheader"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()

    fixture = {
        "task_id": TASK_ID,
        "matrix": "G07",
        "phase": "P5",
        "upstream_functions": ["nxn_broadphase", "sap_broadphase"],
        "upstream_reference": {
            "revision": PINNED_REVISION,
            "warp": wp.__version__,
            "mujoco_warp": "3.15.0",
            "mujoco": mujoco.__version__,
            "note": "冻结上游 NXN/SAP 宽相直接产出候选对；输出顺序由原子计数决定，比较按集合规范化。",
        },
        "enum": {
            "BroadphaseType": {
                "NXN": NXN,
                "SAP_TILE": SAP_TILE,
                "SAP_SEGMENTED": SAP_SEGMENTED,
            },
            "BroadphaseFilter": {"PLANE": PLANE, "SPHERE": SPHERE, "AABB": AABB, "OBB": OBB},
            "MJ_MAXVAL": float(mujoco.mjMAXVAL),
        },
        "sources": model_info,
        "models": models,
        "cases": cases,
        "environment": {"python": platform.python_version(), "gpu": gpu},
    }
    FIXTURE_JSON.write_text(
        json.dumps(fixture, ensure_ascii=False, indent=1) + "\n", encoding="utf-8"
    )

    native_total = 0
    native_missing = 0
    margin_match = 0
    margin_cases = 0
    for case in cases:
        native = case.get("native")
        if not native:
            continue
        native_total += native["native_contacts"]
        native_missing += len(native["contacts_missing_from_candidates"])
        if case["name"].startswith("margin-"):
            margin_cases += 1
            if native["native_ncon"][0] == case["output"]["ncollision"]:
                margin_match += 1
    EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)
    evidence = {
        "task_id": TASK_ID,
        "generated_by": "scripts/authoring/author-g07-broadphase.ps1",
        "models": model_info,
        "cases": len(cases),
        "capacity_cases": [case["name"] for case in cases if "full_reference" in case],
        "native_cross_check": {
            "cases_with_native": sum(1 for case in cases if "native" in case),
            "native_contacts": native_total,
            "contacts_missing_from_candidates": native_missing,
            "margin_cases": margin_cases,
            "margin_candidate_equals_native_ncon": margin_match,
            "note": "原生 MuJoCo 3.15 f64 mj_kinematics+mj_collision：接触对必须全部包含在候选集中；边距案例候选计数与原生接触计数逐案相等。",
        },
        "environment": {"python": platform.python_version(), "gpu": gpu},
        "outputs": ["fixtures/broadphase/broadphase-cases.json"],
    }
    EVIDENCE_JSON.write_text(
        json.dumps(evidence, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(
        "g07-p5 broadphase authoring OK:"
        f" models={len(models)} cases={len(cases)}"
        f" native_contacts={native_total} missing={native_missing}"
        f" margin_match={margin_match}/{margin_cases}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
