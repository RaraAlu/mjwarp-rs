"""G3 sign 语义修正片独立复核。

只读/可复现地核对：
1. 上游语义：pinned `include/warp-headers/builtin.h` 的 sign 定义 + 上游 `collision_sdf.py`
   三处用法（radial_field / grad_cylinder / compute_grad）；冻结源与 pinned 上游树逐字节一致；
2. 产品实现：`primitives.rs::upstream_sign` 表达式与 5 个调用点（radial_field 3、grad_cylinder 1、
   `descent.rs::compute_grad` 1），产品 `src/` 内无残留活跃 `signum()`；
3. 改前代码：从提交 `23661557` 取 `primitives.rs`（4 处 `signum()`）与 `descent.rs`（手写三分支
   0/NaN→0）；
4. **独立最小复现**：rustc 编译 `qa_sign_repro.rs`，对 `-0.0`/`0.0`/`NaN`/±有限/±inf 逐输入
   比较「改前 vs 改后 vs 上游期望」，并演示 radial_field 与 compute_grad 两条产品路径的差异；
5. 测试覆盖缺口：`descent.rs` 是否有 `compute_grad` 的直接反向判别用例（观察项）；
6. 范围：修正片只动 2 个产品文件 + 证据 + normify 记录；G3 已 verified 证据数字未改。

用法：python qa_sign_check.py
"""

import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
EVIDENCE = Path(__file__).resolve().parent
BUILD = REPO / "target" / "qa-sdf-sign"

PRE_FIX = "23661557"
FIX = "11696fc8"
ADDENDA = "8f25e883"
PINNED = "71da24d956378a87a703b6e1442b13aec0c4ac29"
PINNED_TREE = REPO / "target" / "mjwarp-parallel" / "upstream"
FROZEN = REPO / "docs" / "evidence" / "d04-complex-physics" / "authoring-runs" / "run-5" / "frozen-source"


def load(p):
    return json.loads(Path(p).read_text(encoding="utf-8"))


def git(*args, check=False):
    p = subprocess.run(["git", "-c", "core.longpaths=true", *args], cwd=REPO, capture_output=True)
    if check and p.returncode != 0:
        raise RuntimeError(p.stderr.decode("utf-8", "replace"))
    return p


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def strip_comment(line):
    idx = line.find("//")
    return line[:idx] if idx >= 0 else line


