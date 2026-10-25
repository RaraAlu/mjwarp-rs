# -*- coding: utf-8 -*-
"""QA independent enumeration for 2026-10-25-t4-source-hash-repin-13-r2.

Does not import or reuse the delivery scripts/drift.json values as input.
Algorithm replicated from the Rust test sources (read, not copied from delivery):
  - single file            : sha256(LF(file bytes))
  - src/physics/mod.rs     : sha256(LF(concat(anchor include_str! list, joined with "\n")))
  - resident combined      : same rule on docs/evidence/t4-g03-retest/resident-source-anchor.rs
  - transfer combined      : sha256(LF(concat(transfer.rs, "\n", buffer.rs, "\n", session.rs)))
  - gravity package sha256 : sha256(raw package file bytes)  (include_str! in test)
  - gravity_compensation   : sha256(raw file bytes)          (fs::read in test)
  - inventory_sha256       : sha256(raw fixture bytes)       (claim under test)
"""
import hashlib
import json
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
OUT = REPO / "docs/development-evidence/t4-source-hash-repin-13-independent-check"
OUT.mkdir(parents=True, exist_ok=True)
HEAD = subprocess.run(["git", "rev-parse", "HEAD"], cwd=REPO, capture_output=True, text=True).stdout.strip()
PARENT = "61fb6baf~1"
CUR_FIXTURE = "fixtures/precision/t4-current-kernels.json"
DELIVERY_README = "docs/development-evidence/t4-source-hash-repin-13/README.md"
DELIVERY_RECEIPT = "docs/development-evidence/t4-source-hash-repin-13/receipt.json"


def sha(b: bytes) -> str:
    return hashlib.sha256(b).hexdigest()


def lf(b: bytes) -> bytes:
    return b.replace(b"\r\n", b"\n")


def read_bytes(rel: str) -> bytes:
    return (REPO / rel).read_bytes()


def git_show(rev: str, rel: str) -> bytes:
    p = subprocess.run(["git", "show", f"{rev}:{rel}"], cwd=REPO, capture_output=True)
    if p.returncode != 0:
        raise RuntimeError(f"git show {rev}:{rel} failed: {p.stderr[:200]}")
    return p.stdout


def anchor_files(anchor_rel: str):
    """Parse include_str! targets from a concat! anchor file (independent re-read)."""
    text = read_bytes(anchor_rel).decode("utf-8")
    rels = re.findall(r'include_str!\("([^"]+)"\)', text)
    base = (REPO / anchor_rel).parent
    files = [(base / r).resolve() for r in rels]
    return text, rels, files


def combined_digest(files) -> tuple:
    joined = b"\n".join(f.read_bytes() for f in files)
    joined = lf(joined)
    return sha(joined), len(joined)


def concat_files(rels) -> bytes:
    return lf(b"\n".join(read_bytes(r) for r in rels))


result = {
    "task": "2026-10-25-t4-source-hash-repin-13-independent-check",
    "subject_task": "2026-10-25-t4-source-hash-repin-13-r2",
    "head": HEAD,
    "parent_rev": PARENT,
    "generated_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "algorithm": {
        "single_file": "sha256(LF(file bytes))",
        "physics_mod_rs": "sha256(LF(concat(include_str! list of t4-resident-anchors/physics.rs, joined with \\n)))",
        "resident_combined": "sha256(LF(concat(include_str! list of docs/evidence/t4-g03-retest/resident-source-anchor.rs, joined with \\n)))",
        "transfer_combined": "sha256(LF(concat(transfer.rs, \\n, transfer/buffer.rs, \\n, transfer/session.rs)))",
        "gravity_package_sha256": "sha256(raw package file bytes)",
        "gravity_compensation": "sha256(raw file bytes)",
        "inventory_sha256": "sha256(raw fixture bytes) per delivery claim; both raw and LF variants reported",
    },
}

# ---- 1) 20 product_sources -------------------------------------------------
cur = json.loads(read_bytes(CUR_FIXTURE))
parent = json.loads(git_show(PARENT, CUR_FIXTURE))
anchor_text, anchor_rels, anchor_paths = anchor_files("docs/development-evidence/t4-resident-anchors/physics.rs")
mod_digest, mod_len = combined_digest(anchor_paths)
result["physics_core_anchor"] = {
    "anchor_file": "docs/development-evidence/t4-resident-anchors/physics.rs",
    "include_str_list": anchor_rels,
    "concat_len_bytes_lf": mod_len,
    "computed_combined": mod_digest,
}

