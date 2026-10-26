"""把 finalize_capture.ps1 输出的 `key.path=value` 行转换成 JSON（避免 PowerShell 5.1 ConvertTo-Json 的
嵌套 OrderedDictionary 缺陷）。value 以 `[`/`{` 开头时按 JSON 解析，`true`/`false` 转 bool，整数/浮点转数值。

用法：python -B kv_to_json.py <input.kv> <output.json>
"""

import json
import sys
from pathlib import Path


def parse(value):
    if value.startswith("[") or value.startswith("{"):
        return json.loads(value)
    if value == "true":
        return True
    if value == "false":
        return False
    try:
        return int(value)
    except ValueError:
        pass
    try:
        return float(value)
    except ValueError:
        return value


def main():
    source = Path(sys.argv[1])
    target = Path(sys.argv[2])
    root = {}
    for line in source.read_text(encoding="utf-8").splitlines():
        if not line or "=" not in line:
            continue
        path, value = line.split("=", 1)
        node = root
        keys = path.split(".")
        for key in keys[:-1]:
            node = node.setdefault(key, {})
        node[keys[-1]] = parse(value)
    target.write_text(json.dumps(root, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"wrote {target}")


if __name__ == "__main__":
    main()
