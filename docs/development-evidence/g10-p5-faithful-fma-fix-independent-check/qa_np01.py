# -*- coding: utf-8 -*-
"""QA check: NP01 hostSiteRegistration registration (real entries, original rows untouched).

Writes qa-np01.json.
"""
import json
import subprocess
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
TASK = REPO / "docs/development-evidence/g10-p5-faithful-fma-fix-independent-check"
NP01_DIR = REPO / "docs/development-evidence/np01-type-mapping"
INV = NP01_DIR / "inventory.json"
README = NP01_DIR / "README.md"
REV = "50a0acd1"


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


old_text = subprocess.run(["git", "-C", str(REPO), "show",
                           f"{REV}^:docs/development-evidence/np01-type-mapping/inventory.json"],
                          capture_output=True).stdout.decode("utf-8")
old = json.loads(old_text)
now = json.loads(read(INV))
reg = now.get("hostSiteRegistration") or {}
site_semantics = reg.get("siteSemantics") or []
host_coverage = reg.get("hostCoverage") or []
result = {
    "inventory_keys_old": list(old.keys()),
    "inventory_keys_new": list(now.keys()),
    "hostSiteRegistration_present": "hostSiteRegistration" in now,
    "siteSemantics_count": len(site_semantics),
    "siteSemantics_sites": [item.get("site") for item in site_semantics],
    "hostCoverage_count": len(host_coverage),
    "hostCoverage_names": [item.get("id") for item in host_coverage],
    "hostCoverage_has_types": all("types" in item for item in host_coverage),
    "hostCoverage_no_f64": all(item.get("types", {}).get("doubleInProduct") == "0"
                               for item in host_coverage),
    "hostCoverage_has_status": all(bool(item.get("status")) for item in host_coverage),
    "rows_old": len(old.get("rows", [])),
    "rows_new": len(now.get("rows", [])),
    "rows_unchanged": old.get("rows") == now.get("rows"),
    "all_original_top_keys_unchanged": all(now.get(k) == old.get(k) for k in old.keys()),
    "gjk_host_semantics_registered": any("gjk" in str(item.get("site", "")) and
                                         "CPU" in str(item.get("hostRecipe", ""))
                                         for item in site_semantics),
    "device_ptx_registered": any("fma.rn" in str(item.get("deviceRecipe", ""))
                                 for item in site_semantics),
    "render_fused_registered": any("render" in str(item.get("site", ""))
                                   and "fused" in str(item.get("hostRounding", ""))
                                   for item in site_semantics),
    "relmat_fused_registered": any("relmat" in str(item.get("site", ""))
                                   and "fused" in str(item.get("hostRounding", ""))
                                   for item in site_semantics),
    "readme_has_host_section": "host 站点口径" in read(README),
    "readme_section_excerpt": [line.strip() for line in read(README).splitlines()
                               if "host" in line.lower()][:8],
    "siteSemantics_full": site_semantics,
    "hostCoverage_full": host_coverage,
}
(TASK / "qa-np01.json").write_text(
    json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps({k: v for k, v in result.items()
                  if k not in ("siteSemantics_full", "hostCoverage_full", "siteSemantics_sites")},
                 ensure_ascii=False, indent=2))
