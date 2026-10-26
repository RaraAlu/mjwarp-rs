"""GPU 参考采集的 CPU 侧准备/校验（不触发 GPU、不跑 warp 后端）：

1. 旧 CPU 参考（`upstream-mjwarp-f32.json`）的输入契约自检：5 case 的 qpos/qpos_bits/xml_sha256 与
   夹具/fixtures 逐位对得上，`upstream_revision` 正确；
2. 派生脚本 `capture_mjwarp_f32_gpu.py` 与旧脚本的 `CASES`/`FLOAT_FIELDS`/`INT_FIELDS`/`load_qpos`
   逻辑逐字比对（diff 只允许 device/输出名/kind/task_id/purpose/provenance/kernel cache 目录）；
3. 输出对照表 `inputs-vs-old.json`（供证据 README 使用）。

用法（CPU，任意时间）：
    python -B verify_inputs.py
退出码 0 = 全部一致；非 0 = 有差异（打印明细）。
"""

import hashlib
import json
import struct
import sys
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

ROOT = Path(r"D:\Rust\mjwarp-rs")
EVIDENCE = ROOT / "docs/development-evidence/g10-p5-g2-gpu-reference-capture"
OLD = ROOT / "docs/development-evidence/g10-p5-gjk-epa/upstream-mjwarp-f32.json"
OLD_SCRIPT = ROOT / "docs/development-evidence/g10-p5-gjk-epa/capture_mjwarp_f32.py"
NEW_SCRIPT = EVIDENCE / "capture_mjwarp_f32_gpu.py"
UPSTREAM_REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
CASES = [
    ("cylinder-box", "fixtures/g10-p5/models/cylinder-box.xml", "fixtures/g10-p5/references/cylinder-box.json", True),
    ("ellipsoid-box", "fixtures/g10-p5/models/ellipsoid-box.xml", "fixtures/g10-p5/references/ellipsoid-box.json", True),
    ("mesh-mesh", "fixtures/g10-p5/models/mesh-mesh.xml", "fixtures/g10-p5/references/mesh-mesh.json", True),
    ("convex-mesh", "fixtures/mjwarp-complex-physics/convex-mesh.xml", "fixtures/mjwarp-complex-physics/reference.json", False),
    ("degenerate-mesh", "fixtures/mjwarp-complex-physics/degenerate-mesh.xml", "fixtures/mjwarp-complex-physics/reference.json", False),
]
FLOAT_FIELDS = ("dist", "pos", "frame", "includemargin", "friction", "solref",
                "solreffriction", "solimp", "adhesion")
INT_FIELDS = ("dim", "geom", "flex", "elem", "vert", "worldid", "type", "geomcollisionid")


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def bits32(values):
    return tuple(struct.unpack("<I", struct.pack("<f", float(v)))[0] for v in values)


def load_qpos(reference_path, standalone, name):
    value = json.loads((ROOT / reference_path).read_text(encoding="utf-8"))
    if not standalone:
        value = next(case for case in value["collision_cases"] if case["name"] == name)
    return value["input"]["qpos"]["values"]


def extract_block(text, marker):
    start = text.index(marker)
    depth = 0
    for index in range(start, len(text)):
        char = text[index]
        if char in "([{":
            depth += 1
        elif char in ")]}":
            depth -= 1
            if depth == 0:
                return text[start:index + 1]
    raise ValueError(f"unbalanced block for {marker}")


