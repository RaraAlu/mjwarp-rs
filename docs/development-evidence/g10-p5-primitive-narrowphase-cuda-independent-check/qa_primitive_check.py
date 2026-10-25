"""QA 独立判据：设备 primitive 输出 vs 上游 f32 GPU 参考（`qa-upstream-gpu.json`）。

不读交付的 `bit_equal`/`window-output.txt` 汇总：输入用 QA 自己的 dumper 落盘的原始数组
（`target/qa-primitive/prod-out/<case>/`），参考直接取 QA 已提交的上游捕获 JSON。
判据沿用 G5 口径（本脚本自己实现，不 import 交付比较器）：

- 结构/计数/参数/整数：零容差（位级）；
- `dist`：位级优先，否则 ≤4 ULP（`ref == 0` 用 4 ULP@1.0）；
- `pos`：`scale = max_i |ref_i|`（行内），`4·ULP(scale)`；
- `frame`：`4 ULP@1.0`；
- 行序：按 `(worldid, dist, pos, frame)` 位级全序排序后按多重集比较（同时报告未排序位等）。

用法：
  python qa_primitive_check.py --numeric-only --case capsule-box
  python qa_primitive_check.py --json target/qa-primitive/qa-primitive-check.json
  python qa_primitive_check.py --self-test      # 反向判别力自检（不改产品/证据）
"""

import argparse
import hashlib
import json
import pathlib
import sys

import numpy as np

REPO = pathlib.Path(r"D:\Rust\mjwarp-rs")
G5_CASES = REPO / "docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases"
QA_CASES = REPO / "docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-gpu/cases"
REFERENCE = REPO / "docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-upstream-gpu.json"
DUMP = REPO / "target/qa-primitive/prod-out"
CASES = ["sphere-box-shallow", "sphere-box-deep", "capsule-box"]
INPUT_BINS = [
    "geom_xpos", "geom_xmat", "geom_size", "geom_margin", "geom_gap", "geom_rbound",
    "geom_aabb", "geom_type", "geom_dataid", "mesh_vertadr", "mesh_vertnum", "mesh_graphadr",
    "mesh_octadr", "mesh_vert", "mesh_graph", "hfield_size", "hfield_nrow", "hfield_ncol",
    "hfield_adr", "hfield_data", "oct_aabb", "oct_child", "oct_coeff",
]
F32_FIELDS = [
    ("dist", 1), ("pos", 3), ("frame", 9), ("includemargin", 1), ("friction", 5),
    ("solref", 2), ("solreffriction", 2), ("solimp", 5), ("adhesion", 1),
]
I32_FIELDS = [("dim", 1), ("geom", 2), ("worldid", 1), ("geomcollisionid", 1), ("type", 1)]
EFC_FIELD = "efc_address"


def load_meta(case_dir: pathlib.Path):
    meta = {}
    for line in (case_dir / "meta.txt").read_text(encoding="utf-8").splitlines():
        if " " in line:
            key, value = line.split(" ", 1)
            meta[key] = value
    return meta


def read_f32(directory: pathlib.Path, name: str):
    return np.fromfile(directory / f"{name}.bin", dtype="<f4")


def read_i32(directory: pathlib.Path, name: str):
    return np.fromfile(directory / f"{name}.bin", dtype="<i4")


def bits_of(values):
    return np.asarray(values, dtype=np.float32).view("<u4").astype(np.uint64)


def ordered_key(value: np.float32):
    """f32 的单调位序（与交付比较器 `ordered` 同定义，独立实现）。"""
    bits = int(np.array([value], dtype=np.float32).view("<u4")[0])
    if bits & 0x80000000:
        return -int(bits & 0x7FFFFFFF)
    return int(bits)


def ordered_keys(values):
    vals = np.asarray(values, dtype=np.float32)
    bits = vals.view("<u4").astype(np.int64)
    keys = np.where(bits < 0, -(bits & np.int64(0x7FFFFFFF)), bits)
    return keys


def ulp_distance(a, b):
    return int(abs(int(ordered_key(np.float32(a))) - int(ordered_key(np.float32(b)))))


def ulp_at(value):
    v = np.float32(value)
    nxt = np.nextafter(v, np.float32(np.inf))
    return float(nxt - v)


