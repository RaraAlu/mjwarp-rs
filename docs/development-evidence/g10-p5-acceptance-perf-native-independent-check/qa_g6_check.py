"""G6 独立复核：验收映射 / perf 方法学 / 窗口合规 / native 落点 的程序化核对。

输入只读：g10-p5-acceptance-map/{map.json,perf/*}、docs/numerical-precision-contract.md、
G5/G3/G4 既有 verified 证据、git 历史。输出 qa-g6-check.json + 控制台摘要。

用法：python qa_g6_check.py
"""

import json
import statistics
import subprocess
import sys
from pathlib import Path

import numpy as np

REPO = Path(__file__).resolve().parents[3]
MAPD = REPO / "docs" / "development-evidence" / "g10-p5-acceptance-map"
PERF = MAPD / "perf"
EVIDENCE = Path(__file__).resolve().parent

PINNED_UPSTREAM = "71da24d956378a87a703b6e1442b13aec0c4ac29"
WINDOW_HEAD = "9c0a0480"
PRODUCT_BASE = "6510317f"


def load(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def git(*args, check=False):
    p = subprocess.run(["git", "-c", "core.longpaths=true", *args], cwd=REPO,
                       capture_output=True)
    if check and p.returncode != 0:
        raise RuntimeError(p.stderr.decode("utf-8", "replace"))
    return p


def strip_evidence_path(raw):
    s = raw.strip()
    for sep in ("（", "::", "："):
        if sep in s:
            s = s.split(sep, 1)[0]
    return s.strip()


def p95_nearest_rank(samples):
    s = sorted(samples)
    if len(s) == 1:
        return float(s[0])
    idx = int(np.ceil(0.95 * len(s))) - 1
    return float(s[max(0, min(idx, len(s) - 1))])


def p95_linear(samples):
    return float(np.percentile(samples, 95))


def stats_block(samples):
    s = sorted(samples)
    return {
        "n": len(s),
        "min": float(s[0]),
        "max": float(s[-1]),
        "median_statistics": float(statistics.median(s)),
        "median_nearest_rank": float(s[(len(s) - 1) // 2]),
        "p95_nearest_rank": p95_nearest_rank(s),
        "p95_linear": p95_linear(s),
    }


def main():
    out = {"schema": 1, "kind": "qa-g6-independent-check"}

    # ---------- 1) 验收映射：72 条 evidence_paths + 状态一致性 ----------
    mapj = load(MAPD / "map.json")
    path_rows = []
    for r in mapj["requirements"]:
        for raw in r["evidence_paths"]:
            p = strip_evidence_path(raw)
            path_rows.append({"req": r["requirement_id"], "raw": raw, "path": p,
                              "exists": (REPO / p).exists()})
    for u in mapj["uncovered"]:
        for raw in u.get("evidence_paths", []):
            p = strip_evidence_path(raw)
            path_rows.append({"req": u.get("requirement_id", "?"), "raw": raw, "path": p,
                              "exists": (REPO / p).exists()})
    status_rows = []
    for r in mapj["requirements"]:
        st = r["status"]
        status_rows.append({
            "id": r["requirement_id"],
            "status": st,
            "reason_non_empty": bool((r.get("reason") or "").strip()),
            "impact_non_empty": bool((r.get("impact") or "").strip()),
            "measured_non_empty": bool((r.get("measured") or "").strip()),
            "criterion_non_empty": bool((r.get("criterion_quote") or "").strip()),
            "evidence_count": len(r["evidence_paths"]),
        })
    from collections import Counter
    st_counts = Counter(r["status"] for r in mapj["requirements"])
    untracked_paths = []
    for p in sorted({r["path"] for r in path_rows if r["exists"]}):
        if git("ls-files", "--error-unmatch", "--", p).returncode != 0:
            untracked_paths.append(p)
    out["map"] = {
        "summary_declared": mapj["summary"],
        "recounted": {"requirements": len(mapj["requirements"]),
                      "met": st_counts.get("met", 0),
                      "partial": st_counts.get("partial", 0),
                      "unmet": st_counts.get("unmet", 0),
                      "uncovered": len(mapj["uncovered"])},
        "evidence_path_total": len(path_rows),
        "evidence_path_missing": [r for r in path_rows if not r["exists"]],
        "evidence_path_untracked": untracked_paths,
        "status_rows": status_rows,
        "partial_or_unmet": [r for r in status_rows if r["status"] != "met"],
        "upstream_field": mapj.get("upstream"),
        "upstream_matches_pinned": mapj.get("upstream") == PINNED_UPSTREAM,
    }

    # ---------- 2) G6-1.4 sdf-box：从 G5 result.json 独立取数 ----------
    g5_result = load(REPO / "docs/development-evidence/g10-p5-gpu-kernels/gpu-window/result.json")
    sdf = None
    for c in g5_result.get("cases", []):
        if isinstance(c, dict) and c.get("case") == "sdf-box":
            sdf = c
    g614 = {}
    if sdf:
        g614 = {
            "keys_available": list(sdf.keys()),
            "device_count": sdf.get("device_count"),
            "reference_count": sdf.get("reference_count"),
            "dist_bit_equal": sdf.get("dist_bit_equal"),
            "dist_components": sdf.get("dist_components"),
            "dist_max_ulp": sdf.get("dist_max_ulp"),
            "pos_bit_equal": sdf.get("pos_bit_equal"),
            "pos_components": sdf.get("pos_components"),
            "pos_max_abs": sdf.get("pos_max_abs"),
            "pos_scale_max": sdf.get("pos_scale_max"),
            "pos_max_ulps_at_scale": sdf.get("pos_max_ulps_at_scale"),
            "frame_bit_equal": sdf.get("frame_bit_equal"),
            "frame_components": sdf.get("frame_components"),
            "frame_max_abs": sdf.get("frame_max_abs"),
            "nccd": sdf.get("nccd"), "overflow": sdf.get("overflow"),
            "two_runs_sorted_identical": sdf.get("two_runs_sorted_identical"),
            "map_measured_consistent": None,
        }
        # 与 map G6-1.4 measured 文本交叉核对（只做数字存在性核对）
        measured = ""
        for r in mapj["requirements"]:
            if r["requirement_id"] == "G6-1.4":
                measured = r["measured"]
        g614["map_measured_consistent"] = all(tok in measured for tok in
                                              ("80/80", "203/240", "675/720", "7.4506e-9"))
        g614["map_measured_tokens"] = {
            "80/80": "80/80" in measured, "203/240": "203/240" in measured,
            "675/720": "675/720" in measured, "7.4506e-9": "7.4506e-9" in measured,
            "1.0 ULP@scale": "1.0 ULP@scale" in measured,
        }
    out["g6_1_4_sdf_box"] = g614

    # ---------- 3) perf：原始样本独立重算（host 9 / device 7 / upstream 9）----------
    summary = load(PERF / "perf-summary.json")
    cases = {c["case"]: c for c in summary["cases"]}
    perf = {"recompute": [], "mismatches": [], "coverage": summary.get("coverage"),
            "notes_present": {
                "no_compile_jit": any("编译" in n and "JIT" in n for n in summary.get("notes", [])),
                "no_cross_hardware": any("不跨硬件外推" in n for n in summary.get("notes", [])),
                "contacts_per_s_magnitude_only": any("量级参考" in n for n in summary.get("notes", [])),
                "host_window_disclosed": any("独占" in n or "窗口" in n for n in summary.get("notes", [])),
            }}

    for f in sorted((PERF / "host").glob("*.json")):
        d = load(f)
        st = stats_block(d["samples_ns"])
        computed_cps = d["ncontacts"] * 1e9 / st["median_statistics"] if st["median_statistics"] else None
        rec = {"case": d["case"], "kind": "host", "samples": st["n"],
               "median_stored": d.get("median_ns"), "median_recomputed_avg2": st["median_statistics"],
               "p95_stored": d.get("p95_ns"), "p95_recomputed_nr": st["p95_nearest_rank"],
               "p95_recomputed_linear": st["p95_linear"],
               "ncontacts": d.get("ncontacts"),
               "contacts_per_s_stored": d.get("contacts_per_s"),
               "contacts_per_s_recomputed": computed_cps}
        sc = cases.get(d["case"], {}).get("host_cpu_single_thread")
        if sc:
            rec["summary_match"] = (
                sc.get("samples") == st["n"]
                and abs((sc.get("median_ns") or -1) - st["median_statistics"]) < 1e-6
                and abs((sc.get("p95_ns") or -1) - st["p95_nearest_rank"]) < 1e-6
                and sc.get("ncontacts") == d.get("ncontacts")
            )
            if not rec["summary_match"]:
                perf["mismatches"].append({"case": d["case"], "kind": "host", "summary": sc, "recomputed": st})
        else:
            rec["summary_match"] = "absent"
        perf["recompute"].append(rec)

    for f in sorted((PERF / "device").glob("*.json")):
        d = load(f)
        k = stats_block(d["kernel_ns"])
        e = stats_block(d["e2e_ns"])
        rec = {"case": d["case"], "kind": "device",
               "kernel_samples": k["n"], "e2e_samples": e["n"],
               "kernel_median_stored": d.get("kernel_median_ns"),
               "kernel_median_recomputed": k["median_statistics"],
               "e2e_median_stored": d.get("e2e_median_ns"),
               "e2e_median_recomputed": e["median_statistics"],
               "median_def_matches": (d.get("kernel_median_ns") == k["median_statistics"]
                                      and d.get("e2e_median_ns") == e["median_statistics"]),
               "kernel_p95_nr": k["p95_nearest_rank"], "e2e_p95_nr": e["p95_nearest_rank"],
               "nacon": d.get("nacon"), "nccd": d.get("nccd"), "overflow": d.get("overflow"),
               "device": d.get("device"), "cc": d.get("compute_capability"),
               "cudart": d.get("cudart_version"), "nworld": d.get("nworld"),
               "naconmax": d.get("naconmax"), "free_before": d.get("free_mib_before"),
               "free_after": d.get("free_mib_after"), "total_mib": d.get("total_mib"),
               "kernel_method": d.get("kernel_only_method"),
               "e2e_method": d.get("end_to_end_method")}
        sc = cases.get(d["case"], {})
        smk = sc.get("device_kernel_only"); sme = sc.get("device_end_to_end")
        meta = sc.get("device_case_meta") or {}
        env_dev = (summary.get("environment") or {}).get("device") or {}
        if smk and sme:
            rec["summary_match"] = (
                smk.get("samples") == k["n"] and sme.get("samples") == e["n"]
                and abs(smk.get("median_ns", -1) - k["median_statistics"]) < 1e-6
                and abs(sme.get("median_ns", -1) - e["median_statistics"]) < 1e-6
                and abs(smk.get("p95_ns", -1) - k["p95_nearest_rank"]) < 1e-6
                and abs(sme.get("p95_ns", -1) - e["p95_nearest_rank"]) < 1e-6
            )
            rec["meta_match"] = (
                meta.get("kernel_kind") == d.get("kernel_kind")
                and meta.get("nworld") == d.get("nworld")
                and meta.get("naconmax") == d.get("naconmax")
                and meta.get("efc_width") == d.get("efc_width")
                and meta.get("nacon") == d.get("nacon")
                and meta.get("nccd") == d.get("nccd")
                and meta.get("overflow") == d.get("overflow")
            )
            rec["device_env_match"] = (
                env_dev.get("device") == d.get("device")
                and str(env_dev.get("compute_capability")) == str(d.get("compute_capability"))
                and env_dev.get("cudart_version") == d.get("cudart_version")
            )
            if not rec["summary_match"]:
                perf["mismatches"].append({"case": d["case"], "kind": "device",
                                           "kernel_summary": smk, "e2e_summary": sme,
                                           "kernel_recomputed": k, "e2e_recomputed": e})
        else:
            rec["summary_match"] = "absent"
        perf["recompute"].append(rec)

    # upstream 列
    up = load(PERF / "upstream-cpu.json")
    up_cases = {}
    for c in up["cases"]:
        up_cases[c["case"]] = c
    up_rows = []
    for name, c in sorted(up_cases.items()):
        samples = c.get("samples_ns") or c.get("timed_ns") or []
        st = stats_block(samples) if samples else None
        sc = cases.get(name, {}).get("upstream_f32_cpu")
        row = {"case": name, "samples": st["n"] if st else 0}
        if st and sc:
            row.update({
                "median_stored_summary": sc.get("median_ns"),
                "median_recomputed": st["median_statistics"],
                "p95_stored_summary": sc.get("p95_ns"),
                "p95_json_stored": c.get("p95_ns"),
                "p95_linear": st["p95_linear"],
                "p95_nearest_rank": st["p95_nearest_rank"],
                # perf-summary 用 nearest-rank；upstream-cpu.json 用 numpy 线性插值（perf/README §5.8）
                "summary_p95_is_nearest_rank": abs((sc.get("p95_ns") or -1) - st["p95_nearest_rank"]) < 1e-6,
                "json_p95_is_linear": abs((c.get("p95_ns") or -1) - st["p95_linear"]) < 1e-6,
                "summary_match": (sc.get("samples") == st["n"]
                                  and abs(sc.get("median_ns", -1) - st["median_statistics"]) < 1e-6
                                  and abs(sc.get("p95_ns", -1) - st["p95_nearest_rank"]) < 1e-6),
                "ncontacts": c.get("ncontacts"),
                "count_matches_reference": c.get("count_matches_reference"),
                "contacts_per_s_stored": sc.get("contacts_per_s"),
                "contacts_per_s_recomputed": (c.get("ncontacts") * 1e9 / st["median_statistics"]
                                              if c.get("ncontacts") and st["median_statistics"] else None),
            })
            if not row["summary_match"]:
                perf["mismatches"].append({"case": name, "kind": "upstream",
                                           "summary": sc, "recomputed": st})
        up_rows.append(row)
    perf["upstream_rows"] = up_rows
    perf["upstream_meta"] = {k: up.get(k) for k in
                             ("upstream", "upstream_head_verified", "upstream_head_matches_pinned",
                              "warmup", "reps", "p95_method", "excluded_from_samples",
                              "python_gc", "synchronization", "timer", "load_ms_total",
                              "compile_ms_total", "make_data_ms_total", "totals",
                              "covered_cases", "uncovered_cases", "failures")}
    perf["columns"] = summary.get("columns")
    perf["environment"] = summary.get("environment")

    # artifacts-sha256.txt / build-report.txt：证据文件与探针源哈希复核
    import hashlib

    def sha256_of(path):
        return hashlib.sha256(Path(path).read_bytes()).hexdigest()

    inv_rows = []
    inv_lines = [ln for ln in (PERF / "artifacts-sha256.txt").read_text(encoding="utf-8").splitlines()
                 if ln.strip() and not ln.lstrip().startswith("#")]
    for ln in inv_lines:
        parts = ln.split("\t")
        if len(parts) != 2:
            inv_rows.append({"line": ln, "status": "malformed"})
            continue
        rel, want = parts
        f = REPO / rel
        if not f.exists():
            inv_rows.append({"path": rel, "status": "missing"})
            continue
        got = sha256_of(f)
        inv_rows.append({"path": rel, "status": "ok" if got == want else "mismatch",
                         "sha256": got})
    build_txt = (PERF / "build-report.txt").read_text(encoding="utf-8")
    src_rows = []
    for ln in build_txt.splitlines():
        if ln.startswith("source_sha256 "):
            _, name, want = ln.split()
            f = PERF / name
            src_rows.append({"name": name, "exists": f.exists(),
                             "sha256_match": f.exists() and sha256_of(f) == want})
    probe_bin = REPO / "target" / "g6-perf" / "probe_perf.exe"
    perf["artifacts"] = {
        "inventory_lines": len(inv_lines),
        "inventory_ok": sum(1 for r in inv_rows if r.get("status") == "ok"),
        "inventory_not_ok": [r for r in inv_rows if r.get("status") != "ok"],
        "build_source_rows": src_rows,
        "probe_binary_still_present": probe_bin.exists(),
        "probe_binary_sha256_matches_build_report": (
            probe_bin.exists()
            and ("probe_binary_sha256=" + sha256_of(probe_bin)) in build_txt),
        "note_line_present": "probe_perf.exe is a local build product" in
                             (PERF / "artifacts-sha256.txt").read_text(encoding="utf-8"),
    }
    out["perf"] = perf

    # ---------- 4) 窗口合规 ----------
    wl = (PERF / "window-log.txt").read_text(encoding="utf-8")
    wm_rows = [ln.strip() for ln in
               (PERF / "window-memory-samples.txt").read_text(encoding="utf-8").splitlines()
               if ln.strip()]
    mem = [[int(x) for x in ln.split(",")] for ln in wm_rows]
    def log_value(key):
        vals = []
        for ln in wl.splitlines():
            if f"{key}=" in ln:
                part = ln.split(f"{key}=", 1)[1].strip()
                vals.append(part.split()[0] if " " in part else part)
        return vals
    win_env = (summary.get("environment") or {}).get("window") or {}
    window = {
        "head_revision_in_log": log_value("head_revision")[-1] if log_value("head_revision") else None,
        "product_diff_claim": log_value("product_diff_6510317f_to_head")[-1] if log_value("product_diff_6510317f_to_head") else None,
        "worktree_changes_count": log_value("window_worktree_changes_count")[-1] if log_value("window_worktree_changes_count") else None,
        "guard_policy": log_value("guard_policy")[-1] if log_value("guard_policy") else None,
        "baseline_used_mib": log_value("baseline_used_mib")[-1] if log_value("baseline_used_mib") else None,
        "peak_used_mib": log_value("peak_memory_used_mib")[-1] if log_value("peak_memory_used_mib") else None,
        "peak_delta_mib": log_value("peak_delta_mib")[-1] if log_value("peak_delta_mib") else None,
        "after_used_mib": log_value("after_used_mib")[-1] if log_value("after_used_mib") else None,
        "sample_count": log_value("sample_count")[-1] if log_value("sample_count") else None,
        "elapsed_seconds": log_value("elapsed_seconds")[-1] if log_value("elapsed_seconds") else None,
        "baseline_compute_apps_count": log_value("baseline_compute_apps_count")[-1] if log_value("baseline_compute_apps_count") else None,
        "after_compute_apps_count": log_value("after_compute_apps_count")[-1] if log_value("after_compute_apps_count") else None,
        "task_related_count_values": log_value("task_related_count"),
        "memory_samples_rows": len(mem),
        "memory_peak_recomputed": max(r[0] for r in mem),
        "memory_end": mem[-1][0],
        "memory_delta_recomputed": max(r[0] for r in mem) - mem[0][0],
        "summary_window_env": win_env,
        "git_head": git("rev-parse", "HEAD").stdout.decode().strip(),
        "git_diff_product_6510317f_to_window_head": [
            ln for ln in git("diff", "--name-only", f"{PRODUCT_BASE}..{WINDOW_HEAD}",
                             "--", "src", "tests", "Cargo.toml", "Cargo.lock")
            .stdout.decode().splitlines() if ln.strip()],
        "git_diff_product_6510317f_to_HEAD": [
            ln for ln in git("diff", "--name-only", f"{PRODUCT_BASE}..HEAD",
                             "--", "src", "tests", "Cargo.toml", "Cargo.lock")
            .stdout.decode().splitlines() if ln.strip()],
        "git_diff_product_window_head_to_HEAD": [
            ln for ln in git("diff", "--name-only", f"{WINDOW_HEAD}..HEAD",
                             "--", "src", "tests", "Cargo.toml", "Cargo.lock")
            .stdout.decode().splitlines() if ln.strip()],
        "g6_close_commit": {
            "sha": git("rev-parse", "ad53eaff").stdout.decode().strip(),
            "title": git("log", "-1", "--format=%s", "ad53eaff").stdout.decode().strip(),
            "changed_files": [ln.split("\t")[-1] for ln in
                              git("show", "--name-status", "--format=", "ad53eaff")
                              .stdout.decode("utf-8", "replace").splitlines() if ln.strip()],
        },
        "window_head_is_ancestor_of_HEAD": git("merge-base", "--is-ancestor", WINDOW_HEAD, "HEAD").returncode == 0,
    }
    window["consistency"] = {
        "head_matches_summary": window["head_revision_in_log"] == win_env.get("head_revision"),
        "baseline_matches": str(window["baseline_used_mib"]) == str(win_env.get("baseline_used_mib")),
        "peak_matches_summary": str(window["peak_used_mib"]) == str(win_env.get("peak_memory_used_mib")),
        "delta_matches_summary": str(window["peak_delta_mib"]) == str(win_env.get("peak_delta_mib")),
        "samples_match_file": int(window["sample_count"]) == len(mem),
        "peak_matches_file": int(window["peak_used_mib"]) == window["memory_peak_recomputed"],
        "end_matches_file": int(window["after_used_mib"]) == window["memory_end"],
        "product_diff_empty_at_window_head": window["git_diff_product_6510317f_to_window_head"] == [],
    }
    window["observation"] = {
        "product_diff_at_HEAD_not_empty": window["git_diff_product_6510317f_to_HEAD"] != [],
        "reason": "窗口日志的 `product_diff_6510317f_to_head=<empty>` 绑定窗口起点 HEAD 9c0a0480；"
                  "G6 收口提交 ad53eaff 在窗口之后把 host harness（test-only：`#[cfg(test)] mod perf_report`、"
                  "helper 抽取 + #[ignore] 性能测试）提交进来，因此 `git diff 6510317f..当前 HEAD -- src tests` 非空。"
                  "perf/README §1 已披露窗口时 9 项未提交改动、§8 说明 5 处 src 改动均为 test-only。",
        "disclosed_in_perf_readme": ("9 项未提交" in (PERF / "README.md").read_text(encoding="utf-8")
                                     and "test-only" in (PERF / "README.md").read_text(encoding="utf-8")),
    }
    out["window"] = window

    # ---------- 5) native 契约：提交 09896792 vs HEAD，四条措辞 ----------
    head_contract = (REPO / "docs/numerical-precision-contract.md").read_text(encoding="utf-8")
    commit_contract = git("show", "09896792:docs/numerical-precision-contract.md").stdout.decode("utf-8", "replace")

    def section(text):
        marker = "## 接触字段（native 侧）"
        i = text.find(marker)
        if i < 0:
            return None
        j = text.find("\n## ", i + len(marker))
        return text[i:j if j > 0 else len(text)].strip()

    head_sec = section(head_contract)
    commit_sec = section(commit_contract)
    commits = {
        "09896792_section_present": commit_sec is not None,
        "HEAD_section_present": head_sec is not None,
        "head_section_equals_commit_09896792": head_sec is not None and head_sec == commit_sec,
        "contract_modified_since_09896792": [
            ln for ln in git("diff", "--name-only", "09896792..HEAD",
                             "--", "docs/numerical-precision-contract.md")
            .stdout.decode().splitlines() if ln.strip()],
        "statements": {
            "native_side_diagnostic_only": "接触字段的native侧只作诊断" in (head_sec or ""),
            "pending_not_admitted": "待验收不放行" in (head_sec or ""),
            "qa_native_f64_diagnostic_only": "qa-native-f64.json" in (head_sec or "")
                                              and "只作诊断" in (head_sec or ""),
            "accuracy_still_by_upstream_f32": "接触精度仍由对上游f32的判据承担" in (head_sec or ""),
            "redline_other_fields_only": "在其它字段照旧生效" in (head_sec or "")
                                         and "接触字段的native侧不适用该红线" in (head_sec or ""),
            "no_effective_atol_rtol": "本节不写入任何生效atol或rtol" in (head_sec or ""),
            "sample_size_1case_2contacts": "1个case、2个接触点" in (head_sec or ""),
        },
        "section_text": head_sec,
    }
    out["native_contract"] = commits

    # native-framework / map 是否把诊断写成放行
    nf = (MAPD / "native-framework.md").read_text(encoding="utf-8")
    map_text = (MAPD / "map.json").read_text(encoding="utf-8")
    release_words = ("已放行", "允许放行", "native 放行", "原生放行", "视为放行", "判为通过")
    def positive_release_hits(text):
        hits = []
        for ln in text.splitlines():
            if any(w in ln for w in release_words):
                # 允许出现在「不得写/不表示/不放行」等否定语境
                if any(neg in ln for neg in ("不得", "不写", "不表示", "不放行", "不提供", "不适用", "撤回")):
                    continue
                hits.append(ln.strip())
        return hits
    out["native_release_claim_scan"] = {
        "native_framework_positive_release_lines": positive_release_hits(nf),
        "map_json_positive_release_lines": positive_release_hits(map_text),
        "framework_mentions_sample_size": "1 case / 2 接触点" in nf or "2 个接触点" in nf,
        "framework_mentions_hfield_unpairable": "不可配对" in nf and "样本不增" in nf,
        "framework_mentions_no_atol": "不写任何生效 atol/rtol" in nf,
        "e_numbers_in_framework": {
            "dist": "3.979e-7" in nf, "pos": "4.546e-4" in nf,
            "frame": "1.170e-3" in nf, "normal": "1.170e-3" in nf,
        },
    }

    # ---------- 6) partial 项交叉核对材料 ----------
    g5_readme = (REPO / "docs/development-evidence/g10-p5-gpu-kernels/README.md").read_text(encoding="utf-8")
    qa5 = load(REPO / "docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-compare-gpu.json")
    g5_receipt = load(REPO / "docs/development-evidence/g10-p5-gpu-kernels/receipt.json")
    contract = head_contract
    model_fields_record = None
    for f in sorted((REPO / ".normify" / "changes").glob("*model-fields*.json")):
        try:
            rec = load(f)
            model_fields_record = {"path": str(f.relative_to(REPO)),
                                   "status": rec.get("status"),
                                   "title": rec.get("title")}
        except Exception:
            pass
    out["partial_cross_checks"] = {
        "G6-1.2": {
            "why_partial": "HFIELD(32) 的 overflow=32 实测只在 QA 独立复核证据，G5 修复片 README 只写常量映射 → 与 G6-1.7 同为「只有 QA 独立证据」层级",
            "g5_readme_mentions_maxconpair_constant": "MJ_MAXCONPAIR" in g5_readme,
            "g5_readme_mentions_overflow_eq_32": "overflow=32" in g5_readme or "overflow = 32" in g5_readme,
            "qa5_hfield_overflow_measured": (qa5.get("custom_cases") or {}).get("hfield-maxconpair-overflow"),
            "qa5_hfield_overflow_in_qa_readme": "hfield-overflow" in
                (REPO / "docs/development-evidence/g10-p5-gpu-kernels-independent-check/README.md")
                .read_text(encoding="utf-8"),
            "g5_receipt_unmet_first": (g5_receipt.get("unmet") or [None])[0],
        },
        "G6-1.7": {
            "why_partial": "naconmax/naccdmax 截断 case 由 QA 独立复核构造并通过，G5 修复片自己未造这些 case → 与 G6-1.2 同层级",
            "qa5_capacity_cases": qa5.get("capacity_cases"),
            "g5_readme_mentions_truncation_unmet": ("naconmax" in g5_readme and "截断" in g5_readme),
            "g5_receipt_unmet": g5_receipt.get("unmet"),
        },
        "G6-2.1": {
            "why_partial": "G2–G5 已顺序写入，但 multicontact/NVRTC/primitive 仍是模块缺口（uncovered 1–3）→ 范围未闭合，不判 met",
            "g5_readme_unmet_mentions_multicontact": ("multicontact" in g5_readme and "未实装" in g5_readme),
            "uncovered_ids": [u.get("requirement_id") for u in mapj["uncovered"]],
            "model_fields_record": model_fields_record,
        },
        "G6-2.4": {
            "why_partial": "NP01/NP02 有映射与证据；NP03 native 接触字段按拍板 B 只作诊断、5% 红线不适用；NP04 只有「已记录」→ 不判 met",
            "contract_has_np_order_rule": "先核对NP01与NP02" in contract and "再执行NP03与NP04" in contract,
            "contract_says_np_not_fully_accepted": "NP01至NP04仍不整体放行" in contract,
            "contract_np03_scope": "机制落地不表示NP03全量通过" in contract,
            "contract_native_contact_redline_excluded": "接触字段的native侧不适用该红线" in contract,
            "contract_5pct_redline_present": "1.05 * E_mjwarp" in contract,
        },
    }

    # ---------- 7) 判定 ----------
    path_ok = len(out["map"]["evidence_path_missing"]) == 0
    counts_ok = (out["map"]["recounted"]["requirements"] == mapj["summary"]["requirements_total"]
                 and out["map"]["recounted"]["met"] == mapj["summary"]["met"]
                 and out["map"]["recounted"]["partial"] == mapj["summary"]["partial"]
                 and out["map"]["recounted"]["unmet"] == mapj["summary"]["unmet"]
                 and out["map"]["recounted"]["uncovered"] == mapj["summary"]["uncovered_total"])
    perf_ok = len(perf["mismatches"]) == 0
    artifacts_ok = (perf["artifacts"]["inventory_lines"] > 0
                    and perf["artifacts"]["inventory_not_ok"] == []
                    and all(r["sha256_match"] for r in perf["artifacts"]["build_source_rows"]))
    window_ok = all(window["consistency"][k] for k in window["consistency"])
    native_ok = (commits["head_section_equals_commit_09896792"]
                 and all(commits["statements"].values()))
    out["verdict"] = {
        "evidence_paths_all_exist": path_ok,
        "map_counts_match": counts_ok,
        "perf_recompute_all_match": perf_ok,
        "perf_artifacts_sha256_ok": artifacts_ok,
        "window_consistency_ok": window_ok,
        "native_contract_ok": native_ok,
        "no_positive_release_claims": (len(out["native_release_claim_scan"]["native_framework_positive_release_lines"]) == 0
                                       and len(out["native_release_claim_scan"]["map_json_positive_release_lines"]) == 0),
        "g6_1_4_measured_consistent": bool(g614.get("map_measured_consistent")),
        "partial_1_2_and_1_7_same_level": (all(r["status"] == "partial" for r in status_rows
                                                if r["id"] in ("G6-1.2", "G6-1.7"))),
        "window_observation_nonblocking": window["observation"]["product_diff_at_HEAD_not_empty"]
                                          and window["observation"]["disclosed_in_perf_readme"],
        "pass": bool(path_ok and counts_ok and perf_ok and artifacts_ok and window_ok and native_ok
                     and g614.get("map_measured_consistent")),
    }

    (EVIDENCE / "qa-g6-check.json").write_text(
        json.dumps(out, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")

    # 控制台摘要（数值为主）
    print("== map ==")
    print("  paths total", out["map"]["evidence_path_total"], "missing",
          len(out["map"]["evidence_path_missing"]))
    print("  recounted", out["map"]["recounted"], "declared", mapj["summary"])
    for row in out["map"]["partial_or_unmet"]:
        print("  ", row["id"], row["status"], "reason", row["reason_non_empty"],
              "impact", row["impact_non_empty"], "paths", row["evidence_count"])
    print("== G6-1.4 sdf-box ==")
    print(" ", json.dumps({k: v for k, v in g614.items() if k != "keys_available"}))
    print("== perf recompute ==")
    print("  host rows", sum(1 for r in perf["recompute"] if r["kind"] == "host"),
          "device rows", sum(1 for r in perf["recompute"] if r["kind"] == "device"),
          "upstream rows", len(up_rows))
    print("  mismatches", len(perf["mismatches"]))
    for r in perf["recompute"]:
        if r["kind"] == "host":
            print("   H %-16s n=%d median %.1f vs %.1f p95nr %.1f vs %.1f match=%s cps %.1f"
                  % (r["case"], r["samples"], r["median_stored"], r["median_recomputed_avg2"],
                     r["p95_stored"], r["p95_recomputed_nr"], r["summary_match"],
                     r["contacts_per_s_stored"] or -1))
        else:
            print("   D %-16s k=%d/%.1f e=%d/%.1f match=%s meta=%s"
                  % (r["case"], r["kernel_samples"], r["kernel_median_stored"],
                     r["e2e_samples"], r["e2e_median_stored"], r["summary_match"],
                     r.get("meta_match")))
    for r in up_rows:
        print("   U %-16s n=%d median %.1f vs %.1f p95nr %.1f vs %.1f json_lin %.1f match=%s"
              % (r["case"], r["samples"], r["median_stored_summary"], r["median_recomputed"],
                 r["p95_stored_summary"], r["p95_nearest_rank"], r["p95_linear"],
                 r["summary_match"]))
    print("  notes_present", perf["notes_present"])
    print("== window ==")
    print(" ", json.dumps({k: window[k] for k in
                           ("head_revision_in_log", "product_diff_claim", "baseline_used_mib",
                            "peak_used_mib", "peak_delta_mib", "after_used_mib", "sample_count",
                            "elapsed_seconds", "guard_policy", "baseline_compute_apps_count",
                            "after_compute_apps_count", "memory_samples_rows",
                            "memory_peak_recomputed", "memory_end")}))
    print("  consistency", json.dumps(window["consistency"]))
    print("  diff window_head", window["git_diff_product_6510317f_to_window_head"])
    print("  diff HEAD", window["git_diff_product_6510317f_to_HEAD"])
    print("  diff window_head..HEAD", window["git_diff_product_window_head_to_HEAD"])
    print("== native contract ==")
    print("  present", commits["09896792_section_present"], "HEAD same", commits["head_section_equals_commit_09896792"])
    print("  statements", json.dumps(commits["statements"]))
    print("== native release scan ==")
    print(" ", json.dumps({k: v for k, v in out["native_release_claim_scan"].items()}))
    print("== verdict ==")
    print(" ", json.dumps(out["verdict"]))
    print("report:", EVIDENCE / "qa-g6-check.json")
    return 0 if out["verdict"]["pass"] else 1


if __name__ == "__main__":
    sys.exit(main())
