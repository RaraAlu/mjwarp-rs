# -*- coding: utf-8 -*-
"""Generate artifacts-sha256.json for the QA review evidence directory."""
import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
OUT = REPO / "docs/development-evidence/t4-source-hash-repin-13-independent-check"
MANIFEST = "artifacts-sha256.json"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


rows = []
for path in sorted(OUT.rglob("*")):
    if not path.is_file() or path.name == MANIFEST:
        continue
    rows.append({
        "path": str(path.relative_to(OUT)).replace("\\", "/"),
        "sha256": sha256(path),
        "bytes": path.stat().st_size,
    })

manifest = {
    "task": "2026-10-25-t4-source-hash-repin-13-independent-check",
    "generated_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "file_count": len(rows),
    "files": rows,
}
(OUT / MANIFEST).write_text(json.dumps(manifest, ensure_ascii=False, indent=2, sort_keys=False) + "\n", encoding="utf-8")
print(f"wrote {MANIFEST}: {len(rows)} files")
for r in rows:
    print(f"  {r['path']} {r['sha256'][:12]} {r['bytes']}")
