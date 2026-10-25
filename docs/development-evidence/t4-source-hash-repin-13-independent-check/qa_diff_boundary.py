# -*- coding: utf-8 -*-
"""QA boundary check for 2026-10-25-t4-source-hash-repin-13-r2.

Independently re-derives the re-pin diff boundary: the content commit must only
change identity expectation literals; no product source, kernel/PTX, assertion
logic, tolerance, exclusion list, G01 script or unchanged-protection path.
"""
import json
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
OUT = REPO / "docs/development-evidence/t4-source-hash-repin-13-independent-check"
HEAD = subprocess.run(["git", "rev-parse", "HEAD"], cwd=REPO, capture_output=True, text=True).stdout.strip()
BASE = "61fb6baf~1"
HEAD_C = "61fb6baf"

FORBIDDEN_PREFIX = ["src/", "native/", "include/", "scripts/", ".cargo/", "build.rs", "Cargo.toml", "Cargo.lock"]
ALLOWED_PREFIX = [
    "fixtures/precision/",
    "fixtures/gravity-upstream-abi/",
    "docs/evidence/t4-g03-retest/",
    "tests/",
    "docs/development-evidence/t4-source-hash-repin-13/",
    ".normify/",
]
# current-identity consumer files that tests re-verify against live sources
CONTENT_EXPECTED = [
    "docs/evidence/t4-g03-retest/camlight-internal/map.json",
    "docs/evidence/t4-g03-retest/flex-internal/map.json",
    "docs/evidence/t4-g03-retest/flex-matrices/map.json",
    "docs/evidence/t4-g03-retest/rigid-rotate-recapture/previous-kernels.json",
    "docs/evidence/t4-g03-retest/tendon-wake-internal/map.json",
    "fixtures/gravity-upstream-abi/nonzero-product-chain-package.json",
    "fixtures/gravity-upstream-abi/three-d-package.json",
    "fixtures/precision/g01-field-internal.json",
    "fixtures/precision/g01-inputs.json",
    "fixtures/precision/g01-rigid-internal.json",
    "fixtures/precision/g01-state-packing.json",
    "fixtures/precision/g01-topology-frames.json",
    "fixtures/precision/t4-current-capture-status.json",
    "fixtures/precision/t4-current-kernels.json",
    "tests/support/t4_rigid_internal/tests.rs",
    "tests/t4_gravity_3d_product_chain.rs",
    "tests/t4_gravity_nonzero_product_chain.rs",
]
ALLOWED_BIN_KEYS = {
    "sha256", "old", "new", "previous", "frozen", "mid",
    "inventory_sha256", "previous_inventory_sha256", "package_sha256",
    "source_sha256", "ptx_sha256", "receipt_sha256", "physics_core",
    "resident", "gravity_compensation",
}


def git(*args) -> str:
    p = subprocess.run(["git"] + list(args), cwd=REPO, capture_output=True)
    return p.stdout.decode("utf-8", errors="replace")


def git_bytes(*args) -> bytes:
    p = subprocess.run(["git"] + list(args), cwd=REPO, capture_output=True)
    return p.stdout


def mask(line: str) -> str:
    line = re.sub(r"\b[0-9a-f]{64}\b", "<H64>", line)
    line = re.sub(r"\b[0-9a-f]{40}\b", "<H40>", line)
    return line


def numstat():
    out = {}
    for line in git("diff", "--numstat", f"{BASE}..{HEAD_C}").splitlines():
        parts = line.split("\t")
        if len(parts) == 3:
            out[parts[2]] = {"added": parts[0], "deleted": parts[1], "binary": parts[0] == "-"}
    return out


def text_line_diff(path: str):
    """Return paired (-,+) changed lines with masking verdict."""
    raw = git("diff", "-U0", f"{BASE}..{HEAD_C}", "--", path)
    removed, added = [], []
    for line in raw.splitlines():
        if line.startswith("---") or line.startswith("+++"):
            continue
        if line.startswith("-"):
            removed.append(line[1:])
        elif line.startswith("+"):
            added.append(line[1:])
    pairs = []
    for i in range(max(len(removed), len(added))):
        old = removed[i] if i < len(removed) else None
        new = added[i] if i < len(added) else None
        if old is None or new is None:
            pairs.append({"old": old, "new": new, "masked_equal": False, "reason": "unpaired line"})
            continue
        pairs.append({
            "old": old, "new": new,
            "masked_equal": mask(old) == mask(new),
            "has_hex": bool(re.search(r"\b[0-9a-f]{64}\b|\b[0-9a-f]{40}\b", old + new)),
        })
    return pairs


