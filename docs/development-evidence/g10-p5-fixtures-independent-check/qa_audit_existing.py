"""QA independent G0b review: recompute the d04 existing-reference audit from the source case JSONs."""

import hashlib
import json
import sys
from pathlib import Path

import mujoco

ROOT = Path(r"D:\Rust\mjwarp-rs")
CASES = ROOT / "docs/evidence/d04-complex-physics/authoring-runs/run-5/generated/cases"
FIX = ROOT / "fixtures/g10-p5"
DELIVERED = FIX / "existing-reference-audit.json"
GEOM_TYPE_NAMES = {
    0: "plane", 1: "hfield", 2: "sphere", 3: "capsule", 4: "ellipsoid",
    5: "cylinder", 6: "box", 7: "mesh", 8: "sdf",
}


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def recompute(path):
    doc = json.loads(path.read_text(encoding="utf-8"))
    out = doc["output"]
    fields = out["fields"]
    count = out["count"]
    n_dist = len(fields["dist"]["values"])
    world = fields["worldid"]["values"]
    geom_flat = fields["geom"]["values"]
    pairs = [tuple(geom_flat[i:i + 2]) for i in range(0, len(geom_flat), 2)]
    dists = fields["dist"]["values"]
    per_world = {}
    for w in sorted(set(world)):
        per_world[str(w)] = world.count(w)
    return {
        "name": doc["name"],
        "worlds": doc["input"]["worlds"],
        "source_xml": doc["input"]["path"],
        "source_xml_sha256": doc["input"]["sha256"],
        "count": count,
        "dist_len_equals_count": n_dist == count,
        "worldid_len_equals_count": len(world) == count,
        "contacts_per_world": per_world,
        "geom_pairs": sorted(set(pairs)),
        "dist_min": round(min(dists), 6),
        "dist_max": round(max(dists), 6),
        "field_names": sorted(fields.keys()),
        "field_count": len(fields),
        "boundary_evidence": doc.get("boundary_evidence"),
        "sha256": sha256(path),
        "path": str(path.relative_to(ROOT)).replace("\\", "/"),
        "families": doc.get("families", []),
        "qpos_shape": (doc["input"].get("qpos") or {}).get("shape"),
    }


def geom_types(xml_rel):
    model = mujoco.MjModel.from_xml_path(str(ROOT / xml_rel))
    out = []
    for i in range(model.ngeom):
        out.append({
            "id": i,
            "name": mujoco.mj_id2name(model, mujoco.mjtObj.mjOBJ_GEOM, i),
            "type": GEOM_TYPE_NAMES.get(int(model.geom_type[i]), str(int(model.geom_type[i]))),
            "dataid": int(model.geom_dataid[i]),
        })
    return out


