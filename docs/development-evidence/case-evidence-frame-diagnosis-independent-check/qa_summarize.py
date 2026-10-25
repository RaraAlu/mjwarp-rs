# -*- coding: utf-8 -*-
"""Assemble QA evidence summaries for the frame-diagnosis independent check.

Inputs (raw, produced by run_probe.ps1 / run_frame_matrix.ps1):
  docs/development-evidence/case-evidence-frame-diagnosis-independent-check/probe-*.log
  .../frame-matrix.json
  target/qa-frame-diag/frame/*.log (+ env/toolchain records)
Outputs (evidence):
  qa-probe-summary.json, qa-exclusions.json, frame-matrix-summary.txt, logs/*.log
"""
import json
import re
import shutil
import subprocess
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
EVIDENCE = REPO / "docs/development-evidence/case-evidence-frame-diagnosis-independent-check"
RAW = REPO / "target/qa-frame-diag/frame"
TREE = REPO / "target/qa-frame-diag/tree"
LOGS_OUT = EVIDENCE / "logs"
LOGS_OUT.mkdir(parents=True, exist_ok=True)

HEAD = subprocess.run(["git", "rev-parse", "HEAD"], cwd=REPO, capture_output=True, text=True).stdout.strip()

PROBE_BUILDS = [
    ("gnu-default", "+1.99.0-x86_64-pc-windows-gnu", ""),
    ("msvc-default", "+1.99.0-x86_64-pc-windows-msvc", ""),
    ("gnu-fma", "+1.99.0-x86_64-pc-windows-gnu", "-C target-feature=+fma"),
    ("msvc-fma", "+1.99.0-x86_64-pc-windows-msvc", "-C target-feature=+fma"),
]
COMPARISONS = [
    "muladd_vs_hw_fma",
    "muladd_vs_unfused",
    "muladd_vs_f64_double",
    "hw_fma_vs_f64_double",
    "hw_fma_vs_unfused",
    "unfused_vs_f64_double",
]
# Delivery numbers from docs/development-evidence/case-evidence-frame-diagnosis/
# (probe-fmaf.log / README §3.3), kept only as comparison data.
DELIVERY_PROBE_4096 = {
    "gnu-default": {"muladd_vs_hw_fma": 991, "muladd_vs_unfused": 986, "muladd_vs_f64_double": 991},
    "msvc-default": {"muladd_vs_hw_fma": 0, "muladd_vs_unfused": 55, "muladd_vs_f64_double": 0},
    # delivery only reported "+fma builds agree with each other = 0/4096"
    "gnu-fma": {"cross_build_only": 0},
}

CMP_RE = re.compile(r"^CMP (\S+) diffs=(\d+) max_ulp=(\d+) ulp_eq_1=(\d+) ulp_ge_2=(\d+)$")
SAMPLE_RE = re.compile(r"^SAMPLE(?:_WORST)? (\S+) a_bits=([0-9a-f]{8}) b_bits=([0-9a-f]{8}) c_bits=([0-9a-f]{8}) got_bits=([0-9a-f]{8}) want_bits=([0-9a-f]{8}) ulp=(\d+)$")
DIGEST_RE = re.compile(r"^DIGEST muladd=([0-9a-f]{16}) hw_fma=([0-9a-f]{16}) unfused=([0-9a-f]{16}) f64_double=([0-9a-f]{16})$")


def render(o, indent=0):
    """Compact JSON: dict rows stay one line each (evidence files stay <=500 lines)."""
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


def parse_probe(tag):
    text = (EVIDENCE / f"probe-{tag}.log").read_text(encoding="utf-8", errors="replace")
    blocks = []
    current = None
    for line in text.splitlines():
        if line.startswith("PROBE n="):
            current = {"n": int(re.search(r"n=(\d+)", line).group(1)), "comparisons": {}, "samples": {}, "digests": None}
            blocks.append(current)
        elif current is not None:
            m = CMP_RE.match(line)
            if m:
                current["comparisons"][m.group(1)] = {
                    "diffs": int(m.group(2)),
                    "max_ulp": int(m.group(3)),
                    "ulp_eq_1": int(m.group(4)),
                    "ulp_ge_2": int(m.group(5)),
                }
                continue
            m = SAMPLE_RE.match(line)
            if m:
                current["samples"].setdefault(m.group(1), []).append({
                    "a_bits": m.group(2), "b_bits": m.group(3), "c_bits": m.group(4),
                    "got_bits": m.group(5), "want_bits": m.group(6), "ulp": int(m.group(7)),
                })
                continue
            m = DIGEST_RE.match(line)
            if m:
                current["digests"] = {"muladd": m.group(1), "hw_fma": m.group(2), "unfused": m.group(3), "f64_double": m.group(4)}
    return blocks


