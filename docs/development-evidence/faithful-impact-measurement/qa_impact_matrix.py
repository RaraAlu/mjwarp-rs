# -*- coding: utf-8 -*-
"""Build impact-matrix.json for the faithful-FMA impact measurement.

Sources (all in this evidence dir / target raw dirs):
  - gpu-window-<attempt3>-summary.json  : valid window, 10 items x 2 modes
  - cpu logs: sdf-relmat-<mode>.log, case-evidence-print/assert-<mode>.log, sdf-host-<mode>.log
  - build logs: build-<mode>.log (rustc command line -> RUSTFLAGS/config interaction)
Attempts 1 (guard abort) and 2 (env-scope bug, invalid) are documented separately
and excluded from the matrix.
"""
import json
import re
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
EVIDENCE = REPO / "docs/development-evidence/faithful-impact-measurement"
LOGS = EVIDENCE / "logs"
ATTEMPT3 = "gpu-window-20261025-180638-summary.json"
MODES = ["gnu-default", "gnu-fma"]


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


# ---------------- CPU items ------------------------------------------------
cpu_items = []
sdf_lines = {}
for mode in ("gnu-default", "gnu-fma", "gnu-fma-ldc"):
    text = read(LOGS / f"sdf-relmat-{mode}.log")
    sdf_lines[mode] = [l.strip() for l in text.splitlines() if l.startswith("[SDF-rot]")]
    rel = next((l for l in sdf_lines[mode] if "rel_mat_warp" in l), "")
    scenes = sorted(l.split("]")[1].split(":")[0].strip() for l in sdf_lines[mode] if "bit-equal" in l)
    sdf_host = read(LOGS / f"sdf-host-{mode}.log")
    sdf_host_result = next((l.strip() for l in sdf_host.splitlines() if "test result:" in l), "")
    cpu_items.append({
        "mode": mode,
        "rel_mat_10_10": "10/10 world" in rel and "8 world" in rel,
        "rel_mat_line": rel,
        "sdf_scenes_bit_equal": all("80/80 contacts bit-equal" in l and "max_abs 0e0" in l for l in sdf_lines[mode] if "bit-equal" in l),
        "sdf_scene_count": len(scenes),
        "sdf_scene_max_abs_zero": all("max_abs 0e0" in l for l in sdf_lines[mode] if "bit-equal" in l),
        "sdf_host_fixture_test": sdf_host_result,
    })

def case_evidence(mode):
    print_log = read(LOGS / f"case-evidence-print-{mode}.log")
    assert_log = read(LOGS / f"case-evidence-assert-{mode}.log")
    line = next((l.strip() for l in print_log.splitlines() if "ellipsoid-box" in l), "")
    m = re.search(r"max abs ([0-9.eE+-]+)", line)
    frame = m.group(1) if m else ""
    result = next((l.strip() for l in assert_log.splitlines() if "test result:" in l), "")
    return {"frame": frame, "assert_result": result, "pass": "ok." in result, "line": line}

ce = {mode: case_evidence(mode) for mode in ("gnu-default", "gnu-fma", "gnu-fma-ldc")}

# ---------------- GPU items (valid attempt 3) ------------------------------
summary = json.loads(read(LOGS / ATTEMPT3))
runs = {}
for row in summary["results"]:
    runs[(row["mode"], row["filter"])] = row


def parse_collision(key_lines):
    cases = []
    for line in key_lines:
        m = re.search(r"case=(\S+) kind=(\S+) nacon=(\d+) nccd=(\d+) bit_equal=(\d+)/(\d+) max_dist_ulp=(\d+) max_pos_ulp=(\d+) max_frame_ulp=(\d+) failures=(\d+)", line)
        if m:
            cases.append({
                "case": m.group(1), "kind": m.group(2), "nacon": int(m.group(3)), "nccd": int(m.group(4)),
                "bit_equal": m.group(5) + "/" + m.group(6), "max_dist_ulp": int(m.group(7)),
                "max_pos_ulp": int(m.group(8)), "max_frame_ulp": int(m.group(9)), "failures": int(m.group(10)),
            })
    return cases


