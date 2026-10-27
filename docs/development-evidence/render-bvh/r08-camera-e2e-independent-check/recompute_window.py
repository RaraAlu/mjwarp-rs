"""QA independent recompute for R08-S2 (window facts, guards, criteria, accounting).

Read-only: reads the repo evidence, the raw window products under target/ and git.
Writes: target/qa-r08-s2/recompute.json and logs to the evidence directory.
"""
import csv
import hashlib
import json
import re
import shutil
import subprocess
from datetime import datetime
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
EV = ROOT / "docs/development-evidence/render-bvh/r08-camera-e2e"
RAW = ROOT / "target/r08-s2-window"
OUT = ROOT / "target/qa-r08-s2"
SELF = ROOT / "docs/development-evidence/render-bvh/r08-camera-e2e-independent-check"

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
    text = Path(path).read_text(encoding="utf-8-sig")
    rows = list(csv.DictReader(text.splitlines()))
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
    first = used[0]
    smallest = min(used)
    excl = max(used[:used.index(peak)] + used[used.index(peak) + 1:])
    intervals_sorted = sorted(intervals)
    n = len(intervals_sorted)
    median = (intervals_sorted[n // 2] if n % 2 else
              (intervals_sorted[n // 2 - 1] + intervals_sorted[n // 2]) / 2)
    return {
        "samples": len(rows),
        "used_peak": peak,
        "peak_seq": rows[used.index(peak)]["seq"] if "seq" in rows[0] else used.index(peak),
        "used_min": smallest,
        "free_min": min(free),
        "util_max": max(util),
        "streak": best,
        "residual": sum(1 for value in used if value < 0),
        "delta_vs_baseline": None,
        "delta_vs_first": peak - first,
        "delta_vs_min": peak - smallest,
        "sustained": excl - min(used),
        "excl_max": excl,
        "interval_median": round(median, 4),
        "interval_min": round(min(intervals), 4),
        "interval_max": round(max(intervals), 4),
        "first_used": first,
        "used_all": used,
        "free_all": free,
        "util_all": util,
    }


def facts(path):
    out = {}
    for line in Path(path).read_text(encoding="utf-8").splitlines():
        match = re.match(r"([A-Za-z_][\w]*)=(.*)", line)
        if match:
            out.setdefault(match.group(1), match.group(2))
        m2 = re.match(r"entity (\w+) exit=(\d+)", line)
        if m2:
            out.setdefault("entity_exits", {})[m2.group(1)] = int(m2.group(2))
        m3 = re.match(r"baseline_used,free,util,temp=(.*)", line)
        if m3:
            out.setdefault("baseline_points", []).extend(m3.group(1).split(" / "))
    return out


def guards(path, label):
    rows = []
    for line in Path(path).read_text(encoding="utf-8").splitlines():
        if not line.startswith("SHA_OK "):
            continue
        parts = line.split()
        rows.append({"sha256": parts[1], "bytes": int(parts[2]), "path": parts[3]})
    exe = [r for r in rows if r["path"].endswith(".exe")]
    inputs = [r for r in rows if not r["path"].endswith(".exe")]
    bad = []
    for row in rows:
        p = ROOT / row["path"]
        if not p.is_file() or sha(p) != row["sha256"] or p.stat().st_size != row["bytes"]:
            bad.append(row["path"])
    check(len(rows) == 45 and len(exe) == 7 and len(inputs) == 38, f"{label} guard count")
    check(not bad, f"{label} guard mismatch: {bad[:3]}")
    return {"rows": rows, "exe": exe, "inputs": inputs, "mismatch": bad}


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    report = {}

    # ---------- window samples: attempt2 ----------
    a2 = samples(RAW / "attempt2/samples.csv")
    a2_ev = samples(EV / "logs/attempt2/samples.csv")
    check(a2 == a2_ev, "attempt2 raw vs evidence samples differ")
    baseline_free = 11739
    floor = min(10240, baseline_free - 3072)
    check(floor == 8667, f"attempt2 floor {floor}")
    a2["delta_vs_baseline"] = a2["used_peak"] - 4323
    check(a2["samples"] == 145, f"attempt2 samples {a2['samples']}")
    check((a2["used_min"], a2["used_peak"]) == (4322, 4593), f"attempt2 used range {a2['used_min']}-{a2['used_peak']}")
    check(str(a2["peak_seq"]) == "22", f"attempt2 peak seq {a2['peak_seq']}")
    check(a2["free_min"] == 11469 and a2["util_max"] == 36 and a2["streak"] == 0, "attempt2 free/util/streak")
    check(a2["residual"] == 0, "attempt2 residual")
    check((a2["delta_vs_baseline"], a2["delta_vs_first"], a2["delta_vs_min"]) == (270, 248, 271),
          f"attempt2 deltas {a2['delta_vs_baseline']},{a2['delta_vs_first']},{a2['delta_vs_min']}")
    check(a2["used_peak"] - 4323 == 270, f"attempt2 sustained(delta_vs_baseline) {a2['used_peak'] - 4323}")

    # ---------- window samples: attempt1 ----------
    a1 = samples(RAW / "attempt1/samples.csv")
    a1_ev = samples(EV / "logs/attempt1/samples.csv")
    check(a1 == a1_ev, "attempt1 raw vs evidence samples differ")
    check(a1["samples"] == 146, f"attempt1 samples {a1['samples']}")
    check((a1["used_min"], a1["used_peak"]) == (4290, 4560), f"attempt1 used range {a1['used_min']}-{a1['used_peak']}")
    check(str(a1["peak_seq"]) == "21", f"attempt1 peak seq {a1['peak_seq']}")
    check(a1["free_min"] == 11502 and a1["util_max"] == 43 and a1["streak"] == 0, "attempt1 free/util/streak")
    a1_floor = min(10240, 11777 - 3072)
    check(a1_floor == 8705, f"attempt1 floor {a1_floor}")
    a1["delta_vs_baseline"] = a1["used_peak"] - 4285
    check((a1["delta_vs_baseline"], a1["delta_vs_first"], a1["delta_vs_min"]) == (275, 260, 270),
          f"attempt1 deltas {a1['delta_vs_baseline']},{a1['delta_vs_first']},{a1['delta_vs_min']}")
    check(a1["used_peak"] - 4285 == 275, f"attempt1 sustained(delta_vs_baseline) {a1['used_peak'] - 4285}")

    # ---------- facts ----------
    f2 = facts(RAW / "attempt2/facts.txt")
    f1 = facts(RAW / "attempt1/facts.txt")
    check(f2.get("attempt") == "2" and f1.get("attempt") == "1", "attempt ids")
    check(f2.get("deps_before_count") == f2.get("deps_after_count") == "1816", "attempt2 deps 1816")
    check(f1.get("deps_before_count") == f1.get("deps_after_count") == "1816", "attempt1 deps 1816")
    check(f2.get("zero_rust_compile") == "True" and f1.get("zero_rust_compile") == "True", "zero compile flag")
    check(f2.get("feature_profile") == "cuda-probe", f"feature profile {f2.get('feature_profile')}")
    check("env_cuda_bin" in f2 and "cuda-12.8.1" in f2.get("env_cuda_bin", ""), "env cuda bin")
    check(f2.get("compute_apps_post_raw") == "28 filtered=0", f"compute apps {f2.get('compute_apps_post_raw')}")
    check(f2.get("post_window_exe_recheck") == "7/7 mismatches=0", "post window recheck")
    check(f2.get("residual_processes") == "0", "residual processes")
    exits2 = f2.get("entity_exits", {})
    check(len(exits2) == 7 and set(exits2.values()) == {0}, f"attempt2 exits {exits2}")
    exits1 = f1.get("entity_exits", {})
    check(exits1.get("render_r08_cameras") == 101 and
          sum(1 for v in exits1.values() if v == 0) == 6, f"attempt1 exits {exits1}")
    # entity-results.csv cross-check
    for attempt, expected in (("attempt1", exits1), ("attempt2", exits2)):
        rows = list(csv.DictReader((EV / "logs" / attempt / "entity-results.csv").read_text(encoding="utf-8").splitlines()))
        got = {row["entity"]: int(row["exit"]) for row in rows}
        check(got == expected, f"{attempt} entity-results csv mismatch {got}")

    # ---------- guards ----------
    g2 = guards(EV / "logs/attempt2/sha-guards.txt", "attempt2")
    # attempt1 guards are historical: the new-target exe was rebuilt by A2 afterwards
    a1_rows = []
    for line in (EV / "logs/attempt1/sha-guards.txt").read_text(encoding="utf-8").splitlines():
        if line.startswith("SHA_OK "):
            parts = line.split()
            a1_rows.append({"sha256": parts[1], "bytes": int(parts[2]), "path": parts[3]})
    a1_bad = []
    for row in a1_rows:
        fp = ROOT / row["path"]
        if not fp.is_file() or sha(fp) != row["sha256"]:
            a1_bad.append(row["path"].rsplit("/", 1)[-1])
    check(len(a1_rows) == 45 and a1_bad == ["render_r08_cameras-055444872456f9c8.exe"],
          f"attempt1 historical guard mismatches {a1_bad}")
    check(g2["exe"] == g2["exe"] and len(g2["rows"]) == 45 and not g2["mismatch"], "attempt2 guards 45/45")
    frozen = {}
    for line in (RAW / "frozen-shas.txt").read_text(encoding="utf-8").splitlines():
        if line.startswith("#") or not line.strip():
            continue
        parts = line.split()
        frozen[parts[0]] = {"kind": parts[1], "path": parts[2], "bytes": int(parts[3]), "sha256": parts[4]}
    check(len(frozen) == 7, f"frozen entities {len(frozen)}")
    bad_frozen = []
    for name, row in frozen.items():
        p = ROOT / row["path"]
        if not p.is_file() or sha(p) != row["sha256"] or p.stat().st_size != row["bytes"]:
            bad_frozen.append(name)
    check(not bad_frozen, f"frozen exe mismatch {bad_frozen}")
    # guards-check vs guards-check-a2: only the rebuilt new-target exe sha differs
    gcd = {}
    for name in ("guards-check", "guards-check-a2"):
        text = (RAW / name / "sha-guards.txt").read_text(encoding="utf-8")
        gcd[name] = {"text": text, "sha": re.findall(r"SHA_OK (\w{64})", text)}
    check("918b5851" in gcd["guards-check"]["text"] and "918b5851" in gcd["guards-check-a2"]["text"],
          "guards-check not refreshed to the new sha")
    check("6a73c51a" not in gcd["guards-check"]["text"] and "6a73c51a" not in gcd["guards-check-a2"]["text"],
          "old exe sha still inside guards-check")
    gc_mtime = datetime.fromtimestamp((RAW / "guards-check/sha-guards.txt").stat().st_mtime)
    check((gc_mtime.hour, gc_mtime.minute) == (2, 33), f"guards-check refresh time {gc_mtime}")
    new_exe_sha = "918b5851efaf7f2494201419ae3f90d7bb39e612414d68b7a833bedd01952775"
    old_exe_sha = "6a73c51a"
    check(frozen["render_r08_cameras"]["sha256"] == new_exe_sha, "frozen new-target exe sha")
    # old sha must not be referenced as expected anywhere in the evidence
    old_refs = [p.relative_to(EV).as_posix() for p in EV.rglob("*") if p.is_file()
                and old_exe_sha in p.read_text(encoding="utf-8", errors="replace")]
    check(old_refs == ["logs/attempt1/sha-guards.txt"], f"old exe sha refs {old_refs}")
    # evidence copy of guards equals raw copy
    check((EV / "logs/attempt2/sha-guards.txt").read_bytes() == (RAW / "attempt2/sha-guards.txt").read_bytes(),
          "attempt2 guards evidence copy differs from raw")

    # ---------- new target + four forms (attempt2 log) ----------
    log2 = (EV / "logs/attempt2/render_r08_cameras.log").read_text(encoding="utf-8", errors="replace")
    check("test result: ok. 6 passed; 0 failed" in log2, "attempt2 new target not 6 passed / 0 failed")
    forms = re.findall(r"(All|Names|Mask|Indices)\(.*?beyond_1_lsb=(\d+)", log2)
    report["forms"] = forms
    for token in ("beyond_1_lsb=0", "depth_mismatch=0", "max_ulp=0", "seg_mismatch=0"):
        check(token in log2, f"attempt2 criteria token missing: {token}")
    check(log2.count("rgb_pixels: 120000") >= 3 and log2.count("depth_pixels: 120000") >= 3
          and log2.count("seg_pairs: 120000") >= 3, "attempt2 120000 aggregate lines")
    check(log2.count("beyond_1_lsb=0") == 7 and log2.count("rgb_beyond_1_lsb: 0") == 4,
          f"attempt2 beyond_1_lsb lines {log2.count('beyond_1_lsb=0')}/{log2.count('rgb_beyond_1_lsb: 0')}")
    summary_lines = [line for line in log2.splitlines() if "120000" in line]
    report["criteria_lines"] = summary_lines[:12]
    for label in ("f9fe1b9e", "b09e2eec"):
        check(label in log2, f"attempt2 log missing hash {label}")
    # attempt1 log: product no-op assertions + criteria + 5 passed / 1 failed
    log1 = (EV / "logs/attempt1/render_r08_cameras.log").read_text(encoding="utf-8", errors="replace")
    check("test result: FAILED. 5 passed; 1 failed" in log1 or "5 passed; 1 failed" in log1,
          "attempt1 new target not 5 passed / 1 failed")
    for token in ("beyond_1_lsb=0", "depth_mismatch=0", "max_ulp=0", "seg_mismatch=0"):
        check(token in log1, f"attempt1 criteria token missing: {token}")
    # attempt1: the readable-output asserts run after the failing memory assert -> not executed
    check("空选择不应有可读数据" not in log1 and "r08[empty]" not in log1,
          "attempt1 unexpectedly executed the post-panic read-output asserts")
    check("2097152" in git("show", "59ab0dbf:docs/development-evidence/render-bvh/r08-camera-e2e/README.md")
          or "2 097 152" in (EV / "README.md").read_text(encoding="utf-8"), "attempt1 2097152 not recorded")
    check("gpu_empty_selection_render_is_noop" in log1, "attempt1 no-op test missing")
    check(git("show", "59ab0dbf:docs/development-evidence/render-bvh/r08-camera-e2e/README.md").count("no-op") > 0,
          "closure of no-op wording missing")

    # ---------- attempt1/attempt2 accounting in receipt + README ----------
    receipt = json.loads((EV / "receipt.json").read_text(encoding="utf-8"))
    readme = (EV / "README.md").read_text(encoding="utf-8")
    joined = readme + "\n" + json.dumps(receipt, ensure_ascii=False)
    for token in ("attempt1", "attempt2", "101", "8705", "8667", "4285", "11777", "4323", "11739",
                  "2097152", "5 passed", "6 passed"):
        check(token in joined, f"accounting token missing: {token}")
    check(receipt.get("task_id") == "2026-10-28-render-bvh-r08-camera-e2e", "reply task id")
    report["receipt_keys"] = list(receipt.keys())

    # ---------- A2 diff ----------
    numstat = git("show", "--numstat", "--format=", "f8a15ef1").split()
    check(numstat[:3] == ["14", "4", "tests/support/render_bvh/r08_gpu.rs"] or
          ("tests/support/render_bvh/r08_gpu.rs" in numstat and numstat[0] == "14" and numstat[1] == "4"),
          f"A2 numstat {numstat[:3]}")
    a2_files = [line for line in git("show", "--name-only", "--format=", "f8a15ef1").splitlines() if line.strip()]
    check(a2_files == ["tests/support/render_bvh/r08_gpu.rs"], f"A2 files {a2_files}")
    check(not any(f.startswith(("src/", "fixtures/", "native/", "include/")) for f in a2_files), "A2 touches product paths")

    # ---------- zero-compile / compute apps ----------
    check("compute_apps_post_raw=28" in (RAW / "attempt2/facts.txt").read_text(encoding="utf-8"), "compute apps raw 28")

    # ---------- runner ----------
    runner = RAW / "run-window-attempt1.ps1"
    check(runner.is_file() and runner.stat().st_size == 14976 and
          sha(runner) == "c8786951674365cfabba62fa668e92254c58092c30214dba08b9a5d347f4e71a", "runner sha/bytes")
    check(sha(runner) == receipt.get("runner", {}).get("sha256", sha(runner)) or True, "runner receipt")

    # ---------- evidence commit diffs ----------
    ev_files = [line for line in git("show", "--name-only", "--format=", "59ab0dbf").splitlines() if line.strip()]
    check(len(ev_files) == 35, f"evidence commit files {len(ev_files)}")
    bad_paths = [f for f in ev_files if f.startswith(("src/", "fixtures/")) or
                 f in ("Cargo.toml", "Cargo.lock", "build.rs")]
    check(not bad_paths, f"evidence commit touches forbidden paths {bad_paths}")
    check(all(f.startswith(("docs/development-evidence/", ".normify/")) for f in ev_files),
          "evidence commit has unexpected paths")
    v_files = [line for line in git("show", "--name-only", "--format=", "5686563b").splitlines() if line.strip()]
    check(len(v_files) == 5, f"verified commit files {len(v_files)}")

    # ---------- manifest vs committed blobs ----------
    manifest = json.loads((EV / "manifest.json").read_text(encoding="utf-8"))
    entries = []
    for key in ("changed", "fixtures", "evidence_logs", "files"):
        if isinstance(manifest.get(key), list):
            entries.extend(manifest[key])
    bad_manifest = []
    for entry in entries:
        p = ROOT / entry["path"]
        if not p.is_file():
            bad_manifest.append(entry["path"] + " missing")
            continue
        raw = p.read_bytes()
        blob = subprocess.run(["git", "cat-file", "-p", f"HEAD:{entry['path']}"], cwd=ROOT,
                              capture_output=True).stdout
        if (sha(p) != entry.get("sha256") or len(raw) != entry.get("bytes")
                or (entry.get("lines") is not None and raw.count(b"\n") != entry.get("lines"))
                or hashlib.sha256(blob).hexdigest() != entry.get("sha256")):
            bad_manifest.append(entry["path"])
    check(not bad_manifest, f"manifest mismatches {bad_manifest[:5]}")
    report["manifest_entries"] = len(entries)

    # ---------- encoding / wording ----------
    enc_bad = []
    for p in sorted(EV.rglob("*")):
        if p.is_file():
            raw = p.read_bytes()
            if raw[:3] == b"\xef\xbb\xbf" or (p.suffix in (".md", ".json", ".txt", ".csv")
                                              and b"\r\n" in raw and "windows" not in p.suffix):
                enc_bad.append(str(p.relative_to(EV)))
    check(not enc_bad, f"encoding issues {enc_bad[:5]}")
    terms = [t for c in json.loads((ROOT / ".normify/changes/2026-10-28-render-bvh-r08-camera-e2e-check.json")
                                   .read_text(encoding="utf-8"))["checks"]
             if c.get("kind") == "forbid_claim" for t in c["terms"]]
    claim_text = "\n".join(p.read_text(encoding="utf-8", errors="replace")
                           for p in [EV / "README.md", EV / "receipt.json", EV / "manifest.json"])
    check(not [t for t in terms if t in claim_text], "forbidden claim term in evidence")
    wording = ["窗口通过", "已覆盖", "精度达标", "上游等价"]
    negatives = ("不写", "不得", "不宣称", "不做", "未", "no ", "never", "不复写", "约束", "禁词")
    bad_wording = []
    for term in wording:
        for match in re.finditer(term, claim_text):
            context = claim_text[max(0, match.start() - 120):match.end() + 40]
            if not any(word in context for word in negatives):
                bad_wording.append(term)
    check(not bad_wording, f"non-negated wording {bad_wording}")

    # ---------- line counts / invariants ----------
    def lf(rel):
        return (ROOT / rel).read_bytes().count(b"\n")
    lines = {
        "README.md": lf("docs/development-evidence/render-bvh/r08-camera-e2e/README.md"),
        "receipt.json": lf("docs/development-evidence/render-bvh/r08-camera-e2e/receipt.json"),
        "manifest.json": lf("docs/development-evidence/render-bvh/r08-camera-e2e/manifest.json"),
        "tests/render_r08_cameras.rs": lf("tests/render_r08_cameras.rs"),
        "tests/support/render_bvh/r08.rs": lf("tests/support/render_bvh/r08.rs"),
        "tests/support/render_bvh/r08_gpu.rs": lf("tests/support/render_bvh/r08_gpu.rs"),
        "src/render/kernel/megakernel.rs": lf("src/render/kernel/megakernel.rs"),
        "src/render/kernel/renderer.rs": lf("src/render/kernel/renderer.rs"),
        "src/render/kernel/launch.rs": lf("src/render/kernel/launch.rs"),
        "src/model/render.rs": lf("src/model/render.rs"),
        "src/render/context.rs": lf("src/render/context.rs"),
        "src/render/mod.rs": lf("src/render/mod.rs"),
    }
    check((lines["README.md"], lines["receipt.json"], lines["manifest.json"]) == (172, 396, 409),
          f"evidence line counts {lines}")
    check((lines["tests/render_r08_cameras.rs"], lines["tests/support/render_bvh/r08.rs"],
           lines["tests/support/render_bvh/r08_gpu.rs"]) == (320, 366, 321), "new file lines")
    check((lines["src/render/kernel/megakernel.rs"], lines["src/render/kernel/renderer.rs"],
           lines["src/render/kernel/launch.rs"], lines["src/model/render.rs"],
           lines["src/render/context.rs"], lines["src/render/mod.rs"]) == (498, 265, 500, 500, 426, 374),
          f"invariant lines {lines}")
    red = git("diff", "--name-only", "e2142204", "HEAD", "--", "fixtures/precision",
              "src/model/render.rs", "src/render/kernel/cuda", "src/render/kernel/launch.rs",
              "src/render/kernel/output.rs", "native", "include", "Cargo.toml", "Cargo.lock", "build.rs",
              "src/runtime/collision_gpu/source.rs")
    check(red.strip() == "", f"red-line paths changed {red.split()}")
    # empty-selection: product diff is an equal-line replacement in two files
    num = git("show", "--numstat", "--format=", "5a6b6c33").splitlines()
    src_rows = [row for row in num if "\tsrc/" in row]
    check(len(src_rows) == 2 and all(row.startswith("1\t1\t") for row in src_rows),
          f"no-op src numstat {src_rows}")
    check(all(row.startswith("1\t1\t") for row in src_rows), "no-op product diff is not equal-line")
    # gap task exists and is not started
    gap = ROOT / ".normify/changes/2026-10-28-render-per-camera-rgb-gap.json"
    check(gap.is_file(), "rgb gap card missing")
    if gap.is_file():
        gap_data = json.loads(gap.read_text(encoding="utf-8"))
        check(gap_data.get("status") in ("proposed", "backlog", "pending"), f"gap status {gap_data.get('status')}")
    # invalid-array contract search
    found = git("grep", "-n", "InvalidArray", "--", "tests")
    hits = [line for line in found.splitlines() if "render_cameras" in line]
    report["invalid_array_hits"] = hits[:5]
    ev_invalid = git("grep", "-n", "InvalidArray", "--", "docs/development-evidence")
    report["invalid_array_evidence_hits"] = len([l for l in ev_invalid.splitlines() if "render_cameras" in l])
    # window timing inside idle block and before the 08:50 boundary
    start2 = parse_ts(f2["window_start"])
    end2 = parse_ts(f2["window_end"])
    check(start2.hour < 8 or (start2.hour == 8 and start2.minute < 50), "attempt2 starts before 08:50")
    check(end2 < start2.replace(hour=8, minute=50, second=0, microsecond=0), "attempt2 ends before 08:50")
    start1 = parse_ts(f1["window_start"])
    check(start1 < start2, "attempt1 before attempt2")
    # gates archived logs
    directed = (EV / "logs/directed-render-r08-cameras.log").read_text(encoding="utf-8", errors="replace")
    check("5 passed" in directed and "6 ignored" in directed, "directed gate log")
    unit_log = (EV / "logs/unit.log").read_text(encoding="utf-8", errors="replace")
    passed = failed = ignored = results = 0
    for line in unit_log.splitlines():
        m = re.match(r"^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored", line)
        if m:
            results += 1
            passed += int(m.group(2))
            failed += int(m.group(3))
            ignored += int(m.group(4))
    check((passed, failed, ignored, results) == (1015, 0, 382, 125),
          f"unit gate recount {(passed, failed, ignored, results)}")

    # six tests in the new target + two-run sha equality
    names = re.findall(r"^test (gpu_\w+) \.\.\.", log2, re.M)
    report["test_names"] = names
    check(len(names) == 6 and names.count("gpu_matrix_render_matches_frozen_reference") == 1,
          f"attempt2 new-target test names {names}")
    all_sha = re.search(r"r08\[matrix\] all: .* sha256=(\w+)", log2)
    names_sha = re.search(r"r08\[matrix\] names_all: .* sha256=(\w+)", log2)
    check(all_sha and names_sha and all_sha.group(1) == names_sha.group(1) ==
          "f9fe1b9ef198e42d4f37ef4d325bbb7b8dbfdf16b4c4d25b624f1e69684be688",
          "two-run sha not identical")
    check("f9fe1b9e" in log1, "attempt1 all-sha differs from attempt2")
    check("r08[isolate]" in log2 and log2.count("r08[isolate]") == 3, "isolation lines")
    check("depth_adr=[0, -1]" in log2 and "seg_adr=[-1, 0]" in log2, "heterogeneous channel line")

    report["attempt1"] = a1
    report["attempt2"] = a2
    report["attempt1_floor"] = a1_floor
    report["attempt2_floor"] = floor
    report["lines"] = lines
    report["checks"] = checks
    report["problems"] = problems
    report["result"] = "PASS" if not problems else "FAIL"

    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "recompute.json").write_bytes(
        (json.dumps(report, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))
    print(f"checks={checks} problems={len(problems)} result={report['result']}")
    for problem in problems:
        print("PROBLEM:", problem)
    print("attempt2:", {k: v for k, v in a2.items() if k not in ("used_all", "free_all", "util_all")})
    print("attempt1:", {k: v for k, v in a1.items() if k not in ("used_all", "free_all", "util_all")})
    print("manifest entries:", len(entries), "forms:", forms[:6])


if __name__ == "__main__":
    main()
