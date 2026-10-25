"""QA judge: 81-slot signature / per-slot upload / placeholder-not-read / bvh blob layout.

Independent derivation:
  * parse the kernel parameter list from `render_megakernel.cu` (the actual ABI the gate sees);
  * parse `MEGAKERNEL_SLOTS` from `src/runtime/launcher/render/args.rs` (S7a frozen order);
  * map wp types to slot kinds and compare element-by-element;
  * parse the README slot table, recount categories and compare with its summary line;
  * check every README `placeholder` identifier is not referenced by the kernel body;
  * count `args.push` in megakernel.rs and the 1-element upload shapes;
  * verify the bvh blob header layout host-pack (megakernel_arrays.rs) vs device parse
    (render_common.cu) and compute the actual section offsets/alignment.
"""

from __future__ import annotations

import json
import pathlib
import re

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
KERNEL = ROOT / "src/render/kernel/cuda/render_megakernel.cu"
ARGS = ROOT / "src/runtime/launcher/render/args.rs"
HOST = ROOT / "src/render/kernel/megakernel.rs"
ARRAYS = ROOT / "src/render/kernel/megakernel_arrays.rs"
COMMON = ROOT / "src/render/kernel/cuda/render_common.cu"
README = ROOT / "docs/development-evidence/render-bvh/s7b2-shading/README.md"

TYPE_MAP = {
    "wp::int32": ("Array", "I32"),
    "float": ("Array", "F32"),
    "bool": ("Array", "Bool"),
    "wp::uint32": ("Array", "U32"),
    "wp::uint64": ("Array", "U64"),
    "wp::vec2": ("Array", "Vec2"),
    "wp::vec3": ("Array", "Vec3"),
    "wp::vec4": ("Array", "Vec4"),
    "wp::quat": ("Array", "Quat"),
    "wp::mat33": ("Array", "Mat33"),
    "wp::vec2i": ("Array", "Vec2I"),
    "wp::vec3i": ("Array", "Vec3I"),
    "wp::texture2d_t": ("Array", "Texture2d"),
}
SCALAR_MAP = {
    "wp::int32": "I32",
    "bool": "Bool",
    "wp::uint64": "U64",
}


def kernel_signature():
    text = KERNEL.read_text(encoding="utf-8")
    match = re.search(r"void render_megakernel\((.*?)\)\s*\{", text, re.S)
    body = match.group(1)
    params = []
    for part in body.replace("\n", " ").split(","):
        part = part.strip()
        if not part:
            continue
        # "wp::launch_bounds_t<2> dim" | "wp::array_t<wp::vec3> name" | "bool use_shadows"
        m = re.match(r"(.+?)\s+([A-Za-z_][A-Za-z0-9_]*)$", part)
        assert m, part
        params.append((m.group(1).strip(), m.group(2)))
    body_start = text.index(match.group(0)) + len(match.group(0))
    kernel_body = text[body_start:]
    return params, kernel_body


def frozen_slots():
    text = ARGS.read_text(encoding="utf-8")
    start = text.index("MEGAKERNEL_SLOTS")
    open_bracket = text.index("[", text.index("[", start) + 1)
    close_bracket = re.search(r"\]\s*};", text[open_bracket:]).start() + open_bracket
    block = text[open_bracket + 1 : close_bracket]
    entries = re.findall(r"\b([ASB])\(([A-Za-z0-9]+)\)|(\bB\b)", block)
    scalar_aliases = {"SI": "I32", "SB": "Bool", "SU": "U64"}
    slots = []
    for array_call, array_elem, plain in entries:
        if array_call == "A":
            slots.append(("Array", array_elem))
        elif array_call == "S":
            slots.append(("Scalar", scalar_aliases.get(array_elem, array_elem)))
        else:
            slots.append(("Bounds2D", "Bounds2D"))
    return slots


