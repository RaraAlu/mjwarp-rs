"""QA G2 independent review: verify G2 evidence integrity, scope claims, gate/test truthfulness."""

import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
EV = ROOT / "docs/development-evidence/g10-p5-gjk-epa"
CANDS = ["900d1fe", "735abaf"]


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def git(*args):
    return subprocess.run(["git", "-C", str(ROOT)] + list(args), capture_output=True)


def blob_sha(path, commit):
    res = git("rev-parse", "%s:%s" % (commit, path))
    return res.stdout.decode().strip() if res.returncode == 0 else None


def worktree_sha(path):
    res = git("hash-object", str(ROOT / path))
    return res.stdout.decode().strip()


def main():
    report = {"checks": [], "failed": []}

    def check(name, ok, detail=None):
        report["checks"].append({"check": name, "ok": bool(ok), "detail": detail})
        if not ok:
            report["failed"].append({"check": name, "detail": detail})

    evidence_files = ["README.md", "receipt.json", "capture_mjwarp_f32.py",
                      "upstream-mjwarp-f32.json", "native-error-seed.json", "gjk-epa-run.log"]
    for name in evidence_files:
        rel = "docs/development-evidence/g10-p5-gjk-epa/" + name
        check("blob_equal:" + name, blob_sha(rel, "735abaf") == worktree_sha(rel),
              {"blob": blob_sha(rel, "735abaf"), "worktree": worktree_sha(rel)})
        check("unchanged_since_735abaf:" + name, blob_sha(rel, "735abaf") == blob_sha(rel, "HEAD"))
    # capture script sha recorded inside the reference JSON
    upstream = json.loads((EV / "upstream-mjwarp-f32.json").read_text(encoding="utf-8"))
    check("capture_script_sha_matches",
          upstream["provenance"]["authoring_script_sha256"] == sha256(EV / "capture_mjwarp_f32.py"),
          {"recorded": upstream["provenance"]["authoring_script_sha256"]})
    check("provenance_versions",
          upstream["provenance"]["versions"] ==
          {"python": "3.12.13", "mujoco": "3.15.0", "warp": "1.15.0",
           "mujoco_warp": "3.15.0", "numpy": "2.5.4"}, upstream["provenance"]["versions"])
    check("provenance_device_cpu", upstream["provenance"]["device"] == "cpu")
    check("provenance_flags",
          upstream["provenance"]["authoring_only"] is True
          and upstream["provenance"]["reads_product_tree"] is False
          and upstream["provenance"]["product_python_dependency"] is False)
    check("upstream_revision", upstream.get("upstream_revision") == "71da24d956378a87a703b6e1442b13aec0c4ac29")
    check("five_cases", sorted(upstream["cases"]) ==
          ["convex-mesh", "cylinder-box", "degenerate-mesh", "ellipsoid-box", "mesh-mesh"],
          sorted(upstream["cases"]))
    check("two_captures_all_true",
          all(case["two_captures_bitwise_identical"] for case in upstream["cases"].values()))
    check("counts", {k: v["output"]["count"] for k, v in upstream["cases"].items()} ==
          {"cylinder-box": 8, "ellipsoid-box": 2, "mesh-mesh": 2, "convex-mesh": 2, "degenerate-mesh": 2})
    # 900d1fe scope
    diff = git("diff", "--name-only", "900d1fe^..900d1fe").stdout.decode().splitlines()
    allowed = ("src/physics/collision/gjk/", "src/physics/collision.rs",
               "tests/gjk_epa.rs", "docs/development-evidence/g10-p5-gjk-epa/")
    outside = [p for p in diff if p and not p.startswith(allowed)]
    check("900d1fe_scope", outside == [], outside)
    check("900d1fe_no_cargo", not any(p.startswith("Cargo") for p in diff))
    check("900d1fe_no_existing_narrowphase",
          not any(p in ("src/physics/collision/narrowphase.rs", "src/physics/collision/filter.rs",
                        "src/physics/collision/sap.rs", "src/physics/collision/nxn.rs",
                        "src/physics/collision/tests.rs") for p in diff))
    check("900d1fe_no_s2_s8_evidence",
          not any(p.startswith("docs/development-evidence/render-bvh/") or
                  p.startswith("docs/evidence/d04-complex-physics/") or
                  p.startswith("fixtures/") for p in diff))
    # 735abaf scope
    diff2 = git("diff", "--name-only", "735abaf^..735abaf").stdout.decode().splitlines()
    check("735abaf_scope", all(p.startswith(".normify/") or
                               p.startswith("docs/development-evidence/g10-p5-gjk-epa/") or
                               p == "tests/gjk_epa.rs" for p in diff2), diff2)
    check("tests_gjk_epa_removed_at_735abaf",
          git("cat-file", "-e", "735abaf:tests/gjk_epa.rs").returncode != 0)
    check("gjk_source_unchanged_after_735abaf",
          git("diff", "--name-only", "735abaf..HEAD", "--",
              "src/physics/collision/gjk").stdout.decode().strip() == "")
    # gate truthfulness
    case_evidence = (ROOT / "src/physics/collision/gjk/case_evidence.rs").read_text(encoding="utf-8")
    check("gate_test_present", "fn upstream_f32_within_4ulp()" in case_evidence)
    check("gate_test_not_ignored",
          "#[ignore" not in case_evidence.split("fn upstream_f32_within_4ulp()")[0].split("#[test]")[-1])
    check("native_bitwise_exploration_ignored",
          '#[ignore = "探索项：native 是 f64，f32 实现不可能位级一致，不作门禁"]' in case_evidence)
    # no code reads the seed file as tolerance: the only references are the writer in case_evidence.rs
    grep = subprocess.run(["git", "-C", str(ROOT), "grep", "-n", "native-error-seed"],
                          capture_output=True)
    hits = grep.stdout.decode(errors="replace").splitlines()
    code_hits = [h for h in hits if h.startswith(("src/", "tests/"))]
    reader_hits = [h for h in code_hits if "case_evidence.rs" not in h]
    check("seed_not_read_as_tolerance", reader_hits == [] and
          "read_to_string(\"docs/development-evidence/g10-p5-gjk-epa/native-error-seed.json\")" not in
          (ROOT / "src/physics/collision/gjk/case_evidence.rs").read_text(encoding="utf-8"),
          reader_hits)
    # stale-evidence bookkeeping (reported, not pass/fail of deliverable)
    report["stale_evidence"] = {
        "run_log_has_final_gate_table": "[判据]" in (EV / "gjk-epa-run.log").read_text(
            encoding="utf-8", errors="replace"),
        "run_log_mentions_input_aligned_contract": "input_aligned_contract" in (
            EV / "gjk-epa-run.log").read_text(encoding="utf-8", errors="replace"),
        "receipt_gate_test_module": json.loads((EV / "receipt.json").read_text(encoding="utf-8"))
        ["final_criterion"]["gate_test"],
        "actual_gate_module": "physics::collision::gjk::case_evidence::upstream_f32_within_4ulp",
        "readme_unit_claim": "557",
        "receipt_unit_claim": json.loads((EV / "receipt.json").read_text(encoding="utf-8"))
        ["gates"]["unit"]["tests_passed"],
        "unit_gate_artifact_735abaf": json.loads(
            git("show", "735abaf:.normify/runs/2026-10-25-g10-p5-gjk-epa/unit.json")
            .stdout.decode(encoding="utf-8"))["tests"]["passed"],
        "case_compare_lines_actual": len(
            (ROOT / "src/physics/collision/gjk/case_compare.rs").read_text(encoding="utf-8").splitlines()),
        "receipt_case_compare_lines": json.loads((EV / "receipt.json").read_text(encoding="utf-8"))
        ["artifacts"]["test"]["lines"],
    }
    # independent post-review commits (for the report)
    report["post_review_commits"] = git("log", "--oneline", "735abaf..HEAD").stdout.decode(
        errors="replace").splitlines()
    report["overall"] = {"all_checks_ok": not report["failed"]}
    (ROOT / "target/qa-g2/evidence_check_report.json").write_text(
        json.dumps(report, ensure_ascii=False, indent=1), encoding="utf-8")
    print("checks=%d failed=%d" % (len(report["checks"]), len(report["failed"])))
    for item in report["failed"]:
        print("  FAIL", item["check"], item["detail"])
    print("stale:", json.dumps(report["stale_evidence"], ensure_ascii=False))
    print("post_review_commits:", report["post_review_commits"])


if __name__ == "__main__":
    main()
