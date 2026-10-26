"""Align a disposable detached worktree with the main worktree's *bytes* for every
tracked file that the main tree has not modified (so EOL-sensitive audit tests read
the same bytes they read in the main tree), while leaving HEAD content in place for
files the main tree has dirty (in-flight M2 work must NOT leak into the gate tree).

Safety: only files that `git -C <wt> ls-files` lists (tracked) are touched; every
write target is verified to be inside the worktree before writing.
"""
import subprocess
import sys
from pathlib import Path

MAIN = Path(r"D:\Rust\mjwarp-rs")
WT = Path(r"D:\Rust\mjwarp-rs\target\qa-m0afix\wt-head")

dirty_out = subprocess.run(
    ["git", "-C", str(MAIN), "status", "--porcelain", "-uall"],
    capture_output=True, text=True, encoding="utf-8", errors="replace", check=True,
).stdout
dirty = set()
for line in dirty_out.splitlines():
    if len(line) > 3:
        dirty.add(line[3:].strip().replace("\\", "/"))

tracked = subprocess.run(
    ["git", "-C", str(WT), "ls-files"],
    capture_output=True, text=True, encoding="utf-8", errors="replace", check=True,
).stdout.splitlines()

copied = 0
skipped_dirty = 0
same = 0
for rel in tracked:
    rel = rel.strip().replace("\\", "/")
    if not rel:
        continue
    if rel in dirty:
        skipped_dirty += 1
        continue
    src = MAIN / rel
    dst = WT / rel
    if not src.is_file() or not dst.is_file():
        continue
    src_bytes = src.read_bytes()
    dst_bytes = dst.read_bytes()
    if src_bytes == dst_bytes:
        same += 1
        continue
    resolved = dst.resolve()
    if not str(resolved).startswith(str(WT.resolve())):
        raise SystemExit(f"refusing to write outside worktree: {resolved}")
    dst.write_bytes(src_bytes)
    copied += 1

print(f"tracked={len(tracked)} same={same} copied={copied} skipped_dirty={skipped_dirty}")
print("dirty entries in main tree:", len(dirty))