probe = {"builds": {}, "generator": {
    "kind": "64-bit LCG",
    "multiplier": "6364136223846793005",
    "increment": "1442695040888963407",
    "seed": "0x2026102500000001",
    "value": "f32(top24(state))/2^24 * 3 - 1 (range [-1,2))",
    "order": "a, b, c = three consecutive LCG outputs",
    "reference_hw_fma": "inline asm vfmadd231ss (CPUID FMA3 present, checked at runtime)",
}}
hw_by_n = {}
for tag, toolchain, flags in PROBE_BUILDS:
    blocks = parse_probe(tag)
    probe["builds"][tag] = {
        "toolchain": toolchain,
        "extra_rustc_flags": flags,
        "runs": blocks,
    }
    for block in blocks:
        if block.get("digests"):
            hw_by_n.setdefault(block["n"], set()).add(block["digests"]["hw_fma"])
probe["hw_fma_reference_digest_by_n"] = {str(n): sorted(values) for n, values in sorted(hw_by_n.items())}
probe["hw_fma_reference_stable_across_builds"] = all(len(values) == 1 for values in hw_by_n.values())

# Cross-check the two +fma builds and MSVC default against hardware FMA at 1e6.
probe["faithfulness_at_1e6"] = {}
for tag in probe["builds"]:
    block = next((b for b in probe["builds"][tag]["runs"] if b["n"] == 1_000_000), None)
    if block:
        probe["faithfulness_at_1e6"][tag] = block["comparisons"]["muladd_vs_hw_fma"]

# Delivery comparison at n=4096 (comparison data only, not reproduced exactly:
# delivery log does not record its LCG parameters or seeds).
comparison = []
for tag in ("gnu-default", "msvc-default", "gnu-fma"):
    block = next((b for b in probe["builds"][tag]["runs"] if b["n"] == 4096), None)
    row = {"build": tag, "mine_4096": {}, "delivery_4096": DELIVERY_PROBE_4096.get(tag, {})}
    for comparison_name in ("muladd_vs_hw_fma", "muladd_vs_unfused", "muladd_vs_f64_double"):
        mine = block["comparisons"][comparison_name]["diffs"] if block else None
        delivery = DELIVERY_PROBE_4096.get(tag, {}).get(comparison_name)
        row["mine_4096"][comparison_name] = mine
        if delivery is not None:
            row.setdefault("delta_mine_minus_delivery", {})[comparison_name] = mine - delivery
    if tag == "gnu-fma":
        row["delta_note"] = "delivery did not report per-reference counts for +fma builds; only cross-build agreement 0/4096"
    comparison.append(row)
probe["delivery_compare_4096"] = comparison
probe["note"] = (
    "Delivery probe log records no LCG parameters/seeds, so its exact 991/4096 cannot be "
    "reproduced bit-for-bit; this probe uses its own documented LCG. Rates agree (GNU default "
    "~24-25% 1-ULP differences vs hardware FMA; MSVC default and +fma builds 0)."
)
(EVIDENCE / "qa-probe-summary.json").write_text(render(probe) + "\n", encoding="utf-8")

# ---- exclusions / environment matrix --------------------------------------
frame = json.loads((EVIDENCE / "frame-matrix.json").read_text(encoding="utf-8"))

DELIVERY_FRAMES = {
    "gnu-default": ("pass", "3.838112e-7"),
    "msvc-default": ("fail", "6.351978e-7"),
    "gnu-fma": ("fail", "6.351978e-7"),
    "msvc-fma": ("fail", "6.351978e-7"),
    "gnu-release": ("pass", "3.838112e-7"),
    "msvc-release": ("fail", "6.351978e-7"),
    "msvc-plus-ldc": ("fail", "6.351978e-7"),
    "gnu-no-config": ("pass", "3.838112e-7"),
}

def git_hash(rel, rev=None):
    if rev:
        return subprocess.run(["git", "rev-parse", f"{rev}:{rel}"], cwd=REPO, capture_output=True, text=True).stdout.strip()
    return subprocess.run(["git", "hash-object", rel], cwd=TREE, capture_output=True, text=True).stdout.strip()

inputs = {}
for rel in (
    "src/physics/collision/gjk/case_evidence.rs",
    "src/physics/collision/gjk/case_compare.rs",
    "src/physics/collision/gjk/case_data.rs",
    "fixtures/g10-p5/references/ellipsoid-box.json",
):
    worktree_hash = git_hash(rel)
    head_hash = git_hash(rel, "eac9af71")
    inputs[rel] = {"worktree": worktree_hash, "head_eac9af71": head_hash, "identical": worktree_hash == head_hash}

env_records = {}
for cell in [row["id"] for row in frame if "command_assert" in row]:
    env_file = RAW / f"env-{cell}.txt"
    toolchain_file = RAW / f"toolchain-{cell}.txt"
    entry = {"env_snapshot": [], "toolchain_record": []}
    if env_file.exists():
        entry["env_snapshot"] = env_file.read_text(encoding="utf-8").splitlines()
    if toolchain_file.exists():
        entry["toolchain_record"] = toolchain_file.read_text(encoding="utf-8").splitlines()
    env_records[cell] = entry

