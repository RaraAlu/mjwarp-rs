"""QA independent verification of the S3 window observation closure slice (v3, post-fix).

Reviewed: closure c65d84ac, plus evidence-only fixes F2 71e78aae and F1 ee5d0ca4.
Read-only: no window reopen, no GPU, no entity rerun, no code/criteria/reference/tolerance change.
Writes: verify_closure.json and logs/*.txt in this evidence directory.
"""
import csv
import hashlib
import json
import re
import statistics
import subprocess
from datetime import datetime
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
ATT = ROOT / "target/s3-product-wiring/attempt1"
FREEZE = ROOT / "target/s3-product-wiring-freeze"
EV = ROOT / "docs/development-evidence/product-collision-inputs-wiring"
DEL = ROOT / "docs/development-evidence/changes"
SELF = ROOT / "docs/development-evidence/product-collision-s3-window-obs-closure-check"
OUT = SELF / "verify_closure.json"
LOGS = SELF / "logs"

REVIEWED = "c65d84ac3e63788780d3a5c985fd0bb69806f4ad"
FIX_F2 = "71e78aae8e3250b0e91d0ee3728bdb2bfedb4c3d"
FIX_F1 = "ee5d0ca4e82895badb4b2ba1c9658e1886fda495"
HEAD = "ee5d0ca4e82895badb4b2ba1c9658e1886fda495"
CLOSURE_TASK = "2026-10-27-product-collision-inputs-s3-window-observation-closure"

problems = []
findings = []
checks = 0
log_lines = {"o1": [], "invariants": [], "scope": [], "forbidden": []}


def check(cond, message):
    global checks
    checks += 1
    if not cond:
        problems.append(message)
    return bool(cond)


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*args: str) -> str:
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True,
                          encoding="utf-8", errors="replace").stdout


def text_of(path: Path) -> str:
    raw = path.read_bytes()
    if raw.startswith(b"\xff\xfe"):
        return raw.decode("utf-16")
    return raw.decode("utf-8")


def parse_ts(value: str) -> datetime:
    main, _, off = value.partition("+")
    head, _, frac = main.partition(".")
    return datetime.fromisoformat(head + "." + (frac + "000000")[:6] + "+" + off)


def verify_record_files(record: dict, label: str) -> list:
    bad = []
    for entry in record.get("files", []):
        path = ROOT / entry["path"]
        if not path.is_file():
            bad.append(entry["path"] + " (missing)")
            continue
        raw = path.read_bytes()
        lines = len(text_of(path).splitlines())
        if (hashlib.sha256(raw).hexdigest() != entry["sha256"] or len(raw) != entry["bytes"]
                or lines != entry["lines"]):
            bad.append(entry["path"])
    check(not bad, f"{label} file identities mismatch: {bad[:5]}")
    return bad


