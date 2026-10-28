"""QA recompute part 2: freeze integrity, manifest, hygiene, consistency flags."""
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


def blob(p):
    r = subprocess.run(["git", "show", f"HEAD:{p}"], cwd=REPO, capture_output=True)
    return r.stdout if r.returncode == 0 else None


def btext(p):
    b = blob(p)
    return b.decode("utf-8", errors="replace") if b is not None else None


def sha_file(p):
    return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()


def sha_bytes(b):
    return hashlib.sha256(b).hexdigest()


def main():
    guards_raw = (REPO / "scripts/validate/render-bvh/guards.json").read_bytes()
    guards = json.loads(guards_raw.decode("utf-8-sig"))
    # 1. triple equality live == guards == stash
    triple = []
    for e in guards["frozen_exes"]:
        live = REPO / e["path"]
        stash = REPO / e["stash_path"]
        row = {
            "entity": e["entity"],
            "guard_sha_equals_stash_field": e["sha256"] == e["stash_sha256"],
            "live_sha_matches": live.exists() and sha_file(live) == e["sha256"],
            "live_bytes_matches": live.exists() and live.stat().st_size == e["bytes"],
            "stash_sha_matches": stash.exists() and sha_file(stash) == e["sha256"],
            "stash_bytes_matches": stash.exists() and stash.stat().st_size == e["bytes"],
            "live_mtime": live.stat().st_mtime_ns if live.exists() else None,
        }
        triple.append(row)
    OUT["frozen_exes_triple"] = triple

    # 2. sha-guards rows vs guards
    sg = [l.split() for l in btext(f"{EV}/attempt2/sha-guards.txt").splitlines() if l.strip()]
    gmap = {e["path"]: (e["sha256"], e["bytes"]) for e in guards["frozen_exes"]}
    gmap.update({e["path"]: (e["sha256"], e["bytes"]) for e in guards["guard_inputs"]})
    sgmap = {r[3]: (r[1], int(r[2])) for r in sg}
    OUT["sha_guards_vs_guards"] = {
        "rows": len(sg),
        "same_key_set": set(sgmap) == set(gmap),
        "all_match": all(sgmap.get(k) == v for k, v in gmap.items()),
        "tags": sorted(set(r[0] for r in sg)),
    }

    # 3. manifest
    man = btext(f"{EV}/manifest.txt")
    rows = [l for l in man.splitlines() if l and not l.startswith("#")]
    bad_c, bad_d, selfref = [], [], False
    for l in rows:
        digest, size, path = l.split()
        size = int(size)
        if path == f"{EV}/manifest.txt":
            selfref = True
        b = blob(path)
        if b is None or sha_bytes(b) != digest or len(b) != size:
            bad_c.append(path)
        p = REPO / path
        if not p.exists():
            bad_d.append({"path": path, "why": "missing"})
        else:
            wb = p.read_bytes()
            if sha_bytes(wb) != digest or len(wb) != size:
                bad_d.append({"path": path, "why": "mismatch"})
    tree = subprocess.run(["git", "ls-tree", "-r", "--name-only", "HEAD", f"{EV}/"],
                          cwd=REPO, capture_output=True).stdout.decode().split()
    listed = {l.split()[-1] for l in rows}
    OUT["manifest"] = {
        "rows": len(rows),
        "bad_committed": bad_c,
        "bad_disk": bad_d,
        "self_reference": selfref,
        "unlisted": sorted(p for p in tree if p not in listed and not p.endswith("manifest.txt")),
        "extra_listed": sorted(p for p in listed if p not in tree),
        "covers_scripts": all(f"scripts/validate/render-bvh/{n}" in listed for n in
                              ("guards.json", "run-window.ps1", "freeze-guards.ps1", "monitor-until-clean.ps1")),
    }

    # 4. hygiene: all committed evidence blobs
    files = [p for p in tree]
    hyg = []
    for f in files:
        b = blob(f)
        try:
            b.decode("utf-8")
            u8 = True
        except UnicodeDecodeError:
            u8 = False
        if b.count(b"\x00") or b.count(b"\r") or not u8 or b[:3] == b"\xef\xbb\xbf":
            hyg.append({"path": f, "nul": b.count(b"\x00"), "cr": b.count(b"\r"), "utf8": u8})
    # 禁词词表：用码点构造，避免在证据文件里出现词表字面（6 条全局 + 本卡 forbid_claim 4 条）
    forb = ["".join(chr(c) for c in seq) for seq in [[71, 49, 48, 24050, 23454, 29616], [23436, 25972, 29289, 29702, 36890, 36807], [21452, 24179, 21488, 36890, 36807], [19978, 28216, 23436, 20840, 31561, 20215], [21407, 29983, 25918, 34892], [31934, 24230, 36798, 26631], [31383, 21475, 36890, 36807], [20135, 21697, 28210, 26579, 39564, 25910, 36890, 36807], [19978, 28216, 31561, 20215]]]
    hits = []
    for f in files:
        t = blob(f).decode("utf-8", errors="replace")
        h = [w for w in forb if w in t]
        if h:
            hits.append({"path": f, "words": h})
    OUT["hygiene"] = {"files": len(files), "non_lf_or_non_utf8": hyg, "forbidden_hits": hits}

    # 5. guard_aggregate.json vs facts parity
    ga = json.loads(btext(f"{EV}/attempt2/guard_aggregate.json"))
    facts = {}
    for line in btext(f"{EV}/attempt2/facts.txt").splitlines():
        if "=" in line and not line.startswith("#"):
            k, v = line.split("=", 1)
            facts[k.strip()] = v.strip()
    ftxt = btext(f"{EV}/attempt2/facts.txt")
    def fnum(pat):
        m = re.search(pat, ftxt)
        return int(m.group(1)) if m else None
    pairs = {
        "samples": (ga["samples"], fnum(r"samples=(\d+)")),
        "used_peak_mib": (ga["used_peak_mib"], fnum(r"used_peak_mib=(\d+)")),
        "peak_seq": (ga["peak_seq"], fnum(r"peak_seq=(\d+)")),
        "used_min_mib": (ga["used_min_mib"], fnum(r"used_min_mib=(\d+)")),
        "free_min_mib": (ga["free_min_mib"], fnum(r"free_min_mib=(\d+)")),
        "util_max_percent": (ga["util_max_percent"], fnum(r"util_max_percent=(\d+)")),
        "floor_mib": (ga["floor_mib"], fnum(r"floor_mib=(\d+)")),
        "baseline_free_mib": (ga["baseline_free_mib"], fnum(r"baseline_free_mib=(\d+)")),
        "guards_aggregate_sha256": (ga["guards_aggregate_sha256"],
                                     (re.search(r"guards_aggregate_sha256=(\w+)", ftxt) or [None]).group(1) if re.search(r"guards_aggregate_sha256=(\w+)", ftxt) else None),
        "sha_guards": (ga["sha_guards"], "73/73"),
    }
    OUT["aggregate_vs_facts"] = {k: {"json": a, "facts": b, "match": a == b} for k, (a, b) in pairs.items()}
    OUT["aggregate_vs_facts"]["all_match"] = all(v["match"] for v in OUT["aggregate_vs_facts"].values()
                                                 if isinstance(v, dict) and "match" in v)

    # 6. exe mtimes vs guards generated_at
    gen = guards["generated_at"]
    OUT["guards_generated_at"] = gen
    OUT["exe_mtimes"] = {r["entity"]: r["live_mtime"] for r in triple}

    # 7. attempt1 untouched (mtimes well before attempt2 window start)
    a1 = WIN / "attempt1"
    OUT["attempt1"] = {
        "files": sorted(p.name for p in a1.glob("*")),
        "max_mtime": max(p.stat().st_mtime for p in a1.glob("*")),
    }
    OUT["attempt2_window_start_epoch"] = (WIN / "attempt2").stat().st_mtime

    # 8. guard-manifest.txt rows
    gm = btext(f"{EV}/materials/guard-manifest.txt")
    gm_rows = [l for l in gm.splitlines() if l.strip()]
    gm_data = [l for l in gm_rows if not l.startswith("#")]
    OUT["guard_manifest"] = {"rows": len(gm_rows), "data_rows": len(gm_data), "head": gm_rows[:3], "tail": gm_rows[-2:]}

    # 9. stale-text detection in README / receipt (contradictions with executed window)
    readme = btext(f"{EV}/README.md")
    receipt = btext(f"{EV}/receipt.json")
    OUT["stale_text"] = {
        "readme_header_says_window_not_opened": "窗口未开、未跑 GPU、未跑 `--ignored`" in readme,
        "readme_old_guards_sha_210f8208": "210f8208" in readme,
        "readme_old_aggregate_105ad0ab": "105ad0ab" in readme,
        "readme_old_freeze_script_7773": "7 773" in readme,
        "readme_old_monitor_2486": "2 486" in readme,
        "readme_has_new_guards_sha_e2444a96": "e2444a96" in readme,
        "readme_says_precheck_37_40_40": "37/40/40" in readme,
        "receipt_authoring_only_true": '"authoring_only": true' in receipt,
        "receipt_window_false": '"window": false' in receipt,
        "receipt_limits_says_no_gpu_entity": "no GPU entity was executed" in receipt,
        "receipt_precheck_samples_37_40_40": '"precheck_util_samples": [\n      37,\n      40,\n      40\n    ]' in receipt,
        "receipt_has_not_a_normal_threshold_pass": '"not_a_normal_threshold_pass": true' in receipt,
        "receipt_has_authoring_exception_false": '"authoring_exception_used": false' in receipt,
    }
    OUT["exception_block"] = btext(f"{EV}/attempt2/exception-block.txt")
    OUT["freeze_attempt2_says_no_override"] = "不带任何例外覆盖" in btext(f"{EV}/materials/freeze-attempt2.txt")

    # 10. git info attributes for guards.json
    ia = REPO / ".git/info/attributes"
    OUT["git_info_attributes"] = ia.read_text(encoding="utf-8", errors="replace") if ia.exists() else None
    lf_norm = guards_raw.replace(b"\r\n", b"\n")
    OUT["guards_lf_normalized_sha256"] = sha_bytes(lf_norm)

    # 11. exception monitor / normal monitor vs target originals
    mon_pairs = {}
    for n in ("monitor-pre-window-attempt2-final.csv", "monitor-pre-window-attempt2-final-result.txt",
              "monitor-pre-window-attempt2-exception.csv", "monitor-pre-window-attempt2-exception-result.txt"):
        evp = REPO / EV / "monitor" / n
        rawp = WIN / n
        mon_pairs[n] = {"evidence": sha_file(evp), "raw": sha_file(rawp) if rawp.exists() else None,
                        "match": rawp.exists() and sha_file(evp) == sha_file(rawp)}
    OUT["monitor_copies"] = mon_pairs

    pathlib.Path(r"D:\Rust\mjwarp-rs\target\qa-s3b-val\recompute2.json").write_text(
        json.dumps(OUT, ensure_ascii=False, indent=1), encoding="utf-8", newline="\n")

    problems = []
    if not all(r["guard_sha_equals_stash_field"] and r["live_sha_matches"] and r["stash_sha_matches"]
               and r["live_bytes_matches"] and r["stash_bytes_matches"] for r in triple):
        problems.append("frozen exe triple equality")
    if not (OUT["sha_guards_vs_guards"]["same_key_set"] and OUT["sha_guards_vs_guards"]["all_match"]
            and OUT["sha_guards_vs_guards"]["rows"] == 73):
        problems.append("sha-guards vs guards")
    if OUT["manifest"]["rows"] != 40 or OUT["manifest"]["bad_committed"] or OUT["manifest"]["bad_disk"] \
            or OUT["manifest"]["self_reference"] or OUT["manifest"]["unlisted"]:
        problems.append("manifest")
    if OUT["hygiene"]["non_lf_or_non_utf8"] or OUT["hygiene"]["forbidden_hits"]:
        problems.append("hygiene")
    if not OUT["aggregate_vs_facts"]["all_match"]:
        problems.append("guard_aggregate vs facts")
    if OUT["guard_manifest"]["data_rows"] != 82:
        problems.append("guard-manifest data rows (expect 9*2 exe + 64 inputs = 82)")
    if not all(v["match"] for v in mon_pairs.values()):
        problems.append("monitor copies")
    OUT["problems"] = problems

    print("triple:", [(r["entity"], r["live_sha_matches"], r["stash_sha_matches"]) for r in triple])
    print("sha-guards vs guards:", OUT["sha_guards_vs_guards"])
    print("manifest:", OUT["manifest"])
    print("hygiene:", OUT["hygiene"])
    print("agg vs facts:", OUT["aggregate_vs_facts"])
    print("guard-manifest rows:", OUT["guard_manifest"]["rows"], "data:", OUT["guard_manifest"]["data_rows"])
    print("attempt1:", OUT["attempt1"])
    print("stale:", json.dumps(OUT["stale_text"], ensure_ascii=False, indent=1))
    print("freeze_attempt2_says_no_override:", OUT["freeze_attempt2_says_no_override"])
    print("git info attributes:", OUT["git_info_attributes"])
    print("guards lf-normalized sha:", OUT["guards_lf_normalized_sha256"])
    print("monitor copies:", {k: v["match"] for k, v in mon_pairs.items()})
    print("gen at:", gen, "exe mtimes:", OUT["exe_mtimes"])
    print("PROBLEMS", problems)
    return 0


if __name__ == "__main__":
    sys.exit(main())
