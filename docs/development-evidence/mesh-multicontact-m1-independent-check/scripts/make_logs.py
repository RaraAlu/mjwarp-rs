"""Generate M1 QA verification log: precheck equality, window sample recompute,
raw/reference byte equality, old reference shas, script shas."""
import csv
import hashlib
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
D = ROOT / "docs/development-evidence/mesh-multicontact-m1"


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


print("=== precheck rerun vs committed precheck")
a = __import__("json").loads((D / "precheck-cpu.json").read_text(encoding="utf-8"))
b = __import__("json").loads((ROOT / "target/qa-m1/precheck-rerun.json").read_text(encoding="utf-8"))
print("result", a["result"], b["result"])
for k in a["cases"]:
    ca, cb = a["cases"][k], b["cases"][k]
    print(
        k,
        "nacon_on", ca["multiccd_on"]["nacon"], cb["multiccd_on"]["nacon"],
        "nacon_off", ca["multiccd_off"]["nacon"], cb["multiccd_off"]["nacon"],
        "ids_eq", ca["multiccd_on"]["geomcollisionid"] == cb["multiccd_on"]["geomcollisionid"],
        "on_dists_eq", ca["multiccd_on"]["dists"] == cb["multiccd_on"]["dists"],
        "off_dists_eq", ca["multiccd_off"]["dists"] == cb["multiccd_off"]["dists"],
        "poly", cb["polygon_data_present"], "hit", cb["multicontact_path_hit"],
    )
print("xml_sha_eq", all(a["cases"][k]["xml_sha256"] == b["cases"][k]["xml_sha256"] for k in a["cases"]))
print("qpos_bits_eq", all(a["cases"][k]["qpos_bits"] == b["cases"][k]["qpos_bits"] for k in a["cases"]))

print("\n=== window samples recompute")
rows = list(csv.DictReader(open(ROOT / "target/m1-mesh-multicontact/attempt1/samples.csv", encoding="utf-8")))
used = [int(r["used_mib"]) for r in rows]
free = [int(r["free_mib"]) for r in rows]
util = [int(r["util_percent"]) for r in rows]
ts = [r["timestamp"] for r in rows]
streak = best = 0
for u in util:
    streak = streak + 1 if u >= 80 else 0
    best = max(best, streak)
print("samples", len(rows), "used_peak", max(used), "free_min", min(free), "util_max", max(util),
      "util_streak_ge80", best, "delta_vs_first", max(used) - used[0],
      "monotonic", all(ts[i] <= ts[i + 1] for i in range(len(ts) - 1)))
print("first_ts", ts[0], "last_ts", ts[-1])

print("\n=== raw A/B vs committed reference")
raw_a = (ROOT / "target/m1-mesh-multicontact/attempt1/upstream-mesh-multicontact-A.json").read_bytes()
raw_b = (ROOT / "target/m1-mesh-multicontact/attempt1/upstream-mesh-multicontact-B.json").read_bytes()
ref = (D / "upstream-mesh-multicontact-gpu.json").read_bytes()
for name, data in (("A", raw_a), ("B", raw_b), ("ref", ref)):
    print(name, len(data), hashlib.sha256(data).hexdigest())
print("A==B", raw_a == raw_b, "A==ref", raw_a == ref, "B==ref", raw_b == ref)

print("\n=== old references sha")
for p in (
    "docs/development-evidence/g10-p5-primitive-capture/upstream-primitive-gpu.json",
    "docs/development-evidence/g10-p5-primitive-branch-coverage/upstream-primitive-branches-gpu.json",
):
    print(p, sha(ROOT / p))

print("\n=== scripts sha (evidence copies) and compare script")
for n in ("capture_mesh_multicontact_gpu.py", "precheck_mesh_multicontact_cpu.py"):
    p = D / "scripts" / n
    print(n, "bytes", p.stat().st_size, "sha", sha(p))
print("compare_primitive_capture.py sha", sha(ROOT / "scripts/compare_primitive_capture.py"))
