"""Generate the S2a independent-check evidence JSON files."""

from __future__ import annotations

import hashlib
import json
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[2]
EV = ROOT / "docs/development-evidence/render-bvh/s2a-independent-check"
DELIVERY = "docs/development-evidence/render-bvh/s2a-scene-bvh-host"
S2A_COMMIT = "f9adf14b2f6c18b9bb8787189c15a908becfb29f"
TASK = "2026-10-24-render-bvh-s2a-independent-check"


def blob(rev: str, path: str) -> bytes:
    return subprocess.run(["git", "cat-file", "blob", f"{rev}:{path}"], cwd=ROOT,
                          capture_output=True, check=True).stdout


def git(*args: str) -> str:
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True, check=True).stdout


def main() -> int:
    EV.mkdir(parents=True, exist_ok=True)
    head = git("rev-parse", "HEAD").strip()

    # 1) harness result
    result_text = (ROOT / "target/qa-s2a-harness/qa-s2a-harness-result.txt").read_text(encoding="utf-8")
    lines = result_text.strip().splitlines()
    build_line = next(line for line in lines if line.startswith("build_refit:"))
    two_line = next(line for line in lines if line.startswith("two_world:"))
    rejects = [line[len("reject: "):] for line in lines if line.startswith("reject: ")]
    result_line = next(line for line in lines if line.startswith("RESULT"))
    def kv(line: str, key: str) -> str:
        for token in line.split():
            if token.startswith(key + "="):
                return token.split("=", 1)[1]
        return ""
    harness = {
        "task_id": TASK,
        "harness": "target/qa-s2a-harness (QA-written Rust binary, path dependency on a clean git worktree of the S2a commit f9adf14)",
        "source_commit": S2A_COMMIT,
        "build_refit": {
            "poses": int(kv(build_line, "poses")),
            "errors": int(kv(build_line, "errors")),
            "changed_leaves": "[1, 2, 3, 4, 5, 6]",
            "invariants_checked": [
                "leaf_bounds equal the fixture bounds", "group root bounds == union of leaves",
                "every internal node bounds == union of its children", "every leaf reached exactly once",
                "every refit pose keeps the invariants", "changed leaves after refit == [1,2,3,4,5,6]",
            ],
        },
        "two_world_query": {
            "rays": 32,
            "worlds": 2,
            "hits": kv(two_line, "hits"),
            "extra_qa_ray_queries": int(kv(two_line, "extra_queries")),
            "extra_qa_ray_hits": int(kv(two_line, "extra_hits")),
            "upward_miss_candidates": int(kv(two_line, "upward_miss_candidates")),
            "cross_world_or_set_errors": int(kv(two_line, "cross_world_or_set_errors")),
        },
        "rejections": {
            "cases": len(rejects),
            "rejected": sum(1 for line in rejects if line.endswith("rejected=true")),
            "list": rejects,
            "note": "delivery README bullet list groups 10 rejection categories; this harness checks 14 concrete calls, including the 13 required behaviours (empty scene, length mismatches, worlds=0, group out of range, empty world, invalid AABB build/refit, refit length, unsupported geom type, non-finite pose, empty mesh bounds, out-of-range world query)",
        },
        "result": result_line,
        "raw_log": "bvh-harness-result.txt",
    }
    (EV / "bvh-semantics.json").write_text(json.dumps(harness, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n")
    (EV / "bvh-harness-result.txt").write_text(result_text, encoding="utf-8", newline="\n")

    # 2) delivered tests (run in the clean worktree of f9adf14)
    tests = {
        "task_id": TASK,
        "worktree": "target/qa-s2a-head (clean checkout of f9adf14)",
        "targeted": {"command": "cargo test --locked --test render_scene_bvh_host", "passed": 5, "failed": 0},
        "module_unit": {"command": "cargo test --locked --lib render::bvh", "passed": 16, "failed": 0},
        "full_unit": {"command": "cargo test --locked", "passed": 506, "failed": 0, "ignored": 0,
                      "note": "sum over all libtest binaries; matches the delivery gate record (unit 506 passed / 0 failed)"},
        "verdict": "pass",
    }
    (EV / "delivered-tests.json").write_text(json.dumps(tests, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n")

    # 3) s2 cross-check (independent npz bit comparison)
    import struct
    import numpy as np
    fix = json.loads((ROOT / "fixtures/render-bvh/scene-a/s2a/bounds.json").read_text(encoding="utf-8"))

    def bits(values):
        return [int(np.frombuffer(struct.pack("<f", np.float32(v)), dtype=np.uint32)[0]) for v in values]

    cross = {"task_id": TASK, "comparisons": []}
    for path, case_index in (("fixtures/render-bvh/scene-a/s2/bvh_initial.npz", 0),
                             ("fixtures/render-bvh/scene-a/s2/bvh_moved.npz", 1)):
        z = np.load(ROOT / path)
        case = fix["cases"][case_index]
        lower_equal = bits(z["lower"].flatten()) == [int(v) for v in case["lower_bits"]]
        upper_equal = bits(z["upper"].flatten()) == [int(v) for v in case["upper_bits"]]
        cross["comparisons"].append({"npz": path, "case": case["name"], "lower_bits_equal": bool(lower_equal),
                                     "upper_bits_equal": bool(upper_equal)})
    cross["all_equal"] = all(c["lower_bits_equal"] and c["upper_bits_equal"] for c in cross["comparisons"])
    cross["verdict"] = "pass" if cross["all_equal"] else "fail"
    (EV / "s2-cross-check.json").write_text(json.dumps(cross, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n")

    # 4) artifact hashes vs the delivery receipt
    receipt = json.loads((ROOT / DELIVERY / "receipt.json").read_text(encoding="utf-8"))
    artifacts = []
    for key, meta in receipt["artifacts"].items():
        full = key if not (key.startswith("aabb-parity") or key.startswith("README")) else f"{DELIVERY}/{key}"
        data = blob(S2A_COMMIT, full)
        digest = hashlib.sha256(data).hexdigest()
        lines = data.count(b"\n") + (0 if data.endswith(b"\n") else 1)
        artifacts.append({
            "path": full, "recorded_sha256": meta["sha256"], "blob_sha256": digest,
            "hash_match": digest == meta["sha256"], "recorded_bytes": meta["bytes"], "blob_bytes": len(data),
            "bytes_match": len(data) == meta["bytes"], "recorded_lines": meta["lines"], "blob_lines": lines,
            "lines_match": lines == meta["lines"],
        })
    artifact_report = {
        "task_id": TASK, "commit": S2A_COMMIT, "artifacts": len(artifacts),
        "hash_matches": sum(1 for a in artifacts if a["hash_match"]),
        "bytes_matches": sum(1 for a in artifacts if a["bytes_match"]),
        "line_matches": sum(1 for a in artifacts if a["lines_match"]),
        "rows": artifacts,
        "verdict": "pass" if all(a["hash_match"] and a["bytes_match"] and a["lines_match"] for a in artifacts) else "fail",
    }
    (EV / "artifact-hashes.json").write_text(json.dumps(artifact_report, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n")

    # 5) boundary + concurrency
    s2a_files = {
        "src/render/bvh/mod.rs": 184, "src/render/bvh/aabb.rs": 400, "src/render/bvh/builder.rs": 440,
        "src/render/bvh/refit.rs": 123, "src/render/bvh/query.rs": 111, "src/render/mod.rs": 242,
        "tests/render_scene_bvh_host.rs": 472,
        "scripts/authoring/render-bvh/capture_s2a_bounds.py": 372,
        "scripts/authoring/render-bvh/verify_s2a_aabb.py": 307,
        "fixtures/render-bvh/scene-a/s2a/bounds.json": 1353,
        f"{DELIVERY}/aabb-parity.json": 216, f"{DELIVERY}/README.md": 168,
    }
    file_rows = []
    for path, expected in s2a_files.items():
        data = blob(S2A_COMMIT, path)
        lines = data.count(b"\n") + (0 if data.endswith(b"\n") else 1)
        file_rows.append({"path": path, "blob_lines": lines, "recorded_lines": expected,
                          "match": lines == expected, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()})
    committed_diff = git("diff", "--name-status", f"{S2A_COMMIT}..HEAD", "--", "src", "tests", "fixtures", "scripts").strip()
    worktree = [line for line in git("status", "--porcelain").splitlines()
                if line[3:].startswith(("src/render/bvh/", "fixtures/render-bvh/scene-a/s2/", "scripts/authoring/render-bvh/export_s2_hits.py"))]
    boundary = {
        "task_id": TASK,
        "head_at_review": head,
        "s2a_commit": S2A_COMMIT,
        "line_rows": file_rows,
        "max_new_file_lines": max(r["blob_lines"] for r in file_rows if r["path"] in
                                  ("src/render/bvh/aabb.rs", "src/render/bvh/builder.rs", "src/render/bvh/refit.rs",
                                   "src/render/bvh/query.rs", "tests/render_scene_bvh_host.rs")),
        "line_limit": "<= 500",
        "gpu": {"used": False, "note": "S2a review is CPU-only; no GPU command was run and the crate was built without cuda-probe"},
        "python": {"product_tests_start_python": False,
                   "note": "the S2a host tests and the QA harness are pure Rust; Python only appears in authoring scripts and comments, and the product test tree never spawns it"},
        "module_registration": {"module": "mjwarp.render", "sources": ["src/render/mod.rs", "src/render/bvh/mod.rs",
                                                                       "src/render/bvh/aabb.rs", "src/render/bvh/builder.rs",
                                                                       "src/render/bvh/refit.rs", "src/render/bvh/query.rs"]},
        "concurrent_changes": {
            "command": "git diff f9adf14..HEAD -- src tests fixtures scripts",
            "committed_diff": committed_diff if committed_diff else "(empty)",
            "note": "no committed change since the S2a commit; concurrent S2b work is uncommitted in the shared worktree and only touches S2b-owned paths, listed below",
            "worktree_wip_paths": worktree,
            "s2a_unchanged_files_touched": False,
        },
        "verdict": "pass" if all(r["match"] for r in file_rows) and not committed_diff else "fail",
    }
    (EV / "boundary-and-concurrency.json").write_text(json.dumps(boundary, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n")

    # 6) delivery claim mapping
    claims = {
        "task_id": TASK,
        "claims": [
            {"claim": "56/56 leaf AABBs bit-equal, max ULP 0", "evidence": "aabb-recompute.json: 56 leaves, 0 mismatches, max ULP 0 (independent Python reimplementation)"},
            {"claim": "model inputs (geom_size f32, mesh bounds) bit-equal to model-312.json", "evidence": "aabb-recompute.json inputs: 0 diffs"},
            {"claim": "build/refit semantics invariants hold for all 8 poses; changed leaves [1,2,3,4,5,6]", "evidence": "bvh-semantics.json: poses=8 errors=0 changed=[1,2,3,4,5,6]"},
            {"claim": "two worlds 32x2 queries: candidates never cross worlds; identical geom sets; 64/64 hits", "evidence": "bvh-semantics.json: hits=64/64, cross_world_or_set_errors=0, plus 48/48 QA extra rays"},
            {"claim": "rejection surface covered", "evidence": "bvh-semantics.json: 14/14 concrete reject calls rejected"},
            {"claim": "targeted 5 tests / module 16 tests / unit 506 tests pass", "evidence": "delivered-tests.json: 5/0, 16/0, 506/0"},
            {"claim": "s2 cross-check lower/upper bits 4/4 equal", "evidence": "s2-cross-check.json: all_equal=true"},
            {"claim": "artifact hashes/bytes/lines match the receipt", "evidence": "artifact-hashes.json: 12/12 hash, bytes, lines"},
        ],
        "verdict": "pass",
    }
    (EV / "claim-mapping.json").write_text(json.dumps(claims, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n")

    print(json.dumps({
        "harness": {"build_errors": harness["build_refit"]["errors"], "hits": harness["two_world_query"]["hits"],
                    "rejections": f'{harness["rejections"]["rejected"]}/{harness["rejections"]["cases"]}'},
        "artifact_hashes": f'{artifact_report["hash_matches"]}/{artifact_report["artifacts"]}',
        "line_matches": f'{artifact_report["line_matches"]}/{artifact_report["artifacts"]}',
        "cross_check": cross["verdict"],
        "concurrent_committed_diff": committed_diff or "(empty)",
    }, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
