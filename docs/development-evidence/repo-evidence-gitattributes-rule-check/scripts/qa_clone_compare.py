#!/usr/bin/env python3
"""Independent fresh-clone check for the `.gitattributes` evidence-dir rule.

Clones the repo into <tmp> (default target/gitattributes-verify-qa, gitignored) with
core.autocrlf=true, then compares every docs/development-evidence/**/manifest.txt in
the clone against the committed blob bytes:
  - checkout 48bc2840 (rule absent): expected drift in uncovered dirs
  - checkout b88f7964 (rule present): expected 0 drift
Also reproduces the 18-manifest sample recorded by the executant evidence.

Usage: python qa_clone_compare.py [--repo PATH] [--tmp PATH] [--out JSON] [--sample JSON]
"""
import argparse
import hashlib
import json
import os
import shutil
import stat
import subprocess
import sys

BEFORE = "48bc2840"
AFTER = "1bfc4ac8"


def sh(*a, cwd):
    return subprocess.run(["git"] + list(a), cwd=cwd, capture_output=True)


def sha(b):
    return hashlib.sha256(b).hexdigest()


def manifests(repo, rev):
    out = sh("ls-tree", "-r", "--name-only", rev, "docs/development-evidence", cwd=repo).stdout.decode()
    return sorted(p for p in out.splitlines() if p.endswith("/manifest.txt"))


def cat(repo, rev, path):
    return sh("show", rev + ":" + path, cwd=repo).stdout


def state(repo, tmp, rev):
    r1 = sh("checkout", "--detach", rev, cwd=tmp)
    r2 = sh("rm", "-r", "--cached", "-q", ".", cwd=tmp)
    r3 = sh("reset", "--hard", "-q", "HEAD", cwd=tmp)
    if r1.returncode or r3.returncode:
        raise SystemExit("state(%s) failed: checkout=%d reset=%d %s" % (
            rev, r1.returncode, r3.returncode, r3.stderr.decode("utf-8", "ignore")[-300:]))
    head = sh("rev-parse", "--short", "HEAD", cwd=tmp).stdout.decode().strip()
    status = sh("status", "--porcelain", cwd=tmp).stdout.decode().strip()
    return head, status


def compare(repo, tmp, rev, paths, ref="blob"):
    match, mism = 0, []
    for p in paths:
        cp = os.path.join(tmp, p)
        if not os.path.isfile(cp):
            mism.append((p, "MISSING"))
            continue
        c = open(cp, "rb").read()
        ref_bytes = cat(repo, rev, p) if ref == "blob" else open(os.path.join(repo, p), "rb").read()
        if sha(c) == sha(ref_bytes):
            match += 1
        else:
            mism.append((p, "ref=%s clone=%s clone_crlf=%s" % (
                len(ref_bytes), len(c), b"\r\n" in c)))
    return match, mism


