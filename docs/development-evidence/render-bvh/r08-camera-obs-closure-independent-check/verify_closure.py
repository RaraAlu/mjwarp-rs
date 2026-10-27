"""QA independent check of the R08-S1 observation closure (O1-O8 disposition + zero numeric regression).

Read-only. Writes target/qa-r08-obs-closure/recompute.json.
"""
import hashlib
import json
import re
import subprocess
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
EV = ROOT / "docs/development-evidence/render-bvh/r08-camera-output"
OUT = ROOT / "target/qa-r08-obs-closure"
REVIEWED = "f305842e"
VERIFIED = "d92797fc"
PRE_CLOSURE = "8156591f"
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


def lf(rel):
    return (ROOT / rel).read_bytes().count(b"\n")


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    readme = (EV / "README.md").read_text(encoding="utf-8")
    receipt = json.loads((EV / "receipt.json").read_text(encoding="utf-8"))
    closure = json.loads((EV / "s3-obs-closure-receipt.json").read_text(encoding="utf-8"))
    code = (ROOT / "src/render/context/camera_output.rs").read_text(encoding="utf-8")
    tests = (ROOT / "src/render/context/tests.rs").read_text(encoding="utf-8")
    report = {}

    # ---------- O2: depth_scale first ----------
    body = code.split("pub fn depth(", 1)[1].split("Ok(CameraImage::new", 1)[0]
    first_check = body.index("DepthScale")
    check(first_check < body.index("depth_raw"), "O2 depth_scale check after buffer lookup")
    check(first_check < body.index("resolution("), "O2 depth_scale check after resolution lookup")
    for token in ("depth_scale_validated_before_channel_and_index_lookups",
                  "ChannelDisabled", "CameraIndex", "DepthScale"):
        check(token in tests, f"O2 test coverage token missing {token}")
    check("错误优先级（观察收口 O2）" in readme, "O2 README priority section")
    check("`depth_scale` 先于相机/通道/缓冲查找校验" in code, "O2 module doc priority")

    # ---------- O3: clamp wording downgraded ----------
    for token in ("不是上游语义保证", "CUDA `fminf/fmaxf`", "未覆盖", "无 `-0`/NaN"):
        check(token in readme, f"O3 README token missing {token}")
    check("不是上游语义保证" in code and "未覆盖（O3/O7）" in code, "O3 module doc wording")
    check("不是上游语义保证" in receipt["api"]["depth_clamp_semantics"], "O3 receipt wording")

    # ---------- O7: two boundary tests, product-only, uncovered stated ----------
    check("depth_boundary_values_are_product_clamp_not_cuda_equivalence" in tests, "O7 boundary test present")
    check("宣称等价" in tests and "未覆盖" in tests, "O7 no-equivalence/uncovered statement in test")
    check("f32::clamp" in tests and "NaN" in tests, "O7 rust clamp difference recorded")

    # ---------- accept items + O8 ----------
    dispositions = {x["id"]: x["action"] for x in closure["dispositions"]}
    check(dispositions.get("O1") == "accept" and dispositions.get("O4") == "accept"
          and dispositions.get("O5") == "accept" and dispositions.get("O6") == "accept",
          f"accept dispositions {dispositions}")
    check(dispositions.get("O2") in ("fix-code", "fix") and dispositions.get("O3") in ("fix-wording", "fix")
          and dispositions.get("O7") in ("fix-tests", "fix"), "fix dispositions")
    check(dispositions.get("O8") == "reference", "O8 disposition")
    check("2026-10-28-repo-fixture-eol-attributes" in json.dumps(closure, ensure_ascii=False), "O8 task id")
    check(closure.get("task_id") == "2026-10-28-render-bvh-r08-camera-obs-closure", "closure task id")

    # ---------- line counts / ratchet ----------
    lines = {p: lf(p) for p in (
        "src/render/context/camera_output.rs", "src/render/context/tests.rs",
        "src/render/context.rs", "tests/render_camera_output.rs",
        "src/render/kernel/launch.rs", "src/render/kernel/megakernel.rs",
        "src/render/kernel/cast.rs", "src/model/render.rs",
    )}
    expected = {"src/render/context/camera_output.rs": 499, "src/render/context/tests.rs": 379,
                "src/render/context.rs": 426, "tests/render_camera_output.rs": 493,
                "src/render/kernel/launch.rs": 500, "src/render/kernel/megakernel.rs": 498,
                "src/render/kernel/cast.rs": 494, "src/model/render.rs": 500}
    check(lines == expected, f"line counts {lines}")
    check(lines["src/render/context/camera_output.rs"] <= 500, "camera_output <=500")
    report["lines"] = lines
    report["ratchet_headroom"] = 500 - lines["src/render/context/camera_output.rs"]

    # ---------- zero-change red lines ----------
    red = git("diff", "--name-only", f"{REVIEWED}^..{VERIFIED}", "--",
              "src/model/render.rs", "src/render/kernel/cuda", "src/render/kernel/launch.rs",
              "src/render/kernel/output.rs", "src/render/kernel/cast.rs",
              "src/render/kernel/megakernel.rs", "src/render/context.rs",
              "fixtures", "Cargo.toml", "Cargo.lock", "native", "include")
    check(red.strip() == "", f"red-line paths changed {red.split()}")
    changed = [line for line in git("diff", "--name-only", f"{PRE_CLOSURE}..{VERIFIED}").splitlines() if line.strip()]
    code_changed = [f for f in changed if f.startswith(("src/", "tests/"))]
    check(code_changed == ["src/render/context/camera_output.rs", "src/render/context/tests.rs"],
          f"code changed {code_changed}")
    commit1 = [f for f in git("show", "--name-only", "--format=", REVIEWED).splitlines() if f.strip()]
    commit2 = [f for f in git("show", "--name-only", "--format=", VERIFIED).splitlines() if f.strip()]
    check(len(commit1) == 13 and len(commit2) == 6, f"closure commit files {len(commit1)}/{len(commit2)}")

    # ---------- numeric non-regression: contract from the S1 pass review ----------
    check(readme.count("499 行") >= 1 and "379 行" in readme, "README line-count updatess")
    # archived closure gate logs
    unit_lines = (EV / "logs/closure/unit.log").read_text(encoding="utf-8", errors="replace").splitlines()
    passed = failed = ignored = results = 0
    for line in unit_lines:
        m = re.match(r"^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored", line)
        if m:
            results += 1
            passed += int(m.group(2))
            failed += int(m.group(3))
            ignored += int(m.group(4))
    check((passed, failed, ignored, results) == (1017, 0, 382, 125),
          f"archived closure unit {(passed, failed, ignored, results)}")
    directed = (EV / "logs/closure/directed-depth-priority.log").read_text(encoding="utf-8", errors="replace")
    check("2 passed" in directed, "archived directed depth-priority log")

    # ---------- evidence format ----------
    enc_bad = []
    for p in sorted(EV.rglob("*")):
        if p.is_file() and p.suffix in (".md", ".json", ".txt", ".log", ".csv"):
            raw = p.read_bytes()
            if raw[:3] == b"\xef\xbb\xbf" or b"\r\n" in raw:
                enc_bad.append(p.relative_to(EV).as_posix())
    check(not enc_bad, f"encoding issues {enc_bad[:5]}")
    terms = [t for c in json.loads((ROOT / ".normify/changes/2026-10-28-render-bvh-r08-camera-obs-closure-check.json")
                                   .read_text(encoding="utf-8"))["checks"]
             if c.get("kind") == "forbid_claim" for t in c["terms"]]
    text = readme + json.dumps(receipt, ensure_ascii=False) + json.dumps(closure, ensure_ascii=False)
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
    print("lines:", lines)
    print("changed files:", len(changed), "code:", code_changed)


if __name__ == "__main__":
    main()
