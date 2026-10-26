"""把 GPU 参考 JSON 的接触行按稳定键重排，得到可跨进程逐字节比较的规范形。

用途：warp GPU 后端写接触数组用 atomic 计数，raw 行序可能随进程/调度变化；两轮独立进程的
「整文件 sha 相同」要求对 row-order 做规范化。排序键 = INT_FIELDS 行内容 + FLOAT_FIELDS 的 IEEE 位型
（全序、确定性），只改 `output.fields` 的行序，其余字段（provenance/输入/计数）原样保留。

用法：python -B canonicalize.py <input.json> <output.json>
"""

import json
import struct
import sys
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

FLOAT_FIELDS = ("dist", "pos", "frame", "includemargin", "friction", "solref",
                "solreffriction", "solimp", "adhesion")
INT_FIELDS = ("dim", "geom", "flex", "elem", "vert", "worldid", "type", "geomcollisionid")


def float_bits(value):
    return struct.unpack("<I", struct.pack("<f", float(value)))[0]


def row_key(fields, index):
    key = []
    for name in INT_FIELDS:
        rows = fields[name]
        if len(rows) > index:
            key.extend(int(value) for value in rows[index])
    for name in FLOAT_FIELDS:
        rows = fields[name]
        if len(rows) > index:
            key.extend(float_bits(value) for value in rows[index])
    return tuple(key)


def main():
    source = Path(sys.argv[1])
    target = Path(sys.argv[2])
    report = json.loads(source.read_text(encoding="utf-8"))
    for case in report["cases"].values():
        output = case["output"]
        count = output["count"]
        fields = output["fields"]
        if count == 0:
            continue
        order = sorted(range(count), key=lambda index: row_key(fields, index))
        for name in FLOAT_FIELDS + INT_FIELDS:
            rows = fields[name]
            if len(rows) < count:
                continue  # 该字段对本 case 无数据（例如无 flex 接触时 flex/elem/vert 为空数组）
            fields[name] = [rows[index] for index in order]
    target.write_text(
        json.dumps(report, ensure_ascii=False, separators=(",", ":")), encoding="utf-8"
    )
    print(f"canonicalized {source} -> {target}")


if __name__ == "__main__":
    main()
