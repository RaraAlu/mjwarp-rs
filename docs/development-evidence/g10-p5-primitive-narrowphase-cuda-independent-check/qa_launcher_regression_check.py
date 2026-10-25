"""QA 独立回归核对：16 段拼装后的产品 launcher 输出 vs G5 探针 `out1`（逐字节）。

背景：本片把设备源从 14 段扩到 16 段（primitive/primitive_wrapper），需独立确认既有
三条 launcher case 未被拼装/PTX 变化带偏。做法与 NVRTC 片一致：QA 自己的 dumper 用
产品公开 API 重新落盘，再与本脚本逐字节比较 **G5 窗口的 `out1`**（不作任何容差，
因为同配方此前已到达全位级；有差异即报告）。

用法：
  python qa_launcher_regression_check.py --dumps target/qa-primitive/prod-out
  python qa_launcher_regression_check.py --dumps ... --case hfield-sphere
"""

import argparse
import json
import pathlib
import sys

REPO = pathlib.Path(r"D:\Rust\mjwarp-rs")
G5_CASES = REPO / "docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases"
DEFAULT_DUMPS = REPO / "target/qa-primitive/prod-out"
CASES = ["hfield-sphere", "sdf-box", "ellipsoid-box"]
INT_FIELDS = {"dim": 1, "geom": 2,
              "efc_address": None, "worldid": 1, "geomcollisionid": 1, "type": 1}
COUNT_FIELDS = {"nacon": 1, "nccd": 1, "overflow": None}
F32_FIELDS = {"dist": 1, "pos": 3, "frame": 9, "includemargin": 1, "friction": 5,
              "solref": 2, "solreffriction": 2, "solimp": 5, "adhesion": 1}


def read_meta(case_dir):
    meta = {}
    for line in (case_dir / "meta.txt").read_text(encoding="utf-8").splitlines():
        parts = line.split()
        if len(parts) == 2:
            meta[parts[0]] = parts[1]
    return meta


def compare_case(case, dumps):
    ref_dir = G5_CASES / case / "out1"
    out_dir = pathlib.Path(dumps) / case
    meta = read_meta(G5_CASES / case)
    nworld = int(meta["nworld"])
    efc_width = int(meta["efc_width"])
    ref_nacon = int.from_bytes((ref_dir / "nacon.bin").read_bytes(), "little", signed=True)
    numerator = 0
    total = 0
    failures = []
    fields = {}
    # 计数类：只核对整值相等，不计入 bit 分母（分母 = nworld(overflow) + nacon*(35+efc)）。
    counts = {}
    for name in COUNT_FIELDS:
        dev_path = out_dir / f"{name}.bin"
        if not dev_path.exists():
            failures.append(f"{name}.bin 缺失（QA dump）")
            continue
        dev_bytes = dev_path.read_bytes()
        ref_bytes = (ref_dir / f"{name}.bin").read_bytes()
        counts[name] = {"device": int.from_bytes(dev_bytes, "little", signed=True),
                        "reference": int.from_bytes(ref_bytes, "little", signed=True),
                        "bytes_equal": dev_bytes == ref_bytes}
        if name == "overflow":
            n = len(ref_bytes) // 4
            pairs = min(len(dev_bytes), len(ref_bytes)) // 4
            equal = sum(1 for i in range(pairs)
                        if dev_bytes[i * 4:(i + 1) * 4] == ref_bytes[i * 4:(i + 1) * 4])
            fields[name] = {"components": n, "bit_equal": equal,
                            "bytes_equal": dev_bytes == ref_bytes}
            numerator += equal
            total += n
            if dev_bytes != ref_bytes:
                failures.append(f"overflow 位级不等 dev={dev_bytes.hex()} ref={ref_bytes.hex()}")
        elif dev_bytes != ref_bytes:
            failures.append(f"{name} 不等 dev={counts[name]['device']} "
                            f"ref={counts[name]['reference']}")
    for name in list(F32_FIELDS) + list(INT_FIELDS):
        width = F32_FIELDS.get(name, INT_FIELDS.get(name))
        dev = (out_dir / f"{name}.bin")
        ref = (ref_dir / f"{name}.bin")
        if not dev.exists():
            failures.append(f"{name}.bin 缺失（QA dump）")
            continue
        dev_bytes = dev.read_bytes()
        ref_bytes = ref.read_bytes()
        count = len(ref_bytes) // 4
        if name == "efc_address":
            count = ref_nacon * efc_width
            ref_bytes = ref_bytes[:count * 4]
            dev_bytes = dev_bytes[:count * 4]
        delta = abs(len(dev_bytes) - len(ref_bytes))
        if delta:
            failures.append(f"{name}.bin 长度不符 dev={len(dev_bytes)} ref={len(ref_bytes)}")
        pairs = min(len(dev_bytes), len(ref_bytes)) // 4
        equal = sum(
            1 for i in range(pairs)
            if dev_bytes[i * 4:(i + 1) * 4] == ref_bytes[i * 4:(i + 1) * 4]
        )
        fields[name] = {"components": count, "bit_equal": equal,
                        "bytes_equal": dev_bytes == ref_bytes}
        numerator += equal
        total += count
        if not dev_bytes == ref_bytes:
            for i in range(pairs):
                if dev_bytes[i * 4:(i + 1) * 4] != ref_bytes[i * 4:(i + 1) * 4]:
                    failures.append(
                        f"{name}[{i}] 位级不等 dev={dev_bytes[i * 4:(i + 1) * 4].hex()} "
                        f"ref={ref_bytes[i * 4:(i + 1) * 4].hex()}")
    denominator = nworld + ref_nacon * (35 + efc_width)
    return {
        "case": case,
        "nacon": [counts.get("nacon", {}).get("device"), ref_nacon],
        "nccd": [counts.get("nccd", {}).get("device"),
                 int.from_bytes((ref_dir / "nccd.bin").read_bytes(), "little", signed=True)],
        "denominator_recomputed": denominator,
        "bit_equal": numerator,
        "bit_total": total,
        "matches_denominator": total == denominator,
        "counts": counts,
        "fields": fields,
        "failures": failures,
    }


def main():
    try:
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    except Exception:
        pass
    parser = argparse.ArgumentParser()
    parser.add_argument("--dumps", default=str(DEFAULT_DUMPS))
    parser.add_argument("--case", default=None)
    parser.add_argument("--json", default=None)
    parser.add_argument("--numeric-only", action="store_true",
                        help="窗口内用：只打印摘要 + 失败明细，失败退出码 1")
    args = parser.parse_args()
    cases = [args.case] if args.case else CASES
    data = {"dumps": str(pathlib.Path(args.dumps)), "reference": "G5 gpu-window/cases/<case>/out1",
            "cases": []}
    failed = False
    for case in cases:
        entry = compare_case(case, args.dumps)
        data["cases"].append(entry)
        print(f"case={case} nacon={entry['nacon']} nccd={entry['nccd']} "
              f"bit_equal={entry['bit_equal']}/{entry['bit_total']} "
              f"denominator={entry['denominator_recomputed']} "
              f"failures={len(entry['failures'])}")
        for failure in entry["failures"][:12]:
            print(f"FAIL {case}: {failure}")
        if entry["failures"] or not entry["matches_denominator"] or entry["bit_equal"] != entry["bit_total"]:
            failed = True
    if args.json:
        pathlib.Path(args.json).write_text(
            json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
