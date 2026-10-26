# -*- coding: utf-8 -*-
"""QA check: G5 bitwise unchanged pre vs post (relmat / sdf_relmat / sdf_host / gjk_epa).

Compares my own pre (r2 parent copy) and post (r2 main tree) logs line-by-line after
dropping build/run header lines. Writes qa-g5-diff.json.
"""
import json
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
TASK = REPO / "docs/development-evidence/g10-p5-faithful-fma-fix-independent-check"
LOGS = TASK / "logs"
TARGETS = ("relmat", "sdf-relmat", "sdf-host", "gjk-epa")
DROP_PREFIXES = ("Finished", "Running", "Compiling", "Blocking", "warning:", "   Compiling",
                 "    Finished", "     Running", "note:", "  -->", "   |", "=", "--")


def read_text(path: Path) -> str:
    raw = path.read_bytes()
    if raw.startswith(b"\xff\xfe"):
        return raw.decode("utf-16")
    if raw.startswith(b"\xef\xbb\xbf"):
        return raw[3:].decode("utf-8")
    return raw.decode("utf-8", errors="replace")


def semantic_lines(path: Path) -> list:
    if not path.is_file():
        return []
    out = []
    for line in read_text(path).splitlines():
        text = line.rstrip()
        if not text.strip():
            continue
        if any(text.strip().startswith(prefix.strip()) for prefix in DROP_PREFIXES):
            continue
        out.append(text.strip())
    return out


result = {"targets": {}}
for target in TARGETS:
    pre = semantic_lines(LOGS / f"g5-pre-{target}.log")
    post = semantic_lines(LOGS / f"g5-post-{target}.log")
    # libtest may run test cases in a nondeterministic order: compare as multisets
    diffs = []
    pre_sorted, post_sorted = sorted(pre), sorted(post)
    for left, right in zip(pre_sorted, post_sorted):
        if left != right:
            diffs.append({"pre": left, "post": right})
    result["targets"][target] = {
        "pre_lines": len(pre),
        "post_lines": len(post),
        "identical_as_multiset": pre_sorted == post_sorted,
        "diff_count": len(diffs) + abs(len(pre) - len(post)),
        "first_diffs": diffs[:5],
        "post_semantic": post,
    }

sdf = result["targets"]["sdf-relmat"]["post_semantic"]
result["claims_verified"] = {
    "relmat_4_passed": any("4 passed" in line for line in result["targets"]["relmat"]["post_semantic"]),
    "sdf_10_of_10_world": any("10/10 world" in line for line in sdf),
    "sdf_80_of_80_lines": sum(1 for line in sdf if "80/80 contacts bit-equal" in line),
    "all_sdf_max_abs_zero": all(("max_abs 0e0" in line) for line in sdf if "80/80" in line),
    "sdf_host_1_passed": any("1 passed" in line for line in result["targets"]["sdf-host"]["post_semantic"]),
    "gjk_epa_3_passed": any("3 passed" in line for line in result["targets"]["gjk-epa"]["post_semantic"]),
}
result["all_identical"] = all(entry["identical_as_multiset"] for entry in result["targets"].values())

(TASK / "qa-g5-diff.json").write_text(
    json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(f"all_identical={result['all_identical']}")
for target, entry in result["targets"].items():
    print(target, "identical=", entry["identical_as_multiset"], "lines=", entry["pre_lines"], entry["post_lines"])
print(json.dumps(result["claims_verified"], ensure_ascii=False))