def row_key(worldid, dist, pos, frame, row):
    return (
        int(worldid[row]),
        ordered_key(dist[row]),
        *(ordered_key(pos[row * 3 + i]) for i in range(3)),
        *(ordered_key(frame[row * 9 + i]) for i in range(9)),
    )


def row_order(worldid, dist, pos, frame):
    return sorted(range(len(worldid)),
                  key=lambda row: row_key(worldid, dist, pos, frame, row))


def permute(values, order, width):
    out = []
    for row in order:
        out.extend(values[row * width:(row + 1) * width])
    return np.asarray(out)


class Report:
    def __init__(self, case):
        self.case = case
        self.failures = []
        self.bit_equal = 0
        self.total = 0
        self.unsorted_bit_equal = 0
        self.max_dist_ulp = 0
        self.max_pos_ulp = 0
        self.max_frame_ulp = 0
        self.field_bits = {}

    def add(self, field, device, reference, width, kind):
        device = np.asarray(device)
        reference = np.asarray(reference)
        if len(device) != len(reference):
            self.failures.append(f"{field}: 长度不符 dev={len(device)} ref={len(reference)}")
            return
        rows = len(reference) // width
        equal = 0
        for index, (dev, ref) in enumerate(zip(device, reference)):
            self.total += 1
            if int(np.float32(dev).view("<u4")) == int(np.float32(ref).view("<u4")):
                equal += 1
                self.bit_equal += 1
                continue
            dev32, ref32 = np.float32(dev), np.float32(ref)
            ulp = ulp_distance(dev32, ref32)
            row = index // width
            if kind == "dist":
                self.max_dist_ulp = max(self.max_dist_ulp, ulp)
                bound_ok = (abs(float(dev32) - float(ref32)) <= 4.0 * ulp_at(np.float32(1.0))
                            if float(ref32) == 0.0 else ulp <= 4)
            elif kind == "pos":
                self.max_pos_ulp = max(self.max_pos_ulp, ulp)
                row_ref = reference[row * 3:(row + 1) * 3]
                scale = float(np.max(np.abs(row_ref))) if len(row_ref) else 0.0
                if scale == 0.0:
                    scale = 1.0
                bound_ok = abs(float(dev32) - float(ref32)) <= 4.0 * ulp_at(scale)
            elif kind == "frame":
                self.max_frame_ulp = max(self.max_frame_ulp, ulp)
                bound_ok = abs(float(dev32) - float(ref32)) <= 4.0 * ulp_at(np.float32(1.0))
            else:
                bound_ok = False
            if not bound_ok:
                self.failures.append(
                    f"{field}[{index}] 超判据 dev={float(dev32)!r} ref={float(ref32)!r} ulp={ulp}")
        self.field_bits[field] = {"bit_equal": equal, "total": len(reference)}


def json_f32(case_ref, name):
    return np.asarray([np.float32(v) for v in case_ref["fields"][name]], dtype=np.float32)


def json_i32(case_ref, name):
    return np.asarray([int(v) for v in case_ref["fields"][name]], dtype=np.int32)


def check_inputs(case, case_ref):
    """独立输入守卫：23 项 bin 的 sha256/count 与 QA 参考 `input_bins` 逐项相等。"""
    result = {}
    for name in INPUT_BINS:
        expected = case_ref["input_bins"][name]
        qa_path = QA_CASES / case / f"{name}.bin"
        g5_path = G5_CASES / case / f"{name}.bin"
        raw = qa_path.read_bytes()
        sha = hashlib.sha256(raw).hexdigest()
        count = len(raw) // 4
        g5_sha = hashlib.sha256(g5_path.read_bytes()).hexdigest()
        result[name] = {
            "sha256": sha,
            "count": count,
            "matches_reference_sha": sha == expected["sha256"],
            "matches_reference_count": count == expected["count"],
            "matches_g5_tracked_sha": sha == g5_sha,
        }
    return result


