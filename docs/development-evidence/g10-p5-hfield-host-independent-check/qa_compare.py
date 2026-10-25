"""QA 三方比对（上游 f32 / QA oracle / 产品 hfield 宿主）。

- 交付 3 case：QA 独立重采 vs 交付 fixture 逐位；
- 每个场景逐 world：`worldid` 分组后按 `(dist,pos,frame)` 位型多重集比较（含重复点），
  同时比较 count、filter 判定/bounds（oracle vs 产品 probe）、prism 计数、overflow；
- 与 1e-3 去重有关的缓存重数统计（oracle 侧）。

用法：python qa_compare.py [--oracle <json>] [--product <json>]
"""

import argparse
import json
import struct
from pathlib import Path

EVIDENCE = Path(__file__).resolve().parent
ROOT = EVIDENCE.parents[2]
CASES = EVIDENCE / "qa-upstream-cases.json"


def bits(value):
    return struct.unpack("<I", struct.pack("<f", float(value)))[0]


def row_key(contact):
    return (bits(contact["dist"]),) + tuple(bits(v) for v in contact["pos"]) + tuple(
        bits(v) for v in contact["frame"]
    )


def ulp_diff(a, b):
    return abs(bits(a) - bits(b))


def load(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def upstream_worlds(scene):
    fields = scene["output"]["fields"]
    worlds = {}
    for index in range(scene["output"]["count"]):
        world = int(fields["worldid"][index][0])
        worlds.setdefault(world, []).append({
            "dist": fields["dist"][index][0],
            "pos": fields["pos"][index],
            "frame": fields["frame"][index],
            "geomcollisionid": int(fields["geomcollisionid"][index][0]),
        })
    return worlds


def compare_rows(name, left, right, label_left, label_right, failures):
    """逐位比较两条接触列表（先按位型多重集排序）。"""
    left_sorted = sorted(left, key=row_key)
    right_sorted = sorted(right, key=row_key)
    result = {
        "count": [len(left_sorted), len(right_sorted)],
        "exact": [row_key(c) for c in left_sorted] == [row_key(c) for c in right_sorted],
        "dist_max_ulp": 0,
        "pos_max_ulp": 0,
        "frame_max_abs": 0.0,
        "dist_bit_equal": 0,
        "pos_bit_equal": 0,
        "frame_bit_equal": 0,
    }
    if len(left_sorted) != len(right_sorted):
        failures.append(
            f"{name}: 接触数 {label_left}={len(left_sorted)} != {label_right}={len(right_sorted)}"
        )
    for a, b in zip(left_sorted, right_sorted):
        if bits(a["dist"]) == bits(b["dist"]):
            result["dist_bit_equal"] += 1
        result["dist_max_ulp"] = max(result["dist_max_ulp"], ulp_diff(a["dist"], b["dist"]))
        pos_equal = [bits(x) == bits(y) for x, y in zip(a["pos"], b["pos"])]
        result["pos_bit_equal"] += sum(pos_equal) // 3 if all(pos_equal) else 0
        for x, y in zip(a["pos"], b["pos"]):
            result["pos_max_ulp"] = max(result["pos_max_ulp"], ulp_diff(x, y))
        frame_equal = [bits(x) == bits(y) for x, y in zip(a["frame"], b["frame"])]
        result["frame_bit_equal"] += sum(frame_equal)
        for x, y in zip(a["frame"], b["frame"]):
            result["frame_max_abs"] = max(result["frame_max_abs"], abs(x - y))
    if not result["exact"]:
        failures.append(
            f"{name}: {label_left} vs {label_right} 非逐位一致 "
            f"(count {len(left_sorted)}/{len(right_sorted)}, "
            f"dist max {result['dist_max_ulp']} ULP, pos max {result['pos_max_ulp']} ULP, "
            f"frame max {result['frame_max_abs']:.3e})"
        )
    return result


def compare_product_like(reference, candidate, label, failures, out_rows):
    """候选侧（oracle 或产品）与上游逐场景逐 world 比对。"""
    ref_scenes = {scene["name"]: scene for scene in reference["scenes"]}
    summary = {"scenes": 0, "worlds": 0, "contacts": 0, "exact_scenes": 0, "failures": 0}
    for scene in candidate["scenes"]:
        name = scene["name"]
        ref_scene = ref_scenes[name]
        upstream = upstream_worlds(ref_scene)
        scene_ok = True
        for world_out in scene["worlds"]:
            world = int(world_out["world"])
            expected = upstream.get(world, [])
            actual = world_out["contacts"]
            label_name = f"{label}/{name}/world{world}"
            result = compare_rows(label_name, actual, expected, label, "upstream", failures)
            summary["worlds"] += 1
            summary["contacts"] += len(actual)
            if not result["exact"]:
                scene_ok = False
                out_rows.append({
                    "scene": name, "world": world, "comparison": label_name, **result,
                })
            # 重复点/去重统计（上游接触的多重集重数）
            upstream_keys = [row_key(c) for c in expected]
            out_rows_dedup = {
                "upstream_count": len(upstream_keys),
                "upstream_distinct": len(set(upstream_keys)),
            }
            if "cached_count" in world_out:
                out_rows_dedup.update({
                    "cached_count": world_out["cached_count"],
                    "cached_distinct_pos": world_out["cached_distinct_pos"],
                    "cached_distinct_full": world_out["cached_distinct_full"],
                    "prism_count": world_out["prism_count"],
                    "filter_accept": world_out["filter_accept"],
                    "candidate_cells": world_out["candidate_cells"],
                    "height_skipped": world_out["height_skipped"],
                    "noncontact": world_out["noncontact"],
                    "overflow": world_out["overflow"],
                })
            out_rows.append({"scene": name, "world": world, "dedup_stats": out_rows_dedup})
        summary["scenes"] += 1
        if scene_ok:
            summary["exact_scenes"] += 1
    summary["failures"] = len(failures)
    return summary


def compare_oracle_vs_product(oracle, product, failures, rows):
    oracle_scenes = {scene["name"]: scene for scene in oracle["scenes"]}
    summary = {"scenes": 0, "worlds": 0, "bounds_equal": 0, "bounds_total": 0,
               "prism_equal": 0, "prism_total": 0, "overflow_equal": 0, "overflow_total": 0}
    for scene in product["scenes"]:
        name = scene["name"]
        oracle_scene = oracle_scenes[name]
        for world_out in scene["worlds"]:
            world = int(world_out["world"])
            oracle_world = next(w for w in oracle_scene["worlds"] if int(w["world"]) == world)
            label = f"product-vs-oracle/{name}/world{world}"
            compare_rows(label, world_out["contacts"], oracle_world["contacts"],
                         "product", "oracle", failures)
            summary["worlds"] += 1
            summary["bounds_total"] += 1
            summary["prism_total"] += 1
            summary["overflow_total"] += 1
            if world_out.get("bounds") == oracle_world.get("bounds"):
                summary["bounds_equal"] += 1
            else:
                failures.append(f"{label}: filter bounds 不一致 {world_out.get('bounds')} vs {oracle_world.get('bounds')}")
            if world_out.get("prism_count") == oracle_world.get("prism_count"):
                summary["prism_equal"] += 1
            else:
                failures.append(
                    f"{label}: prism_count {world_out.get('prism_count')} vs {oracle_world.get('prism_count')}")
            if bool(world_out.get("overflow")) == bool(oracle_world.get("overflow")):
                summary["overflow_equal"] += 1
            else:
                failures.append(f"{label}: overflow {world_out.get('overflow')} vs {oracle_world.get('overflow')}")
        summary["scenes"] += 1
    return summary


def compare_delivered_fixtures(cases, failures):
    """3 个交付 case：QA 重采 vs 交付 fixture 逐位。"""
    result = {}
    for scene in cases["scenes"]:
        fixture_path = scene.get("delivered_fixture")
        if not fixture_path:
            continue
        delivered = load(ROOT / fixture_path)
        recaptured = upstream_worlds(scene)
        delivered_rows = []
        fields = delivered["case"]["output"]["fields"]
        for index in range(delivered["case"]["output"]["count"]):
            delivered_rows.append({
                "dist": fields["dist"][index][0],
                "pos": fields["pos"][index],
                "frame": fields["frame"][index],
                "geomcollisionid": int(fields["geomcollisionid"][index][0]),
            })
        all_rows = [row for rows in recaptured.values() for row in rows]
        check = compare_rows(f"recapture/{scene['name']}", all_rows, delivered_rows,
                             "qa-recapture", "delivered-fixture", failures)
        result[scene["name"]] = {
            **check,
            "fixture_count": delivered["case"]["output"]["count"],
            "qa_count": len(all_rows),
            "fixture_two_captures": delivered["case"]["two_captures_bitwise_identical"],
            "qa_two_captures": scene["two_captures_bitwise_identical"],
            "fixture_device": delivered["provenance"]["device"],
            "fixture_versions": delivered["provenance"]["versions"],
        }
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--oracle", default=str(EVIDENCE / "qa-oracle-output.json"))
    parser.add_argument("--product", default=str(EVIDENCE / "qa-product-output.json"))
    parser.add_argument("--out", default=str(EVIDENCE / "qa-compare-report.json"))
    args = parser.parse_args()

    cases = load(CASES)
    oracle_path = Path(args.oracle)
    product_path = Path(args.product)
    failures = []

    report = {
        "schema": 1,
        "task_id": "2026-10-25-g10-p5-hfield-host-independent-check",
        "sources": {
            "cases": str(CASES),
            "oracle": str(oracle_path) if oracle_path.exists() else None,
            "product": str(product_path) if product_path.exists() else None,
        },
    }

    report["delivered_fixture_recapture"] = compare_delivered_fixtures(cases, failures)
    print("== 交付 fixture provenance ==")
    for name, value in report["delivered_fixture_recapture"].items():
        print(f"  {name:<22} count {value['qa_count']}/{value['fixture_count']} exact={value['exact']} "
              f"dist {value['dist_bit_equal']}/{value['fixture_count']} pos {value['pos_bit_equal']}/"
              f"{value['fixture_count']} frame {value['frame_bit_equal']}/{value['fixture_count'] * 9} "
              f"two_captures qa={value['qa_two_captures']} delivered={value['fixture_two_captures']}")

    if oracle_path.exists():
        oracle = load(oracle_path)
        rows = []
        report["oracle_vs_upstream"] = compare_product_like(cases, oracle, "oracle", failures, rows)
        report["oracle_rows"] = rows
        print("== oracle vs upstream（同配方 f32） ==")
        print(f"  scenes {report['oracle_vs_upstream']['exact_scenes']}/"
              f"{report['oracle_vs_upstream']['scenes']} 逐位一致，worlds "
              f"{report['oracle_vs_upstream']['worlds']}，contacts "
              f"{report['oracle_vs_upstream']['contacts']}")
    else:
        report["oracle_vs_upstream"] = None

    if product_path.exists():
        product = load(product_path)
        rows = []
        report["product_vs_upstream"] = compare_product_like(cases, product, "product", failures, rows)
        report["product_rows"] = rows
        report["product_vs_oracle"] = compare_oracle_vs_product(oracle, product, failures, [])
        print("== product vs upstream ==")
        print(f"  scenes {report['product_vs_upstream']['exact_scenes']}/"
              f"{report['product_vs_upstream']['scenes']} 逐位一致，worlds "
              f"{report['product_vs_upstream']['worlds']}，contacts "
              f"{report['product_vs_upstream']['contacts']}")
        print("== product vs oracle ==")
        print(f"  {report['product_vs_oracle']}")
    else:
        report["product_vs_upstream"] = None
        report["product_vs_oracle"] = None

    report["failures"] = failures
    report["blocking"] = len(failures)
    report["summary"] = build_summary(cases, report)
    Path(args.out).write_text(
        json.dumps(report, ensure_ascii=False, indent=1), encoding="utf-8")
    print(f"failures: {len(failures)}  -> {args.out}")
    for failure in failures[:20]:
        print("  FAIL", failure)


def build_summary(cases, report):
    """汇总证据用的关键表：provenance、逐场景结论、重复点结构、溢出/过滤/边界专项。"""
    summary = {}
    recapture = report["delivered_fixture_recapture"]
    summary["provenance"] = {
        "delivered_cases_recaptured_bitwise": {
            name: {
                "exact": value["exact"],
                "count": value["count"],
                "fixture_two_captures": value["fixture_two_captures"],
                "qa_two_captures": value["qa_two_captures"],
                "fixture_device": value["fixture_device"],
                "fixture_versions": value["fixture_versions"],
            }
            for name, value in recapture.items()
        },
        "all_exact": all(value["exact"] for value in recapture.values()),
    }

    # 逐场景：产品 vs 上游（逐位）、产品 vs oracle（逐位）、上游重复点结构。
    scene_rows = []
    oracle_scenes = {scene["name"]: scene for scene in load(EVIDENCE / "qa-oracle-output.json")["scenes"]} \
        if (EVIDENCE / "qa-oracle-output.json").exists() else {}
    product_path = EVIDENCE / "qa-product-output.json"
    product_scenes = {scene["name"]: scene for scene in load(product_path)["scenes"]} \
        if product_path.exists() else {}
    for scene in cases["scenes"]:
        name = scene["name"]
        fields = scene["output"]["fields"]
        upstream_rows = []
        geomcollisionid_ok = True
        per_world = {}
        for index in range(scene["output"]["count"]):
            world = int(fields["worldid"][index][0])
            contact = {
                "dist": fields["dist"][index][0],
                "pos": fields["pos"][index],
                "frame": fields["frame"][index],
            }
            upstream_rows.append(contact)
            per_world.setdefault(world, []).append(contact)
        for world, rows in per_world.items():
            ids = [int(fields["geomcollisionid"][index][0])
                   for index in range(scene["output"]["count"])
                   if int(fields["worldid"][index][0]) == world]
            geomcollisionid_ok = geomcollisionid_ok and sorted(ids) == list(range(len(rows)))
        keys = [row_key(contact) for contact in upstream_rows]
        row = {
            "scene": name,
            "upstream_count": len(upstream_rows),
            "upstream_distinct_full": len(set(keys)),
            "duplicate_rows": len(keys) - len(set(keys)),
            "upstream_overflow": scene["output"]["overflow"],
            "two_captures_bitwise_identical": scene["two_captures_bitwise_identical"],
            "upstream_geomcollisionid_is_ordinal": geomcollisionid_ok,
        }
        if name in oracle_scenes:
            oracle_world = oracle_scenes[name]["worlds"][0]
            row.update({
                "oracle_filter_accept": oracle_world["filter_accept"],
                "oracle_prism_count": oracle_world["prism_count"],
                "oracle_candidate_cells": oracle_world["candidate_cells"],
                "oracle_height_skipped": sum(w["height_skipped"] for w in oracle_scenes[name]["worlds"]),
                "oracle_cached_count": sum(w["cached_count"] for w in oracle_scenes[name]["worlds"]),
                "oracle_cached_distinct_full": sum(w["cached_distinct_full"] for w in oracle_scenes[name]["worlds"]),
                "oracle_overflow": any(w["overflow"] for w in oracle_scenes[name]["worlds"]),
            })
        if name in product_scenes:
            product_world = product_scenes[name]["worlds"][0]
            row.update({
                "product_filter_accept": product_world["filter_accept"],
                "product_prism_count": product_world["prism_count"],
                "product_overflow": any(w["overflow"] for w in product_scenes[name]["worlds"]),
            })
        scene_rows.append(row)
    summary["scenes"] = scene_rows

    # 专项：溢出、filter 拒绝、边界 cell、旋转、六类配对。
    def pick(prefix):
        return [row for row in scene_rows if row["scene"].startswith(prefix)]

    summary["overflow"] = pick("syn-overflow")
    summary["filter_reject"] = pick("syn-outside") + pick("syn-above") + pick("syn-below") + pick("syn-gap")
    summary["boundary_cells"] = pick("syn-flat") + pick("syn-trunc")
    summary["tangential"] = pick("syn-side")
    summary["rotated"] = pick("syn-rot")
    summary["geom_classes"] = pick("syn-class")
    summary["dedup"] = [row for row in scene_rows
                        if row["scene"].startswith("deliv") or row["scene"].startswith("syn-dedup")]
    summary["product_vs_oracle"] = report.get("product_vs_oracle")
    summary["product_vs_upstream"] = report.get("product_vs_upstream")
    summary["oracle_vs_upstream"] = report.get("oracle_vs_upstream")
    if product_path.exists():
        summary["helper_rotation_check"] = load(product_path).get("helper_rotation_check")
    return summary


if __name__ == "__main__":
    main()
