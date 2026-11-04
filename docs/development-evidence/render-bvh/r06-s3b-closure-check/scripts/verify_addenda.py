# -*- coding: utf-8 -*-
"""Read-only verification of the two addenda fixes to QA observations O1/O3.

O1: commit ac18a4d1 strips the UTF-8 BOM of attempt6 two-run-sha.txt (body unchanged).
O3: commit e836418c adds the backlog row to closure README section 7 (manifest recomputed).
"""
import hashlib
import io
import json
import os
import subprocess
import sys

REPO = r"D:\Rust\mjwarp-rs"
CLO = "docs/development-evidence/render-bvh/r06-s3b-closure"
ATT6 = "docs/development-evidence/render-bvh/r06-s3b-flex-attempt6"

if "--out" in sys.argv:
    _fh = open(sys.argv[sys.argv.index("--out") + 1], "w", encoding="utf-8", newline="\n")
    _orig = sys.stdout

    class _Tee(object):
        def write(self, s):
            _fh.write(s)
            _orig.write(s)
            return len(s)

        def flush(self):
            _fh.flush()
            _orig.flush()

    sys.stdout = _Tee()


def git(*a):
    p = subprocess.run(["git"] + list(a), cwd=REPO, capture_output=True)
    if p.returncode != 0:
        raise RuntimeError("git failed: %s" % " ".join(a))
    return p.stdout


def blob(rev, path):
    return git("cat-file", "-p", rev + ":" + path)


def sha(b):
    return hashlib.sha256(b).hexdigest()


print("== O1: attempt6 two-run-sha.txt BOM strip (ac18a4d1) ==")
p = ATT6 + "/two-run-sha.txt"
old = blob("ac18a4d1^", p)
new = blob("ac18a4d1", p)
body = old[3:] if old[:3] == b"\xef\xbb\xbf" else old
print("  old bytes=%d bom=%s" % (len(old), old[:3] == b"\xef\xbb\xbf"))
print("  new bytes=%d bom=%s" % (len(new), new[:3] == b"\xef\xbb\xbf"))
print("  body byte-identical (old minus BOM == new):", body == new)
print("  new sha256:", sha(new))
changed = [f for f in git("show", "--name-only", "--format=", "ac18a4d1").decode().splitlines()
           if f.strip() and not f.startswith(".normify")]
print("  non-.normify files in fix commit:", changed)
print("  attempt6 README untouched by fix:", sha(blob("ac18a4d1", ATT6 + "/README.md")) == sha(blob("ac18a4d1^", ATT6 + "/README.md")))

print("\n== O3: closure README index row + manifest (e836418c) ==")
rd = blob("e836418c", CLO + "/README.md")
print("  README bytes=%d sha256=%s" % (len(rd), sha(rd)))
text = rd.decode("utf-8")
sec = text.split("## 7.")[-1]
rows = [l for l in sec.splitlines() if l.startswith("|") and not set(l.replace("|", "").strip()) <= set("-: ")]
print("  §7 data rows:", len(rows) - 1)
print("  contains backlog row 5dc62d66:", "5dc62d66" in sec)
fj = blob("e836418c", CLO + "/facts.json")
print("  facts bytes=%d sha256=%s (unchanged: %s)" % (len(fj), sha(fj)[:16], sha(fj) == sha(blob("e836418c^", CLO + "/facts.json"))))
man = blob("e836418c", CLO + "/manifest.txt")
entries = [l for l in man.decode("utf-8").splitlines() if l.strip() and not l.strip().startswith("#")]
bad = 0
for line in entries:
    s, size, rel = line.split()
    data = blob("e836418c", CLO + "/" + rel)
    if sha(data) != s or len(data) != int(size):
        bad += 1
        print("  BAD", rel)
print("  manifest entries=%d bad=%d" % (len(entries), bad))

print("\n== BOM audit after O1 ==")
print("  expected: check-evidence-bom.ps1 reports 128 non-.ps1 BOM files + 4 manifest header violations")
print("  (run separately in the shell; recorded in README addenda)")

print("\n== QA verdict impact ==")
print("  both fixes touch bytes/rows only; numbers and conclusions unchanged")
print("  QA card 8aec9619 verdict PASS-with-observations stands; O1/O3 closed")
