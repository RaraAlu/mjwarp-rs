# -*- coding: utf-8 -*-
"""Normalize evidence text files to UTF-8 without BOM.

Some logs were captured by a Windows PowerShell 5.1 redirect (UTF-16LE+BOM) or written with
`Out-File -Encoding utf8` (UTF-8+BOM). Evidence must be UTF-8 without BOM, so this script
converts only those files and records what changed.
"""
from pathlib import Path

TASK = Path(r"D:\Rust\mjwarp-rs\docs\development-evidence\g10-p5-faithful-fma-fix-independent-check")
changed = []
for path in sorted(TASK.rglob("*")):
    if not path.is_file() or path.name == "artifacts-sha256.json":
        continue
    raw = path.read_bytes()
    if raw.startswith(b"\xff\xfe"):
        text = raw.decode("utf-16")
        path.write_text(text, encoding="utf-8")
        changed.append(f"{path.relative_to(TASK)}: utf-16 -> utf-8")
    elif raw.startswith(b"\xef\xbb\xbf"):
        path.write_bytes(raw[3:])
        changed.append(f"{path.relative_to(TASK)}: utf-8-bom -> utf-8")
(TASK / "logs" / "encoding-normalized.txt").write_text(
    "\n".join(changed) + "\n" if changed else "(no conversion needed)\n", encoding="utf-8")
print(f"converted={len(changed)}")
for line in changed:
    print(line)
