"""QA M3 split-purity check: verify the 500-line-ratchet split is a pure move.

- old validate_tests.rs (11a13f15) vs new validate_mesh_poly_tests.rs: CubeMeshPoly
  struct/impl and the three moved test bodies must be verbatim (whitespace-normalised)
- old assembly sha256_hex/f32_bits/json_f32_bits helper bodies must be verbatim in
  tests/support/mesh_poly_case_support.rs
- test-name sets: old names preserved, only new tests added
- scope-selfcheck reproduction: M3 task schedule.files vs the committed change set

Output: target/qa-m3/split-audit.json
"""
import json
import re
import subprocess
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
OUT = ROOT / "target/qa-m3/split-audit.json"


def show(rev_path: str) -> str:
    return subprocess.run(
        ["git", "-C", str(ROOT), "show", rev_path],
        capture_output=True, text=True, encoding="utf-8", errors="replace", check=True,
    ).stdout


def norm(text: str) -> str:
    return re.sub(r"\s+", " ", text).strip()


def block(text: str, needle: str, end_marker: str = "\n}\n") -> str:
    start = text.index(needle)
    end = text.index(end_marker, start) + len(end_marker)
    return norm(text[start:end])


old_validate = show("11a13f15:src/runtime/collision_gpu/validate_tests.rs")
new_child = (ROOT / "src/runtime/collision_gpu/validate_mesh_poly_tests.rs").read_text(encoding="utf-8")
new_parent = (ROOT / "src/runtime/collision_gpu/validate_tests.rs").read_text(encoding="utf-8")

moves = {}
for needle, label in [
    ("struct CubeMeshPoly {", "CubeMeshPoly struct"),
    ("impl CubeMeshPoly {", "CubeMeshPoly impl"),
    ("fn accepts_real_cube_mesh_poly_shapes()", "accepts_real_cube_mesh_poly_shapes"),
    ("fn rejects_inconsistent_mesh_poly_sidecar()", "rejects_inconsistent_mesh_poly_sidecar"),
    ("fn rejects_inconsistent_mesh_poly_sums()", "rejects_inconsistent_mesh_poly_sums"),
]:
    old_text = block(old_validate, needle)
    found = old_text in norm(new_child)
    moves[label] = {"verbatim_in_child": found, "still_in_parent": norm(old_text) in norm(new_parent)}
    if not found:
        moves[label]["status"] = "NOT FOUND"

old_assembly = show("11a13f15:tests/g10_p5_mesh_poly_case_assembly.rs")
support = (ROOT / "tests/support/mesh_poly_case_support.rs").read_text(encoding="utf-8")
helpers = {}
for needle in ["fn sha256_hex(", "fn f32_bits(", "fn json_f32_bits("]:
    old_text = block(old_assembly, needle)
    helpers[needle] = norm(old_text) in norm(support)
    if not helpers[needle]:
        helpers[needle] = "MISSING"

# test name sets
old_validate_names = set(re.findall(r"fn (\w+)\(\)", old_validate))
new_validate_names = set(re.findall(r"fn (\w+)\(\)", new_child)) | set(
    re.findall(r"fn (\w+)\(\)", new_parent)
)
old_assembly_names = set(re.findall(r"fn (\w+)\(\)", old_assembly))
new_assembly_text = (
    (ROOT / "tests/g10_p5_mesh_poly_case_assembly.rs").read_text(encoding="utf-8")
    + (ROOT / "tests/g10_p5_mesh_m3_case_assembly.rs").read_text(encoding="utf-8")
)
new_assembly_names = set(re.findall(r"fn (\w+)\(\)", new_assembly_text))

# scope-selfcheck reproduction for the M3 commit
task = json.loads(
    (ROOT / ".normify/changes/2026-10-26-mesh-multicontact-m3-mesh-mesh.json").read_text(
        encoding="utf-8"
    )
)
scope = task["schedule"]["files"]
changed = subprocess.run(
    ["git", "-C", str(ROOT), "diff", "--name-only", "11a13f15..f1e5cda1"],
    capture_output=True, text=True, encoding="utf-8", check=True,
).stdout.split("\n")
changed = [p.strip() for p in changed if p.strip() and not p.startswith((".normify/", "target/"))]


def in_scope(path: str) -> bool:
    for entry in scope:
        entry = entry.replace("\\", "/")
        if path == entry:
            return True
        if entry.endswith("/"):
            if path.startswith(entry):
                return True
        elif path.startswith(entry + "/"):
            return True
    return False


outside = [p for p in changed if not in_scope(p)]
report = {
    "kind": "qa-m3-split-audit",
    "task_id": "2026-10-26-mesh-multicontact-m3-independent-check",
    "validate_tests_moves": moves,
    "assembly_helpers_moves": helpers,
    "test_names": {
        "validate_old": sorted(n for n in old_validate_names if "fn " not in n),
        "validate_preserved": sorted(old_validate_names & new_validate_names),
        "validate_missing": sorted(old_validate_names - new_validate_names),
        "validate_added": sorted(new_validate_names - old_validate_names),
        "assembly_old": sorted(old_assembly_names),
        "assembly_preserved": sorted(old_assembly_names & new_assembly_names),
        "assembly_missing": sorted(old_assembly_names - new_assembly_names),
        "assembly_added": sorted(new_assembly_names - old_assembly_names),
    },
    "scope_selfcheck_reproduction": {
        "note": "reproduced with the same matching rules as scripts/scope-selfcheck.mjs against the committed M3 change set (without .normify/target)",
        "changed_paths": len(changed),
        "in_scope": len(changed) - len(outside),
        "outside": outside,
        "matches_claim_75_paths_0_out_of_scope": len(changed) == 75 and not outside,
    },
    "test_file_lines": {
        "src/runtime/collision_gpu/validate_tests.rs": len((ROOT / "src/runtime/collision_gpu/validate_tests.rs").read_text(encoding="utf-8").split("\n")),
        "src/runtime/collision_gpu/validate_mesh_poly_tests.rs": len((ROOT / "src/runtime/collision_gpu/validate_mesh_poly_tests.rs").read_text(encoding="utf-8").split("\n")),
        "tests/g10_p5_mesh_poly_case_assembly.rs": len((ROOT / "tests/g10_p5_mesh_poly_case_assembly.rs").read_text(encoding="utf-8").split("\n")),
        "tests/g10_p5_mesh_m3_case_assembly.rs": len((ROOT / "tests/g10_p5_mesh_m3_case_assembly.rs").read_text(encoding="utf-8").split("\n")),
    },
}
OUT.parent.mkdir(parents=True, exist_ok=True)
OUT.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
print(json.dumps(report, ensure_ascii=False, indent=2))
