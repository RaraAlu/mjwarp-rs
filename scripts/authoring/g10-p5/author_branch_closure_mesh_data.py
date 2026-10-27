#!/usr/bin/env python3
"""Authoring-only generator for tests/support/branch_closure_mesh_data.rs (S4 / UNC-3 U6-U8).

Why this script exists
----------------------
The S4 branch-closure device target needs the MuJoCo mesh topology arrays
(`mesh_vert` / `mesh_graph` + the 9 `mesh_poly*` arrays) of three S3 fixtures as Rust
constants, because product tests must never launch Python.  The constants therefore live
in `tests/support/branch_closure_mesh_data.rs`, and this script is the pinned,
re-runnable provenance for them: it re-compiles the same fixtures with the pinned
MuJoCo Python bindings and must reproduce the checked-in file byte for byte.

It is authoring-only: the product crate, the device sources and the test targets never
call it, and it is not part of any build or test path.

Required environment (pinned)
-----------------------------
* Python 3.12.13 (`target/mjwarp-parallel/venv/Scripts/python.exe`)
* mujoco 3.15.0 (pinned; the polygon/hull layout is produced by the MuJoCo compiler,
  so a different MuJoCo version may legitimately produce different orderings)
* numpy (only used for exact f32 bit handling)

Usage
-----
    <pinned-python> scripts/authoring/g10-p5/author_branch_closure_mesh_data.py --check
    <pinned-python> scripts/authoring/g10-p5/author_branch_closure_mesh_data.py --table
    <pinned-python> scripts/authoring/g10-p5/author_branch_closure_mesh_data.py --emit <path>

`--check` (default) verifies the pinned input fixture sha256 values, regenerates the
whole Rust file in memory and compares it byte for byte with the checked-in constant
file, printing a per-array sha256 digest.  A CRLF/BOM normalized checkout (Windows
`core.autocrlf=true`) is accepted and reported; the constant bytes themselves are never
normalized.  Exit code 0 = PASS, 1 = mismatch.
"""

from __future__ import annotations

import argparse
import hashlib
import sys

import mujoco
import numpy as np

PINNED_MUJOCO = "3.15.0"
PINNED_PYTHON = "3.12.13"

REPO_ROOT = __import__("pathlib").Path(__file__).resolve().parents[3]
OUTPUT_REL = "tests/support/branch_closure_mesh_data.rs"

# Pinned inputs: the three S3 fixtures plus the OBJ assets they reference.
INPUTS = [
    ("fixtures/g10-p5-mesh-multicontact/mesh-mesh-two-assets.xml", "1d71151364f4ba910308c80a42817f7a9b7fe843e42afb8396a3b5277f1b3bfb", "U6 model"),
    ("fixtures/g10-p5-primitive/tetra.obj", "051e75316a32b22094343d249d1b4c5a08af1659f537ab40d9f263fe79a3fe0a", "U6 mesh asset"),
    ("fixtures/g10-p5-mesh-multicontact/cube.obj", "90a8a990ae17a9265c28ada66788d2f523e15653a1419a28b8074c6edf17603d", "U6 mesh asset"),
    ("fixtures/g10-p5-mesh-multicontact/mesh-limits-cone-box.xml", "6ce3552f2cab48fdbbb66a6d397790d18748dcb0a5a87b91186b5742e297f731", "U7 model"),
    ("fixtures/g10-p5-mesh-multicontact/fan-cone-24.obj", "4bc34a51a60d76469a68c64e6132cfd1f5713072488ea8deefd44a4ca517f73b", "U7 mesh asset"),
    ("fixtures/g10-p5-mesh-multicontact/mesh-limits-prism-box.xml", "1bee35cfa3378d96d18317ff75bbf4af0561f7bd71915574e2ea596bc8d4f3b5", "U8 model"),
    ("fixtures/g10-p5-mesh-multicontact/prism-20.obj", "a4477d3b080479e7356ac14bcd67c33272b2771eadd40b14fe3f40360e92cee4", "U8 mesh asset"),
]

# (group prefix, model XML) in the order the constants appear in the constant file.
SPECS = [
    ("U6", "fixtures/g10-p5-mesh-multicontact/mesh-mesh-two-assets.xml"),
    ("U7", "fixtures/g10-p5-mesh-multicontact/mesh-limits-cone-box.xml"),
    ("U8", "fixtures/g10-p5-mesh-multicontact/mesh-limits-prism-box.xml"),
]

