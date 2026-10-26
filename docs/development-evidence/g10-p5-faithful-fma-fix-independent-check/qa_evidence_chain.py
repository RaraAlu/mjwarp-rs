# -*- coding: utf-8 -*-
"""QA check: delivery evidence chain and fact-checking for the A fix (r2).

Verifies the pre/post delivery logs against my own runs, the host-gnu/host-msvc claim,
the evidence-chain citations, and that B-scratch numbers have a traceable basis.
Writes qa-evidence-chain.json.
"""
import json
import re
import subprocess
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
TASK = REPO / "docs/development-evidence/g10-p5-faithful-fma-fix-independent-check"
EV = REPO / "docs/development-evidence/faithful-fma-fix"
LOGS = EV / "logs"
REV = "50a0acd1"


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace") if path.is_file() else ""


def judge_lines(path: Path) -> list:
    return [line.strip() for line in read(path).splitlines() if line.strip().startswith("[判据]")]


def frame_abs(lines: list, case: str):
    for line in lines:
        if line.startswith("[判据]") and case in line:
            match = re.search(r"frame 位等分量 (\d+)/(\d+) max abs ([0-9.eE+-]+)", line)
            if match:
                return {"bit_equal": [int(match.group(1)), int(match.group(2))],
                        "max_abs": float(match.group(3))}
    return None


def result_lines(path: Path) -> list:
    return [line.strip() for line in read(path).splitlines() if "test result:" in line]


result = {"delivery": {}, "my_runs": {}, "chain": {}}
for mode in ("gnu-default", "msvc", "gnu-fma"):
    pre = judge_lines(LOGS / f"pre-fix-case-evidence-{mode}.log")
    post = judge_lines(LOGS / f"post-fix-case-evidence-{mode}.log")
    result["delivery"][mode] = {
        "pre_judge_lines": pre,
        "post_judge_lines": post,
        "pre_ellipsoid_frame": frame_abs(pre, "ellipsoid-box"),
        "post_ellipsoid_frame": frame_abs(post, "ellipsoid-box"),
        "pre_test_result": result_lines(LOGS / f"pre-fix-case-evidence-{mode}.log"),
        "post_test_result": result_lines(LOGS / f"post-fix-case-evidence-{mode}.log"),
    }

# my own pre-fix runs (parent copy), independent of the delivery pre logs
for mode in ("gnu-default", "msvc"):
    my_pre_log = TASK / "logs" / f"pre-fix-case-evidence-{mode}-qa.log"
    if my_pre_log.is_file():
        lines = judge_lines(my_pre_log)
        result.setdefault("my_pre", {})[mode] = {
            "judge_lines": lines,
            "ellipsoid_frame": frame_abs(lines, "ellipsoid-box"),
            "assert_test_result": result_lines(TASK / "logs" / f"pre-fix-case-evidence-{mode}-assert-qa.log"),
        }

# my own runs (three-build matrix logs, if the build job has finished)
for mode in ("gnu-default", "msvc", "gnu-fma"):
    my_log = TASK / "logs" / f"{mode}-case-evidence.log"
    if my_log.is_file():
        my_lines = judge_lines(my_log)
        delivery_post = result["delivery"][mode]["post_judge_lines"]
        result["my_runs"][mode] = {
            "judge_lines": my_lines,
            "matches_delivery_post": my_lines == delivery_post,
            "ellipsoid_frame": frame_abs(my_lines, "ellipsoid-box"),
        }

# chain citations must exist
citations = {
    "frame_diagnosis": REPO / "docs/development-evidence/case-evidence-frame-diagnosis/README.md",
    "gpu_reference_capture": REPO / "docs/development-evidence/g10-p5-g2-gpu-reference-capture/README.md",
    "gjk_epa": REPO / "docs/development-evidence/g10-p5-gjk-epa/README.md",
    "exact_fma_relmat": REPO / "docs/development-evidence/exact-f32-fma-relmat/README.md",
    "faithful_impact": REPO / "docs/development-evidence/faithful-impact-measurement/README.md",
}
result["chain"]["cited_evidence"] = {
    name: {"exists": path.is_file(), "bytes": path.stat().st_size if path.is_file() else 0}
    for name, path in citations.items()}

