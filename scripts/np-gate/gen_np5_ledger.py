# NOT in the 93 guards inputs; adding it to guards requires a user re-sign (NP5-00).
# -*- coding: utf-8 -*-
"""NP5-00 ledger draft generator (draft only, writes under target/).
Reads committed NP01/NP02 evidence lists + CUDA surface snapshot and emits
np-gate-ledger.json. Read-only w.r.t. repo; no network; no GPU.
"""
import json, hashlib, pathlib, subprocess, datetime, sys

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
OUT_DIR = ROOT / "target/render-bvh/gl2-ptx-diff"
NP01 = ROOT / "docs/development-evidence/np01-type-mapping"
NP02 = ROOT / "docs/development-evidence/np02-evidence"


def sha(p):
    return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()


def nsha(p):
    # text-normalized (LF) hash for product sources, same convention as NP02 tool
    b = pathlib.Path(p).read_bytes().replace(b"\r\n", b"\n")
    return hashlib.sha256(b).hexdigest()


def jload(p):
    return json.loads(pathlib.Path(p).read_text(encoding="utf-8"))


def head():
    r = subprocess.run(["git", "-C", str(ROOT), "log", "-1", "--format=%H%n%s"],
                       capture_output=True, text=True, encoding="utf-8")
    a = r.stdout.strip().splitlines()
    return a[0], (a[1] if len(a) > 1 else "")


def commit_of(path):
    r = subprocess.run(["git", "-C", str(ROOT), "log", "-1", "--format=%h %ai", "--", path],
                       capture_output=True, text=True, encoding="utf-8")
    return r.stdout.strip()


inv1 = jload(NP01 / "inventory.json")
inv2 = jload(NP02 / "inventory.json")
gap = jload(NP02 / "gap.json")
surf = jload(NP01 / "cuda-surface.json")

# ---------------- NP01 30 paths + field tables ----------------
ft_dir = NP01 / "field-tables"
np01_paths = []
for row in inv1["rows"]:
    rid = row["id"]
    tpath = ft_dir / f"{rid}.json"
    entry = {
        "id": rid,
        "gate": "NP01",
        "kind": "np01-product-path",
        "product_files": row.get("productFiles", []),
        "status_raw": row.get("status", ""),
        "field_table": {
            "path": tpath.relative_to(ROOT).as_posix(),
            "present": tpath.exists(),
            "sha256": sha(tpath) if tpath.exists() else None,
        },
        "remaining_quoted_from_inventory": row.get("evidence", {}).get("missing", []),
        "ledger_status": "partial" if tpath.exists() else "missing",
        "next_action": "NP5-01 审计既有表 + 补 host/device 逐表达式位级对照（不重写表）",
        "owner": {"author": "backend", "verify": "qa"},
        "blocked_by_user_decision": [],
    }
    np01_paths.append(entry)

# ---------------- NP01 host sites ----------------
ht_dir = NP01 / "host-tables"
host_sites = []
for hc in inv1["hostSiteRegistration"]["hostCoverage"]:
    hid = hc["id"]
    tpath = ht_dir / f"{hid}.json"
    host_sites.append({
        "id": hid,
        "gate": "NP01",
        "kind": "np01-host-site",
        "product_files": hc.get("productFiles", []),
        "field_table": {
            "path": tpath.relative_to(ROOT).as_posix(),
            "present": tpath.exists(),
            "sha256": sha(tpath) if tpath.exists() else None,
        },
        "ledger_status": "partial" if tpath.exists() else "missing",
        "next_action": "NP5-02 精度偏离审计（host 非融合 recipe 前提须声明）",
        "owner": {"author": "backend", "verify": "qa"},
    })

