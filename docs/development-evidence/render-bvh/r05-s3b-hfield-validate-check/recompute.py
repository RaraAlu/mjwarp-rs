"""QA independent recompute for R05-S3b hfield validation window attempt=2 check.

Everything is derived from committed blobs / on-disk raw artifacts; no number is
copied from README/receipt claims (claims are only used as expectations to falsify).
"""
import hashlib
import json
import pathlib
import re
import subprocess
import sys

REPO = pathlib.Path(r"D:\Rust\mjwarp-rs")
EV = "docs/development-evidence/render-bvh/r05-s3b-hfield-validate"
WIN = REPO / "target/render-bvh/r05-s3b-window"
OUT = {}


def blob(path):
    p = subprocess.run(["git", "show", f"HEAD:{path}"], cwd=REPO, capture_output=True)
    if p.returncode != 0:
        raise SystemExit(f"git show failed: {path}")
    return p.stdout


def blob_text(path):
    return blob(path).decode("utf-8")


def sha(b):
    return hashlib.sha256(b).hexdigest()


def gf(path):
    return sha(pathlib.Path(path).read_bytes())


# ---------------------------------------------------------------- A/B monitors
def parse_monitor(text):
    lines = [l for l in text.splitlines() if l.strip()]
    hdr = lines[0].split(",")
    rows = [dict(zip(hdr, l.split(","))) for l in lines[1:]]
    return hdr, rows


def monitor_stats(rows):
    n = len(rows)
    util = [int(r["util_percent"]) for r in rows]
    free = [int(r["free_mib"]) for r in rows]
    clean = [r["clean"].strip().lower() == "true" for r in rows]
    longest = cur = 0
    for c in clean:
        cur = cur + 1 if c else 0
        longest = max(longest, cur)
    return {
        "samples": n,
        "util_min": min(util),
        "util_max": max(util),
        "util_ge_40": sum(1 for u in util if u >= 40),
        "util_ge_60": sum(1 for u in util if u >= 60),
        "free_min_mib": min(free),
        "longest_clean": longest,
        "build_max": max(int(r["build"]) for r in rows),
        "compute_filtered_max": max(int(r["compute_filtered"]) for r in rows),
        "ts_first": rows[0]["ts"],
        "ts_last": rows[-1]["ts"],
        "clean_col_last_streak": int(rows[-1]["streak"]),
    }


def monitor_result_kv(text):
    kv = {}
    for line in text.splitlines():
        if "=" in line:
            k, v = line.split("=", 1)
            kv[k.strip()] = v.strip()
    return kv


# ---------------------------------------------------------------- facts
def parse_facts(text):
    kv = {}
    for line in text.splitlines():
        if "=" in line and not line.startswith("#"):
            k, v = line.split("=", 1)
            kv[k.strip()] = v.strip()
    return kv


