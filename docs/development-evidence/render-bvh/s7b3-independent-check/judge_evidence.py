"""QA judge: S7b3 delivery evidence integrity + companion-fix scope.

Checks:
  * evidence files equal the committed git blobs at the reviewed commit;
  * PTX / host-module / launch / scene.cu / fixture numbers in the receipt against QA's own
    recomputation and the actual files;
  * gpu-run.log, s7b1/s7b2/s6b regression logs and unit-summary.log against the receipt claims;
  * `git diff 624948b..52c22b7` stays inside the S7b3 task scope and leaves S2-S6 evidence,
    fixtures, `src/render/bvh/**` and `tests/render_kernel_rgb.rs` untouched;
  * the S7b2 assertion fix is a separate companion commit (`3ab52cf`) touching only that test.
"""

from __future__ import annotations

import hashlib
import json
import pathlib
import re
import subprocess

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
EVIDENCE = "docs/development-evidence/render-bvh/s7b3-flags"
BASE = "624948b"
TARGET = "52c22b7"
COMPANION = "3ab52cf"


def git(*args: str) -> bytes:
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, check=True).stdout


def git_text(*args: str) -> str:
    return git(*args).decode("utf-8", "replace")


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def main() -> int:
    report: dict = {}
    receipt = json.loads((ROOT / EVIDENCE / "receipt.json").read_text(encoding="utf-8"))

    report["evidence_files"] = []
    for path in sorted(p for p in (ROOT / EVIDENCE).iterdir() if p.is_file()):
        rel = str(path.relative_to(ROOT)).replace("\\", "/")
        working = path.read_bytes()
        committed = git("show", f"{TARGET}:{rel}")
        report["evidence_files"].append(
            {
                "path": path.name,
                "bytes": len(working),
                "sha256": sha256(working),
                "equals_committed_blob": working == committed,
            }
        )
    report["receipt_evidence_list_matches"] = sorted(receipt["evidence_files"]) == sorted(
        entry["path"] for entry in report["evidence_files"]
    )

    # PTX facts from QA harness
    harness = json.loads((ROOT / "target/qa-s7b3/out/harness_base.json").read_text(encoding="utf-8"))
    gate_entries = {entry["entry"]: entry for entry in receipt["precision_gate"]["entries"]}
    report["ptx"] = {
        "megakernel": {
            "qa_bytes": harness["ptx"]["bytes"],
            "qa_source_sha256": harness["ptx"]["source_sha256"],
            "qa_ptx_sha256": harness["ptx"]["ptx_sha256"],
            "receipt_bytes": gate_entries["render_megakernel"]["ptx_bytes"],
            "receipt_source_sha256": gate_entries["render_megakernel"]["source_sha256"],
            "receipt_ptx_sha256": gate_entries["render_megakernel"]["ptx_sha256"],
            "entries": harness["ptx"]["entries"],
            "f64_lines": harness["ptx"]["f64_lines"],
        },
        "aa_resolve": {
            "qa_bytes": harness["aa_resolve"]["bytes"],
            "qa_source_sha256": harness["aa_resolve"]["source_sha256"],
            "qa_ptx_sha256": harness["aa_resolve"]["ptx_sha256"],
            "receipt_bytes": gate_entries["aa_resolve"]["ptx_bytes"],
            "receipt_source_sha256": gate_entries["aa_resolve"]["source_sha256"],
            "receipt_ptx_sha256": gate_entries["aa_resolve"]["ptx_sha256"],
            "entry_present": harness["aa_resolve"]["entry_present"],
        },
    }
    report["ptx"]["all_match"] = all(
        report["ptx"][name]["qa_bytes"] == report["ptx"][name]["receipt_bytes"]
        and report["ptx"][name]["qa_source_sha256"] == report["ptx"][name]["receipt_source_sha256"]
        and report["ptx"][name]["qa_ptx_sha256"] == report["ptx"][name]["receipt_ptx_sha256"]
        for name in ("megakernel", "aa_resolve")
    )

    # host module facts claimed in the receipt (lines / bytes / sha256) vs committed files
    modules = {
        "megakernel_rs": "src/render/kernel/megakernel.rs",
        "megakernel_aa_rs": "src/render/kernel/megakernel_aa.rs",
        "megakernel_arrays_rs": "src/render/kernel/megakernel_arrays.rs",
    }
    report["host_modules"] = {}
    for key, path in modules.items():
        data = (ROOT / path).read_bytes()
        modules_spec = receipt["precision_gate"]["host_modules"][key]
        report["host_modules"][key] = {
            "lines": len(data.decode("utf-8").splitlines()),
            "bytes": len(data),
            "sha256": sha256(data),
            "receipt": modules_spec,
            "bytes_and_sha_match": len(data) == modules_spec["bytes"]
            and sha256(data) == modules_spec["sha256"],
            "receipt_lines": modules_spec["lines"],
            "lines_match": len(data.decode("utf-8").splitlines()) == modules_spec["lines"],
        }
    report["launch_rs"] = {
        "lines": len((ROOT / "src/render/kernel/launch.rs").read_text(encoding="utf-8").splitlines()),
    }
    report["scene_cu_sha256"] = sha256((ROOT / "src/render/bvh/cuda/scene.cu").read_bytes())

    # logs vs receipt
    def text(name: str) -> str:
        return (ROOT / EVIDENCE / name).read_bytes().decode("utf-16", "replace")

    gpu = text("gpu-run.log")
    s7b1 = text("s7b1-regression.log")
    s7b2 = text("s7b2-regression.log")
    s6b = text("s6b-regression.log")
    unit = text("unit-summary.log")
    capture = text("capture.log")
    unit_results = re.findall(r"test result: (ok|FAILED)\. (\d+) passed; (\d+) failed", unit)
    report["logs"] = {
        "gpu_passed_failed": re.search(r"test result: ok\. (\d+) passed; (\d+) failed", gpu).groups(),
        "gpu_elapsed": re.search(r"finished in ([0-9.]+)s", gpu).group(1),
        "s7b1_passed_failed": re.search(r"test result: ok\. (\d+) passed; (\d+) failed", s7b1).groups(),
        "s7b2_result": re.search(r"test result: (ok|FAILED)\. (\d+) passed; (\d+) failed", s7b2).groups(),
        "s7b2_failed_test": re.search(r"(test [a-z0-9_]+) \.\.\. FAILED", s7b2).group(1)
        if re.search(r"(test [a-z0-9_]+) \.\.\. FAILED", s7b2)
        else None,
        "s6b_passed_failed": re.search(r"test result: ok\. (\d+) passed; (\d+) failed", s6b).groups(),
        "unit_binaries": len(unit_results),
        "unit_passed": sum(int(p) for _, p, _ in unit_results),
        "unit_failed": sum(int(f) for _, _, f in unit_results),
        "capture_variant_shas": re.findall(r"^(\w+): rgb_sha=([0-9a-f]{64}) changed=\[([^\]]*)\]", capture, re.M),
        "capture_determinism_lines": len(re.findall(r"determ", capture)),
    }
    report["receipt_claims"] = {
        "s7b3_suite_passed": receipt["gpu"]["s7b3_suite"]["passed"],
        "s7b3_elapsed_s": receipt["gpu"]["s7b3_suite"]["elapsed_s"],
        "s7b2_regression_passed": receipt["gpu"]["s7b2_regression"]["passed"],
        "s7b2_regression_failed": receipt["gpu"]["s7b2_regression"]["failed"],
        "unit_tests_passed": receipt["gates"]["unit"]["tests_passed"],
        "unit_binaries_ok": receipt["gates"]["unit"]["test_binaries_ok"],
    }

    # fixture sha claims vs files (index file is covered in judge_fixtures; here the base refs)
    report["base_fixture_refs"] = {
        "rgb_fast_math_false_sha256": sha256(
            (ROOT / "fixtures/render-bvh/scene-a/s7b/rgb_fast_math_false.json").read_bytes()
        ),
        "depth_seg_fast_math_false_sha256": sha256(
            (ROOT / "fixtures/render-bvh/scene-a/s7b/depth_seg_fast_math_false.json").read_bytes()
        ),
    }

    # diff scope
    changed = git_text("diff", "--name-only", BASE, TARGET).splitlines()
    task_scope = json.loads(
        (ROOT / ".normify/changes/2026-10-25-render-bvh-s7b3-flags.json").read_text(encoding="utf-8")
    )["schedule"]["files"]
    allowed = set(task_scope) | {
        ".normify/changes/2026-10-25-render-bvh-s7b3-flags.json",
        ".normify/modules/mjwarp/render.md",
    }
    allowed |= {path for path in changed if path.startswith(".normify/runs/")}
    out_of_scope = [
        path
        for path in changed
        if path not in allowed and not any(path.startswith(scope.rstrip("/") + "/") for scope in allowed)
    ]
    report["diff"] = {
        "range": f"{BASE}..{TARGET}",
        "changed_count": len(changed),
        "out_of_scope": out_of_scope,
        "scope_ok": not out_of_scope,
        "s2_s6_evidence_or_fixtures_changed": [
            path
            for path in changed
            if (
                path.startswith("docs/development-evidence/render-bvh/s")
                and path.split("/")[3].startswith(("s0", "s1", "s2", "s3", "s4", "s5", "s6"))
            )
            or (path.startswith("fixtures/") and re.search(r"/s[0-6]/", path))
        ],
        "render_bvh_src_changed": [path for path in changed if path.startswith("src/render/bvh/")],
        "s7b2_reject_test_changed_in_target": "tests/render_kernel_rgb.rs" in changed,
    }
    companion_changed = git_text("diff", "--name-only", f"{TARGET}", COMPANION).splitlines()
    report["companion_fix"] = {
        "commit": COMPANION,
        "changed": companion_changed,
        "only_s7b2_test": [
            path for path in companion_changed if not path.startswith(".normify/")
        ]
        == ["tests/render_kernel_rgb.rs"],
        "fix_trailer": "Normify-Fixes: 2026-10-25-render-bvh-s7b2-shading" in git_text(
            "log", "-1", "--format=%B", COMPANION
        ),
    }
    # the fixed test now asserts 3 rejection cases (AA removed from the list)
    current_test = (ROOT / "tests/render_kernel_rgb.rs").read_text(encoding="utf-8")
    report["companion_fix"]["aa_case_removed"] = '"samples_per_pixel=2"' not in current_test
    report["companion_fix"]["remaining_rejections"] = re.findall(
        r'"(use_shadows|use_precomputed_rays=false|render_skybox)"', current_test
    )

    out = ROOT / "target/qa-s7b3/out/judge_evidence.json"
    out.write_text(json.dumps(report, indent=1), encoding="utf-8")
    print(json.dumps(report, indent=1))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
