"""QA independent recompute for arch-volume-module-sources-check (P1 scope f86b8575..f38b83fd)."""
import hashlib
import json
import pathlib
import re
import subprocess
import sys

REPO = pathlib.Path(r"D:\Rust\mjwarp-rs")
EV = "docs/development-evidence/render-bvh/arch-volume-module-sources"
BASE = "f86b8575"
TIP = "f38b83fd"
OUT = {}
MODS = ["mjwarp/render.md", "mjwarp/model/core.md", "mjwarp/io/core.md",
        "mjwarp/io/render.md", "mjwarp/math.md"]


def show(rev, path):
    r = subprocess.run(["git", "show", f"{rev}:{path}"], cwd=REPO, capture_output=True)
    return r.stdout if r.returncode == 0 else None


def stext(rev, path):
    b = show(rev, path)
    return b.decode("utf-8", errors="replace") if b is not None else None


def parse_sources(md_text):
    """Return list of [[source]] paths in file order; also count."""
    return re.findall(r"\[\[source\]\]\s*\n\s*path\s*=\s*\"([^\"]+)\"", md_text)


def main():
    # 1. module source sets before/after (all module md files in the tree)
    tree_before = subprocess.run(["git", "ls-tree", "-r", "--name-only", BASE, ".normify/modules"],
                                 cwd=REPO, capture_output=True).stdout.decode().split()
    tree_after = subprocess.run(["git", "ls-tree", "-r", "--name-only", TIP, ".normify/modules"],
                                cwd=REPO, capture_output=True).stdout.decode().split()
    OUT["module_files"] = {"before": len(tree_before), "after": len(tree_after),
                           "added": sorted(set(tree_after) - set(tree_before)),
                           "removed": sorted(set(tree_before) - set(tree_after))}
    reg_before, reg_after = {}, {}
    for p in tree_before:
        t = stext(BASE, p)
        if t is not None:
            reg_before[p] = parse_sources(t)
    for p in tree_after:
        t = stext(TIP, p)
        if t is not None:
            reg_after[p] = parse_sources(t)
    OUT["total_sources_before"] = sum(len(v) for v in reg_before.values())
    OUT["total_sources_after"] = sum(len(v) for v in reg_after.values())
    # per-module deltas
    deltas = {}
    for p in sorted(set(reg_before) | set(reg_after)):
        b = reg_before.get(p, [])
        a = reg_after.get(p, [])
        added = [x for x in a if x not in b]
        removed = [x for x in b if x not in a]
        if added or removed:
            deltas[p] = {"before": len(b), "after": len(a), "added": added, "removed": removed}
    OUT["module_deltas"] = deltas
    # the 24-file claim
    claimed = {
        "mjwarp/render.md": [
            "src/render/kernel/cast.rs", "src/render/kernel/megakernel.rs",
            "src/render/kernel/megakernel_aa.rs", "src/render/kernel/megakernel_arrays.rs",
            "src/render/kernel/output.rs", "src/render/kernel/renderer.rs",
            "src/render/kernel/cuda/render.cu", "src/render/kernel/cuda/render_common.cu",
            "src/render/kernel/cuda/render_math.cu", "src/render/kernel/cuda/render_megakernel.cu",
            "src/render/kernel/cuda/render_shade.cu", "src/render/kernel/cuda/aa_resolve.cu",
            "src/render/context/tests.rs"],
        "mjwarp/model/core.md": ["src/model/volume.rs", "src/model/volume_input.rs",
                                 "src/model/collision_inputs.rs"],
        "mjwarp/io/core.md": ["src/io/volume.rs", "src/io/collision_inputs.rs",
                              "src/io/state/tests.rs", "include/mjwarp_collision_inputs_probe.h",
                              "native/model_probe_collision_inputs.cpp"],
        "mjwarp/io/render.md": ["include/mjwarp_render_probe.h", "native/model_probe_render.cpp"],
        "mjwarp/math.md": ["src/math/fma/tests.rs"],
    }
    actual_map = {}
    for p, d in deltas.items():
        rel = p.replace(".normify/modules/", "")
        actual_map[rel] = d["added"]
    OUT["claimed_vs_actual"] = {
        "claimed_total": sum(len(v) for v in claimed.values()),
        "actual_total": sum(len(v) for v in actual_map.values()),
        "per_module_equal": {k: sorted(claimed[k]) == sorted(actual_map.get(k, [])) for k in claimed},
        "unexpected_module_changes": sorted(set(actual_map) - set(claimed)),
        "removed_anywhere": {p: d["removed"] for p, d in deltas.items() if d["removed"]},
    }
    # 2. were these files registered anywhere before?
    all_before = {(p, src) for p, lst in reg_before.items() for src in lst}
    prev_where = {}
    for p, lst in reg_before.items():
        for src in lst:
            prev_where.setdefault(src, []).append(p.replace(".normify/modules/", ""))
    OUT["previously_registered"] = {src: prev_where.get(src) for m in claimed.values() for src in m
                                    if src in prev_where}
    # duplicates after
    dups = {}
    for p, lst in reg_after.items():
        counts = {}
        for s in lst:
            counts[s] = counts.get(s, 0) + 1
        d = {s: c for s, c in counts.items() if c > 1}
        if d:
            dups[p] = d
    OUT["duplicates_after"] = dups

    # 3. line counts of the 24 files (LF) vs README table
    readme = stext(TIP, f"{EV}/README.md")
    table = re.findall(r"^\| mjwarp\.[a-z.]+ \| `([^`]+)` \| (\d+) \|", readme, re.M)
    line_rows = {}
    for path, claimed_lines in table:
        f = REPO / path
        actual = f.read_bytes().count(b"\n") if f.exists() else None
        line_rows[path] = {"claimed": int(claimed_lines), "actual_lf": actual,
                           "exists": f.exists(), "tracked": bool(subprocess.run(
                               ["git", "ls-files", "--error-unmatch", path], cwd=REPO,
                               capture_output=True).stdout)}
    OUT["line_counts"] = line_rows
    OUT["line_table_rows"] = len(table)

    # 4. zero code change vs baseline
    code_paths = ["src", "tests", "fixtures", "include", "native", "Cargo.toml", "Cargo.lock"]
    OUT["code_diff"] = subprocess.run(["git", "diff", "--stat", f"{BASE}..{TIP}", "--"] + code_paths,
                                      cwd=REPO, capture_output=True).stdout.decode().strip()
    OUT["full_diff_paths"] = subprocess.run(["git", "diff", "--name-only", f"{BASE}..{TIP}"],
                                            cwd=REPO, capture_output=True).stdout.decode().split()
    OUT["diff_only_normify_and_evidence"] = all(
        p.startswith(".normify/") or p.startswith("docs/development-evidence/")
        for p in OUT["full_diff_paths"])

    # 5. committed == worktree for scope (HEAD == TIP)
    head = subprocess.run(["git", "rev-parse", "HEAD"], cwd=REPO, capture_output=True).stdout.decode().strip()
    OUT["head"] = head
    scope_prefixes = [".normify/modules/", f"{EV}/"]
    dirty = []
    for pref in scope_prefixes:
        r = subprocess.run(["git", "diff", "--name-only", "HEAD", "--", pref], cwd=REPO,
                           capture_output=True)
        dirty.extend(r.stdout.decode().split())
    OUT["worktree_dirty_in_scope"] = dirty
    OUT["scope_clean_vs_head"] = not dirty

    # 6. evidence files: count / LF / BOM / receipt task_id / forbidden words
    ev_files = subprocess.run(["git", "ls-tree", "-r", "--name-only", TIP, f"{EV}/"],
                              cwd=REPO, capture_output=True).stdout.decode().split()
    ev = {"count": len(ev_files), "non_lf": [], "bom": [], "files": []}
    for p in ev_files:
        b = show(TIP, p)
        ev["files"].append(p.split("/")[-1])
        if b.count(b"\r") or b.count(b"\x00"):
            ev["non_lf"].append(p)
        if b[:3] == b"\xef\xbb\xbf":
            ev["bom"].append(p)
    OUT["evidence"] = ev
    rec = json.loads(stext(TIP, f"{EV}/receipt.json"))
    OUT["receipt"] = {"task_id": rec.get("task_id"), "kind": rec.get("kind"),
                      "files": rec["scope"]["files"], "by_module": rec["scope"]["by_module"],
                      "validate": rec["validate"], "module_counts": rec["module_counts"]}
    terms = [[0x5168, 0x90E8, 0x6A21, 0x5757, 0x6E90, 0x5DF2, 0x767B, 0x8BB0],
             [0x67B6, 0x6784, 0x6811, 0x5B8C, 0x6574],
             [0x63A5, 0x53E3, 0x7B49, 0x4EF7]]
    words = ["".join(chr(c) for c in t) for t in terms]
    forb = []
    for p in ev_files:
        t = stext(TIP, p)
        hits = [f"T{i}" for i, w in enumerate(words) if w in t]
        if hits:
            forb.append({"file": p.split("/")[-1], "terms": hits,
                         "lines": [i + 1 for i, l in enumerate(t.splitlines()) if any(w in l for w in words)][:4]})
    OUT["forbidden_hits"] = forb

    # 7. registry totals sanity + unregistered recompute
    tracked = subprocess.run(["git", "ls-files", "src", "include", "native", "build.rs"],
                             cwd=REPO, capture_output=True).stdout.decode().split()
    OUT["tracked_src_include_native"] = len(tracked)
    registered_union = set()
    for lst in reg_after.values():
        registered_union.update(lst)
    # also registered at BASE for the "before" number
    registered_before = set()
    for lst in reg_before.values():
        registered_before.update(lst)
    OUT["registered_before_union"] = len(registered_before)
    unreg_before = sorted(set(tracked) - registered_before)
    unreg_after = sorted(set(tracked) - registered_union)
    OUT["unregistered"] = {"before": len(unreg_before), "after": len(unreg_after),
                           "delta": len(unreg_before) - len(unreg_after),
                           "after_by_top": {d: sum(1 for x in unreg_after if x.split("/")[0] == d)
                                            for d in ("src", "include", "native", "build.rs")}}
    src_unreg = [x for x in unreg_after if x.startswith("src/")]
    OUT["unregistered_src_after"] = len(src_unreg)
    OUT["unregistered_src_by_area"] = {}
    for x in src_unreg:
        parts = x.split("/")
        key = "/".join(parts[1:2])
        OUT["unregistered_src_by_area"][key] = OUT["unregistered_src_by_area"].get(key, 0) + 1
    # are the 24 now all registered?
    all24 = [s for m in claimed.values() for s in m]
    OUT["all24_registered_after"] = {s: any(s in lst for lst in reg_after.values()) for s in all24}
    OUT["all24_tracked"] = {s: bool(subprocess.run(["git", "ls-files", "--error-unmatch", s],
                                                   cwd=REPO, capture_output=True).stdout) for s in all24}
    OUT["unregistered_before_contains_24"] = {s: s in unreg_before for s in all24}

    # problems
    problems = []
    if OUT["claimed_vs_actual"]["actual_total"] != 24 or not all(OUT["claimed_vs_actual"]["per_module_equal"].values()):
        problems.append("24-file mapping mismatch")
    if OUT["previously_registered"]:
        problems.append("some file was already registered before")
    if OUT["duplicates_after"]:
        problems.append("duplicate source entries")
    if any(v["actual_lf"] != v["claimed"] for v in line_rows.values()):
        problems.append("line count mismatch")
    if OUT["code_diff"]:
        problems.append("code change present")
    if not OUT["diff_only_normify_and_evidence"]:
        problems.append("diff outside .normify/evidence")
    if OUT["worktree_dirty_in_scope"]:
        problems.append("scope dirty vs HEAD")
    if OUT["evidence"]["count"] != 12 or OUT["evidence"]["non_lf"] or OUT["evidence"]["bom"]:
        problems.append("evidence files/hygiene")
    if OUT["receipt"]["task_id"] != "2026-10-28-arch-volume-module-sources":
        problems.append("receipt task_id")
    if OUT["unregistered"]["before"] != 178 or OUT["unregistered"]["after"] != 154:
        problems.append("unregistered counts")
    if not all(OUT["all24_registered_after"].values()) or not all(OUT["all24_tracked"].values()):
        problems.append("24 files not all registered/tracked")
    OUT["problems"] = problems

    pathlib.Path(r"D:\Rust\mjwarp-rs\target\qa-arch\log\recompute.json").write_text(
        json.dumps(OUT, ensure_ascii=False, indent=1), encoding="utf-8", newline="\n")
    print("total sources before/after:", OUT["total_sources_before"], OUT["total_sources_after"])
    print("deltas:", {k: (v["before"], v["after"], len(v["added"]), v["removed"]) for k, v in deltas.items()})
    print("claimed vs actual:", OUT["claimed_vs_actual"])
    print("previously registered:", OUT["previously_registered"])
    print("dups:", dups)
    print("line rows:", OUT["line_table_rows"], "mismatch:", [k for k, v in line_rows.items() if v["actual_lf"] != v["claimed"]])
    print("code diff:", OUT["code_diff"] or "(empty)")
    print("diff only normify/evidence:", OUT["diff_only_normify_and_evidence"], "head:", OUT["head"])
    print("worktree dirty in scope:", OUT["worktree_dirty_in_scope"])
    print("evidence:", OUT["evidence"]["count"], OUT["evidence"]["non_lf"], OUT["evidence"]["bom"])
    print("receipt:", OUT["receipt"])
    print("tracked src/include/native:", OUT["tracked_src_include_native"], "registered before union:", OUT["registered_before_union"])
    print("unregistered:", OUT["unregistered"])
    print("unreg src by area:", OUT["unregistered_src_by_area"])
    print("24 all registered:", all(OUT["all24_registered_after"].values()), "tracked:", all(OUT["all24_tracked"].values()))
    print("forbidden:", OUT["forbidden_hits"])
    print("PROBLEMS", problems)
    return 0


if __name__ == "__main__":
    sys.exit(main())