rows = []
for stamp in cur["product_sources"]:
    path = stamp["path"]
    if path == "src/physics/mod.rs":
        computed = mod_digest
        how = "physics_core combined digest"
    else:
        computed = sha(lf(read_bytes(path)))
        how = "single file LF"
    old = next((s["sha256"] for s in parent["product_sources"] if s["path"] == path), None)
    rows.append({
        "path": path,
        "fixture_sha256": stamp["sha256"],
        "computed_sha256": computed,
        "match": computed == stamp["sha256"],
        "parent_fixture_sha256": old,
        "changed_vs_parent": old != stamp["sha256"],
        "how": how,
    })
result["product_sources"] = {
    "total": len(rows),
    "match": sum(1 for r in rows if r["match"]),
    "mismatch": [r for r in rows if not r["match"]],
    "rows": rows,
}

# ---- 2) drift set vs parent fixture ---------------------------------------
drift_current_vs_parent = [r for r in rows if r["changed_vs_parent"]]
drift_expected = [
    "src/physics/attached.rs", "src/physics/com_velocity.rs", "src/physics/mass_matrix.rs",
    "src/physics/mass_solve.rs", "src/physics/mod.rs", "src/physics/resident.rs",
    "src/physics/rne_bias.rs", "src/runtime/transfer.rs", "src/runtime/transfer/kernel.rs",
]
result["drift"] = {
    "count_vs_parent_fixture": len(drift_current_vs_parent),
    "paths": [r["path"] for r in drift_current_vs_parent],
    "expected_paths": drift_expected,
    "path_set_equal": sorted(r["path"] for r in drift_current_vs_parent) == sorted(drift_expected),
    "rows": [{
        "path": r["path"],
        "old_parent_fixture": r["parent_fixture_sha256"],
        "new_fixture": r["fixture_sha256"],
        "computed_live": r["computed_sha256"],
        "new_fixture_equals_live": r["match"],
    } for r in drift_current_vs_parent],
}

# ---- 3) compare with delivery tables (§1.1 + receipt rows) ----------------
delivery_rows = json.loads(read_bytes(DELIVERY_RECEIPT))["identity"]["t4_current_kernels_product_sources"]["rows"]
delivery_by_path = {d["path"]: d for d in delivery_rows}
cur_by_path = {r["path"]: r for r in rows}
cmp_rows = []
for path in drift_expected:
    d = delivery_by_path.get(path, {})
    c = cur_by_path[path]
    cmp_rows.append({
        "path": path,
        "delivery_old": d.get("old"),
        "delivery_new": d.get("new"),
        "delivery_old_equals_parent_fixture": d.get("old") == c["parent_fixture_sha256"],
        "delivery_new_equals_computed": d.get("new") == c["computed_sha256"],
        "delivery_new_equals_fixture": d.get("new") == c["fixture_sha256"],
    })
result["delivery_table_compare"] = {
    "rows": cmp_rows,
    "all_old_match_parent": all(r["delivery_old_equals_parent_fixture"] for r in cmp_rows),
    "all_new_match_computed": all(r["delivery_new_equals_computed"] for r in cmp_rows),
    "delivery_row_count": len(delivery_rows),
    "drift_count_claimed": json.loads(read_bytes(DELIVERY_RECEIPT))["identity"]["t4_current_kernels_product_sources"]["drift"],
}

# ---- 4) physics_core three objects ---------------------------------------
anchor_sha_raw = sha(read_bytes("docs/development-evidence/t4-resident-anchors/physics.rs"))
anchor_sha_lf = sha(lf(read_bytes("docs/development-evidence/t4-resident-anchors/physics.rs")))
live_mod_single_lf = sha(lf(read_bytes("src/physics/mod.rs")))
result["physics_core_three_objects"] = {
    "recorded_in_fixture": cur_by_path["src/physics/mod.rs"]["fixture_sha256"],
    "computed_combined": mod_digest,
    "combined_matches_fixture": mod_digest == cur_by_path["src/physics/mod.rs"]["fixture_sha256"],
    "anchor_file_sha256_raw": anchor_sha_raw,
    "anchor_file_sha256_lf": anchor_sha_lf,
    "live_mod_rs_single_file_lf": live_mod_single_lf,
    "distinct_three": len({mod_digest, anchor_sha_raw, anchor_sha_lf, live_mod_single_lf}),
}

# ---- 5) cross checks: other combos, native inputs, gravity packages -------
cross = {}

