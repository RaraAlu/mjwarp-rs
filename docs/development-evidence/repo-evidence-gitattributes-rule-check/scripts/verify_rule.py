#!/usr/bin/env python3
"""QA review of the `.gitattributes` evidence-dir rule (commit ce4a7d18) + the two
follow-up evidence fixes (R07 658c3677, G10 b88f7964). Read-only.

Run from the repo root: python verify_rule.py
Exit code 0 = all FAIL checks passed; observations (OBS) do not fail the run.
"""
import hashlib
import json
import os
import subprocess
import sys

REPO = os.getcwd()
BEFORE = "48bc2840"
RULE = "ce4a7d18"
R07 = "658c3677"
G10 = "b88f7964"
EVID = "docs/development-evidence/repo-evidence-gitattributes-rule"
fails = []


def sh(*a, cwd=REPO, text=True):
    r = subprocess.run(["git"] + list(a), cwd=cwd, capture_output=True)
    return r.stdout.decode("utf-8", "ignore") if text else r.stdout


def blob(rev, path):
    return sh("show", rev + ":" + path, text=False)


def sha(b):
    return hashlib.sha256(b).hexdigest()


def ok(label, cond, detail=""):
    print(("PASS " if cond else "FAIL ") + label + (" :: " + detail if detail else ""))
    if not cond:
        fails.append(label)


def obs(label, detail=""):
    print("OBS  " + label + (" :: " + detail if detail else ""))


def attr(rev, path):
    r = subprocess.run(["git", "check-attr", "text", "diff", "--source=" + rev, "--", path],
                       cwd=REPO, capture_output=True).stdout.decode("utf-8", "ignore")
    return " ; ".join(x.split(": ", 1)[-1] for x in r.strip().splitlines())


status_lines = [l for l in sh("status", "--porcelain").splitlines() if l.strip()]
own = (".normify/", "docs/development-evidence/repo-evidence-gitattributes-rule-check/")
other = [l for l in status_lines if not l[3:].lstrip().startswith(own)]
print("== repo HEAD:", sh("rev-parse", "--short", "HEAD").strip(),
      "dirty=", len(status_lines), "dirty_outside_this_review=", other)
ok("no unrelated working-tree changes", other == [], repr(other))

print("\n[1] rule diff: +1/-0 and appended at end")
diff = sh("diff", BEFORE + ".." + RULE, "--", ".gitattributes")
added = [l[1:] for l in diff.splitlines() if l.startswith("+") and not l.startswith("+++")]
removed = [l[1:] for l in diff.splitlines() if l.startswith("-") and not l.startswith("---")]
ok("added lines = 1", len(added) == 1, repr(added))
ok("removed lines = 0", len(removed) == 0, repr(removed))
ok("added line is the rule", added == ["docs/development-evidence/** -text -diff"], repr(added))
parent_lines = blob(BEFORE, ".gitattributes").decode().splitlines()
rule_lines = blob(RULE, ".gitattributes").decode().splitlines()
ok("rule is last line", rule_lines[-1] == "docs/development-evidence/** -text -diff")
ok("line count +1", len(rule_lines) == len(parent_lines) + 1, "%d -> %d" % (len(parent_lines), len(rule_lines)))
ok("all pre-existing lines byte-identical", parent_lines == rule_lines[:-1])

print("\n[2] check-attr before/after")
cases = {
    "docs/development-evidence/g10-p5-summary/receipt.json": ("text: unspecified ; diff: unspecified", "text: unset ; diff: unset"),
    "docs/development-evidence/g10-p5-summary-check/manifest.txt": ("text: unspecified ; diff: unspecified", "text: unset ; diff: unset"),
    "docs/development-evidence/np01-e-criterion-a2/manifest.txt": ("text: unspecified ; diff: unspecified", "text: unset ; diff: unset"),
    "docs/development-evidence/criteria-purpose-audit/manifest.txt": ("text: unset ; diff: unset", "text: unset ; diff: unset"),
    "docs/evidence/resident-crb/receipt.json": ("text: unset ; diff: unset", "text: unset ; diff: unset"),
    "scripts/validate/repo-write-lock.ps1": ("text: unspecified ; diff: unspecified", "text: unspecified ; diff: unspecified"),
    "src/lib.rs": ("text: unspecified ; diff: unspecified", "text: unspecified ; diff: unspecified"),
    "fixtures/render-bvh/scene-a/model-312.json": ("text: unspecified ; diff: unspecified", "text: unspecified ; diff: unspecified"),
}
for p, (before, after) in cases.items():
    b, a = attr(BEFORE, p), attr(RULE, p)
    ok("attr " + p, (b, a) == (before, after), "before=%s | after=%s" % (b, a))