def main():
    facts = parse_facts(blob_text(f"{EV}/attempt2/facts.txt"))
    final_csv = parse_monitor(blob_text(f"{EV}/monitor/monitor-pre-window-attempt2-final.csv"))
    exc_csv = parse_monitor(blob_text(f"{EV}/monitor/monitor-pre-window-attempt2-exception.csv"))
    final_res = monitor_result_kv(blob_text(f"{EV}/monitor/monitor-pre-window-attempt2-final-result.txt"))
    exc_res = monitor_result_kv(blob_text(f"{EV}/monitor/monitor-pre-window-attempt2-exception-result.txt"))

    OUT["monitor_normal"] = {
        "recomputed": monitor_stats(final_csv[1]),
        "recorded": {k: final_res.get(k) for k in
                     ("samples", "longest_clean", "required", "util_min", "util_max",
                      "util_ge_40_samples", "util_ge_60_samples", "free_min_mib", "decision",
                      "window_commands_executed", "mode")},
        "csv_rows": len(final_csv[1]),
    }
    OUT["monitor_exception"] = {
        "recomputed": monitor_stats(exc_csv[1]),
        "recorded": {k: exc_res.get(k) for k in
                     ("samples", "longest_clean", "required", "util_min", "util_max",
                      "util_ge_40_samples", "util_ge_60_samples", "free_min_mib", "window_commands_executed",
                      "mode", "approved_by", "not_a_normal_threshold_pass",
                      "authoring_exception_used", "signature_unchanged", "guards_sha256", "guards_bytes")},
        "csv_rows": len(exc_csv[1]),
    }

    # guard.log / window-command
    gl = blob_text(f"{EV}/attempt2/guard.log").splitlines()
    OUT["guard_log"] = {"lines": gl, "has_precheck_ok": any(l.startswith("PRECHECK_OK") for l in gl),
                        "guards_ok_73": any("GUARDS_OK=73/73" in l for l in gl),
                        "window_run_done": any("WINDOW_RUN_DONE" in l for l in gl)}
    wc = blob_text(f"{EV}/attempt2/window-command.txt").strip()
    OUT["window_command"] = {"text": wc, "has_openutil60": "-OpenUtilLimit 60" in wc,
                             "has_attempt2": "-Attempt 2" in wc}
    pre = [l for l in gl if l.startswith("PRECHECK_OK")]
    pre_utils = []
    if pre:
        for part in pre[0].split("open=", 1)[1].split(";"):
            bits = part.strip().split("/")
            if len(bits) == 4:
                pre_utils.append(int(bits[2]))
    OUT["precheck_samples_util"] = pre_utils
    OUT["precheck_has_ge_40_sample"] = any(u >= 40 for u in pre_utils)

    # ------------------------------------------------------------ C samples
    samples_text = blob_text(f"{EV}/attempt2/samples.csv")
    srows = [l.split(",") for l in samples_text.splitlines()[1:] if l.strip()]
    seq = [int(r[0]) for r in srows]
    used = [int(r[2]) for r in srows]
    free = [int(r[3]) for r in srows]
    util = [int(r[4]) for r in srows]
    peak = max(used)
    peak_seq = seq[used.index(peak)]
    streak = mx = 0
    for u in util:
        streak = streak + 1 if u >= 80 else 0
        mx = max(mx, streak)
    floor = int(facts["floor_mib"])
    baseline_used = int(facts["baseline_used,free,util,temp".split(" /")[0].split("=")[1].split(",")[0]]) if False else None
    base_line = facts.get("baseline_used,free,util,temp", "")
    baseline_first_used = int(base_line.split(" / ")[0].split(",")[0])
    OUT["window_samples"] = {
        "rows": len(srows),
        "seq_first": seq[0],
        "seq_last": seq[-1],
        "used_peak_mib": peak,
        "peak_seq": peak_seq,
        "used_min_mib": min(used),
        "free_min_mib": min(free),
        "util_min": min(util),
        "util_max": max(util),
        "util_streak_ge_80": mx,
        "floor_mib": floor,
        "floor_breached": min(free) < floor,
        "delta_vs_first_sample": peak - used[0],
        "delta_vs_baseline_first": peak - baseline_first_used,
        "increment_limit_mib": int(re.match(r"\d+", facts["increment_limit_mib"]).group()),
        "increment_exceeded": (peak - baseline_first_used) > int(re.match(r"\d+", facts["increment_limit_mib"]).group()),
    }

    # ------------------------------------------------------------ D/E guards
    guards_raw = (REPO / "scripts/validate/render-bvh/guards.json").read_bytes()
    guards = json.loads(guards_raw.decode("utf-8-sig"))
    OUT["guards_json"] = {
        "sha256": sha(guards_raw), "bytes": len(guards_raw),
        "cr": guards_raw.count(b"\r"), "lf": guards_raw.count(b"\n"),
        "attempt": guards.get("attempt"),
        "frozen_exes": len(guards.get("frozen_exes", [])),
        "guard_inputs": len(guards.get("guard_inputs", [])),
        "aggregate_sha256": guards.get("aggregate_sha256"),
        "generated_at": guards.get("generated_at"),
    }
    # recompute aggregate: sha256 over sorted repo-relative paths' sha hex, concatenated
    entries = []
    for e in guards["frozen_exes"]:
        entries.append((e["path"], e.get("sha256") or e.get("live_sha256"), e.get("bytes") or e.get("live_bytes")))
    for e in guards["guard_inputs"]:
        entries.append((e["path"], e["sha256"], e["bytes"]))
    concat = "".join(h for _, h, _ in sorted(entries, key=lambda t: t[0]))
    OUT["guards_json"]["aggregate_recomputed"] = sha(concat.encode())
    OUT["guards_json"]["entries"] = len(entries)

    # live exe sha vs guards vs stash
    exe_rows = []
    for e in guards["frozen_exes"]:
        live = REPO / e["live_path"] if "live_path" in e else None
        row = {"entity": e.get("entity"), "keys": sorted(e.keys())}
        exe_rows.append(row)
    OUT["frozen_exes_keys"] = sorted(guards["frozen_exes"][0].keys())

    # sha-guards.txt (73 rows)
    sg = [l.split() for l in blob_text(f"{EV}/attempt2/sha-guards.txt").splitlines() if l.strip()]
    OUT["sha_guards_rows"] = len(sg)
    bad = []
    exe_checked = {}
    for parts in sg:
        tag, digest, size, path = parts[0], parts[1], int(parts[2]), parts[3]
        p = REPO / path
        if not p.exists():
            bad.append({"path": path, "why": "missing"})
            continue
        b = p.read_bytes()
        if sha(b) != digest or len(b) != size:
            bad.append({"path": path, "why": "mismatch", "got": sha(b), "want": digest})
        if "debug/deps" in path.replace("\\", "/"):
            exe_checked[path] = digest
    OUT["sha_guards"] = {"rows": len(sg), "bad": bad, "exe_count": len(exe_checked)}

    # stash dir 9/9
    stash = WIN / "frozen-exes"
    stash_rows = []
    if stash.exists():
        for f in sorted(stash.glob("*.exe")):
            stash_rows.append({"name": f.name, "sha256": gf(f), "bytes": f.stat().st_size})
    OUT["stash"] = stash_rows

    # ------------------------------------------------------------ F entities
    er = [l.split(",") for l in blob_text(f"{EV}/attempt2/entity-results.csv").splitlines()[1:] if l.strip()]
    ent = [{"entity": r[0], "kind": r[1], "exit": int(r[2]), "wall_ms": int(r[3])} for r in er]
    OUT["entities"] = {
        "count": len(ent), "all_exit_zero": all(e["exit"] == 0 for e in ent),
        "new_entities": [e["entity"] for e in ent if e["kind"] == "new"],
        "regression_entities": [e["entity"] for e in ent if e["kind"] == "regression"],
        "facts_match": all(f"entity {e['entity']} exit={e['exit']} wall_ms={e['wall_ms']}" in blob_text(f"{EV}/attempt2/facts.txt")
                           for e in ent),
    }

    # ------------------------------------------------------------ G hfield log
    hl = blob_text(f"{EV}/attempt2/logs/render_hfield_reference.log")
    groups = {}
    for m in re.finditer(r"r05-s3b\[(hfield-only|hfield-primitive)\] cam(\d) rgb: compared=(\d+) mismatched=(\d+) max_byte_diff=(\d+) beyond_1_lsb=(\d+)", hl):
        g, cam = m.group(1), m.group(2)
        groups.setdefault(g, {})[f"cam{cam}_rgb"] = {"compared": int(m.group(3)), "mismatched": int(m.group(4)),
                                                     "max_byte_diff": int(m.group(5)), "beyond_1_lsb": int(m.group(6))}
    for m in re.finditer(r"r05-s3b\[(hfield-only|hfield-primitive)\] cam(\d) pixels=(\d+) depth_mismatch=(\d+) max_ulp=(\d+) beyond_ulp=(\d+) seg_mismatch=(\d+) depth_zero_pixels=(\d+)", hl):
        g, cam = m.group(1), m.group(2)
        groups.setdefault(g, {})[f"cam{cam}_depth_seg"] = {"pixels": int(m.group(3)), "depth_mismatch": int(m.group(4)),
                                                            "max_ulp": int(m.group(5)), "beyond_ulp": int(m.group(6)),
                                                            "seg_mismatch": int(m.group(7)), "depth_zero": int(m.group(8))}
    run_shas = {}
    for m in re.finditer(r"r05-s3b\[(hfield-only|hfield-primitive)\] run sha256: rgb=([0-9a-f]{64}) depth=([0-9a-f]{64}) seg=([0-9a-f]{64}) \(captured_at=([^)]+)\)", hl):
        run_shas[m.group(1)] = {"rgb": m.group(2), "depth": m.group(3), "seg": m.group(4), "captured_at": m.group(5)}
    OUT["hfield_log"] = {"groups": groups, "run_shas": run_shas,
                         "test_results_ok": hl.count("test result: ok. 2 passed; 0 failed") == 1,
                         "run_sha_lines": len(run_shas)}

    # ------------------------------------------------------------ H references
    refs = {}
    s3a_manifest = blob_text("docs/development-evidence/render-bvh/r05-s3a-hfield-capture/manifest.txt")
    for g in ("hfield-only", "hfield-primitive"):
        cj = json.loads(blob_text(f"fixtures/render-bvh/scene-hfield/capture-hfield/{g}/capture.json"))
        files = cj.get("files_sha256", {})
        rows = []
        for name, digest in files.items():
            p = REPO / "fixtures/render-bvh/scene-hfield/capture-hfield" / g / name
            e = {"file": name, "capture_json_sha": digest,
                 "on_disk_sha": gf(p) if p.exists() else None,
                 "in_s3a_manifest": digest in s3a_manifest}
            rows.append(e)
        refs[g] = {"captured_at": cj.get("captured_at"), "files": rows,
                   "all_match": all(r["on_disk_sha"] == r["capture_json_sha"] for r in rows),
                   "all_in_s3a_manifest": all(r["in_s3a_manifest"] for r in rows)}
    OUT["references"] = refs

    # ------------------------------------------------------------ I manifest
    man = blob_text(f"{EV}/manifest.txt")
    mrows = [l for l in man.splitlines() if l and not l.startswith("#")]
    mbad_committed = []
    mbad_disk = []
    self_ref = False
    for l in mrows:
        digest, size, path = l.split()
        size = int(size)
        if path.endswith("manifest.txt"):
            self_ref = True
        b = blob(path)
        if sha(b) != digest or len(b) != size:
            mbad_committed.append(path)
        p = REPO / path
        if not p.exists():
            mbad_disk.append({"path": path, "why": "missing"})
        else:
            wb = p.read_bytes()
            if sha(wb) != digest or len(wb) != size:
                mbad_disk.append({"path": path, "why": "mismatch", "disk": sha(wb), "manifest": digest, "disk_bytes": len(wb), "bytes": size})
    listed = {l.split()[-1] for l in mrows}
    tree = subprocess.run(["git", "ls-tree", "-r", "--name-only", "HEAD", f"{EV}/"],
                          cwd=REPO, capture_output=True).stdout.decode().split()
    rel = {t[len(EV) + 1:] for t in tree}
    OUT["manifest"] = {"rows": len(mrows), "bad_committed": mbad_committed, "bad_disk": mbad_disk,
                       "self_reference": self_ref,
                       "unlisted": sorted(p for p in rel if p not in listed and p != "manifest.txt"),
                       "extra_listed": sorted(p for p in listed if p not in rel)}

    # ------------------------------------------------------------ J zero change
    diff = subprocess.run(["git", "diff", "--name-status", "d1f7f851..HEAD"], cwd=REPO, capture_output=True).stdout.decode()
    OUT["chain_diff"] = [l for l in diff.splitlines() if l]
    prot = subprocess.run(["git", "diff", "--stat", "d1f7f851..HEAD", "--", "src", "tests", "fixtures", "scripts/validate/render-bvh/criteria", "docs/development-evidence/render-bvh/r05-s3b-hfield-validate/attempt1"],
                          cwd=REPO, capture_output=True).stdout.decode()
    OUT["protected_diff"] = prot.strip()

    # ------------------------------------------------------------ K attempt1 / inputs
    a1 = WIN / "attempt1"
    a2 = WIN / "attempt2"
    OUT["attempt1_present"] = a1.exists()
    OUT["attempt1_files"] = sorted(p.name for p in a1.glob("*")) if a1.exists() else []
    OUT["attempt2_files"] = sorted(p.name for p in a2.glob("*")) if a2.exists() else []
    OUT["attempt1_facts_head"] = (a1 / "facts.txt").read_text(encoding="utf-8", errors="replace").splitlines()[:6] if (a1 / "facts.txt").exists() else None
    # 64 read-only inputs vs superseded attempt1 guards
    old = WIN / "guards-attempt1-superseded.json"
    OUT["old_guards_present"] = old.exists()
    if old.exists():
        og = json.loads(old.read_text(encoding="utf-8-sig"))
        oc = {e["path"]: (e["sha256"], e["bytes"]) for e in og["guard_inputs"]}
        nc = {e["path"]: (e["sha256"], e["bytes"]) for e in guards["guard_inputs"]}
        OUT["guard_inputs_vs_attempt1"] = {
            "old_count": len(oc), "new_count": len(nc),
            "identical": oc == nc,
            "diff": [(k, oc.get(k), nc.get(k)) for k in sorted(set(oc) | set(nc)) if oc.get(k) != nc.get(k)],
        }
        OUT["old_guards_sha256"] = sha(old.read_bytes())
    # group aggregates of guard_inputs
    def group_agg(pred):
        sel = [(e["path"], e["sha256"], e["bytes"]) for e in guards["guard_inputs"] if pred(e["path"])]
        sel.sort(key=lambda t: t[0])
        return {"files": len(sel), "bytes": sum(b for _, _, b in sel),
                "aggregate": sha("".join(h for _, h, _ in sel).encode())}
    OUT["capture_group"] = group_agg(lambda p: "capture-hfield" in p)
    OUT["legacy_group"] = group_agg(lambda p: "scene-a" in p or "scene-e2e" in p)
    OUT["shadow_group"] = group_agg(lambda p: "scene-shadow" in p)

    # ------------------------------------------------------------ L expected values
    OUT["facts"] = facts
    OUT["guard_aggregate_json"] = json.loads(blob_text(f"{EV}/attempt2/guard_aggregate.json"))

    # ------------------------------------------------------------ problems
    problems = []
    m = OUT["monitor_normal"]["recomputed"]
    if m["samples"] != 354 or m["longest_clean"] != 45 or m["util_ge_40"] != 33 or m["util_ge_60"] != 0:
        problems.append("normal monitor recompute mismatch")
    if final_res.get("decision") != "STOP" or final_res.get("window_commands_executed") != "0":
        problems.append("normal monitor decision")
    e = OUT["monitor_exception"]["recomputed"]
    if e["samples"] != 60 or e["longest_clean"] != 60 or e["util_min"] < 36 or e["util_max"] > 41:
        problems.append("exception monitor recompute mismatch")
    if exc_res.get("not_a_normal_threshold_pass") != "true" or exc_res.get("samples") != "60":
        problems.append("exception record")
    if not OUT["window_command"]["has_openutil60"]:
        problems.append("window command lacks -OpenUtilLimit 60")
    if not (OUT["guard_log"]["has_precheck_ok"] and OUT["guard_log"]["guards_ok_73"] and OUT["guard_log"]["window_run_done"]):
        problems.append("guard.log sequence")
    s = OUT["window_samples"]
    if s["rows"] != 171 or s["floor_breached"] or s["increment_exceeded"] or s["util_streak_ge_80"] != 0:
        problems.append("window samples")
    if OUT["guards_json"]["sha256"] != "e2444a96128f33575bffab0f9420a1844c2f6cf2d3b60ef309f1fcad9b7eecb3":
        problems.append("guards.json sha")
    # The recorded aggregate uses PowerShell culture sort; an ordinal sort provably gives another value
    # (documented in materials/aggregate-sort-note.txt). Records expectation, not a failure.
    OUT["aggregate_ordinal_caliber"] = OUT["guards_json"]["aggregate_recomputed"]
    OUT["aggregate_ordinal_matches_documented"] = (
        OUT["guards_json"]["aggregate_recomputed"]
        == "89895a621bd154122c9f4e19d8b67f825309397019215b35b117566a11b0007b")
    if not OUT["aggregate_ordinal_matches_documented"]:
        problems.append("aggregate ordinal caliber not reproduced")
    if OUT["guards_json"]["aggregate_sha256"] != facts.get("guards_aggregate_sha256"):
        problems.append("aggregate vs facts")
    if OUT["sha_guards"]["rows"] != 73 or OUT["sha_guards"]["bad"]:
        problems.append("sha-guards rows/bad")
    if len(stash_rows) != 9:
        problems.append("stash count")
    for r in stash_rows:
        if exe_checked and r["sha256"] not in exe_checked.values():
            problems.append(f"stash sha not in sha-guards: {r['name']}")
    if not OUT["entities"]["all_exit_zero"] or OUT["entities"]["count"] != 9:
        problems.append("entities")
    for g, gv in groups.items():
        for cam in ("cam0", "cam1"):
            rg = gv.get(f"{cam}_rgb", {})
            dg = gv.get(f"{cam}_depth_seg", {})
            if rg.get("mismatched") or rg.get("max_byte_diff") or rg.get("beyond_1_lsb"):
                problems.append(f"rgb {g} {cam}")
            if dg.get("depth_mismatch") or dg.get("max_ulp") or dg.get("beyond_ulp") or dg.get("seg_mismatch"):
                problems.append(f"depth/seg {g} {cam}")
    zo = {f"{g}.{cam}": gv[f"{cam}_depth_seg"]["depth_zero"] for g, gv in groups.items() for cam in ("cam0", "cam1")}
    out["depth_zero"] = zo
    if set(zo.values()) != {29776, 37050}:
        problems.append("depth_zero values")
    if len(run_shas) != 2 or len({v["captured_at"] for v in run_shas.values()}) != 1:
        problems.append("run sha lines")
    for g, rv in refs.items():
        if not rv["all_match"] or not rv["all_in_s3a_manifest"]:
            problems.append(f"reference shas {g}")
    if refs["hfield-only"]["captured_at"] != refs["hfield-primitive"]["captured_at"]:
        problems.append("captured_at differs")
    if OUT["manifest"]["bad_committed"]:
        problems.append("manifest committed-blob rows")
    if OUT["protected_diff"]:
        problems.append("protected diff non-empty")
    if not OUT["attempt1_present"]:
        problems.append("attempt1 missing")
    gi = OUT.get("guard_inputs_vs_attempt1", {})
    if not gi.get("identical"):
        problems.append("64 inputs differ from attempt1")
    if OUT.get("old_guards_sha256") != "210f82088702ce0ea00bb357a88b1c75948f94b9ef48a632ad76546be1ba287d":
        problems.append("superseded guards archive sha")
    OUT["problems"] = problems

    pathlib.Path(r"D:\Rust\mjwarp-rs\target\qa-s3b-val\recompute.json").write_text(
        json.dumps(OUT, ensure_ascii=False, indent=1), encoding="utf-8", newline="\n")

    print("== monitor normal ==", OUT["monitor_normal"]["recomputed"])
    print("== monitor normal recorded ==", OUT["monitor_normal"]["recorded"])
    print("== monitor exception ==", OUT["monitor_exception"]["recomputed"])
    print("== monitor exception recorded ==", OUT["monitor_exception"]["recorded"])
    print("== guard.log ==", OUT["guard_log"]["lines"])
    print("== precheck utils ==", pre_utils, "any>=40:", OUT["precheck_has_ge_40_sample"])
    print("== window samples ==", s)
    print("== guards.json ==", {k: v for k, v in OUT["guards_json"].items() if k != "entries"})
    print("== sha-guards ==", {k: (v if k != "bad" else v[:3]) for k, v in OUT["sha_guards"].items()})
    print("== stash ==", [(r["name"], r["sha256"][:8], r["bytes"]) for r in stash_rows])
    print("== entities ==", OUT["entities"])
    print("== hfield groups ==", json.dumps(groups, ensure_ascii=False))
    print("== run shas ==", run_shas)
    print("== references ==", json.dumps(refs, ensure_ascii=False)[:800])
    print("== manifest ==", OUT["manifest"])
    print("== chain diff ==", OUT["chain_diff"])
    print("== protected diff ==", OUT["protected_diff"] or "(empty)")
    print("== attempt1 ==", OUT["attempt1_present"], OUT["attempt1_files"], OUT["attempt2_files"])
    print("== attempt1 facts ==", OUT["attempt1_facts_head"])
    print("== guard inputs vs a1 ==", {k: v for k, v in gi.items() if k != "diff"}, gi.get("diff", [])[:3])
    print("== old guards sha ==", OUT.get("old_guards_sha256"))
    print("== groups ==", OUT["capture_group"], OUT["legacy_group"], OUT["shadow_group"])
    print("== depth zero ==", zo)
    print("== facts guards_aggregate ==", facts.get("guards_aggregate_sha256"), "| sha_guards:", facts.get("sha_guards"))
    print("PROBLEMS", problems)
    return 0


out = OUT
if __name__ == "__main__":
    sys.exit(main())