def json_field_diff(a, b, path=""):
    """Recursive structural diff; returns list of (json_path, old, new)."""
    diffs = []
    if type(a) is not type(b):
        return [(path, a, b)]
    if isinstance(a, dict):
        for k in sorted(set(a) | set(b)):
            if k not in a or k not in b:
                diffs.append((f"{path}.{k}", a.get(k), b.get(k)))
            else:
                diffs += json_field_diff(a[k], b[k], f"{path}.{k}")
    elif isinstance(a, list):
        if len(a) != len(b):
            diffs.append((f"{path}[len]", len(a), len(b)))
        for i in range(min(len(a), len(b))):
            diffs += json_field_diff(a[i], b[i], f"{path}[{i}]")
    else:
        if a != b:
            diffs.append((path, a, b))
    return diffs


def leaf_key(json_path: str) -> str:
    return re.sub(r"\[\d+\]", "", json_path).split(".")[-1]


def status_map():
    out = {}
    for line in git("diff", "--name-status", f"{BASE}..{HEAD_C}").splitlines():
        parts = line.split("\t")
        if len(parts) >= 2:
            out[parts[-1]] = parts[0]
    return out


stats = numstat()
statuses = status_map()
result = {
    "task": "2026-10-25-t4-source-hash-repin-13-independent-check",
    "subject_commit": HEAD_C,
    "head": HEAD,
    "base": BASE,
    "generated_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "files": {},
}
files = sorted(stats)
result["file_count"] = len(files)
result["forbidden_paths"] = [p for p in files if any(p == f or p.startswith(f) for f in FORBIDDEN_PREFIX)]
result["not_allowed_paths"] = [p for p in files if not any(p.startswith(a) for a in ALLOWED_PREFIX)]
result["content_files"] = [p for p in files if not p.startswith("docs/development-evidence/") and not p.startswith(".normify/")]
result["governance_files"] = [p for p in files if p.startswith(".normify/")]
result["content_files_match_delivery"] = sorted(result["content_files"]) == sorted(CONTENT_EXPECTED)

# per-file analysis (added files are evidence/governance records, not literal edits)
text_pairs = {}
bin_diffs = {}
added_files = []
governance_structural_diff = {}
for path in files:
    info = stats[path]
    if statuses.get(path) == "A":
        added_files.append(path)
        continue
    if path.startswith(".normify/changes/"):
        old_j = json.loads(git_bytes("show", f"{BASE}:{path}").decode("utf-8"))
        new_j = json.loads(git_bytes("show", f"{HEAD_C}:{path}").decode("utf-8"))
        governance_structural_diff[path] = [{"path": p, "old": str(o), "new": str(n)} for p, o, n in json_field_diff(old_j, new_j)]
        continue
    if info["binary"]:
        old_raw = git_bytes("show", f"{BASE}:{path}")
        new_raw = git_bytes("show", f"{HEAD_C}:{path}")
        entry = {"binary": True, "bytes_old": len(old_raw), "bytes_new": len(new_raw)}
        try:
            old_j = json.loads(old_raw.decode("utf-8"))
            new_j = json.loads(new_raw.decode("utf-8"))
            diffs = json_field_diff(old_j, new_j)
            entry["json_field_diffs"] = [{"path": p, "old": str(o)[:80], "new": str(n)[:80],
                                          "leaf_key": leaf_key(p)} for p, o, n in diffs]
            entry["all_diffs_are_identity_keys"] = all(leaf_key(p) in ALLOWED_BIN_KEYS for p, _, _ in diffs)
            entry["diff_count"] = len(diffs)
        except Exception as exc:  # noqa: BLE001
            entry["json_parse_error"] = str(exc)
            entry["all_diffs_are_identity_keys"] = False
        bin_diffs[path] = entry
    else:
        pairs = text_line_diff(path)
        text_pairs[path] = pairs

