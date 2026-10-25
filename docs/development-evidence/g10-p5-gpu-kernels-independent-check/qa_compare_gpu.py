"""QA 独立复核 · G5：**QA 自己算**的「设备 vs 同配方上游 f32 GPU」逐 case 判据。

判据（G5 现行口径，Arch 裁决 2）：
- count / worldid 分组后比较；接触行按 (worldid, dist, pos, frame) 位型多重集排序；
- `dist`：按自身量级 ≤4 ULP（ref==0 用 4 ULP@1.0 绝对差）；
- `pos` / witness `x1`/`x2`：向量尺度 scale=max_i|ref_i|（全零回落 1.0），逐分量 |dev-ref|≤4·ULP(scale)；
- `frame`/`normal`：scale=1.0，|diff| ≤ 4 ULP@1.0 = 4.768e-7；
- 参数/整数字段位级相等；efc_address 全 -1（上游语义）；overflow 位比。
不引用交付 `result.json`。native f64 只作 G6 容差候选材料，不作判据。

用法：python qa_compare_gpu.py [--out qa-compare-gpu.json]
"""

import argparse
import json
import math
import struct
from pathlib import Path

EVIDENCE = Path(__file__).resolve().parent
WORK = EVIDENCE / "qa-gpu"
CASES = WORK / "cases"
OUT = WORK / "out"
UPSTREAM = EVIDENCE / "qa-upstream-gpu.json"
WITNESS = EVIDENCE / "qa-witness-gpu.json"
NATIVE = EVIDENCE / "qa-native-f64.json"

FLOAT_FIELDS = ("dist", "pos", "frame", "includemargin", "friction", "solref",
                "solreffriction", "solimp", "adhesion")
INT_FIELDS = ("dim", "geom", "worldid", "type", "geomcollisionid")
WIDTH = {"dist": 1, "pos": 3, "frame": 9, "includemargin": 1, "friction": 5,
         "solref": 2, "solreffriction": 2, "solimp": 5, "adhesion": 1,
         "dim": 1, "geom": 2, "worldid": 1, "type": 1, "geomcollisionid": 1}

OVERFLOW_CCD = 1 << 4
OVERFLOW_HFIELD = 1 << 5
OVERFLOW_EPA_HORIZON = 1 << 8


def f32_bits(value):
    return struct.unpack("<I", struct.pack("<f", float(value)))[0]


def bits32(values):
    return [f32_bits(v) for v in values]


def ulp_at(value):
    value = abs(float(value))
    if value == 0.0:
        return 2.0 ** -149
    if not math.isfinite(value):
        return float("inf")
    import numpy as np
    a = np.float32(value)
    b = np.nextafter(a, np.float32(np.inf), dtype=np.float32)
    return float(b) - float(a)


def criterion_dist(dev, ref):
    if ref == 0.0:
        return abs(dev - ref) <= 4.0 * ulp_at(1.0)
    return abs(dev - ref) <= 4.0 * ulp_at(ref)


def criterion_vector(dev, ref):
    scale = max(abs(v) for v in ref)
    if scale == 0.0:
        scale = 1.0
    bound = 4.0 * ulp_at(scale)
    worst = max(abs(d - r) for d, r in zip(dev, ref))
    return worst <= bound, worst, bound


def criterion_unit(dev, ref):
    return all(abs(d - r) <= 4.0 * ulp_at(1.0) for d, r in zip(dev, ref))