def main():
    report = {"schema": 1, "kind": "qa-g3-sign-independent-check",
              "task_id": "2026-10-25-g10-p5-sdf-primitives-sign-independent-check"}

    # ---------- 1) 上游语义 ----------
    builtin = (REPO / "include" / "warp-headers" / "builtin.h").read_text(encoding="utf-8")
    sign_lines = [(i + 1, ln) for i, ln in enumerate(builtin.splitlines())
                  if "sign(T x)" in ln and "adj_sign" not in ln]
    upstream_file = PINNED_TREE / "mujoco_warp" / "_src" / "collision_sdf.py"
    up_lines = upstream_file.read_text(encoding="utf-8").splitlines()
    snippets = {
        "radial_field_L152_154": up_lines[151:154],
        "grad_cylinder_L275": up_lines[274],
        "compute_grad_L629_630": up_lines[628:630],
    }
    frozen_same = (FROZEN / "_src" / "collision_sdf.py").exists() and \
        sha256(FROZEN / "_src" / "collision_sdf.py") == sha256(upstream_file)
    pinned_head = subprocess.run(["git", "-C", str(PINNED_TREE), "rev-parse", "HEAD"],
                                 capture_output=True).stdout.decode().strip()
    report["upstream"] = {
        "builtin_sign_definition": [
            {"line": ln, "text": txt.strip()} for ln, txt in sign_lines],
        "builtin_semantics_is_lt_zero": all("x < T(0) ? -1 : 1" in txt for _, txt in sign_lines),
        "pinned_tree_head": pinned_head,
        "pinned_tree_matches": pinned_head == PINNED,
        "frozen_source_equals_pinned_tree": frozen_same,
        "frozen_sha256": sha256(FROZEN / "_src" / "collision_sdf.py") if (FROZEN / "_src" / "collision_sdf.py").exists() else None,
        "usage_snippets": snippets,
        "usage_checks": {
            "radial_field_three_signs": all("wp.sign(x[" in t for t in snippets["radial_field_L152_154"]),
            "grad_cylinder_sign_p2": "wp.sign(p[2])" in snippets["grad_cylinder_L275"],
            "compute_grad_sign_max_val": "wp.sign(max_val)" in snippets["compute_grad_L629_630"][0]
                                         and "max_grad * sign" in snippets["compute_grad_L629_630"][1],
        },
    }

    # ---------- 2) 产品实现 ----------
    prim = (REPO / "src" / "physics" / "collision" / "sdf" / "primitives.rs").read_text(encoding="utf-8")
    descent = (REPO / "src" / "physics" / "collision" / "sdf" / "descent.rs").read_text(encoding="utf-8")
    m = re.search(r"pub\(crate\) fn upstream_sign\(x: f32\) -> f32 \{\n(.*?)\n\}", prim, re.S)
    body = m.group(1).strip() if m else None
    signum_active = []
    for f in sorted((REPO / "src").rglob("*.rs")):
        lines = f.read_text(encoding="utf-8", errors="replace").splitlines()
        # 测试模块（#[cfg(test)] 起的尾部）不算产品代码；测试里刻意用 signum 做反向判别
        test_start = None
        for i, ln in enumerate(lines):
            if ln.strip().startswith("#[cfg(test)]"):
                test_start = i
        for i, ln in enumerate(lines):
            if test_start is not None and i >= test_start:
                continue
            code = strip_comment(ln)
            if "signum(" in code and "upstream_sign" not in code:
                signum_active.append({"file": str(f.relative_to(REPO)), "line": i + 1,
                                      "text": ln.strip()})
    report["product"] = {
        "upstream_sign_body": body,
        "body_is_lt_zero": body is not None and "x < 0.0" in body and "-1.0" in body and "1.0" in body,
        "radial_field_calls": len(re.findall(r"upstream_sign\(x\[", prim)),
        "grad_cylinder_call": "upstream_sign(p[2])" in prim,
        "descent_uses_upstream_sign": "primitives::upstream_sign(max_val)" in descent,
        "descent_old_three_branch_absent": "} else {\n        0.0\n    };" not in descent,
        "active_signum_in_src": signum_active,
    }

    # ---------- 3) 改前代码 ----------
    old_prim = git("show", f"{PRE_FIX}:src/physics/collision/sdf/primitives.rs").stdout.decode("utf-8")
    old_des = git("show", f"{PRE_FIX}:src/physics/collision/sdf/descent.rs").stdout.decode("utf-8")
    old_signum_lines = [ln.strip() for ln in old_prim.splitlines()
                        if "signum(" in strip_comment(ln)]
    old_block = None
    mm = re.search(r"let sign = if max_val > 0\.0 \{(.*?)\};", old_des, re.S)
    if mm:
        old_block = "let sign = if max_val > 0.0 {" + mm.group(1) + "};"
    report["pre_fix"] = {
        "primitives_signum_count": len(old_signum_lines),
        "primitives_signum_lines": old_signum_lines,
        "descent_old_sign_block": old_block,
        "descent_old_branch_zero_for_nonpositive": old_block is not None and "0.0" in old_block,
    }

    # ---------- 4) 独立最小复现 ----------
    BUILD.mkdir(parents=True, exist_ok=True)
    repro_src = EVIDENCE / "qa_sign_repro.rs"
    repro_exe = BUILD / "qa_sign_repro.exe"
    c = subprocess.run(["rustc", "-O", str(repro_src), "-o", str(repro_exe)],
                       capture_output=True)
    repro_out = ""
    repro_rc = None
    if c.returncode == 0:
        r = subprocess.run([str(repro_exe)], capture_output=True)
        repro_out = r.stdout.decode("utf-8", "replace")
        repro_rc = r.returncode
    sign_rows, cg_rows, rf_rows, grad_rows = [], [], [], []
    for ln in repro_out.splitlines():
        parts = dict(kv.split("=", 1) for kv in ln.split(" ")[1:])
        parts["kind"] = ln.split(" ")[0]
        if parts["kind"] == "SIGN":
            sign_rows.append({"name": parts["name"],
                              "old_matches_upstream": parts["old_matches_upstream"] == "true",
                              "new_matches_upstream": parts["new_matches_upstream"] == "true"})
        elif parts["kind"] == "CG":
            cg_rows.append({"name": parts["name"],
                            "old_matches_upstream": parts["old_matches_upstream"] == "true",
                            "new_matches_upstream": parts["new_matches_upstream"] == "true"})
        elif parts["kind"] == "RF":
            rf_rows.append({"name": parts["name"], "old": parts["old"], "new": parts["new"]})
        elif parts["kind"] == "GRAD":
            grad_rows.append({"name": parts["name"], "old_sign": parts["old_sign"],
                              "new_sign": parts["new_sign"],
                              "old_grad": parts["old_grad"], "new_grad": parts["new_grad"]})
    old_fail_sign = [r["name"] for r in sign_rows if not r["old_matches_upstream"]]
    old_fail_cg = [r["name"] for r in cg_rows if not r["old_matches_upstream"]]
    new_fail_all = [r["name"] for r in sign_rows + cg_rows if not r["new_matches_upstream"]]
    expected_old_fail_sign = {r["name"] for r in sign_rows if r["name"] in ("-0.0", "NaN")}
    expected_old_fail_cg = {r["name"] for r in cg_rows if r["name"] in ("0.0", "-0.0", "NaN")}
    report["repro"] = {
        "rustc_returncode": c.returncode,
        "rustc_stderr": c.stderr.decode("utf-8", "replace")[-800:],
        "run_returncode": repro_rc,
        "sign_rows": sign_rows,
        "compute_grad_sign_rows": cg_rows,
        "radial_field_rows": rf_rows,
        "compute_grad_rows": grad_rows,
        "old_signum_fails_on": old_fail_sign,
        "old_compute_grad_branch_fails_on": old_fail_cg,
        "new_impl_fails_on": new_fail_all,
        "old_fails_exactly_expected": set(old_fail_sign) == expected_old_fail_sign
                                      and set(old_fail_cg) == expected_old_fail_cg,
        "new_matches_all": new_fail_all == [],
        "radial_field_differs": all(r["old"] != r["new"] for r in rf_rows),
        "compute_grad_zero_branch_differs": all(
            r["old_grad"] != r["new_grad"] for r in grad_rows if r["name"] in ("0.0", "-0.0", "NaN")),
        "raw_output_lines": len(repro_out.splitlines()),
    }

    # ---------- 5) 覆盖缺口（观察） ----------
    prim_tests = len(re.findall(r"#\[test\]", prim.split("mod tests")[-1])) if "mod tests" in prim else 0
    descent_tests = len(re.findall(r"#\[test\]", descent))
    compute_grad_refs = []
    for f in list((REPO / "tests").glob("*.rs")) + list((REPO / "src").rglob("*.rs")):
        try:
            text = f.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        if "compute_grad(" in text:
            compute_grad_refs.append(str(f.relative_to(REPO)))
    report["coverage"] = {
        "primitives_new_tests": prim_tests,
        "descent_tests": descent_tests,
        "descent_has_compute_grad_reverse_test": descent_tests > 0,
        "compute_grad_referenced_in": compute_grad_refs,
        "gap": "descent.rs::compute_grad 的 0/NaN→0 旧分支没有直接反向判别用例" if descent_tests == 0 else None,
    }

    # ---------- 6) 范围 / 已 verified 数字未改 ----------
    fix_files = [ln.split("\t")[-1] for ln in
                 git("show", "--name-status", "--format=", FIX).stdout.decode("utf-8", "replace").splitlines()
                 if ln.strip()]
    add_files = [ln.split("\t")[-1] for ln in
                 git("show", "--name-status", "--format=", ADDENDA).stdout.decode("utf-8", "replace").splitlines()
                 if ln.strip()]
    g3_evidence_diff = [ln for ln in git("diff", "--name-only", f"{PRE_FIX}..HEAD", "--",
                                         "docs/development-evidence/g10-p5-sdf-host",
                                         "docs/development-evidence/g10-p5-sdf-host-r2",
                                         "docs/development-evidence/g10-p5-sdf-relmat-fix")
                        .stdout.decode().splitlines() if ln.strip()]
    forbidden_scope = [f for f in fix_files
                       if f.startswith(("Cargo.toml", "Cargo.lock", "rust-toolchain.toml",
                                        "fixtures/mjwarp-complex-physics/"))]
    report["scope"] = {
        "fix_commit": FIX,
        "fix_files": fix_files,
        "addenda_commit": ADDENDA,
        "addenda_files": add_files,
        "addenda_only_normify_records": all(f.startswith(".normify/") for f in add_files),
        "g3_verified_evidence_diff_since_pre_fix": g3_evidence_diff,
        "forbidden_scope_changes": forbidden_scope,
    }

    # ---------- 判定 ----------
    up = report["upstream"]
    prod = report["product"]
    rep = report["repro"]
    report["verdict"] = {
        "upstream_semantics_verified": up["builtin_semantics_is_lt_zero"]
                                      and up["pinned_tree_matches"]
                                      and up["frozen_source_equals_pinned_tree"]
                                      and all(up["usage_checks"].values()),
        "product_impl_matches": prod["body_is_lt_zero"]
                                and prod["radial_field_calls"] == 3
                                and prod["grad_cylinder_call"]
                                and prod["descent_uses_upstream_sign"]
                                and prod["active_signum_in_src"] == [],
        "pre_fix_reverse_discriminated": rep["old_fails_exactly_expected"] and rep["new_matches_all"],
        "pre_fix_radial_field_and_compute_grad_differ": rep["radial_field_differs"]
                                                        and rep["compute_grad_zero_branch_differs"],
        "scope_clean": report["scope"]["forbidden_scope_changes"] == []
                        and report["scope"]["g3_verified_evidence_diff_since_pre_fix"] == []
                        and report["scope"]["addenda_only_normify_records"],
        "coverage_gap_observation": report["coverage"]["gap"],
        "pass": False,
    }
    v = report["verdict"]
    v["pass"] = all([v["upstream_semantics_verified"], v["product_impl_matches"],
                     v["pre_fix_reverse_discriminated"],
                     v["pre_fix_radial_field_and_compute_grad_differ"],
                     v["scope_clean"]])

    (EVIDENCE / "qa-sign-check.json").write_text(
        json.dumps(report, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")

    # 控制台摘要
    print("== upstream ==")
    print("  builtin sign lines:", report["upstream"]["builtin_sign_definition"])
    print("  pinned tree HEAD:", up["pinned_tree_head"], "matches:", up["pinned_tree_matches"])
    print("  frozen source == pinned tree:", up["frozen_source_equals_pinned_tree"])
    print("  usage checks:", json.dumps(up["usage_checks"]))
    print("== product ==")
    print("  upstream_sign body:", prod["upstream_sign_body"])
    print("  radial_field calls:", prod["radial_field_calls"],
          "grad_cylinder:", prod["grad_cylinder_call"],
          "descent uses:", prod["descent_uses_upstream_sign"],
          "active signum:", prod["active_signum_in_src"])
    print("== pre-fix ==")
    print("  old signum count:", report["pre_fix"]["primitives_signum_count"])
    print("  old descent block:", (report["pre_fix"]["descent_old_sign_block"] or "").replace("\n", " "))
    print("== repro ==")
    print("  rustc rc:", rep["rustc_returncode"], "run rc:", rep["run_returncode"])
    print("  old signum fails on:", rep["old_signum_fails_on"])
    print("  old compute_grad branch fails on:", rep["old_compute_grad_branch_fails_on"])
    print("  new impl fails on:", rep["new_impl_fails_on"])
    print("  radial_field differs:", rep["radial_field_differs"],
          "compute_grad zero branch differs:", rep["compute_grad_zero_branch_differs"])
    for r in rep["radial_field_rows"]:
        print("   RF", r["name"], "old", r["old"], "new", r["new"])
    for r in rep["compute_grad_rows"]:
        print("   GRAD", r["name"], "old_sign", r["old_sign"], "new_sign", r["new_sign"],
              "old_grad", r["old_grad"], "new_grad", r["new_grad"])
    print("== coverage ==")
    print(" ", json.dumps(report["coverage"], ensure_ascii=False))
    print("== scope ==")
    print("  fix files:", report["scope"]["fix_files"])
    print("  addenda only normify:", report["scope"]["addenda_only_normify_records"])
    print("  g3 evidence diff:", report["scope"]["g3_verified_evidence_diff_since_pre_fix"])
    print("== verdict ==")
    print(" ", json.dumps(report["verdict"], ensure_ascii=False))
    print("report:", EVIDENCE / "qa-sign-check.json")
    return 0 if v["pass"] else 1


if __name__ == "__main__":
    sys.exit(main())
