# -*- coding: utf-8 -*-
"""QA read-only check of allowlist assert fix commit 8c15ad75. No repo writes."""
import hashlib
import io
import json
import subprocess
import sys

REPO = r"D:\Rust\mjwarp-rs"
C = "8c15ad75"
EV = "docs/development-evidence/render-bvh/r06-s3b-flex-allowlist-assert"


def git(*a, binary=False):
    p = subprocess.run(["git"] + list(a), cwd=REPO, capture_output=True)
    if p.returncode != 0:
        raise RuntimeError("git failed: %s %s" % (" ".join(a), p.stderr.decode("utf-8", "replace")))
    return p.stdout if binary else p.stdout.decode("utf-8", "replace")


def blob(path):
    return git("cat-file", "-p", "%s:%s" % (C, path), binary=True)


def sha(b):
    return hashlib.sha256(b).hexdigest()


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

print("== 0. commit ==")
print(" ", git("log", "-1", "--format=%H", C).strip())
files = [l for l in git("show", "--name-only", "--format=", C).splitlines() if l.strip()]
print("  files:", len(files))
for f in files:
    print("   ", f)

print("\n== 1. evidence manifest self-check ==")
man = blob(EV + "/manifest.txt")
print("  manifest bytes=%d sha=%s first=%r" % (len(man), sha(man), chr(man[0])))
entries = [l for l in man.decode("utf-8").splitlines() if l.strip() and not l.strip().startswith("#")]
bad = 0
for line in entries:
    s, size, rel = line.split()
    data = blob(EV + "/" + rel)
    if sha(data) != s or len(data) != int(size):
        bad += 1
        print("  BAD", rel)
print("  entries=%d bad=%d" % (len(entries), bad))
# artifact allowlist.rs must equal the committed source
src = blob("tests/support/render_bvh/allowlist.rs")
art = blob(EV + "/artifacts/allowlist.rs")
print("  evidence artifacts/allowlist.rs == tests/support/render_bvh/allowlist.rs:", sha(src) == sha(art), "bytes", len(src))

print("\n== 2. diff artifact check (parent test file + diff == committed test file) ==")
parent = git("rev-parse", C + "^").strip()
old = git("cat-file", "-p", "%s:tests/render_flex_validate.rs" % parent, binary=True)
new = blob("tests/render_flex_validate.rs")
diff = blob(EV + "/artifacts/render-flex-validate.diff")
d = diff.decode("utf-8", "replace")
print("  diff bytes:", len(diff))
print("  diff removes old strict depth assert:", '-        assert_eq!((mismatched, beyond), (0, 0), "cam{} depth' in d)
print("  diff adds allowlist module wiring:", '+mod allowlist;' in d or '+#[path = "support/render_bvh/allowlist.rs"]' in d)
print("  diff replaces compare() call:", 'allowlist::assert_depth_allowlisted(camera.index' in d)
print("  diff contains no support-module content:", 'render_bvh/allowlist.rs' not in d.split("diff --git")[1] if "diff --git" in d else True)

print("\n== 3. line counts / 500-line rule ==")
for f in ("tests/render_flex_validate.rs", "tests/support/render_bvh/allowlist.rs"):
    t = blob(f).decode("utf-8")
    print("  %-46s lines=%d" % (f, len(t.splitlines())))

print("\n== 4. explicit constants / scoping / fail messages ==")
sup = src.decode("utf-8")
for pat in ("ALLOWLIST_CAMERA: usize = 2", "ALLOWLIST_PIXEL_X: usize = 134", "ALLOWLIST_PIXEL_Y: usize = 128",
            "ALLOWLIST_FLEXID: i32 = 0", "ALLOWLIST_SEG_TYPE: i32 = 9",
            "assert_depth_allowlisted", "mismatched={strict_mismatched}", "bits={bits:#010x}",
            "ULP", "beyond4"):
    print("  %-40s %s" % (pat, pat in sup))
print("  camera==ALLOWLIST_CAMERA gate:", "camera == ALLOWLIST_CAMERA" in sup)
print("  seg guard flexid/type:", "[ALLOWLIST_FLEXID, ALLOWLIST_SEG_TYPE]" in sup)
print("  tests in support module:", sup.count("#[test]"))

print("\n== 5. test file wiring ==")
tf = new.decode("utf-8")
print("  #[path] mod allowlist:", '#[path = "support/render_bvh/allowlist.rs"]' in tf and "mod allowlist;" in tf)
print("  calls allowlist::assert_depth_allowlisted:", "allowlist::assert_depth_allowlisted" in tf)
print("  old inline allowlist impl removed:", "const ALLOWLIST_CAMERA" not in tf)
print("  old strict assert removed (mismatched, beyond), (0, 0) for depth):", '(mismatched, beyond), (0, 0), "cam{} depth' not in tf)

print("\n== 6. gates in commit ==")
for g in ("fmt", "clippy", "clippy-cuda", "unit"):
    p = ".normify/runs/2026-11-04-render-r06-s3b-flex-allowlist-assert-fix-r2/%s.json" % g
    if p not in files:
        print("  %-11s MISSING" % g)
        continue
    j = json.loads(blob(p).decode("utf-8"))
    extra = ""
    if g == "unit":
        t = j.get("tests", {})
        extra = " tests=%s/%s" % (t.get("passed"), t.get("failed"))
    print("  %-11s pass=%s exit=%s at=%s%s" % (g, j.get("passed"), j.get("exit"), j.get("at"), extra))

print("\n== 7. evidence README + selftest log claims ==")
rd = blob(EV + "/README.md").decode("utf-8")
log = blob(EV + "/logs/cpu-selftest-run.log")
txt = log.decode("utf-16") if log[:2] == b"\xff\xfe" else log.decode("utf-8", "replace")
print("  README bytes:", len(rd.encode("utf-8")))
for pat in ("allowlist", "cam2", "134", "128", "bit-first", "CPU"):
    print("  README contains %-10s %s" % (pat, pat in rd))
print("  log encoding:", "UTF-16LE" if log[:2] == b"\xff\xfe" else "UTF-8", "bytes:", len(log))
for pat in ("allowlist_pixel_within_4_ulp_passes_cpu", "allowlist_pixel_beyond_4_ulp_fails_cpu",
            "non_allowlisted_pixel_mismatch_fails_cpu", "allowlist_is_camera_scoped_cpu"):
    print("  log contains %-46s %s" % (pat, pat in txt))
print("  log test result lines:", [l.strip() for l in txt.splitlines() if "test result:" in l])
print("  log 'running N tests':", [l.strip() for l in txt.splitlines() if l.strip().startswith("running")])

print("\n== 8. card status ==")
card = json.loads(blob(".normify/changes/2026-11-04-render-r06-s3b-flex-allowlist-assert-fix-r2.json").decode("utf-8"))
print("  fix-r2 status=%s closed_at=%s files=%s" % (card["status"], card.get("closed_at"), card["schedule"]["files"]))