def rmtree_force(path):
    """git objects are read-only on Windows; clear the bit before deleting."""

    def on_exc(func, p, exc):
        os.chmod(p, stat.S_IWRITE)
        func(p)

    shutil.rmtree(path, onexc=on_exc)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", default=os.getcwd())
    ap.add_argument("--tmp", default=os.path.join(os.getcwd(), "target", "gitattributes-verify-qa"))
    ap.add_argument("--out", default="")
    ap.add_argument("--sample", default="docs/development-evidence/repo-evidence-gitattributes-rule/verify/clone-compare.json")
    a = ap.parse_args()
    repo, tmp = os.path.abspath(a.repo), os.path.abspath(a.tmp)
    result = {
        "repo": repo,
        "tmp_clone": tmp,
        "before": BEFORE,
        "after": AFTER,
        "git_version": sh("--version", cwd=repo).stdout.decode().strip(),
        "core_autocrlf": sh("config", "core.autocrlf", cwd=repo).stdout.decode().strip(),
        "commands": [
            "git -c core.autocrlf=true -c core.longpaths=true clone --local --no-checkout <repo> <tmp>",
            "git -C <tmp> config core.longpaths true   # deep docs/evidence paths exceed MAX_PATH",
            "git -c core.autocrlf=true checkout --detach <rev>",
            "git rm -r --cached . && git reset --hard HEAD   # force repopulate with current attributes",
            "compare every docs/development-evidence/**/manifest.txt clone bytes vs commit blob bytes",
        ],
    }
    if os.path.isdir(tmp):
        rmtree_force(tmp)
    os.makedirs(os.path.dirname(tmp), exist_ok=True)
    r = subprocess.run(["git", "-c", "core.autocrlf=true", "-c", "core.longpaths=true",
                        "clone", "--local", "--no-checkout", repo, tmp], capture_output=True)
    if r.returncode:
        raise SystemExit("clone failed: " + r.stderr.decode("utf-8", "ignore")[-300:])
    subprocess.run(["git", "config", "core.longpaths", "true"], cwd=tmp, capture_output=True)

    print("== tmp clone:", tmp)
    # before
    head, status = state(repo, tmp, BEFORE)
    ms_before = manifests(repo, BEFORE)
    match, mism = compare(repo, tmp, BEFORE, ms_before)
    attrs = subprocess.run(["git", "check-attr", "text", "--"] + ms_before, cwd=tmp,
                           capture_output=True).stdout.decode()
    unspec = sum(1 for l in attrs.splitlines() if l.endswith(": unspecified"))
    print("before HEAD=%s manifests=%d match_blob=%d mismatch=%d attr_unspecified=%d status=%r"
          % (head, len(ms_before), match, len(mism), unspec, status[:60]))
    for p, why in mism:
        print("  MISMATCH", p, "::", why)
    sample_paths = []
    if os.path.isfile(os.path.join(repo, a.sample)):
        sample_paths = json.load(open(os.path.join(repo, a.sample), encoding="utf-8"))["manifest_samples"]
    s_match, s_mism = compare(repo, tmp, BEFORE, sample_paths, ref="worktree")
    print("sample18 vs main worktree: match=%d mismatch=%d" % (s_match, len(s_mism)))
    result["before_state"] = {
        "head": head, "clone_status": status,
        "manifests_total": len(ms_before), "match_blob": match,
        "mismatch": [p for p, _ in mism],
        "attr_text_unspecified": unspec,
        "sample18_match_worktree": s_match,
        "sample18_mismatch": [p for p, _ in s_mism],
    }

    # after
    head, status = state(repo, tmp, AFTER)
    ms_after = manifests(repo, AFTER)
    match, mism = compare(repo, tmp, AFTER, ms_after)
    attrs = subprocess.run(["git", "check-attr", "text", "--"] + ms_after, cwd=tmp,
                           capture_output=True).stdout.decode()
    unspec = sum(1 for l in attrs.splitlines() if l.endswith(": unspecified"))
    print("after HEAD=%s manifests=%d match_blob=%d mismatch=%d attr_unspecified=%d status=%r"
          % (head, len(ms_after), match, len(mism), unspec, status[:60]))
    result["after_state"] = {
        "head": head, "clone_status": status,
        "manifests_total": len(ms_after), "match_blob": match,
        "mismatch": [p for p, _ in mism],
        "attr_text_unspecified": unspec,
    }
    result["conclusion"] = {
        "rule_necessary": len(result["before_state"]["mismatch"]) > 0,
        "rule_effective": len(result["after_state"]["mismatch"]) == 0,
        "sample18_consistent_with_executant_at_before":
            s_match == 17 and
            [p for p, _ in s_mism] == ["docs/development-evidence/mesh-polygon-gt16-host-diagnostics/manifest.txt"],
    }
    if a.out:
        os.makedirs(os.path.dirname(os.path.abspath(a.out)), exist_ok=True)
        with open(a.out, "w", encoding="utf-8", newline="\n") as f:
            json.dump(result, f, ensure_ascii=False, indent=2)
            f.write("\n")
        print("wrote", a.out)
    print("RESULT:", "PASS" if result["after_state"]["mismatch"] == [] else "FAIL")


if __name__ == "__main__":
    main()