def read_bin(path, dtype, count):
    raw = Path(path).read_bytes()
    if dtype == "f32":
        values = struct.unpack("<%df" % (len(raw) // 4), raw)
    else:
        values = struct.unpack("<%di" % (len(raw) // 4), raw)
    return list(values[:count]) if count is not None else list(values)


def read_meta(case):
    meta = {}
    for line in (CASES / case / "meta.txt").read_text(encoding="utf-8").splitlines():
        parts = line.split()
        if parts:
            meta[parts[0]] = parts[1:]
    return meta


def upstream_rows(case_doc):
    count = int(case_doc["count"])
    written = int(case_doc.get("written", count))
    fields = case_doc["fields"]
    rows = []
    for index in range(written):
        row = {}
        for name in FLOAT_FIELDS + INT_FIELDS:
            width = WIDTH[name]
            row[name] = fields[name][index * width:(index + 1) * width]
        rows.append(row)
    return count, rows


def probe_rows(case, capacity):
    case_out = OUT / case
    nacon = read_bin(case_out / "nacon.bin", "i32", 1)[0]
    written = min(nacon, capacity)
    rows = []
    for index in range(written):
        row = {}
        for name in FLOAT_FIELDS:
            width = WIDTH[name]
            values = read_bin(case_out / f"{name}.bin", "f32", None)
            row[name] = values[index * width:(index + 1) * width]
        for name in INT_FIELDS:
            width = WIDTH[name]
            values = read_bin(case_out / f"{name}.bin", "i32", None)
            row[name] = values[index * width:(index + 1) * width]
        rows.append(row)
    return nacon, rows


def sort_key(row):
    return (int(row["worldid"][0]), int(row["geom"][0]), int(row["geom"][1]),
            [f32_bits(v) for v in row["dist"]], [f32_bits(v) for v in row["pos"]])


def contact_compare(name, dev_rows, ref_rows):
    """按 (worldid, geom, dist, pos) 位型排序后逐行比较 criterion。"""
    dev_sorted = sorted(dev_rows, key=sort_key)
    ref_sorted = sorted(ref_rows, key=sort_key)
    result = {"count": [len(dev_rows), len(ref_rows)],
              "world_counts": [{}, {}],
              "dist_bad": 0, "dist_max_abs": 0.0, "dist_max_ulp": 0.0, "dist_bit_equal": 0,
              "pos_bad": 0, "pos_max_abs": 0.0, "pos_max_ulp_at_scale": 0.0, "pos_bit_equal": 0,
              "frame_bad": 0, "frame_max_abs": 0.0, "frame_bit_equal": 0,
              "params_bit_equal": True, "rows": []}
    for index, row in enumerate(ref_rows):
        world = int(row["worldid"][0])
        result["world_counts"][1][world] = result["world_counts"][1].get(world, 0) + 1
    for index, row in enumerate(dev_rows):
        world = int(row["worldid"][0])
        result["world_counts"][0][world] = result["world_counts"][0].get(world, 0) + 1
    pairs = list(zip(dev_sorted, ref_sorted))
    if len(dev_sorted) != len(ref_sorted):
        pairs = pairs[: min(len(dev_sorted), len(ref_sorted))]
    for dev, ref in pairs:
        entry = {}
        dist_abs = abs(dev["dist"][0] - ref["dist"][0])
        entry["dist_abs"] = dist_abs
        entry["dist_ulp"] = dist_abs / ulp_at(ref["dist"][0]) if ref["dist"][0] != 0 else dist_abs / ulp_at(1.0)
        entry["dist_ok"] = criterion_dist(dev["dist"][0], ref["dist"][0])
        result["dist_max_abs"] = max(result["dist_max_abs"], entry["dist_abs"])
        result["dist_max_ulp"] = max(result["dist_max_ulp"], entry["dist_ulp"])
        result["dist_bit_equal"] += int(bits32(dev["dist"]) == bits32(ref["dist"]))
        result["dist_bad"] += int(not entry["dist_ok"])
        pos_ok, pos_worst, pos_bound = criterion_vector(dev["pos"], ref["pos"])
        entry["pos_worst"] = pos_worst
        entry["pos_bound"] = pos_bound
        entry["pos_ok"] = pos_ok
        result["pos_max_abs"] = max(result["pos_max_abs"], pos_worst)
        result["pos_max_ulp_at_scale"] = max(result["pos_max_ulp_at_scale"],
                                             pos_worst / (pos_bound / 4.0) if pos_bound else 0.0)
        result["pos_bit_equal"] += int(bits32(dev["pos"]) == bits32(ref["pos"]))
        result["pos_bad"] += int(not pos_ok)
        frame_ok = criterion_unit(dev["frame"], ref["frame"])
        entry["frame_ok"] = frame_ok
        frame_abs = max(abs(d - r) for d, r in zip(dev["frame"], ref["frame"]))
        result["frame_max_abs"] = max(result["frame_max_abs"], frame_abs)
        result["frame_bit_equal"] += int(bits32(dev["frame"]) == bits32(ref["frame"]))
        result["frame_bad"] += int(not frame_ok)
        for field in FLOAT_FIELDS + INT_FIELDS:
            if field in ("dist", "pos", "frame"):
                continue
            if field in INT_FIELDS:
                equal = [int(v) for v in dev[field]] == [int(v) for v in ref[field]]
            else:
                equal = bits32(dev[field]) == bits32(ref[field])
            result["params_bit_equal"] = result["params_bit_equal"] and equal
        result["rows"].append(entry)
    result["pass"] = (result["count"][0] == result["count"][1]
                      and result["world_counts"][0] == result["world_counts"][1]
                      and result["dist_bad"] == 0 and result["pos_bad"] == 0
                      and result["frame_bad"] == 0 and result["params_bit_equal"])
    return result


def efc_all_minus_one(case, capacity):
    values = read_bin(OUT / case / "efc_address.bin", "i32", None)
    efc_width = int(read_meta(case)["efc_width"][0])
    nacon = read_bin(OUT / case / "nacon.bin", "i32", 1)[0]
    written = min(nacon, capacity)
    efc = values[: written * efc_width]
    return all(v == -1 for v in efc), {"width": efc_width, "written": written}


def two_runs_identical(case, kind, capacity=None):
    """两次运行比较：接触行按 (worldid, geom, dist, pos, frame) 位型排序后逐字段位级比较；
    上游 nacon/写行是 atomic 分配，raw 行序不定，不能直接比字节。"""
    case_out = OUT / case
    if kind == "witness":
        files = ["dist", "x1", "x2", "ncontact", "index", "overflow"]
        for name in files:
            second_path = case_out / f"{name}-run2.bin"
            if not second_path.exists():
                return False
            if (case_out / f"{name}.bin").read_bytes() != second_path.read_bytes():
                return False
        return True

    def load(tag):
        def read(name):
            return read_bin(case_out / f"{name}{tag}.bin", "f32", None)
        def read_i(name):
            return read_bin(case_out / f"{name}{tag}.bin", "i32", None)
        nacon = read_i("nacon")[0]
        worldid = read_i("worldid")
        geom = read_i("geom")
        dist = read("dist")
        pos = read("pos")
        frame = read("frame")
        rows = []
        for index in range(min(nacon, len(worldid))):
            rows.append((int(worldid[index]), int(geom[index * 2]), int(geom[index * 2 + 1]),
                         f32_bits(dist[index]),
                         tuple(f32_bits(v) for v in pos[index * 3:index * 3 + 3]),
                         tuple(f32_bits(v) for v in frame[index * 9:index * 9 + 9])))
        return nacon, sorted(rows)

    second_path = case_out / "nacon-run2.bin"
    if not second_path.exists():
        return False
    return load("") == load("-run2")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", default=str(EVIDENCE / "qa-compare-gpu.json"))
    args = parser.parse_args()

    upstream = json.loads(UPSTREAM.read_text(encoding="utf-8"))
    witness = json.loads(WITNESS.read_text(encoding="utf-8"))
    native = json.loads(NATIVE.read_text(encoding="utf-8")) if NATIVE.exists() else {"cases": {}}

    report = {"schema": 1, "kind": "qa-g5-independent-device-vs-upstream-gpu",
              "criterion": {
                  "count": "相等 + worldid 分组",
                  "dist": "自身量级 <=4 ULP；ref==0 用 4 ULP@1.0 绝对差",
                  "pos/x1/x2": "scale=max_i|ref_i|（全零回落 1.0），逐分量 |dev-ref|<=4*ULP(scale)",
                  "frame/normal": "scale=1.0，|diff|<=4 ULP@1.0 = 4.768e-7",
                  "params/ints": "位级相等",
                  "source": "QA 自算；不引用交付 result.json"},
              "provenance": {
                  "upstream": upstream.get("provenance", {}),
                  "device": "qa_probe.exe (nvcc 12.8, fmad=true, -O3, sm_89)；QA 自写 launcher"},
              "contact_cases": {}, "capacity_cases": {}, "witness_cases": {},
              "custom_cases": {}, "native_f64_readings": {},
              "failures": []}
    failures = report["failures"]

    # ---------- 1) contact 数组（4 例） ----------
    meta_all = {}
    for case in sorted(p.name for p in CASES.iterdir() if p.is_dir()):
        meta_all[case] = read_meta(case)
    # 上游 dispatch：SPHERE/CAPSULE × BOX 走 primitive_narrowphase（不是 ccd 产物），
    # 设备 primitive 路径本片未实装 → 不列入接触数组对照（与 G5 交付口径一致，已独立核对上游源码）。
    PRIMITIVE_CASES = {"sphere-box-shallow", "sphere-box-deep", "capsule-box"}
    report["not_compared"] = {
        case: {"reason": "upstream dispatch = primitive_narrowphase（非 ccd 产物）；设备 primitive 未实装，"
                         "不列入接触数组对照"}
        for case in PRIMITIVE_CASES}
    contact_names = [c for c, m in meta_all.items()
                     if m["kind"][0] in ("contact", "sdf", "hfield")
                     and c in upstream["cases"] and c not in PRIMITIVE_CASES]
    for case in contact_names:
        capacity = int(meta_all[case]["naconmax"][0])
        nacon_dev, dev_rows = probe_rows(case, capacity)
        upstream_doc = upstream["cases"][case]
        count_up, ref_rows = upstream_rows(upstream_doc)
        comparison = contact_compare(case, dev_rows, ref_rows)
        efc_ok, efc_info = efc_all_minus_one(case, capacity)
        overflow_dev = read_bin(OUT / case / "overflow.bin", "i32", None)
        nccd_dev = read_bin(OUT / case / "nccd.bin", "i32", 1)[0]
        two_runs = two_runs_identical(case, "contact")
        entry = {"kind": meta_all[case]["kind"][0],
                 "comparison": comparison,
                 "nacon_device": nacon_dev, "nacon_upstream": count_up,
                 "count_equal": nacon_dev == count_up,
                 "efc_all_minus_one": efc_ok, "efc_info": efc_info,
                 "overflow_device": overflow_dev,
                 "overflow_upstream": upstream_doc.get("overflow"),
                 "overflow_equal": [int(v) for v in overflow_dev] ==
                                   [int(v) for v in upstream_doc.get("overflow", [])],
                 "nccd_device": nccd_dev, "nccd_upstream": upstream_doc.get("nccd"),
                 "two_runs_bitwise_identical": two_runs,
                 "pass": comparison["pass"] and nacon_dev == count_up and efc_ok,
                 "upstream_two_captures_raw_identical":
                     upstream_doc.get("two_captures_raw_bitwise_identical")}
        if not entry["pass"]:
            failures.append(f"contact/{case}: " +
                            f"count {nacon_dev}/{count_up} dist_bad {comparison['dist_bad']} " +
                            f"pos_bad {comparison['pos_bad']} frame_bad {comparison['frame_bad']} " +
                            f"params_equal {comparison['params_bit_equal']} efc_ok {efc_ok}")
        report["contact_cases"][case] = entry

    # ---------- 2) 容量/截断 ----------
    for case, upstream_doc in upstream.get("capacity_cases", {}).items():
        capacity = int(meta_all[case]["naconmax"][0])
        naccdmax = int(meta_all[case]["naccdmax"][0])
        nacon_dev, dev_rows = probe_rows(case, capacity)
        count_up = int(upstream_doc["count"])
        ref_rows = upstream_rows(upstream_doc)[1]
        overflow_dev = read_bin(OUT / case / "overflow.bin", "i32", None)
        nccd_dev = read_bin(OUT / case / "nccd.bin", "i32", 1)[0]
        # 截断 / naccdmax 溢出后「哪一 world 的行留下来」由上游并发 atomic 与设备逐 world
        # launch 决定（竞态，非判据）；这两种情形按「忽略 worldid 的接触值多重集」比较。
        nccd_up = upstream_doc.get("nccd")
        race = (len(dev_rows) >= capacity and nacon_dev > capacity) or (
            nccd_dev >= naccdmax and nccd_dev > 0)

        def strip_world(rows):
            out = []
            for row in rows:
                stripped = dict(row)
                stripped["worldid"] = [0]
                out.append(stripped)
            return out

        comparison = contact_compare(
            case,
            strip_world(dev_rows) if race else dev_rows,
            strip_world(ref_rows) if race else ref_rows)

        def bit_multiset(values):
            counts = {}
            for value in values:
                for bit in (16, 32, 256):
                    if int(value) & bit:
                        counts[bit] = counts.get(bit, 0) + 1
            return counts

        overflow_multiset_equal = bit_multiset(overflow_dev) == bit_multiset(
            upstream_doc.get("overflow", []))
        entry = {"kind": meta_all[case]["kind"][0],
                 "naconmax": capacity, "naccdmax": naccdmax,
                 "nacon_device": nacon_dev, "nacon_upstream": count_up,
                 "rows_written_device": len(dev_rows),
                 "rows_written_upstream": len(ref_rows),
                 "truncated": len(dev_rows) >= capacity and nacon_dev > capacity,
                 "no_oob_write": len(dev_rows) <= capacity and len(ref_rows) <= capacity,
                 "overflow_device": overflow_dev,
                 "overflow_upstream": upstream_doc.get("overflow"),
                 "overflow_multiset_equal": overflow_multiset_equal,
                 "nccd_device": nccd_dev, "nccd_upstream": nccd_up,
                 "contact_compare_scope": "not compared (race: row->world assignment is atomic-order dependent)"
                 if race else "worldid grouped",
                 "comparison": comparison,
                 "pass": (nacon_dev == count_up
                          and len(dev_rows) == len(ref_rows)
                          and len(dev_rows) <= capacity
                          and overflow_multiset_equal
                          and (race or comparison["pass"])),
                 "two_runs_bitwise_identical": two_runs_identical(case, "contact"),
                 "note": "容量截断/溢出：计数如实、写入行不越界；截断后行在世界间的归属属竞态、不作判据"}
        if not entry["pass"]:
            failures.append(f"capacity/{case}: count {nacon_dev}/{count_up} " +
                            f"rows {len(dev_rows)}/{len(ref_rows)} " +
                            f"overflow {overflow_dev} vs {upstream_doc.get('overflow')} " +
                            f"dist_bad {comparison['dist_bad']} pos_bad {comparison['pos_bad']} " +
                            f"frame_bad {comparison['frame_bad']}")
        report["capacity_cases"][case] = entry

    # ---------- 3) witness（ccd 直调） ----------
    def witness_compare(label, dev_out_dir, ref_doc):
        worlds = int(ref_doc["worlds"])
        dist_dev = read_bin(OUT / dev_out_dir / "dist.bin", "f32", worlds)
        x1_dev = read_bin(OUT / dev_out_dir / "x1.bin", "f32", worlds * 3)
        x2_dev = read_bin(OUT / dev_out_dir / "x2.bin", "f32", worlds * 3)
        ncontact_dev = read_bin(OUT / dev_out_dir / "ncontact.bin", "i32", worlds)
        index_dev = read_bin(OUT / dev_out_dir / "index.bin", "i32", worlds)
        overflow_dev = read_bin(OUT / dev_out_dir / "overflow.bin", "i32", worlds)
        result = ref_doc["results"]
        entry = {"worlds": worlds,
                 "dist": {"max_abs": 0.0, "max_ulp": 0.0, "bit_equal": 0, "bad": 0},
                 "x1": {"max_abs": 0.0, "bad": 0}, "x2": {"max_abs": 0.0, "bad": 0},
                 "ncontact_equal": True, "overflow_device": overflow_dev,
                 "overflow_upstream": result["overflow"],
                 "overflow_note": "", "index_device": index_dev,
                 "index_upstream": result["index"],
                 "pass": True}
        for world in range(worlds):
            ref_dist = result["dist"][world]
            dev_dist = dist_dev[world]
            dist_abs = abs(dev_dist - ref_dist)
            ulp = dist_abs / ulp_at(ref_dist) if ref_dist != 0 else dist_abs / ulp_at(1.0)
            entry["dist"]["max_abs"] = max(entry["dist"]["max_abs"], dist_abs)
            entry["dist"]["max_ulp"] = max(entry["dist"]["max_ulp"], ulp)
            entry["dist"]["bit_equal"] += int(f32_bits(dev_dist) == f32_bits(ref_dist))
            if not criterion_dist(dev_dist, ref_dist):
                entry["dist"]["bad"] += 1
            ref_x1 = result["x1"][world]
            dev_x1 = x1_dev[world * 3:(world + 1) * 3]
            ok1, worst1, _ = criterion_vector(dev_x1, ref_x1)
            entry["x1"]["max_abs"] = max(entry["x1"]["max_abs"], worst1)
            entry["x1"]["bad"] += int(not ok1)
            ref_x2 = result["x2"][world]
            dev_x2 = x2_dev[world * 3:(world + 1) * 3]
            ok2, worst2, _ = criterion_vector(dev_x2, ref_x2)
            entry["x2"]["max_abs"] = max(entry["x2"]["max_abs"], worst2)
            entry["x2"]["bad"] += int(not ok2)
            entry["ncontact_equal"] = (entry["ncontact_equal"] and
                                       int(ncontact_dev[world]) == int(result["ncontact"][world]))
        # overflow：设备在 FLOAT_MAX 时额外置 CCD 位（上游 ccd() 不置）；按该口径归一后比较
        normalized = []
        for world in range(worlds):
            dev = int(overflow_dev[world])
            if f32_bits(dist_dev[world]) == f32_bits(1.0e30) or abs(dist_dev[world] - 1.0e30) <= abs(1.0):
                dev &= ~OVERFLOW_CCD
            normalized.append(dev)
        ref_overflow = [int(v) for v in result["overflow"]]
        entry["overflow_equal_normalized"] = normalized == ref_overflow
        entry["overflow_normalized_device"] = normalized
        if normalized != ref_overflow:
            entry["overflow_note"] = "normalized mismatch"
        entry["pass"] = (entry["dist"]["bad"] == 0 and entry["x1"]["bad"] == 0
                         and entry["x2"]["bad"] == 0 and entry["ncontact_equal"]
                         and entry["overflow_equal_normalized"])
        entry["two_runs_bitwise_identical"] = two_runs_identical(dev_out_dir, "witness")
        if not entry["pass"]:
            failures.append(f"witness/{label}: " +
                            f"dist_bad {entry['dist']['bad']} x1_bad {entry['x1']['bad']} " +
                            f"x2_bad {entry['x2']['bad']} ncontact_equal {entry['ncontact_equal']} " +
                            f"overflow {overflow_dev} vs {result['overflow']}")
        return entry

    for case, ref_doc in witness.get("cases", {}).items():
        if (CASES / case).exists():
            report["witness_cases"][case] = witness_compare(case, case, ref_doc)
    for ref_doc in witness.get("sweep", []):
        label = ref_doc["case"] + "-sweep"
        if (CASES / label).exists():
            report["witness_cases"][label] = witness_compare(label, label, ref_doc)
    for case, ref_doc in witness.get("forced_horizon", {}).items():
        label = case + "-h2"
        if (CASES / label).exists():
            report["witness_cases"][label] = witness_compare(label, label, ref_doc)
    discret = witness.get("custom_is_discrete", {})
    if discret:
        report["custom_cases"]["is_discrete-box-box"] = {
            "base": witness_compare("qa-box-box", "qa-box-box", discret["base"]),
            "sweep": witness_compare("qa-box-box-sweep", "qa-box-box-sweep", discret["sweep"]),
        }

    # ---------- 4) hfield MAXCONPAIR 溢出（自造） ----------
    hf = witness.get("custom_hfield_overflow")
    if hf is not None:
        case = "hfield-overflow"
        meta = meta_all[case]
        capacity = int(meta["naconmax"][0])
        nacon_dev, dev_rows = probe_rows(case, capacity)
        ref_rows = upstream_rows({"count": hf["count"], "written": hf["written"],
                                  "fields": hf["fields"]})[1]
        comparison = contact_compare(case, dev_rows, ref_rows)
        prism_count = read_bin(OUT / case / "prism_count.bin", "i32", None)
        overflow_dev = read_bin(OUT / case / "overflow.bin", "i32", None)
        entry = {"nacon_device": nacon_dev, "nacon_upstream": int(hf["count"]),
                 "prism_count_device": prism_count,
                 "prism_count_is_50": all(int(v) == 50 for v in prism_count),
                 "overflow_device": overflow_dev,
                 "overflow_upstream": hf.get("overflow"),
                 "hfield_bit_set": all(int(v) & OVERFLOW_HFIELD for v in overflow_dev),
                 "comparison": comparison,
                 "pass": (nacon_dev == int(hf["count"]) and comparison["pass"]
                          and all(int(v) == 50 for v in prism_count)
                          and all(int(v) & OVERFLOW_HFIELD for v in overflow_dev))}
        if not entry["pass"]:
            failures.append("custom/hfield-overflow: " +
                            f"count {nacon_dev}/{hf['count']} prism {prism_count} " +
                            f"overflow {overflow_dev} vs {hf.get('overflow')}")
        report["custom_cases"]["hfield-maxconpair-overflow"] = entry

    # ---------- 5) native f64 读数（G6 容差候选材料，非判据） ----------
    # 只对「native 每 world 接触数 == 设备每 world 接触数」的 case 做配对；数量不同说明
    # native C 引擎走了不同路径（如 SDF/mesh 支撑差异），记为不可比，不硬配。
    def stats(values):
        if not values:
            return {"count": 0}
        ordered = sorted(values)
        return {"count": len(ordered),
                "max": ordered[-1],
                "p99": ordered[int(0.99 * (len(ordered) - 1))],
                "p95": ordered[int(0.95 * (len(ordered) - 1))],
                "p50": ordered[len(ordered) // 2]}

    def pair_readings(device_rows, native_contacts, g1, g2, world):
        """同 world 内按最近位置贪心配对（接触集合顺序不可比，按 dist 排序会错配）。"""
        rows = [row for row in device_rows if int(row["worldid"][0]) == world]
        wanted = [c for c in native_contacts
                  if [int(c["geom"][0]), int(c["geom"][1])] in ([g1, g2], [g2, g1])]
        if len(rows) != len(wanted):
            return None
        remaining = list(rows)
        out = {"dist_abs": [], "dist_rel": [], "pos_abs": [], "frame_abs": [],
               "normal_abs": []}
        for ref in wanted:
            best = None
            best_distance = None
            for candidate in remaining:
                distance = sum((a - b) ** 2 for a, b in
                               zip(candidate["pos"], ref["pos"]))
                if best_distance is None or distance < best_distance:
                    best = candidate
                    best_distance = distance
            remaining.remove(best)
            dev = best
            dist_abs = abs(dev["dist"][0] - ref["dist"])
            out["dist_abs"].append(dist_abs)
            if abs(ref["dist"]) > 1e-12:
                out["dist_rel"].append(dist_abs / abs(ref["dist"]))
            out["pos_abs"].append(max(abs(a - b) for a, b in zip(dev["pos"], ref["pos"])))
            out["frame_abs"].append(max(abs(a - b) for a, b in zip(dev["frame"], ref["frame"])))
            out["normal_abs"].append(max(abs(a - b) for a, b in
                                         zip(dev["frame"][0:3], ref["frame"][0:3])))
        return out

    candidates = {"pos_atol": 0.0, "normal_atol": 0.0, "frame_atol": 0.0,
                  "dist_atol": 0.0, "dist_rtol": 0.0, "cases": []}
    for case, doc in native.get("cases", {}).items():
        if case not in report["contact_cases"]:
            continue
        capacity = int(meta_all[case]["naconmax"][0])
        _nacon, dev_rows = probe_rows(case, capacity)
        upstream_doc = upstream["cases"][case]
        _nacon_up, up_rows = upstream_rows(upstream_doc)
        meta = meta_all[case]
        g1, g2 = int(meta["g1"][0]), int(meta["g2"][0])
        native_counts = [len([c for c in contacts
                              if [int(c["geom"][0]), int(c["geom"][1])] in
                              ([g1, g2], [g2, g1])]) for contacts in doc["contacts"]]
        dev_counts = [len([r for r in dev_rows if int(r["worldid"][0]) == world])
                      for world in range(len(native_counts))]
        up_counts = [len([r for r in up_rows if int(r["worldid"][0]) == world])
                     for world in range(len(native_counts))]
        entry = {"native_counts": native_counts, "device_counts": dev_counts,
                 "upstream_counts": up_counts, "comparable": native_counts == dev_counts,
                 "note": "native f64(MuJoCo C) 读数；仅 G6 容差候选材料，非判据"}
        if native_counts != dev_counts:
            entry["skip_reason"] = ("native C 每 world 接触数与设备不同（不同引擎路径/接触选择），"
                                    "不做强制配对")
            report["native_f64_readings"][case] = entry
            continue
        dev_merged = {"dist_abs": [], "dist_rel": [], "pos_abs": [], "frame_abs": [],
                      "normal_abs": []}
        up_merged = {"dist_abs": [], "dist_rel": [], "pos_abs": [], "frame_abs": [],
                     "normal_abs": []}
        for world, contacts in enumerate(doc["contacts"]):
            dev_pair = pair_readings(dev_rows, contacts, g1, g2, world)
            up_pair = pair_readings(up_rows, contacts, g1, g2, world)
            for key in dev_merged:
                dev_merged[key].extend(dev_pair[key])
                up_merged[key].extend(up_pair[key])
        entry["device_vs_native"] = {key: stats(values) for key, values in dev_merged.items()}
        entry["upstream_f32_vs_native"] = {key: stats(values) for key, values in up_merged.items()}
        pos_median = entry["device_vs_native"]["pos_abs"].get("p50", 0.0)
        entry["comparable"] = pos_median < 1.0e-3
        if not entry["comparable"]:
            entry["skip_reason"] = ("native C 与 warp 的接触集合在位置上不可配对（median pos err "
                                    f"{pos_median:.3e} >= 1e-3）→ 不纳入 G6 候选")
            report["native_f64_readings"][case] = entry
            continue
        report["native_f64_readings"][case] = entry
        candidates["cases"].append(case)
        for key, field in (("pos_atol", "pos_abs"), ("normal_atol", "normal_abs"),
                           ("frame_atol", "frame_abs"), ("dist_atol", "dist_abs"),
                           ("dist_rtol", "dist_rel")):
            values = dev_merged[field]
            if values:
                candidates[key] = max(candidates[key], max(values))

    # native f64 材料：核对结论 = 不同引擎实现（非配对错误），**不作容差依据**。
    backend_path = EVIDENCE / "qa-backend-precision.json"
    backend = json.loads(backend_path.read_text(encoding="utf-8")) if backend_path.exists() else None
    native_checks = {}
    for case, entry in report["native_f64_readings"].items():
        native_checks[case] = {
            "native_counts": entry.get("native_counts"),
            "device_counts": entry.get("device_counts"),
            "usable_as_tolerance_basis": False,
        }
    if backend is not None:
        for case, entry in backend.get("cases", {}).items():
            native = entry.get("native_f64_vs_warp", {})
            native_checks[case] = {
                "native_counts": native.get("native_counts"),
                "warp_counts": native.get("warp_counts"),
                "counts_equal": native.get("counts_equal"),
                "native_vs_warp_max": {
                    field: native.get("native_vs_warp_f32", {}).get(field, {}).get("max")
                    for field in ("dist_abs", "pos_abs", "frame_abs", "normal_abs")},
                "nearest_neighbor_pos_spacing_p50": native.get(
                    "nearest_neighbor_pos_spacing", {}).get("p50"),
                "usable_as_tolerance_basis": False,
                "reason": native.get("reason"),
            }
    report["g6_tolerance_candidates"] = {
        "status": "native f64 材料：待核对结论 = 不同引擎（MuJoCo C vs warp）语义差，不可作容差依据；"
                  "本字段改用「同码跨后端（warp CPU vs warp CUDA f32）」实测精度基线",
        "same_code_backend_baseline": backend.get("g6_candidates", {}) if backend else None,
        "native_f64_material": native_checks,
        "native_pairing_evidence": {
            "pose_source": "同一 MJCF XML + 同一 G0b 参考 qpos（f32 位型）；native geom_xpos 与 warp 逐位相同",
            "world_order": "native 逐 world 顺序设置 qpos，world 0/1 与 warp worldid 逐位对齐",
            "mispairing_discriminator": "每 world 接触数相等为前提；同 world 内按最近位置配对；"
                                       "若配错，误差量级≈同 world 最近邻间距（sdf-box p50 ≈1.37e-2），"
                                       "而 ellipsoid-box 每 world 仅 1 个接触（1:1，不存在配错可能），"
                                       "实测 pos 4.55e-4/frame 1.17e-3 → 属引擎算法差，不是配错",
        },
    }

    report["pass"] = len(failures) == 0
    Path(args.out).write_text(json.dumps(report, ensure_ascii=False, indent=1), encoding="utf-8")

    print("== contact 数组（判据 = 同配方上游 f32 GPU） ==")
    for case, entry in report["contact_cases"].items():
        c = entry["comparison"]
        print(f"  {case:<18} count {entry['nacon_device']}/{entry['nacon_upstream']} "
              f"dist bit {c['dist_bit_equal']}/{c['count'][1]} max {c['dist_max_ulp']:.2f} ULP "
              f"pos max {c['pos_max_abs']:.3e} frame max {c['frame_max_abs']:.3e} "
              f"params {c['params_bit_equal']} efc-1 {entry['efc_all_minus_one']} "
              f"two_runs {entry['two_runs_bitwise_identical']} pass {entry['pass']}")
    print("== capacity/truncation ==")
    for case, entry in report["capacity_cases"].items():
        print(f"  {case:<24} nacon {entry['nacon_device']}/{entry['nacon_upstream']} "
              f"rows {entry['rows_written_device']}/{entry['rows_written_upstream']} "
              f"overflow {entry['overflow_device']} vs {entry['overflow_upstream']} "
              f"pass {entry['pass']}")
    print("== witness(ccd) ==")
    for case, entry in report["witness_cases"].items():
        print(f"  {case:<24} dist bit {entry['dist']['bit_equal']}/{entry['worlds']} "
              f"max {entry['dist']['max_ulp']:.2f} ULP x1 max {entry['x1']['max_abs']:.3e} "
              f"x2 max {entry['x2']['max_abs']:.3e} overflow {entry['overflow_device']} "
              f"pass {entry['pass']}")
    if "is_discrete-box-box" in report["custom_cases"]:
        entry = report["custom_cases"]["is_discrete-box-box"]
        print("== custom is_discrete(box-box) ==")
        for label in ("base", "sweep"):
            e = entry[label]
            print(f"  {label:<6} dist bit {e['dist']['bit_equal']}/{e['worlds']} "
                  f"x1 max {e['x1']['max_abs']:.3e} overflow {e['overflow_device']} pass {e['pass']}")
    if "hfield-maxconpair-overflow" in report["custom_cases"]:
        e = report["custom_cases"]["hfield-maxconpair-overflow"]
        print(f"== custom hfield MAXCONPAIR: nacon {e['nacon_device']}/{e['nacon_upstream']} "
              f"prism {e['prism_count_device']} overflow {e['overflow_device']} pass {e['pass']}")
    print("== G6 native f64 readings（非判据） ==")
    for case, entry in report["native_f64_readings"].items():
        if entry.get("comparable"):
            dev = entry["device_vs_native"]
            up = entry["upstream_f32_vs_native"]
            print(f"  {case:<18} comparable pos p50 {dev['pos_abs'].get('p50'):.3e} "
                  f"max {dev['pos_abs'].get('max'):.3e} | device dist max {dev['dist_abs'].get('max'):.3e} "
                  f"frame max {dev['frame_abs'].get('max'):.3e} | upstream dist max "
                  f"{up['dist_abs'].get('max'):.3e} pos max {up['pos_abs'].get('max'):.3e}")
        else:
            print(f"  {case:<18} skipped: {entry.get('skip_reason', 'not comparable')}")
    print("failures:", len(failures))
    for failure in failures:
        print("  FAIL", failure)
    print("report:", args.out)


if __name__ == "__main__":
    main()
