# -*- coding: utf-8 -*-
"""QA independent check: attempt5 criteria allowlist consistency.

Read-only. Objects:
  material   ecc93b56 : docs/development-evidence/render-bvh/r06-s3b-flex-validate/
  signature  3406f1ca : .normify/changes/2026-11-04-render-r06-s3b-flex-allowlist-signature.json
                        + docs/development-evidence/render-bvh/r06-s3b-flex-allowlist/README.md
  QA support a3f690df : docs/development-evidence/render-bvh/r06-s3b-flex-f32-order-check/DECISION-SUPPORT-user-signature.md

Checks: minimal scope wording, no-expansion phrases, bit-first completeness, D8 separation,
freeze-input answer, receipt material sha256/bytes, non-claims, signature metadata.

Usage: python qa_criteria_check.py [--out <logfile>]
"""
import hashlib
import io
import json
import os
import re
import sys

REPO = r"D:\Rust\mjwarp-rs"
MAT = os.path.join(REPO, "docs", "development-evidence", "render-bvh", "r06-s3b-flex-validate")
SIG = os.path.join(REPO, "docs", "development-evidence", "render-bvh", "r06-s3b-flex-allowlist")
QA = os.path.join(REPO, "docs", "development-evidence", "render-bvh", "r06-s3b-flex-f32-order-check")
CARDS = os.path.join(REPO, ".normify", "changes")

if "--out" in sys.argv:
    _fh = open(sys.argv[sys.argv.index("--out") + 1], "w", encoding="utf-8", newline="\n")
    _orig = sys.stdout

    class _Tee(object):
        def write(self, s):
            _fh.write(s)
            _orig.write(s)
            return len(s)

        def flush(self):
            _fh.flush()
            _orig.flush()

    sys.stdout = _Tee()


def sha(b):
    return hashlib.sha256(b).hexdigest()


def read(p, binary=False):
    d = open(p, "rb").read()
    return d if binary else d.decode("utf-8")


def git_head():
    import subprocess
    return subprocess.run(["git", "rev-parse", "HEAD"], cwd=REPO, capture_output=True, text=True).stdout.strip()


print("== 0. environment ==")
print("python:", sys.version.split()[0], "| repo HEAD:", git_head())

crit = read(os.path.join(MAT, "materials", "attempt5-criteria.md"))
readme = read(os.path.join(MAT, "README.md"))
sig_readme = read(os.path.join(SIG, "README.md"))
sig_card = json.loads(read(os.path.join(CARDS, "2026-11-04-render-r06-s3b-flex-allowlist-signature.json")))
receipt = json.loads(read(os.path.join(MAT, "receipt.json")))
qa_doc = read(os.path.join(QA, "DECISION-SUPPORT-user-signature.md"))

print("\n== 1. signature card metadata ==")
print("card status:", sig_card["status"], "| owner:", sig_card["schedule"]["owner"])
acc = sig_card.get("acceptance", [])
print("acceptance items:", len(acc))
print("signature evidence dir files:", sorted(os.listdir(SIG)))

print("\n== 2. receipt material sha256/bytes vs current files ==")
ma = receipt["attempt5_acceptance"]
ok_all = True
for rel, exp in ma["material_sha256"].items():
    b = read(os.path.join(MAT, *rel.split("/")), binary=True)
    ok = sha(b) == exp["sha256"] and len(b) == exp["bytes"]
    ok_all &= ok
    print("  %-32s %s bytes=%d sha=%s" % (rel, "OK" if ok else "MISMATCH", len(b), sha(b)[:16]))
print("  all material hashes OK:", ok_all)

print("\n== 3. allowlist minimal scope wording (three sources) ==")
scope = {
    "material": crit,
    "signature_readme": sig_readme,
    "qa_decision_support": qa_doc,
}
for name, text in scope.items():
    checks = {
        "cam2": "cam2" in text,
        "pixel_134_128": ("(134,128)" in text) or ("(x=134, y=128)" in text) or ("(x=134,y=128)" in text),
        "flexid0": "flexid0" in text,
        "ulp_le_4": ("ULP ≤ 4" in text) or ("ULP≤4" in text) or ("ULP <= 4" in text) or ("ULP≤4" in text),
        "beyond4_zero": ("beyond4 = 0" in text) or ("beyond4=0" in text),
        "beyond4_must_zero_clause": ("beyond4 ≠ 0" in text) or ("beyond4 不为 0" in text) or ("beyond4 不为 0 即失败" in text),
    }
    missing = [k for k, v in checks.items() if not v]
    print("  %-20s missing=%s" % (name, missing if missing else "none"))