def parse_depth_seg(key_lines):
    out = {"recipe": {}, "fast_math": {}, "reference": {}}
    for line in key_lines:
        m = re.search(r"s7b1\[recipe\] (cam\d): depth_mismatch=(\d+) max_ulp=(\d+).*seg_mismatch=(\d+)", line)
        if m:
            out["recipe"][m.group(1)] = {"depth_mismatch": int(m.group(2)), "max_ulp": int(m.group(3)), "seg_mismatch": int(m.group(4))}
        m = re.search(r"s7b1\[s0b-fast-math\] (cam\d): depth_mismatch=(\d+) max_ulp=(\d+).*seg_mismatch=(\d+)", line)
        if m:
            out["fast_math"][m.group(1)] = {"depth_mismatch": int(m.group(2)), "max_ulp": int(m.group(3)), "seg_mismatch": int(m.group(4))}
        m = re.search(r"s7b1 reference: rays=(\d+) recipe_max_ulp=(\d+) known_cross_recipe_beyond4=(\d+)", line)
        if m:
            out["reference"] = {"rays": int(m.group(1)), "recipe_max_ulp": int(m.group(2)), "known_cross_recipe_beyond4": int(m.group(3))}
    return out


def parse_rgb(key_lines):
    out = {"regress": {}, "same_recipe": {}, "cross_recipe_record": {}, "hashes": {}}
    for line in key_lines:
        m = re.search(r"s7b2\[regress\] (cam\d): depth_mismatch=(\d+) max_ulp=(\d+) seg_mismatch=(\d+)", line)
        if m:
            out["regress"][m.group(1)] = {"depth_mismatch": int(m.group(2)), "max_ulp": int(m.group(3)), "seg_mismatch": int(m.group(4))}
        m = re.search(r"s7b2\[rgb-same-recipe\] (cam\d): pixels=(\d+) mismatched=(\d+) max_byte_diff=(\d+) beyond_1_lsb=(\d+)", line)
        if m:
            out["same_recipe"][m.group(1)] = {"pixels": int(m.group(2)), "mismatched": int(m.group(3)),
                                              "max_byte_diff": int(m.group(4)), "beyond_1_lsb": int(m.group(5))}
        m = re.search(r"s7b2\[rgb-cross-recipe-record\] (cam\d): beyond_1_lsb=(\d+) max_byte_diff=(\d+)", line)
        if m:
            out["cross_recipe_record"][m.group(1)] = {"beyond_1_lsb": int(m.group(2)), "max_byte_diff": int(m.group(3))}
        m = re.search(r"depth_seg_hash=(\w+) rgb_hash=(\w+)", line)
        if m:
            out["hashes"] = {"depth_seg": m.group(1), "rgb": m.group(2)}
    return out


