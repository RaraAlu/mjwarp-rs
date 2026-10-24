"""Generate an independent MuJoCo 3.12 model dumper from the reference JSON schema.

The generated C++ program loads a committed .mjb with the MuJoCo 3.12 C API and
prints every array that model-312.json records, using the JSON metadata only for
names/kinds/counts (no S1 bridge code is involved). QA uses it to re-derive the
reference values from the native library itself.
"""

from __future__ import annotations

import json
import pathlib

ROOT = pathlib.Path(__file__).resolve().parents[2]
REF = ROOT / "fixtures/render-bvh/scene-a/model-312.json"
OUT = ROOT / "target/qa-render-bvh-s1/dump_model.cpp"

# name -> (C type, printf format, member expression, is_array)
SPECIAL = {
    "vis.headlight.ambient": ("float", "%.9g", "m->vis.headlight.ambient", True),
    "vis.headlight.diffuse": ("float", "%.9g", "m->vis.headlight.diffuse", True),
    "vis.headlight.specular": ("float", "%.9g", "m->vis.headlight.specular", True),
    "vis.headlight.active": ("int", "%d", "m->vis.headlight.active", False),
    "vis.map.znear": ("float", "%.9g", "m->vis.map.znear", False),
    "vis.map.zfar": ("float", "%.9g", "m->vis.map.zfar", False),
    "stat.extent": ("double", "%.17g", "m->stat.extent", False),
    "stat.center": ("double", "%.17g", "m->stat.center", True),
    "names": ("uint8_t", "%u", "m->names", True),
}
KIND = {
    "int32": ("int32_t", "%d"),
    "int64": ("int64_t", "%lld"),
    "float32": ("float", "%.9g"),
    "float64": ("double", "%.17g"),
    "byte": ("uint8_t", "%u"),
}


def emit_field(lines: list[str], name: str, ctype: str, fmt: str, expr: str, count: int, is_array: bool) -> None:
    lines.append(f'  std::printf("%s\\"{name}\\":[", first ? "" : ","); first = 0;')
    if is_array:
        lines.append(
            f'  for (int i = 0; i < {count}; ++i) std::printf(i ? ",{fmt}" : "{fmt}", ({ctype}){expr}[i]);'
        )
    else:
        lines.append(f'  std::printf("{fmt}", ({ctype}){expr});')
    lines.append('  std::printf("]");')


def main() -> int:
    payload = json.loads(REF.read_text(encoding="utf-8"))
    arrays = {k: v for k, v in payload["arrays"].items() if isinstance(v, dict)}
    lines = [
        "#include <cstdint>",
        "#include <cstdio>",
        '#include "mujoco/mujoco.h"',
        "",
        "int main(int argc, char** argv) {",
        '  if (argc < 2) { std::fprintf(stderr, "usage: %s model.mjb\\n", argv[0]); return 2; }',
        "  mjModel* m = mj_loadModel(argv[1], nullptr);",
        '  if (!m) { std::fprintf(stderr, "failed to load %s\\n", argv[1]); return 3; }',
        '  std::printf("{");',
        "  int first = 1;",
    ]
    for name, entry in arrays.items():
        if name in SPECIAL:
            ctype, fmt, expr, is_array = SPECIAL[name]
        else:
            ctype, fmt = KIND[entry["kind"]]
            expr = f"m->{name}"
            is_array = True
        emit_field(lines, name, ctype, fmt, expr, entry["count"], is_array)
    lines += ['  std::printf("}\\n");', "  mj_deleteModel(m);", "  return 0;", "}", ""]
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text("\n".join(lines), encoding="utf-8", newline="\n")
    print(f"wrote {OUT} fields={len(arrays)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
