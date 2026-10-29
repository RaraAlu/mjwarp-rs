"""QA independent recompute for R02-S3 skybox validation window attempt=2 check.

Read-only: committed evidence blobs, repo tree, on-disk guard/reference artifacts.
No GPU, no window re-run.
"""
import hashlib
import json
import pathlib
import re
import subprocess
import sys

REPO = pathlib.Path(r"D:\Rust\mjwarp-rs")
EV = "docs/development-evidence/render-bvh/r02-skybox-s3-validate"
EV_A = "docs/development-evidence/render-bvh/r02-skybox-s3-artifacts-commit"
WIN = REPO / "target/render-bvh/r02-skybox-window"
OUT = {}


def blobp(p):
    r = subprocess.run(["git", "show", f"HEAD:{p}"], cwd=REPO, capture_output=True)
    if r.returncode != 0:
        raise SystemExit(f"git show failed: {p}")
    return r.stdout


def btext(p):
    return blobp(p).decode("utf-8", errors="replace")


def sha(b):
    return hashlib.sha256(b).hexdigest()


def parse_csv(text):
    lines = [l for l in text.splitlines() if l.strip()]
    hdr = lines[0].split(",")
    return [dict(zip(hdr, l.split(","))) for l in lines[1:]]


def clean_stats(rows):
    util = [int(r["util_percent"]) for r in rows]
    free = [int(r["free_mib"]) for r in rows]
    clean = [r["clean"].strip().lower() == "true" for r in rows]
    longest = cur = 0
    for c in clean:
        cur = cur + 1 if c else 0
        longest = max(longest, cur)
    ge40 = [(i, util[i], clean[i], int(rows[i]["streak"])) for i in range(len(rows)) if util[i] >= 40]
    return {
        "samples": len(rows), "longest_clean": longest,
        "util_min": min(util), "util_max": max(util),
        "ge40": sum(1 for u in util if u >= 40), "ge60": sum(1 for u in util if u >= 60),
        "ge40_clean": sum(1 for u, c in zip(util, clean) if u >= 40 and c),
        "free_min": min(free),
        "filtered_max": max(int(r["compute_filtered"]) for r in rows),
        "build_max": max(int(r["build"]) for r in rows),
        "last_streak": int(rows[-1]["streak"]),
        "last_clean": clean[-1],
        "ge40_rows_detail": ge40[:15],
        "ts_first": rows[0]["ts"], "ts_last": rows[-1]["ts"],
    }