# ---------------- NP01 unregistered product paths (new after 2026-10-23) ----------------
R = "src/render"
UNREG = [
    # render kernel entries
    ("render_megakernel", "render-kernel-entry", [f"{R}/kernel/cuda/render_megakernel.cu"], None, None),
    ("render_depth_seg", "render-kernel-entry", [f"{R}/kernel/cuda/render.cu"], None, None),
    ("aa_resolve", "render-kernel-entry", [f"{R}/kernel/cuda/aa_resolve.cu"], "aa_resolve", "verified"),
    ("build_rays", "render-kernel-entry", [f"{R}/camera/cuda/build_rays.cu"], "build_rays", "verified"),
    ("mesh_bvh_ray", "render-kernel-entry", [f"{R}/bvh/cuda/mesh.cu"], "mesh_bvh_ray", "verified"),
    ("scene_bvh_ray", "render-kernel-entry", [f"{R}/bvh/cuda/scene.cu"], "scene_bvh_ray", "verified"),
    ("sample_texture2d", "render-kernel-entry", [f"{R}/texture/sample.cu"], "sample_texture2d", "verified"),
    ("s7a_abi_probe", "render-kernel-entry", [f"{R}/kernel/launch.rs"], "s7a_abi_probe", "verified"),
    # render host chain
    ("render_per_world_host", "render-host-chain", [f"{R}/kernel/per_world.rs"], None, None),
    ("world_light_host", "render-host-chain", [f"{R}/kernel/world_light.rs"], None, None),
    ("megakernel_arrays_host", "render-host-chain", [f"{R}/kernel/megakernel_arrays.rs"], None, None),
    ("render_diag_optin", "render-host-diagnostic", [f"{R}/kernel/diag.rs"], None, None),
    # collision narrowphase (CUDA assembly; no NP01 row today)
    ("mjwarp_collision_contact", "collision-kernel-entry", ["src/physics/collision/cuda/product.cu"], "mjwarp_collision_contact", "verified"),
    ("mjwarp_collision_contact_multi", "collision-kernel-entry", ["src/physics/collision/cuda/product.cu"], "mjwarp_collision_contact_multi", "verified"),
    ("mjwarp_collision_primitive", "collision-kernel-entry", ["src/physics/collision/cuda/product.cu"], "mjwarp_collision_primitive", "verified"),
    ("mjwarp_collision_boxbox", "collision-kernel-entry", ["src/physics/collision/cuda/product.cu"], "mjwarp_collision_boxbox", "verified"),
    ("mjwarp_collision_hfield", "collision-kernel-entry", ["src/physics/collision/cuda/product.cu"], "mjwarp_collision_hfield", "verified"),
    ("mjwarp_collision_sdf", "collision-kernel-entry", ["src/physics/collision/cuda/product.cu"], "mjwarp_collision_sdf", "verified"),
    ("collision_device_libs", "collision-device-lib-aggregate",
     ["src/physics/collision/cuda/"], None, None),
    # NP01 host semantics registered but no field table (hostSiteRegistration.siteSemantics only)
    ("host_site_render_light_dir_normalize", "np01-host-site-uncovered",
     ["src/render/kernel/megakernel_arrays.rs"], None, None),
    ("host_site_relmat_relative_matrix", "np01-host-site-uncovered",
     ["src/physics/collision/relmat.rs"], None, None),
    # product numeric helper modules absent from the 30-row inventory
    ("math_fma_exact_f32_helper", "np01-product-helper",
     ["src/math/fma.rs"], None, None),
    ("render_quat_cast_host", "np01-product-helper",
     ["src/render/kernel/cast.rs"], None, None),
]
np02_by_name = {r["name"]: r for r in inv2["rows"]}
unreg = []
for (uid, kind, files, np02_row, np02_state) in UNREG:
    present = all((ROOT / f).exists() for f in files)
    if kind == "np01-host-site-uncovered":
        action = "NP5-01b 新增 host-tables 表（B3 仅覆盖 gjk/ccd/epa 三站点；该站点只在 siteSemantics 登记、无表）"
    elif kind == "np01-product-helper":
        action = "NP5-01b 新增 NP01 行 + 逐字段表（f32 单次舍入 helper / quat→mat 宿主路径；当前 30 行清单未含）"
    elif np02_row:
        action = "NP5-01b 新增 NP01 逐字段类型表；NP02 已有行，核对其 source 身份"
    else:
        action = "NP5-01b 新增 NP01 逐字段表；NP02 无行 → NP5-03 补登记"
    unreg.append({
        "id": uid,
        "gate": "NP01",
        "kind": kind,
        "product_files": files,
        "products_present": present,
        "np01_status": "absent（未入 30 条清单；2026-10-23 后新增/实现）",
        "np02_row": np02_row,
        "np02_state": np02_state,
        "next_action": action,
        "owner": {"author": "backend", "verify": "qa"},
    })

