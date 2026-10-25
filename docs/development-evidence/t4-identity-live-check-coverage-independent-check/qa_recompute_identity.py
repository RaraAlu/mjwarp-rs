# -*- coding: utf-8 -*-
"""Independent recomputation of the T4 identity fixture (QA check; self-written).

Does NOT import or run tests/support/t4_identity_live.rs. Recomputes:
  - each of the 20 `product_sources` entries with sha256(LF-normalized bytes);
  - `src/physics/mod.rs` with the physics_core composite convention: parse the
    include_str! list from the anchor docs/development-evidence/t4-resident-anchors/physics.rs
    (NOT hardcoded), read each member, LF-normalize, join with b'\\n', sha256.
Also cross-checks: entry count, duplicate paths, every path exists, and the path
set against the delivery's EXPECTED_PATHS constant (parsed textually; reported,
not trusted). Writes qa-identity-recompute.json.
"""
import hashlib
import json
import re
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
TASK = REPO / "docs/development-evidence/t4-identity-live-check-coverage-independent-check"
FIXTURE = REPO / "fixtures/precision/t4-current-kernels.json"
ANCHOR = REPO / "docs/development-evidence/t4-resident-anchors/physics.rs"
SUPPORT = REPO / "tests/support/t4_identity_live.rs"


def normalize_lf(data: bytes) -> bytes:
    return data.replace(b"\r\n", b"\n")


def sha256_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def live_digest(path: str) -> dict:
    full = REPO / path
    raw = full.read_bytes()
    normalized = normalize_lf(raw)
    return {
        "path": path,
        "bytes_raw": len(raw),
        "bytes_lf": len(normalized),
        "sha256_lf": sha256_hex(normalized),
        "had_crlf": len(normalized) != len(raw),
    }


def physics_core() -> dict:
    anchor_text = ANCHOR.read_text(encoding="utf-8")
    members = re.findall(r'include_str!\("([^"]+)"\)', anchor_text)
    anchor_dir = ANCHOR.parent
    parts = []
    member_rows = []
    for member in members:
        full = (anchor_dir / member).resolve()
        raw = full.read_bytes()
        normalized = normalize_lf(raw)
        parts.append(normalized)
        member_rows.append({
            "include_str": member,
            "resolved_relative_to_repo": str(full.relative_to(REPO)).replace("\\", "/"),
            "bytes_raw": len(raw),
            "bytes_lf": len(normalized),
            "sha256_lf": sha256_hex(normalized),
        })
    combined = b"\n".join(parts)
    return {
        "convention": "anchor include_str! members (in order), each LF-normalized, joined with b'\\n', sha256",
        "anchor": str(ANCHOR.relative_to(REPO)).replace("\\", "/"),
        "anchor_include_str_order": members,
        "members": member_rows,
        "combined_bytes": len(combined),
        "sha256_lf": sha256_hex(combined),
    }


def expected_paths_from_support() -> list:
    text = SUPPORT.read_text(encoding="utf-8")
    block = re.search(r"EXPECTED_PATHS: \[&str; \d+\] = \[(.*?)\];", text, re.S)
    if not block:
        return []
    return re.findall(r'"([^"]+)"', block.group(1))


fixture = json.loads(FIXTURE.read_text(encoding="utf-8"))
entries = fixture["product_sources"]
composite = physics_core()

rows = []
mismatches = []
for entry in entries:
    path = entry["path"]
    expected = entry["sha256"]
    if path == "src/physics/mod.rs":
        actual = composite["sha256_lf"]
        kind = "physics_core_composite"
    else:
        info = live_digest(path)
        actual = info["sha256_lf"]
        kind = "single_file"
    ok = actual == expected
    rows.append({
        "path": path,
        "kind": kind,
        "expected": expected,
        "actual": actual,
        "match": ok,
    })
    if not ok:
        mismatches.append(f"{path}: expected={expected} actual={actual}")

paths = [entry["path"] for entry in entries]
duplicates = sorted({p for p in paths if paths.count(p) > 1})
missing_files = [p for p in paths if not (REPO / p).is_file()]
support_paths = expected_paths_from_support()

result = {
    "fixture": str(FIXTURE.relative_to(REPO)).replace("\\", "/"),
    "fixture_git_status": "(checked separately in qa-static-checks.json / logs)",
    "entry_count": len(entries),
    "duplicates": duplicates,
    "missing_files": missing_files,
    "physics_core": composite,
    "rows": rows,
    "mismatch_count": len(mismatches),
    "mismatches": mismatches,
    "path_set_cross_check": {
        "fixture_path_count": len(paths),
        "delivery_expected_paths_count": len(support_paths),
        "fixture_minus_delivery": sorted(set(paths) - set(support_paths)),
        "delivery_minus_fixture": sorted(set(support_paths) - set(paths)),
        "note": "the canonical path set is defined by the delivery's EXPECTED_PATHS constant; "
                "this cross-check is reported, not treated as an independent oracle. "
                "Collection completeness behaviour is verified by the tamper cases.",
    },
}
(TASK / "qa-identity-recompute.json").write_text(
    json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

print(f"entries={len(entries)} mismatches={len(mismatches)} duplicates={len(duplicates)} missing_files={len(missing_files)}")
print(f"physics_core sha256={composite['sha256_lf']} members={composite['anchor_include_str_order']}")
print(f"expected_paths_from_support={len(support_paths)}")
for row in rows:
    print(f"{'OK  ' if row['match'] else 'DIFF'} {row['path']} {row['kind']} expected={row['expected'][:16]} actual={row['actual'][:16]}")
