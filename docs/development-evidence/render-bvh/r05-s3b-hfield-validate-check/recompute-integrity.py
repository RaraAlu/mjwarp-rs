"""QA recompute part 3: manifest self-ref, entity table copy, zero-change span, planning materials."""
import hashlib
import json
import pathlib
import subprocess
import sys

REPO = pathlib.Path(r"D:\Rust\mjwarp-rs")
EV = "docs/development-evidence/render-bvh/r05-s3b-hfield-validate"
OUT = {}


def blob(p):
    r = subprocess.run(["git", "show", f"HEAD:{p}"], cwd=REPO, capture_output=True)
    return r.stdout if r.returncode == 0 else None


def btext(p):
    b = blob(p)
    return b.decode("utf-8", errors="replace") if b is not None else None


def sha(b):
    return hashlib.sha256(b).hexdigest()


def main():
    man = btext(f"{EV}/manifest.txt")
    rows = [l.split() for l in man.splitlines() if l and not l.startswith("#")]
    self_ref = [r[2] for r in rows if r[2] == f"{EV}/manifest.txt"]
    OUT["manifest_self_reference_exact"] = self_ref
    OUT["manifest_rows"] = len(rows)
    OUT["manifest_covers"] = {
        "readme": any(r[2].endswith(f"{EV}/README.md") for r in rows),
        "receipt": any(r[2].endswith(f"{EV}/receipt.json") for r in rows),
        "guards_json": any(r[2] == "scripts/validate/render-bvh/guards.json" for r in rows),
        "scripts": [r[2] for r in rows if r[2].startswith("scripts/validate/render-bvh/")],
    }

    # entity table copy from the authoring evidence
    authoring = "docs/development-evidence/render-bvh/r05-s3b-hfield-target-authoring/window/entity-table.json"
    a = blob(authoring)
    c = blob(f"{EV}/materials/entity-table.json")
    OUT["entity_table_copy_identical"] = (a is not None and c is not None and sha(a) == sha(c))
    OUT["entity_table_authoring_sha"] = sha(a) if a is not None else None

    # zero change: whole S3b chain since authoring close (2c56cc1a) and this window chain
    def diffstat(rev, paths):
        r = subprocess.run(["git", "diff", "--stat", f"{rev}..HEAD", "--"] + paths, cwd=REPO, capture_output=True)
        return r.stdout.decode().strip()

    OUT["zero_change_since_2c56cc1a"] = {
        "src": diffstat("2c56cc1a", ["src"]),
        "tests": diffstat("2c56cc1a", ["tests"]),
        "fixtures": diffstat("2c56cc1a", ["fixtures"]),
        "cu": diffstat("2c56cc1a", ["src/render/kernel/cuda"]),
    }
    OUT["zero_change_window_chain"] = {
        "src_tests_fixtures": diffstat("d1f7f851", ["src", "tests", "fixtures"]),
        "scripts_all": diffstat("d1f7f851", ["scripts"]),
    }
    # commits of the window chain
    log = subprocess.run(["git", "log", "--oneline", "--name-status", "d1f7f851..HEAD"],
                         cwd=REPO, capture_output=True).stdout.decode().splitlines()
    OUT["window_chain_log"] = log

    # planning materials vs runtime formula
    plan = btext(f"{EV}/materials/window-plan.txt")
    base = btext(f"{EV}/materials/gpu-baseline.txt")
    OUT["planning"] = {
        "window_plan_head": plan.splitlines()[:8] if plan else None,
        "gpu_baseline_head": base.splitlines()[:6] if base else None,
    }
    # facts-derived floor check: floor = min(10240, baseline_free - 3072)
    facts = btext(f"{EV}/attempt2/facts.txt")
    import re
    bfree = int(re.search(r"baseline_free_mib=(\d+)", facts).group(1))
    floor = int(re.search(r"floor_mib=(\d+)", facts).group(1))
    OUT["floor_formula_ok"] = (floor == min(10240, bfree - 3072), bfree, floor)

    # receipt claims vs recomputed numbers (spot-check key fields)
    rc = json.loads(btext(f"{EV}/receipt.json"))
    aw = rc["attempt2_window"]
    OUT["receipt_claims_vs_recompute"] = {
        "attempt": (rc.get("attempt"), aw.get("window_start")),
        "entity_exits_all_zero": all(v[0] == 0 for v in aw["entity_exits"].values()),
        "sha_guards": aw["sha_guards"],
        "post_window_exe_recheck": aw["post_window_exe_recheck"],
        "samples": aw["samples"],
        "free_min_mib": aw["free_min_mib"],
        "floor_mib": aw["floor_mib"],
        "not_a_normal_threshold_pass": aw["not_a_normal_threshold_pass"],
        "precheck_util_samples_claimed": aw.get("precheck_util_samples"),
    }
    pathlib.Path(r"D:\Rust\mjwarp-rs\target\qa-s3b-val\recompute3.json").write_text(
        json.dumps(OUT, ensure_ascii=False, indent=1), encoding="utf-8", newline="\n")
    print(json.dumps(OUT, ensure_ascii=False, indent=1)[:4000])
    return 0


if __name__ == "__main__":
    sys.exit(main())
