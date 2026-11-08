"""Generate evidence artifacts/logs/receipt for the §2.4 capture-counts card (authoring-only; zero GPU).

- runs the perworld_capture_meta.py CPU selfcheck and saves the raw output;
- computes old (HEAD) vs new (working tree) script identity (sha256/bytes/lines);
- static checks: the helper is defined in the meta module only, the main script imports it once and
  calls it once, and the call site is after the two-capture bitwise-identity checks;
- writes artifacts/script-static-checks.json and receipt.json (manifest is regenerated separately
  after README.md is written).
"""

import ast
import hashlib
import json
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[5]
EVIDENCE = pathlib.Path(__file__).resolve().parents[1]
MAIN = "scripts/capture_render_perworld.py"
META = "scripts/perworld_capture_meta.py"
TASK = "2026-11-08-render-perworld-l-v2-capture-discriminability-counts"


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def old_bytes(relative: str) -> bytes:
    return subprocess.run(["git", "cat-file", "blob", f"HEAD:{relative}"], cwd=ROOT,
                          capture_output=True, check=True).stdout


def line_count(data: bytes) -> int:
    return data.count(b"\n")


def main() -> int:
    python = sys.executable
    selfcheck = subprocess.run([python, "-B", str(ROOT / META)], cwd=ROOT, capture_output=True)
    selfcheck_text = (f"===== python -B scripts/perworld_capture_meta.py (exit={selfcheck.returncode}) =====\n"
                      + selfcheck.stdout.decode("utf-8", errors="replace").replace("\r\n", "\n").strip("\n") + "\n")
    EVIDENCE.joinpath("logs").mkdir(parents=True, exist_ok=True)
    EVIDENCE.joinpath("logs/selfcheck.log").write_text(selfcheck_text, encoding="utf-8", newline="\n")

    identities = {}
    for relative in (MAIN, META):
        before, after = old_bytes(relative), (ROOT / relative).read_bytes()
        identities[relative] = {
            "before": {"sha256": sha256(before), "bytes": len(before), "lines": line_count(before)},
            "after": {"sha256": sha256(after), "bytes": len(after), "lines": line_count(after)},
        }

    main_text = (ROOT / MAIN).read_text(encoding="utf-8")
    meta_text = (ROOT / META).read_text(encoding="utf-8")
    tree = ast.parse(main_text)
    meta_tree = ast.parse(meta_text)
    function_defs = [node.name for node in ast.walk(meta_tree) if isinstance(node, ast.FunctionDef)]
    main_defs = [node.name for node in ast.walk(tree) if isinstance(node, ast.FunctionDef)]
    import_calls = [node for node in ast.walk(meta_tree)
                    if isinstance(node, ast.Call) and getattr(node.func, "id", "") == "fail"]
    call_lines = [node.lineno for node in ast.walk(tree)
                  if isinstance(node, ast.Call) and getattr(node.func, "id", "") == "discriminability_counts"]
    identity_line = next(node.lineno for node in ast.walk(tree)
                         if isinstance(node, ast.If) and "first_hashes" in ast.unparse(node.test)
                         and "second_hashes" in ast.unparse(node.test))
    static = {
        "schema": 1,
        "kind": "capture-2-4-script-static-checks",
        "helper_defined_in_meta_only": function_defs.count("discriminability_counts") == 1
                                      and "discriminability_counts" not in main_defs,
        "main_imports_helper_once": main_text.count("discriminability_counts") == 2,  # import + call
        "main_call_count": len(call_lines),
        "call_after_two_capture_identity_check": bool(call_lines) and all(line > identity_line for line in call_lines),
        "two_capture_identity_check_line": identity_line,
        "main_call_lines": call_lines,
        "new_module_files": [],
        "line_counts": {relative: identities[relative]["after"]["lines"] for relative in identities},
        "all_line_counts_within_500": all(identity["after"]["lines"] <= 500 for identity in identities.values()),
        "meta_fail_references": len(import_calls),
    }
    static["result"] = "PASS" if (static["helper_defined_in_meta_only"] and static["main_imports_helper_once"]
                                  and static["main_call_count"] == 1
                                  and static["call_after_two_capture_identity_check"]
                                  and static["all_line_counts_within_500"]) else "FAIL"
    EVIDENCE.joinpath("artifacts").mkdir(parents=True, exist_ok=True)
    EVIDENCE.joinpath("artifacts/script-static-checks.json").write_text(
        json.dumps(static, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")

    receipt = {
        "schema": 1,
        "task_id": TASK,
        "status": "implemented",
        "slice": "P8-渲染与BVH",
        "owner": "Backend",
        "kind": "capture-discriminability-counts",
        "positioning": "Signed spec v3 §2.4 requires capture.json to carry the reference-side discriminability "
                       "counts (C1 bitwise+magnitude, C2 depth/seg, two diagnostic items). The capture script now "
                       "computes them with a helper bit-identical to the G-L0 pilot readout and writes them after the "
                       "two-capture bitwise-identity checks. GPU-free: no capture, no CUDA.",
        "source": {
            "spec": "docs/development-evidence/render-bvh/perworld-l-v2-light-criterion/spec.md",
            "spec_sha256": "bb064fe0a85ab38abba8043768666a0d9f73fc53d9f2f3dd72eace060c8a924d",
            "sections": ["§2.4 deterministic + provenance (discriminability counts)"],
            "readout_script": {
                "path": "docs/development-evidence/render-bvh/perworld-l-v2-pilot-g-l0/materials/readout_c1_c2.py",
                "sha256": "4a054abaf1f0b73a...",  # replaced below by the real digest
                "note": "computation is bit-identical: same unpack, f32->f64 max-channel diff >= 1/255, depth bits, seg pairs",
            },
        },
        "deliverables": {
            MAIN: identities[MAIN]["after"],
            META: identities[META]["after"],
            "evidence": {
                "README.md": "narrative (fields/structure, readout parity, pilot cross-check, default behavior)",
                "artifacts/crosscheck-pilot-counts.json": "pilot attempt-3 coordinates vs readout",
                "artifacts/dryrun-compare.json": "HEAD vs working tree dry-run leaf compare",
                "artifacts/script-static-checks.json": "helper placement / import / call-site static checks",
                "logs/crosscheck-pilot-counts.log": "raw cross-check output (pilot npz under target/, not committed)",
                "logs/dryrun-compare.log": "raw dry-run compare output",
                "logs/selfcheck.log": "module CPU selfcheck output",
            },
        },
        "before_after_lines": {relative: {"before": identities[relative]["before"]["lines"],
                                          "after": identities[relative]["after"]["lines"]} for relative in identities},
        "crosscheck": {
            "generated_by": "materials/crosscheck_pilot_counts.py",
            "pilot_dir": "target/render-bvh/perworld-l-v2-gl0/pilot-out/nw3-light-hinge02040",
            "pilot_data_committed": False,
            "readout": "target/render-bvh/perworld-l-v2-gl0/window/readout.json",
            "counts": "C1 magnitude 38285/40644, C1 bitwise 43604/41815, C2 depth 35828/33883, C2 seg 16041/16754, "
                      "diagnostic cam-persp depth/seg 0/0, diagnostic cam-hinge rgb magnitude 50363/52329 "
                      "(bitwise 54380/54250)",
            "matches_readout_json": True,
        },
        "default_behavior": {
            "generated_by": "materials/compare_dryrun.py",
            "unexpected": 0,
            "only_expected_difference": "recipe.script.sha256/bytes",
            "worlds_1": "exit 1 + identical stderr; no out_dir created (unchanged)",
        },
        "boundaries": {
            "gpu_free": True,
            "no_capture": True,
            "unchanged": ["fixtures/", "src/", "tests/",
                          "docs/development-evidence/render-bvh/perworld-l-v2-light-criterion/"],
        },
        "non_claims": {
            "statement": "Only capture.json count recording + CPU self-check: no capture-validity claim, no "
                         "discriminability-restoration claim, no rendering numeric conclusion. Real counts come from "
                         "the G-L1 rendered reference; pilot coordinates are only a readout-parity cross-check.",
        },
    }
    readout_path = ROOT / "docs/development-evidence/render-bvh/perworld-l-v2-pilot-g-l0/materials/readout_c1_c2.py"
    receipt["source"]["readout_script"]["sha256"] = sha256(readout_path.read_bytes())
    receipt["source"]["readout_script"]["bytes"] = len(readout_path.read_bytes())
    EVIDENCE.joinpath("receipt.json").write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + "\n",
                                                  encoding="utf-8", newline="\n")
    print(json.dumps({"identities": identities, "static": static, "selfcheck_exit": selfcheck.returncode},
                     ensure_ascii=False, indent=2))
    return 0 if static["result"] == "PASS" and selfcheck.returncode == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
