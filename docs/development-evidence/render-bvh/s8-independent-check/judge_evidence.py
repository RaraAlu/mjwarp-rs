"""QA judge: S8 delivery evidence integrity, honesty of the r01-p8 closure, scope.

Checks:
  * evidence files equal the committed git blobs; logs vs receipt claims (suite counts/times,
    host test, regression counts, camlight/perturbation numbers, lifecycle growth);
  * input fixture + authoring script size/sha, camlight model counts, referenced provenance
    files exist;
  * `git diff 15cb6f7..d8bc9d8` scope (only S8 paths; **no product src changes**), size ratchet
    (tests 499 / support 496 lines < 500);
  * r01-p8 ten-item mapping: every trace path exists; statements match the slice evidence files
    (spot-checked keywords); unmet list covers nworld>1, shadow/skybox/splat/flex/hfield/SDF/
    multi-camera, Linux/T4, upstream equivalence, performance, higher spp;
  * literal-vs-claimed scan for the four forbidden phrases (must not be *asserted*; report any
    literal occurrence, even negated);
  * known non-blocking item: cam0 vs host quat_to_mat 1 ULP / 2 of 9 components.
"""

from __future__ import annotations

import hashlib
import json
import pathlib
import re
import subprocess
import struct

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
EVIDENCE = "docs/development-evidence/render-bvh/s8-e2e"
BASE = "15cb6f7"
TARGET = "d8bc9d8"


def git(*args: str) -> bytes:
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, check=True).stdout


def git_text(*args: str) -> str:
    return git(*args).decode("utf-8", "replace")


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def f32(bits: int) -> float:
    return struct.unpack("f", struct.pack("I", bits))[0]


