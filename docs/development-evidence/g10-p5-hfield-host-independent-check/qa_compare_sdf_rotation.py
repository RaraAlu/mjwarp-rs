"""SDF 相对旋转约定判定：上游 f32 / 产品原样（转置 helper）/ 修正语义（上游 aᵀb）三方对照。

用法：python qa_compare_sdf_rotation.py
输出：qa-sdf-rotation-report.json（逐场景逐 world 的量级数字 + 结论）
"""

import json
import struct
from pathlib import Path

EVIDENCE = Path(__file__).resolve().parent
CASES = EVIDENCE / "qa-sdf-rotation-cases.json"
PRODUCT = EVIDENCE / "qa-sdf-rotation-product.json"
OUT = EVIDENCE / "qa-sdf-rotation-report.json"


def bits(value):
    return struct.unpack("<I", struct.pack("<f", float(value)))[0]


def ulp_diff(a, b):
    return abs(bits(a) - bits(b))


def row_key(contact):
    return (bits(contact["dist"]),) + tuple(bits(v) for v in contact["pos"]) + tuple(
        bits(v) for v in contact["frame"]
    )


def upstream_worlds(scene):
    fields = scene["output"]["fields"]
    worlds = {}
    for index in range(scene["output"]["count"]):
        world = int(fields["worldid"][index][0])
        worlds.setdefault(world, []).append({
            "dist": fields["dist"][index][0],
            "pos": fields["pos"][index],
            "frame": fields["frame"][index],
        })
    return worlds


def compare(left, right):
    left_sorted = sorted(left, key=row_key)
    right_sorted = sorted(right, key=row_key)
    result = {
        "count": [len(left_sorted), len(right_sorted)],
        "bit_equal_contacts": sum(
            1 for a, b in zip(left_sorted, right_sorted) if row_key(a) == row_key(b)),
        "dist_max_ulp": 0,
        "dist_max_abs": 0.0,
        "dist_bit_equal": 0,
        "pos_max_abs": 0.0,
        "pos_bit_equal": 0,
        "frame_max_abs": 0.0,
        "frame_bit_equal": 0,
    }
    for a, b in zip(left_sorted, right_sorted):
        result["dist_max_ulp"] = max(result["dist_max_ulp"], ulp_diff(a["dist"], b["dist"]))
        result["dist_max_abs"] = max(result["dist_max_abs"], abs(a["dist"] - b["dist"]))
        result["dist_bit_equal"] += int(bits(a["dist"]) == bits(b["dist"]))
        pos_equal = all(bits(x) == bits(y) for x, y in zip(a["pos"], b["pos"]))
        result["pos_bit_equal"] += int(pos_equal)
        for x, y in zip(a["pos"], b["pos"]):
            result["pos_max_abs"] = max(result["pos_max_abs"], abs(x - y))
        frame_equal = all(bits(x) == bits(y) for x, y in zip(a["frame"], b["frame"]))
        result["frame_bit_equal"] += int(frame_equal)
        for x, y in zip(a["frame"], b["frame"]):
            result["frame_max_abs"] = max(result["frame_max_abs"], abs(x - y))
    return result