# B-scratch claim: numbers cited but no scratch log in the committed evidence
b_mention = "实测不可行" in read(EV / "README.md")
b_logs = sorted(p.name for p in LOGS.glob("*")) if LOGS.is_dir() else []
b_scratch_logs = [name for name in b_logs if name.lower().startswith(("b-", "scratch", "dry"))]
result["chain"]["b_scratch"] = {
    "mentioned_in_readme": b_mention,
    "scratch_log_in_evidence": b_scratch_logs,
    "evidence_log_files": b_logs,
}
# cross-check B-claim numbers with my earlier independent CPU-vs-GPU reference diff
capture_diff = REPO / "docs/development-evidence/g10-p5-g2-gpu-reference-capture-independent-check/qa-new-old-diff.json"
if capture_diff.is_file():
    data = json.loads(capture_diff.read_text(encoding="utf-8"))
    result["chain"]["b_claim_vs_my_cpu_gpu_reference_diff"] = {
        case: data["cases"][case]["float_fields"]["frame"]["max_abs"]
        for case in ("ellipsoid-box", "mesh-mesh", "convex-mesh", "degenerate-mesh")
        if case in data["cases"]}
    result["chain"]["b_claim_mesh_frame_matches"] = (
        abs(result["chain"]["b_claim_vs_my_cpu_gpu_reference_diff"].get("mesh-mesh", 0)
            - 5.364418e-7) < 1e-12)
    result["chain"]["b_claim_ellipsoid_frame_matches"] = (
        abs(result["chain"]["b_claim_vs_my_cpu_gpu_reference_diff"].get("ellipsoid-box", 0)
            - 8.192876e-7) < 1e-12)

# fact-check: host-gnu pre was green by coupling; host-msvc pre was red
pre_gnu = result["delivery"]["gnu-default"]
pre_msvc = result["delivery"]["msvc"]
my_pre_gnu = result.get("my_pre", {}).get("gnu-default", {})
my_pre_msvc = result.get("my_pre", {}).get("msvc", {})
result["facts"] = {
    "pre_gnu_green": any("ok." in line and "0 failed" in line for line in pre_gnu["pre_test_result"]),
    "pre_gnu_frame_3_838112e_7": bool(pre_gnu["pre_ellipsoid_frame"]
                                      and abs(pre_gnu["pre_ellipsoid_frame"]["max_abs"] - 3.838112e-7) < 1e-12),
    "pre_msvc_red": any("FAILED" in line for line in pre_msvc["pre_test_result"]),
    "pre_msvc_frame_6_351978e_7": bool(pre_msvc["pre_ellipsoid_frame"]
                                      and abs(pre_msvc["pre_ellipsoid_frame"]["max_abs"] - 6.351978e-7) < 1e-12),
    "post_all_green": all(any("ok." in line and "0 failed" in line
                              for line in result["delivery"][mode]["post_test_result"])
                          for mode in ("gnu-default", "msvc", "gnu-fma")),
    "README_claims_host_gnu_coupling": ("也是绿的" in read(EV / "README.md")) and ("偶合" in read(EV / "README.md")),
    "README_claims_full_g01_not_run": "未执行" in read(EV / "README.md"),
}
if my_pre_gnu:
    result["facts"]["my_pre_gnu_green"] = any(
        "ok." in line and "0 failed" in line for line in my_pre_gnu["assert_test_result"] + my_pre_gnu["judge_lines"])
    result["facts"]["my_pre_gnu_frame_3_838112e_7"] = bool(
        my_pre_gnu["ellipsoid_frame"]
        and abs(my_pre_gnu["ellipsoid_frame"]["max_abs"] - 3.838112e-7) < 1e-12)
if my_pre_msvc:
    result["facts"]["my_pre_msvc_red"] = any(
        "FAILED" in line for line in my_pre_msvc["assert_test_result"] + my_pre_msvc["judge_lines"])
    result["facts"]["my_pre_msvc_frame_6_351978e_7"] = bool(
        my_pre_msvc["ellipsoid_frame"]
        and abs(my_pre_msvc["ellipsoid_frame"]["max_abs"] - 6.351978e-7) < 1e-12)
result["facts"]["delivery_post_equals_my_post"] = all(
    entry.get("matches_delivery_post") for entry in result["my_runs"].values()) if result["my_runs"] else None