def main() -> int:
    report: dict = {}
    receipt = json.loads((ROOT / EVIDENCE / "receipt.json").read_text(encoding="utf-8"))
    readme = (ROOT / EVIDENCE / "README.md").read_text(encoding="utf-8")

    report["evidence_files"] = []
    for path in sorted(p for p in (ROOT / EVIDENCE).iterdir() if p.is_file()):
        rel = str(path.relative_to(ROOT)).replace("\\", "/")
        working = path.read_bytes()
        committed = git("show", f"{TARGET}:{rel}")
        text_compare = working.replace(b"\r\n", b"\n") == committed.replace(b"\r\n", b"\n")
        report["evidence_files"].append(
            {
                "path": path.name,
                "bytes": len(working),
                "sha256": sha256(working),
                "bytes_committed": len(committed),
                "equals_committed_blob": working == committed,
                "equals_committed_blob_after_eol_normalization": text_compare,
                "eol_artifact": working != committed and text_compare,
            }
        )
    report["receipt_evidence_list_matches"] = sorted(receipt["evidence_files"]) == sorted(
        entry["path"] for entry in report["evidence_files"]
    )

    # fixture + script
    fixture_path = ROOT / "fixtures/render-bvh/scene-e2e/camlight-model.json"
    fixture = fixture_path.read_bytes()
    script_path = ROOT / "scripts/authoring/render-bvh-s7b/capture_e2e_camlight.py"
    script = script_path.read_bytes()
    counts = json.loads(fixture.decode("utf-8"))["counts"]
    report["fixture"] = {
        "bytes": len(fixture),
        "lines": len(fixture.decode("utf-8").splitlines()),
        "sha256": sha256(fixture),
        "receipt": receipt["input_fixture"],
        "matches": (
            len(fixture) == receipt["input_fixture"]["bytes"]
            and sha256(fixture) == receipt["input_fixture"]["sha256"]
            and len(fixture.decode("utf-8").splitlines()) == receipt["input_fixture"]["lines"]
        ),
        "counts": counts,
        "counts_match": counts
        == {"nq": 8, "nv": 7, "nbody": 3, "njnt": 2, "ngeom": 7, "nsite": 0, "nmocap": 0, "ncam": 2, "nlight": 3},
        "script_sha256": sha256(script.replace(b"\r\n", b"\n")),
        "script_bytes": len(script),
        "script_lines": len(script.decode("utf-8").splitlines()),
        "script_matches": (
            sha256(script.replace(b"\r\n", b"\n")) == receipt["input_fixture"]["authoring_script"]["sha256"]
            and len(script.decode("utf-8").splitlines()) == receipt["input_fixture"]["authoring_script"]["lines"]
        ),
        "provenance_sources_exist": {
            source: (ROOT / source).exists()
            for source in json.loads(fixture.decode("utf-8"))["provenance"]["camera_data_reference"]["sources"]
        },
    }

    # logs vs receipt claims
    def text(name: str) -> str:
        data = (ROOT / EVIDENCE / name).read_bytes()
        if data.startswith(b"\xef\xbb\xbf"):
            return data.decode("utf-8-sig", "replace")
        if data.startswith(b"\xff\xfe") or data.startswith(b"\xfe\xff"):
            return data.decode("utf-16", "replace")
        try:
            return data.decode("utf-8")
        except UnicodeDecodeError:
            return data.decode("utf-16", "replace")

    gpu = text("gpu-run.log")
    host = text("host-run.log")
    regression = text("regression.log")
    regression_suites = {}
    current = None
    for line in regression.splitlines():
        match = re.search(r"Running tests\\(\w+)\.rs", line)
        if match:
            current = match.group(1)
            continue
        match = re.search(
            r"test result: ok\. (\d+) passed; (\d+) failed;[^\n]*finished in ([0-9.]+)s", line
        )
        if match and current:
            regression_suites[current] = {
                "passed": int(match.group(1)),
                "failed": int(match.group(2)),
                "elapsed_s": float(match.group(3)),
            }
            current = None
    report["logs"] = {
        "gpu_result_all": [
            (int(passed), int(failed), float(elapsed))
            for passed, failed, elapsed in re.findall(
                r"test result: ok\. (\d+) passed; (\d+) failed;[^\n]*finished in ([0-9.]+)s", gpu
            )
        ],
        "gpu_result": [
            group
            for group in re.findall(
                r"test result: ok\. (\d+) passed; (\d+) failed;[^\n]*finished in ([0-9.]+)s", gpu
            )
            if group[0] == "5"
        ][0] if re.search(r"test result: ok\. 5 passed", gpu) else None,
        "host_result": re.search(r"test result: ok\. (\d+) passed; (\d+) failed", host).groups()
        if re.search(r"test result: ok\.", host)
        else None,
        "camlight_bits": re.search(r"bit_diff=PoseBitDiff \{([^}]*)\}", gpu).group(1),
        "camlight_host_ulp": re.search(r"camera0 vs host quat_to_mat: max_ulp=(\d+) components_diff=(\d+)/9", gpu).groups(),
        "reverse_tampered_rgb": re.search(r"篡改夹具位姿作输入：RGB 变化 (\d+)/120000", gpu).group(1),
        "reverse_perturb": re.search(r"rgb 变化 (\d+)、depth 变化 (\d+)、seg 变化 (\d+)", gpu).groups(),
        "lifecycle_growth": re.search(r"baseline=(\d+) B free=(\d+) B growth=(\d+) B", gpu).groups(),
        "regression_suites": regression_suites,
    }
    report["receipt_claims"] = {
        "suite_passed": receipt["gpu"]["suite"]["passed"],
        "suite_elapsed": receipt["gpu"]["suite"]["elapsed_s"],
        "host_passed": receipt["gpu"]["suite"]["host_passed"],
        "tampered_rgb": receipt["reverse_proof"]["tampered_fixture_poses_as_input_rgb_changed_pixels"],
        "perturb_rgb": receipt["reverse_proof"]["product_perturbation"]["rgb_changed_pixels"],
        "perturb_depth": receipt["reverse_proof"]["product_perturbation"]["depth_changed_pixels"],
        "perturb_seg": receipt["reverse_proof"]["product_perturbation"]["seg_changed_pixels"],
        "growth": receipt["lifecycle"]["device_memory_bytes"]["growth"],
        "no_regression": {
            key: receipt["no_regression"][key]["passed"] for key in receipt["no_regression"]
        },
    }
    # body xpos claim from QA camlight dump
    camlight = json.loads((ROOT / "target/qa-s8/out/camlight.json").read_text(encoding="utf-8"))
    body_bits = camlight["product_body_xpos"]
    body_values = [[f32(bits) for bits in body] for body in body_bits]
    report["body_xpos"] = {
        "qa_values": body_values,
        "receipt_values": receipt["pose_verification"]["product_fk_body_xpos"],
    }

    # diff scope and no product src change
    changed = git_text("diff", "--name-only", BASE, TARGET).splitlines()
    report["diff"] = {
        "range": f"{BASE}..{TARGET}",
        "changed": changed,
        "changed_count": len(changed),
        "src_changes": [path for path in changed if path.startswith("src/")],
        "non_s8_paths": [
            path
            for path in changed
            if not (
                path.startswith(".normify/")
                or path.startswith("docs/development-evidence/render-bvh/s8-e2e/")
                or path in (
                    "fixtures/render-bvh/scene-e2e/camlight-model.json",
                    "scripts/authoring/render-bvh-s7b/capture_e2e_camlight.py",
                    "tests/render_e2e.rs",
                    "tests/support/render_bvh/e2e.rs",
                )
            )
        ],
    }
    report["size_ratchet"] = {}
    for path, receipt_key in (
        ("tests/render_e2e.rs", "tests_render_e2e_rs"),
        ("tests/support/render_bvh/e2e.rs", "tests_support_render_bvh_e2e_rs"),
    ):
        data = (ROOT / path).read_bytes()
        entry = {
            "lines": len(data.decode("utf-8").splitlines()),
            "bytes": len(data),
            "sha256": sha256(data),
            "receipt": receipt["size_ratchet"][receipt_key],
        }
        entry["under_500"] = entry["lines"] < 500
        entry["matches_receipt"] = (
            entry["lines"] == entry["receipt"]["lines"]
            and entry["bytes"] == entry["receipt"]["bytes"]
            and entry["sha256"] == entry["receipt"]["sha256"]
        )
        report["size_ratchet"][path] = entry

    # r01-p8 mapping: traces exist, and the ten items are the expected ten
    traces = []
    for item in receipt["r01_p8_closure"]:
        for path in item["trace"].split(";"):
            path = path.strip()
            if path.endswith((".json", ".md")):
                traces.append(path)
    unique_traces = list(dict.fromkeys(traces))
    report["r01_p8"] = {
        "items": [item["item"] for item in receipt["r01_p8_closure"]],
        "item_count": len(receipt["r01_p8_closure"]),
        "traces": unique_traces,
        "missing_traces": [path for path in unique_traces if not (ROOT / path).exists()],
        "readme_section_6": "r01-p8 收口映射" in readme,
    }
    # spot-check two trace claims against their evidence files
    s6b = (ROOT / "docs/development-evidence/render-bvh/s6b-context-lifecycle/README.md").read_text(encoding="utf-8")
    s7b3 = json.loads(
        (ROOT / "docs/development-evidence/render-bvh/s7b3-flags/receipt.json").read_text(encoding="utf-8")
    )
    report["r01_p8"]["spot_check"] = {
        "s6b_growth_0": "growth 0 B" in s6b or "growth=0 B" in s6b or "0 B" in s6b,
        "s7b3_ptx": s7b3["precision_gate"]["entries"][0]["ptx_bytes"] == 197583,
    }

    # unmet coverage
    unmet_text = " ".join(receipt["unmet"])
    report["unmet"] = {
        "items": receipt["unmet"],
        "covers": {
            "nworld>1": "nworld>1" in unmet_text or "nworld > 1" in unmet_text,
            "shadow/skybox/splat/flex/hfield": all(
                key in unmet_text for key in ("阴影", "天空盒", "splat", "柔体", "hfield")
            ),
            "linux_t4": "Linux" in unmet_text or "T4" in unmet_text,
            "upstream_equivalence": "上游完全等价" in unmet_text,
            "performance": "性能" in unmet_text,
            "higher_spp": "更高 spp" in unmet_text,
        },
    }

    # forbidden-phrase scan (literal vs negated)
    terms = ["完整渲染通过", "完整R01通过", "完整 R01 通过", "产品渲染验收通过", "上游完全等价", "双平台"]
    report["forbidden_phrases"] = {
        term: {
            "readme_count": readme.count(term),
            "receipt_count": json.dumps(receipt, ensure_ascii=False).count(term),
            "negated_contexts": [
                line.strip()
                for line in readme.splitlines()
                if term in line and any(marker in line for marker in ("不宣称", "未宣称", "未声明", "不声明"))
            ],
        }
        for term in terms
    }

    # known auxiliary metric
    report["host_quat_to_mat"] = {
        "receipt_note": receipt["pose_verification"]["cam0"]["host_quat_to_mat_aux"],
        "authoritative_reference_bits_mismatches": receipt["pose_verification"]["cam0"]["origin_bit_mismatches"]
        + receipt["pose_verification"]["cam0"]["xmat_bit_mismatches"]
        + receipt["pose_verification"]["cam1"]["origin_bit_mismatches"]
        + receipt["pose_verification"]["cam1"]["xmat_bit_mismatches"],
        "classification": "auxiliary metric (host helper not used in the render path); authoritative "
        "comparison is 0-bit-difference against the pinned data poses",
    }

    out = ROOT / "target/qa-s8/out/judge_evidence.json"
    out.write_text(json.dumps(report, indent=1, ensure_ascii=False), encoding="utf-8")
    print(json.dumps(report, indent=1, ensure_ascii=False)[:9000])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