print("\n[3] no blob rewrite (sample)")
samples = [
    "docs/development-evidence/g10-p5-summary/receipt.json",
    "docs/development-evidence/g10-p5-summary/README.md",
    "docs/development-evidence/g10-p5-summary/manifest.txt",
    "docs/development-evidence/g10-p5-summary-check/README.md",
    "docs/development-evidence/np01-e-criterion-a2-check/manifest.txt",
    "docs/development-evidence/render-bvh/r07-splat-precheck/README.md",
    "docs/development-evidence/criteria-purpose-audit/manifest.txt",
    "docs/development-evidence/g10-p5-mesh-cap20/manifest.txt",
    "scripts/validate/repo-write-lock.ps1",
    "src/lib.rs",
]
same = 0
for p in samples:
    same += sh("rev-parse", BEFORE + ":" + p).strip() == sh("rev-parse", RULE + ":" + p).strip()
ok("blob ids identical in sample", same == len(samples), "%d/%d" % (same, len(samples)))

print("\n[4] commit scope")
ns = [l for l in sh("diff-tree", "-r", "--name-status", BEFORE, RULE).splitlines()]
expected_prefix = (".gitattributes", ".normify/changes/2026-11-05-repo-evidence-gitattributes-rule.json",
                   ".normify/modules/mjwarp/index.md", ".normify/runs/2026-11-05-repo-evidence-gitattributes-rule/",
                   "docs/development-evidence/repo-evidence-gitattributes-rule/")
bad = [l for l in ns if not any(l.split("\t", 1)[1].startswith(p) for p in expected_prefix)]
ok("only expected paths", not bad, repr(bad))
ok("no src/tests/fixtures/docs/evidence changes",
   not any("\t" + p in "\n".join(ns) for p in ("src/", "tests/", "fixtures/", "docs/evidence/")),
   "files=%d" % len(ns))

print("\n[5] executant evidence dir")
raw = blob("HEAD", EVID + "/manifest.txt")
n = 0
for line in raw.decode().splitlines():
    if not line.strip() or line.startswith("#"):
        continue
    h, b, rel = line.split()
    data = blob("HEAD", EVID + "/" + rel)          # entries are relative to this dir
    ok("manifest entry " + rel, sha(data) == h and str(len(data)) == b)
    n += 1
ok("manifest entries = 8", n == 8, str(n))
rec = json.loads(blob("HEAD", EVID + "/receipt.json").decode())
ok("receipt task_id", rec.get("task_id") == "2026-11-05-repo-evidence-gitattributes-rule")
card = json.loads(blob("HEAD", ".normify/changes/2026-11-05-repo-evidence-gitattributes-rule.json").decode())
ok("executant card verified", card["status"] == "verified", card["status"])
readme = blob("HEAD", EVID + "/README.md").decode()
for token in ("不 renormalize", "回滚", "效果范围", "git checkout --"):
    ok("README documents: " + token, token in readme)
obs("manifest header says 'repo-relative path' but entries are relative to the evidence dir",
    "self-check requires base dir; other evidence manifests use repo-relative paths")

print("\n[6] .git/info/attributes")
info = os.path.join(REPO, ".git", "info", "attributes")
ok(".git/info/attributes not committed", sh("ls-files", ".git/info/attributes").strip() == "")
obs(".git/info/attributes exists (pre-existing)", "exists=%s content=%r" % (
    os.path.exists(info), open(info, encoding="utf-8").read() if os.path.exists(info) else ""))

