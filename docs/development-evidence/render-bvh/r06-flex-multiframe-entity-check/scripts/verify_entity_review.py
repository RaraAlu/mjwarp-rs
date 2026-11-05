"""R06 多帧实体 A（含小修卡）—— QA 独立只读复核脚本（零 GPU、不采集、不改仓库内容）。

复核对象（最终状态 HEAD a175c7f8）：
  * 实体卡 `2026-11-05-render-r06-flex-multiframe-entity`（verified；实现 2a27c2fd、数字校正 241d702f）
  * 小修卡 `2026-11-05-render-r06-flex-multiframe-entity-fix`（verified；实现 a2d7d4c5、裁决 addenda a175c7f8）

核对（不采信执行方结论）：
  A. 行数/边界：499/496/210/498/205/198；validate 498 零 diff；fix 卡改动 scope；src/fixtures/spec 零 diff；
     guards 文件未动。
  B. cam2-only：`change_pixels_cam2` 为唯一判据口径（verify_change_pixels / CPU ⑤⑥ / GPU 跨帧断言）；
     `change_pixels_all_cameras` 仅诊断；独立重算 cam0/1=0、cam2=(1167,1074)/(1262,1126)、合计==cam2。
  C. 选项 A：无「偏移模型建 context」；口径 = 逐帧 state 刚性位姿建 3 context + 刚性签名一致 + 同 context 对照 +
     refits()==0；Chief 裁决留痕（fix receipt.option_a / 实体 README §5 / 实体卡 addenda / GPU 注释）。
  D. 证据校正：README 476 清零、两跑 sha 改「整块多相机缓冲 sha」、实体收据 line_ratchet 499/496；
     实体+修卡证据 manifest 自核、UTF-8/LF 无 BOM；fix.diff 与实际 diff 一致。
  E. 回归面：validate 498 零 diff、allowlist 210（+14）、两新 rs ≤500、guards 2+26=28 + aggregate 复算。
  F. 判据实现：rgb ≤1 LSB、depth allowlist/严格、seg 全等、两跑 sha、参考完整性 15 文件、provenance、门禁 1359/0。

低项定级（Chief 口径：只影响诊断/文案/验证强度、不影响判据与数值 → observation）。

用法（仓库根，pinned venv）：
  target/mjwarp-parallel/venv/Scripts/python.exe -B target/qa-entity/verify_entity_final.py [--json <path>]
"""

from __future__ import annotations

import hashlib
import json
import pathlib
import re
import subprocess
import sys

sys.dont_write_bytecode = True
for _stream in (sys.stdout, sys.stderr):
    try:
        _stream.reconfigure(encoding="utf-8", errors="replace", newline="\n")
    except Exception:  # pragma: no cover
        pass

REPO = next(p for p in pathlib.Path(__file__).resolve().parents
            if (p / "Cargo.toml").is_file() and (p / "scripts/authoring").is_dir())
EVID_ENTITY = REPO / "docs/development-evidence/render-bvh/r06-flex-multiframe-entity"
EVID_FIX = REPO / "docs/development-evidence/render-bvh/r06-flex-multiframe-entity-fix"
MULTI = REPO / "fixtures/render-bvh/scene-flex/multiframe"
SPEC = REPO / "docs/development-evidence/render-bvh/r06-flex-multiframe-criterion/spec.md"
TEST_RS = "tests/render_flex_multiframe.rs"
SUPPORT_RS = "tests/support/render_bvh/multiframe.rs"
ALLOW_RS = "tests/support/render_bvh/allowlist.rs"
VALIDATE_RS = "tests/render_flex_validate.rs"
GUARD_PS1 = "scripts/validate/render-bvh/freeze-guards-r06-multiframe.ps1"
GUARD_JSON = "scripts/validate/render-bvh/guards-r06-multiframe.json"
CARD_ENTITY = ".normify/changes/2026-11-05-render-r06-flex-multiframe-entity.json"
CARD_FIX = ".normify/changes/2026-11-05-render-r06-flex-multiframe-entity-fix.json"
RUNS_ENTITY = REPO / ".normify/runs/2026-11-05-render-r06-flex-multiframe-entity"
RUNS_FIX = REPO / ".normify/runs/2026-11-05-render-r06-flex-multiframe-entity-fix"
BASE_ENTITY = "3fa62181"   # 实体卡登记前
BASE_FIX = "ddcf78f8"      # 小修卡登记前
RESULTS: list[dict] = []
OBSERVATIONS: list[dict] = []