def main():
    cases = json.loads(CASES.read_text(encoding="utf-8"))
    product = json.loads(PRODUCT.read_text(encoding="utf-8"))
    product_scenes = {scene["name"]: scene for scene in product["scenes"]}
    report = {
        "schema": 1,
        "task_id": "2026-10-25-g10-p5-hfield-host-independent-check",
        "kind": "qa-sdf-relative-rotation-verdict",
        "criterion": "上游 f32（device=cpu，两捕获位等）为唯一参考；product=产品原样，corrected=上游 aᵀb 语义",
        "oct_files_bitwise_equal_to_fixture": {
            name: value["bitwise_equal_to_fixture"]
            for name, value in cases["oct_files"].items()
        },
        "scenes": [],
    }
    for scene in cases["scenes"]:
        name = scene["name"]
        upstream = upstream_worlds(scene)
        product_scene = product_scenes[name]
        row = {
            "scene": name,
            "note": scene["note"],
            "initpoints": scene["sdf_initpoints"],
            "iterations": scene["sdf_iterations"],
            "qr": {},
        }
        for world_out in product_scene["worlds"]:
            world = int(world_out["world"])
            expected = upstream.get(world, [])
            product_rows = world_out["product_contacts"]
            corrected_rows = world_out["corrected_computed_relmat_contacts"]
            warp_rows = world_out["corrected_warp_relmat_contacts"]
            fma_rows = world_out["corrected_fma_relmat_contacts"]
            product_cmp = compare(product_rows, expected)
            corrected_cmp = compare(corrected_rows, expected)
            warp_cmp = compare(warp_rows, expected)
            fma_cmp = compare(fma_rows, expected)
            rel_product = world_out["rel_mat_product"]
            rel_upstream = world_out["rel_mat_upstream_computed"]
            rel_warp = world_out["rel_mat_warp"]
            rel_max_abs = max(abs(a - b) for a, b in zip(rel_product, rel_upstream))
            rel_computed_vs_warp = sum(
                1 for a, b in zip(rel_upstream, rel_warp) if bits(a) != bits(b))
            row["qr"][f"world{world}"] = {
                "upstream_count": len(expected),
                "product": product_cmp,
                "corrected": corrected_cmp,
                "corrected_warp_relmat": warp_cmp,
                "corrected_fma_relmat": fma_cmp,
                "rel_mat_fma_matches_warp": bool(world_out["rel_mat_fma_matches_warp"]),
                "rel_mat_max_abs_diff_product_vs_upstream": rel_max_abs,
                "rel_mat_cells_diff_computed_vs_warp": rel_computed_vs_warp,
                "rel_mat_product": rel_product,
                "rel_mat_upstream": rel_upstream,
                "rel_mat_warp": rel_warp,
            }
        report["scenes"].append(row)
        print(f"== {name} ({scene['note']})")
        for key, value in row["qr"].items():
            product_cmp = value["product"]
            corrected_cmp = value["corrected"]
            warp_cmp = value["corrected_warp_relmat"]
            fma_cmp = value["corrected_fma_relmat"]
            print(f"  {key}: count up={value['upstream_count']} "
                  f"product={product_cmp['count'][0]} corrected={corrected_cmp['count'][0]}")
            print(f"    product  : bit-equal contacts {product_cmp['bit_equal_contacts']}, "
                  f"dist max {product_cmp['dist_max_abs']:.3e} abs / {product_cmp['dist_max_ulp']} ULP, "
                  f"pos max {product_cmp['pos_max_abs']:.3e}, frame max {product_cmp['frame_max_abs']:.3e}")
            print(f"    corrected: bit-equal contacts {corrected_cmp['bit_equal_contacts']}, "
                  f"dist max {corrected_cmp['dist_max_abs']:.3e} abs / {corrected_cmp['dist_max_ulp']} ULP, "
                  f"pos max {corrected_cmp['pos_max_abs']:.3e}, frame max {corrected_cmp['frame_max_abs']:.3e}")
            print(f"    corrected+warp_relmat: bit-equal contacts {warp_cmp['bit_equal_contacts']}, "
                  f"dist max {warp_cmp['dist_max_abs']:.3e} abs, pos max {warp_cmp['pos_max_abs']:.3e}, "
                  f"frame max {warp_cmp['frame_max_abs']:.3e}")
            print(f"    corrected+fma_relmat: bit-equal contacts {fma_cmp['bit_equal_contacts']}, "
                  f"dist max {fma_cmp['dist_max_abs']:.3e} abs, pos max {fma_cmp['pos_max_abs']:.3e}, "
                  f"frame max {fma_cmp['frame_max_abs']:.3e}; fma_mat_matches_warp="
                  f"{value['rel_mat_fma_matches_warp']}")
            print(f"    rel_mat |product-upstream| max {value['rel_mat_max_abs_diff_product_vs_upstream']:.6f}; "
                  f"computed-vs-warp diff cells {value['rel_mat_cells_diff_computed_vs_warp']}")

    # 结论汇总
    verdict = {"ident_control": {}, "rotated": {}, "conclusion": ""}
    for scene in report["scenes"]:
        name = scene["scene"]
        bucket = "ident_control" if name.endswith("ident") else "rotated"
        worst_product = {"dist_max_abs": 0.0, "pos_max_abs": 0.0, "frame_max_abs": 0.0}
        worst_corrected = {"dist_max_abs": 0.0, "pos_max_abs": 0.0, "frame_max_abs": 0.0}
        worst_warp = {"dist_max_abs": 0.0, "pos_max_abs": 0.0, "frame_max_abs": 0.0}
        worst_fma = {"dist_max_abs": 0.0, "pos_max_abs": 0.0, "frame_max_abs": 0.0}
        for value in scene["qr"].values():
            for key in worst_product:
                worst_product[key] = max(worst_product[key], value["product"][key])
                worst_corrected[key] = max(worst_corrected[key], value["corrected"][key])
                worst_warp[key] = max(worst_warp[key], value["corrected_warp_relmat"][key])
                worst_fma[key] = max(worst_fma[key], value["corrected_fma_relmat"][key])
        verdict[bucket][name] = {
            "product_vs_upstream_worst": worst_product,
            "corrected_vs_upstream_worst": worst_corrected,
            "corrected_warp_relmat_vs_upstream_worst": worst_warp,
            "corrected_fma_relmat_vs_upstream_worst": worst_fma,
        }
    report["verdict"] = verdict
    OUT.write_text(json.dumps(report, ensure_ascii=False, indent=1), encoding="utf-8")
    print("wrote", OUT)


if __name__ == "__main__":
    main()