def parse_flags(key_lines):
    out = {"variants": {}, "summary": "", "aa2": {}, "baseline": {}}
    for line in key_lines:
        m = re.search(r"s7b3\[(\w+)\] (cam\d): mismatched=(\d+) max_byte_diff=(\d+) beyond_1_lsb=(\d+)", line)
        if m and m.group(1) not in ("aa2", "baseline"):
            out["variants"].setdefault(m.group(1), {})[m.group(2)] = {"mismatched": int(m.group(3)),
                                                                      "max_byte_diff": int(m.group(4)), "beyond_1_lsb": int(m.group(5))}
        m = re.search(r"s7b3\[(\w+)\] (cam\d): depth_mismatch=(\d+) max_ulp=(\d+) seg_mismatch=(\d+)", line)
        if m and m.group(1) == "culling_off":
            out["variants"].setdefault("culling_off_depth", {})[m.group(2)] = {"depth_mismatch": int(m.group(3)),
                                                                               "max_ulp": int(m.group(4)), "seg_mismatch": int(m.group(5))}
        m = re.search(r"s7b3\[aa2\] (cam\d): mismatched=(\d+) max_byte_diff=(\d+) beyond_1_lsb=(\d+)", line)
        if m:
            out["aa2"].setdefault(m.group(1), {})["rgb"] = {"mismatched": int(m.group(2)), "max_byte_diff": int(m.group(3)), "beyond_1_lsb": int(m.group(4))}
        m = re.search(r"s7b3\[aa2\] (cam\d): depth_mismatch=(\d+) max_ulp=(\d+) seg_mismatch=(\d+)", line)
        if m:
            out["aa2"].setdefault(m.group(1), {})["depth"] = {"depth_mismatch": int(m.group(2)), "max_ulp": int(m.group(3)), "seg_mismatch": int(m.group(4))}
        m = re.search(r"s7b3\[baseline\] (cam\d): rgb_mismatched=(\d+) max_byte_diff=(\d+) beyond_1_lsb=(\d+)", line)
        if m:
            out["baseline"].setdefault(m.group(1), {})["rgb"] = {"rgb_mismatched": int(m.group(2)), "max_byte_diff": int(m.group(3)), "beyond_1_lsb": int(m.group(4))}
        m = re.search(r"s7b3\[baseline\] (cam\d): depth_mismatch=(\d+) max_ulp=(\d+) seg_mismatch=(\d+)", line)
        if m:
            out["baseline"].setdefault(m.group(1), {})["depth"] = {"depth_mismatch": int(m.group(2)), "max_ulp": int(m.group(3)), "seg_mismatch": int(m.group(4))}
        m = re.search(r"s7b3 flag variants: (\d+)/(\d+) matched", line)
        if m:
            out["summary"] = f"{m.group(1)}/{m.group(2)} variants matched"
    return out


def parse_e2e(key_lines):
    out = {"depth": {}, "rgb": {}, "cross_recipe_record": {}, "hashes": {}, "camlight": {}}
    for line in key_lines:
        m = re.search(r"s8\[e2e\] (cam\d) depth: compared=(\d+) mismatch=(\d+) max_ulp=(\d+) seg_mismatch=(\d+)", line)
        if m:
            out["depth"][m.group(1)] = {"compared": int(m.group(2)), "mismatch": int(m.group(3)), "max_ulp": int(m.group(4)), "seg_mismatch": int(m.group(5))}
        m = re.search(r"s8\[e2e\] (cam\d) rgb: pixels=(\d+) mismatched=(\d+) max_byte_diff=(\d+) beyond_1_lsb=(\d+)", line)
        if m:
            out["rgb"][m.group(1)] = {"pixels": int(m.group(2)), "mismatched": int(m.group(3)),
                                      "max_byte_diff": int(m.group(4)), "beyond_1_lsb": int(m.group(5))}
        m = re.search(r"s8\[e2e\]\[cross-recipe-record\] (cam\d): beyond_1_lsb=(\d+) max_byte_diff=(\d+)", line)
        if m:
            out["cross_recipe_record"][m.group(1)] = {"beyond_1_lsb": int(m.group(2)), "max_byte_diff": int(m.group(3))}
        m = re.search(r"depth_seg_hash=(\w+) rgb_hash=(\w+)", line)
        if m:
            out["hashes"] = {"depth_seg": m.group(1), "rgb": m.group(2)}
        m = re.search(r"bit_diff=PoseBitDiff \{ origin_mismatches: \[(\d+), (\d+)\], xmat_mismatches: \[(\d+), (\d+)\], max_origin_abs: ([\d.]+), max_xmat_abs: ([\d.]+)", line)
        if m:
            out["camlight"] = {"origin_mismatches": [int(m.group(1)), int(m.group(2))],
                               "xmat_mismatches": [int(m.group(3)), int(m.group(4))],
                               "max_origin_abs": float(m.group(5)), "max_xmat_abs": float(m.group(6))}
    return out