# (constant suffix, mjModel field, Rust type) - the 14 arrays consumed by MeshData.
FIELDS = [
    ("VERT", "mesh_vert", "f32"),
    ("GRAPH", "mesh_graph", "i32"),
    ("VERTADR", "mesh_vertadr", "i32"),
    ("VERTNUM", "mesh_vertnum", "i32"),
    ("GRAPHADR", "mesh_graphadr", "i32"),
    ("POLYNUM", "mesh_polynum", "i32"),
    ("POLYADR", "mesh_polyadr", "i32"),
    ("POLYNORMAL", "mesh_polynormal", "f32"),
    ("POLYVERTADR", "mesh_polyvertadr", "i32"),
    ("POLYVERTNUM", "mesh_polyvertnum", "i32"),
    ("POLYVERT", "mesh_polyvert", "i32"),
    ("POLYMAPADR", "mesh_polymapadr", "i32"),
    ("POLYMAPNUM", "mesh_polymapnum", "i32"),
    ("POLYMAP", "mesh_polymap", "i32"),
]

# Verbatim header of the checked-in constant file (lines 1-28).  The wording of the
# "generation script" paragraph describes the state at commit b3e56a5f; it is kept
# verbatim so that regeneration stays byte-identical (this script is now in the repo).
HEADER = """//! S4 · UNC-3 分支收口：U6/U7/U8 的 mesh 拓扑常量（**authoring-only 生成**，不写产品路径）。
//!
//! 数据来源：pinned mujoco 3.15.0 从 S3 落盘 fixture 解析出的 `mjModel` 字段（逐值同源）：
//! - U6 `fixtures/g10-p5-mesh-multicontact/mesh-mesh-two-assets.xml`（cube.obj + tetra.obj）
//! - U7 `fixtures/g10-p5-mesh-multicontact/mesh-limits-cone-box.xml`（fan-cone-24.obj）
//! - U8 `fixtures/g10-p5-mesh-multicontact/mesh-limits-prism-box.xml`（prism-20.obj）
//!
//! 生成脚本：authoring-only（未入库；与本文件同源生成，任务证据记录其用法）（本文件已在任务证据
//! 记录 sha256；target 只读本常量 + S3 参考 JSON + 只读 fixture sha，不运行 Python）。
//! 设备侧 `multicontact_mesh.cu` 消费的就是这些字段（`mesh_vert` / `mesh_graph` + 9 个 polygon
//! 数组，形状语义见 M0a 修正片 `validate_mesh_poly`）。

pub(crate) struct MeshData {
    pub(crate) vert: &'static [f32],
    pub(crate) graph: &'static [i32],
    pub(crate) vertadr: &'static [i32],
    pub(crate) vertnum: &'static [i32],
    pub(crate) graphadr: &'static [i32],
    pub(crate) polynum: &'static [i32],
    pub(crate) polyadr: &'static [i32],
    pub(crate) polynormal: &'static [f32],
    pub(crate) polyvertadr: &'static [i32],
    pub(crate) polyvertnum: &'static [i32],
    pub(crate) polyvert: &'static [i32],
    pub(crate) polymapadr: &'static [i32],
    pub(crate) polymapnum: &'static [i32],
    pub(crate) polymap: &'static [i32],
}"""


def sha256_file(path) -> str:
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def rust_f32(value) -> str:
    """Shortest round-trip Rust `Debug`-style representation of an f32 value."""
    narrow = np.float32(value)
    if narrow == 0:
        return "-0.0" if np.signbit(narrow) else "0.0"
    positional = np.format_float_positional(narrow, unique=True, trim="-")
    scientific = np.format_float_scientific(narrow, unique=True, trim="-")
    parts = scientific.split("e")
    exponent = int(parts[1]) if len(parts) > 1 else 0
    if exponent < -4 or exponent >= 16:
        return scientific
    return positional if "." in positional else positional + ".0"


def wrap_items(items, per_line):
    lines = []
    for start in range(0, len(items), per_line):
        lines.append("    " + ", ".join(items[start:start + per_line]) + ",")
    return lines


def extract_arrays(xml_path):
    """Return the 14 arrays for one model, flattened exactly as the device consumes them."""
    model = mujoco.MjModel.from_xml_path(str(xml_path))
    arrays = []
    for suffix, field, rust_type in FIELDS:
        array = np.asarray(getattr(model, field)).reshape(-1)
        if rust_type == "f32":
            array = array.astype(np.float32, copy=False)
        else:
            array = array.astype(np.int32, copy=False)
        arrays.append((suffix, rust_type, array))
    return arrays


def canonical_bytes(rust_type, array) -> bytes:
    if rust_type == "f32":
        return np.asarray(array, dtype="<f4").tobytes()
    return np.asarray(array, dtype="<i4").tobytes()


def data_block(name, rust_type, array) -> str:
    values = [rust_f32(value) if rust_type == "f32" else str(int(value)) for value in array]
    per_line = 8 if rust_type == "f32" else 20
    lines = ["#[rustfmt::skip]", f"pub(crate) const {name}: [{rust_type}; {len(array)}] = ["]
    lines.extend(wrap_items(values, per_line))
    lines.append("];")
    return "\n".join(lines)


