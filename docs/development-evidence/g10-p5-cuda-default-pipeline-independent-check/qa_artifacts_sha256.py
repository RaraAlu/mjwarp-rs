"""QA 证据清单：为证据目录下所有入库文件生成 sha256 清单。

输出 `artifacts-sha256.json`（path -> {bytes, sha256}），用于交付前自检与会话后追溯。
不包含：本脚本自身输出、`*.pyc`、`__pycache__`、`target/`、dumper 的构建产物。
"""

import hashlib
import json
import pathlib
import sys

EVIDENCE = pathlib.Path(__file__).resolve().parent
SKIP_NAMES = {"artifacts-sha256.json", "artifacts-sha256.txt"}


def main():
    files = {}
    for path in sorted(EVIDENCE.rglob("*")):
        if not path.is_file():
            continue
        rel = path.relative_to(EVIDENCE)
        parts = set(rel.parts)
        if parts & {"__pycache__", "target"}:
            continue
        if path.suffix in {".pyc", ".bin", ".o", ".obj"}:
            continue
        if path.name in SKIP_NAMES:
            continue
        data = path.read_bytes()
        files[rel.as_posix()] = {
            "bytes": len(data),
            "sha256": hashlib.sha256(data).hexdigest(),
        }
    payload = {"root": "docs/development-evidence/g10-p5-cuda-default-pipeline-independent-check",
               "file_count": len(files), "files": files}
    text = json.dumps(payload, indent=2, ensure_ascii=False) + "\n"
    (EVIDENCE / "artifacts-sha256.json").write_text(text, encoding="utf-8")
    print(text)
    return 0


if __name__ == "__main__":
    sys.exit(main())