item_specs = [
    ("launcher regression", "g10_p5_nvrtc_launcher", "product_nvrtc_launcher_matches_g5_probe_out1", parse_collision),
    ("primitive 3 cases", "g10_p5_primitive_cuda", "primitive_launcher_matches_qa_upstream_gpu", parse_collision),
    ("multicontact 3 cases", "g10_p5_multicontact_cuda", "multicontact_launcher_matches_qa_upstream_gpu", parse_collision),
    ("S7b1 depth/seg", "render_kernel_depth_seg", "depth_and_seg_match_frozen_reference_for_both_cameras", parse_depth_seg),
    ("S7b2 RGB", "render_kernel_rgb", "rgb_matches_reference_and_depth_seg_do_not_regress", parse_rgb),
    ("S7b3 flag variants", "render_kernel_flags", "flag_variants_match_own_references", parse_flags),
    ("S7b3 AA spp=2", "render_kernel_flags", "aa2_matches_same_recipe_reference", parse_flags),
    ("S7b3 baseline no-regression", "render_kernel_flags", "baseline_no_regression_after_flags_and_aa", parse_flags),
    ("S8 e2e same-recipe", "render_e2e", "end_to_end_two_cameras_match_same_recipe_reference", parse_e2e),
    ("S8 camlight pose bits", "render_e2e", "product_camlight_poses_match_reference_bits", parse_e2e),
]

gpu_items = []
for item, target, filt, parser in item_specs:
    per_mode = {}
    for mode in MODES:
        row = runs.get((mode, filt))
        per_mode[mode] = {"exit": row["exit"], "wall_s": row["wall_s"], "metrics": parser(row["key_lines"]),
                          "log": row["log"]}
    same = per_mode["gnu-default"]["metrics"] == per_mode["gnu-fma"]["metrics"] and \
        per_mode["gnu-default"]["exit"] == per_mode["gnu-fma"]["exit"]
    gpu_items.append({
        "item": item, "target": target, "filter": filt,
        "verified_expectation": "pass (bit-level / <=1 LSB thresholds as recorded in the slice evidence)",
        "gnu_default": per_mode["gnu-default"],
        "gnu_fma": per_mode["gnu-fma"],
        "identical_between_modes": same,
        "flip": not same,
        "reference": "frozen reference present",
    })

# ---------------- window facts (valid attempt 3) ---------------------------
pre = read(LOGS / ATTEMPT3.replace("summary.json", "pre.txt")).splitlines()[0]
post = read(LOGS / ATTEMPT3.replace("summary.json", "post.txt")).splitlines()[0]
guard_params = {
    "increment_limit_mib": 3072,
    "free_floor_mib": 10240,
    "new_cuda_compute_abort": "yes unless process exe matches ^(g10_p5_|render_) for this task",
    "cpu_only_processes": "recorded only, no abort",
    "sample_interval_ms": 250,
}
window = {
    "start_utc": summary["window_start_utc"], "end_utc": summary["window_end_utc"],
    "duration_s": 146,
    "pre_snapshot": pre, "post_snapshot": post,
    "start_used_mib": summary["start_used_mib"],
    "samples": summary["samples"],
    "guard_tripped": summary["guard_tripped"],
    "guard_params": guard_params,
    "residual_compute_processes_of_this_task": "0 (all test exes exited; pre/post query-compute-apps recorded)",
    "in_window_compilation": "0 (all 20 run logs contain no 'Compiling ' line; binaries from out-of-window build)",
}

# ---------------- assemble -------------------------------------------------
flips = [{"item": "case_evidence::upstream_f32_within_4ulp (CPU)",
          "gnu_default": {"result": "pass", "frame": ce["gnu-default"]["frame"]},
          "gnu_fma": {"result": "fail", "frame": ce["gnu-fma"]["frame"]},
          "delta": "pass -> fail; frame 3.838112e-7 -> 6.351978e-7 (> 4.8e-7 threshold)"}]
unchanged_gpu = [g["item"] for g in gpu_items if not g["flip"]]
unchanged_cpu = ["G5 rel_mat 10/10 world", "SDF 40/40 (5 scenes, 80/80 contacts each)"]

