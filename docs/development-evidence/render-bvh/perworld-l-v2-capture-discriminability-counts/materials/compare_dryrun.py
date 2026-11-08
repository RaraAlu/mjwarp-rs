"""Default-behavior comparison for the §2.4 capture-counts card: HEAD scripts vs working-tree scripts.

Runs the same dry-run commands with the old (HEAD) and new (working tree) capture scripts and
diffs the JSON leaves; the only expected difference is `recipe.script.{sha256,bytes}`.
Also checks `--worlds 1` behavior and that dry-run creates no out_dir.

Writes artifacts/dryrun-compare.json and logs/dryrun-compare.log into the card evidence dir.
Usage: <pinned python> -B <this script>
"""

import json
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[5]
EVIDENCE = pathlib.Path(__file__).resolve().parents[1]
SCRIPTS = ("scripts/capture_render_perworld.py", "scripts/perworld_capture_meta.py")
OUT_ROOT = ROOT / "target/render-bvh/perworld-l-v2-counts-dryrun"
OUT_DIR = OUT_ROOT / "nw3-light-hinge02040"
ALLOWED = {"/recipe/script/sha256", "/recipe/script/bytes"}
CASES = {
    "default": ["--dry-run", "--out-root", str(OUT_ROOT), "--out-dir", str(OUT_DIR),
                "--reference-tag", "nw3-light-hinge02040"],
    "explicit-light-scene": ["--dry-run", "--scene", "fixtures/render-bvh/scene-a-light/scene.xml",
                             "--out-root", str(OUT_ROOT), "--out-dir", str(OUT_DIR),
                             "--reference-tag", "nw3-light-hinge02040"],
    "worlds-1": ["--dry-run", "--worlds", "1", "--out-root", str(OUT_ROOT), "--out-dir", str(OUT_DIR),
                 "--reference-tag", "nw3-light-hinge02040"],
}


def version_bytes(version: str) -> dict:
    result = {}
    for relative in SCRIPTS:
        if version == "new":
            result[relative] = (ROOT / relative).read_bytes()
        else:
            result[relative] = subprocess.run(["git", "cat-file", "blob", f"HEAD:{relative}"],
                                              cwd=ROOT, capture_output=True, check=True).stdout
    return result


def leaves(value, prefix=""):
    found = {}
    if isinstance(value, dict):
        for key, item in value.items():
            found.update(leaves(item, f"{prefix}/{key}"))
    elif isinstance(value, list):
        for index, item in enumerate(value):
            found.update(leaves(item, f"{prefix}/{index}"))
    else:
        found[prefix] = value
    return found


def run_case(python: str, case: str) -> dict:
    command = [python, "-B", str(ROOT / SCRIPTS[0]), *CASES[case]]
    completed = subprocess.run(command, cwd=ROOT, capture_output=True)
    return {"exit": completed.returncode,
            "stdout": completed.stdout.decode("utf-8", errors="replace"),
            "stderr": completed.stderr.decode("utf-8", errors="replace"),
            "out_dir_exists": OUT_DIR.exists()}


def main() -> int:
    python = sys.executable
    new_scripts = version_bytes("new")
    old_scripts = version_bytes("old")
    assert all(new_scripts[relative] != old_scripts[relative] for relative in SCRIPTS), "scripts must differ from HEAD"
    runs = {"old": {}, "new": {}}
    try:
        for version, payload in (("old", old_scripts), ("new", new_scripts)):
            for relative, data in payload.items():
                (ROOT / relative).write_bytes(data)
            for case in CASES:
                runs[version][case] = run_case(python, case)
                print(f"{version} {case}: exit={runs[version][case]['exit']} out_dir={runs[version][case]['out_dir_exists']}")
    finally:
        for relative, data in new_scripts.items():
            (ROOT / relative).write_bytes(data)
    cases = {}
    unexpected = 0
    for case in CASES:
        old_run, new_run = runs["old"][case], runs["new"][case]
        entry = {"exit": {"old": old_run["exit"], "new": new_run["exit"]},
                 "stderr_identical": old_run["stderr"] == new_run["stderr"],
                 "stdout_identical": old_run["stdout"] == new_run["stdout"],
                 "out_dir_created": {"old": old_run["out_dir_exists"], "new": new_run["out_dir_exists"]}}
        if case == "worlds-1":
            if old_run["exit"] != 1 or new_run["exit"] != 1:
                unexpected += 1
            if old_run["stderr"] != new_run["stderr"]:
                unexpected += 1
            if old_run["out_dir_exists"] or new_run["out_dir_exists"]:
                unexpected += 1
        else:
            old_plan = json.loads(old_run["stdout"])
            new_plan = json.loads(new_run["stdout"])
            old_leaves, new_leaves = leaves(old_plan), leaves(new_plan)
            added = sorted(set(new_leaves) - set(old_leaves))
            removed = sorted(set(old_leaves) - set(new_leaves))
            changed = sorted(key for key in set(old_leaves) & set(new_leaves) if old_leaves[key] != new_leaves[key])
            unexpected_leaves = [key for key in added + removed + changed if key not in ALLOWED]
            unexpected += len(unexpected_leaves)
            entry.update({"added": added, "removed": removed, "changed": changed,
                          "unexpected_leaves": unexpected_leaves,
                          "cameras": new_plan.get("model_static", {}).get("cameras"),
                          "cam_active_fixed_only": new_plan.get("cam_active_fixed_only"),
                          "cam_active": new_plan.get("model_static", {}).get("cam_active"),
                          "legacy_manifest_entries": new_plan.get("legacy_manifest_entries")})
        cases[case] = entry
    artifact = {
        "schema": 1,
        "kind": "capture-2-4-default-behavior-compare",
        "old_scripts": {relative: {"sha256": __import__("hashlib").sha256(data).hexdigest(), "bytes": len(data)}
                        for relative, data in old_scripts.items()},
        "new_scripts": {relative: {"sha256": __import__("hashlib").sha256(data).hexdigest(), "bytes": len(data)}
                        for relative, data in new_scripts.items()},
        "cases": cases,
        "unexpected": unexpected,
        "result": "PASS" if unexpected == 0 else "FAIL",
    }
    EVIDENCE.joinpath("artifacts").mkdir(parents=True, exist_ok=True)
    EVIDENCE.joinpath("artifacts/dryrun-compare.json").write_text(
        json.dumps(artifact, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")
    lines = [
        f"===== dry-run compare: HEAD scripts vs working-tree scripts (exit={0 if unexpected == 0 else 1}) =====",
        "command: <pinned python> -B scripts/capture_render_perworld.py --dry-run [--scene …] [--worlds 1] "
        "--out-root target/render-bvh/perworld-l-v2-counts-dryrun --out-dir … --reference-tag nw3-light-hinge02040",
        json.dumps(artifact, ensure_ascii=False, indent=2),
        "SUMMARY " + ("PASS" if unexpected == 0 else "FAIL"),
    ]
    EVIDENCE.joinpath("logs").mkdir(parents=True, exist_ok=True)
    EVIDENCE.joinpath("logs/dryrun-compare.log").write_text("\n".join(lines) + "\n",
                                                             encoding="utf-8", newline="\n")
    print(json.dumps(artifact, ensure_ascii=False, indent=2))
    return 0 if unexpected == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
