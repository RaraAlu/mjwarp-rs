# -*- coding: utf-8 -*-
"""Independent scan of f32::mul_add / FMA helpers in src/ and tests/."""
import json
import re
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
OUT = REPO / "docs/development-evidence/case-evidence-frame-diagnosis-independent-check"
HEAD = (REPO / ".git/HEAD").read_text(encoding="utf-8").strip()

patterns = {
    "mul_add_call": re.compile(r"\bmul_add\s*\("),
    "mul_add_mention": re.compile(r"mul_add"),
    "helper_fma_f32": re.compile(r"\bfn\s+fma_f32\s*\("),
    "helper_fused_mul_add": re.compile(r"\bfn\s+fused_mul_add\s*\("),
}


def classify(rel: str) -> str:
    parts = rel.replace("\\", "/")
    if parts.startswith("tests/"):
        return "test_crate"
    if "/tests/" in parts or parts.endswith("/tests.rs"):
        return "test_module_in_src"
    return "product_src"


rows = []
for root_name in ("src", "tests"):
    for path in sorted((REPO / root_name).rglob("*.rs")):
        rel = str(path.relative_to(REPO)).replace("\\", "/")
        text = path.read_text(encoding="utf-8", errors="replace")
        for lineno, line in enumerate(text.splitlines(), 1):
            for name, pattern in patterns.items():
                if pattern.search(line):
                    rows.append({
                        "file": rel,
                        "line": lineno,
                        "kind": name,
                        "class": classify(rel),
                        "text": line.strip()[:220],
                    })

# Explicit checks required by the acceptance criteria.
by_file = {}
for row in rows:
    by_file.setdefault(row["file"], []).append(row)

def calls(rel):
    return [r for r in by_file.get(rel, []) if r["kind"] == "mul_add_call"]

result = {
    "task": "2026-10-25-case-evidence-frame-diagnosis-independent-check",
    "generated_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "head": HEAD,
    "patterns": list(patterns),
    "rows": rows,
    "product_mul_add_call_sites": [r for r in rows if r["kind"] == "mul_add_call" and r["class"] == "product_src"],
    "test_mul_add_call_sites": [r for r in rows if r["kind"] == "mul_add_call" and r["class"] != "product_src"],
    "required_sites": {
        "src/physics/collision/gjk/polytope.rs": calls("src/physics/collision/gjk/polytope.rs"),
        "src/render/kernel/megakernel_arrays.rs": calls("src/render/kernel/megakernel_arrays.rs"),
    },
    "collision_dir_mul_add_calls": [r for r in rows if r["kind"] == "mul_add_call" and r["file"].startswith("src/physics/collision/")],
    "precedent_helpers": [
        r for r in rows if r["kind"] in ("helper_fma_f32", "helper_fused_mul_add")
    ],
    "precedent_comments": [
        r for r in rows if r["kind"] == "mul_add_mention" and r["file"] in (
            "src/physics/collision/relmat.rs",
            "src/render/bvh/aabb.rs",
        )
    ],
}

(OUT / "qa-mul-add-scan.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(f"rows={len(rows)} product_calls={len(result['product_mul_add_call_sites'])} test_calls={len(result['test_mul_add_call_sites'])}")
for r in result["product_mul_add_call_sites"]:
    print(f"PRODUCT {r['file']}:{r['line']} {r['text']}")
for r in result["test_mul_add_call_sites"]:
    print(f"TEST    {r['file']}:{r['line']} {r['text']}")
print("collision_dir_calls=", len(result["collision_dir_mul_add_calls"]))
for r in result["precedent_helpers"]:
    print(f"HELPER  {r['file']}:{r['line']} {r['text']}")