def main():
    delivered = json.loads(DELIVERED.read_text(encoding="utf-8"))
    cases = {}
    report = {"cases": {}, "mismatches": [], "delivered_meta": {
        "reusable_for_g2": delivered.get("reusable_for_g2"),
        "regression_only": delivered.get("regression_only"),
        "gaps": delivered.get("gaps"),
        "conclusion": delivered.get("conclusion"),
    }}
    for path in sorted(CASES.glob("collision-*.json")):
        mine = recompute(path)
        cases[mine["name"]] = mine
    # compare delivered per-case rows against my recomputation
    for name, delivered_row in delivered["cases"].items():
        mine = cases.get(name)
        if mine is None:
            report["mismatches"].append({"case": name, "issue": "delivered case missing from d04 dir"})
            continue
        row = {"type_checks": geom_types(mine["source_xml"])}
        for key in ["worlds", "count", "contacts_per_world", "geom_pairs", "dist_min",
                    "dist_max", "field_names", "field_count", "sha256", "path"]:
            got = mine[key]
            want = delivered_row.get(key)
            if key == "geom_pairs":
                got = [tuple(p) for p in got]
                want = [tuple(p) for p in (want or [])]
            row[key] = {"qa": got, "delivered": want, "match": got == want}
            if got != want:
                report["mismatches"].append({"case": name, "field": key, "qa": got, "delivered": want})
        report["cases"][name] = row
    # cases present in delivered audit must be exactly the d04 collision set
    extra = sorted(set(delivered["cases"]) - set(cases))
    missing = sorted(set(cases) - set(delivered["cases"]))
    report["case_set"] = {"delivered_extra": extra, "delivered_missing": missing,
                          "d04_count": len(cases), "delivered_count": len(delivered["cases"])}
    # reuse classification: verify claimed capabilities against actual geom types
    claims = {
        "box-box-multiccd": ("box+box", "support/EPA/multicontact"),
        "convex-mesh": ("mesh+box", "mesh support"),
        "degenerate-mesh": ("mesh+mesh", "degenerate path"),
        "hfield-box": ("hfield+box", "hfield prism CCD"),
        "sdf-cylinder": ("cylinder+sdf", "SDF narrowphase"),
        "sphere-sphere": ("sphere+sphere", "primitive analytic"),
    }
    reuse_check = {}
    for name, (want_pair, _cap) in claims.items():
        types = report["cases"].get(name, {}).get("type_checks")
        if types is None:
            continue
        pairs = cases[name]["geom_pairs"]
        resolved = []
        for pair in pairs:
            resolved.append([types[g]["type"] if 0 <= g < len(types) else f"bad({g})" for g in pair])
        reuse_check[name] = {"declared_geom_pair": want_pair, "resolved_types": resolved}
    report["reuse_capability_resolution"] = reuse_check
    # gap fill mapping: gaps mentioned in audit vs new capture names
    new_cases = sorted(p.stem for p in (FIX / "references").glob("*.json") if p.stem != "index")
    report["new_capture_cases"] = new_cases
    report["gap_fill_mapping"] = {
        "sphere-box": ["sphere-box-shallow", "sphere-box-deep"],
        "capsule-box": ["capsule-box"],
        "cylinder-box": ["cylinder-box"],
        "ellipsoid-box": ["ellipsoid-box"],
        "hfield-sphere / hfield-capsule": ["hfield-sphere", "hfield-capsule"],
        "sdf-box": ["sdf-box"],
        "非退化 mesh-mesh": ["mesh-mesh"],
        "box-box margin!=0": ["box-box-margin"],
        "SDF 距离/梯度越界探针": "not captured (out of G0b direct-reference scope; probe exists in d04 volume layer)",
    }
    report["no_existing_case_recaptured"] = sorted(set(new_cases) & set(cases)) == []
    report["overall"] = {
        "all_delivered_rows_match_qa": len(report["mismatches"]) == 0,
        "no_existing_case_recaptured": report["no_existing_case_recaptured"],
        "case_set_ok": extra == [] and missing == [],
    }
    out = ROOT / "target/qa-g0b/audit_qa_report.json"
    out.write_text(json.dumps(report, ensure_ascii=False, indent=1), encoding="utf-8")
    print("cases: d04=%d delivered=%d extra=%s missing=%s" % (
        len(cases), len(delivered["cases"]), extra, missing))
    for name, row in report["cases"].items():
        bad = [k for k, v in row.items() if isinstance(v, dict) and v.get("match") is False]
        print("%-22s count=%-3d per_world=%-12s pairs=%s %s" % (
            name, cases[name]["count"], cases[name]["contacts_per_world"],
            cases[name]["geom_pairs"], "MISMATCH:" + str(bad) if bad else "ok"))
    print("reuse:", json.dumps(reuse_check))
    print("new_cases:", new_cases, "no_existing_case_recaptured =", report["no_existing_case_recaptured"])
    print("mismatches:", report["mismatches"])
    print("wrote", out)


if __name__ == "__main__":
    sys.exit(main())