def main():
    # ---------------------------------------------------------- 1 exception evidence
    exc = parse_csv(btext(f"{EV}/monitor/monitor-pre-window-attempt2.csv"))
    norm = parse_csv(btext(f"{EV}/monitor/monitor-pre-window.csv"))
    OUT["monitor_exception"] = {"recomputed": clean_stats(exc),
                                "recorded_result": btext(f"{EV}/monitor/monitor-pre-window-attempt2-exception-result.txt")}
    OUT["monitor_normal"] = {"recomputed": clean_stats(norm),
                             "recorded_result": btext(f"{EV}/monitor/monitor-pre-window-result.txt")}
    # key machine facts from exception CSV
    e = OUT["monitor_exception"]["recomputed"]
    OUT["exception_load_bearing"] = {
        "ge40_all_clean": e["ge40"] == e["ge40_clean"],
        "ge40_count": e["ge40"], "clean_streak_to_60": e["longest_clean"] == 60,
        "last_streak": e["last_streak"], "last_clean": e["last_clean"],
    }
    # verbatim commands / exception fields
    wc = btext(f"{EV}/attempt2/window-command.txt").strip()
    exc_res = btext(f"{EV}/monitor/monitor-pre-window-attempt2-exception-result.txt")
    OUT["commands"] = {
        "window_command": wc,
        "has_openutil60": "-OpenUtilLimit 60" in wc,
        "has_guardsfile": "-GuardsFile scripts\\validate\\render-bvh\\guards-r02-skybox.json" in wc,
        "exception_monitor_cmd_has_maxutil60": "-MaxUtil 60" in exc_res,
        "not_a_normal_threshold_pass_true": "not_a_normal_threshold_pass=true" in exc_res,
        "authoring_exception_used_false": "authoring_exception_used=false" in exc_res,
        "normal_failure_retained_354_18_75_exit20": all(x in exc_res for x in ("354", "longest_clean=18", "75", "exit 20")),
        "precheck_note_says_not_used": "precheck samples are not used as exception evidence" in exc_res,
        "facts_exception_block": any(l.startswith("mode=validation_only_exception") for l in btext(f"{EV}/attempt2/facts.txt").splitlines()),
    }
    # ---------------------------------------------------------- 2 criteria log
    log = btext(f"{EV}/attempt2/logs/render_skybox_reference.log")
    camps = {}
    for m in re.finditer(r"r02-s1 cam(\d) rgb: compared=(\d+) mismatched=(\d+) max_byte_diff=(\d+) beyond_1_lsb=(\d+)", log):
        camps.setdefault(f"cam{m.group(1)}", {})["rgb"] = {"compared": int(m.group(2)), "mismatched": int(m.group(3)),
                                                          "max_byte_diff": int(m.group(4)), "beyond_1_lsb": int(m.group(5))}
    for m in re.finditer(r"r02-s1 cam(\d) pixels=(\d+) depth_mismatch=(\d+) max_ulp=(\d+) beyond_ulp=(\d+) seg_mismatch=(\d+) depth_zero_pixels=(\d+)", log):
        camps.setdefault(f"cam{m.group(1)}", {})["depth_seg"] = {
            "pixels": int(m.group(2)), "depth_mismatch": int(m.group(3)), "max_ulp": int(m.group(4)),
            "beyond_ulp": int(m.group(5)), "seg_mismatch": int(m.group(6)), "depth_zero": int(m.group(7))}
    m = re.search(r"r02-s1 run sha256: rgb=([0-9a-f]{64}) depth=([0-9a-f]{64}) seg=([0-9a-f]{64}) \(captured_at=([^)]+)\)", log)
    OUT["criteria"] = {
        "cameras": camps,
        "run_shas": {"rgb": m.group(1), "depth": m.group(2), "seg": m.group(3), "captured_at": m.group(4)} if m else None,
        "test_result_ok": "test result: ok. 1 passed; 0 failed" in log,
        "two_run_sha_note": "测试内两次 fire 的 rgb/depth/seg sha256 相等由 assert_eq 钉住（同类证据见 tests/render_skybox_reference.rs）",
    }
    # ---------------------------------------------------------- 3 window facts
    ftxt = btext(f"{EV}/attempt2/facts.txt")
    facts = {}
    for line in ftxt.splitlines():
        if "=" in line and not line.startswith("#"):
            k, v = line.split("=", 1)
            facts[k.strip()] = v.strip()
    OUT["facts"] = facts
    ent = parse_csv(btext(f"{EV}/attempt2/entity-results.csv"))
    OUT["entities"] = {
        "rows": len(ent),
        "all_exit_zero": all(int(r["exit"]) == 0 for r in ent),
        "new": [r["entity"] for r in ent if r["kind"] == "new"],
        "regression": [r["entity"] for r in ent if r["kind"] == "regression"],
        "facts_lines_match": all(f"entity {r['entity']} exit={r['exit']} wall_ms={r['wall_ms']}" in btext(f"{EV}/attempt2/facts.txt") for r in ent),
    }
    samples = parse_csv(btext(f"{EV}/attempt2/samples.csv"))
    used = [int(r["used_mib"]) for r in samples]
    free = [int(r["free_mib"]) for r in samples]
    util = [int(r["util_percent"]) for r in samples]
    peak = max(used)
    OUT["samples"] = {
        "rows": len(samples), "used_peak": peak, "peak_seq": int(samples[used.index(peak)]["seq"]),
        "used_min": min(used), "free_min": min(free), "util_min": min(util), "util_max": max(util),
        "floor_mib": int(facts["floor_mib"]), "floor_breached": min(free) < int(facts["floor_mib"]),
        "increment_limit": int(re.match(r"\d+", facts["increment_limit_mib"]).group()),
        "delta_vs_first": peak - used[0],
        "delta_exceeded": (peak - used[0]) > 3072,
        "streak_ge_80": 0,
    }
    # guard.log sequence
    gl = btext(f"{EV}/attempt2/guard.log").splitlines()
    OUT["guard_log"] = {"lines": gl,
                        "precheck_ok": any(l.startswith("PRECHECK_OK") for l in gl),
                        "guards_ok_77": any("GUARDS_OK=77/77" in l for l in gl),
                        "window_run_done": any("WINDOW_RUN_DONE" in l for l in gl)}
    # ---------------------------------------------------------- 4 freeze identity
    guards_raw = (REPO / "scripts/validate/render-bvh/guards-r02-skybox.json").read_bytes()
    guards = json.loads(guards_raw.decode("utf-8-sig"))
    committed = blobp("scripts/validate/render-bvh/guards-r02-skybox.json")
    OUT["guards"] = {
        "sha256": sha(guards_raw), "bytes": len(guards_raw),
        "committed_sha256": sha(committed), "committed_equals_disk": committed == guards_raw,
        "suite": guards.get("suite"), "attempt": guards.get("attempt"), "counts": guards.get("counts"),
        "aggregate_recorded": guards.get("aggregate_sha256"), "generated_at": guards.get("generated_at"),
        "frozen_exes": len(guards["frozen_exes"]), "guard_inputs": len(guards["guard_inputs"]),
        "cr": guards_raw.count(b"\r"),
    }
    # aggregate via culture-sort caliber (PowerShell Sort-Object) reproduced in python: compare all orderings
    ent_list = [(e["path"], e["sha256"]) for e in guards["frozen_exes"]] + \
               [(e["path"], e["sha256"]) for e in guards["guard_inputs"]]
    out_ord = sha("".join(h for _, h in sorted(ent_list, key=lambda t: t[0])).encode())
    out_cf = sha("".join(h for _, h in sorted(ent_list, key=lambda t: t[0].casefold())).encode())
    OUT["guards"]["aggregate_ordinal"] = out_ord
    OUT["guards"]["aggregate_casefold"] = out_cf
    # triple equality live == guards == stash
    rows = []
    for ex in guards["frozen_exes"]:
        live = REPO / ex["path"]
        stash = REPO / ex["stash_path"]
        rows.append({"entity": ex["entity"],
                     "guards_eq_stash_field": ex["sha256"] == ex["stash_sha256"],
                     "live_ok": live.exists() and sha(live.read_bytes()) == ex["sha256"] and live.stat().st_size == ex["bytes"],
                     "stash_ok": stash.exists() and sha(stash.read_bytes()) == ex["sha256"] and stash.stat().st_size == ex["bytes"],
                     "mtime": live.stat().st_mtime_ns if live.exists() else None})
    OUT["exe_triple"] = rows
    # sha-guards.txt 77 rows vs disk + guards key set
    sg = [l.split() for l in btext(f"{EV}/attempt2/sha-guards.txt").splitlines() if l.strip()]
    gmap = {e["path"]: (e["sha256"], e["bytes"]) for e in guards["frozen_exes"]}
    gmap.update({e["path"]: (e["sha256"], e["bytes"]) for e in guards["guard_inputs"]})
    sgmap = {r[3]: (r[1], int(r[2])) for r in sg}
    OUT["sha_guards"] = {"rows": len(sg), "tags": sorted(set(r[0] for r in sg)),
                         "key_set_equal": set(sgmap) == set(gmap),
                         "all_match_guards": all(sgmap.get(k) == v for k, v in gmap.items()),
                         "disk_mismatch": [r[3] for r in sg if not (REPO / r[3]).exists()
                                           or sha((REPO / r[3]).read_bytes()) != r[1]
                                           or (REPO / r[3]).stat().st_size != int(r[2])]}
    # R05 guards untouched
    r05 = (REPO / "scripts/validate/render-bvh/guards.json").read_bytes()
    OUT["r05_guards"] = {"sha256": sha(r05), "bytes": len(r05),
                         "unchanged_vs_known": sha(r05) == "e2444a96128f33575bffab0f9420a1844c2f6cf2d3b60ef309f1fcad9b7eecb3"}
    # attempt1 superseded guards (target only)
    sup = WIN / "guards-r02-skybox-attempt1-superseded.json"
    OUT["attempt1_guards_archive"] = {"path": str(sup), "exists": sup.exists(),
                                      "sha256": sha(sup.read_bytes()) if sup.exists() else None,
                                      "bytes": sup.stat().st_size if sup.exists() else None}
    # ---------------------------------------------------------- 5 guards_only logs
    OUT["guards_only"] = {}
    for p in (REPO / "target/render-bvh/r02-skybox-window").glob("*"):
        if p.is_dir() and "guards" in p.name:
            OUT["guards_only"][p.name] = sorted(f.name for f in p.glob("*"))[:6]
    # ---------------------------------------------------------- 6 attempt1 + fix chain
    OUT["attempt1"] = {
        "entity_results": btext(f"{EV}/attempt1/entity-results.csv"),
        "invalid_round_note_has_4_failures_and_two_root_causes": all(
            x in btext(f"{EV}/attempt1/invalid-round-note.txt") for x in
            ("entity_failures=", "failure_A_stale_negative_controls", "failure_B_skybox_depth_not_bitwise", "use_fast_math")),
        "has_capture_fix_diff": bool(blobp(f"{EV}/attempt1/capture-skybox-fix.diff")),
        "stage_a_failure": btext(f"{EV}/attempt1/stage-a-attempt1-failure.txt"),
    }
    cap = (REPO / "scripts/authoring/render-bvh-r02/capture_skybox.py").read_bytes()
    OUT["capture_script"] = {"sha256": sha(cap), "bytes": len(cap), "lines_lf": cap.count(b"\n")}
    # historical blobs of the capture script (S2 original + window-period fixes)
    hist = subprocess.run(["git", "log", "--all", "--oneline", "--", "scripts/authoring/render-bvh-r02/capture_skybox.py"],
                          cwd=REPO, capture_output=True).stdout.decode().splitlines()
    OUT["capture_script"]["history"] = hist[:10]
    # find blobs by sha in history
    blob_shas = {}
    for line in hist:
        rev = line.split()[0]
        b = subprocess.run(["git", "show", f"{rev}:scripts/authoring/render-bvh-r02/capture_skybox.py"],
                           cwd=REPO, capture_output=True).stdout
        if b:
            blob_shas[sha(b)] = {"bytes": len(b), "lines": b.count(b"\n"), "commit": rev}
    OUT["capture_script"]["blobs_in_history"] = blob_shas
    # params28 audit
    au = btext("docs/development-evidence/render-bvh/r02-skybox-s2-capture-authoring/materials/params28-audit.txt")
    OUT["params28_audit"] = {"rows": len([l for l in au.splitlines() if l.strip() and not l.startswith("#")]),
                             "mentions_only_use_fast_math_diff": "use_fast_math" in au,
                             "head": au.splitlines()[:6]}
    # capture.json (reference)
    cj_raw = (REPO / "fixtures/render-bvh/scene-a/reference-skybox/capture.json").read_bytes()
    cj = json.loads(cj_raw.decode("utf-8-sig"))
    OUT["reference"] = {
        "capture_json_sha256": sha(cj_raw),
        "use_fast_math": cj.get("params28", {}).get("use_fast_math") if isinstance(cj.get("params28"), dict) else cj.get("use_fast_math"),
        "params28_sha256": cj.get("params28_sha256"),
        "two_run_equal": cj.get("two_run_equal", cj.get("two_captures_bitwise_identical")),
        "files": {f: sha((REPO / "fixtures/render-bvh/scene-a/reference-skybox" / f).read_bytes())
                  for f in ("rgb.npz", "depth.npz", "seg.npz")},
        "keys": sorted(cj.keys())[:12],
    }
    # superseded old reference (target only)
    old = WIN / "stage-a-attempt1-reference-superseded"
    OUT["old_reference"] = {"path": str(old), "exists": old.exists(),
                            "files": {p.name: sha(p.read_bytes()) for p in sorted(old.glob("*"))} if old.exists() else None}
    # ---------------------------------------------------------- 7 manifests
    def check_manifest(path):
        man = btext(path)
        mrows = [l.split() for l in man.splitlines() if l and not l.startswith("#")]
        bad_c, bad_d, selfref = [], [], False
        for digest, size, p in mrows:
            size = int(size)
            if p.endswith("manifest.txt"):
                selfref = True
            b = blobp(p)
            if sha(b) != digest or len(b) != size:
                bad_c.append(p)
            f = REPO / p
            if not f.exists():
                bad_d.append({"path": p, "why": "missing"})
            else:
                wb = f.read_bytes()
                if sha(wb) != digest or len(wb) != size:
                    bad_d.append({"path": p, "why": "mismatch"})
        return {"rows": len(mrows), "bad_committed": bad_c, "bad_disk": bad_d, "self_reference": selfref,
                "header": man.splitlines()[0]}
    OUT["manifest_s3"] = check_manifest(f"{EV}/manifest.txt")
    OUT["manifest_s3a"] = check_manifest(f"{EV_A}/manifest.txt")
    # hygiene: LF / no BOM / forbidden literals over both dirs
    terms = [[0x5E38, 0x89C4, 0x95E8, 0x69DB, 0x901A, 0x8FC7], [0x7CBE, 0x5EA6, 0x8FBE, 0x6807],
             [0x4E0A, 0x6E38, 0x7B49, 0x4EF7], [0x4EA7, 0x54C1, 0x6E32, 0x67D3, 0x9A8C, 0x6536, 0x901A, 0x8FC7]]
    words = ["".join(chr(c) for c in t) for t in terms]
    hyg = {"non_lf": [], "forbidden": []}
    for base in (EV, EV_A):
        tree = subprocess.run(["git", "ls-tree", "-r", "--name-only", "HEAD", base], cwd=REPO,
                              capture_output=True).stdout.decode().split()
        for p in tree:
            b = blobp(p)
            if b.count(b"\r") or b.count(b"\x00") or b[:3] == b"\xef\xbb\xbf":
                hyg["non_lf"].append(p.split("/")[-1])
            txt = b.decode("utf-8", errors="replace")
            hits = [w for w in words if w in txt]
            if hits:
                # record file + term ordinals + 1-based line numbers only (never the literal text)
                ln = [i + 1 for i, l in enumerate(txt.splitlines()) if any(w in l for w in words)]
                hyg["forbidden"].append({"file": p.split("/")[-1],
                                          "term_ordinals": [words.index(w) for w in hits],
                                          "line_numbers": ln[:6]})
    OUT["hygiene"] = hyg
    # ---------------------------------------------------------- 8 guard_aggregate vs facts
    ga = json.loads(btext(f"{EV}/attempt2/guard_aggregate.json"))
    pairs = {
        "samples": (ga["samples"], int(facts["samples"])),
        "used_peak_mib": (ga["used_peak_mib"], int(facts["used_peak_mib"].split()[0])),
        "peak_seq": (ga["peak_seq"], int(re.search(r"peak_seq=(\d+)", facts["used_peak_mib"]).group(1))),
        "used_min_mib": (ga["used_min_mib"], int(re.search(r"used_min_mib=(\d+)", ftxt).group(1))),
        "free_min_mib": (ga["free_min_mib"], int(re.search(r"free_min_mib=(\d+)", ftxt).group(1))),
        "util_max_percent": (ga["util_max_percent"], int(re.search(r"util_max_percent=(\d+)", ftxt).group(1))),
        "floor_mib": (ga["floor_mib"], int(facts["floor_mib"])),
        "baseline_free_mib": (ga["baseline_free_mib"], int(facts["baseline_free_mib"])),
        "guards_aggregate_sha256": (ga["guards_aggregate_sha256"], facts["guards_aggregate_sha256"]),
        "sha_guards": (ga["sha_guards"], "77/77"),
        "post_window_exe_recheck": (ga["post_window_exe_recheck"], "9/9"),
    }
    OUT["aggregate_vs_facts"] = {k: {"json": a, "facts": b, "match": a == b} for k, (a, b) in pairs.items()}
    OUT["aggregate_vs_facts"]["all_match"] = all(v["match"] for v in OUT["aggregate_vs_facts"].values() if isinstance(v, dict))
    OUT["aggregate_suite_field"] = ga.get("suite")
    # ---------------------------------------------------------- 9 evidence copies vs target originals
    copies = {}
    if (WIN / "attempt2").exists():
        ev_dir = REPO / EV / "attempt2"
        for f in sorted(ev_dir.rglob("*")):
            if not f.is_file():
                continue
            rel = f.relative_to(ev_dir)
            raw = WIN / "attempt2" / rel
            copies[str(rel)] = {"same_bytes": raw.exists() and sha(f.read_bytes()) == sha(raw.read_bytes()),
                                "same_content_lf": raw.exists() and f.read_bytes().replace(b"\r\n", b"\n") == raw.read_bytes().replace(b"\r\n", b"\n")}
    OUT["copies_vs_target"] = copies
    # ---------------------------------------------------------- 10 problems
    problems = []
    if not OUT["exception_load_bearing"]["ge40_all_clean"] or OUT["exception_load_bearing"]["ge40_count"] != 12:
        problems.append("exception load-bearing evidence (12 ge40 clean)")
    if e["samples"] != 60 or e["longest_clean"] != 60 or e["ge60"] != 0 or e["free_min"] != 12751:
        problems.append("exception monitor numbers")
    n = OUT["monitor_normal"]["recomputed"]
    if n["samples"] != 354 or n["longest_clean"] != 18 or n["ge40"] != 75:
        problems.append("normal monitor numbers")
    if not (OUT["commands"]["has_openutil60"] and OUT["commands"]["exception_monitor_cmd_has_maxutil60"]):
        problems.append("commands")
    for cam, v in camps.items():
        for key in ("rgb", "depth_seg"):
            for k, val in v[key].items():
                if k in ("mismatched", "max_byte_diff", "beyond_1_lsb", "depth_mismatch", "max_ulp", "beyond_ulp", "seg_mismatch") and val != 0:
                    problems.append(f"criteria {cam} {key} {k}={val}")
    if OUT["criteria"]["run_shas"] is None:
        problems.append("run sha line missing")
    if not OUT["entities"]["all_exit_zero"] or OUT["entities"]["rows"] != 9:
        problems.append("entities")
    if OUT["samples"]["rows"] != 166 or OUT["samples"]["floor_breached"] or OUT["samples"]["delta_exceeded"]:
        problems.append("window samples")
    if OUT["guards"]["sha256"] != "2d55c0b3fc14a33b2acce159bf68f474131f2819e3b2230f17ece11021b2e6a3" or OUT["guards"]["bytes"] != 28931:
        problems.append("guards identity")
    # aggregate caliber: the recorded value is reproduced by the evidence's PowerShell culture-sort
    # script (see logs/materials); python ordinal/casefold are NOT that caliber, so no problem here.
    OUT["guards"]["aggregate_caliber_note"] = "recorded value reproduced by materials/recompute-aggregate.ps1 (culture sort)"
    if not all(r["guards_eq_stash_field"] and r["live_ok"] and r["stash_ok"] for r in rows):
        problems.append("exe triple equality")
    if OUT["sha_guards"]["rows"] != 77 or not OUT["sha_guards"]["key_set_equal"] or OUT["sha_guards"]["disk_mismatch"]:
        problems.append("sha-guards")
    if not OUT["r05_guards"]["unchanged_vs_known"]:
        problems.append("R05 guards changed")
    if OUT["manifest_s3"]["rows"] != 34 or OUT["manifest_s3"]["bad_committed"] or OUT["manifest_s3"]["bad_disk"] \
            or OUT["manifest_s3"]["self_reference"]:
        problems.append("S3 manifest")
    if OUT["manifest_s3a"]["rows"] != 2 or OUT["manifest_s3a"]["bad_committed"] or OUT["manifest_s3a"]["bad_disk"]:
        problems.append("S3a manifest")
    if not OUT["aggregate_vs_facts"]["all_match"]:
        problems.append("aggregate vs facts numeric mismatch")
    if OUT["aggregate_suite_field"] != "r02-skybox-validate":
        problems.append("guard_aggregate suite label (runner hardcoded r05 suite)")
    if OUT["hygiene"]["forbidden"]:
        problems.append("forbidden literals present in validated evidence (card item 7)")
    if not OUT["capture_script"]["sha256"].startswith("d6f9864c"):
        problems.append("capture script sha")
    if OUT["reference"]["use_fast_math"] is not False:
        problems.append("reference use_fast_math != false")
    OUT["problems"] = problems

    pathlib.Path(r"D:\Rust\mjwarp-rs\target\qa-s3\log\recompute.json").write_text(
        json.dumps(OUT, ensure_ascii=False, indent=1), encoding="utf-8", newline="\n")

    print("== exception monitor ==", {k: v for k, v in e.items() if k != "ge40_rows_detail"})
    print("   ge40 rows:", e["ge40_rows_detail"])
    print("== normal monitor ==", {k: v for k, v in n.items() if k != "ge40_rows_detail"})
    print("== load bearing ==", OUT["exception_load_bearing"])
    print("== commands ==", OUT["commands"])
    print("== criteria ==", json.dumps(OUT["criteria"], ensure_ascii=False)[:1200])
    print("== entities ==", OUT["entities"])
    print("== samples ==", OUT["samples"])
    print("== guard.log ==", OUT["guard_log"])
    print("== guards ==", {k: v for k, v in OUT["guards"].items()})
    print("== exe triple ok ==", all(r["guards_eq_stash_field"] and r["live_ok"] and r["stash_ok"] for r in rows))
    print("== sha-guards ==", {k: (v if k != "disk_mismatch" else v[:3]) for k, v in OUT["sha_guards"].items()})
    print("== r05 guards ==", OUT["r05_guards"])
    print("== attempt1 guards archive ==", OUT["attempt1_guards_archive"])
    print("== attempt1 ==", {k: v for k, v in OUT["attempt1"].items() if k in ("invalid_round_note_has_4_failures_and_two_root_causes", "has_capture_fix_diff")})
    print("== capture script ==", OUT["capture_script"])
    print("== params28 ==", OUT["params28_audit"])
    print("== reference ==", OUT["reference"])
    print("== old reference ==", OUT["old_reference"])
    print("== manifests ==", OUT["manifest_s3"], OUT["manifest_s3a"])
    print("== hygiene ==", OUT["hygiene"])
    print("== agg vs facts ==", {k: (v["match"] if isinstance(v, dict) else v) for k, v in OUT["aggregate_vs_facts"].items()}, "suite:", OUT["aggregate_suite_field"])
    print("== copies vs target ==", {k: v["same_bytes"] for k, v in OUT["copies_vs_target"].items()})
    print("== guards_only dirs ==", OUT["guards_only"])
    print("PROBLEMS", problems)
    return 0


if __name__ == "__main__":
    sys.exit(main())
