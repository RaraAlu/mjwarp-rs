"""QA recompute of the structure/leaf-too-coarse 33->34 delta (independent).

Parses every module md at the two P1 revisions, rebuilds leaf sets and tests
candidate coarse thresholds against the recorded warning counts (33 / 34).
"""
import json
import pathlib
import re
import subprocess
import sys

REPO = pathlib.Path(r"D:\Rust\mjwarp-rs")
BASE = "f86b8575"
TIP = "f38b83fd"


def parse(rev):
    tree = subprocess.run(["git", "ls-tree", "-r", "--name-only", rev, ".normify/modules"],
                          cwd=REPO, capture_output=True).stdout.decode().split()
    mods = {}
    for p in tree:
        if not p.endswith(".md"):
            continue
        t = subprocess.run(["git", "show", f"{rev}:{p}"], cwd=REPO,
                           capture_output=True).stdout.decode("utf-8", errors="replace")
        mid = re.search(r'^id = "([^"]+)"', t, re.M).group(1)
        srcs = re.findall(r'\[\[source\]\]\s*\n\s*path\s*=\s*"([^"]+)"', t)
        mods[mid] = srcs
    return mods


def leaf_ids(mods):
    ids = set(mods)
    parents = {i for i in ids for j in ids if j != i and j.startswith(i + ".")}
    return {i for i in ids if i not in parents}


def lines(rev, path):
    b = subprocess.run(["git", "show", f"{rev}:{path}"], cwd=REPO, capture_output=True).stdout
    return b.count(b"\n")


def main():
    out = {}
    for rev in (BASE, TIP):
        mods = parse(rev)
        leaves = leaf_ids(mods)
        stats = {}
        for m in leaves:
            n = len(mods[m])
            tot = sum(lines(rev, s) for s in mods[m])
            mx = max([lines(rev, s) for s in mods[m]] or [0])
            stats[m] = {"sources": n, "total_lines": tot, "max_file_lines": mx}
        out[rev] = {"modules": len(mods), "leaves": len(leaves), "leaf_stats": stats}
    variants = {
        "n_ge_2": lambda s: s["sources"] >= 2,
        "n_ge_2_and_total_gt_220": lambda s: s["sources"] >= 2 and s["total_lines"] > 220,
        "n_ge_2_or_max_gt_220": lambda s: s["sources"] >= 2 or s["max_file_lines"] > 220,
        "n_gt_1_or_max_gt_220": lambda s: s["sources"] > 1 or s["max_file_lines"] > 220,
        "n_ge_3": lambda s: s["sources"] >= 3,
    }
    res = {}
    for name, f in variants.items():
        b = {m for m, s in out[BASE]["leaf_stats"].items() if f(s)}
        a = {m for m, s in out[TIP]["leaf_stats"].items() if f(s)}
        res[name] = {"before": len(b), "after": len(a),
                     "flipped_in": sorted(a - b), "flipped_out": sorted(b - a)}
    out["variants"] = res
    changed = {}
    for m in out[BASE]["leaf_stats"]:
        if out[BASE]["leaf_stats"][m] != out[TIP]["leaf_stats"].get(m):
            changed[m] = {"before": out[BASE]["leaf_stats"][m], "after": out[TIP]["leaf_stats"].get(m)}
    out["changed_leaf_stats"] = changed
    pathlib.Path(r"D:\Rust\mjwarp-rs\target\qa-arch\log\leaf.json").write_text(
        json.dumps(out, ensure_ascii=False, indent=1), encoding="utf-8", newline="\n")
    print("modules/leaves:", out[BASE]["modules"], out[BASE]["leaves"], "->",
          out[TIP]["modules"], out[TIP]["leaves"])
    for name, r in res.items():
        print(f"{name}: before={r['before']} after={r['after']} flipped_in={r['flipped_in']}")
    print("changed leaf stats:", json.dumps(changed, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    sys.exit(main())