# ---------------- NP02 100 kernels ----------------
np02_kernels = []
for r in inv2["rows"]:
    srcp = r.get("source", {}).get("path", "")
    np02_kernels.append({
        "name": r["name"],
        "route": r.get("route"),
        "module": r.get("module"),
        "source_path": srcp,
        "source_sha256": r.get("source", {}).get("sha256"),
        "evidence_state": r.get("evidence_state"),
        "receipts": r.get("artifact", {}).get("receipts"),
        "ptx_verified": r.get("artifact", {}).get("ptx_verified"),
        "owner": {"author": "backend", "verify": "qa"},
        "gap_ref": None,
    })
kmap = {k["name"]: k for k in np02_kernels}

gap_owner_map = {
    "已知漂移（裁决 B：历史版本证据，不重采不重基线）": ("archive", "none（裁决 B 归档；不重采不重基线）"),
    "派生（证据由基核承担）": ("covered-by-base", "none"),
    "未实现（不计入 Windows 缺失）": ("reclassify-or-blocker", "chief/user（见 np5-rulings-memo.md）"),
    "部分覆盖（NP01 缺口）": ("partial", "backend"),
    "不适用（辅助源，无独立内核）": ("n/a-archived", "none"),
    "不适用（无可独立计数内核）": ("n/a-archived", "none"),
    "不适用（非内核路径）": ("n/a-archived", "none"),
    "未判定": ("undetermined", "backend+qa"),
    "部分缺失": ("partial-missing", "chief（历史实体不补造口径）"),
    "deferred": ("deferred-linux", "user（Linux 解冻）"),
}
np02_gaps = []
for g in gap["items"]:
    st = g.get("status", "")
    action, owner = gap_owner_map.get(st, ("unknown", "chief"))
    np02_gaps.append({
        "id": g.get("id"),
        "name": g.get("name"),
        "status": st,
        "reason": g.get("reason"),
        "upstream_files": g.get("upstream_files", []),
        "product_paths": g.get("product_paths", []),
        "ledger_action": action,
        "owner": owner,
    })

# current-tree reclassification evidence for the 7 "unimplemented" gaps
su = [g for g in np02_gaps if g["ledger_action"] == "reclassify-or-blocker"]
subsystem_recheck = {
    "渲染/BVH": {"now_present": (ROOT / "src/render").is_dir(),
                 "evidence": "src/render/（bvh/camera/kernel/texture，8 个 .cu + 42 文件面快照）+ NP02 已登记 6 个渲染内核回执（build_rays/mesh_bvh_ray/scene_bvh_ray/s7a_abi_probe/aa_resolve/sample_texture2d）",
                 "verdict": "已实现（NP02 已覆盖渲染内核对；NP01 无行 → 走 NP5-01b，不是实现阻塞）"},
    "地形/SDF 碰撞": {"now_present": (ROOT / "src/physics/collision/cuda/sdf_math.cu").exists() and (ROOT / "src/physics/collision/cuda/hfield.cu").exists(),
                 "evidence": "sdf_math.cu/sdf_narrowphase.cu/sdf_octree.cu/hfield.cu 在 42 文件面内；NP02 有 mjwarp_collision_hfield/sdf 回执；SDF 发布结论 2026-11-02（sdf-release-v2.2）",
                 "verdict": "已实现（类型/证据登记走 NP01 新增行 + NP02 回执行；不是实现阻塞）"},
    "凸体/网格 GJK、EPA、convex 支持映射": {"now_present": (ROOT / "src/physics/collision/cuda/gjk_core.cu").exists() and (ROOT / "src/physics/collision/cuda/epa.cu").exists(),
                 "evidence": "gjk_core.cu/epa.cu/polytope*.cu/support.cu/simplex.cu 在面内；g10-p5 family closeout 已 verified",
                 "verdict": "已实现（同上，走登记；G08/G09 的边界口径不改）"},
    "柔体碰撞（flex collision）": {"now_present": (ROOT / "src/physics/collision/flex.rs").exists(),
                 "evidence": "产品无 flex 碰撞产品路径（multicontact_mesh.cu 为网格多接触）",
                 "verdict": "未实现 → 真实现阻塞"},
    "约束求解（constraint）": {"now_present": (ROOT / "src/physics/constraint.rs").exists(),
                 "evidence": "无产品路径",
                 "verdict": "未实现 → 真实现阻塞"},
    "求解器管线（solver：CG/Newton）": {"now_present": (ROOT / "src/physics/solver.rs").exists(),
                 "evidence": "无产品路径（mass_solve 仅为分解切片）",
                 "verdict": "未实现 → 真实现阻塞"},
    "岛屿/休眠分组（island）": {"now_present": (ROOT / "src/physics/island.rs").exists(),
                 "evidence": "仅 sleep.rs 树级切片",
                 "verdict": "未实现 → 真实现阻塞"},
}