def main():
    failures = []

    if not NEW_SCRIPT.exists():
        print(f"FAIL 派生脚本不存在：{NEW_SCRIPT}")
        return 1
    if not OLD.exists() or not OLD_SCRIPT.exists():
        print("FAIL 旧参考/旧脚本不存在")
        return 1

    old = json.loads(OLD.read_text(encoding="utf-8"))
    print(f"old sha256 = {sha256(OLD)}")
    print(f"old script sha256 = {sha256(OLD_SCRIPT)}")
    print(f"new script sha256 = {sha256(NEW_SCRIPT)}")
    print(f"old upstream_revision = {old['upstream_revision']}")
    if old["upstream_revision"] != UPSTREAM_REVISION:
        failures.append("old upstream_revision 不匹配")
    if old["provenance"]["device"] != "cpu":
        failures.append(f"old provenance.device 应为 cpu：{old['provenance']['device']}")

    # 2) 派生脚本逐字块比对：CASES / FLOAT_FIELDS / INT_FIELDS / load_qpos 函数体
    old_text = OLD_SCRIPT.read_text(encoding="utf-8")
    new_text = NEW_SCRIPT.read_text(encoding="utf-8")
    old_cases = extract_block(old_text, "CASES = [")
    new_cases = extract_block(new_text, "CASES = [")
    if old_cases != new_cases:
        failures.append("CASES 块与旧脚本不一致")
    else:
        print("CASES 块逐字一致 ✓")
    for marker in ("FLOAT_FIELDS = (", "INT_FIELDS = ("):
        if extract_block(old_text, marker) != extract_block(new_text, marker):
            failures.append(f"{marker.split(' =')[0]} 与旧脚本不一致")
        else:
            print(f"{marker.split(' =')[0]} 块逐字一致 ✓")
    old_capture = extract_block(old_text, "def capture(")
    new_capture = extract_block(new_text, "def capture(")
    if old_capture != new_capture:
        failures.append("capture() 函数体与旧脚本不一致（两次捕获/17 字段逻辑必须逐字不变）")
    else:
        print("capture() 块逐字一致 ✓")
    old_load = extract_block(old_text, "def load_qpos(")
    new_load = extract_block(new_text, "def load_qpos(")
    if old_load != new_load:
        failures.append("load_qpos() 函数体与旧脚本不一致")
    else:
        print("load_qpos() 块逐字一致 ✓")

    # 3) 旧参考输入自检（qpos/qpos_bits/xml_sha256/rev）
    table = {"upstream_revision": UPSTREAM_REVISION, "cases": {}}
    print("\n== 旧 CPU 参考的输入契约自检 ==")
    print(f"{'case':<16} {'count':>5} {'nq':>3} {'nworld':>6} {'qpos_bits_match':>15} {'xml_sha256_match':>16}")
    for name, xml_path, reference_path, standalone in CASES:
        entry = old["cases"].get(name)
        if entry is None:
            failures.append(f"旧参考缺 case {name}")
            continue
        values = load_qpos(reference_path, standalone, name)
        expected_bits = [bits32([value])[0] for value in values]
        bits_match = entry["qpos_bits"] == expected_bits and entry["qpos"] == values
        xml_match = entry["xml_sha256"] == sha256(ROOT / xml_path)
        if not bits_match:
            failures.append(f"{name} qpos/qpos_bits 与夹具不一致")
        if not xml_match:
            failures.append(f"{name} xml_sha256 与模型文件不一致")
        table["cases"][name] = {
            "xml": xml_path,
            "xml_sha256": entry["xml_sha256"],
            "xml_sha256_matches_file": xml_match,
            "qpos_len": len(values),
            "qpos_bits": entry["qpos_bits"],
            "qpos_bits_match_fixture": bits_match,
            "count": entry["output"]["count"],
            "two_captures_bitwise_identical": entry["two_captures_bitwise_identical"],
        }
        print(f"{name:<16} {entry['output']['count']:>5} {'-':>3} {'-':>6} {str(bits_match):>15} {str(xml_match):>16}")

    EVIDENCE.mkdir(parents=True, exist_ok=True)
    out = EVIDENCE / "inputs-vs-old.json"
    out.write_text(json.dumps(table, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"\nwrote {out}")

    if failures:
        print("\nFAIL:")
        for failure in failures:
            print("  -", failure)
        return 1
    print("\nOK: 输入契约与派生脚本块比对全部一致")
    return 0


if __name__ == "__main__":
    sys.exit(main())