# gate/commit bookkeeping
runs = REPO / ".normify/runs/2026-10-25-g10-p5-faithful-fma-fix-r2"
unit = json.loads((runs / "unit.json").read_text(encoding="utf-8")) if (runs / "unit.json").is_file() else {}
receipt = json.loads((EV / "receipt.json").read_text(encoding="utf-8")) if (EV / "receipt.json").is_file() else {}
gate_json = TASK / "qa-default-gate.json"
default_gate = json.loads(gate_json.read_text(encoding="utf-8")) if gate_json.is_file() else {}
three_build = TASK / "qa-three-build.json"
three = json.loads(three_build.read_text(encoding="utf-8")) if three_build.is_file() else {}
result["gate"] = {
    "runs_unit": unit.get("tests"),
    "gate_exit_in_runs": unit.get("exit"),
    "receipt_task_id": receipt.get("task_id"),
    "receipt_head_before": receipt.get("head_before"),
    "gpu_window_files_in_evidence": [p.name for p in LOGS.glob("*window*")],
    "g01_full_logs_in_evidence": [p.name for p in LOGS.glob("*g01*")],
}
result["g01_directed"] = {
    "host_gnu_my_full_default_gate": {
        "passed": default_gate.get("passed"), "failed": default_gate.get("failed"),
        "ignored": default_gate.get("ignored"), "all_ok": default_gate.get("all_ok"),
    },
    "host_msvc_my_case_evidence": {
        "test_result": (three.get("modes", {}).get("msvc", {}) or {}).get("assert_test_result"),
    },
    "pre_gnu_my_run": result["facts"].get("my_pre_gnu_green"),
    "pre_msvc_my_run": result["facts"].get("my_pre_msvc_red"),
    "full_g01_run": bool([p.name for p in LOGS.glob("*g01*")]),
    "delivery_claims_no_full_g01": "未执行" in read(EV / "README.md"),
}

# delivery src.patch must equal the actual source diff of the r2 commit
src_patch = (LOGS / "src.patch").read_text(encoding="utf-8")
git_diff = subprocess.run(["git", "-C", str(REPO), "diff", f"{REV}^..{REV}", "--", "src"],
                          capture_output=True).stdout.decode("utf-8")
normalize = lambda text: "\n".join(line.rstrip() for line in text.replace("\r\n", "\n").splitlines()
                                   if line.strip() or True)
result["src_patch"] = {
    "evidence_bytes": len(src_patch.encode("utf-8")),
    "actual_diff_bytes": len(git_diff.encode("utf-8")),
    "matches_actual_src_diff": normalize(src_patch).strip() == normalize(git_diff).strip(),
    "touches_only_two_src_files": all(
        path in src_patch for path in ("src/physics/collision/gjk/polytope.rs",
                                       "src/render/kernel/megakernel_arrays.rs")),
}
result["chain_completeness"] = {
    "readme_mentions_gnu_fmaf_noncompliant": "IEEE 单次舍入" in read(EV / "README.md"),
    "readme_mentions_cpu_reference_nonfused": ("CPU 后端" in read(EV / "README.md"))
    and ("非融合" in read(EV / "README.md")),
    "readme_mentions_c_d_not_relaxed": ("C（容差重推）" in read(EV / "README.md"))
    or ("C" in read(EV / "README.md") and "不放宽" in read(EV / "README.md")),
    "readme_mentions_gpu_zero": "GPU 窗口 0" in read(EV / "README.md"),
    "all_cited_evidence_exists": all(item["exists"] for item in result["chain"]["cited_evidence"].values()),
}
result["delivery_log_provenance"] = {
    "post_gnu_compiled_main_tree": "Compiling mjwarp-rs v0.1.0 (D:\\Rust\\mjwarp-rs)" in read(
        LOGS / "post-fix-case-evidence-gnu-default.log"),
    "pre_gnu_reused_cache_no_compile_line": "Compiling mjwarp-rs" not in read(
        LOGS / "pre-fix-case-evidence-gnu-default.log"),
}
(TASK / "qa-evidence-chain.json").write_text(
    json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps({
    "my_runs_match": {mode: entry.get("matches_delivery_post") for mode, entry in result["my_runs"].items()},
    "facts": result["facts"],
    "b_scratch": result["chain"]["b_scratch"],
    "b_claims_vs_my_diff": result["chain"].get("b_claim_vs_my_cpu_gpu_reference_diff"),
    "gate": result["gate"],
}, ensure_ascii=False, indent=2))
