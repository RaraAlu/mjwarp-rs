"""QA G0b review: write sha256 manifest of the QA evidence directory (manifest excludes itself)."""

import hashlib
import json
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
EV = ROOT / "docs/development-evidence/g10-p5-fixtures-independent-check"
OUT = EV / "qa-artifact-manifest.json"

files = []
for path in sorted(EV.rglob("*")):
    if not path.is_file() or path == OUT:
        continue
    raw = path.read_bytes()
    files.append({
        "path": str(path.relative_to(EV)).replace("\\", "/"),
        "bytes": len(raw),
        "sha256": hashlib.sha256(raw).hexdigest(),
    })
doc = {"schema": 1, "task_id": "2026-10-25-g10-p5-fixtures-independent-check",
       "note": "sha256 of the QA review artifacts in this directory; manifest itself excluded",
       "files": files}
OUT.write_text(json.dumps(doc, ensure_ascii=False, indent=1), encoding="utf-8")
print("files:", len(files), "manifest:", OUT)
