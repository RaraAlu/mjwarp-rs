# -*- coding: utf-8 -*-
"""Build temporary tampered fixture copies for the QA reverse-discrimination check.

The copies live under target/qa-t4-check/ (NOT the repo fixture). Each variant is
derived from the current repo fixture:
  - tampered-sha.json         : one expected sha256 changed (first hex digit 0 -> 1)
  - tampered-path-rename.json : one path renamed (src/physics/sleep.rs -> src/physics/sleep_renamed.rs)
  - tampered-path-remove.json : one path removed (19 entries)
  - normal-copy.json          : unmodified copy (control; must pass)
Writes qa-fixture-tamper.json (what was changed) and exits non-zero on error.
"""
import json
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
TASK = REPO / "docs/development-evidence/t4-identity-live-check-coverage-independent-check"
FIXTURE = REPO / "fixtures/precision/t4-current-kernels.json"
OUT = REPO / "target/qa-t4-check"
OUT.mkdir(parents=True, exist_ok=True)

fixture = json.loads(FIXTURE.read_text(encoding="utf-8"))
base = fixture["product_sources"]
record = {"source": str(FIXTURE.relative_to(REPO)).replace("\\", "/"), "variants": {}}


def write(name: str, mutate):
    data = json.loads(json.dumps(fixture))
    original = [dict(entry) for entry in data["product_sources"]]
    mutation = mutate(data["product_sources"])
    path = OUT / name
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    record["variants"][name] = {"file": str(path), "mutation": mutation, "original_entries": original}
    return path


def tamper_sha(entries):
    target = entries[0]["path"]
    before = entries[0]["sha256"]
    entries[0]["sha256"] = ("1" if before[0] != "1" else "2") + before[1:]
    return {"kind": "sha", "path": target, "before": before, "after": entries[0]["sha256"]}


def tamper_rename(entries):
    target = "src/physics/sleep.rs"
    renamed = "src/physics/sleep_renamed.rs"
    for entry in entries:
        if entry["path"] == target:
            entry["path"] = renamed
            return {"kind": "path_rename", "before": target, "after": renamed}
    raise SystemExit(f"path not found in fixture: {target}")


def tamper_remove(entries):
    target = "src/physics/sleep.rs"
    kept = [entry for entry in entries if entry["path"] != target]
    if len(kept) == len(entries):
        raise SystemExit(f"path not found in fixture: {target}")
    entries.clear()
    entries.extend(kept)
    return {"kind": "path_remove", "removed": target, "before_count": len(kept) + 1, "after_count": len(kept)}


write("tampered-sha.json", tamper_sha)
write("tampered-path-rename.json", tamper_rename)
write("tampered-path-remove.json", tamper_remove)
write("normal-copy.json", lambda entries: {"kind": "none"})

(TASK / "qa-fixture-tamper.json").write_text(
    json.dumps(record, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(f"wrote {len(record['variants'])} fixture copies under {OUT}")
for name, info in record["variants"].items():
    print(f"  {name}: {info['mutation']}")
