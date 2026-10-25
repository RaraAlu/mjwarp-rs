"""QA judge: delivery evidence integrity (S7b2 shading slice).

Checks, independently:
  * every evidence file's bytes equal the committed git blob at f228099;
  * the delivered numbers in README/receipt against the logs (PTX bytes/sha, histograms,
    depth_seg_hash/rgb_hash, unit 532/0, s7b1 6/6, s6b 5/5, capture facts);
  * `git diff 14ee028..f228099` stays inside the S7b2 + companion task scopes and leaves the
    S2-S6 evidence, fixtures, `src/render/bvh/**`, `render_math.cu` and `launch.rs` untouched.
"""

from __future__ import annotations

import hashlib
import json
import pathlib
import re
import subprocess

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
EVIDENCE = "docs/development-evidence/render-bvh/s7b2-shading"
RECEIPT = ROOT / EVIDENCE / "receipt.json"
README = ROOT / EVIDENCE / "README.md"
BASE = "14ee028"
TARGET = "f228099"


def git(*args: str) -> bytes:
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, check=True).stdout


def git_text(*args: str) -> str:
    return git(*args).decode("utf-8", "replace")


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def blob(path: str) -> bytes:
    return git("show", f"{TARGET}:{path}")


def main() -> int:
    report: dict = {}
    receipt = json.loads(RECEIPT.read_text(encoding="utf-8"))
    readme = README.read_text(encoding="utf-8")

    # 1. evidence files vs committed blobs
    evidence_files = sorted(
        str(path.relative_to(ROOT)).replace("\\", "/")
        for path in (ROOT / EVIDENCE).iterdir()
        if path.is_file()
    )
    report["evidence_files"] = []
    for path in evidence_files:
        working = (ROOT / path).read_bytes()
        committed = blob(path)
        report["evidence_files"].append(
            {
                "path": path.rsplit("/", 1)[-1],
                "bytes": len(working),
                "sha256": sha256(working),
                "working_equals_committed_blob": working == committed,
            }
        )
    report["receipt_evidence_list_matches"] = sorted(
        receipt["evidence_files"]
    ) == sorted(item["path"] for item in report["evidence_files"])

    # 2. README/receipt numeric claims vs logs
    gpu_log = (ROOT / EVIDENCE / "gpu-run.log").read_bytes().decode("utf-16")
    s7b1_log = (ROOT / EVIDENCE / "s7b1-regression.log").read_bytes().decode("utf-16")
    s6b_log = (ROOT / EVIDENCE / "s6b-regression.log").read_bytes().decode("utf-16")
    unit_log = (ROOT / EVIDENCE / "unit-summary.log").read_bytes().decode("utf-16", "replace")
    capture_log = (ROOT / EVIDENCE / "capture.log").read_bytes().decode("utf-16")
    report["logs"] = {
        "gpu_ptx_line": re.search(r"source_sha256=([0-9a-f]{64}) ptx_sha256=([0-9a-f]{64}) ptx_bytes=(\d+)", gpu_log).groups(),
        "gpu_rgb_histograms": re.findall(r"cam\d: pixels=\d+ mismatched=\d+ max_byte_diff=\d+ beyond_1_lsb=\d+ first=None histogram\(0,1,2,3,>3\)=\[([^\]]+)\]", gpu_log),
        "gpu_hash_line": re.search(r"depth_seg_hash=([0-9a-f]+) rgb_hash=([0-9a-f]+)", gpu_log).groups(),
        "s7b1_passed": re.search(r"test result: ok\. (\d+) passed; (\d+) failed", s7b1_log).groups(),
        "s6b_passed": re.search(r"test result: ok\. (\d+) passed; (\d+) failed", s6b_log).groups(),
        "unit_line": (
            {
                "binaries": len(re.findall(r"test result: ok\.", unit_log)),
                "passed": sum(int(value) for value in re.findall(r"test result: ok\. (\d+) passed", unit_log)),
                "failed": sum(int(value) for value in re.findall(r"(\d+) failed", unit_log)),
            }
            if re.search(r"test result: ok\.", unit_log)
            else None
        ),
        "capture_div": re.search(r"ptx div_rn=(\d+) div_approx=(\d+)", capture_log).groups(),
    }
    report["claims"] = {
        "receipt_ptx_bytes": receipt["precision_gate"]["ptx_bytes"],
        "receipt_ptx_sha256": receipt["precision_gate"]["ptx_sha256"],
        "receipt_source_sha256": receipt["precision_gate"]["source_sha256"],
        "readme_ptx_bytes_195594": "195,594 B" in readme,
        "readme_rgb_hash": "ed50f750936c275" in readme,
        "readme_depth_seg_hash": "03efa95416bbd008" in readme,
        "gpu_log_rgb_hash": report["logs"]["gpu_hash_line"][1],
        "readme_rgb_hash_matches_log": report["logs"]["gpu_hash_line"][1] in readme,
        "receipt_depth_seg_hash": receipt["gpu"]["run_determinism"]["depth_seg_hash"],
        "receipt_rgb_hash": receipt["gpu"]["run_determinism"]["rgb_hash"],
    }
    # actual lines of evidence
    report["readme_lines"] = len(readme.splitlines())
    report["receipt_lines"] = len(RECEIPT.read_text(encoding="utf-8").splitlines())

    # 3. scope of the diff
    changed = git_text("diff", "--name-only", BASE, TARGET).splitlines()
    task_scope = json.loads(
        (ROOT / ".normify/changes/2026-10-25-render-bvh-s7b2-shading.json").read_text(encoding="utf-8")
    )["schedule"]["files"]
    companion_scope = json.loads(
        (ROOT / ".normify/changes/2026-10-25-render-bvh-texture-handles.json").read_text(encoding="utf-8")
    )["schedule"]["files"]
    allowed = set(task_scope) | set(companion_scope) | {
        ".normify/changes/2026-10-25-render-bvh-s7b2-shading.json",
        ".normify/changes/2026-10-25-render-bvh-texture-handles.json",
        ".normify/modules/mjwarp/render.md",
    }
    allowed |= {path for path in changed if path.startswith(".normify/runs/")}
    out_of_scope = [
        path
        for path in changed
        if path not in allowed and not any(path.startswith(scope.rstrip("/") + "/") for scope in allowed)
    ]
    report["diff"] = {
        "changed_count": len(changed),
        "out_of_scope": out_of_scope,
        "scope_ok": not out_of_scope,
    }
    report["unchanged_checks"] = {
        "s2_s6_evidence_changed": [
            path
            for path in changed
            if path.startswith("docs/development-evidence/render-bvh/s")
            and path.split("/")[3].startswith(("s0", "s1", "s2", "s3", "s4", "s5", "s6"))
        ],
        "s2_s6_fixtures_changed": [
            path
            for path in changed
            if path.startswith("fixtures/") and re.search(r"/s[0-6]/", path)
        ],
        "render_bvh_src_changed": [path for path in changed if path.startswith("src/render/bvh/")],
        "render_math_cu_changed": [
            path for path in changed if path == "src/render/kernel/cuda/render_math.cu"
        ],
        "launch_rs_changed": [
            path for path in changed if path == "src/render/kernel/launch.rs"
        ],
        "launch_rs_lines_at_target": len(blob("src/render/kernel/launch.rs").splitlines()),
        "scene_cu_sha256_at_target": sha256(blob("src/render/bvh/cuda/scene.cu")),
    }
    report["receipt_claims_vs_independent"] = {
        "kernel_source_bytes": len(
            b"".join(
                blob(path)
                for path in (
                    "src/render/bvh/cuda/scene.cu",
                    "src/render/kernel/cuda/render_math.cu",
                    "src/render/kernel/cuda/render_common.cu",
                    "src/render/kernel/cuda/render_shade.cu",
                    "src/render/kernel/cuda/render_megakernel.cu",
                )
            )
        ),
        "kernel_source_lines": sum(
            len(blob(path).splitlines())
            for path in (
                "src/render/bvh/cuda/scene.cu",
                "src/render/kernel/cuda/render_math.cu",
                "src/render/kernel/cuda/render_common.cu",
                "src/render/kernel/cuda/render_shade.cu",
                "src/render/kernel/cuda/render_megakernel.cu",
            )
        ),
        "receipt_kernel_source_bytes": receipt["abi"]["kernel_source_bytes"],
        "receipt_kernel_source_lines": receipt["abi"]["kernel_source_lines"],
        "fixture_sha256": {
            "rgb.json": sha256((ROOT / "fixtures/render-bvh/scene-a/s7b/rgb.json").read_bytes()),
            "rgb_fast_math_false.json": sha256(
                (ROOT / "fixtures/render-bvh/scene-a/s7b/rgb_fast_math_false.json").read_bytes()
            ),
        },
        "receipt_fixture_sha256": {
            "rgb.json": receipt["references"]["s0b_fast_math"]["sha256"],
            "rgb_fast_math_false.json": receipt["references"]["same_recipe_fast_math_false"]["sha256"],
        },
        "capture_script_sha256": sha256(
            (ROOT / "scripts/authoring/render-bvh-s7b/capture_rgb_fast_math_false.py")
            .read_bytes()
            .replace(b"\r\n", b"\n")
        ),
        "receipt_capture_script_sha256": receipt["references"]["same_recipe_fast_math_false"][
            "capture_script_sha256"
        ],
    }

    out = ROOT / "target/qa-s7b2/out/judge_evidence.json"
    out.write_text(json.dumps(report, indent=1), encoding="utf-8")
    print(json.dumps(report, indent=1)[:7000])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
