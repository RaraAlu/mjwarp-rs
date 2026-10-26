# -*- coding: utf-8 -*-
"""QA static/scope checks for the option-A fix (r2).

Read-only: git diffs, source text, cheat-check paths, NP01 inventory, delivery logs.
Writes qa-scope-static.json.
"""
import json
import re
import subprocess
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
TASK = REPO / "docs/development-evidence/g10-p5-faithful-fma-fix-independent-check"
REV = "50a0acd1"
EV = REPO / "docs/development-evidence/faithful-fma-fix"
POLY = REPO / "src/physics/collision/gjk/polytope.rs"
MEGA = REPO / "src/render/kernel/megakernel_arrays.rs"
CASE_EV = REPO / "src/physics/collision/gjk/case_evidence.rs"
CASE_CMP = REPO / "src/physics/collision/gjk/case_compare.rs"
NP01 = REPO / "docs/development-evidence/np01-type-mapping/inventory.json"


def git(*args):
    out = subprocess.run(["git", "-C", str(REPO), *args], capture_output=True,
                         text=True, encoding="utf-8", errors="replace")
    return out.stdout.strip(), out.returncode


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def sha256(path: Path) -> str:
    import hashlib
    return hashlib.sha256(path.read_bytes()).hexdigest()


def parse_judge_lines(path: Path) -> list:
    if not path.is_file():
        return []
    return [line.strip() for line in read(path).splitlines() if line.strip().startswith("[判据]")]


head, _ = git("rev-parse", "HEAD")
parent, _ = git("rev-parse", f"{REV}^")
names, _ = git("diff", "--name-status", f"{REV}^..{REV}")
name_rows = [line for line in names.splitlines() if line.strip()]

result = {
    "head": head,
    "r2": REV,
    "r2_parent": parent,
    "commit_files": name_rows,
    "commit_stat": git("show", "--stat", "--oneline", REV)[0].splitlines()[-1],
    "render_site_changed_in_commit": any("src/render/kernel/megakernel_arrays.rs" in line for line in name_rows),
    "fix_r2_acceptance_allows_megakernel": "megakernel 段可复用" in read(
        REPO / ".normify/changes/2026-10-25-g10-p5-faithful-fma-fix-r2.json"),
    "qa_acceptance_literal_says_render_must_not_change": True,
}

# 1) forbidden paths must not be in the commit
forbidden = ["relmat.rs", "render/bvh/aabb.rs", "case_compare.rs", "case_evidence.rs",
             ".normify/gates.toml", "scripts/", "fixtures/", "Cargo.toml", "Cargo.lock",
             "upstream-mjwarp-f32.json", "upstream-mjwarp-f32-gpu.json"]
result["forbidden_path_hits"] = [
    line for line in name_rows if any(f in line for f in forbidden)]

# 2) polytope content
def code_only(text: str) -> str:
    return "\n".join(line.split("//", 1)[0] for line in text.splitlines())


poly = read(POLY)
poly_code = code_only(poly)
result["polytope"] = {
    "has_mul_add_in_code": "mul_add" in poly_code,
    "has_fma_f32_in_code": "fma_f32" in poly_code,
    "has_f64_in_code": bool(re.search(r"\bf64\b|as f64", poly_code)),
    "explicit_two_rounding": ("r[1] * r[1] + head" in poly and "r[2] * r[2] + tail" in poly),
    "mentions_cpu_recipe_and_gpu_semantics": ("CPU" in poly and "fma.rn" in poly),
    "head_tail_lines": [line.strip() for line in poly.splitlines()
                        if "let head" in line or "let tail" in line or "face_norm2[idx]" in line],
}
mega = read(MEGA)
result["render"] = {
    "has_fma_f32": "crate::math::fma::fma_f32" in mega,
    "has_mul_add_on_light_site": "z.mul_add(z" in mega,
    "light_line": next((line.strip() for line in mega.splitlines()
                        if "fma_f32(z" in line or line.strip().startswith("let length =")), ""),
}

# 3) full src site scan
scan = []
for path in sorted((REPO / "src").rglob("*.rs")):
    text = read(path)
    for index, line in enumerate(text.splitlines(), 1):
        if re.search(r"\bmul_add\b|fma_f32\s*\(|fused_mul_add", line):
            scan.append({"path": str(path.relative_to(REPO)).replace("\\", "/"),
                         "line": index, "text": line.strip()})
result["src_site_scan"] = [f"{s['path']}:{s['line']}: {s['text']}" for s in scan]
result["product_sites"] = [
    f"{s['path']}:{s['line']}: {s['text']}"
    for s in scan if not s["path"].startswith(
        ("src/math/fma/tests.rs", "src/runtime/transfer/kernel/tests/precision.rs"))]

product_code_mul_add = []
for path in sorted((REPO / "src").rglob("*.rs")):
    rel = str(path.relative_to(REPO)).replace("\\", "/")
    if rel.endswith("/tests.rs") or "/tests/" in rel or rel.startswith("src/math/fma/tests"):
        continue
    if rel.startswith("src/runtime/transfer/kernel/tests/"):
        continue
    for index, line in enumerate(read(path).splitlines(), 1):
        code = line.split("//", 1)[0]
        if "mul_add" in code and "fused_mul_add" not in code:
            product_code_mul_add.append({"path": rel, "line": index, "code": code.strip()})
