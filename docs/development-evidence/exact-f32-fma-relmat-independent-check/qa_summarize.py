# -*- coding: utf-8 -*-
"""Assemble QA evidence summaries for the exact-f32-fma/relmat independent check."""
import json
import re
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
TASK = REPO / "docs/development-evidence/exact-f32-fma-relmat-independent-check"
LOGS = TASK / "probe-logs"
RUNLOGS = TASK / "logs"
BUILDS = ["gnu-default", "gnu-fma", "msvc-default"]


def render(o, indent=0):
    sp = "  " * indent
    sp2 = "  " * (indent + 1)
    if isinstance(o, dict):
        if not o:
            return "{}"
        parts = [f"{sp2}{json.dumps(k, ensure_ascii=False)}: {render(v, indent + 1)}" for k, v in o.items()]
        return "{\n" + ",\n".join(parts) + "\n" + sp + "}"
    if isinstance(o, list):
        if not o:
            return "[]"
        if all(not isinstance(x, (dict, list)) for x in o):
            return json.dumps(o, ensure_ascii=False)
        parts = [sp2 + json.dumps(x, ensure_ascii=False) if isinstance(x, dict) else sp2 + render(x, indent + 1) for x in o]
        return "[\n" + ",\n".join(parts) + "\n" + sp + "]"
    return json.dumps(o, ensure_ascii=False)


def read(path):
    return Path(path).read_text(encoding="utf-8", errors="replace")


def parse_probe(path):
    text = read(path)
    out = {"info": "", "seam_env_check": "", "stats": {}, "boundary_mismatches": [], "relmat_chain": [], "summary": ""}
    for line in text.splitlines():
        if "INFO cpuid_fma=" in line:
            out["info"] = line.strip()
        elif line.startswith("SEAM_ENV_CHECK"):
            out["seam_env_check"] = line.strip()
        elif line.startswith("STAT "):
            m = re.match(r"STAT (\S+) n=(\d+) diffs=(\d+) max_ulp=(-?\d+) digest=([0-9a-f]+)", line)
            if m:
                out["stats"][m.group(1)] = {"n": int(m.group(2)), "diffs": int(m.group(3)),
                                            "max_ulp": int(m.group(4)), "digest": m.group(5)}
        elif line.startswith("BOUNDARY_MISMATCH"):
            out["boundary_mismatches"].append(line.strip())
        elif line.startswith("RELMAT_CHAIN"):
            out["relmat_chain"].append(line.strip())
        elif line.startswith("SUMMARY"):
            out["summary"] = line.strip()
    return out


probe = {
    "purpose": "independent probe of src/math/fma.rs: dispatch path vs seam-forced software path vs inline-asm hardware FMA",
    "probe_source": "probe/qa_fma_probe.rs (self-written; does not use src/math/fma/tests.rs)",
    "snapshot": "probe/fma_snapshot.rs = mechanical copy of src/math/fma.rs with leading '//!' -> '//' only (see probe/qa-fma-snapshot.json)",
    "reference": "inline asm vfmadd231ss (hardware IEEE FMA); no #[target_feature] wrapper",
    "generator": {"lcg_mult": "6364136223846793005", "lcg_inc": "1442695040888963407",
                  "lcg_seed": "0x2026102500000001", "mapping": "f32(top24(state))/2^24*3-1 in [-1,2)"},
    "sizes": {"generated": [4096, 1000000], "boundary_cases": 46},
    "builds": {},
}
for build in BUILDS:
    probe["builds"][build] = {
        "plain": parse_probe(LOGS / f"probe-{build}.log"),
        "seam_env": parse_probe(LOGS / f"probe-{build}-seam.log"),
    }
probe["delivery_claim_compare"] = {
    "delivery_dispatch_4096_1e6_boundary": "0 mismatches (three builds)",
    "mine_dispatch_4096_1e6": "0 mismatches in all three builds (matches)",
    "mine_boundary_dispatch": "0 / 46 (matches)",
    "delivery_soft_boundary": "0 mismatches on its 58-case set",
    "mine_seam_forced_boundary": "8 / 46 mismatches in all three builds (CONTRADICTS the delivery claim)",
    "delivery_seam_test": "samples=4154 mismatches=0 (its own boundary set does not include the failing classes)",
}

