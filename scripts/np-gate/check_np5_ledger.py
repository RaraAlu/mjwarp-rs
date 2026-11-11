# NOT in the 93 guards inputs; adding it to guards requires a user re-sign (NP5-00).
# -*- coding: utf-8 -*-
"""Independent checker for np-gate-ledger.json draft (target/ only, read-only)."""
import json, hashlib, pathlib

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
D = ROOT / "target/render-bvh/gl2-ptx-diff"
led = json.loads((D / "np-gate-ledger.json").read_text(encoding="utf-8"))
inv1 = json.loads((ROOT / "docs/development-evidence/np01-type-mapping/inventory.json").read_text(encoding="utf-8"))
inv2 = json.loads((ROOT / "docs/development-evidence/np02-evidence/inventory.json").read_text(encoding="utf-8"))
gap = json.loads((ROOT / "docs/development-evidence/np02-evidence/gap.json").read_text(encoding="utf-8"))

fails = []
def chk(name, cond, detail=""):
    print(("PASS " if cond else "FAIL ") + name + ((" | " + str(detail)) if detail else ""))
    if not cond:
        fails.append(name)

# 1 schema/shape
chk("schema==1", led.get("schema") == 1)
chk("np01 paths == inventory ids", {e["id"] for e in led["np01"]["paths"]} == {r["id"] for r in inv1["rows"]},
    len(led["np01"]["paths"]))
chk("np02 kernels == inventory names", {e["name"] for e in led["np02"]["kernels"]} == {r["name"] for r in inv2["rows"]},
    len(led["np02"]["kernels"]))
chk("np02 gaps == gap items", len(led["np02"]["gaps"]) == len(gap["items"]), len(led["np02"]["gaps"]))

# 2 field tables present + sha self-consistent
bad = []
for e in led["np01"]["paths"]:
    ft = e["field_table"]
    p = ROOT / ft["path"]
    if not (p.exists() and ft["present"]):
        bad.append(e["id"] + ":missing")
        continue
    if hashlib.sha256(p.read_bytes()).hexdigest() != ft["sha256"]:
        bad.append(e["id"] + ":sha")
chk("30 field tables present & sha match", not bad, bad)
bad = []
for e in led["np01"]["host_sites"]:
    ft = e["field_table"]
    p = ROOT / ft["path"]
    if not (p.exists() and ft["present"]):
        bad.append(e["id"] + ":missing")
    elif hashlib.sha256(p.read_bytes()).hexdigest() != ft["sha256"]:
        bad.append(e["id"] + ":sha")
chk("3 host tables present & sha match", not bad, bad)

# 3 unregistered units exist in tree
bad = [u["id"] for u in led["np01"]["unregistered_product_units"] if not u["products_present"]]
chk(f"{len(led['np01']['unregistered_product_units'])} unregistered units all present in tree", not bad, bad)

# 4 totals arithmetic
t = led["totals"]
chk("totals arithmetics", t["ledger_entries_total"] == len(led["np01"]["paths"]) + len(led["np01"]["host_sites"])
    + len(led["np01"]["unregistered_product_units"]) + len(led["np02"]["kernels"])
    + len(led["np02"]["missing_kernel_rows"]) + len(led["np02"]["gaps"]), t["ledger_entries_total"])
chk("field table 30/30", t["np01_paths_with_field_table"] == 30)
chk("host table 3/3", t["np01_host_sites_with_table"] == 3)

# 5 every entry has owner + action/next
nob = []
for e in led["np01"]["paths"] + led["np01"]["host_sites"]:
    if "owner" not in e or "next_action" not in e:
        nob.append(e["id"])
for u in led["np01"]["unregistered_product_units"]:
    if "owner" not in u or "next_action" not in u:
        nob.append(u["id"])
for k in led["np02"]["kernels"]:
    if "owner" not in k:
        nob.append(k["name"])
for g in led["np02"]["gaps"]:
    if "owner" not in g or "ledger_action" not in g:
        nob.append(str(g.get("name")))
chk("all entries have owner/action", not nob, nob[:5])

# 6 gap action histogram vs status histogram
from collections import Counter
want = Counter(gap_owner_status := [g.get("status") for g in gap["items"]])
got_actions = Counter(g["ledger_action"] for g in led["np02"]["gaps"])
chk("9 drift archived", got_actions["archive"] == 9, got_actions["archive"])
chk("7 unimplemented routed to reclassify/blocker", got_actions["reclassify-or-blocker"] == 7, got_actions["reclassify-or-blocker"])
chk("gap statuses mapped (no unknown)", "unknown" not in got_actions)

# 7 subsystem recheck present for all 7
sub = led["np02"]["subsystem_recheck"]
chk("7 subsystems rechecked", len(sub) == 7, list(sub.keys()))

# 8 no pass claims on gates/platforms (hash-verification wording is factual, not a gate claim)
import re
txt = json.dumps(led, ensure_ascii=False)
bad = []
for m in re.finditer(r"(NP01|NP02|门禁|双平台|上游等价|发布)[^。；\"]{0,10}通过", txt):
    pre = txt[max(0, m.start() - 14):m.start()]
    if not re.search(r"(不宣称|不是|未|不得|不构成)", pre):
        bad.append(m.group(0))
chk("no unnegated gate pass claim", not bad, bad[:5])
chk("non-claim statement present", "不宣称" in txt)

# 9 drift row matches actual
nd = led["np02"]["new_source_drift"][0]
cur = hashlib.sha256((ROOT / nd["source"]).read_bytes().replace(b"\r\n", b"\n")).hexdigest()
chk("scene.cu drift recorded==current", nd["current_sha256_lf"] == cur and nd["recorded_sha256"] != cur)

print()
print("RESULT:", "PASS" if not fails else "FAIL " + str(fails))
