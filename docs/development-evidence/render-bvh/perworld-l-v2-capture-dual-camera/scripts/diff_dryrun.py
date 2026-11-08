"""Leaf-by-leaf dry-run comparison for the dual-camera card (zero GPU).

Compares the before/after `--dry-run` logs captured by `target/run-dual-dryruns.ps1`
and writes `artifacts/dryrun-diff.json` + `artifacts/worlds1-behavior.json`.

Allowed differences: `/recipe/script/sha256` and `/recipe/script/bytes` only (the
main script was edited). Everything else must be identical (`unexpected=0`).
`--worlds 1` must keep failing by design with identical stderr/exit and no out_dir.
"""

from __future__ import annotations

import json
import pathlib
import sys

EV = pathlib.Path(__file__).resolve().parents[1]
LOGS = EV / "logs"
ART = EV / "artifacts"
ALLOWED = {"/recipe/script/sha256", "/recipe/script/bytes"}
CASES = ("default-dryrun", "explicit-dryrun")
results: list[tuple[str, bool]] = []


def check(name: str, ok: bool, detail: str = "") -> None:
    results.append((name, ok))
    print(("PASS " if ok else "FAIL ") + name + ((" :: " + detail) if detail else ""))


def flatten(value, prefix="") -> dict:
    out = {}
    if isinstance(value, dict):
        for key, item in value.items():
            out.update(flatten(item, f"{prefix}/{key}"))
    elif isinstance(value, list):
        for index, item in enumerate(value):
            out.update(flatten(item, f"{prefix}/{index}"))
    else:
        out[prefix] = value
    return out


def diff_case(case: str) -> dict:
    before = flatten(json.loads((LOGS / f"before-{case}.json").read_text(encoding="utf-8")))
    after = flatten(json.loads((LOGS / f"after-{case}.json").read_text(encoding="utf-8")))
    removed = sorted(set(before) - set(after))
    added = sorted(set(after) - set(before))
    changed = sorted(k for k in before.keys() & after.keys() if before[k] != after[k])
    allowed = sorted(k for k in changed + added if k in ALLOWED)
    unexpected = sorted(k for k in changed + added + removed if k not in ALLOWED)
    entry = {
        "before_script_sha256": before.get("/recipe/script/sha256"),
        "before_script_bytes": before.get("/recipe/script/bytes"),
        "after_script_sha256": after.get("/recipe/script/sha256"),
        "after_script_bytes": after.get("/recipe/script/bytes"),
        "changed": {k: {"before": before[k], "after": after[k]} for k in changed},
        "added": {k: after[k] for k in added},
        "removed": removed,
        "allowed": allowed,
        "unexpected": unexpected,
        "unexpected_count": len(unexpected),
    }
    check(f"{case}: allowed-only ({', '.join(allowed) or 'none'})", not unexpected, str(unexpected))
    check(f"{case}: removed=0", not removed, str(removed))
    check(f"{case}: added=0", not added, str(added))
    return entry


def main() -> int:
    ART.mkdir(parents=True, exist_ok=True)
    diff = {case: diff_case(case) for case in CASES}

    # explicit 3-camera facts must be unchanged between the two versions
    for tag, name in (("before", "before"), ("after", "after")):
        plan = json.loads((LOGS / f"{name}-explicit-dryrun.json").read_text(encoding="utf-8"))
        cameras = plan["model_static"]["cameras"]
        check(f"{tag}: cameras-3 (persp/hinge/track)",
              cameras == [[0, 0, "cam-persp"], [1, 0, "cam-hinge"], [2, 3, "cam-track"]],
              json.dumps(cameras))
        check(f"{tag}: cam_active_fixed_only", plan["cam_active_fixed_only"] == [True, True, False],
              json.dumps(plan["cam_active_fixed_only"]))
        check(f"{tag}: legacy 112", plan["legacy_manifest_entries"] == 112,
              str(plan.get("legacy_manifest_entries")))
    default_plan = json.loads((LOGS / "after-default-dryrun.json").read_text(encoding="utf-8"))
    check("default: legacy 107", default_plan["legacy_manifest_entries"] == 107,
          str(default_plan.get("legacy_manifest_entries")))

    # --worlds 1 behaviour: identical exit/stderr/stdout and no out_dir
    worlds1 = {}
    for name in ("before", "after"):
        stderr = (LOGS / f"{name}-worlds1-dryrun.stderr.log").read_bytes()
        stdout = (LOGS / f"{name}-worlds1-dryrun.json").read_bytes()
        runs = json.loads((LOGS / f"{name}-runs.json").read_text(encoding="utf-8"))
        worlds1[name] = {
            "exit_code": runs[f"{name}-worlds1-dryrun"]["exit_code"],
            "stderr": stderr.decode("utf-8"),
            "stderr_bytes": len(stderr),
            "stdout_bytes": len(stdout),
            "out_dir_created": runs[f"{name}-worlds1-dryrun"]["out_dir_exists"],
        }
    check("worlds1: both exit 1",
          worlds1["before"]["exit_code"] == worlds1["after"]["exit_code"] == 1)
    check("worlds1: stderr identical", worlds1["before"]["stderr"] == worlds1["after"]["stderr"])
    check("worlds1: no out_dir", not worlds1["before"]["out_dir_created"]
          and not worlds1["after"]["out_dir_created"])
    check("worlds1: stdout empty", worlds1["before"]["stdout_bytes"] == 0
          and worlds1["after"]["stdout_bytes"] == 0)
    (ART / "dryrun-diff.json").write_text(
        json.dumps({"schema": 1, "allowed": sorted(ALLOWED), "cases": diff},
                   indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    (ART / "worlds1-behavior.json").write_text(
        json.dumps({"schema": 1, "cases": worlds1}, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8", newline="\n")

    passed = sum(1 for _, ok in results if ok)
    print(f"SUMMARY {'PASS' if passed == len(results) else 'FAIL'} {passed}/{len(results)}")
    return 0 if passed == len(results) else 1


if __name__ == "__main__":
    sys.exit(main())
