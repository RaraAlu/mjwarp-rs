"""生成本证据目录的 artifacts sha256 manifest（排除不入库的 .bin 目录与 pyc）。"""

import hashlib
import json
from pathlib import Path

EVIDENCE = Path(__file__).resolve().parent
MANIFEST = EVIDENCE / "artifacts-sha256.json"

# 不入库：qa-gpu/cases/** 与 qa-gpu/out/**（.bin 输入/输出）、__pycache__、*.pyc
EXCLUDED_PREFIXES = ("qa-gpu/cases/", "qa-gpu/out/", "__pycache__/")


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def excluded(rel):
    rel = rel.as_posix()
    if rel.startswith(EXCLUDED_PREFIXES):
        return True
    return rel.endswith(".pyc")


def main():
    entries = {}
    for path in sorted(EVIDENCE.rglob("*")):
        if not path.is_file():
            continue
        rel = path.relative_to(EVIDENCE)
        if path.resolve() == MANIFEST.resolve() or excluded(rel):
            continue
        entries[rel.as_posix()] = {
            "bytes": path.stat().st_size,
            "sha256": sha256(path),
        }
    value = {
        "schema": 1,
        "task_id": "2026-10-25-g10-p5-gpu-kernels-independent-check",
        "note": "本目录入库证据文件 sha256（manifest 自身、qa-gpu/cases/**、qa-gpu/out/**、__pycache__/*.pyc 除外）",
        "files": entries,
    }
    MANIFEST.write_text(json.dumps(value, ensure_ascii=False, indent=1) + "\n",
                        encoding="utf-8")
    print(f"wrote {MANIFEST.name}: {len(entries)} files")


if __name__ == "__main__":
    main()
