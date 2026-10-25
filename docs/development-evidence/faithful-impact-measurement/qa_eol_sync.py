# -*- coding: utf-8 -*-
"""Align the pinned worktree working tree with the main checkout's bytes.

Why: a fresh `git worktree add` on this machine checks text files out with
CRLF (core.autocrlf=true), while the verified environment's main checkout holds
a mix of LF/CRLF bytes. Tests that hash working-tree bytes (e.g. fixture capture
scripts) then fail for EOL only.

Method: copy every TRACKED file from the main checkout into the worktree, except
tracked files that are modified in the main checkout (those must not leak the
in-flight relmat work into this read-only measurement). Untracked main-tree files
are never copied. The worktree commit stays 5e000cb9; only working-tree bytes
(EOL/identical-blob files) are aligned with the verified checkout.
"""
import shutil
import subprocess
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
TREE = REPO / "target/worktrees/faithful-5e000cb9"
LOG = REPO / "docs/development-evidence/faithful-impact-measurement/logs/eol-sync.txt"


def run(args, cwd):
    return subprocess.run(args, cwd=cwd, capture_output=True, text=True).stdout


tracked = [p for p in run(["git", "ls-files", "-z"], TREE).split("\0") if p]
main_status = run(["git", "status", "--porcelain", "-z", "--untracked-files=no"], REPO)
modified = set()
entries = [e for e in main_status.split("\0") if e]
i = 0
while i < len(entries):
    entry = entries[i]
    code, path = entry[:2], entry[3:]
    if "R" in code or "C" in code:
        i += 1  # paired source path follows
    modified.add(path.strip('"'))
    i += 1

copied = 0
skipped = []
missing = 0
copy_errors = []
for rel in tracked:
    main_path = REPO / rel
    wt_path = TREE / rel
    if rel in modified or not main_path.is_file():
        skipped.append(rel)
        if not main_path.is_file():
            missing += 1
        continue
    try:
        if wt_path.is_file() and wt_path.read_bytes() == main_path.read_bytes():
            continue
        shutil.copyfile(main_path, wt_path)
        copied += 1
    except (FileNotFoundError, OSError) as exc:
        # Windows MAX_PATH / very long authoring-evidence paths only.
        copy_errors.append(f"{rel}\t{exc}")

capture = "scripts/authoring/g10-p5/capture_sdf_rotation_references.py"
capture_bytes = (TREE / capture).read_bytes()
import hashlib
digest = hashlib.sha256(capture_bytes).hexdigest()

lines = [
    f"worktree={TREE}",
    f"worktree_head={run(['git', 'rev-parse', 'HEAD'], TREE).strip()}",
    "purpose=working-tree bytes aligned to verified main checkout (tracked files only; in-flight modified tracked files excluded)",
    f"tracked_files={len(tracked)}",
    f"copied_or_replaced={copied}",
    f"skipped_modified_in_main={len(skipped)}",
    f"missing_in_main={missing}",
    f"copy_errors_long_path={len(copy_errors)}",
    f"capture_script_sha256_after_sync={digest}",
    "",
    "## skipped (modified in main checkout; not copied)",
]
lines += sorted(skipped)
lines += ["", "## copy errors (long paths; not needed by measurement tests)"]
lines += copy_errors
LOG.parent.mkdir(parents=True, exist_ok=True)
LOG.write_text("\n".join(lines) + "\n", encoding="utf-8")
print("\n".join(lines[:8]))