# ---------------- static checks -------------------------------------------
def parse_asm(tag):
    text = read(TASK / f"asm-vfmadd-{tag}.txt")
    info = {}
    for line in text.splitlines():
        if "=" in line and not line.startswith("#"):
            key, value = line.split("=", 1)
            info[key.strip()] = value.strip()
        elif re.match(r"^\d+: ", line):
            info.setdefault("context", []).append(line.strip())
    return info


def has_env(rel, needle=b"MJWARP_FORCE_SOFT_FMA"):
    path = REPO / rel
    if not path.is_file():
        return None
    return needle in path.read_bytes()


test_exes = sorted((REPO / "target/qa-fma-check/tests/debug/deps").glob("mjwarp_rs-*.exe"))
binary_scan = {
    "product_debug": {"path": "target/qa-fma-check/prod-debug/debug/mjwarp-rs.exe",
                      "contains_env": has_env("target/qa-fma-check/prod-debug/debug/mjwarp-rs.exe")},
    "product_release": {"path": "target/qa-fma-check/prod-release/release/mjwarp-rs.exe",
                        "contains_env": has_env("target/qa-fma-check/prod-release/release/mjwarp-rs.exe")},
    "deps_exes": [],
}
for exe in test_exes:
    rel = str(exe.relative_to(REPO)).replace("\\", "/")
    binary_scan["deps_exes"].append({"path": rel, "size": exe.stat().st_size, "contains_env": has_env(rel)})

g5 = {}
for name in ("lib-math-fma", "lib-relmat", "test-g10_p5_sdf_relmat", "test-g10_p5_sdf_host", "test-gjk_epa", "lib-math-fma-seam"):
    if name == "lib-math-fma-seam":
        path = RUNLOGS / "lib-math-fma-seam.log"
        seam_path = RUNLOGS / "product-seam-test-nocapture.log"
    else:
        path = RUNLOGS / f"g5-{name}.log"
        seam_path = None
    if not path.exists():
        continue
    text = read(path)
    g5[name] = {
        "result_lines": [l.strip() for l in text.splitlines() if "test result:" in l],
        "sdf_lines": [l.strip() for l in text.splitlines() if "80/80 contacts bit-equal" in l or "rel_mat_warp" in l],
        "seam_lines": [l.strip() for l in (read(seam_path) if seam_path else text).splitlines()
                       if l.startswith("SEAM") or "PATH cpuid" in l],
    }

static = {
    "purpose": "path hit / seam boundary / G5 bitwise / diff scope / f64 audit cross-checks",
    "asm": {"gnu": parse_asm("gnu"), "msvc": parse_asm("msvc")},
    "binary_seam_scan": binary_scan,
    "seam_source_scan": {
        "env_read_sites": ["src/math/fma.rs:122 (inside #[cfg(test)] mod test_seam)"],
        "product_build_excludes_seam": "confirmed by product debug/release binary scan (contains_env=false)",
        "test_harness_contains_env": "confirmed (54,796,088-byte lib test harness contains_env=true)",
        "no_runtime_env_branch_in_product": True,
    },
    "three_builds": {
        build: probe["builds"][build]["plain"]["info"] for build in BUILDS
    },
    "g5_rerun": g5,
    "diff_scope": {
        "commit": "f0dceb5b",
        "src_files_changed": [
            "src/math/fma.rs (new, 449 lines)",
            "src/math/fma/tests.rs (new, 377 lines)",
            "src/math/mod.rs (+2)",
            "src/physics/collision/relmat.rs (22 lines changed: removed private f64 fma helper, now uses crate::math::fma::fma_f32)",
            "src/render/bvh/aabb.rs (+6, comments only)",
        ],
        "polytope_rs_changed": False,
        "megakernel_arrays_rs_changed": False,
        "aabb_implementation_changed": False,
        "criteria_tolerance_exclusions_gates_scripts_changed": False,
    },
    "f64_audit": {
        "delivery_audit_occurrences": 227,
        "delivery_audit_classification": "allowed-test/diagnostic 158, allowed-diagnostic/check 30, allowed-ABI/type 13, allowed-PTX-gate 7, allowed-error-payload 7, allowed-ABI-conversion 4, allowed-readback 1, allowed-input-check 2, allowed-fixture-comment 3, fixed-this-slice 2, pending-review 0",
        "independent_check": "added lines containing f64 in the commit are documentation only (fma.rs docs, aabb comment); fma.rs code has no f64; relmat f64 helper removed; contract NP01 note present at docs/numerical-precision-contract.md:346-348",
    },
    "integer_method_deviation": {
        "registered": True,
        "where": "src/math/fma.rs module doc '合规论证' section + README §10 + contract NP01 note",
        "status": "Arch-accepted method deviation (integer exact simulation instead of Dekker/TwoSum)",
    },
}

