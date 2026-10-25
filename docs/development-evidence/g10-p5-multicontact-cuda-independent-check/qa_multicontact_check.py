"""QA 独立判据：设备 multicontact / boxbox / 既有 contact 输出 vs 上游 f32 GPU 参考。
不读交付的 `bit_equal`/`window-output.txt` 汇总：输入用 QA 自己的 dumper 落盘的原始数组
（`target/qa-multicontact/prod-out/<case>/`），参考直接取 QA 已提交的
`qa-upstream-gpu.json`。判据沿用 G5 口径，但按本片语义做两处适配：
- 行序：按 `(worldid, geomcollisionid)` 排序后按多重集比较（multicontact 的 `geomcollisionid`
  即 `id_`，是同一 world 内的接触序号；`dist` 在 cylinder-box 上全相同，不能用作排序键）；
- `nccd`：参考未捕获（null），产品值只记录、不判失败（与 G5 同口径）。
其余：结构/计数/参数/整数零容差；`dist` 位级优先否则 ≤4 ULP（`ref==0` 用 4 ULP@1.0）；
`pos` 按行内 `scale=max_i|ref_i|` 的 `4·ULP(scale)`；`frame` 按 scale=1.0 的 4 ULP；
`efc_address` 逐项期望 -1。
用法：
  python qa_multicontact_check.py --numeric-only --case cylinder-box
  python qa_multicontact_check.py --json target/qa-multicontact/qa-multicontact-check.json
  python qa_multicontact_check.py --self-test   # 判据反向判别力自检（合成 dump，不跑 GPU）
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
DEFAULT_DUMPS = REPO / "target/qa-multicontact/prod-out"
CASES = ["cylinder-box", "box-box-margin", "mesh-mesh"]
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
KERNELS = {
    "cylinder-box": "contact_multi",
    "box-box-margin": "boxbox",
    "mesh-mesh": "contact",
}
def read_meta(case_dir):
    meta = {}
    for line in (case_dir / "meta.txt").read_text(encoding="utf-8").splitlines():
        if " " in line:
            key, value = line.split(" ", 1)
            meta[key] = value
    return meta
def read_f32(directory, name):
    return np.fromfile(directory / f"{name}.bin", dtype="<f4")
def read_i32(directory, name):
    return np.fromfile(directory / f"{name}.bin", dtype="<i4")
def bits_of(values):
    return np.asarray(values, dtype=np.float32).view("<u4").astype(np.uint64)
def ordered_key(value):
    bits = int(np.array([value], dtype=np.float32).view("<u4")[0])
    if bits & 0x80000000:
        return -int(bits & 0x7FFFFFFF)
    return int(bits)
def ulp_distance(a, b):
    return int(abs(ordered_key(np.float32(a)) - ordered_key(np.float32(b))))
def ulp_at(value):
    v = np.float32(value)
    return float(np.nextafter(v, np.float32(np.inf)) - v)
def row_order(worldid, gcid):
    """按 (worldid, geomcollisionid) 稳定排序；同键时保持原相对顺序（稳定）。"""
    return sorted(range(len(worldid)), key=lambda row: (int(worldid[row]), int(gcid[row]), row))
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
        self.max_dist_ulp = 0
        self.max_pos_ulp = 0
        self.max_frame_ulp = 0
        self.field_bits = {}
        self.tolerance_details = []
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
            abs_diff = abs(float(dev32) - float(ref32))
            detail = {
                "field": field,
                "index": int(index),
                "device": float(dev32),
                "reference": float(ref32),
                "ulp": ulp,
                "abs_diff": abs_diff,
                "bits_equal": False,
                "within_criterion": bool(bound_ok),
            }
            if kind == "dist":
                detail["bound"] = (4.0 * ulp_at(np.float32(1.0)) if float(ref32) == 0.0
                                   else "4 ULP")
            elif kind == "pos":
                row_ref = reference[row * 3:(row + 1) * 3]
                scale = float(np.max(np.abs(row_ref))) if len(row_ref) else 0.0
                scale = 1.0 if scale == 0.0 else scale
                detail["scale"] = scale
                detail["bound"] = 4.0 * ulp_at(scale)
            elif kind == "frame":
                detail["bound"] = 4.0 * ulp_at(np.float32(1.0))
            self.tolerance_details.append(detail)
            if not bound_ok:
                self.failures.append(
                    f"{field}[{index}] 超判据 dev={float(dev32)!r} ref={float(ref32)!r} ulp={ulp}")
        self.field_bits[field] = {"bit_equal": equal, "total": len(reference)}
def json_f32(case_ref, name):
    return np.asarray([np.float32(v) for v in case_ref["fields"][name]], dtype=np.float32)
def json_i32(case_ref, name):
    return np.asarray([int(v) for v in case_ref["fields"][name]], dtype=np.int32)
def check_inputs(case, case_ref):
    result = {}
    for name in INPUT_BINS:
        expected = case_ref["input_bins"][name]
        qa_path = QA_CASES / case / f"{name}.bin"
        g5_path = G5_CASES / case / f"{name}.bin"
        raw = qa_path.read_bytes()
        sha = hashlib.sha256(raw).hexdigest()
        g5_sha = hashlib.sha256(g5_path.read_bytes()).hexdigest()
        result[name] = {
            "sha256": sha,
            "count": len(raw) // 4,
            "matches_reference_sha": sha == expected["sha256"],
            "matches_reference_count": len(raw) // 4 == expected["count"],
            "matches_g5_tracked_sha": sha == g5_sha,
        }
    return result
def reference_arrays(case_ref):
    arrays = {}
    for name, width in F32_FIELDS:
        arrays[name] = json_f32(case_ref, name)
    for name, width in I32_FIELDS:
        arrays[name] = json_i32(case_ref, name)
    return arrays
def compare_arrays(case, arrays, case_ref):
    """arrays: dict(name -> np array)，含 nacon/nccd/overflow/efc_address。返回报告 dict。"""
    meta = read_meta(QA_CASES / case)
    nworld = int(meta["nworld"])
    efc_width = int(meta["efc_width"])
    report = Report(case)
    nacon = int(arrays["nacon"][0])
    nccd = int(arrays["nccd"][0])
    overflow = np.asarray(arrays["overflow"], dtype=np.uint32)
    reference_overflow = np.asarray([int(v) for v in case_ref["overflow"]], dtype=np.uint32)
    reference_count = int(case_ref["count"])
    written = int(case_ref["written"])
    if nacon != reference_count:
        report.failures.append(f"nacon dev={nacon} ref={reference_count}")
    if len(overflow) != len(reference_overflow) or not np.array_equal(overflow, reference_overflow):
        report.failures.append(f"overflow dev={overflow.tolist()} ref={reference_overflow.tolist()}")
    report.total += len(reference_overflow)
    report.bit_equal += int(np.sum(overflow == reference_overflow))
    report.field_bits["overflow"] = {
        "bit_equal": int(np.sum(overflow == reference_overflow)),
        "total": len(reference_overflow),
    }
    rows = nacon
    denominator = nworld + rows * (35 + efc_width)
    reference = reference_arrays(case_ref)
    device = {}
    for name, width in F32_FIELDS + I32_FIELDS:
        width = int(width)
        values = np.asarray(arrays[name])
        expected_len = rows * width
        if len(values) < expected_len:
            report.failures.append(f"{name} 长度不足 dev={len(values)} 需 {expected_len}")
            values = np.resize(values, expected_len)
        device[name] = values[:expected_len]
        if len(reference[name]) != expected_len:
            report.failures.append(
                f"参考 {name} 长度 {len(reference[name])} != rows*width {expected_len}")
    order_dev = row_order(device["worldid"], device["geomcollisionid"])
    order_ref = row_order(reference["worldid"], reference["geomcollisionid"])
    keys_dev = [(int(device["worldid"][r]), int(device["geomcollisionid"][r])) for r in range(rows)]
    duplicate_keys = len(set(keys_dev)) != len(keys_dev)
    for name, width in F32_FIELDS:
        width = int(width)
        dev_sorted = permute(device[name], order_dev, width)
        ref_sorted = permute(reference[name], order_ref, width)
        kind = {"dist": "dist", "pos": "pos", "frame": "frame"}.get(name, "exact")
        if kind == "exact":
            if len(dev_sorted) != len(ref_sorted):
                report.failures.append(
                    f"{name}: 长度不符 dev={len(dev_sorted)} ref={len(ref_sorted)}")
                continue
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
        if len(dev_sorted) != len(ref_sorted):
            report.failures.append(
                f"{name}: 长度不符 dev={len(dev_sorted)} ref={len(ref_sorted)}")
            continue
        equal = int(np.sum(dev_sorted == ref_sorted))
        report.total += len(dev_sorted)
        report.bit_equal += equal
        report.field_bits[name] = {"bit_equal": equal, "total": len(dev_sorted)}
        for index, (a, b) in enumerate(zip(dev_sorted, ref_sorted)):
            if int(a) != int(b):
                report.failures.append(f"{name}[{index}] 不等 dev={int(a)} ref={int(b)}")
    # 严格位级规则（Arch 2026-10-25 窗口条件 5）：除 box-box-margin 允许 1 个 pos 1 ULP
    # 之外，任何非位级项都视为阻断（其它 case 非位级、或 pos 出现 >1 个非位级、或 ulp != 1）。
    non_bit_fields = {name: data["total"] - data["bit_equal"]
                      for name, data in report.field_bits.items()}
    allowed = {"pos": 1} if case == "box-box-margin" else {}
    strict_failures = []
    for name, count in non_bit_fields.items():
        allowed_count = allowed.get(name, 0)
        if count > allowed_count:
            strict_failures.append(
                f"额外非位级 {name} {count} 项（允许 {allowed_count}）")
    if case == "box-box-margin":
        pos_details = [d for d in report.tolerance_details if d["field"] == "pos"]
        if len(pos_details) > 1 or (pos_details and pos_details[0]["ulp"] != 1):
            strict_failures.append(
                f"box-box-margin 非位级 pos 多于 1 项或 ulp!=1: "
                f"{[(d['index'], d['ulp']) for d in pos_details]}")
    elif report.tolerance_details:
        strict_failures.append(
            f"{case} 出现非位级项（严格规则不允许）："
            f"{[(d['field'], d['index'], d['ulp']) for d in report.tolerance_details]}")
    report.failures.extend(strict_failures)
    efc = np.asarray(arrays["efc_address"])[: rows * efc_width]
    equal = int(np.sum(efc == -1))
    report.total += len(efc)
    report.bit_equal += equal
    report.field_bits["efc_address"] = {"bit_equal": equal, "total": len(efc)}
    for index, value in enumerate(efc):
        if int(value) != -1:
            report.failures.append(f"efc_address[{index}] 应为 -1 dev={int(value)}")
    return {
        "case": case,
        "kernel": KERNELS.get(case, "?"),
        "nacon": nacon,
        "nccd_device": nccd,
        "nccd_reference": case_ref.get("nccd"),
        "overflow": overflow.tolist(),
        "reference_count": reference_count,
        "reference_written": written,
        "rows": rows,
        "denominator_recomputed": denominator,
        "duplicate_sort_keys": duplicate_keys,
        "bit_equal": report.bit_equal,
        "total": report.total,
        "field_bits": report.field_bits,
        "max_dist_ulp": report.max_dist_ulp,
        "max_pos_ulp": report.max_pos_ulp,
        "max_frame_ulp": report.max_frame_ulp,
        "tolerance_details": report.tolerance_details,
        "strict_rule": "仅 box-box-margin 允许 1 个 pos 1 ULP；其它非位级即阻断",
        "failures": report.failures,
    }
def numeric_case(case, case_ref, dumps):
    case_dir = pathlib.Path(dumps) / case
    arrays = {
        "nacon": read_i32(case_dir, "nacon"),
        "nccd": read_i32(case_dir, "nccd"),
        "overflow": read_i32(case_dir, "overflow"),
    }
    for name, width in F32_FIELDS:
        arrays[name] = read_f32(case_dir, name)
    for name, width in I32_FIELDS:
        arrays[name] = read_i32(case_dir, name)
    arrays["efc_address"] = read_i32(case_dir, "efc_address")
    return compare_arrays(case, arrays, case_ref)
def _synth_arrays(case_ref, meta):
    nacon = int(case_ref["count"])
    efc_width = int(meta["efc_width"])
    arrays = {}
    for name, width in F32_FIELDS:
        arrays[name] = np.asarray([np.float32(v) for v in case_ref["fields"][name]], dtype=np.float32)
    for name, width in I32_FIELDS:
        arrays[name] = np.asarray([int(v) for v in case_ref["fields"][name]], dtype=np.int32)
    arrays["nacon"] = np.asarray([nacon], dtype=np.int32)
    arrays["nccd"] = np.asarray([0], dtype=np.int32)
    arrays["overflow"] = np.asarray([int(v) for v in case_ref["overflow"]], dtype=np.int32)
    arrays["efc_address"] = np.full(nacon * efc_width, -1, dtype=np.int32)
    return arrays
def _clone(arrays):
    return {key: np.array(value, copy=True) for key, value in arrays.items()}
def self_test(ref):
    """判据反向判别力自检：全部用「参考合成 dump」，不跑 GPU。"""
    case = "box-box-margin"
    case_ref = ref["cases"][case]
    meta = read_meta(QA_CASES / case)
    base = _synth_arrays(case_ref, meta)
    baseline = compare_arrays(case, base, case_ref)
    def run(name, mutate, expected):
        arrays = _clone(base)
        mutate(arrays)
        report = compare_arrays(case, arrays, case_ref)
        failed = bool(report["failures"])
        bits = report["bit_equal"]
        total = report["total"]
        if expected == "pass":
            ok = not failed
        elif expected == "pass_but_not_all_bits":
            ok = not failed and bits < total
        elif expected == "fail":
            ok = failed
        else:
            ok = False
        return {
            "scenario": name,
            "expectation": expected,
            "bit_equal": f"{bits}/{total}",
            "max_dist_ulp": report["max_dist_ulp"],
            "max_pos_ulp": report["max_pos_ulp"],
            "max_frame_ulp": report["max_frame_ulp"],
            "failures": len(report["failures"]),
            "first_failure": report["failures"][0] if report["failures"] else None,
            "as_expected": ok,
        }
    def plus_ulp(values, index, ulps=1):
        values[index] = np.frombuffer(
            (np.uint32(values.view("<u4")[index]) + np.uint32(ulps)).tobytes(), dtype="<f4")[0]
    def single_contact(arrays):
        keep = np.asarray([i for i, g in enumerate(arrays["geomcollisionid"]) if int(g) == 0])
        arrays["nacon"] = np.asarray([len(keep)], dtype=np.int32)
        for name, width in F32_FIELDS + I32_FIELDS:
            width = int(width)
            rows = np.asarray(arrays[name]).reshape(-1, width)
            arrays[name] = rows[keep].reshape(-1)
        efc_width = int(meta["efc_width"])
        efc = np.asarray(arrays["efc_address"]).reshape(-1, efc_width)
        arrays["efc_address"] = efc[keep].reshape(-1)
    def drop_one(arrays):
        arrays["nacon"] = np.asarray([int(arrays["nacon"][0]) - 1], dtype=np.int32)
    def swap_rows(arrays):
        order = row_order(arrays["worldid"], arrays["geomcollisionid"])
        for name, width in F32_FIELDS + I32_FIELDS:
            width = int(width)
            rows = np.asarray(arrays[name]).reshape(-1, width)
            perm = [1, 0] + list(range(2, len(rows)))
            arrays[name] = rows[perm].reshape(-1)
    def permute_rows(arrays):
        for name, width in F32_FIELDS + I32_FIELDS:
            width = int(width)
            rows = np.asarray(arrays[name]).reshape(-1, width)
            perm = list(range(len(rows)))
            perm[0], perm[1] = perm[1], perm[0]
            arrays[name] = rows[perm].reshape(-1)
    scenarios = [
        run("baseline_reference_dump", lambda a: None, "pass"),
        run("pos_plus_1ulp", lambda a: plus_ulp(a["pos"], 0, 1), "pass_but_not_all_bits"),
        run("dist_plus_1ulp_blocks_under_strict", lambda a: plus_ulp(a["dist"], 0, 1), "fail"),
        run("pos_plus_1e-3", lambda a: a["pos"].__setitem__(0, a["pos"][0] + np.float32(1e-3)), "fail"),
        run("frame_plus_1e-6", lambda a: a["frame"].__setitem__(0, a["frame"][0] + np.float32(1e-6)), "fail"),
        run("single_contact_fallback", single_contact, "fail"),
        run("drop_one_contact", drop_one, "fail"),
        run("swap_two_rows", swap_rows, "pass"),
        run("permute_rows", permute_rows, "pass"),
        run("nccd_999_record_only", lambda a: a["nccd"].__setitem__(0, 999), "pass"),
        run("overflow_bit_set", lambda a: a["overflow"].__setitem__(0, 1), "fail"),
        run("efc_address_zero", lambda a: a["efc_address"].__setitem__(0, 0), "fail"),
    ]
    return {
        "case": case,
        "nature": "合成 dump 自检（由上游参考构造，不跑 GPU；不是真实设备数据）",
        "baseline_bit_equal": f"{baseline['bit_equal']}/{baseline['total']}",
        "baseline_denominator": baseline["denominator_recomputed"],
        "scenarios": scenarios,
        "all_as_expected": all(item["as_expected"] for item in scenarios),
    }
def main():
    try:
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    except Exception:
        pass
    parser = argparse.ArgumentParser()
    parser.add_argument("--json", default=None)
    parser.add_argument("--numeric-only", action="store_true")
    parser.add_argument("--case", default=None)
    parser.add_argument("--dumps", default=str(DEFAULT_DUMPS))
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    ref = json.loads(REFERENCE.read_text(encoding="utf-8"))
    if args.self_test:
        result = self_test(ref)
        text = json.dumps(result, indent=2, ensure_ascii=False)
        print(text)
        if args.json:
            pathlib.Path(args.json).write_text(text + "\n", encoding="utf-8")
        return 0 if result["all_as_expected"] else 1
    cases = [args.case] if args.case else CASES
    data = {
        "reference": str(REFERENCE.relative_to(REPO)),
        "dumps": str(args.dumps),
        "reference_meta": {
            "device": ref.get("device"),
            "provenance": ref.get("provenance"),
            "two_captures_raw_bitwise_identical": {
                case: ref["cases"][case].get("two_captures_raw_bitwise_identical")
                for case in CASES
            },
        },
        "sort_key": "(worldid, geomcollisionid)",
        "cases": [],
    }
    failed = False
    for case in cases:
        case_ref = ref["cases"][case]
        inputs = check_inputs(case, case_ref)
        numeric = numeric_case(case, case_ref, args.dumps)
        data["cases"].append({"case": case, "inputs": inputs, "numeric": numeric})
        print(f"case={case} kernel={numeric['kernel']} nacon={numeric['nacon']} "
              f"nccd={numeric['nccd_device']}(ref={numeric['nccd_reference']}) "
              f"bit_equal={numeric['bit_equal']}/{numeric['total']} "
              f"denominator={numeric['denominator_recomputed']} "
              f"max_dist_ulp={numeric['max_dist_ulp']} max_pos_ulp={numeric['max_pos_ulp']} "
              f"max_frame_ulp={numeric['max_frame_ulp']} failures={len(numeric['failures'])}")
        for failure in numeric["failures"]:
            print(f"FAIL {case}: {failure}")
        if numeric["failures"]:
            failed = True
    if args.json:
        pathlib.Path(args.json).write_text(
            json.dumps(data, ensure_ascii=False, separators=(",", ":")) + "\n",
            encoding="utf-8")
    return 1 if failed else 0
if __name__ == "__main__":
    sys.exit(main())