result = {
    "task": "2026-10-25-faithful-impact-measurement",
    "generated_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "pinned_commit": "5e000cb9",
    "worktree": "target/worktrees/faithful-5e000cb9",
    "repo_head_at_review": "3a738c29",
    "modes": {
        "gnu-default": "repo default GNU (config rustflags -C link-dead-code=yes)",
        "gnu-fma": "GNU + RUSTFLAGS=-C target-feature=+fma (config rustflags overridden, no link-dead-code)",
        "gnu-fma-ldc": "GNU + RUSTFLAGS='-C target-feature=+fma -C link-dead-code=yes' (CPU-only isolation variant)",
    },
    "rustflags_config_interaction": {
        "default_rustc_line_has_link_dead_code": True,
        "gnu_fma_rustc_line_has_link_dead_code": False,
        "gnu_fma_rustc_line_has_target_feature_fma": True,
        "gnu_fma_ldc_rustc_line_has_both": True,
        "conclusion": "setting RUSTFLAGS replaces the .cargo/config.toml gnu rustflags; link-dead-code is dropped unless re-added explicitly",
        "evidence": "logs/rustc-command-lines.txt",
    },
    "clean_build_criterion": {
        "dedicated_target_dirs": ["target/faithful/gnu-default", "target/faithful/gnu-fma", "target/faithful/gnu-fma-ldc",
                                   "target/faithful/gnu-default-cuda", "target/faithful/gnu-fma-cuda"],
        "cargo_rust_env_cleared": True,
        "rustc": "1.99.0 (b940084d7 2026-09-28) host x86_64-pc-windows-gnu",
        "cargo": "1.99.0 (5f94df478 2026-08-27)",
        "deps_files": {"cpu_default": 71, "cpu_fma": 71, "cpu_fma_ldc": 71, "cuda_probe_build": 91},
    },
    "cpu_items": {
        "modes": cpu_items,
        "case_evidence": ce,
        "flip": "case_evidence only",
    },
    "gpu_items": gpu_items,
    "window": window,
    "attempts": [
        {"attempt": 1, "status": "ABORTED_BY_GUARD_INVALID", "reason": "conservative name-based guard tripped on another agent's CPU lib-test binary (mjwarp_rs-<hash>); not a GPU contention event; only 2/20 runs; excluded from matrix", "record": "logs/window-attempt-1-aborted.txt"},
        {"attempt": 2, "status": "INVALID_EXCLUDED", "reason": "driver script cleared CARGO_TARGET_DIR/RUSTFLAGS after each test; runs 2-10 of each mode fell back to the worktree target (in-window recompile) and the +fma mode degraded to default GNU for 9/10 runs; excluded from matrix", "record": "logs/window-attempt-2-invalid.txt"},
        {"attempt": 3, "status": "VALID_USED", "reason": "env kept stable per mode; 20/20 exit 0; no in-window compilation; correct per-mode target dirs", "record": "logs/window-attempt-3-final.txt"},
    ],
    "summary": {
        "flips": flips,
        "unchanged_gpu_items": unchanged_gpu,
        "unchanged_cpu_items": unchanged_cpu,
        "impact_scope": "narrow: only case_evidence::upstream_f32_within_4ulp (CPU frame judgement) flips; all GPU launcher/primitive/multicontact bit-level comparisons and all render S7b1/S7b2/S7b3/S8 same-recipe comparisons are byte-identical between the two modes in this window",
        "missing_reference_items": [],
    },
    "boundaries": {
        "read_only": True,
        "product_code_modified": False,
        "criteria_tolerance_modified": False,
        "new_reference_captured": False,
        "gpu_window": "10-minute approved window; each mode serial",
        "not_run": ["render perf", "Linux", "other GPUs", "kernel-level disassembly of why render is insensitive"],
        "claims": "reports only which verified results change/not change under the faithful mode; no fix/acceptance claim",
    },
}
(EVIDENCE / "impact-matrix.json").write_text(render(result) + "\n", encoding="utf-8")
print("wrote impact-matrix.json")
print("gpu items:", len(gpu_items), "flips:", [g["item"] for g in gpu_items if g["flip"]])
print("identical_gpu:", sum(1 for g in gpu_items if g["identical_between_modes"]))
print("case_evidence:", ce)
print("window:", window["start_utc"], window["end_utc"], window["samples"]["count"], "samples", window["samples"]["used_mib_max"], window["samples"]["free_mib_min"])