def numeric_case(case, case_ref):
    case_dir = DUMP / case
    meta = load_meta(QA_CASES / case)
    nworld = int(meta["nworld"])
    efc_width = int(meta["efc_width"])
    report = Report(case)

    nacon = int(read_i32(case_dir, "nacon")[0])
    nccd = int(read_i32(case_dir, "nccd")[0])
    overflow = read_i32(case_dir, "overflow").astype(np.uint32)
    reference_overflow = np.asarray([int(v) for v in case_ref["overflow"]], dtype=np.uint32)
    reference_count = int(case_ref["count"])
    written = int(case_ref["written"])

    if nacon != reference_count:
        report.failures.append(f"nacon dev={nacon} ref={reference_count}")
    if nccd != 0:
        report.failures.append(f"nccd dev={nccd} 期望 0")
    if len(overflow) != len(reference_overflow) or not np.array_equal(overflow, reference_overflow):
        report.failures.append(f"overflow dev={overflow.tolist()} ref={reference_overflow.tolist()}")
    report.total += len(reference_overflow)
    report.bit_equal += int(np.sum(overflow == reference_overflow))
    report.field_bits["overflow"] = {
        "bit_equal": int(np.sum(overflow == reference_overflow)),
        "total": len(reference_overflow),
    }

    rows = nacon
    device = {
        "dist": read_f32(case_dir, "dist"),
        "pos": read_f32(case_dir, "pos"),
        "frame": read_f32(case_dir, "frame"),
        "includemargin": read_f32(case_dir, "includemargin"),
        "friction": read_f32(case_dir, "friction"),
        "solref": read_f32(case_dir, "solref"),
        "solreffriction": read_f32(case_dir, "solreffriction"),
        "solimp": read_f32(case_dir, "solimp"),
        "adhesion": read_f32(case_dir, "adhesion"),
        "dim": read_i32(case_dir, "dim"),
        "geom": read_i32(case_dir, "geom"),
        "worldid": read_i32(case_dir, "worldid"),
        "geomcollisionid": read_i32(case_dir, "geomcollisionid"),
        "type": read_i32(case_dir, "type"),
        "efc_address": read_i32(case_dir, "efc_address"),
        "overflow": overflow,
    }
    lengths = {}
    for name in ["dist", "pos", "frame", "includemargin", "friction", "solref", "solreffriction",
                 "solimp", "adhesion", "dim", "geom", "efc_address", "worldid",
                 "geomcollisionid", "type", "overflow"]:
        lengths[name] = int(len(device[name]))

    # 分母独立复算：nworld + nacon*(35+efc_width)；同时核对各数组实际长度容量。
    denominator = nworld + rows * (35 + efc_width)
    if rows != written:
        report.failures.append(f"dumped rows={rows} != reference written={written}")

    reference = {}
    for name, width in F32_FIELDS + I32_FIELDS:
        arr = json_f32(case_ref, name) if (name, width) in F32_FIELDS else json_i32(case_ref, name)
        reference[name] = arr
        expected_len = rows * width
        if len(arr) != expected_len:
            report.failures.append(f"参考 {name} 长度 {len(arr)} != rows*width {expected_len}")

    order_dev = row_order(device["worldid"], device["dist"], device["pos"], device["frame"])
    order_ref = row_order(reference["worldid"], reference["dist"], reference["pos"],
                          reference["frame"])

    for name, width in F32_FIELDS:
        width = int(width)
        dev_sorted = permute(device[name], order_dev, width)
        ref_sorted = permute(reference[name], order_ref, width)
        kind = {"dist": "dist", "pos": "pos", "frame": "frame"}.get(name, "exact")
        if kind == "exact":
            dev_bits = bits_of(dev_sorted)
            ref_bits = bits_of(ref_sorted)
            equal = int(np.sum(dev_bits == ref_bits))
            report.total += len(dev_bits)
            report.bit_equal += equal
            report.field_bits[name] = {"bit_equal": equal, "total": len(dev_bits)}
            for index, (a, b) in enumerate(zip(dev_bits, ref_bits)):
                if a != b:
                    report.failures.append(
                        f"{name}[{index}] 位级不等 dev_bits={int(a):#010x} ref_bits={int(b):#010x}")
        else:
            report.add(name, dev_sorted, ref_sorted, width, kind)

    for name, width in I32_FIELDS:
        width = int(width)
        dev_sorted = permute(device[name], order_dev, width)
        ref_sorted = permute(reference[name], order_ref, width)
        equal = int(np.sum(dev_sorted == ref_sorted))
        report.total += len(dev_sorted)
        report.bit_equal += equal
        report.field_bits[name] = {"bit_equal": equal, "total": len(dev_sorted)}
        for index, (a, b) in enumerate(zip(dev_sorted, ref_sorted)):
            if int(a) != int(b):
                report.failures.append(f"{name}[{index}] 不等 dev={int(a)} ref={int(b)}")

    # efc_address：上游 write_contact 只写 -1（参考未捕获）；核对产品初值。
    efc = device["efc_address"]
    equal = int(np.sum(efc == -1))
    report.total += len(efc)
    report.bit_equal += equal
    report.field_bits[EFC_FIELD] = {"bit_equal": equal, "total": len(efc)}
    for index, value in enumerate(efc):
        if int(value) != -1:
            report.failures.append(f"efc_address[{index}] 应为 -1 dev={int(value)}")

    # 未排序位等（行序稳定性，不计入判据）。
    for name, width in F32_FIELDS:
        width = int(width)
        if len(device[name]) != len(reference[name]):
            continue
        if name in ("dist", "pos", "frame"):
            report.unsorted_bit_equal += int(np.sum(
                bits_of(device[name]) == bits_of(reference[name])))
    for name, width in I32_FIELDS:
        if len(device[name]) != len(reference[name]):
            continue
        report.unsorted_bit_equal += int(np.sum(device[name] == reference[name]))

    return {
        "case": case,
        "nacon": nacon,
        "nccd": nccd,
        "overflow": overflow.tolist(),
        "reference_count": reference_count,
        "reference_written": written,
        "rows": rows,
        "denominator_recomputed": denominator,
        "denominator_breakdown": {
            "nworld": nworld,
            "nacon": rows,
            "per_row": 35,
            "efc_width": efc_width,
        },
        "array_lengths": lengths,
        "bit_equal": report.bit_equal,
        "total": report.total,
        "field_bits": report.field_bits,
        "max_dist_ulp": report.max_dist_ulp,
        "max_pos_ulp": report.max_pos_ulp,
        "max_frame_ulp": report.max_frame_ulp,
        "unsorted_bit_equal": report.unsorted_bit_equal,
        "row_order_device": [int(x) for x in order_dev],
        "row_order_reference": [int(x) for x in order_ref],
        "device_dist": [float(np.float32(v)) for v in device["dist"]],
        "reference_dist": [float(np.float32(v)) for v in reference["dist"]],
        "failures": report.failures,
    }