result["text_files"] = {
    p: {
        "changed_lines": len(v),
        "all_masked_equal": all(x["masked_equal"] for x in v),
        "all_have_hex_identity": all(x.get("has_hex") for x in v if x.get("has_hex") is not None),
        "non_literal_pairs": [x for x in v if not x["masked_equal"]],
    } for p, v in text_pairs.items()
}
result["binary_files"] = bin_diffs

# extra assertions
result["tests_files"] = [p for p in result["content_files"] if p.startswith("tests/")]
result["assertion_or_tolerance_files_changed"] = [p for p in files if re.search(r"(exclusion|tolerance|gates\.toml|scripts/)", p.lower())]

# shortstat comparisons
result["shortstat_full_commit"] = git("diff", "--shortstat", f"{BASE}..{HEAD_C}").strip()
result["shortstat_content_only"] = git("diff", "--shortstat", f"{BASE}..{HEAD_C}", "--",
                                       *[p for p in result["content_files"]]).strip()
result["shortstat_delivery_measured_base_to_checkout"] = git("diff", "--shortstat", "4ba11df5..61fb6baf").strip()
result["anatomy"] = {
    "added_files": sorted(added_files),
    "evidence_added": [p for p in added_files if p.startswith("docs/development-evidence/")],
    "normify_runs_added": [p for p in added_files if p.startswith(".normify/runs/")],
    "other_added": [p for p in added_files if not p.startswith("docs/development-evidence/") and not p.startswith(".normify/runs/")],
    "governance_structural_diff": governance_structural_diff,
}

GOVERNANCE_ALLOWED_KEYS = {".status", ".revision.before", ".updated_at", ".closed_at"}
gov_keys_ok = all(
    all(d["path"] in GOVERNANCE_ALLOWED_KEYS for d in diffs)
    for diffs in governance_structural_diff.values()
)

text_ok = all(v["all_masked_equal"] for v in result["text_files"].values())
bin_ok = all(v.get("all_diffs_are_identity_keys", False) for v in result["binary_files"].values())
result["verdict"] = {
    "no_forbidden_paths": not result["forbidden_paths"],
    "no_path_outside_scope": not result["not_allowed_paths"],
    "content_file_set_matches_delivery": result["content_files_match_delivery"],
    "all_modified_text_changes_identity_only_after_hex_mask": text_ok,
    "all_modified_binary_json_changes_identity_keys_only": bin_ok,
    "governance_record_changed_metadata_only": gov_keys_ok,
    "added_files_only_evidence_or_gate_records": not result["anatomy"]["other_added"],
    "no_assertion_tolerance_exclusion_changed": not result["assertion_or_tolerance_files_changed"],
}
result["verdict"]["all_pass"] = all(result["verdict"].values())

(OUT / "qa-diff-boundary.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
lines = [
    f"base={BASE} commit={HEAD_C} head={HEAD}",
    f"files={result['file_count']} content={len(result['content_files'])} evidence_added={len(result['anatomy']['evidence_added'])} runs_added={len(result['anatomy']['normify_runs_added'])}",
    f"forbidden={result['forbidden_paths']} outside_scope={result['not_allowed_paths']}",
    f"content_set_matches_delivery={result['content_files_match_delivery']}",
    f"shortstat_full={result['shortstat_full_commit']}",
    f"shortstat_content_only={result['shortstat_content_only']}",
    f"shortstat_4ba11df5_to_61fb6baf={result['shortstat_delivery_measured_base_to_checkout']}",
]
for p, v in result["text_files"].items():
    lines.append(f"text {p}: lines={v['changed_lines']} masked_equal={v['all_masked_equal']} non_literal={len(v['non_literal_pairs'])}")
for p, v in result["binary_files"].items():
    lines.append(f"bin {p}: count={v.get('diff_count')} identity_only={v.get('all_diffs_are_identity_keys')} bytes={v['bytes_old']}->{v['bytes_new']}")
    for d in v.get("json_field_diffs", []):
        lines.append(f"   {d['path']} : {d['old']} -> {d['new']}")
lines.append(f"verdict={json.dumps(result['verdict'], ensure_ascii=False)}")
(OUT / "qa-diff-boundary.log").write_text("\n".join(lines) + "\n", encoding="utf-8")
print("\n".join(lines))