# 5a transfer combined
transfer_rels = ["src/runtime/transfer.rs", "src/runtime/transfer/buffer.rs", "src/runtime/transfer/session.rs"]
transfer_digest = sha(concat_files(transfer_rels))
cross["transfer_combined"] = {"files": transfer_rels, "computed": transfer_digest}

# 5b resident combined
_, resident_rels, resident_paths = anchor_files("docs/evidence/t4-g03-retest/resident-source-anchor.rs")
resident_digest, resident_len = combined_digest(resident_paths)
cross["resident_combined"] = {
    "anchor_file": "docs/evidence/t4-g03-retest/resident-source-anchor.rs",
    "file_count": len(resident_rels),
    "concat_len_bytes_lf": resident_len,
    "computed": resident_digest,
}

# 5c g01-inputs product_sources (18 entries, incl. 3 native files)
g01 = json.loads(read_bytes("fixtures/precision/g01-inputs.json"))
g01_rows = []
for stamp in g01["product_sources"]:
    computed = sha(lf(read_bytes(stamp["path"])))
    g01_rows.append({"path": stamp["path"], "fixture": stamp["sha256"], "computed": computed, "match": computed == stamp["sha256"]})
cross["g01_inputs_product_sources"] = {
    "total": len(g01_rows),
    "match": sum(1 for r in g01_rows if r["match"]),
    "mismatch": [r for r in g01_rows if not r["match"]],
}

# 5d gravity packages: package sha256 (raw bytes) + product_sources fields
gcomp_raw = sha(read_bytes("src/physics/gravity_compensation.rs"))
gpackages = {}
for rel, test_const in [
    ("fixtures/gravity-upstream-abi/three-d-package.json", "708ced7c46624b86f10a46dff45a66d8ffdfe289cb989b63920d55f3116dc795"),
    ("fixtures/gravity-upstream-abi/nonzero-product-chain-package.json", "96bd92afe05b482630cff7ffcf3eda9626a29e8b9bff01bcf70f17bc2f0af79b"),
]:
    raw = read_bytes(rel)
    pkg = json.loads(raw)
    ps = pkg.get("product_sources", {})
    gpackages[rel] = {
        "package_sha256_raw": sha(raw),
        "package_sha256_lf": sha(lf(raw)),
        "test_const": test_const,
        "test_const_matches_raw": sha(raw) == test_const,
        "test_const_matches_lf": sha(lf(raw)) == test_const,
        "product_sources": ps,
        "physics_core_matches_computed": ps.get("physics_core", {}).get("sha256") == mod_digest,
        "resident_matches_computed": ps.get("resident", {}).get("sha256") == resident_digest,
        "gravity_compensation_matches_raw": ps.get("gravity_compensation", {}).get("sha256") == gcomp_raw,
    }
cross["gravity_packages"] = gpackages

# 5e inventory_sha256 variants
status = json.loads(read_bytes("fixtures/precision/t4-current-capture-status.json"))
cur_raw = read_bytes(CUR_FIXTURE)
parent_blob = git_show(PARENT, CUR_FIXTURE)
cross["inventory_sha256"] = {
    "recorded_new": status["inventory_sha256"],
    "recorded_previous": status["previous_inventory_sha256"],
    "computed_current_worktree_raw": sha(cur_raw),
    "computed_current_worktree_lf": sha(lf(cur_raw)),
    "computed_parent_blob_raw": sha(parent_blob),
    "new_matches_worktree_raw": sha(cur_raw) == status["inventory_sha256"],
    "new_matches_worktree_lf": sha(lf(cur_raw)) == status["inventory_sha256"],
    "previous_matches_parent_blob_raw": sha(parent_blob) == status["previous_inventory_sha256"],
    "worktree_has_crlf": b"\r\n" in cur_raw,
}
result["cross_checks"] = cross

# ---- 6) stale hash scan (old values must not linger in tracked content) ---
def git_grep(args):
    p = subprocess.run(["git", "grep"] + args, cwd=REPO, capture_output=True)
    return p.stdout.decode("utf-8", errors="replace").strip().splitlines()


scan = {}
DELIVERY_DIR = "docs/development-evidence/t4-source-hash-repin-13/"


def hit_path(line):
    if line.startswith("Binary file "):
        return line[len("Binary file "):].rsplit(" matches", 1)[0]
    return line.split(":", 1)[0]


def categorize(lines):
    cats = {"delivery_evidence": [], "historical_evidence": [], "other": []}
    for h in lines:
        p = hit_path(h).replace("\\", "/")
        if p.startswith(DELIVERY_DIR):
            cats["delivery_evidence"].append(p)
        elif p.startswith("docs/development-evidence/") or p.startswith("docs/evidence/"):
            cats["historical_evidence"].append(p)
        else:
            cats["other"].append(p)
    return cats