# ---------------- NP02 post-2026-11-04 deltas ----------------
missing_kernels = [
    {
        "name": "render_megakernel",
        "entry": "render_megakernel",
        "source": "src/render/kernel/cuda/render_megakernel.cu（concat 装配：scene.cu + render_math/flex/common/shadow/shade + megakernel）",
        "why": "cuda-surface 中的真实 __global__；NP02 100 行无对应行（扫描口径只认 const r#... / include_str 单文件，concat! 组装源未生成行）",
        "change_history": "2026-11-11 内 5 次身份变更（f728b2d7、358b84fc、bb29e351、b0e372be、19a68b49）",
        "action": "NP5-03 补登记行 + CPU-only 回执（NVRTC 12.8.93，不建设备上下文）",
        "owner": "backend",
    },
    {
        "name": "render_depth_seg",
        "entry": "render_depth_seg",
        "source": "src/render/kernel/cuda/render.cu（concat 装配：scene.cu + render_math.cu + render.cu）",
        "why": "同上；NP02 无行",
        "change_history": "2026-11-11 P10 改 depth_seg 源（b0e372be 新身份 31912377…/138,857）",
        "action": "同上",
        "owner": "backend",
    },
]
new_drift = [{
    "row": "scene_bvh_ray",
    "source": "src/render/bvh/cuda/scene.cu",
    "recorded_sha256": np02_by_name["scene_bvh_ray"]["source"]["sha256"],
    "current_sha256_lf": nsha(ROOT / "src/render/bvh/cuda/scene.cu"),
    "changed_by": commit_of("src/render/bvh/cuda/scene.cu"),
    "cause": "2026-11-04 22:18 d35cab68（R06-S3b 1D flex 命中链 f32 收缩序）晚于清单生成 2026-11-04T06:30",
    "action": "NP5-03：按 12 新内核同法 CPU-only 补采回执（或按裁决 B 归档，需 Chief 定）",
    "owner": "chief",
}]

# cuda-surface 口径小注：文本计数含注释行
surface_note = {
    "text_count_globals": surf["totals"]["globals"],
    "note": "cuda-surface 的 globals 为文本计数（render.cu 第 15 行注释里也有 `__global__`），真实入口少 1；本账本按真实入口列，不改快照（快照棘轮属 NP01 门禁）。",
}