# ---------------- finding ------------------------------------------------
finding = {
    "id": "F1",
    "severity": "blocking",
    "summary": "software fallback of src/math/fma.rs violates IEEE single-rounding FMA for zero-product and c=inf classes: `c.bits as f32` is a numeric u32->f32 cast instead of a bit reinterpretation",
    "sites": ["src/math/fma.rs:158 (c == Inf branch)", "src/math/fma.rs:169 (a or b zero branch)"],
    "minimal_repro": {"call": "fma_f32(0.0, 1.0, -1.0)", "expected": "0xbf800000 (-1.0)", "got": "0x4f3f8000 (~3.2e9)"},
    "boundary_mismatches_46": 8,
    "boundary_mismatch_cases": [
        "zero_product_neg_c a=00000000 b=3f800000 c=bf800000 got=4f3f8000 want=bf800000",
        "negzero_times_one_plus_two a=80000000 b=3f800000 c=40000000 got=4e800000 want=40000000",
        "one_times_zero_plus_five a=3f800000 b=00000000 c=40a00000 got=4e814000 want=40a00000",
        "zero_times_negone_plus_half a=00000000 b=bf800000 c=3f000000 got=4e7c0000 want=3f000000",
        "zero_b_zero_c_nonzero a=00000000 b=80000000 c=3f800000 got=4e7e0000 want=3f800000",
        "c_inf_finite_ab a=3f800000 b=40000000 c=7f800000 got=4eff0000 want=7f800000",
        "c_neg_inf_finite_ab a=3f800000 b=40000000 c=ff800000 got=4f7f8000 want=ff800000",
        "zero_a_c_inf a=00000000 b=3f800000 c=7f800000 got=4eff0000 want=7f800000",
    ],
    "relmat_chain_impact": "same 3-FMA k-ascending chain as relmat::upstream_relative_matrix, software path: identity x identity 2/9 cells, rz90 x rz90 2/9, ry45 x rz90 3/9 cells corrupted",
    "impact_scope": "this machine (FMA3) uses the hardware dispatch -> shipped G5 results unchanged; a real non-FMA machine would run this fallback for every call and corrupt rel_mat (zero entries are common in rotation matrices, e.g. identity/90-degree)",
    "delivery_claim_contradicted": "README/receipt claim 0/0/0 for both paths on their 58-case boundary set; their set has (0,0,0) and (0,1,-0) but no zero-a/b with finite non-zero c and no finite-a/b with c=inf, so it misses both bug classes",
    "fix_hypothesis": {
        "change": "return c.bits as f32; -> return f32::from_bits(c.bits); (2 sites)",
        "verified_on_snapshot_copy": True,
        "result": "boundary_seam 0/46, all relmat-chain samples 0 mismatched, dispatch/seam random 0/1e6; product tree was NOT modified",
        "record": "qa-fix-hypothesis.json",
    },
}

summary = {
    "task": "2026-10-25-exact-f32-fma-relmat-independent-check",
    "subject_task": "2026-10-25-exact-f32-fma-relmat",
    "subject_commit": "f0dceb5b",
    "review_head": "fef2af9c",
    "verdict": {"result": "blocking", "blocking": 1, "findings": 1, "observations": 5},
    "probe": probe,
    "static_checks": static,
    "finding": finding,
}
(TASK / "qa-probe-summary.json").write_text(render(probe) + "\n", encoding="utf-8")
(TASK / "qa-static-checks.json").write_text(render(static) + "\n", encoding="utf-8")
(TASK / "qa-finding.json").write_text(render(finding) + "\n", encoding="utf-8")
print("wrote qa-probe-summary.json / qa-static-checks.json / qa-finding.json")
print("boundary mismatches (gnu-default seam):", len(probe["builds"]["gnu-default"]["seam_env"]["boundary_mismatches"]))
print("g5:", {k: v["result_lines"] for k, v in g5.items()})
print("binary scan:", binary_scan["product_debug"]["contains_env"], binary_scan["product_release"]["contains_env"],
      [(d["path"].split("/")[-1], d["contains_env"]) for d in binary_scan["deps_exes"]])
