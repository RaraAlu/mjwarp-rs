"""QA independent recompute for R10-S2 (window facts, guards, criteria semantics, red lines).

Read-only. Writes target/qa-r10-s2/recompute.json.
"""
import csv
import hashlib
import json
import re
import subprocess
from datetime import datetime
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
EV = ROOT / "docs/development-evidence/render-bvh/r10-s2-multi-world-e2e"
RAW = ROOT / "target/r10-s2-window"
OUT = ROOT / "target/qa-r10-s2"
BIND_HEAD = "40e9238f219daf980c48af124bfe63e585aa0f42"
problems = []
checks = 0


def check(cond, label):
    global checks
    checks += 1
    if not cond:
        problems.append(label)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def git(*args):
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True,
                          encoding="utf-8", errors="replace").stdout


def parse_ts(value):
    main, _, off = value.partition("+")
    head, _, frac = main.partition(".")
    return datetime.fromisoformat(head + "." + (frac + "000000")[:6] + "+" + off)


def samples(path):
    rows = list(csv.DictReader(Path(path).read_text(encoding="utf-8-sig").splitlines()))
    stamps = [parse_ts(row["timestamp"]) for row in rows]
    intervals = [(stamps[i + 1] - stamps[i]).total_seconds() for i in range(len(stamps) - 1)]
    used = [int(row["used_mib"]) for row in rows]
    free = [int(row["free_mib"]) for row in rows]
    util = [int(row["util_percent"]) for row in rows]
    peak = max(used)
    streak = best = 0
    for value in util:
        streak = streak + 1 if value >= 80 else 0
        best = max(best, streak)
    ordered = sorted(intervals)
    n = len(ordered)
    median = ordered[n // 2] if n % 2 else (ordered[n // 2 - 1] + ordered[n // 2]) / 2
    return {"samples": len(rows), "used_peak": peak,
            "peak_seq": int(rows[used.index(peak)]["seq"]) if "seq" in rows[0] else used.index(peak),
            "used_min": min(used), "free_min": min(free), "util_max": max(util),
            "streak": best, "first_used": used[0],
            "interval_median": round(median, 4), "interval_min": round(min(intervals), 4),
            "interval_max": round(max(intervals), 4)}


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    report = {}

    # ---------- attempt2-run window numbers ----------
    a2 = samples(RAW / "attempt2-run/samples.csv")
    check(a2 == samples(EV / "attempt2-run/samples.csv"), "attempt2-run raw vs evidence samples differ")
    floor = min(10240, 12780 - 3072)
    check(floor == 9708, f"attempt2 floor {floor}")
    check(a2["samples"] == 148, f"attempt2 samples {a2['samples']}")
    check((a2["used_min"], a2["used_peak"], a2["peak_seq"]) == (3283, 3556, 87),
          f"attempt2 used range/seq {a2['used_min']}-{a2['used_peak']}@{a2['peak_seq']}")
    check(a2["free_min"] == 12506 and a2["util_max"] == 9 and a2["streak"] == 0, "attempt2 free/util/streak")
    deltas = (a2["used_peak"] - 3286, a2["used_peak"] - a2["first_used"], a2["used_peak"] - a2["used_min"])
    check(deltas == (270, 90, 273), f"attempt2 deltas {deltas}")

    # ---------- attempt1 numbers ----------
    a1 = samples(EV / "attempt1/samples.csv")
    floor1 = min(10240, 11861 - 3072)
    check(floor1 == 8789, f"attempt1 floor {floor1}")
    check(a1["samples"] == 156 and a1["used_peak"] == 4492 and a1["peak_seq"] == 92,
          f"attempt1 {a1['samples']}/{a1['used_peak']}@{a1['peak_seq']}")
    check(a1["free_min"] == 11570 and a1["util_max"] == 35 and a1["streak"] == 0, "attempt1 free/util/streak")
    deltas1 = (a1["used_peak"] - 4201, a1["used_peak"] - a1["first_used"], a1["used_peak"] - a1["used_min"])
    check(deltas1 == (291, 270, 295), f"attempt1 deltas {deltas1}")

    # ---------- facts cross-check ----------
    def facts(path):
        out = {}
        for line in Path(path).read_text(encoding="utf-8").splitlines():
            m = re.match(r"entity (\w+) exit=(\d+)", line)
            if m:
                out.setdefault("entity_exits", {})[m.group(1)] = int(m.group(2))
                continue
            m = re.match(r"^([A-Za-z_][\w]*)=(.*)$", line)
            if m and "=" not in m.group(2):
                out.setdefault(m.group(1), m.group(2))
            else:
                for key, value in re.findall(r"([A-Za-z_][\w]*)=([^\s]+)", line):
                    out.setdefault(key, value)
        return out

    f2 = facts(RAW / "attempt2-run/facts.txt")
    f1 = facts(EV / "attempt1/facts.txt")
    check(f2.get("attempt") == "2" and f1.get("attempt") == "1", "attempt ids")
    check(f2.get("deps_before_count") == f2.get("deps_after_count") == "1823", "attempt2 deps 1823")
    check(f2.get("deps_before_max_write") == f2.get("deps_after_max_write") == "10/28/2026 04:23:44",
          f"attempt2 max_write unchanged {f2.get('deps_before_max_write')!r}/{f2.get('deps_after_max_write')!r}")
    check(f1.get("deps_before_count") == f1.get("deps_after_count") == "1821", "attempt1 deps 1821")
    check(f2.get("zero_rust_compile") == "True" and f1.get("zero_rust_compile") == "True", "zero compile flags")
    check(f2.get("feature_profile") == "cuda-probe", f"feature profile {f2.get('feature_profile')}")
    check(f2.get("compute_apps_post_raw") == "29" and f2.get("filtered") == "0",
          f"compute apps {f2.get('compute_apps_post_raw')}/{f2.get('filtered')}")
    check(f2.get("post_window_exe_recheck") == "8/8" and f2.get("mismatches") == "0",
          f"post-window exe recheck {f2.get('post_window_exe_recheck')}/{f2.get('mismatches')}")
    check(f2.get("baseline_free_mib") == "12780", f"baseline_free {f2.get('baseline_free_mib')}")
    check(f2.get("residual_processes") == "0", "residual processes")
    check(f2.get("attempt_valid_rounds") == "2", f"valid rounds {f2.get('attempt_valid_rounds')}")
    check(f2.get("samples") == "148" and f2.get("used_peak_mib") == "3556" and f2.get("peak_seq") == "87",
          "facts samples/peak")
    exits2 = f2.get("entity_exits", {})
    check(len(exits2) == 8 and set(exits2.values()) == {0}, f"attempt2 exits {exits2}")
    exits1 = f1.get("entity_exits", {})
    check(exits1.get("render_r10_multi_world") == 101 and sum(1 for v in exits1.values() if v == 0) == 7,
          f"attempt1 exits {exits1}")
    rows2 = list(csv.DictReader((RAW / "attempt2-run/entity-results.csv").read_text(encoding="utf-8").splitlines()))
    check({row["entity"]: int(row["exit"]) for row in rows2} == exits2, "attempt2 entity csv vs facts")
    rows1 = list(csv.DictReader((EV / "attempt1/entity-results.csv").read_text(encoding="utf-8").splitlines()))
    check({row["entity"]: int(row["exit"]) for row in rows1} == exits1, "attempt1 entity csv vs facts")
    # body wall
    wall = (parse_ts(f2["entity_end"]) - parse_ts(f2["entity_start"])).total_seconds()
    check(abs(wall - 41.7) < 0.05, f"attempt2 body wall {wall}")
    window_span = (parse_ts(f2["window_end"]) - parse_ts(f2["window_start"])).total_seconds()
    check(abs(window_span - 44.1) < 0.1, f"attempt2 window span {window_span}")

    # ---------- attempt2-exit9 archive untouched ----------
    exit9 = RAW / "attempt2"
    check(sha(exit9 / "facts.txt") == "069d142d07808ecd74a4311993d4aebe619053b08a825b2ab4c8e8c08e02d7a6"
          and (exit9 / "facts.txt").stat().st_size == 61, "exit9 facts sha/bytes")
    check(sha(exit9 / "guard.log") == "6c5acd894d9d4531acc23cc3dcc8a374137b92499d7d1d60d01350f4489786ca"
          and (exit9 / "guard.log").stat().st_size == 51, "exit9 guard.log sha/bytes")
    check(sha(EV / "attempt2-exit9/facts.txt") == sha(exit9 / "facts.txt"), "exit9 evidence copy differs")
    check(sha(EV / "attempt2-exit9/guard.log") == sha(exit9 / "guard.log"), "exit9 guard evidence copy differs")
    check("precheck_abort" in (exit9 / "facts.txt").read_text(encoding="utf-8"), "exit9 facts content")

    # ---------- guards: 46/46 and post-window 8/8 ----------
    rows = []
    for line in (RAW / "attempt2-run/sha-guards.txt").read_text(encoding="utf-8").splitlines():
        if line.startswith("SHA_OK "):
            parts = line.split()
            rows.append({"sha256": parts[1], "bytes": int(parts[2]), "path": parts[3]})
    exe = [r for r in rows if r["path"].endswith(".exe")]
    inputs = [r for r in rows if not r["path"].endswith(".exe")]
    bad = [r["path"] for r in rows if not (ROOT / r["path"]).is_file()
           or sha(ROOT / r["path"]) != r["sha256"] or (ROOT / r["path"]).stat().st_size != r["bytes"]]
    check(len(rows) == 46 and len(exe) == 8 and len(inputs) == 38, f"guard counts {len(rows)}/{len(exe)}/{len(inputs)}")
    check(not bad, f"guard mismatches {bad[:3]}")
    check(sha(RAW / "guards.json") == "0c849e67c857265e088e6c4a4ccf6de88053f3ba5776f6044a88dfe982cc86e7",
          "guards.json sha")
    runner = RAW / "run-window-attempt1.ps1"
    check(runner.stat().st_size == 14968
          and sha(runner) == "4d636c39731145932a2057477c800e218090cb884542274e3a52e0fe28ce2438",
          "runner sha/bytes")
    frozen_rows = [line.split() for line in (RAW / "frozen-shas.txt").read_text(encoding="utf-8").splitlines()
                   if line.strip() and not line.startswith("#")]
    frozen_bad = []
    frozen_exe = 0
    for parts in frozen_rows:
        digest, size, path = parts[0], int(parts[1]), parts[2]
        if path.endswith(".exe"):
            frozen_exe += 1
        fp = ROOT / path
        if not fp.is_file() or fp.stat().st_size != size or sha(fp) != digest:
            frozen_bad.append(path)
    check(frozen_exe == 8 and not frozen_bad,
          f"frozen shas exe={frozen_exe} bad={frozen_bad[:3]}")
    guards_json = json.loads((RAW / "guards.json").read_text(encoding="utf-8"))
    gj_exe = {row["path"]: (row["bytes"], row["sha256"]) for row in guards_json["frozen_exes"]}
    gj_inputs = {row["path"]: (row["bytes"], row["sha256"]) for row in guards_json["guard_inputs"]}
    guard_rows = {r["path"]: (r["bytes"], r["sha256"]) for r in rows}
    check(gj_exe == {r["path"]: (r["bytes"], r["sha256"]) for r in exe}, "guards.json exe set mismatch")
    check(gj_inputs == {r["path"]: (r["bytes"], r["sha256"]) for r in inputs}, "guards.json input set mismatch")
    check(len(gj_exe) == 8 and len(gj_inputs) == 38, "guards.json counts")
    gc2 = (RAW / "guards-check-r2/sha-guards.txt").read_text(encoding="utf-8")
    gc2_rows = [line for line in gc2.splitlines() if line.startswith("SHA_OK ")]
    check(len(gc2_rows) == 46, f"guards-check-r2 rows {len(gc2_rows)}")
    check(all(any(r["sha256"] in line and r["path"] in line for line in gc2_rows) for r in rows),
          "guards-check-r2 does not match the window guard list")

    # ---------- criteria semantics in the new tests (no loosening) ----------
    test_src = (ROOT / "tests/render_r10_multi_world.rs").read_text(encoding="utf-8")
    for token in ("assert_eq!(diff.mismatched, 0", "diff.max_ulp <= MAX_DEPTH_ULP",
                  "diff_seg(seg, camera).expect(\"seg diff\").mismatched, 0",
                  "&depth_a[depth_stride..]", "&depth1[..]",
                  "stats.beyond_one_lsb_pixels, 0", "&rgb_a[stride..], &rgb1[..]",
                  "两次运行 depth SHA 相同", "两次运行 seg SHA 相同", "两次运行 RGB SHA 相同",
                  "world1 场景位移不得改变 world0", "world1 场景位移后必须全 miss",
                  "world1 场景位移后分割必须全背景"):
        check(token in test_src, f"criteria token missing: {token}")
    check("recipe_depth_seg()" in test_src and "s7b1::REFERENCE" not in test_src,
          "attempt1 recipe mismatch not fixed in test source")
    diff = git("diff", f"{BIND_HEAD}^..{BIND_HEAD}", "--", "tests/render_r10_multi_world.rs")
    added = [line for line in diff.splitlines() if line.startswith("+") and not line.startswith("+++")]
    check(not [line for line in added if "mismatched, 0" in line or "max_ulp" in line],
          "assertions were changed by the fix commit (expected reference-only change)")
    log2 = (RAW / "attempt2-run/logs/render_r10_multi_world.log").read_text(encoding="utf-8")
    check("2 passed; 0 failed" in log2 and "gpu_nworld_two_depth_seg_per_world_cameras ... ok" in log2
          and "gpu_nworld_two_rgb_per_world_cameras ... ok" in log2, "attempt2 new entity log")
    log1 = (EV / "attempt1/logs/render_r10_multi_world.log").read_text(encoding="utf-8")
    check("0 passed; 2 failed" in log1, "attempt1 new entity log")
    check("19074" in log1 and "Capacity { slot: 0, required: 56, actual: 28 }" in log1,
          "attempt1 failure root causes recorded")

    # ---------- evidence receipts / manifest / wording ----------
    receipt = json.loads((EV / "receipt.json").read_text(encoding="utf-8"))
    check(receipt.get("task_id") == "2026-10-28-render-bvh-r10-s2-multi-world", "slice receipt task_id")
    blob = json.dumps(receipt, ensure_ascii=False)
    check(BIND_HEAD in blob, "slice receipt does not bind 40e9238f")
    readme = (EV / "README.md").read_text(encoding="utf-8")
    for token in ("attempt1", "attempt2-exit9", "attempt2-run", "9708", "8789", "148", "3556", "12506",
                  "270/90/273", "46/46", "8/8", "1823", "非验证窗", "backlog",
                  "2026-10-28-render-per-world-geom-light-gap", "render.cu"):
        check(token in readme or token in blob, f"evidence token missing: {token}")
    check("上游 nworld>1" in readme and "不对上游 nworld>1 做任何数值结论" in readme, "upstream nworld>1 caveat")
    observations = []
    if "未覆盖" not in readme + blob:
        observations.append("O4: '未覆盖' literal missing although the README states no numeric conclusion for upstream nworld>1 (semantic caveat present)")
    else:
        observations.append("O4: none")
    wording = ["窗口通过", "已覆盖", "精度达标", "上游等价", "双平台等价"]
    negatives = ("不写", "不得", "不宣称", "不做", "未", "no ", "never", "约束", "禁词", "不")
    bad_wording = []
    for term in wording:
        for match in re.finditer(term, readme + blob):
            context = (readme + blob)[max(0, match.start() - 120):match.end() + 40]
            if not any(word in context for word in negatives):
                bad_wording.append(term)
    check(not bad_wording, f"non-negated wording {bad_wording}")
    terms = [t for c in json.loads((ROOT / ".normify/changes/2026-10-28-render-bvh-r10-s2-check.json")
                                   .read_text(encoding="utf-8"))["checks"]
             if c.get("kind") == "forbid_claim" for t in c["terms"]]
    check(not [t for t in terms if t in readme + blob], "forbidden claim terms present")
    # manifest.txt identities vs evidence copies
    manifest_bad = []
    for line in (EV / "manifest.txt").read_text(encoding="utf-8").splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        digest, size, rel = line.split()
        p = EV / rel
        p = p if p.exists() else ROOT / rel
        if not p.is_file() or sha(p) != digest or p.stat().st_size != int(size):
            manifest_bad.append(rel)
    self_entry = [name for name in manifest_bad if name == "manifest.txt"]
    manifest_bad = [name for name in manifest_bad if name != "manifest.txt"]
    check(not manifest_bad, f"manifest.txt mismatches {manifest_bad[:3]}")
    if self_entry:
        observations.append("O2: manifest.txt lists itself with a stale size (8325 vs current 8286); self-referential row cannot verify")
    # attempt1 copies equal raw attempt1 products
    copy_bad = [name for name in ("facts.txt", "samples.csv", "baseline.txt", "sha-guards.txt")
                if sha(EV / "attempt1" / name) != sha(RAW / "attempt1" / name)]
    check(not copy_bad, f"attempt1 evidence copies differ {copy_bad}")

    # ---------- red lines / line counts ----------
    lines = {rel: (ROOT / rel).read_bytes().count(b"\n") for rel in (
        "src/render/context/camera_output.rs", "src/model/render.rs",
        "src/render/kernel/launch.rs", "src/render/kernel/cast.rs",
        "src/render/kernel/cuda/render.cu", "src/render/kernel/megakernel.rs",
        "tests/render_r10_multi_world.rs")}
    check(lines == {"src/render/context/camera_output.rs": 499, "src/model/render.rs": 500,
                    "src/render/kernel/launch.rs": 500, "src/render/kernel/cast.rs": 494,
                    "src/render/kernel/cuda/render.cu": 499, "src/render/kernel/megakernel.rs": 491,
                    "tests/render_r10_multi_world.rs": 298}, f"line counts {lines}")
    red = git("diff", "--name-only", f"{BIND_HEAD}..HEAD", "--",
              "src/render/context/camera_output.rs", "src/model/render.rs",
              "src/render/kernel/launch.rs", "src/render/kernel/cast.rs",
              "src/render/kernel/cuda", "fixtures", "Cargo.toml", "Cargo.lock")
    check(red.strip() == "", f"red-line paths changed {red.split()}")
    source = (ROOT / "src/runtime/collision_gpu/source.rs").read_text(encoding="utf-8")
    check("DEVICE_SOURCES" in source, "collision device source file missing")
    tests_text = git("grep", "-n", "DEVICE_SOURCES.len(), 28", "--", "src", "tests")
    check(tests_text.strip() != "", "DEVICE_SOURCES=28 anchor missing")
    # 48ea01a2 included untracked registration cards (observation only)
    verified_files = [line for line in git("show", "--name-only", "--format=", "48ea01a2").splitlines()
                      if line.strip()]
    extra_cards = [f for f in verified_files if f.endswith(".json") and
                   any(k in f for k in ("r09-s2-check", "r09-s2-lighting-kernel", "r10-s2-check",
                                        "per-camera-rgb-gap"))]
    report["verified_extra_cards"] = extra_cards
    check(len(extra_cards) == 4, f"expected 4 registration cards in 48ea01a2, got {extra_cards}")

    # ---------- archived gate numbers ----------
    unit = (EV / ".." / "r10-s2-multi-world-e2e").resolve()
    gate_runs = ROOT / ".normify/runs/2026-10-28-render-bvh-r10-s2-multi-world"
    if gate_runs.is_dir():
        unit_json = json.loads((gate_runs / "unit.json").read_text(encoding="utf-8"))
        tests = unit_json.get("tests") or {}
        check(tests.get("passed") == 1040 and tests.get("failed") == 0, f"gate unit json {tests}")

    observations.append("O1: 48ea01a2 used git add -A '.normify' and included 4 registration cards (contents unchanged; Arch accepted)") if report.get("verified_extra_cards") else None
    observations.append("O3: README says the attempt1 log shows mismatched=19074, the log actually shows assertion left:19074 right:0") if "mismatched=19074" in readme else None
    report.update({"observations": observations, "attempt1": a1, "attempt2": a2, "deltas": deltas, "deltas1": deltas1,
                   "floor": floor, "floor1": floor1, "guards": {"rows": len(rows), "exe": len(exe),
                   "inputs": len(inputs), "bad": bad}, "lines": lines,
                   "checks": checks, "problems": problems,
                   "result": "PASS" if not problems else "FAIL"})
    (OUT / "recompute.json").write_bytes(
        (json.dumps(report, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))
    print(f"checks={checks} problems={len(problems)} result={report['result']}")
    for problem in problems:
        print("PROBLEM:", problem)
    print("attempt2:", a2)
    print("attempt1:", a1)


if __name__ == "__main__":
    main()