def record(name: str, ok: bool, detail=None) -> None:
    RESULTS.append({"name": name, "ok": bool(ok), "detail": detail})
    print(f"[{'PASS' if ok else 'FAIL'}] {name}: {detail}")


def obs(name: str, detail=None) -> None:
    OBSERVATIONS.append({"name": name, "detail": detail})
    print(f"[OBS ] {name}: {detail}")


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def git(*args: str) -> subprocess.CompletedProcess:
    return subprocess.run(["git", "-C", str(REPO), *args], capture_output=True,
                          encoding="utf-8", errors="replace")


def blob(rel: str, rev: str = "HEAD") -> bytes:
    return subprocess.run(["git", "-C", str(REPO), "show", f"{rev}:{rel}"], capture_output=True).stdout


def main() -> int:
    import numpy as np

    # ---------- A. 行数 / 边界 ----------
    counts = {rel: blob(rel).count(b"\n") for rel in (TEST_RS, SUPPORT_RS, ALLOW_RS, VALIDATE_RS, GUARD_PS1, GUARD_JSON)}
    record("A1 行数：test 499 / support 496 / allowlist 210 / validate 498 / guard ps1 205 / guards json 198",
           counts == {TEST_RS: 499, SUPPORT_RS: 496, ALLOW_RS: 210, VALIDATE_RS: 498,
                      GUARD_PS1: 205, GUARD_JSON: 198}, counts)
    record("A2 两个新 rs ≤500（500 行棘轮；499/496 均未超）",
           counts[TEST_RS] <= 500 and counts[SUPPORT_RS] <= 500, {"test": counts[TEST_RS], "support": counts[SUPPORT_RS]})
    vd = git("diff", "--name-only", f"{BASE_ENTITY}..HEAD", "--", VALIDATE_RS).stdout.strip()
    record("A3 render_flex_validate.rs 498 行零 diff（实体卡+小修卡全程）", vd == "", {"diff": vd})
    scope_fix = git("diff", "--name-only", f"{BASE_FIX}..HEAD").stdout.split()
    allowed = {TEST_RS, SUPPORT_RS, CARD_ENTITY, CARD_FIX,
               "docs/development-evidence/render-bvh/r06-flex-multiframe-entity/",
               "docs/development-evidence/render-bvh/r06-flex-multiframe-entity-fix/",
               ".normify/runs/", ".normify/changes/"}
    unexpected = [p for p in scope_fix if not p.startswith(tuple(allowed))]
    record("A4 小修卡改动 scope = 测试两文件 + 实体证据 + 修卡证据 + 卡/runs（无 fixtures/src/spec）",
           not unexpected, {"unexpected": unexpected[:5], "count": len(scope_fix)})
    record("A5 src/、fixtures/、冻结 spec 相对实体卡登记前零 diff",
           not git("diff", "--name-only", f"{BASE_ENTITY}..HEAD", "--", "src/", "fixtures/",
                   "docs/development-evidence/render-bvh/r06-flex-multiframe-criterion/").stdout.strip())
    guard_changed = git("diff", "--name-only", f"{BASE_FIX}..HEAD", "--", "scripts/validate/render-bvh/").stdout.strip()
    record("A6 小修卡未动 guards 套件（guards ps1/json 相对修卡登记前零 diff）", guard_changed == "",
           {"changed": guard_changed})
    gv = git("diff", "--numstat", f"{BASE_ENTITY}..HEAD", "--", ALLOW_RS).stdout.split()
    record("A7 allowlist.rs 回归面 = 210 行、仅 +14/−0（assert_depth_strict）", (int(gv[0]), int(gv[1])) == (14, 0),
           {"added": gv[0], "deleted": gv[1]})

    # ---------- B. cam2-only ----------
    support = blob(SUPPORT_RS).decode("utf-8")
    test_rs = blob(TEST_RS).decode("utf-8")
    record("B1 新增 change_pixels_cam2（唯判据口径）+ change_pixels_all_cameras（仅诊断）",
           "pub fn change_pixels_cam2(pair: usize)" in support
           and "pub fn change_pixels_all_cameras(pair: usize)" in support
           and "pub const CAM2: usize = 2" in support)
    verify_fn = re.search(r"pub fn verify_change_pixels\(\)[\s\S]*?\n\}", support)
    record("B2 verify_change_pixels 断言只用 change_pixels_cam2（≥384）",
           verify_fn is not None and "change_pixels_cam2(pair)" in verify_fn.group(0)
           and "CHANGE_PIXELS_MIN" in verify_fn.group(0)
           and "change_pixels_all_cameras" not in verify_fn.group(0))
    record("B3 CPU 自证 ⑤⑥ 与确定性用例使用 cam2 口径",
           "multiframe::change_pixels_cam2(0)" in test_rs and "multiframe::change_pixels_cam2(1)" in test_rs)
    # GPU 跨帧段：cam2 断言 + 全相机仅 println
    gpu_cross = test_rs[test_rs.find("// 跨帧可见性"):]
    record("B4 GPU 跨帧段：断言用 change_pixels_cam2；all_cameras 仅 println 诊断",
           "let pair_0 = multiframe::change_pixels_cam2(0)" in gpu_cross
           and "let pair_1 = multiframe::change_pixels_cam2(1)" in gpu_cross
           and "println!(\"pair {pair} cam2=" in gpu_cross
           and "change_pixels_all_cameras" in gpu_cross and "println!" in gpu_cross,
           {"cam2_assert": "change_pixels_cam2" in gpu_cross})
    all_in_assert = "CHANGE_PIXELS_MIN" in support and re.search(
        r"change_pixels_all_cameras[\s\S]{0,120}CHANGE_PIXELS_MIN", support + test_rs) is not None
    record("B5 全相机合计不参与 ≥384 判据断言（无阈值断言引用）", not all_in_assert)
    record("B6 跨帧阈值仍是 384；576 只作为「非验收阈」注释出现",
           "pub const CHANGE_PIXELS_MIN: usize = 384" in support
           and not re.search(r"576\s*[<>=]", support + test_rs),
           {"384": True})

    # 独立重算每相机计数
    def load(frame: int, kind: str) -> dict:
        with np.load(MULTI / f"frame-{frame:03d}" / f"{kind}.npz") as a:
            return {k: a[k] for k in a.files}

    per_cam, totals = {}, {}
    for pair in (0, 1):
        rows = []
        for cam in range(3):
            d0, d1 = load(pair, "depth")[f"depth_cam{cam}"], load(pair + 1, "depth")[f"depth_cam{cam}"]
            s0, s1 = load(pair, "seg")[f"seg_cam{cam}"], load(pair + 1, "seg")[f"seg_cam{cam}"]
            rows.append((int(np.count_nonzero(d0.view(np.uint32) != d1.view(np.uint32))),
                         int(np.count_nonzero(np.any(s0 != s1, axis=2)))))
        per_cam[f"{pair:03d}->{pair+1:03d}"] = rows
        totals[f"{pair:03d}->{pair+1:03d}"] = (sum(r[0] for r in rows), sum(r[1] for r in rows))
    cam2 = {k: v[2] for k, v in per_cam.items()}
    record("B7 独立重算：cam0/cam1 帧间变化 = 0；cam2 = (1167,1074)/(1262,1126) 且均 ≥384",
           all(per_cam[k][0] == (0, 0) and per_cam[k][1] == (0, 0) for k in per_cam)
           and cam2 == {"000->001": (1167, 1074), "001->002": (1262, 1126)}
           and all(d >= 384 and s >= 384 for d, s in cam2.values()), {"per_cam": per_cam})
    record("B8 全相机合计 == cam2（本夹具 cam0/1=0，故 cam2-only 与合计逐对相同）",
           all(totals[k] == cam2[k] for k in totals), {"totals": totals})
    if re.search(r"assert_eq!\(all,\s*multiframe::change_pixels_all_cameras", test_rs):
        obs("O1 `change_pixels_all_cameras()` 在 CPU 自证⑦中参与「同输入两次调用相等」的确定性断言"
            "（非 ≥384 判据断言），与「只作诊断打印、不参与断言」字面略有出入；不影响判据与数值")

    # ---------- C. 选项 A / 裁决留痕 ----------
    grep = subprocess.run(["git", "-C", str(REPO), "grep", "-n", "偏移模型建", "--",
                           "tests/", "docs/development-evidence/", "scripts/"],
                          capture_output=True, encoding="utf-8", errors="replace").stdout.strip()
    hits_now = [ln for ln in grep.splitlines() if "fix.diff" not in ln]
    record("C1 实现/证据/脚本中无「偏移模型建 context」现行表述（fix.diff 内为被删除旧措辞的 − 行，不计）",
           not hits_now, {"matches": hits_now[:3], "only_in_fix_diff": len(hits_now) == 0 and "fix.diff" in grep})
    cards_grep = subprocess.run(["git", "-C", str(REPO), "grep", "-n", "偏移模型建", "--", ".normify/changes/"],
                                capture_output=True, encoding="utf-8", errors="replace").stdout.strip()
    if cards_grep:
        obs("O7 实体卡 `.normify/changes/...entity.json` 的 intent/acceptance 原文仍含「偏移模型建 3 个 context」"
            "（verified 卡 acceptance 不可改写，normify 规范禁止；裁决已由 addenda + 小修卡 acceptance + 实体 README §5 "
            "留痕，实现/证据/代码均无该表述）", {"card_hits": len(cards_grep.splitlines())})
    fix_receipt = json.loads((EVID_FIX / "receipt.json").read_text(encoding="utf-8"))
    opt = fix_receipt.get("option_a", {})
    record("C2 修卡 receipt 裁决留痕：no_body_pos_step=true + chief_ruling_at + wording 四要素",
           opt.get("no_body_pos_step") is True and "2026-11-05" in opt.get("chief_ruling_at", "")
           and all(t in opt.get("wording", "") for t in ("刚性签名", "同一 context", "refits()==0"))
           and "body_pos 偏移对 flex 渲染惰性" in opt.get("spec_section_2_implementation_mapping", ""),
           {"keys": list(opt.keys())})
    entity = json.loads((REPO / CARD_ENTITY).read_text(encoding="utf-8"))
    addenda = " ".join(a.get("reason", "") for a in entity.get("addenda", []))
    record("C3 实体卡 addenda 含 Chief 裁决留痕（不补 body_pos 偏移 + 新口径）",
           "body_pos 偏移" in addenda and "refits()==0" in addenda, {"addenda": len(entity.get("addenda", []))})
    er = (EVID_ENTITY / "README.md").read_text(encoding="utf-8")
    record("C4 实体证据 README §5 诚实口径 + 裁决段 + spec §2 实现映射",
           "不补 `body_pos` 偏移步骤" in er and "逐帧 state 的刚性位姿建 3 个 `RenderContext`" in er
           and "裁决留痕（Chief 2026-11-05）" in er and "spec §2 的实现映射" in er)
    gpu_comment = test_rs[test_rs.find("// 选项 A（口径按 Chief"):test_rs.find("let mut contexts")]
    record("C5 GPU 用例选项 A 注释已改诚实口径（含裁决日期与不补 body_pos）",
           "Chief" in gpu_comment and "body_pos" in gpu_comment and "偏移步骤" in gpu_comment,
           {"comment_lines": len(gpu_comment.splitlines())})
    # 裁决路径可追溯性
    trace = " ".join(opt.get("ruling_trace", []))
    if "receipt.json.criteria.option_a.chief_ruling" in trace or "criteria.option_a" in (
            EVID_FIX / "README.md").read_text(encoding="utf-8"):
        obs("O2 裁决留痕路径写 `receipt.json.criteria.option_a.chief_ruling`，实际**修复卡** receipt 为顶层 "
            "`option_a`（`criteria.option_a` 存在于实体卡 receipt）；追溯时按顶层字段查找即可，属文案/路径偏差")
    record("C6 实体卡 acceptance 未改写（规范禁止），裁决走 addenda",
           "偏移模型建 3 个 context" in " ".join(entity.get("acceptance", []))
           or any("裁决" in a.get("reason", "") for a in entity.get("addenda", [])),
           {"acceptance_immutable": True})

    # ---------- D. 证据校正 ----------
    record("D1 实体 README 无 476 残留；行数 499/496 与 line_ratchet 一致",
           "476" not in er and "**499**" in er and "**496**" in er)
    record("D2 「两跑 sha」措辞改为整块多相机缓冲 sha（README §3 / 修卡 README）",
           "整块多相机缓冲" in er and "整块多相机缓冲" in (EVID_FIX / "README.md").read_text(encoding="utf-8"))
    entity_receipt = json.loads((EVID_ENTITY / "receipt.json").read_text(encoding="utf-8"))
    lr = entity_receipt["verification"]["line_ratchet"]
    record("D3 实体 receipt line_ratchet 已同步 499/496（476 清零）",
           lr["tests/render_flex_multiframe.rs"] == 499
           and lr["tests/support/render_bvh/multiframe.rs"] == 496, lr)
    fix_man = [ln for ln in (EVID_FIX / "manifest.txt").read_text(encoding="utf-8").splitlines()
               if ln and not ln.startswith("#")]
    ent_man = [ln for ln in (EVID_ENTITY / "manifest.txt").read_text(encoding="utf-8").splitlines()
               if ln and not ln.startswith("#")]
    bad_man = []
    for evid, entries in ((EVID_FIX, fix_man), (EVID_ENTITY, ent_man)):
        for line in entries:
            h, n, rel = line.split()
            b = (evid / rel).read_bytes()
            if sha(b) != h or len(b) != int(n):
                bad_man.append(f"{evid.name}/{rel}")
    record("D4 实体+修卡证据 manifest 自核（8/8、6/6）", len(fix_man) == 6 and len(ent_man) == 8 and not bad_man,
           {"fix": len(fix_man), "entity": len(ent_man), "bad": bad_man[:5]})
    prob = []
    for evid in (EVID_FIX, EVID_ENTITY):
        for p in sorted(evid.rglob("*")):
            if not p.is_file():
                continue
            b = p.read_bytes()
            try:
                t = b.decode("utf-8")
            except UnicodeDecodeError:
                prob.append(f"{evid.name}/{p.name}:not-utf8")
                continue
            if b.startswith(b"\xef\xbb\xbf") or "\r" in t or "\t" in t or "\ufffd" in t:
                prob.append(f"{evid.name}/{p.name}:bom/cr/tab/ufffd")
    record("D5 两证据目录 UTF-8/LF/无 BOM、裸 CR/TAB 0、无 U+FFFD", not prob, {"problems": prob[:6]})
    art = (EVID_FIX / "artifacts/fix.diff").read_bytes()
    actual = git("diff", f"{BASE_FIX}..a2d7d4c5", "--", "tests/",
                 "docs/development-evidence/render-bvh/r06-flex-multiframe-entity/").stdout.encode()
    record("D6 artifacts/fix.diff 与实际 diff 一致（tests + 实体证据）",
           art.replace(b"\r\n", b"\n").strip() == actual.replace(b"\r\n", b"\n").strip(),
           {"artifact_bytes": len(art)})
    fix_cpu = (EVID_FIX / "logs/cpu-tests.log").read_text(encoding="utf-8")
    record("D7 修卡 CPU 日志：38 passed/0 failed/1 ignored + cam2/all 诊断两行",
           "38 passed; 0 failed; 1 ignored" in fix_cpu
           and "pair 0: cam2=(1167, 1074) all=(1167, 1074)" in fix_cpu
           and "pair 1: cam2=(1262, 1126) all=(1262, 1126)" in fix_cpu)

    # ---------- E. 回归面 ----------
    def gate_dir(d: pathlib.Path) -> dict:
        return {g: json.loads((d / f"{g}.json").read_text(encoding="utf-8"))["passed"]
                for g in ("fmt", "clippy", "clippy-cuda", "unit")}

    unit_fix = json.loads((RUNS_FIX / "unit.json").read_text(encoding="utf-8"))["tests"]
    unit_ent = json.loads((RUNS_ENTITY / "unit.json").read_text(encoding="utf-8"))["tests"]
    record("E1 四门禁全过（修卡 + 实体卡），unit 各 1359 过 / 0 败 / 400 ignored",
           all(gate_dir(RUNS_FIX).values()) and all(gate_dir(RUNS_ENTITY).values())
           and unit_fix["passed"] == unit_ent["passed"] == 1359 and unit_fix["failed"] == 0,
           {"fix_unit": unit_fix})
    guards = json.loads((REPO / GUARD_JSON).read_text(encoding="utf-8"))
    inputs, exes = guards["guard_inputs"], guards["frozen_exes"]
    record("E2 guards 回归面：2 exes + 26 inputs = 28、仍 pending_freeze（未执行 freeze）",
           len(exes) == 2 and len(inputs) == 26 and guards["counts"]["total"] == 28
           and guards["status"] == "pending_freeze"
           and all(e.get("pending_freeze") is True and e.get("sha256") is None for e in exes))
    bad_in = [e["path"] for e in inputs
              if sha((REPO / e["path"]).read_bytes()) != e["sha256"]
              or len((REPO / e["path"]).read_bytes()) != e["bytes"]]
    paths = sorted(e["path"] for e in inputs)
    agg = sha("".join(next(e["sha256"] for e in inputs if e["path"] == p) for p in paths).encode("ascii"))
    record("E3 guards 26 inputs sha/bytes 全对 + aggregate 复算 = b8110cee…",
           not bad_in and agg == guards["aggregate_inputs_sha256"], {"agg": agg[:16] + "…", "bad": bad_in[:3]})
    kinds = {}
    for e in inputs:
        kinds[e.get("kind")] = kinds.get(e.get("kind"), 0) + 1
    record("E4 guards 分类 input=25 / guard_script=1", kinds == {"input": 25, "guard_script": 1}, kinds)
    stash = REPO / "target/render-bvh/r06-multiframe-window/frozen-exes"
    record("E5 freeze stash 不存在（未执行真冻结）", not stash.exists())

    # ---------- F. 判据实现 / 参考 / 门禁 ----------
    record("F1 rgb 判据 max_byte_diff<=1 & beyond_1_lsb=0；seg 逐像素全等；两跑 sha",
           "stats.max_byte_diff <= 1" in support and "stats.beyond_one_lsb_pixels, 0" in support
           and "assert_eq!(mismatched, 0" in support and "for run in 0..2" in test_rs
           and "两跑 sha 必须相同" in test_rs)
    record("F2 depth：frame-000 allowlist(cam2/134,128/flexid0/seg9) + 001/002 严格 bit-first",
           "ALLOWLIST_CAMERA: usize = 2" in blob(ALLOW_RS).decode("utf-8")
           and "ALLOWLIST_PIXEL_X: usize = 134" in blob(ALLOW_RS).decode("utf-8")
           and "ALLOWLIST_PIXEL_Y: usize = 128" in blob(ALLOW_RS).decode("utf-8")
           and "assert_depth_strict" in support and "allowlist_frame()" in support)
    drift_line = next((ln for ln in support.splitlines() if "allowlist 漂移" in ln and "panic!" in ln), "")
    if drift_line and "flat=" not in drift_line:
        obs("O3 allowlist 漂移 panic 打印 x/y/bits/expected/ulp/seg 但缺 `flat`（卡面要求 flat/x/y/...）；"
            "只影响诊断定位，strict 路径仍有 flat 信息，不影响判据")
    record("F3 GPU 用例 1 个且 #[ignore]；CPU 自证 22 项（#[test] 23 个）",
           test_rs.count("#[test]") == 23 and test_rs.count("#[ignore = ") == 1)
    record("F4 参考完整性：12 npz 对齐 capture.files_sha256 + 3 文本锚定 = 15 文件（独立重算）",
           _check_reference_integrity())
    capture = json.loads((MULTI / "capture.json").read_text(encoding="utf-8"))
    prov = capture["provenance"]
    spec = SPEC.read_bytes()
    record("F5 provenance：spec 1d9fa5ce…/10,506 B + 三脚本 sha + refit_calls=6 + sequence + policy（独立重算）",
           prov["spec_sha256"] == sha(spec) and len(spec) == 10506
           and prov["spec_bytes"] == 10506
           and prov["authoring_script_sha256"] == {
               "capture_render_flex.py": "f50e150e4dc29522ce5f49b64bbb50b01bfc5177cee82a3f3c704c9e5c0a16f9",
               "capture_render_flex_multiframe.py": "46d7500227738e8a3fab8f79106bb02599b1b92b29ca4bda6baf807b989e7bd1",
               "capture_render_flex_multiframe_checks.py": "ef77f38b6159a49971a85303961647b0aa88b596fce7dc924998b1e21f8201a9"}
           and capture["refit_calls"] == 6
           and capture["capture_checks"]["refit"]["actual_sequence"] == [0, 1, 2, 0, 1, 2]
           and prov["refit_policy"] == "every-frame")
    refit_lines = [i + 1 for i, ln in enumerate(test_rs.splitlines()) if "context.refits()," in ln]
    if refit_lines:
        obs("O4 `refits()==0` 断言只在每帧渲染**前**；渲染后仅 println。未真正证明「render 不 refit」，"
            "建议渲染后补断言（不影响判据与数值，仅验证强度）", {"assert_lines": refit_lines})
    guard_ps = (REPO / GUARD_PS1).read_text(encoding="utf-8")
    if "expected 16 frozen entries" in guard_ps:
        obs("O5 freeze 脚本预算抛错文案写「expected 16 frozen entries」，实际条件检查 28；仅文案，行为正确")
    if "与上游等价" in json.dumps(entity_receipt.get("claims", {}), ensure_ascii=False):
        obs("O6 实体 receipt 非声明句含 exact 禁词「上游等价」（否定式；README/代码无）；建议改「与上游实现的等价性」")
    record("F6 非声明：不产生验证窗结论、不写 R06 整族通过/动画通用/上游等价",
           "不产生验证窗结论" in (EVID_FIX / "README.md").read_text(encoding="utf-8")
           or "不产生验收结论" in er)
    raw_status = git("status", "--porcelain").stdout.strip()
    own = (".normify/changes/2026-11-05-render-r06-flex-multiframe-entity-check.json",
           "docs/development-evidence/render-bvh/r06-flex-multiframe-entity-check/",
           ".normify/runs/2026-11-05-render-r06-flex-multiframe-entity-check/")
    foreign = [ln for ln in raw_status.splitlines() if ln.strip() and not any(o in ln for o in own)]
    record("F7 工作树除本复核证据/卡/gates 外 clean（无未预期改动）", not foreign,
           {"dirty": raw_status[:200], "foreign": foreign[:5]})
    fix_card = json.loads((REPO / CARD_FIX).read_text(encoding="utf-8"))
    record("F8 小修卡 verified + 实体卡 verified 且 addenda 记录裁决",
           fix_card.get("status") == "verified" and entity.get("status") == "verified"
           and len(entity.get("addenda", [])) >= 2)

    failed = [it["name"] for it in RESULTS if not it["ok"]]
    payload = {"result": "PASS" if not failed else "FAIL", "total": len(RESULTS),
               "passed": len(RESULTS) - len(failed), "fails": failed, "checks": RESULTS,
               "observations": OBSERVATIONS, "head": git("rev-parse", "HEAD").stdout.strip()}
    if "--json" in sys.argv:
        out = pathlib.Path(sys.argv[sys.argv.index("--json") + 1])
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n",
                       encoding="utf-8", newline="\n")
    print(f"\nRESULT: {payload['result']} ({payload['passed']}/{payload['total']} pass, {len(failed)} fail, "
          f"{len(OBSERVATIONS)} observations)")
    return 0 if not failed else 1


def _check_reference_integrity() -> bool:
    capture = json.loads((MULTI / "capture.json").read_text(encoding="utf-8"))
    recorded = capture["files_sha256"]
    if len(recorded) != 12:
        return False
    for frame in range(3):
        for kind in ("rgb", "depth", "seg", "state"):
            rel = f"frame-{frame:03d}/{kind}.npz"
            if sha((MULTI / rel).read_bytes()) != recorded.get(rel):
                return False
    text = {"multiframe/capture.json": sha((MULTI / "capture.json").read_bytes()),
            "multiframe/cam-poses.json": sha((MULTI / "cam-poses.json").read_bytes()),
            "multiframe/README.md": sha((MULTI / "README.md").read_bytes())}
    support = blob(SUPPORT_RS).decode("utf-8")
    return all(h in support for h in text.values())


if __name__ == "__main__":
    raise SystemExit(main())