def self_test(cases):
    """反向判别力自检：对已落盘输出做受控变异，确认判据能报错。

    - dist 翻转 1 ULP（应有 bit 不等且 ulp=1，判据内通过 → 说明位级计数敏感）；
    - pos 加 1e-3（应超 4·ULP(scale) → 判据失败）；
    - frame 分量加 2e-7（≤4 ULP@1.0，应仍在判据内，但位级计数下降）；
    - 交换 pos 两个 world 的行（排序后应恢复 → 检验排序机制）。
    """
    ref = json.loads(REFERENCE.read_text(encoding="utf-8"))
    case = "sphere-box-shallow"
    case_ref = ref["cases"][case]
    baseline = numeric_case(case, case_ref)
    if baseline["failures"]:
        raise SystemExit(f"自检基线不干净：{baseline['failures']}")
    case_dir = DUMP / case
    dist = read_f32(case_dir, "dist").copy()
    dist_view = dist.view("<u4")
    dist_view[0] = np.uint32(int(dist_view[0]) + 1)
    report = Report(case)
    report.add("dist", dist, json_f32(case_ref, "dist"), 1, "dist")
    one_ulp_ok = (report.max_dist_ulp == 1 and report.bit_equal == len(dist) - 1
                  and not report.failures)
    pos = read_f32(case_dir, "pos").copy()
    pos[0] += np.float32(1e-3)
    report_pos = Report(case)
    report_pos.add("pos", pos, json_f32(case_ref, "pos"), 3, "pos")
    pos_detected = bool(report_pos.failures)
    frame = read_f32(case_dir, "frame").copy()
    frame[0] += np.float32(2e-7)
    report_frame = Report(case)
    report_frame.add("frame", frame, json_f32(case_ref, "frame"), 9, "frame")
    frame_within = not report_frame.failures and report_frame.bit_equal == len(frame) - 1
    # 交换两行 pos（排序应恢复）
    dev_worldid = read_i32(case_dir, "worldid")
    dev_dist = read_f32(case_dir, "dist")
    dev_pos = read_f32(case_dir, "pos")
    dev_frame = read_f32(case_dir, "frame")
    swapped = dev_pos.reshape(-1, 3)[[1, 0]].reshape(-1)
    order_dev = row_order(dev_worldid, dev_dist, dev_pos, dev_frame)
    unswapped = permute(swapped, order_dev, 3)
    expected = permute(dev_pos, order_dev, 3)
    sort_recovers = bool(np.array_equal(unswapped, expected))
    result = {
        "case": case,
        "baseline_bit_equal": f"{baseline['bit_equal']}/{baseline['total']}",
        "dist_1ulp_detected_as_1_ulp": one_ulp_ok,
        "pos_1e-3_detected_failure": pos_detected,
        "pos_failure_detail": report_pos.failures[:2],
        "frame_2e-7_within_bound_but_not_bit_equal": frame_within,
        "frame_detail": report_frame.field_bits,
        "row_sort_recovers_swapped_worlds": sort_recovers,
        "passed": bool(one_ulp_ok and pos_detected and frame_within and sort_recovers),
    }
    return result


