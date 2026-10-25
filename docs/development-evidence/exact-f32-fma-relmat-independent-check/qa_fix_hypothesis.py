# -*- coding: utf-8 -*-
"""Fix-hypothesis probe: locally patch the two `c.bits as f32` casts in a COPY of
the product snapshot and re-run the same independent probe. This confirms the
root cause and that a 2-line fix suffices. The product tree is NOT modified.
"""
import json
import shutil
import subprocess
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
TASK = REPO / "docs/development-evidence/exact-f32-fma-relmat-independent-check"
SRC = TASK / "probe" / "fma_snapshot.rs"
FIXED = TASK / "probe-fixed"
FIXED.mkdir(exist_ok=True)

text = SRC.read_text(encoding="utf-8")
needle = "return c.bits as f32;"
replacement = "return f32::from_bits(c.bits);"
count = text.count(needle)
lines_before = text.splitlines()
patched = text.replace(needle, replacement)
lines_after = patched.splitlines()
changed = [
    index + 1
    for index, (before, after) in enumerate(zip(lines_before, lines_after))
    if before != after
]
(FIXED / "fma_snapshot.rs").write_text(patched, encoding="utf-8")
shutil.copyfile(TASK / "probe" / "qa_fma_probe.rs", FIXED / "qa_fma_probe.rs")
shutil.copyfile(TASK / "probe" / "tests.rs", FIXED / "tests.rs")

out = REPO / "target/qa-fma-check/probe"
exe = out / "probe-fixed-gnu-default.exe"
compile_cmd = [
    "rustc", "--edition", "2024", "--test", "-C", "opt-level=0", "-C", "debug-assertions=on",
    "-o", str(exe), str(FIXED / "qa_fma_probe.rs"),
]
compiled = subprocess.run(compile_cmd, capture_output=True, text=True)
if compiled.returncode != 0:
    raise SystemExit(compiled.stderr[:2000])
run = subprocess.run([str(exe), "--nocapture", "--test-threads=1"], capture_output=True, text=True)
log = TASK / "probe-logs" / "probe-fixed-gnu-default.log"
log.write_text(run.stdout + run.stderr, encoding="utf-8")

record = {
    "purpose": "fix hypothesis: `c.bits as f32` -> `f32::from_bits(c.bits)` in a COPY of the product snapshot",
    "product_tree_modified": False,
    "replacements": count,
    "changed_line_numbers": changed,
    "compile_command": " ".join(compile_cmd),
    "run_exit": run.returncode,
    "log": "probe-logs/probe-fixed-gnu-default.log",
}
(TASK / "qa-fix-hypothesis.json").write_text(json.dumps(record, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps(record, ensure_ascii=False))
for line in run.stdout.splitlines():
    if line.startswith(("SUMMARY", "STAT boundary", "RELMAT_CHAIN", "SEAM_ENV_CHECK")):
        print(line)