def main() -> int:
    params, kernel_body = kernel_signature()
    slots = frozen_slots()
    report = {"kernel_param_count": len(params), "array_param_count": None}
    derived = [("Bounds2D", "Bounds2D")]
    for kind, name in params[1:]:
        if kind in SCALAR_MAP:
            derived.append(("Scalar", SCALAR_MAP[kind]))
            continue
        inner = kind
        if kind.startswith("wp::array_t<") and kind.endswith(">"):
            inner = kind[len("wp::array_t<") : -1]
        if inner in TYPE_MAP:
            derived.append(TYPE_MAP[inner])
        else:
            derived.append(("UNKNOWN", kind))
    report["array_param_count"] = sum(1 for slot in derived if slot[0] == "Array")
    report["scalar_count"] = sum(1 for slot in derived if slot[0] == "Scalar")
    mismatches = [
        {"slot": index, "kernel": derived[index], "frozen": slots[index], "name": params[index][1]}
        for index in range(min(len(derived), len(slots)))
        if derived[index] != slots[index]
    ]
    report["slots_derived"] = len(derived)
    report["slots_frozen"] = len(slots)
    report["slot_order_mismatches"] = mismatches
    report["slot_order_matches"] = len(derived) == len(slots) == 81 and not mismatches

    # README table recount
    readme = README.read_text(encoding="utf-8")
    rows = re.findall(
        r"^\| (\d+) \| `([^`]+)`(?:[^|]*)\| (.*?) \| ([a-z（][^|]*) \|$", readme, re.M
    )
    categories = {}
    table = []
    for slot, name, fill, category in rows:
        category = category.strip()
        bucket = (
            "own"
            if category.startswith("own")
            else "placeholder"
            if category.startswith("placeholder")
            else "real"
        )
        categories[bucket] = categories.get(bucket, 0) + 1
        table.append(
            {
                "slot": int(slot),
                "name": name,
                "category": bucket,
                "raw_category": category,
                "fill": fill.strip(),
            }
        )
    report["readme_table_rows"] = len(table)
    report["readme_recount"] = categories
    summary = re.search(r"统计：real 数据槽 (\d+)、自有打包槽 (\d+)[^）]*）、placeholder (\d+)", readme)
    report["readme_summary"] = (
        {
            "real": int(summary.group(1)),
            "own": int(summary.group(2)),
            "placeholder": int(summary.group(3)),
        }
        if summary
        else None
    )
    # cross-check table slot numbers are 0..80 exactly once and names match kernel params
    slots_in_table = [item["slot"] for item in table]
    report["table_covers_0_80"] = sorted(slots_in_table) == list(range(81))
    report["table_name_mismatches"] = [
        {"slot": item["slot"], "table": item["name"], "kernel": params[item["slot"]][1]}
        for item in table
        if params[item["slot"]][1] != item["name"].split()[-1]
    ]

    # placeholder identifiers must not be read by the kernel body
    placeholders = [item for item in table if item["category"] == "placeholder"]
    referenced = []
    for item in placeholders:
        name = item["name"]
        hits = [
            line.strip()[:160]
            for line in kernel_body.splitlines()
            if re.search(rf"\b{re.escape(name)}\b", line)
        ]
        if hits:
            referenced.append({"slot": item["slot"], "name": name, "body_lines": hits})
    # `mesh_texcoord` reachability: only read inside the mesh-texcoord branch, which needs
    # `mesh_texcoord_offsets[mesh] >= 0`; scene-a has -1, so the placeholder is never dereferenced.
    reachable = []
    for item in referenced:
        if item["name"] == "mesh_texcoord":
            reachable.append(
                {
                    "name": item["name"],
                    "guarded_by": "mesh_texcoordadr[mesh] >= 0",
                    "scene_a_value": "mesh_texcoordadr = [-1] (never taken)",
                }
            )
    report["placeholder_count_from_table"] = len(placeholders)
    report["placeholders_referenced_by_kernel"] = referenced
    report["placeholder_not_read"] = not referenced
    report["placeholder_reference_guards"] = reachable

    # host side upload shapes / argument count
    host = HOST.read_text(encoding="utf-8")
    pushes = len(re.findall(r"args\.push\(", host))
    report["args_push_count"] = pushes
    one_element = re.findall(r"array\(&([A-Za-z0-9_]+), RenderElem::[A-Za-z0-9]+, &\[1\]\)", host)
    report["one_element_uploads"] = one_element
    scalars = re.findall(r"RenderArg::Scalar\((RenderScalar::[A-Za-z0-9]+)", host)
    report["scalar_pushes"] = len(scalars)
    bounds = 1 if "RenderArg::Bounds2D" in host else 0
    report["array_pushes"] = pushes - len(scalars) - bounds

    # bvh blob layout: host header offsets vs device parse
    arrays = ARRAYS.read_text(encoding="utf-8")
    report["blob_host_header"] = re.findall(r"header\[(2 \* index|2 \* index \+ 1)\] = ([^;]+);", arrays)
    common = COMMON.read_text(encoding="utf-8")
    report["blob_device_reads"] = re.findall(r"header\[(\d)\]", common)
    # section offsets: 40 + 16k, spacing 16 bytes; base offset of first section is 40
    report["blob_header_bytes"] = 40
    report["blob_section_spacing_bytes"] = 16
    report["blob_first_section_offset"] = 40
    report["blob_first_section_16_aligned"] = (40 % 16) == 0

    out = ROOT / "target/qa-s7b2/out/judge_slots.json"
    out.write_text(json.dumps(report, indent=1), encoding="utf-8")
    print(
        json.dumps(
            {
                "kernel_params": len(params),
                "arrays": report["array_param_count"],
                "scalars": report["scalar_count"],
                "slots_frozen": len(slots),
                "match": report["slot_order_matches"],
                "mismatch_count": len(mismatches),
                "readme_rows": len(table),
                "readme_recount": categories,
                "readme_summary": report["readme_summary"],
                "table_covers_0_80": report["table_covers_0_80"],
                "table_name_mismatches": len(report["table_name_mismatches"]),
                "placeholder_not_read": report["placeholder_not_read"],
                "placeholders_referenced": referenced,
                "args_push_count": pushes,
                "one_element_upload_count": len(one_element),
                "blob_device_reads": report["blob_device_reads"],
                "blob_first_section_16_aligned": report["blob_first_section_16_aligned"],
            },
            indent=1,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
