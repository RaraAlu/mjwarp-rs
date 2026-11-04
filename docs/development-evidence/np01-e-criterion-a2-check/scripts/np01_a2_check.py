#!/usr/bin/env python3
"""NP01 E criterion A2 (commit b9cb2993) QA check.

Read-only. Reproduces what the QA review README claims:
  1. contract diff is additive only (+20/-0 on the contract)
  2. existing atol/rtol table + measured worst + E/redline formula untouched
  3. new section carries the signed criterion verbatim (scope, 97 scalars, no E ratio)
  4. no native-scope claim; native diagnostic semantics kept
  5. wording rule (from A2 on) + history snapshot rule
  6. signature record present; evidence manifest 2/2; no src/tests/fixtures diff
  7. G10 closeout fix already follows the A2 wording rule
Run: python np01_a2_check.py   (from repo root)
"""
import glob
import hashlib
import json
import os
import re
import subprocess

REPO = "D:/Rust/mjwarp-rs"
A2 = "b9cb2993"
CONTRACT = "docs/numerical-precision-contract.md"
EVID = "docs/development-evidence/np01-e-criterion-a2"
RECOMPUTE = "docs/development-evidence/np01-e-recompute/README.md"
fails = []


def ok(label, cond, detail=""):
    print(("PASS " if cond else "FAIL ") + label + (" :: " + detail if detail else ""))
    if not cond:
        fails.append(label)


def git(*args):
    return subprocess.run(["git"] + list(args), cwd=REPO, capture_output=True,
                          text=True, encoding="utf-8").stdout


def sha256(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


os.chdir(REPO)
print("== repo", git("rev-parse", "--short", "HEAD").strip())
head = open(CONTRACT, encoding="utf-8").read()
parent = git("show", A2 + "^:" + CONTRACT)

# 1. additive-only diff
print("\n[1] contract diff is additive only")
num = git("show", "--numstat", "--format=", A2, "--", CONTRACT).split()
ok("contract added/deleted = 20/0", num[:2] == ["20", "0"], str(num[:2]))
diff = [l for l in git("show", "--format=", A2, "--", CONTRACT).splitlines()
        if l.startswith(("+", "-"))]
removed = [l for l in diff if l.startswith("-") and not l.startswith("---")]
ok("no existing contract line removed", removed == [], str(removed))
ok("contract file changed only in A2 commit",
   git("log", "--oneline", "-1", "--", CONTRACT).startswith(A2))

# 2. untouched numbers / formulas
print("\n[2] existing numbers and formulas untouched")
table_lines = [l for l in head.splitlines() if re.match(r"\| (dist|pos（各分量）|frame/normal|结构字段)", l)]
table_parent = [l for l in parent.splitlines() if re.match(r"\| (dist|pos（各分量）|frame/normal|结构字段)", l)]
ok("atol/rtol table rows identical to parent", table_lines == table_parent, str(len(table_lines)) + " rows")
ok("atol table has 7 rows", len(table_lines) == 7)
formula = [l for l in head.splitlines() if re.search(r"^(t_i|E_mjwarp|redline)\s*=", l)]
formula_parent = [l for l in parent.splitlines() if re.search(r"^(t_i|E_mjwarp|redline)\s*=", l)]
ok("E_mjwarp/redline formula lines identical", formula == formula_parent, str(formula))

# 3. new section verbatim
print("\n[3] new section content")
sec = head[head.index("### E指标定稿"):]
sec = sec[:sec.index("## 测试项与报告标注")]
for token in ["用户于2026-11-05拍板选项①", "同码跨后端核对", "签名级",
              "实测worst ≤ 已签atol", "不使用E比值作判据",
              "`E_mjwarp ≡ 0`", "1.05 * E_mjwarp", "CONVEX/SDF两路径",
              "dist/pos/frame", "normal并入frame", "97个标量",
              "不宣称native口径", "仅诊断", "不新增数值、不改已签atol",
              "A2 `<提交号>`"]:
    ok("section has: " + token, token in sec)

# 4. evidence
print("\n[4] A2 evidence dir")
for f in ("README.md", "receipt.json", "manifest.txt"):
    ok("evidence file exists " + f, os.path.isfile(EVID + "/" + f))
raw = open(EVID + "/manifest.txt", "rb").read()
ok("manifest header/EOL/BOM", raw.startswith(b"#") and b"\xef\xbb\xbf" not in raw and b"\r\n" not in raw)
n = 0
for line in raw.decode("utf-8").splitlines():
    if not line.strip() or line.startswith("#"):
        continue
    h, b, rel = line.split()
    ok("manifest entry " + rel, sha256(rel) == h and str(os.path.getsize(rel)) == b)
    n += 1
ok("manifest entries = 2", n == 2, str(n))
r = json.load(open(EVID + "/receipt.json", encoding="utf-8"))
ok("receipt task_id", r["task_id"] == "2026-11-05-precision-np01-e-criterion-a2")
ok("receipt signature", r["signature"]["signed_at"] == "2026-11-05"
   and "option 1" in r["signature"]["option"] and r["signature"]["level"] == "signature-level")
ok("receipt scope paths/fields", r["criterion"]["scope"]["paths"] == ["CONVEX", "SDF"]
   and r["criterion"]["scope"]["fields"] == ["dist", "pos", "frame"])
ok("receipt sample = 97", r["criterion"]["sample_scalars"] == 97)
ok("receipt no native claim", r["criterion"]["native_scope_claimed"] is False
   and r["criterion"]["native_diagnostic_only_semantics_kept"] is True)
ok("receipt additive_only", r["contract_change"]["additive_only"] is True
   and r["contract_change"]["existing_lines_modified"] == 0)

# 5. basis pointer
print("\n[5] recompute basis")
rec = open(RECOMPUTE, encoding="utf-8").read()
for token in ["97 个标量", "6/6 项实测 worst ≤ 已签 atol", "1.80×", "same_code_backend_baseline"]:
    ok("recompute README has: " + token, token in rec)

# 6. no code/fixture change
print("\n[6] no code/fixture change")
names = git("show", "--name-only", "--format=", A2).split()
ok("A2 touches no src/tests/fixtures",
   not any(p.startswith(("src/", "tests/", "fixtures/")) for p in names), ",".join(names))
print("  files:", ", ".join(names))

# 7. wording rule used by later closeout
print("\n[7] later closeout follows wording rule")
g10 = open("docs/development-evidence/g10-p5-summary/README.md", encoding="utf-8").read()
ok("G10 summary writes signed + A2 commit", "已签" in g10 and "A2 `b9cb2993`" in g10)

# 8. known stale-but-by-design line
print("\n[8] stale historical line (observation)")
ok("old 'E尚未定稿' line kept as history (expected, reported as O1)",
   "边界：**E尚未定稿**" in head)

# 9. encoding audit of this evidence dir (logs are written by run_logged.py as UTF-8/LF)
print("\n[9] encoding audit (evidence files except logs)")
for p in sorted(glob.glob("docs/development-evidence/np01-e-criterion-a2-check/**/*", recursive=True)):
    if not os.path.isfile(p) or "/logs/" in p.replace("\\", "/"):
        continue
    b = open(p, "rb").read()
    ok("utf8 no BOM no CR " + os.path.basename(p),
       not b.startswith((b"\xef\xbb\xbf", b"\xff\xfe", b"\xfe\xff")) and b"\r" not in b)

print("\nRESULT:", "PASS" if not fails else "FAIL", "fails=" + ",".join(fails))
raise SystemExit(0 if not fails else 1)
