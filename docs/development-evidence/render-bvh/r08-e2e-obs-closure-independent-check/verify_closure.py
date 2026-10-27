"""QA independent check of the R08-S2 observation closure (O1-O4 wording + zero numeric change).

Read-only. Writes target/qa-r08-closure/recompute.json.
"""
import hashlib
import json
import re
import subprocess
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
EV = ROOT / "docs/development-evidence/render-bvh/r08-camera-e2e"
RAW = ROOT / "target/r08-s2-window"
OUT = ROOT / "target/qa-r08-closure"
BASE = "10301dc6"   # QA pass-version of the S2 evidence
HEAD = "254a49c4"

problems = []
checks = 0


def check(cond, label):
    global checks
    checks += 1
    if not cond:
        problems.append(label)


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def git(*args):
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True,
                          encoding="utf-8", errors="replace").stdout


def show(commit, path):
    return git("show", f"{commit}:{path}")


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    readme = (EV / "README.md").read_text(encoding="utf-8")
    receipt = json.loads((EV / "obs-closure-receipt.json").read_text(encoding="utf-8"))
    notes = {
        "attempt1_facts": (EV / "logs/attempt1/facts-note.txt").read_text(encoding="utf-8"),
        "attempt2_facts": (EV / "logs/attempt2/facts-note.txt").read_text(encoding="utf-8"),
        "attempt1_guards": (EV / "logs/attempt1/sha-guards-note.txt").read_text(encoding="utf-8"),
    }
    report = {}

    # ---------- O1 wording by round ----------
    old_phrases = ["各两次", "no-op 语义断言全部成立", "attempt1 起即成立", "两个 renderer 均 `Ok`"]
    real_assertions = [p for p in old_phrases if p in readme
                       and not any(p in line and ("已删除" in line or "删除" in line)
                                   for line in readme.splitlines())]
    check(not real_assertions, f"O1 old over-strong phrase used as a claim {real_assertions}")
    o1_tokens = ["attempt1 已通过", "mega", "depth_seg", "attempt1 未执行", "read(Rgb/Depth/Segmentation).is_err()",
                 "只在 **attempt2**", "Ok×3", "各两次"]
    check("attempt1 已通过" in readme and "depth_seg.render(..)` **一次** `Ok`" in readme,
          "O1 per-round attempt1 wording")
    check("attempt1 未执行" in readme and "只在 **attempt2**" in readme, "O1 uncleared assertions marked")
    check(readme.count("`Ok`×3") >= 2, "O1 Ok x3 count")
    report["o1"] = {"ok_x3": readme.count("`Ok`×3"), "old_phrases_meta_quoted":
                    [p for p in old_phrases if p in readme]}

    # ---------- O2 historical sha note ----------
    note = notes["attempt1_guards"]
    check("6a73c51a7c020217d0c49fa5da272e61e36fa6cc16f4bff2fd0f038cae25065d" in note
          and "94027937" in note.replace("94027937", "94027937"), "O2 old sha/bytes in note")
    check("94027937" in note or "94 027 937" in note, "O2 old bytes formatting")
    check("918b5851efaf7f2494201419ae3f90d7bb39e612414d68b7a833bedd01952775" in note
          or "918b5851" in note, "O2 new sha in note")
    check("94029705" in note, "O2 new bytes in note")
    check("44/45" in note and "预期" in note, "O2 44/45 expected")
    check("6a73c51a" in readme and "44/45" in readme, "O2 README note")
    report["o2"] = {"note_sha_old": "6a73c51a" in note, "note_44_45": "44/45" in note}

    # ---------- O3 sustained definition ----------
    check("sustained_delta_mib" in readme and "peak − baseline" in readme, "O3 definition in README")
    check("4560−4285" in readme and "4593−4323" in readme, "O3 arithmetic in README")
    check("S3 窗口" in readme and "不同" in readme, "O3 S3 difference")
    for label, note_text, arithmetic in (
        ("attempt1", notes["attempt1_facts"], "4560 − 4285 = 275"),
        ("attempt2", notes["attempt2_facts"], "4593 − 4323 = 270"),
    ):
        check("peak" in note_text and "baseline" in note_text, f"O3 note definition {label}")
        check(arithmetic in note_text, f"O3 note arithmetic {label}")
        check("S3" in note_text, f"O3 note S3 difference {label}")
    # numbers still agree with facts files
    f1 = (EV / "logs/attempt1/facts.txt").read_text(encoding="utf-8")
    f2 = (EV / "logs/attempt2/facts.txt").read_text(encoding="utf-8")
    check("sustained_delta_mib=275" in f1 and "sustained_delta_mib=270" in f2, "O3 facts values")
    report["o3"] = {"readme": "peak − baseline" in readme}

    # ---------- O4 guards-check relationship, factual ----------
    gc = RAW / "guards-check"
    g2 = RAW / "guards-check-a2"
    same, diff = [], []
    for name in ("sha-guards.txt", "guard.log", "baseline.txt", "facts.txt", "machine-load.txt"):
        a, b = gc / name, g2 / name
        same.append(name) if sha(a) == sha(b) else diff.append(name)
    check(same == ["sha-guards.txt", "guard.log"], f"O4 identical set {same}")
    check(diff == ["baseline.txt", "facts.txt", "machine-load.txt"], f"O4 differing set {diff}")
    check("逐字节相同" in readme and "baseline.txt" in readme and "machine-load.txt" in readme,
          "O4 README factual wording")
    check("全部内容相同" not in readme and "完全相同" not in readme, "O4 README must not claim full equality")
    report["o4"] = {"identical": same, "different": diff}

    # ---------- zero numeric change vs the pass version ----------
    tokens = ["7 实体", "6 passed / 0 failed", "145", "4322", "4593", "11469", "delta 270 / 248 / 271",
              "floor 8667", "beyond_1_lsb=0", "120000"]
    base_readme = show(BASE, "docs/development-evidence/render-bvh/r08-camera-e2e/README.md")
    for token in ("145", "4322", "4593", "11469", "8667", "270", "248", "271", "101", "8705", "4285", "11777"):
        check(token in readme and token in base_readme, f"zero-change token missing {token}")
    # numeric tables unchanged: compare the facts/samples/guard files between BASE and HEAD
    unchanged = ["logs/attempt1/samples.csv", "logs/attempt2/samples.csv",
                 "logs/attempt1/facts.txt", "logs/attempt2/facts.txt",
                 "logs/attempt1/guard_aggregate.json", "logs/attempt2/guard_aggregate.json",
                 "logs/attempt1/entity-results.csv", "logs/attempt2/entity-results.csv",
                 "logs/attempt1/sha-guards.txt", "logs/attempt2/sha-guards.txt",
                 "logs/attempt2/render_r08_cameras.log", "logs/attempt1/render_r08_cameras.log"]
    changed = [rel for rel in unchanged
               if show(BASE, f"docs/development-evidence/render-bvh/r08-camera-e2e/{rel}")
               != show(HEAD, f"docs/development-evidence/render-bvh/r08-camera-e2e/{rel}")]
    check(not changed, f"numeric artifacts changed {changed}")
    # per evidence-file change list: only README + 3 notes + receipt + task card
    diff_files = [line for line in git("diff", "--name-only", f"{BASE}..{HEAD}").splitlines() if line.strip()]
    check(len(diff_files) == 6, f"closure commit changes {len(diff_files)} files")
    allowed = {"docs/development-evidence/render-bvh/r08-camera-e2e/README.md",
               "docs/development-evidence/render-bvh/r08-camera-e2e/logs/attempt1/facts-note.txt",
               "docs/development-evidence/render-bvh/r08-camera-e2e/logs/attempt1/sha-guards-note.txt",
               "docs/development-evidence/render-bvh/r08-camera-e2e/logs/attempt2/facts-note.txt",
               "docs/development-evidence/render-bvh/r08-camera-e2e/obs-closure-receipt.json",
               ".normify/changes/2026-10-28-render-bvh-r08-e2e-obs-closure.json"}
    check(set(diff_files) == allowed, f"closure touched unexpected files {diff_files}")
    check(not [f for f in diff_files if f.startswith(("src/", "tests/", "fixtures/"))
               or f in ("Cargo.toml", "Cargo.lock")], "closure touches code paths")

    # runner + frozen exes unchanged
    runner = RAW / "run-window-attempt1.ps1"
    check(runner.stat().st_size == 14976 and
          sha(runner) == "c8786951674365cfabba62fa668e92254c58092c30214dba08b9a5d347f4e71a", "runner unchanged")
    frozen = {}
    for line in (RAW / "frozen-shas.txt").read_text(encoding="utf-8").splitlines():
        if line.startswith("#") or not line.strip():
            continue
        parts = line.split()
        frozen[parts[0]] = (parts[2], int(parts[3]), parts[4])
    bad = [name for name, (path, size, digest) in frozen.items()
           if not (ROOT / path).is_file() or (ROOT / path).stat().st_size != size or sha(ROOT / path) != digest]
    check(len(frozen) == 7 and not bad, f"frozen exe changed {bad}")

    # ---------- receipt self-checks and closure receipt ----------
    check(receipt.get("task_id") == "2026-10-28-render-bvh-r08-e2e-obs-closure", "closure receipt task id")
    gate = receipt.get("gate", {})
    check(gate.get("not_applicable") is True or gate.get("not_applicable") == "true", "closure gate not applicable")
    checks_map = {o: receipt.get("observations", {}).get(o, {}) for o in ("O1", "O2", "O3", "O4")}
    report["receipt_observations"] = {o: checks_map[o].get("disposition") if isinstance(checks_map[o], dict) else None
                                     for o in checks_map}

    # ---------- encoding / forbidden terms ----------
    enc_bad = []
    for p in [EV / "README.md", EV / "obs-closure-receipt.json",
              EV / "logs/attempt1/facts-note.txt", EV / "logs/attempt1/sha-guards-note.txt",
              EV / "logs/attempt2/facts-note.txt"]:
        raw = p.read_bytes()
        if raw[:3] == b"\xef\xbb\xbf" or b"\r\n" in raw:
            enc_bad.append(p.name)
    check(not enc_bad, f"encoding issues {enc_bad}")
    terms = [t for c in json.loads((ROOT / ".normify/changes/2026-10-28-render-bvh-r08-e2e-obs-closure-check.json")
                                  .read_text(encoding="utf-8"))["checks"]
             if c.get("kind") == "forbid_claim" for t in c["terms"]]
    text = readme + json.dumps(receipt, ensure_ascii=False) + "\n".join(notes.values()) + json.dumps(
        json.loads((EV / "manifest.json").read_text(encoding="utf-8")), ensure_ascii=False)
    check(not [t for t in terms if t in text], "forbidden terms present")
    wording = ["窗口通过", "已覆盖", "精度达标", "上游等价"]
    negatives = ("不写", "不得", "不宣称", "不做", "未", "不复写", "禁词", "约束")
    bad_wording = []
    for term in wording:
        for match in re.finditer(term, text):
            context = text[max(0, match.start() - 120):match.end() + 40]
            if not any(word in context for word in negatives):
                bad_wording.append(term)
    check(not bad_wording, f"non-negated wording {bad_wording}")

    report["checks"] = checks
    report["problems"] = problems
    report["result"] = "PASS" if not problems else "FAIL"
    (OUT / "recompute.json").write_bytes(
        (json.dumps(report, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))
    print(f"checks={checks} problems={len(problems)} result={report['result']}")
    for problem in problems:
        print("PROBLEM:", problem)
    print("o4:", report["o4"], "o2:", report["o2"])


if __name__ == "__main__":
    main()