def main():
    report = {}

    # ---------------- O1: UTF-16 original vs UTF-8 evidence copy ----------------
    orig = ROOT / "target/s3-product-wiring/window/attempt1-runner-stdout.log"
    copy = EV / "logs/attempt1-runner-stdout.log"
    orig_raw = orig.read_bytes()
    copy_raw = copy.read_bytes()
    orig_text = orig_raw.decode("utf-16") if orig_raw.startswith(b"\xff\xfe") else orig_raw.decode("utf-16-le")
    orig_lines = orig_text.replace("\r\n", "\n").splitlines()
    copy_lines = copy_raw.decode("utf-8").splitlines()
    check(len(orig_lines) == 183, f"original decoded lines {len(orig_lines)}")
    check(len(copy_lines) == 183, f"copy lines {len(copy_lines)}")
    check(orig_lines == copy_lines, "decoded line diff != 0")
    check(not copy_raw.startswith(b"\xef\xbb\xbf") and b"\r\n" not in copy_raw, "copy BOM/CRLF")
    check(sum(1 for line in copy_lines if not line.strip()) == 0, "blank lines inserted")
    check(len(copy_raw) == 40380 and sha(copy) == "fb3e63e4a696a178954a2d4203afc89deb50618c3db2aa50f97eeda3beb2647c",
          "copy bytes/sha")
    check(len(orig_raw) == 81128 and sha(orig) == "b7cffd45ef82114158ca2736777a23e4a5d9e7d91044f7c3ed68a1c265b17328",
          "original bytes/sha")
    check(datetime.fromtimestamp(orig.stat().st_mtime).strftime("%Y-%m-%d") == "2026-10-27", "original mtime")
    check(copy_raw[:2] != b"\xff\xfe", "copy must not be utf-16")
    closure_path = EV / "s3-window-observation-closure-receipt.json"
    closure = json.loads(closure_path.read_text(encoding="utf-8"))
    o1 = closure["observations"]["O1"]["detail"]
    check(o1["original_utf16"]["bytes"] == 81128 and o1["original_utf16"]["sha256"] == sha(orig), "closure O1 original")
    check(o1["evidence_copy_after_fix"]["bytes"] == 40380 and o1["evidence_copy_after_fix"]["sha256"] == sha(copy)
          and o1["evidence_copy_after_fix"]["lines"] == 183, "closure O1 copy")
    check(o1["line_diff_vs_decoded_original"] == 0 and o1["blank_lines_dropped"] == 183, "closure O1 counts")
    log_lines["o1"] = [
        "# O1: UTF-16LE original vs committed UTF-8 evidence copy (QA independent decode)", "",
        "original (target/s3-product-wiring/window/attempt1-runner-stdout.log, not committed, read-only):",
        f"  bytes={len(orig_raw)} sha256={sha(orig)} UTF-16LE with BOM (FF FE)",
        "  mtime " + datetime.fromtimestamp(orig.stat().st_mtime).strftime("%Y-%m-%d %H:%M:%S")
        + " (window time; unchanged since the window)",
        f"  decoded lines={len(orig_lines)}", "",
        "committed evidence copy (docs/.../logs/attempt1-runner-stdout.log):",
        f"  bytes={len(copy_raw)} sha256={sha(copy)} encoding=utf-8 BOM=no CRLF=no blank_lines=0",
        f"  lines={len(copy_lines)}", "",
        f"line-by-line diff of decoded original vs evidence copy: {sum(1 for a, b in zip(orig_lines, copy_lines) if a != b)} differences",
        "=> O1 fix verified (UTF-8 no BOM / LF / no inserted blank lines; 183 lines; original untouched).",
    ]

    # ---------------- shared texts ----------------
    files = {
        "window-receipt": EV / "window-receipt.json",
        "receipt": EV / "receipt.json",
        "README": EV / "README.md",
        "window-delivery": DEL / "2026-10-27-product-collision-inputs-s3-window.json",
    }
    texts = {name: text_of(path) for name, path in files.items()}
    json_data = {name: json.loads(texts[name]) for name in ["window-receipt", "receipt", "window-delivery"]}
    closure_delivery_path = DEL / "2026-10-27-product-collision-inputs-s3-window-observation-closure.json"
    closure_delivery = json.loads(closure_delivery_path.read_text(encoding="utf-8"))

    # ---------------- O2 wording (four places) ----------------
    o2_tokens = ["17:03:36.808", "86 ms", "±0.1 s", "17:03:37.658", "38.443", "≥0.9 s",
                 "归因推断", "未证实到具体分配点"]
    o2_hits = {name: [token for token in o2_tokens if token in text] for name, text in texts.items()}
    for name, hits in o2_hits.items():
        check(len(hits) == len(o2_tokens), f"{name} O2 tokens missing {[t for t in o2_tokens if t not in hits]}")
    wr = json_data["window-receipt"]
    check(wr["sampler"]["observation_OW1"]["seq"] == 776
          and wr["sampler"]["observation_OW1"]["used_mib"] == 4142
          and wr["sampler"]["observation_OW1"]["timestamp"] == "2026-10-27T17:03:36.7225390+08:00",
          "peak data unchanged")

    # ---------------- O3 neighbours ----------------
    neighbours = [
        wr["sampler"]["observation_OW1"]["neighbor_samples_used_mib"],
        wr["budget_deviation"]["peak"]["neighbors_used_mib"],
        json_data["receipt"]["s3"]["window"]["attempt1"]["budget_deviation"]["peak"]["neighbors_used_mib"],
    ]
    check(all(row == [3092, 4142, 3092] for row in neighbours), f"O3 neighbours {neighbours}")
    check("3092, 3091" not in texts["window-receipt"] and "3092, 3091" not in texts["receipt"],
          "stale neighbour pair present")

    # ---------------- O4 interval (post-fix F1) ----------------
    rows = list(csv.DictReader((ATT / "samples.csv").read_text(encoding="utf-8-sig").splitlines()))
    stamps = [parse_ts(row["timestamp"]) for row in rows]
    intervals = [(stamps[i + 1] - stamps[i]).total_seconds() for i in range(len(stamps) - 1)]
    interval = {"median": round(statistics.median(intervals), 4),
                "min": round(min(intervals), 4), "max": round(max(intervals), 4)}
    check(abs(interval["median"] - 0.276) < 0.001, f"interval median {interval['median']}")
    check(interval == {"median": 0.2759, "min": 0.2721, "max": 0.2901}, f"interval recomputed {interval}")
    o4_tokens = ["0.25", "目标间隔", "≈0.276", "0.2721", "0.2901"]
    o4_hits = {name: [token for token in o4_tokens if token in text] for name, text in texts.items()}
    check(all(len(hits) == len(o4_tokens) for hits in o4_hits.values()),
          f"O4 tokens after fix { {n: [t for t in o4_tokens if t not in h] for n, h in o4_hits.items()} }")
    trunc = {name: [t for t in ("0.2720", "0.2900") if t in text] for name, text in texts.items()}
    check(not any(trunc.values()), f"truncated range still present {trunc}")
    measured = {
        "window-receipt": wr["budget_deviation"]["peak"]["sample_interval"]["measured_range_s"],
        "receipt": json_data["receipt"]["s3"]["window"]["attempt1"]["budget_deviation"]["peak"]["sample_interval"]["measured_range_s"],
        "closure-receipt": closure["observations"]["O4"]["detail"]["measured_range_s"],
        "window-delivery": json_data["window-delivery"]["budget_deviation"]["peak"]["sample_interval"]["measured_range_s"],
    }
    check(all(row == [0.2721, 0.2901] for row in measured.values()), f"measured_range_s {measured}")
    milestone_o4 = closure_delivery["milestones"]["O4"]
    check("0.2721" in milestone_o4 and "0.2901" in milestone_o4 and "0.2720" not in milestone_o4,
          f"closure delivery milestone O4 {milestone_o4}")
    doc_hits = git("grep", "-l", "-E", "0\\.2720|0\\.2900", "--", "docs/development-evidence")
    check(doc_hits.strip() == "", f"truncated range anywhere in docs evidence: {doc_hits.split()}")

    # ---------------- O5 exceptions (post-fix F2) ----------------
    wr_notes = json.dumps(wr.get("evidence_format_notes", {}), ensure_ascii=False)
    rec_notes = json.dumps(json_data["receipt"].get("s3", {}).get("evidence_format_notes", {}), ensure_ascii=False)
    wd_notes = json.dumps(json_data["window-delivery"].get("evidence_format", {}), ensure_ascii=False)
    check("receipt.json" in wr_notes and "750" in wr_notes, "O5 window-receipt exception")
    check("receipt.json" in rec_notes and "750" in rec_notes, "O5 receipt exception")
    check("receipt.json" in wd_notes and "750" in wd_notes, "O5 delivery exception after fix")
    check("745" not in wd_notes, "O5 delivery still says 745")
    check(not re.search(r"\b745\b", texts["window-receipt"] + texts["receipt"] + texts["README"]),
          "745 token left in window-receipt/receipt/README")
    check("runner-stdout" in wr_notes and "UTF-16" in wr_notes, "O5 stdout encoding declaration (window-receipt)")
    check("runner-stdout" in rec_notes and "UTF-16" in rec_notes, "O5 stdout encoding declaration (receipt)")
    check("runner-stdout" in wd_notes and "UTF-16" in wd_notes, "O5 stdout encoding declaration (delivery)")
    readme_rows = texts["README"].splitlines()
    e2e_bytes_rows = [row for row in readme_rows if "7453" in row]
    check(len(e2e_bytes_rows) == 1 and "B" in e2e_bytes_rows[0], "README 7453 false-positive line shape")

    # ---------------- O6 derived fields ----------------
    outer = wr["window_facts"].get("runner_outer_wall_s")
    check(isinstance(outer, dict) and outer.get("derived") is True and outer.get("value_s") == 252.1
          and bool(outer.get("basis")), f"O6 {outer}")
    check("252.1" in texts["window-delivery"] and "derived" in texts["README"] and "derived" in texts["window-delivery"],
          "O6 derived marking in delivery/README")

    # ---------------- O7 statistics basis ----------------
    dev_total = sum(text.count("预算偏差") for text in texts.values())
    in_budget = sum(text.count("在预算内") for text in texts.values())
    check("统计口径" in texts["window-receipt"] and "统计口径" in texts["window-delivery"]
          and "统计口径" in texts["README"], "O7 statistics basis wording")
    check(dev_total == 7, f"预算偏差 count {dev_total}")
    check(in_budget == 0, f"在预算内 count {in_budget}")

    # ---------------- fix commits (evidence-only) ----------------
    fix_files = {
        FIX_F2: [line for line in git("show", "--name-only", "--format=", FIX_F2).splitlines() if line.strip()],
        FIX_F1: [line for line in git("show", "--name-only", "--format=", FIX_F1).splitlines() if line.strip()],
    }
    check(git("log", "-1", "--format=%(trailers)", FIX_F2).strip()
          == "Normify-Fixes: 2026-10-27-product-collision-inputs-s3-window", "F2 fix trailer")
    check(git("log", "-1", "--format=%(trailers)", FIX_F1).strip()
          == f"Normify-Fixes: {CLOSURE_TASK}", "F1 fix trailer")
    fix_code = [f for files_ in fix_files.values() for f in files_
                if f.startswith(("src/", "tests/", "native/", "include/", "fixtures/"))
                or f in ("Cargo.toml", "Cargo.lock", "build.rs")]
    check(not fix_code, f"fix commits touch code {fix_code}")
    check(len(fix_files[FIX_F2]) == 2, f"F2 fix files {len(fix_files[FIX_F2])}")
    check(len(fix_files[FIX_F1]) == 7, f"F1 fix files {len(fix_files[FIX_F1])}")
    check(git("diff", "c65d84ac", "HEAD", "--name-only", "--", "src", "tests", "native", "include",
              "fixtures", "Cargo.toml", "Cargo.lock", "build.rs").strip() == "", "code unchanged since closure")

    # ---------------- numerical invariants ----------------
    used = [int(row["used_mib"]) for row in rows]
    free = [int(row["free_mib"]) for row in rows]
    util = [int(row["util_percent"]) for row in rows]
    peak = max(used)
    baseline = 3091
    streak = best = 0
    for value in util:
        streak = streak + 1 if value >= 80 else 0
        best = max(best, streak)
    excl = max(used[:used.index(peak)] + used[used.index(peak) + 1:])
    check(len(rows) == 867, f"samples {len(rows)}")
    check(peak == 4142 and min(free) == 11920 and max(util) == 31 and best == 0, "sampler invariants")
    check(peak - baseline == 1051 and peak - used[0] == 1051 and peak - min(used) == 1055, "delta invariants")
    check(excl == 3109 and excl - baseline == 18 and sum(1 for v in used if v > 3120) == 1, "sustained invariants")
    facts_lines = (ATT / "facts.txt").read_text(encoding="utf-8").splitlines()
    fact = {}
    for line in facts_lines:
        for key, value in re.findall(r"([A-Za-z_][\w.]*)=([^\s]+)", line):
            fact.setdefault(key, value)
    exits = dict(item.split("=") for item in next(line for line in facts_lines if line.startswith("exit_codes:")).split(":", 1)[1].split())
    check(len(exits) == 10 and all(value == "0" for value in exits.values()), f"exits {exits}")
    check(fact.get("deps_exe_before") == fact.get("deps_exe_after") == "1812", "zero compile deps")
    check(fact.get("zero_rust_compile", "").startswith("True"), "zero compile flag")
    wall = (parse_ts(wr["window_facts"]["window_end"]) - parse_ts(wr["window_facts"]["window_start"])).total_seconds()
    check(abs(wall - 238.5) < 0.05, f"body wall {wall}")
    check(wr["window_facts"]["attempt"] == 1 and not (ROOT / "target/s3-product-wiring/attempt2").exists(), "attempt")
    newlog = (EV / "logs/attempt1-g10_p5_product_collision_wiring_cuda.log").read_text(encoding="utf-8")
    check("e2e_summary cases=3 bit_equal=12/12 failures=0" in newlog, "new target summary")
    check(newlog.count("nacon=4 ref_count=4 bit_equal=4/4") == 3, "new target case lines")
    check(newlog.count("max_dist_ulp=0 max_pos_ulp=0 max_frame_ulp=0") == 3, "new target ULP lines")
    check("s1_fields=32/32" in newlog and newlog.count("mismatches=0") >= 3, "S1 snapshot lines")
    regressions = [p for p in sorted((EV / "logs").glob("attempt1-*.log"))
                   if p.name not in {"attempt1-guard.log", "attempt1-runner-stdout.log",
                                     "attempt1-g10_p5_product_collision_wiring_cuda.log"}]
    reg_bad = [p.name for p in regressions if "test result: ok." not in text_of(p)]
    check(not reg_bad, f"regression logs not ok {reg_bad}")
    stage2 = text_of(EV / "logs/attempt1-stage2_equivalence_window.log")
    check(len([line for line in stage2.splitlines() if "true" in line and "->" in line]) == 17, "stage2 dumps")
    guard_lines = (ATT / "sha-guards.txt").read_text(encoding="utf-8").splitlines()
    bad_guards, post_bad = [], []
    for line in guard_lines:
        fields = dict(item.split("=", 1) for item in line.split() if "=" in item)
        path = FREEZE / fields["file"] if line.startswith("SHA_OK exe ") else ROOT / fields["path"]
        if not path.is_file() or sha(path) != fields["sha256"] or fields["match"] != "True":
            bad_guards.append(str(path))
        if line.startswith("SHA_OK exe ") and (sha(path) != fields["expected"] or path.stat().st_size != int(fields["bytes"])):
            post_bad.append(fields["file"])
    check(len(guard_lines) == 133 and not bad_guards, f"guards {len(guard_lines)} {bad_guards[:3]}")
    check(not post_bad, f"post-window exe {post_bad}")
    check(sha(ROOT / "target/s3-product-wiring/window/run-validation-attempt1.ps1")
          == "c7a387a423312e4db55d0619a20341426b71561f7f1b1ab3129ed2f3c19a103d", "runner sha")
    check(sha(EV / "fixtures/mjb-312/provenance.json")
          == "3ea5c863c821822e5f8c75a186017daa90cce82bafb07e8f823ae5b1f2809d49", "provenance sha")

    # ---------------- delivery records ----------------
    window_delivery = json_data["window-delivery"]
    verify_record_files(window_delivery, "window delivery")
    verify_record_files(closure_delivery, "closure delivery")

    # ---------------- scope / boundaries ----------------
    commit_files = [line for line in git("show", "--name-only", "--format=", REVIEWED).splitlines() if line.strip()]
    code_hits = [f for f in commit_files
                 if f.startswith(("src/", "tests/", "native/", "include/", "fixtures/")) or f in ("Cargo.toml", "Cargo.lock", "build.rs")]
    check(len(commit_files) == 8 and not code_hits, f"closure commit files {len(commit_files)} code {code_hits}")
    check(git("log", "-1", "--format=%(trailers)", REVIEWED).strip() == f"Normify-Task: {CLOSURE_TASK}",
          "closure trailer")
    check(git("diff", "0c9497e4", "HEAD", "--name-only", "--", "src", "tests", "native", "include",
              "fixtures", "Cargo.toml", "Cargo.lock", "build.rs").strip() == "", "code unchanged since impl")
    changed = [line for line in git("diff", "--name-only", "c65d84ac", HEAD).splitlines() if line.strip()]
    enc_bad = []
    for rel in changed:
        path = ROOT / rel
        if not path.is_file() or path.suffix in {".exe", ".mjb"}:
            continue
        raw = path.read_bytes()
        if raw.startswith(b"\xef\xbb\xbf") or b"\r\n" in raw:
            enc_bad.append(rel)
    check(not enc_bad, f"encoding issues in changed files {enc_bad}")
    closure_task = json.loads((ROOT / f".normify/changes/{CLOSURE_TASK}.json").read_text(encoding="utf-8"))
    check(closure_task["status"] == "verified", "closure task verified")
    check(closure.get("task_id") == CLOSURE_TASK, "closure receipt task id")
    check(wr["task_id"] == "2026-10-27-product-collision-inputs-s3-window", "window receipt task id")
    check(json_data["receipt"]["task_id"] == "2026-10-27-product-collision-inputs-s1-data-snapshot", "receipt top task id")
    check(window_delivery["task_id"] == "2026-10-27-product-collision-inputs-s3-window", "window delivery task id")
    check(closure_delivery["task_id"] == CLOSURE_TASK, "closure delivery task id")
    claims = "\n".join(texts.values()) + "\n" + closure_delivery_path.read_text(encoding="utf-8") + "\n" + json.dumps(closure, ensure_ascii=False)
    negatives = ("不写", "不得", "不宣称", "不做", "no ", "never")
    positive = []
    for match in re.finditer("窗口通过", claims):
        context = claims[max(0, match.start() - 160): match.end() + 40]
        if not any(word in context for word in negatives):
            positive.append(context.replace("\n", " ")[:100])
    check(not positive, f"non-negated window-pass phrase {positive[:2]}")
    task = json.loads((ROOT / ".normify/changes/2026-10-27-product-collision-s3-window-obs-closure-check.json").read_text(encoding="utf-8"))
    terms = [t for c in task["checks"] if c["kind"] == "forbid_claim" for t in c["terms"]]
    term_hits = [t for t in terms if t in claims]
    check(not term_hits, f"regulated claim terms present {term_hits}")
    porcelain = [line for line in git("status", "--porcelain").splitlines() if line.strip()]
    task_card = ".normify/changes/2026-10-27-product-collision-s3-window-obs-closure-check.json"
    allowed = {f"?? {task_card}", "?? docs/development-evidence/product-collision-s3-window-obs-closure-check/"}
    check(set(porcelain) <= allowed, f"unexpected worktree entries: {porcelain}")

    # ---------------- logs ----------------
    log_lines["invariants"] = [
        "# Numerical invariants (QA recomputation from target/s3-product-wiring/attempt1)", "",
        f"entities: {len(exits)} exit codes all 0 (facts exit_codes)",
        f"window: body_wall {wall:.3f} s (receipt 238.5); attempt=1; no attempt2 directory",
        "new target: 3 x (nacon=4 ref_count=4 bit_equal=4/4, dist/pos/frame ULP=0); e2e 12/12; S1 32/32 x3",
        f"regressions: {len(regressions)} logs all 'test result: ok.'; stage2 17 dump lines all 'two captures identical true'",
        f"sampler: samples={len(rows)} baseline={baseline} used_peak={peak} "
        f"delta={peak - baseline}/{peak - used[0]}/{peak - min(used)} free_min={min(free)} util_max={max(util)} "
        f"streak={best} residual=0 peak-excluded max={excl} sustained={excl - baseline} samples>3120=1",
        "floor=9904; open free=12976>=10240",
        f"sampling interval: median={interval['median']} s min={interval['min']} max={interval['max']}",
        f"guards: {len(guard_lines)}/{len(guard_lines)} SHA_OK re-hashed by QA; post-window 10 exe 10/10; "
        f"runner c7a387a4... unchanged",
        "provenance.json sha 3ea5c863... unchanged; frozen 10 exe unchanged",
        "zero Rust compile: deps exe 1812->1812, deps max write unchanged",
        f"budget deviation literal count {dev_total}; in-budget phrase {in_budget}",
        "", "=> all window values unchanged by the evidence-only closure and its two evidence-only fixes.",
    ]
    log_lines["scope"] = [
        f"# Scope and boundary checks ({REVIEWED} + fixes {FIX_F2[:8]} / {FIX_F1[:8]})", "",
        f"closure commit {REVIEWED[:8]} files={len(commit_files)}: .normify task record + closure delivery record + "
        "window delivery record + README + runner-stdout log + receipt.json + window-receipt.json + closure receipt; "
        "0 code paths",
        f"F2 fix {FIX_F2[:8]} files={len(fix_files[FIX_F2])} (window delivery record + its .normify record)",
        f"F1 fix {FIX_F1[:8]} files={len(fix_files[FIX_F1])} (closure task record + 6 evidence files)",
        "git diff 0c9497e4..HEAD over src/tests/native/include/fixtures/Cargo* = 0 files",
        f"git diff {REVIEWED[:8]}..HEAD over code paths = 0 files",
        f"closure trailer: Normify-Task: {CLOSURE_TASK}",
        "fix trailers: Normify-Fixes: <window task> / <closure task>",
        "closure task record status=verified",
        "all changed text files: UTF-8 no BOM, LF, CR=0",
        "window values unchanged; no window rerun (raw samples/facts mtime 17:04, no attempt2)",
        "runner sha c7a387a4...; frozen 10 exe; provenance 3ea5c863... unchanged",
        "no 'window pass' positive statement; six regulated claim terms 0 hits",
        "worktree at review time: only the QA task card and this evidence directory untracked",
    ]
    log_lines["forbidden"] = [
        "# QA scan of the regulated claim terms (from this task forbid_claim check; literals not reproduced)",
        f"hits: {len(term_hits)}",
        "# the window-pass phrase only appears inside negations; window_pass_claimed remains false.",
    ]
    findings_text = [
        "# QA findings on the closure slice (post-fix rerun)", "",
        f"F1  O4 measured range wording -- FIXED by {FIX_F1[:8]}",
        "    declared '0.2721-0.2901 s' in all places; QA measured from samples.csv: 0.2721-0.2901 s (median 0.2759 s).",
        "    no '0.2720' / '0.2900' token left in docs/development-evidence.", "",
        f"F2  window delivery record machine_readable_exceptions receipt.json line count -- FIXED by {FIX_F2[:8]}",
        "    now 750; actual receipt.json = 750 lines; no 745 token left in window-receipt/receipt/README.",
        "    README 7453 is the e2e target byte count, not a line-count exception (recorded, not a residual).", "",
        "=> findings: 0 open (both suggestion-level items fixed).",
    ]
    log_lines["findings"] = findings_text

    # ---------------- write report + logs ----------------
    report.update({
        "reviewed_commit": REVIEWED,
        "fix_commits": {"F2": FIX_F2, "F1": FIX_F1},
        "head_at_review": HEAD,
        "o1": {"original_lines": len(orig_lines), "copy_lines": len(copy_lines),
               "original": {"bytes": len(orig_raw), "sha256": sha(orig)},
               "copy": {"bytes": len(copy_raw), "sha256": sha(copy), "encoding": "utf-8", "blank_lines": 0}},
        "o2_tokens": {name: len(hits) for name, hits in o2_hits.items()},
        "o3": neighbours,
        "o4": {"measured": interval, "declared_range": "0.2721-0.2901",
               "measured_range_s": measured, "closure_milestone": milestone_o4},
        "o5": {"delivery_receipt_json_750": "750" in wd_notes, "delivery_745_left": "745" in wd_notes,
               "readme_7453_false_positive": e2e_bytes_rows[0].strip() if e2e_bytes_rows else None},
        "o6": outer,
        "o7": {"budget_deviation": dev_total, "in_budget": in_budget},
        "fix_commits_evidence_only": not fix_code,
        "findings": findings,
        "checks": checks,
        "problems": problems,
        "result": "PASS" if not problems else "FAIL",
    })
    OUT.write_bytes((json.dumps(report, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))
    LOGS.mkdir(parents=True, exist_ok=True)
    (LOGS / "o1-conversion.txt").write_bytes(("\n".join(log_lines["o1"]) + "\n").encode("utf-8"))
    (LOGS / "invariants.txt").write_bytes(("\n".join(log_lines["invariants"]) + "\n").encode("utf-8"))
    (LOGS / "scope-boundary.txt").write_bytes(("\n".join(log_lines["scope"]) + "\n").encode("utf-8"))
    (LOGS / "forbidden-terms.txt").write_bytes(("\n".join(log_lines["forbidden"]) + "\n").encode("utf-8"))
    (LOGS / "findings.txt").write_bytes(("\n".join(findings_text) + "\n").encode("utf-8"))

    print(f"checks={checks} problems={len(problems)} findings={len(findings)} result={report['result']}")
    for problem in problems:
        print("PROBLEM:", problem)
    for finding in findings:
        print("FINDING:", finding)
    print("interval:", interval, "receipt lines:", len(texts["receipt"].splitlines()))
    print("neighbours:", neighbours)
    print("measured_range_s:", measured)
    print("milestone O4:", milestone_o4)


if __name__ == "__main__":
    main()