for r in delivery_rows:
    old, new = r["old"], r["new"]
    old_hits = [h for h in git_grep(["-n", "-F", old]) if h]
    new_hits = [h for h in git_grep(["-l", "-F", new]) if h]
    cats = categorize(old_hits)
    scan[r["path"]] = {
        "old_occurrences_total": len(old_hits),
        "old_occurrences_by_file": {
            "delivery_evidence": sorted(set(cats["delivery_evidence"])),
            "historical_evidence_unique_files": len(set(cats["historical_evidence"])),
            "historical_evidence_samples": sorted(set(cats["historical_evidence"]))[:6],
            "other_files": sorted(set(cats["other"])),
        },
        "new_occurrence_files": sorted(set(new_hits)),
    }
result["stale_hash_scan"] = scan

# ---- verdict --------------------------------------------------------------
checks = {
    "product_sources_20_of_20": result["product_sources"]["match"] == 20,
    "drift_set_is_9_expected_paths": result["drift"]["count_vs_parent_fixture"] == 9 and result["drift"]["path_set_equal"],
    "delivery_old_values_match_parent_fixture": result["delivery_table_compare"]["all_old_match_parent"],
    "delivery_new_values_match_independent_compute": result["delivery_table_compare"]["all_new_match_computed"],
    "physics_core_combined_matches_fixture": result["physics_core_three_objects"]["combined_matches_fixture"],
    "g01_inputs_18_of_18": cross["g01_inputs_product_sources"]["match"] == cross["g01_inputs_product_sources"]["total"],
}
result["verdict"] = {"checks": checks, "all_pass": all(checks.values())}

def render(o, indent=0):
    """Compact JSON: object rows stay one line each (keeps evidence <=500 lines)."""
    sp = "  " * indent
    sp2 = "  " * (indent + 1)
    if isinstance(o, dict):
        if not o:
            return "{}"
        parts = [f"{sp2}{json.dumps(k, ensure_ascii=False)}: {render(v, indent + 1)}" for k, v in o.items()]
        return "{\n" + ",\n".join(parts) + "\n" + sp + "}"
    if isinstance(o, list):
        if not o:
            return "[]"
        if all(not isinstance(x, (dict, list)) for x in o):
            return json.dumps(o, ensure_ascii=False)
        parts = []
        for x in o:
            if isinstance(x, dict):
                parts.append(sp2 + json.dumps(x, ensure_ascii=False))
            else:
                parts.append(sp2 + render(x, indent + 1))
        return "[\n" + ",\n".join(parts) + "\n" + sp + "]"
    return json.dumps(o, ensure_ascii=False)


(OUT / "qa-identity-enum.json").write_text(render(result) + "\n", encoding="utf-8")

# compact log
lines = [
    f"head={HEAD} parent={PARENT}",
    f"product_sources match {result['product_sources']['match']}/{result['product_sources']['total']}",
    f"drift count={result['drift']['count_vs_parent_fixture']} path_set_equal={result['drift']['path_set_equal']}",
    f"delivery old match={result['delivery_table_compare']['all_old_match_parent']} new match={result['delivery_table_compare']['all_new_match_computed']}",
    f"physics_core combined={mod_digest}",
    f"anchor raw={anchor_sha_raw} lf={anchor_sha_lf} live_mod_single={live_mod_single_lf}",
    f"transfer combined={transfer_digest}",
    f"resident combined={resident_digest}",
    f"g01-inputs {cross['g01_inputs_product_sources']['match']}/{cross['g01_inputs_product_sources']['total']}",
    f"inventory new_matches_raw={cross['inventory_sha256']['new_matches_worktree_raw']} new_matches_lf={cross['inventory_sha256']['new_matches_worktree_lf']} prev_matches_parent_blob={cross['inventory_sha256']['previous_matches_parent_blob_raw']}",
]
for rel, g in gpackages.items():
    lines.append(f"{rel} pkg_raw={g['package_sha256_raw']} test_const_raw_ok={g['test_const_matches_raw']} physics_ok={g['physics_core_matches_computed']} resident_ok={g['resident_matches_computed']} gravity_ok={g['gravity_compensation_matches_raw']}")
lines.append(f"verdict all_pass={result['verdict']['all_pass']}")
(OUT / "qa-identity-enum.log").write_text("\n".join(lines) + "\n", encoding="utf-8")
print("\n".join(lines))
