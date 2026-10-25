# -*- coding: utf-8 -*-
"""Assemble post-fix (F1 fixed) QA summaries from my own logs.

Inputs (all produced by this QA check, post-fix tree at 3b7db351):
  probe-logs-postfix/probe-*.log      self-written probe, 3 builds x (plain, seam env)
  logs-postfix/g5-*.log               clean-target G5 reruns + new default-gate edge test
  logs-postfix/seam-binary-scan-postfix.txt
  probe-logs/probe-*.log              PREFIX logs, re-parsed here as the "before" side

Outputs:
  qa-probe-summary-postfix.json       before/after probe matrices
  qa-g5-postfix.json                  G5 bitwise results, edge-class test, binary scan
"""
import json
import re
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
TASK = REPO / "docs/development-evidence/exact-f32-fma-relmat-independent-check"
LOGS_BEFORE = TASK / "probe-logs"
LOGS_AFTER = TASK / "probe-logs-postfix"
RUNLOGS = TASK / "logs-postfix"
BUILDS = ["gnu-default", "gnu-fma", "msvc-default"]


def read(path):
    return Path(path).read_text(encoding="utf-8", errors="replace")


def parse_probe(path):
    out = {"info": "", "seam_env_check": "", "stats": {}, "boundary_mismatches": [],
           "relmat_chain": [], "summary": ""}
    for line in read(path).splitlines():
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


def probe_matrix(logs_dir):
    return {build: {"plain": parse_probe(logs_dir / f"probe-{build}.log"),
                    "seam_env": parse_probe(logs_dir / f"probe-{build}-seam.log")}
            for build in BUILDS}


def compact(entry):
    """Keep the decision-relevant numbers per build/mode; full STAT lines stay in the logs."""
    stats = entry["stats"]
    return {
        "info": entry["info"],
        "seam_env_check": entry["seam_env_check"],
        "dispatch_4096": stats.get("dispatch_4096"),
        "dispatch_1e6": stats.get("dispatch_1e6"),
        "seam_4096": stats.get("seam_4096"),
        "seam_1e6": stats.get("seam_1e6"),
        "soft_1e6": stats.get("soft_1e6"),
        "boundary_dispatch": stats.get("boundary_dispatch"),
        "boundary_seam": stats.get("boundary_seam"),
        "dispatch_vs_seam_1e6": stats.get("dispatch_vs_seam_1e6"),
        "muladd_4096_diffs": stats.get("muladd_4096", {}).get("diffs"),
        "summary": entry["summary"],
        "relmat_chain": entry["relmat_chain"],
        "boundary_mismatches": entry["boundary_mismatches"],
    }


before = probe_matrix(LOGS_BEFORE)
after = probe_matrix(LOGS_AFTER)


def chain_cells(relmat_lines):
    cells = {}
    for line in relmat_lines:
        m = re.search(r"RELMAT_CHAIN name=(\S+) cells_mismatched=(\d+)/9", line)
        if m:
            cells[m.group(1)] = int(m.group(2))
    return cells