print("\n[7] advisory: R07 fix", R07)
rp = "docs/development-evidence/render-bvh/r07-splat-precheck/README.md"
old, new = blob(R07 + "^", rp), blob(R07, rp)
ok("R07 README LF-only (CR=0, TAB=0)", new.count(b"\r") == 0 and new.count(b"\t") == 0,
   "CR=%d TAB=%d" % (new.count(b"\r"), new.count(b"\t")))
ok("R07 backticks restored", "`transmittance" in new.decode() and "`receipt.json`" in new.decode())
ok("R07 §3 unchanged vs 24244bc2",
   [l for l in old.decode().splitlines() if l.startswith("- **`shade_splats`**")]
   == [l for l in new.decode().splitlines() if l.startswith("- **`shade_splats`**")])
ok("R07 receipt untouched",
   blob(R07 + "^", "docs/development-evidence/render-bvh/r07-splat-precheck/receipt.json")
   == blob(R07, "docs/development-evidence/render-bvh/r07-splat-precheck/receipt.json"))

print("\n[8] advisory: G10 cosmetic", G10)
rr = "docs/development-evidence/g10-p5-summary/receipt.json"
old, new = blob(G10 + "^", rr), blob(G10, rr)
old_l, new_l = old.decode().splitlines(), new.decode().splitlines()
diffl = [(a, b) for a, b in zip(old_l, new_l) if a != b]
ok("G10 only indentation changed", len(diffl) == 1 and diffl[0][1] == "  " + diffl[0][0],
   "diff_lines=%d" % len(diffl))
ok("G10 receipt JSON valid + counts unchanged",
   json.loads(old)["requirement_counts"] == json.loads(new)["requirement_counts"])
ok("G10 README untouched by cosmetic commit", "README.md" not in sh("show", "--name-only", "--format=", G10))

print("\n[9] addenda closure: executant evidence fixed by 1bfc4ac8")
FIXC = "1bfc4ac8"
ok("rule blob unchanged since ce4a7d18",
   sh("rev-parse", "HEAD:.gitattributes").strip() == sh("rev-parse", RULE + ":.gitattributes").strip())
ok("fix commit does not touch .gitattributes",
   ".gitattributes" not in [l.split("\t")[-1] for l in sh("diff-tree", "-r", "--name-only", FIXC + "^", FIXC).splitlines()])
man = blob("HEAD", EVID + "/manifest.txt").decode()
ok("manifest header says evidence-dir-relative", "evidence-dir-relative path" in man.splitlines()[0], man.splitlines()[0])
ok("manifest declares base dir", ("# base dir: " + EVID) in man)
n = okc = 0
for line in man.splitlines():
    if not line.strip() or line.startswith("#"):
        continue
    h, b, rel = line.split()
    data = blob("HEAD", EVID + "/" + rel)
    okc += (sha(data) == h and str(len(data)) == b)
    n += 1
ok("executant manifest re-checked 8/8 with base dir", n == 8 and okc == 8, "%d/%d" % (okc, n))
rec = json.loads(blob("HEAD", EVID + "/receipt.json").decode())
g = rec.get("git_info_attributes") or {}
actual = open(os.path.join(REPO, ".git", "info", "attributes"), encoding="utf-8").read().splitlines()
ok("receipt git_info_attributes.exists_at_capture=true", g.get("exists_at_capture") is True)
ok("receipt lines match actual .git/info/attributes", g.get("lines") == actual, repr(actual))
ok("receipt changed_by_task=false", g.get("changed_by_task") is False)
ok("receipt evidence_revision.base_commit=ce4a7d18",
   (rec.get("evidence_revision") or {}).get("base_commit") == "ce4a7d18")

print("\nRESULT:", "PASS" if not fails else "FAIL", "fails=" + ",".join(fails))
raise SystemExit(0 if not fails else 1)