def main():
    try:
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    except Exception:
        pass
    parser = argparse.ArgumentParser()
    parser.add_argument("--json", default=None)
    parser.add_argument("--numeric-only", action="store_true", help="只做数值判据（窗口内用）")
    parser.add_argument("--case", default=None, help="只核一个 case（配合 --numeric-only）")
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--dumps", default=None, help="QA dumper 输出目录")
    args = parser.parse_args()

    global DUMP
    if args.dumps:
        DUMP = pathlib.Path(args.dumps).resolve()

    ref = json.loads(REFERENCE.read_text(encoding="utf-8"))

    if args.self_test:
        result = self_test(ref["cases"])
        print(json.dumps(result, indent=2, ensure_ascii=False))
        if args.json:
            pathlib.Path(args.json).write_text(
                json.dumps(result, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        return 0 if result["passed"] else 1

    cases = [args.case] if args.case else CASES
    if args.numeric_only:
        data = {
            "reference": str(REFERENCE.relative_to(REPO)),
            "dumps": str(DUMP.relative_to(REPO)),
            "reference_meta": {
                "device": ref.get("device"),
                "provenance": ref.get("provenance"),
            },
            "cases": [],
        }
        failed = False
        for case in cases:
            case_ref = ref["cases"][case]
            inputs = check_inputs(case, case_ref)
            numeric = numeric_case(case, case_ref)
            data["cases"].append({"case": case, "inputs": inputs, "numeric": numeric})
            print(f"case={case} nacon={numeric['nacon']} nccd={numeric['nccd']} "
                  f"bit_equal={numeric['bit_equal']}/{numeric['total']} "
                  f"max_dist_ulp={numeric['max_dist_ulp']} "
                  f"max_pos_ulp={numeric['max_pos_ulp']} "
                  f"max_frame_ulp={numeric['max_frame_ulp']} "
                  f"failures={len(numeric['failures'])}")
            for failure in numeric["failures"]:
                print(f"FAIL {case}: {failure}")
            if numeric["failures"]:
                failed = True
        if args.json:
            pathlib.Path(args.json).write_text(
                json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        return 1 if failed else 0

    data = {
        "reference": str(REFERENCE.relative_to(REPO)),
        "dumps": str(DUMP.relative_to(REPO)),
        "reference_meta": {
            "device": ref.get("device"),
            "provenance": ref.get("provenance"),
            "two_captures_raw_bitwise_identical": {
                case: ref["cases"][case].get("two_captures_raw_bitwise_identical")
                for case in CASES
            },
        },
        "cases": [],
    }
    failed = False
    for case in cases:
        case_ref = ref["cases"][case]
        inputs = check_inputs(case, case_ref)
        numeric = numeric_case(case, case_ref)
        data["cases"].append({"case": case, "inputs": inputs, "numeric": numeric})
        print(f"case={case} nacon={numeric['nacon']} nccd={numeric['nccd']} "
              f"bit_equal={numeric['bit_equal']}/{numeric['total']} "
              f"unsorted_bit_equal={numeric['unsorted_bit_equal']} "
              f"max_dist_ulp={numeric['max_dist_ulp']} max_pos_ulp={numeric['max_pos_ulp']} "
              f"max_frame_ulp={numeric['max_frame_ulp']} failures={len(numeric['failures'])}")
        for failure in numeric["failures"]:
            print(f"FAIL {case}: {failure}")
        if numeric["failures"]:
            failed = True
    if args.json:
        pathlib.Path(args.json).write_text(
            json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
