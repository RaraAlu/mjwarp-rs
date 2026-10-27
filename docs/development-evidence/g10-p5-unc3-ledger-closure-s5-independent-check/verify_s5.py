"""UNC-3 S5 独立复核复算（只读；不跑 cargo/GPU）。HEAD=77055c95（S5 主体 7b99cbf4 + 补正 77055c95）。"""
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
OUT = Path(__file__).with_name("verify_s5.json")
BASE = "2a0de8ab"          # S5 登记前（U8 backlog 提交）
BODY = "7b99cbf4"          # S5 主体
AFTER = "77055c95"         # S5 补正（HEAD）
MAP_PATH = "docs/development-evidence/g10-p5-acceptance-map/map.json"


def git_show(rev: str, path: str) -> bytes:
    return subprocess.run(["git", "show", f"{rev}:{path}"], cwd=REPO, capture_output=True).stdout


def git(*args) -> str:
    return subprocess.run(["git", *args], cwd=REPO, capture_output=True).stdout.decode("utf-8", "replace")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def diff_paths(a, b, prefix=""):
    changed, added, removed = [], [], []
    if type(a) is not type(b):
        changed.append(prefix or "<root>")
        return changed, added, removed
    if isinstance(a, dict):
        for key in a.keys() | b.keys():
            path = f"{prefix}.{key}" if prefix else key
            if key not in b:
                removed.append(path)
            elif key not in a:
                added.append(path)
            else:
                c, ad, rm = diff_paths(a[key], b[key], path)
                changed += c
                added += ad
                removed += rm
        return changed, added, removed
    if isinstance(a, list):
        if len(a) != len(b):
            changed.append(f"{prefix}[len]")
            return changed, added, removed
        for index, (x, y) in enumerate(zip(a, b)):
            c, ad, rm = diff_paths(x, y, f"{prefix}[{index}]")
            changed += c
            added += ad
            removed += rm
        return changed, added, removed
    if a != b:
        changed.append(prefix)
    return changed, added, removed