def mesh_block(group) -> str:
    lines = ["#[rustfmt::skip]", f"pub(crate) const {group}_MESH: MeshData = MeshData {{"]
    lines.extend(f"    {suffix.lower()}: &{group}_{suffix}," for suffix, _field, _ty in FIELDS)
    lines.append("};")
    return "\n".join(lines)


def build_text():
    """Return (rust_source, rows) where rows carries the per-array sha256 digest."""
    blocks = []
    rows = []
    for group, rel_xml in SPECS:
        for suffix, rust_type, array in extract_arrays(REPO_ROOT / rel_xml):
            name = f"{group}_{suffix}"
            blocks.append(data_block(name, rust_type, array))
            rows.append((group, name, rust_type, len(array), canonical_bytes(rust_type, array)))
    blocks.extend(mesh_block(group) for group, _rel in SPECS)
    source = HEADER + "\n\n" + "\n\n".join(blocks) + "\n"
    return source, rows


def combined_digest(rows) -> str:
    digest = hashlib.sha256()
    for _group, name, rust_type, count, raw in rows:
        digest.update(f"{name}|{rust_type}|{count}|".encode("ascii"))
        digest.update(raw)
    return digest.hexdigest()


def array_table(rows):
    print("| group | const | count | bytes | sha256 (LE raw values) |")
    print("| --- | --- | --- | --- | --- |")
    for group, name, _rust_type, count, raw in rows:
        print(f"| {group} | `{name}` | {count} | {len(raw)} | `{hashlib.sha256(raw).hexdigest()}` |")


def check_inputs():
    ok = True
    for rel_path, expected, role in INPUTS:
        actual = sha256_file(REPO_ROOT / rel_path)
        status = "OK" if actual == expected else "MISMATCH"
        if actual != expected:
            ok = False
        print(f"INPUT {status} {rel_path} role={role} bytes={(REPO_ROOT / rel_path).stat().st_size} sha256={actual} expected={expected}")
    return ok


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--check", action="store_true", help="verify inputs and byte-compare regenerated output (default)")
    parser.add_argument("--emit", metavar="PATH", help="write the regenerated Rust file to PATH")
    parser.add_argument("--table", action="store_true", help="print the markdown table of input/output digests")
    parser.add_argument("--allow-mujoco-version-mismatch", action="store_true", help="do not require pinned mujoco")
    args = parser.parse_args()

    print(f"ENV python={sys.version.split()[0]} mujoco={mujoco.__version__} numpy={np.__version__}")
    if mujoco.__version__ != PINNED_MUJOCO and not args.allow_mujoco_version_mismatch:
        print(f"FAIL pinned mujoco {PINNED_MUJOCO} required, got {mujoco.__version__}")
        return 1
    if sys.version.split()[0] != PINNED_PYTHON:
        print(f"WARN pinned python {PINNED_PYTHON}, got {sys.version.split()[0]}")

    inputs_ok = check_inputs()
    source, rows = build_text()
    digest = combined_digest(rows)
    print(f"ARRAYS groups={len(SPECS)} fields_per_group={len(FIELDS)} total={len(rows)} combined_sha256={digest}")

    if args.table:
        array_table(rows)
        print(f"OUTPUT {OUTPUT_REL} bytes={len(source.encode('utf-8'))} sha256={hashlib.sha256(source.encode('utf-8')).hexdigest()}")

    if args.emit:
        target = REPO_ROOT / args.emit
        with open(target, "w", encoding="utf-8", newline="") as handle:
            handle.write(source)
        print(f"EMIT {args.emit} bytes={target.stat().st_size} sha256={sha256_file(target)}")

    if args.emit is None or args.check:
        checked_in = REPO_ROOT / OUTPUT_REL
        raw = checked_in.read_bytes()
        if raw.startswith(b"\xef\xbb\xbf"):
            raw = raw[3:]
            print("CHECK normalized=BOM-stripped")
        current = raw.decode("utf-8")
        normalized = current.replace("\r\n", "\n")
        if normalized != current:
            print("CHECK normalized=CRLF-to-LF")
        same = normalized == source
        print(f"CHECK {OUTPUT_REL} bytes_checked_in={checked_in.stat().st_size} sha256_checked_in={sha256_file(checked_in)}")
        print(f"FULL_FILE_MATCH={same}")
        if not same:
            for index, (generated_line, current_line) in enumerate(zip(source.split("\n"), current.split("\n")), start=1):
                if generated_line != current_line:
                    print(f"DIFF line {index}\n  generated={generated_line!r}\n  checked_in={current_line!r}")
                    break
        print(f"RESULT={'PASS' if inputs_ok and same else 'FAIL'}")
        return 0 if inputs_ok and same else 1
    return 0 if inputs_ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