exclusions = {
    "task": "2026-10-25-case-evidence-frame-diagnosis-independent-check",
    "generated_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "head_reviewed": "eac9af71",
    "repo_head_now": HEAD,
    "product_diff_eac9af71_to_now": subprocess.run(
        ["git", "diff", "--stat", "eac9af71..HEAD", "--", "src", "fixtures", "scripts", "Cargo.toml", "Cargo.lock"],
        cwd=REPO, capture_output=True, text=True).stdout.strip(),
    "worktree_status_product": subprocess.run(
        ["git", "status", "--porcelain", "--", "src", "fixtures", "scripts", "Cargo.toml", "Cargo.lock"],
        cwd=TREE, capture_output=True, text=True).stdout.strip(),
    "clean_build_record": env_records,
    "input_identity": inputs,
    "env_var_check": {
        "gnu-default": "CARGO*/RUST* 0 before clear, 0 after clear",
        "gnu-default-envset": "same binary with G2_DUMP_NATIVE_ERROR=1 -> exit 0 (unchanged)",
    },
    "cells": frame,
    "delivery_compare": {
        row["id"]: {
            "mine": {"exit": row.get("exit_assert_test"), "frame": row.get("frame_value")},
            "delivery": {
                "verdict": DELIVERY_FRAMES.get(row["id"], (None, None))[0],
                "frame": DELIVERY_FRAMES.get(row["id"], (None, None))[1],
            },
        }
        for row in frame if row["id"] in DELIVERY_FRAMES
    },
    "exclusion_conclusions": {
        "env_vars": "排除：清空 CARGO*/RUST* 后结果不变；G2_DUMP_NATIVE_ERROR 设置后结果不变",
        "link_dead_code": "排除：MSVC 显式加 -C link-dead-code=yes 仍红 (6.351978e-7)；GNU 去掉 .cargo/config.toml 仍绿 (3.838112e-7)",
        "release": "排除优化因素：GNU release 绿 (3.838112e-7)、MSVC release 红 (6.351978e-7)，与 debug 同结论",
        "artifact_origin": "全部 8 格使用全新专用 target（deps 62/65/67），两侧均复现",
        "fixtures_inputs": "该测试 4 个关键输入与 HEAD eac9af71 逐字节一致",
    },
}
(EVIDENCE / "qa-exclusions.json").write_text(json.dumps(exclusions, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

# ---- human-readable frame matrix summary ----------------------------------
lines = [
    "# Product-level frame matrix (independent reproduction; each cell = brand-new CARGO_TARGET_DIR)",
    "# test: physics::collision::gjk::case_evidence::upstream_f32_within_4ulp (assert, exit) and",
    "#       our_contacts_match_upstream_f32_within_4ulp (prints [judge] line with ellipsoid-box frame)",
    "",
]
for row in frame:
    if "command_assert" not in row:
        lines += [f"## {row['id']}", f"note: {row.get('note','')}", f"exit={row.get('exit_assert_test')}",
                  f"result: {row.get('assert_result_line')}", ""]
        continue
    lines += [
        f"## {row['id']}",
        f"tree={row['tree']} toolchain={row['toolchain'] or '(repo default GNU)'} rustflags={row['rustflags'] or '(none)'} release={row['release']}",
        f"rustc {row['rustc_host']}",
        f"target={row['target_dir']} deps_files={row['deps_files']} cargo_rust_env_before={row['cargo_rust_env_before_count']}",
        f"$ {row['command_assert']}",
        f"exit={row['exit_assert_test']} | {row['assert_result_line']}",
    ]
    if row.get("assert_panic_frame"):
        lines.append(f"panic: {row['assert_panic_frame']}")
    lines += [
        f"$ {row['command_print']}",
        f"exit={row['exit_print_test']} frame={row['frame_value']}",
        f"line: {row['print_frame_line']}",
        "",
    ]
(EVIDENCE / "frame-matrix-summary.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")

# ---- copy raw logs --------------------------------------------------------
copied = 0
for log in sorted(RAW.glob("*.log")):
    shutil.copyfile(log, LOGS_OUT / log.name)
    copied += 1
print(f"wrote qa-probe-summary.json, qa-exclusions.json, frame-matrix-summary.txt; copied {copied} raw logs to logs/")
print("probe hw reference stable:", probe["hw_fma_reference_stable_across_builds"], probe["hw_fma_reference_digest_by_n"])
for row in comparison:
    print(row["build"], "mine", row["mine_4096"], "delivery", row["delivery_4096"])
print("inputs identical:", all(v["identical"] for v in inputs.values()))