def main():
    out = {}
    before = json.loads(git_show(BASE, MAP_PATH).decode("utf-8"))
    body = json.loads(git_show(BODY, MAP_PATH).decode("utf-8"))
    after = json.loads(git_show(AFTER, MAP_PATH).decode("utf-8"))

    # ---- 1. 总计 diff（BASE → AFTER）：只允许 UNC-1/UNC-3 + summary.note + top-level s5_update ----
    changed, added, removed = diff_paths(before, after)
    out["map_diff_base_to_after"] = {
        "changed": sorted(changed),
        "added": sorted(added),
        "removed": sorted(removed),
        "unexpected_changed": [p for p in changed if not p.startswith(("uncovered[", "summary.note"))],
        "unexpected_added": [p for p in added if not p.startswith(("uncovered[", "s5_update"))],
        "unexpected_removed": removed,
    }

    def index_of(doc, rid):
        return [i for i, item in enumerate(doc["uncovered"]) if item.get("requirement_id") == rid]

    out["uncovered_ids"] = [item.get("requirement_id") for item in after["uncovered"]]
    modified_uncovered = set()
    for path in changed + added:
        if path.startswith("uncovered["):
            index = int(path.split("[")[1].split("]")[0])
            modified_uncovered.add(after["uncovered"][index].get("requirement_id"))
    out["modified_uncovered"] = sorted(modified_uncovered)
    out["unexpected_uncovered"] = sorted(modified_uncovered - {"UNC-1", "UNC-3"})

    untouched = [rid for rid in out["uncovered_ids"] if rid not in modified_uncovered]
    out["untouched_uncovered"] = untouched
    identical = True
    for rid in untouched:
        i, j = index_of(before, rid)[0], index_of(after, rid)[0]
        if before["uncovered"][i] != after["uncovered"][j]:
            identical = False
    out["untouched_uncovered_identical"] = identical and len(untouched) == 4
    out["requirements_identical"] = before["requirements"] == after["requirements"]
    out["summary_counts_unchanged"] = all(
        before["summary"].get(k) == after["summary"].get(k)
        for k in ("requirements_total", "met", "partial", "unmet", "uncovered_total")
    )
    out["top_level_new_keys"] = sorted(set(after) - set(before))

    # ---- 2. 补正口径：UNC-1/UNC-3 reason/impact 为字符串追加，旧文保留在 previous_* ----
    append_check = {}
    for rid in ("UNC-1", "UNC-3"):
        i, j = index_of(before, rid)[0], index_of(after, rid)[0]
        old, new = before["uncovered"][i], after["uncovered"][j]
        entry = {}
        for field in ("reason", "impact"):
            old_value = old.get(field, "")
            new_value = new.get(field, "")
            entry[field] = {
                "old_preserved_as_prefix": new_value.startswith(old_value),
                "old_present": old_value in new_value,
                "previous_field_equals_old": new.get("s5_update", {}).get(f"previous_{field}") == old_value,
                "status_unchanged": old.get("status") == new.get("status"),
            }
        entry["added_keys"] = sorted(set(new) - set(old))
        append_check[rid] = entry
    out["append_check"] = append_check
    out["append_check_ok"] = all(
        entry[field]["old_preserved_as_prefix"] and entry[field]["previous_field_equals_old"] and entry[field]["status_unchanged"]
        for entry in append_check.values()
        for field in ("reason", "impact")
    ) and all(entry["added_keys"] == ["s5_update"] for entry in append_check.values())

    # ---- 3. 补正提交 77055c95 只动预期文件 ----
    out["correction_commit_files"] = [line for line in git("show", "--name-only", "--format=", AFTER).splitlines() if line.strip()]

    # ---- 4. S2 branch-map mesh 计数 ----
    branch_map = json.loads((REPO / "docs/development-evidence/unc3-branch-layer-static-map/branch-map.json").read_text(encoding="utf-8"))
    mesh_entries = [e for e in branch_map["entries"] if e.get("family") == "mesh-multicontact"]
    statuses = {}
    for entry in mesh_entries:
        statuses[entry["status"]] = statuses.get(entry["status"], 0) + 1
    out["s2_mesh_family"] = {"total": len(mesh_entries), "by_status": statuses,
                             "expected": {"已 case 覆盖": 3, "仅静态核对": 4, "仅限制存在": 2, "未覆盖": 5}}
    out["s2_mesh_counts_match"] = all(statuses.get(k, 0) == v for k, v in out["s2_mesh_family"]["expected"].items()) and len(mesh_entries) == 14
    out["s2_verifiable_total"] = branch_map["counts"]["by_status"].get("可验证")

    # ---- 5. UNC-3 六项 vs S4 日志 ----
    logs = REPO / "docs/development-evidence/g10-p5-branch-closure-validation/logs"
    branch_text = (logs / "validation-attempt1-g10_p5_branch_closure_cuda.log").read_text(encoding="utf-8")
    launcher_text = (logs / "validation-attempt1-g10_p5_nvrtc_launcher.log").read_text(encoding="utf-8")
    cases = {}
    for match in re.finditer(r"case=([a-z0-9\-]+) kernel=\w+ nacon=(\d+) ref_count=(\d+) failures=(\d+)", branch_text):
        cases[match.group(1)] = {"nacon": int(match.group(2)), "ref": int(match.group(3)), "failures": int(match.group(4))}
    launcher = {}
    for match in re.finditer(r"case=([a-z0-9\-]+) kind=\w+ nacon=(\d+) nccd=\d+ bit_equal=(\d+)/(\d+) max_dist_ulp=(\d+)", launcher_text):
        launcher[match.group(1)] = {"nacon": int(match.group(2)), "bit": f"{match.group(3)}/{match.group(4)}", "dist_ulp": int(match.group(5))}
    out["unc3_numbers"] = {
        "U1": cases.get("plane-cylinder-degenerate"),
        "U3": {k: cases.get(k) for k in ("plane-box-corner-face-down", "plane-box-corner-face-up",
                                         "plane-box-corner-tilt-x", "plane-box-corner-tilt-y", "plane-box-corner-single")},
        "U4": cases.get("plane-capsule-lying"),
        "U5": cases.get("plane-cylinder-deep-tilt"),
        "U6": cases.get("mesh-mesh-two-assets"),
        "U9": launcher.get("hfield-capsule"),
    }
    out["unc3_numbers_match"] = (
        cases.get("plane-cylinder-degenerate", {}).get("nacon") == 3
        and cases.get("plane-capsule-lying", {}).get("nacon") == 2
        and cases.get("plane-cylinder-deep-tilt", {}).get("nacon") == 4
        and cases.get("mesh-mesh-two-assets", {}).get("nacon") == 3
        and launcher.get("hfield-capsule", {}).get("bit") == "275/275"
        and launcher.get("hfield-capsule", {}).get("dist_ulp") == 0
        and all(item and item["nacon"] == item["ref"] and item["failures"] == 0 for item in cases.values())
    )
    out["counting"] = {"case_level": 3 + 4 + 3 + 6 + 4 + 10 + 2 + 1, "expected": 33}

    # ---- 6. s5-receipt changed_files + map_text_correction ----
    receipt = json.loads((REPO / "docs/development-evidence/g10-p5-unc3-ledger-governance/s5-receipt.json").read_text(encoding="utf-8"))
    files_check = []
    for entry in receipt.get("changed_files", []):
        path = REPO / entry["path"]
        ok = path.exists()
        has_hash = "bytes" in entry and "sha256" in entry
        files_check.append({
            "path": entry["path"],
            "self_entry_no_hash": not has_hash,
            "bytes_match": has_hash and ok and path.stat().st_size == entry.get("bytes"),
            "sha_match": has_hash and ok and sha256(path) == entry.get("sha256"),
        })
    out["changed_files_check"] = {
        "rows": files_check,
        "hashed_rows": sum(1 for r in files_check if not r["self_entry_no_hash"]),
        "all_match": all(r.get("bytes_match") and r.get("sha_match") for r in files_check if not r["self_entry_no_hash"]),
    }
    out["map_text_correction_present"] = "map_text_correction" in receipt
    out["map_text_correction"] = receipt.get("map_text_correction")

    # ---- 7. 交付记录 blob/sha ----
    record = json.loads((REPO / "docs/development-evidence/changes/2026-10-27-unc3-ledger-closure-s5.json").read_text(encoding="utf-8"))
    record_rows = []
    for entry in record.get("files", []):
        path = REPO / entry.get("path", "")
        blob = subprocess.run(["git", "hash-object", entry.get("path", "")], cwd=REPO, capture_output=True, text=True).stdout.strip()
        record_rows.append({
            "path": entry.get("path"),
            "blob_match": blob == entry.get("blob"),
            "sha_match": (not entry.get("sha256")) or (path.exists() and sha256(path) == entry.get("sha256")),
        })
    out["delivery_record"] = {"rows": len(record_rows), "all_match": all(r["blob_match"] and r["sha_match"] for r in record_rows), "rows_detail": record_rows}

    # ---- 8. 新增行措辞（只查 diff 新增行） ----
    added_lines = []
    diff = git("diff", "-U0", "09f6551e", AFTER, "--",
               "docs/development-evidence/g10-p5-acceptance-map/README.md",
               "docs/development-evidence/g10-p5-acceptance-map/map.json",
               "docs/development-evidence/mesh-multicontact-mesh-summary/README.md",
               "docs/development-evidence/g10-p5-unc3-ledger-governance/README.md",
               "docs/development-evidence/g10-p5-unc3-ledger-governance/s5-receipt.json")
    for line in diff.splitlines():
        if line.startswith("+") and not line.startswith("+++"):
            added_lines.append(line[1:])
    s5_task = json.loads((REPO / ".normify/changes/2026-10-27-unc3-ledger-closure-s5.json").read_text(encoding="utf-8"))
    claim_terms = [c["terms"] for c in s5_task.get("checks", []) if c.get("kind") == "forbid_claim"][0]
    phrases = [(f"claim_{i}", term) for i, term in enumerate(claim_terms)]
    phrases += [(f"extra_{i}", p) for i, p in enumerate(
        ["与上游一致", "已验证回退", "U8 已覆盖", "窗口通过", "全分支覆盖", "全量覆盖"])]
    hits = []
    for line in added_lines:
        labels = []
        masked = line
        for label, phrase in phrases:
            if phrase in line:
                labels.append(label)
                if label.startswith("claim_"):
                    masked = masked.replace(phrase, "[claim-term]")
        if labels:
            hits.append({"labels": labels, "line": masked.strip()[:300]})
    out["added_lines"] = len(added_lines)
    out["added_line_phrase_hits"] = hits

    # ---- 9. backlog ----
    backlog = json.loads((REPO / ".normify/changes/2026-10-27-mesh-polygon-gt16-device-gap.json").read_text(encoding="utf-8"))
    out["backlog"] = {
        "status": backlog.get("status"),
        "acceptance_count": len(backlog.get("acceptance", [])),
        "needs_user_approval": any("用户批准" in item for item in backlog.get("acceptance", [])),
        "not_started": backlog.get("status") in ("backlog", "proposed"),
    }

    OUT.write_text(json.dumps(out, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
    brief = {k: out[k] for k in (
        "map_diff_base_to_after", "modified_uncovered", "unexpected_uncovered", "untouched_uncovered",
        "untouched_uncovered_identical", "requirements_identical", "summary_counts_unchanged",
        "top_level_new_keys", "append_check_ok", "correction_commit_files", "s2_mesh_counts_match",
        "s2_verifiable_total", "unc3_numbers_match", "counting", "changed_files_check",
        "map_text_correction_present", "delivery_record", "added_lines", "backlog")}
    print(json.dumps(brief, ensure_ascii=False, indent=1, default=str)[:7000])
    print("=== added-line phrase hits ===")
    for hit in hits:
        print(hit["labels"], "|", hit["line"])
    return 0


if __name__ == "__main__":
    sys.exit(main())