probe = {
    "purpose": "post-fix independent probe of src/math/fma.rs (F1 fixed): dispatch vs seam-forced fallback vs inline-asm hardware FMA",
    "tree": {"git_head": "3b7db351bcf32050ee65f95debdc6d35cdc5febb", "fix_commit": "3b7db351",
             "src_math_fma_sha256": "3c865f4b5bfc9d96c97409ed7d0fd22b019c42cea4b17d950b1325ed17a2486b"},
    "probe_source": "probe-postfix/qa_fma_probe.rs (same self-written probe as the prefix check; does not use src/math/fma/tests.rs)",
    "snapshot": "probe-postfix/fma_snapshot.rs = mechanical copy of the FIXED src/math/fma.rs, leading '//!' -> '//' only, 39 lines; audit in qa-static-checks-postfix.json",
    "reference": "inline asm vfmadd231ss",
    "generator": {"lcg_mult": "6364136223846793005", "lcg_inc": "1442695040888963407",
                  "lcg_seed": "0x2026102500000001", "mapping": "f32(top24(state))/2^24*3-1 in [-1,2)"},
    "sizes": {"generated": [4096, 1000000], "boundary_cases": 46},
    "after_postfix_check": {build: {"plain": compact(after[build]["plain"]),
                                    "seam_env": compact(after[build]["seam_env"])}
                            for build in BUILDS},
    "before_vs_after": {
        "boundary_seam_mismatches": {
            build: {"before": before[build]["seam_env"]["stats"].get("boundary_seam", {}).get("diffs"),
                    "after": after[build]["seam_env"]["stats"].get("boundary_seam", {}).get("diffs")}
            for build in BUILDS},
        "boundary_dispatch_mismatches": {
            build: {"before": before[build]["plain"]["stats"].get("boundary_dispatch", {}).get("diffs"),
                    "after": after[build]["plain"]["stats"].get("boundary_dispatch", {}).get("diffs")}
            for build in BUILDS},
        "relmat_chain_cells_mismatched": {build: {"before": chain_cells(before[build]["seam_env"]["relmat_chain"]),
                                                  "after": chain_cells(after[build]["seam_env"]["relmat_chain"])}
                                          for build in BUILDS},
    },
}
(TASK / "qa-probe-summary-postfix.json").write_text(
    json.dumps(probe, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

# ---- G5 / edge test / binary scan ----
def read_lines(name):
    path = RUNLOGS / name
    return read(path) if path.exists() else ""


g5 = {"tree": probe["tree"], "runs": {}}
for log in sorted(RUNLOGS.glob("g5-*.log")):
    text = read_lines(log.name)
    result = next((line.strip() for line in text.splitlines() if line.startswith("test result:")), "")
    details = [line.strip() for line in text.splitlines()
               if ("80/80" in line or "10/10" in line or "max_abs" in line)]
    g5["runs"][log.stem] = {"log": f"logs-postfix/{log.name}", "test_result": result,
                            "bitwise_detail_lines": details}

edge = read_lines("g5-lib-soft-fma-edge-class.log")
g5["default_gate_edge_class_test"] = {
    "command": "cargo test --locked --lib math::fma::tests::soft_fma_exact_bits_by_edge_class -- --exact --nocapture",
    "log": "logs-postfix/g5-lib-soft-fma-edge-class.log",
    "env_var_set": False,
    "env_snapshot": "logs-postfix/env-snapshot-postfix.txt (MJWARP_FORCE_SOFT_FMA_set=False)",
    "case_lines": len([ln for ln in edge.splitlines() if "F1 a=" in ln]),
    "fail_lines": len([ln for ln in edge.splitlines() if "FAIL" in ln]),
    "test_result": next((ln.strip() for ln in edge.splitlines() if ln.startswith("test result:")), ""),
}
g5["seam_forced_product_run"] = {
    "command": "MJWARP_FORCE_SOFT_FMA=1 cargo test --locked --lib math::fma -- --test-threads=1",
    "log": "logs-postfix/lib-math-fma-seam.log",
    "test_result": next((ln.strip() for ln in read_lines("lib-math-fma-seam.log").splitlines()
                         if ln.startswith("test result:")), ""),
}
g5["binary_seam_scan"] = {
    "log": "logs-postfix/seam-binary-scan-postfix.txt",
    "lines": [ln.strip() for ln in read_lines("seam-binary-scan-postfix.txt").splitlines()
              if "contains_env=" in ln],
}
g5["gate_check"] = {
    "note": "gate/check run on the main tree after the evidence dir was frozen; raw stdout is kept "
            "outside the evidence scope (target/qa-fma-check/gate-check/, not committed) because the "
            "gate scope IS this evidence dir - writing logs into it during the gate run makes "
            "normify check report the gate results as stale.",
    "gate_exit": 0,
    "check_exit": 0,
    "runs_record": ".normify/runs/2026-10-25-exact-f32-fma-relmat-independent-check/",
}
(TASK / "qa-g5-postfix.json").write_text(
    json.dumps(g5, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

print("postfix summaries written")
print(json.dumps(probe["before_vs_after"], ensure_ascii=False, indent=2))
print(json.dumps(g5["default_gate_edge_class_test"], ensure_ascii=False, indent=2))
