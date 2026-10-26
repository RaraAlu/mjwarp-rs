# -*- coding: utf-8 -*-
"""Assemble small human-readable summary/log files for the QA evidence dir."""
import json
import subprocess
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
TASK = REPO / "docs/development-evidence/g10-p5-g2-gpu-reference-capture-independent-check"
LOGS = TASK / "logs"
LOGS.mkdir(parents=True, exist_ok=True)


def load(name):
    return json.loads((TASK / name).read_text(encoding="utf-8"))


old = load("qa-old-untouched.json")
diff = load("qa-new-old-diff.json")
two = load("qa-two-rounds.json")
prov = load("qa-provenance.json")
script = load("qa-script-diff.json")
window = load("qa-window.json")

commands = [
    "# QA independent check commands (all read-only; no GPU window, no cargo)",
    "python qa_old_untouched.py       -> qa-old-untouched.json (exit 0)",
    "python qa_new_old_diff.py        -> qa-new-old-diff.json (exit 0)",
    "python qa_two_rounds.py          -> qa-two-rounds.json (exit 0)",
    "python qa_provenance.py          -> qa-provenance.json (exit 0)",
    "python qa_script_diff.py         -> qa-script-diff.json (exit 0)",
    "python qa_window.py              -> qa-window.json (exit 0)",
    "python qa_artifacts_sha256.py    -> artifacts-sha256.json",
    "run_gate_check.ps1               -> normify gate / normify check (stdout in target/qa-capture-check/gate-check/)",
]
(LOGS / "commands.txt").write_text("\n".join(commands) + "\n", encoding="utf-8")

summary = [
    "# QA summary (facts only; details in the qa-*.json files)",
    "",
    f"HEAD = {old['head']}  parent = {old['parent']}",
    f"old reference sha256 = {old['old_blobs']['docs/development-evidence/g10-p5-gjk-epa/upstream-mjwarp-f32.json']['worktree_sha256']}",
    f"old script    sha256 = {old['old_blobs']['docs/development-evidence/g10-p5-gjk-epa/capture_mjwarp_f32.py']['worktree_sha256']}",
    f"new reference sha256 = {old['new_reference']['worktree_sha256']}  ({old['new_reference']['commit_status']})",
    f"old files commit diff = {old['commit_diff_old_files']}; worktree status old area = {old['worktree_status_old_area']}",
    f"unexpected commit files = {old['unexpected_commit_files']}; src touched = {old['commit_touches_src']}",
    f"case_compare still reads old CPU reference = {old['case_compare_still_old_reference']}; src diff since 209b1039 = {old['src_diff_since_209b1039']}",
    "",
    "# inputs",
    f"revision same = {diff['revision']['same']}; case order same = {diff['case_names']['same_order']}",
    f"counts (new) = {diff['new_counts']}; two_captures_all_true = {diff['new_reference_two_captures_all_true']}",
    "",
    "# two rounds",
    f"round1 raw sha = {two['sha256']['round1_raw']}",
    f"round2 raw sha = {two['sha256']['round2_raw']}  identical = {two['raw_identical']}",
    f"canonical identical = {two['canonical_identical']}; final == round1 raw = {two['final_equals_round1_raw']}",
    f"meta sha matches recomputed = {two['meta_claims']['meta_matches_recomputed']}",
    f"independence: different_cmd={two['independence']['different_cmd']} different_cache={two['independence']['different_cache_dir']} different_wall={two['independence']['different_wall']} both_exit0={two['independence']['both_logs_exit_zero']}",
    f"caches: round1 files={two['caches']['round1']['files']} bytes={two['caches']['round1']['bytes']}; round2 files={two['caches']['round2']['files']} bytes={two['caches']['round2']['bytes']}",
    "",
    "# provenance",
    f"device={prov['device']} revision={prov['upstream_revision']}",
    f"gpu={json.dumps(prov['provenance']['gpu'], ensure_ascii=False)}",
    f"gpu match live nvidia-smi = {prov['gpu_match']}",
    f"version match installed = {prov['version_match']}",
    f"script sha = {prov['script']['sha256']} match provenance = {prov['script']['match']}",
    f"captured_at={prov['captured_at']} matches meta={prov['captured_at_matches_meta']} cache_root_exists={prov['kernel_cache_root_exists']}",
    f"authoring flags = {prov['authoring_flags']}",
    "",
    "# diff magnitude (old CPU vs new GPU, rows paired by INT key)",
    f"structural_fail={diff['structural_fail']} raw_row_order_differs={diff['raw_row_order_differs']}",
    f"float bit-equal = {diff['summary']['float_bit_equal']}/{diff['summary']['float_total']}; frame_max_abs={diff['summary']['frame_max_abs']:.6e}",
]
for case, entry in diff["cases"].items():
    summary.append(
        f"  {case}: count={entry['count']} int_multiset={entry['int_key_multiset_same']} "
        f"frame={entry['float_fields']['frame']['bit_equal']}/{entry['float_fields']['frame']['total']} "
        f"max_abs={entry['float_fields']['frame']['max_abs']:.4e} "
        f"dist_max={entry['float_fields']['dist']['max_abs']:.4e} pos_max={entry['float_fields']['pos']['max_abs']:.4e}")
summary += [
    "",
    "# script diff",
    f"core blocks identical = {script['core_blocks_identical']} ({script['core_blocks']})",
    f"hunks={script['diff_hunks']} changed={script['changed_lines']} categories={script['change_categories']}",
    f"provided diff hunks match my diff = {script['provided_diff']['hunks_match_my_diff']}",
    "",
    "# window",
    f"samples={window['samples']['count']} median_interval={window['samples']['median_interval_s']}s "
    f"used_peak={window['samples']['used_mib']['max']} free_min={window['samples']['free_mib']['min']} "
    f"util_max={window['samples']['util_percent']['max']} temp_max={window['samples']['temp_c']['max']}",
    f"compute apps pre/post = {window['compute_apps']['pre_count']}/{window['compute_apps']['post_count']} "
    f"same_name_set={window['compute_apps']['same_name_set']} same_pid_set={window['compute_apps']['same_pid_set']}",
    f"guard aborted=False? {window['guard']['aborted_false']}; residual empty? {window['guard']['residual_context_pids_empty']}",
    f"matches Arch record = {window['matches_approved_record']}",
]
(LOGS / "summary.txt").write_text("\n".join(summary) + "\n", encoding="utf-8")

# live GPU state at review time (read-only queries; no context)
gpu_line = subprocess.run(["nvidia-smi", "--query-gpu=name,memory.total,driver_version,compute_cap",
                           "--format=csv,noheader"], capture_output=True, text=True,
                          encoding="utf-8", errors="replace").stdout.strip()
apps = subprocess.run(["nvidia-smi", "--query-compute-apps=pid,process_name",
                       "--format=csv,noheader"], capture_output=True, text=True,
                      encoding="utf-8", errors="replace").stdout.strip()
apps_lines = [line for line in apps.splitlines() if line.strip()]
python_apps = [line for line in apps_lines if "python" in line.lower()]
live = [
    "# live nvidia-smi at QA review time (read-only query; no CUDA context created by this check)",
    gpu_line,
    f"compute-apps count = {len(apps_lines)}",
    f"python compute-apps = {python_apps if python_apps else '(none)'}",
    "old/new reference device: old=cpu, new=cuda",
]
(LOGS / "nvidia-smi-live.txt").write_text("\n".join(live) + "\n", encoding="utf-8")
print("wrote logs/commands.txt, logs/summary.txt, logs/nvidia-smi-live.txt")