head_sha, head_subj = head()
ledger = {
    "schema": 1,
    "task": "np5-00（草稿；由 target/ 生成，待 Backend 落 docs/development-evidence/np-gate/）",
    "generated_at": datetime.datetime.now().astimezone().strftime("%Y-%m-%dT%H:%M:%S%z"),
    "producer": "target/render-bvh/gl2-ptx-diff/_gen_np5_ledger.py",
    "scope": "NP01（类型映射 100% / 精度偏离 0）与 NP02（证据覆盖 100% / 缺失 0）全量清单、判据映射、状态与缺口 owner；不宣称任何门禁通过",
    "inputs": {
        "np01_inventory": {"path": "docs/development-evidence/np01-type-mapping/inventory.json", "sha256": sha(NP01 / "inventory.json"), "rows": len(inv1["rows"]), "updatedAt": inv1.get("updatedAt")},
        "np01_field_tables": {"path": "docs/development-evidence/np01-type-mapping/field-tables/", "tables": len(list(ft_dir.glob('*.json')))},
        "np01_host_tables": {"path": "docs/development-evidence/np01-type-mapping/host-tables/", "tables": len(list(ht_dir.glob('*.json')))},
        "np02_inventory": {"path": "docs/development-evidence/np02-evidence/inventory.json", "sha256": sha(NP02 / "inventory.json"), "generated_at": inv2.get("generated_at"), "rows": len(inv2["rows"])},
        "np02_gap": {"path": "docs/development-evidence/np02-evidence/gap.json", "sha256": sha(NP02 / "gap.json"), "generated_at": gap.get("generated_at"), "items": len(gap["items"])},
        "cuda_surface": {"path": "docs/development-evidence/np01-type-mapping/cuda-surface.json", "sha256": sha(NP01 / "cuda-surface.json"), "totals": surf["totals"]},
        "tree": {"head": head_sha, "subject": head_subj},
    },
    "status_vocabulary": {
        "np01": ["verified", "partial", "missing", "absent"],
        "np02": ["verified", "source-drift", "derived", "missing"],
        "gap_action": ["archive", "covered-by-base", "reclassify-or-blocker", "partial", "n/a-archived", "undetermined", "partial-missing", "deferred-linux"],
    },
    "totals": {
        "np01_paths": len(np01_paths),
        "np01_paths_with_field_table": sum(1 for e in np01_paths if e["field_table"]["present"]),
        "np01_host_sites": len(host_sites),
        "np01_host_sites_with_table": sum(1 for e in host_sites if e["field_table"]["present"]),
        "np01_unregistered_product_units": len(unreg),
        "np02_kernels": len(np02_kernels),
        "np02_kernels_missing_rows": len(missing_kernels),
        "np02_gaps": len(np02_gaps),
        "np02_gaps_archive_drift": sum(1 for g in np02_gaps if g["ledger_action"] == "archive"),
        "ledger_entries_total": len(np01_paths) + len(host_sites) + len(unreg) + len(np02_kernels) + len(missing_kernels) + len(np02_gaps),
    },
    "np01": {"paths": np01_paths, "host_sites": host_sites, "unregistered_product_units": unreg},
    "np02": {
        "kernels": np02_kernels,
        "missing_kernel_rows": missing_kernels,
        "new_source_drift": new_drift,
        "gaps": np02_gaps,
        "subsystem_recheck": subsystem_recheck,
        "counts_snapshot": inv2["counts"],
        "surface_note": surface_note,
    },
    "boundaries": [
        "本账本是清单/状态登记，不是 NP01/NP02 通过结论；NP01 的 E 指标仍未定稿、host/device 逐表达式位级对照未补；NP02 的 7 子系统分类需按本账本 recheck 重述。",
        "不改判据/参考/容差；不重采证据；不写产品仓。",
        "9 条已知漂移按用户 2026-10-24 裁决 B 归档，不重采不重基线。",
        "render/diag（src/render/kernel/diag.rs）为 opt-in 诊断，env 未设时零副作用；其内核身份变化不改变数值路径，但属于 NP02 身份记录面。",
    ],
}

out = OUT_DIR / "np-gate-ledger.json"
out.write_text(json.dumps(ledger, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")
print("wrote", out, out.stat().st_size, "bytes", sha(out))
print("totals:", json.dumps(ledger["totals"], ensure_ascii=False))