result["product_code_mul_add_call_sites"] = product_code_mul_add
result["product_code_mul_add_count"] = len(product_code_mul_add)

# 4) cheat-check: criterion/reference/tooling paths untouched
cheat_paths = [
    "src/physics/collision/gjk/case_evidence.rs",
    "src/physics/collision/gjk/case_compare.rs",
    ".normify/gates.toml",
    "scripts/",
    "fixtures/",
    "docs/development-evidence/g10-p5-gjk-epa/upstream-mjwarp-f32.json",
    "docs/development-evidence/g10-p5-gjk-epa/upstream-mjwarp-f32-gpu.json",
]
result["cheat_path_diffs"] = {path: (git("diff", "--name-status", f"{REV}^..{REV}", "--", path)[0]
                                     or "(none)") for path in cheat_paths}

# 5) criterion constants still present and unchanged (semantic text)
case_ev = read(CASE_EV)
result["criterion"] = {
    "frame_threshold_4_8e_7_present": "4.8e-7" in case_ev,
    "ulp4_asserts_present": case_ev.count("<= 4") >= 2,
    "dist_ulp_line": next((line.strip() for line in case_ev.splitlines() if "4.8e-7" in line), ""),
}
result["reference_path"] = {
    "case_compare_reads_cpu_reference": "upstream-mjwarp-f32.json" in read(CASE_CMP),
    "case_compare_reads_gpu_reference": "upstream-mjwarp-f32-gpu.json" in read(CASE_CMP),
    "cpu_reference_sha256": sha256(REPO / "docs/development-evidence/g10-p5-gjk-epa/upstream-mjwarp-f32.json"),
    "gpu_reference_sha256": sha256(REPO / "docs/development-evidence/g10-p5-gjk-epa/upstream-mjwarp-f32-gpu.json"),
}

# 6) NP01 inventory: new registration + original 30 untouched
inv_now = json.loads(read(NP01))
inv_old_text = subprocess.run(["git", "-C", str(REPO), "show", f"{REV}^:docs/development-evidence/np01-type-mapping/inventory.json"],
                               capture_output=True).stdout.decode("utf-8")
inv_old = json.loads(inv_old_text)


def count_entries(value):
    if isinstance(value, dict):
        return sum(count_entries(v) for v in value.values())
    if isinstance(value, list):
        return len(value)
    return 0


result["np01"] = {
    "top_level_keys_old": list(inv_old.keys()),
    "top_level_keys_new": list(inv_now.keys()),
    "hostSiteRegistration_present": "hostSiteRegistration" in inv_now,
    "hostSiteRegistration_summary": {
        "siteSemantics_count": len((inv_now.get("hostSiteRegistration") or {}).get("siteSemantics") or []),
        "hostCoverage_count": len((inv_now.get("hostSiteRegistration") or {}).get("hostCoverage") or []),
        "siteSemantics_sites": [item.get("site") for item in
                                ((inv_now.get("hostSiteRegistration") or {}).get("siteSemantics") or [])],
        "hostCoverage_ids": [item.get("id") for item in
                             ((inv_now.get("hostSiteRegistration") or {}).get("hostCoverage") or [])],
    },
    "old_entries_count": count_entries(inv_old),
    "new_entries_count": count_entries(inv_now),
    "original_payload_unchanged": all(inv_now.get(k) == inv_old.get(k) for k in inv_old.keys()),
    "rows_old": len(inv_old.get("rows", [])),
    "rows_new": len(inv_now.get("rows", [])),
    "rows_unchanged": inv_old.get("rows") == inv_now.get("rows"),
    "readme_has_host_section": "host 站点口径" in read(
        REPO / "docs/development-evidence/np01-type-mapping/README.md"),
}

# 7) delivery evidence: pre/post judge lines and README claims
logs = EV / "logs"
pre = {mode: parse_judge_lines(logs / f"pre-fix-case-evidence-{mode}.log")
       for mode in ("gnu-default", "gnu-fma", "msvc")}
post = {mode: parse_judge_lines(logs / f"post-fix-case-evidence-{mode}.log")
        for mode in ("gnu-default", "gnu-fma", "msvc")}
result["delivery_judge_lines"] = {
    "pre_counts": {mode: len(lines) for mode, lines in pre.items()},
    "post_counts": {mode: len(lines) for mode, lines in post.items()},
    "post_identical_across_modes": post["gnu-default"] == post["gnu-fma"] == post["msvc"],
    "pre_gnu_default": pre["gnu-default"],
    "pre_msvc": pre["msvc"],
    "pre_gnu_fma": pre["gnu-fma"],
    "post_gnu_default": post["gnu-default"],
}
result["delivery_readme_claims"] = {
    "claims_host_gnu_was_green_coupling": ("也是绿的" in read(EV / "README.md"))
    and ("偶合" in read(EV / "README.md")),
    "claims_host_msvc_red_to_green": "6.351978e-7" in read(EV / "README.md"),
    "claims_full_g01_not_run": "未执行" in read(EV / "README.md"),
    "evidence_window_files": [p.name for p in logs.glob("*window*")],
    "mentions_gpu_window_zero": "GPU 窗口 0" in read(EV / "README.md"),
}

(TASK / "qa-scope-static.json").write_text(
    json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result, ensure_ascii=False, indent=2)[:6000])