print("\n== 4. no-expansion phrase scan (must be ABSENT in material/signature) ==")
forbidden = ["4 ULP 即可", "≤4 ULP 即可", "邻近像素", "adjacent", "neighbor", "相邻像素"]
for name, text in (("material_criteria", crit), ("material_readme", readme), ("signature_readme", sig_readme)):
    hits = [p for p in forbidden if p in text]
    print("  %-20s hits=%s" % (name, hits if hits else "none"))

print("\n== 5. rest bit-first completeness (material §2 table + README §9) ==")
need = {
    "rgb_mismatched0": "mismatched = 0" in crit,
    "rgb_max_byte_diff0": "max_byte_diff = 0" in crit,
    "rgb_beyond_1_lsb0": "beyond_1_lsb = 0" in crit,
    "depth_max_ulp_le4": "max_ulp ≤ 4" in crit,
    "depth_beyond_0_ulp": "beyond_0_ulp = 0" in crit,
    "cam0_cam1_covered": ("cam0 / cam1" in crit),
    "cam2_except_pixel": ("除 `(134,128)` 外全部像素" in crit) or ("除 `(134,128)` 外" in crit),
    "seg_exact": "精确相等" in crit and "(flexid,9)" in crit,
    "two_run_sha": "sha256 相等" in crit,
}
print(" ", json.dumps(need, ensure_ascii=False))
print("  README §9 has '其余全部 bit-first':", "其余全部 bit-first" in readme or "其余" in readme)

print("\n== 6. D8 separation ==")
sep = {
    "d8_commit_a1cc22ae": "a1cc22ae" in crit,
    "fix_commit_d35cab68": "d35cab68" in crit,
    "d8_321_px": "321" in crit,
    "single_px_scope": "这 1 px" in crit or "1 px" in crit,
    "not_mixed_declared": ("不混算" in crit) or ("不重新解释" in crit),
}
print(" ", json.dumps(sep, ensure_ascii=False))

print("\n== 7. freeze input list answer (criteria not in guards inputs) ==")
fr = {
    "explicit_no": ("**不含**" in crit) or ("不含" in crit),
    "guard_inputs_14": ("guard_inputs" in crit) and ("14" in crit),
    "pre_window_record_sha": ("sha256" in crit) and ("开窗前" in crit),
}
print(" ", json.dumps(fr, ensure_ascii=False))
print("  receipt flag frozen_input_list_includes_criteria:", ma.get("frozen_input_list_includes_criteria"))

print("\n== 8. non-claims / no second relaxation ==")
nc = {
    "no_upstream_equivalence": "上游等价" in crit or "上游等价" in sig_readme,
    "no_parity_pass": "parity pass" in crit or "parity pass" in sig_readme,
    "no_flex_pass": "flex 渲染通过" in crit,
    "fail_stop": ("失败即停" in crit) or ("立即停手" in crit),
    "no_second_relaxation": ("不二次放宽" in crit) or ("不重试" in crit),
    "attempt5_only": "仅限本次 attempt5" in crit or "仅限 R06-S3b 本次 attempt5" in sig_readme,
}
print(" ", json.dumps(nc, ensure_ascii=False))

print("\n== 9. QA decision-support doc: signed-state / conditional wording ==")
print("  says 'signature-level' (判据放宽/签名级):", ("签名级" in qa_doc) or ("判据放宽" in qa_doc))
print("  minimal scope line present:", ("cam2" in qa_doc and "134" in qa_doc and "beyond4" in qa_doc))
print("  conditional family wording present ('族口径'):", "族口径" in qa_doc)
print("  conditional subset wording present ('或该子集'):", "或该子集" in qa_doc)
print("  -> note: doc was the pre-signature proposal; material/signature card adopt pixel-only;")
print("     per material section 5 any change of the cited doc invalidates attempt5, so this")
print("     check does NOT modify it (recorded as an observation).")

print("\n== 10. verdict inputs ==")
print("material commit ecc93b56, signature commit 3406f1ca, QA support commit a3f690df")
print("blocking items: none expected; observations: decision-support pre-signature conditional wording;")
print("signature original quote relayed by Chief (no independent user-message snapshot).")
