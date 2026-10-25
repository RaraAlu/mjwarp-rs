"""生成本证据目录的 artifacts sha256 manifest（排除 manifest 自身与 pyc）。"""

import hashlib
import json
from pathlib import Path

EVIDENCE = Path(__file__).resolve().parent
MANIFEST = EVIDENCE / "artifacts-sha256.json"


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    entries = {}
    for path in sorted(EVIDENCE.rglob("*")):
        if not path.is_file():
            continue
        if path.resolve() == MANIFEST.resolve():
            continue
        if path.suffix == ".pyc" or "__pycache__" in path.parts:
            continue
        rel = path.relative_to(EVIDENCE).as_posix()
        entries[rel] = {"bytes": path.stat().st_size, "sha256": sha256(path)}
    value = {
        "schema": 1,
        "task_id": "2026-10-25-g10-p5-nvrtc-product-launcher-independent-check",
        "note": "本目录入库证据文件 sha256（manifest 自身与 pyc 除外）",
        "files": entries,
    }
    MANIFEST.write_text(json.dumps(value, ensure_ascii=False, indent=1) + "\n",
                        encoding="utf-8")
    print(f"wrote {MANIFEST.name}: {len(entries)} files")


if __name__ == "__main__":
    main()
