"""R06 多帧采集窗 G-2 —— QA 独立只读复核脚本（零 GPU、不采集、不改仓库内容）。

复核对象：`2026-11-05-render-r06-flex-multiframe-capture-g2`（verified；提交 5f2bd0bb + 证据补充 38a7767c），
HEAD 复核目标 = 38a7767c。逐项独立复算（不采信执行方结论）：

  A. 采集有效性（从落库 npz 独立重算）：frame-000 vs reference/base 15 数组位级全等；
     cam2 两段 depth/seg 变化计数 = 1,167/1,074、1,262/1,126（均 ≥576）、verdict pass-margin；
     位移 k×0.100 m ±1e-6（真实模型 flex_vertbodyid，读采集侧 state.npz.flexvert_xpos）；
     refit actual_sequence=[0,1,2,0,1,2]、refit_calls=6、two_captures_bitwise_identical。
  B. 落库物：fixtures 15 个文件存在且 sha256/bytes 与 evidence `artifacts/files-sha256.json` 15/15 一致；
     与 capture.json.per_frame_files_sha256 一致；`reference/` 与单帧参考相对窗口前提交零 diff。
  C. provenance：capture.json.provenance spec sha/bytes、三脚本 sha 与实际文件（及窗口时点提交 blob）一致；
     refit_policy/refit_bvh。
  D. 窗口合规：基线三点 ≥ floor 11,688 / util<40；monitor 60/60（CSV 60 行、全 clean、free≥floor、
     compute_filtered=0、build=0、util<40、S-2 未启用）；capture exit 0、out_dir_existed_before=false；
     无 `.repeat2`/残留；G-1 负结果证据与卡相对窗口前零 diff；spec/scripts/src/tests 零 diff。
  E. 编码/卫生：evidence manifest 首行 `#`、条目自核；evidence 文件 UTF-8/LF/无 BOM、裸 CR/TAB 0 命中、
     无 U+FFFD；fixtures 文本文件同样检查。
  F. 非声明：多帧验收通过 / 动画一致性已覆盖 / 上游等价 0 命中。

用法（仓库根，pinned venv）：
  target/mjwarp-parallel/venv/Scripts/python.exe -B target/qag2-verify/verify_g2.py [--json <path>]
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
MULTI = REPO / "fixtures/render-bvh/scene-flex/multiframe"
BASE = REPO / "fixtures/render-bvh/scene-flex/reference/base"
SCENE = REPO / "fixtures/render-bvh/scene-flex/scene.xml"
SPEC = REPO / "docs/development-evidence/render-bvh/r06-flex-multiframe-criterion/spec.md"
EVID = REPO / "docs/development-evidence/render-bvh/r06-flex-multiframe-capture-g2"
EVID_G1 = REPO / "docs/development-evidence/render-bvh/r06-flex-multiframe-capture"
G2_CARD = ".normify/changes/2026-11-05-render-r06-flex-multiframe-capture-g2.json"
G1_CARD = ".normify/changes/2026-11-05-render-r06-flex-multiframe-capture.json"
RUNS = REPO / ".normify/runs/2026-11-05-render-r06-flex-multiframe-capture-g2"
WINDOW_BASE = "2b8be485"  # 窗口前最后一个提交（QA r2 复核收口）
CLONE = REPO / "target/qag2-verify/clone"
SPEC_SHA = "1d9fa5cef1ebc829747d63f648d3bab1ec935d8df8e3403bc52c2ac2aa7c40b6"
SCRIPT_SHA = {
    "capture_render_flex.py": "f50e150e4dc29522ce5f49b64bbb50b01bfc5177cee82a3f3c704c9e5c0a16f9",
    "capture_render_flex_multiframe.py": "46d7500227738e8a3fab8f79106bb02599b1b92b29ca4bda6baf807b989e7bd1",
    "capture_render_flex_multiframe_checks.py": "ef77f38b6159a49971a85303961647b0aa88b596fce7dc924998b1e21f8201a9",
}
FLOOR = 11688
RESULTS: list[dict] = []
OBSERVATIONS: list[dict] = []


def obs(name: str, detail=None) -> None:
    """记录一条观察（低/一般，逐项列出，不计入 FAIL）。"""
    OBSERVATIONS.append({"name": name, "detail": detail})
    print(f"[OBS ] {name}: {detail}")


def decode_child(data: bytes) -> str:
    if not data:
        return ""
    try:
        return data.decode("utf-8")
    except UnicodeDecodeError:
        return data.decode("gbk", "replace")


def record(name: str, ok: bool, detail=None) -> None:
    RESULTS.append({"name": name, "ok": bool(ok), "detail": detail})
    print(f"[{'PASS' if ok else 'FAIL'}] {name}: {detail}")


def git(*args: str) -> subprocess.CompletedProcess:
    return subprocess.run(["git", "-C", str(REPO), *args], capture_output=True,
                          encoding="utf-8", errors="replace")


def blob(rel: str, rev: str = "HEAD") -> bytes:
    return subprocess.run(["git", "-C", str(REPO), "show", f"{rev}:{rel}"], capture_output=True).stdout


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_npz(path: pathlib.Path) -> dict:
    import numpy as np

    with np.load(path) as archive:
        return {k: archive[k] for k in archive.files}


def bit_equal(a, b) -> bool:
    import numpy as np

    a = np.asarray(a)
    b = np.asarray(b)
    if a.shape != b.shape:
        return False
    if a.dtype.kind == "f" or b.dtype.kind == "f":
        if a.dtype != b.dtype:
            return False
        return np.array_equal(a.view(np.uint32), b.view(np.uint32))
    return np.array_equal(a, b)


def text_files(root: pathlib.Path):
    for p in sorted(root.rglob("*")):
        if not p.is_file():
            continue
        raw = p.read_bytes()
        if b"\x00" in raw[:8192]:
            continue  # 二进制（npz 等）
        yield p, raw


def main() -> int:
    import numpy as np

    capture = json.loads((MULTI / "capture.json").read_text(encoding="utf-8"))
    cam_poses = json.loads((MULTI / "cam-poses.json").read_text(encoding="utf-8"))
    files_sha = json.loads((EVID / "artifacts/files-sha256.json").read_text(encoding="utf-8"))
    checks_art = json.loads((EVID / "artifacts/capture-checks.json").read_text(encoding="utf-8"))

    # ---------- A. 采集有效性（从落库 npz 独立重算） ----------
    base_state = load_npz(BASE / "state.npz")
    base_arrays: dict = {}
    for kind in ("rgb", "depth", "seg"):
        base_arrays.update(load_npz(BASE / f"{kind}.npz"))
    frames = {}
    for k in range(3):
        d = MULTI / f"frame-{k:03d}"
        frames[k] = {"state": load_npz(d / "state.npz")}
        for kind in ("rgb", "depth", "seg"):
            frames[k][kind] = load_npz(d / f"{kind}.npz")

    mism = []
    checked = 0
    for name in sorted(base_state):
        checked += 1
        if name not in frames[0]["state"] or not bit_equal(frames[0]["state"][name], base_state[name]):
            mism.append(f"state.{name}")
    for kind in ("rgb", "depth", "seg"):
        for name in sorted(base_arrays):
            if not name.startswith(f"{kind}_"):
                continue
            checked += 1
            if name not in frames[0][kind] or not bit_equal(frames[0][kind][name], base_arrays[name]):
                mism.append(f"{kind}.{name}")
    record("A1 frame-000 vs reference/base 15 数组位级全等（落库 npz 独立重算）",
           checked == 15 and not mism, {"arrays_checked": checked, "mismatches": mism[:5]})

    def changed_counts(i: int, j: int) -> tuple[int, int]:
        depth_i = frames[i]["depth"]["depth_cam2"]
        depth_j = frames[j]["depth"]["depth_cam2"]
        seg_i = frames[i]["seg"]["seg_cam2"]
        seg_j = frames[j]["seg"]["seg_cam2"]
        depth_changed = int(np.count_nonzero(depth_i.view(np.uint32) != depth_j.view(np.uint32)))
        seg_changed = int(np.count_nonzero(np.any(seg_i != seg_j, axis=2)))
        return depth_changed, seg_changed

    pairs = {"000->001": changed_counts(0, 1), "001->002": changed_counts(1, 2)}
    expected = {"000->001": (1167, 1074), "001->002": (1262, 1126)}
    record("A2 cam2 两段变化计数 = 证据记录值（1,167/1,074、1,262/1,126）",
           all(pairs[k] == expected[k] for k in pairs), {"measured": pairs})
    record("A3 两段 depth/seg 均 ≥576（margin）", all(d >= 576 and s >= 576 for d, s in pairs.values()),
           {"pairs": pairs})
    declared = capture["delta_selfcheck"]
    record("A4 capture.json.delta_selfcheck.overall == pass-margin（与重算一致）",
           declared["overall"] == "pass-margin" and
           [(p["pair"], p["depth_changed"], p["seg_changed"]) for p in declared["report"]] ==
           [(p, *pairs[p]) for p in ("000->001", "001->002")],
           {"overall": declared["overall"]})

    # 位移：真实模型 flex_vertbodyid + 采集侧 state.npz.flexvert_xpos
    import mujoco

    mjm = mujoco.MjModel.from_xml_path(str(SCENE))
    vertbody = np.asarray(mjm.flex_vertbodyid).reshape(-1)
    attached = [i for i, b in enumerate(vertbody) if b != 0]
    positions = [np.asarray(frames[k]["state"]["flexvert_xpos"], dtype=np.float64).reshape(-1, 3) for k in range(3)]
    disp = {}
    for k in range(3):
        mags = np.linalg.norm(positions[k] - positions[0], axis=1)
        disp[k] = float(mags[attached].mean())
    record("A5 位移断言独立复算：附着顶点均值 = k×0.100 m ±1e-6（采集侧 state）",
           all(abs(disp[k] - k * 0.1) <= 1e-6 for k in range(3)),
           {"attached": len(attached), "world": len(vertbody) - len(attached),
            "means": {k: round(v, 12) for k, v in disp.items()}})
    refit = checks_art["refit"]
    record("A6 refit actual_sequence=[0,1,2,0,1,2]、actual_calls=6、capture.json.refit_calls=6",
           refit["actual_sequence"] == [0, 1, 2, 0, 1, 2] and refit["actual_calls"] == 6
           and capture.get("refit_calls") == 6, {"sequence": refit["actual_sequence"],
                                                 "refit_calls": capture.get("refit_calls")})
    record("A7 两次采集逐位一致 / legacy 参考未动（capture.json 两标记）",
           capture.get("two_captures_bitwise_identical") is True
           and capture.get("legacy_reference_untouched") is True,
           {"bitwise": capture.get("two_captures_bitwise_identical"),
            "legacy": capture.get("legacy_reference_untouched")})

    # ---------- B. 落库物 ----------
    listed = {f["path"]: (f["sha256"], f["bytes"]) for f in files_sha["files"]}
    expected_prefix = "fixtures/render-bvh/scene-flex/multiframe/"
    on_disk = sorted(p.relative_to(REPO).as_posix() for p in MULTI.rglob("*") if p.is_file())
    record("B1 fixtures 文件集 = 15 个（3 帧×4 + capture/cam-poses/README）且无多余文件",
           len(on_disk) == 15 and len(listed) == 15 and set(on_disk) == set(listed),
           {"on_disk": len(on_disk), "listed": len(listed),
            "extra": sorted(set(on_disk) - set(listed))[:5]})
    bad = []
    for rel, (h, n) in listed.items():
        raw = (REPO / rel).read_bytes()
        if sha(raw) != h or len(raw) != n:
            bad.append(rel)
    record("B2 15 个落库文件 sha256/bytes 与 files-sha256.json 逐条一致", not bad, {"bad": bad[:5]})
    perframe = capture.get("per_frame_files_sha256", {})
    pf_bad = []
    for frame, entries in perframe.items():
        for fname, h in entries.items():
            rel = f"fixtures/render-bvh/scene-flex/multiframe/{frame}/{fname}"
            if listed.get(rel, (None, None))[0] != h and (MULTI / frame / fname).is_file():
                pf_bad.append(rel)
    record("B3 capture.json.per_frame_files_sha256 与 files-sha256.json 一致", not pf_bad, {"bad": pf_bad[:5]})
    ref_diff = git("diff", "--name-only", WINDOW_BASE, "--",
                   "fixtures/render-bvh/scene-flex/reference/")
    single_ref_diff = git("diff", "--name-only", WINDOW_BASE, "--",
                          "fixtures/render-bvh/")
    record("B4 reference/（base + 单帧参考）相对窗口前零 diff；窗口只新增 multiframe/",
           ref_diff.stdout.strip() == ""
           and set(single_ref_diff.stdout.split()) == {
               p for p in single_ref_diff.stdout.split() if p.startswith(expected_prefix)},
           {"reference_diff": ref_diff.stdout.strip()[:200],
            "fixtures_diff": single_ref_diff.stdout.split()[:6]})

    # ---------- C. provenance ----------
    prov = capture.get("provenance", {})
    spec_bytes = SPEC.read_bytes()
    record("C1 provenance spec sha/bytes = 1d9fa5ce…/10,506 且与实际 spec 一致",
           prov.get("spec_sha256") == SPEC_SHA and prov.get("spec_bytes") == 10506
           and sha(spec_bytes) == SPEC_SHA and len(spec_bytes) == 10506,
           {"spec_sha": prov.get("spec_sha256", "")[:16], "bytes": prov.get("spec_bytes")})
    actual = {name: sha((REPO / "scripts/authoring" / name).read_bytes()) for name in SCRIPT_SHA}
    prov_scripts = prov.get("authoring_script_sha256", {})
    window_blob = {name: sha(blob(f"scripts/authoring/{name}", "5f2bd0bb")) for name in SCRIPT_SHA}
    record("C2 provenance 三脚本 sha = 期望原值 = 当前文件 = 窗口时点提交 blob",
           prov_scripts == SCRIPT_SHA and actual == SCRIPT_SHA and window_blob == SCRIPT_SHA,
           {"provenance": prov_scripts.get("capture_render_flex.py", "")[:12],
            "actual": actual["capture_render_flex.py"][:12]})
    record("C3 refit_policy=every-frame、refit_bvh=true", prov.get("refit_policy") == "every-frame"
           and prov.get("refit_bvh") is True, {"policy": prov.get("refit_policy")})
    record("C4 证据 artifacts/capture.json 副本与落库 capture.json 同 sha",
           sha((EVID / "artifacts/capture.json").read_bytes()) == sha((MULTI / "capture.json").read_bytes()),
           {"fixtures_sha": sha((MULTI / "capture.json").read_bytes())[:12]})

    # ---------- D. 窗口合规 ----------
    baseline = (EVID / "logs/baseline-three-points.txt").read_text(encoding="utf-8")
    frees = [int(x) for x in re.findall(r"free_points = ([0-9 /]+) MiB", baseline)[0].split("/")]
    floor = int(re.search(r"floor = .* (\d+) MiB", baseline).group(1))
    receipt = json.loads((EVID / "receipt.json").read_text(encoding="utf-8"))
    record("D1 基线三点 free ≥ floor 11,688（取严公式）且 util<40",
           floor == FLOOR and all(f >= FLOOR for f in frees)
           and all(util < 40 for util in receipt["window"]["baseline_three_points"]["util_percent"]),
           {"free": frees, "floor": floor})
    rows = (EVID / "logs/monitor-g2.csv").read_text(encoding="utf-8").splitlines()
    header, body = rows[0].split(","), [r.split(",") for r in rows[1:] if r.strip()]
    col = {name: i for i, name in enumerate(header)}
    ok_rows = [r for r in body if r[col["clean"]] == "True"]
    mins = min(int(r[col["free_mib"]]) for r in body)
    max_util = max(int(r[col["util_percent"]]) for r in body)
    record("D2 monitor 60/60 clean、free_min ≥ floor、util<40、compute_filtered/build 全 0、S-2 未启用",
           len(body) == 60 and len(ok_rows) == 60 and mins >= FLOOR and max_util < 40
           and all(int(r[col["compute_filtered"]]) == 0 and int(r[col["build"]]) == 0 for r in body)
           and receipt["window"]["s2_authoring_exception"] == "已预批但未启用（常规 util<40 全程达标）",
           {"rows": len(body), "clean": len(ok_rows), "free_min": mins, "util_max": max_util})
    hdr = (EVID / "artifacts/capture-log-header.txt").read_text(encoding="utf-8")
    record("D3 capture exit 0、out_dir_existed_before=false（无预存残留）",
           "exit_code: 0" in hdr and "out_dir_existed_before: False" in hdr,
           {"header_lines": len(hdr.splitlines())})
    residue = sorted(p.name for p in MULTI.iterdir() if p.name == ".repeat2")
    dry = json.loads(json.dumps(receipt))["dry_run"]
    record("D4 dry-run exit0/out_dir_created=false/refit_calls=6、无 .repeat2 残留",
           dry["exit_code"] == 0 and dry["out_dir_created"] is False and dry["refit_calls"] == 6
           and not residue, {"residue": residue})
    g1_diff = git("diff", "--name-only", WINDOW_BASE, "--",
                  "docs/development-evidence/render-bvh/r06-flex-multiframe-capture/")
    g1_card_diff = git("diff", "--name-only", WINDOW_BASE, "--", G1_CARD)
    g1_present = EVID_G1.is_dir()
    record("D5 G-1 负结果证据与卡保留不删（相对窗口前零 diff）",
           g1_present and g1_diff.stdout.strip() == "" and g1_card_diff.stdout.strip() == "",
           {"present": g1_present, "diff": g1_diff.stdout.strip()[:120]})
    freeze = git("diff", "--name-only", WINDOW_BASE, "--", "src/", "tests/", "scripts/",
                 "docs/development-evidence/render-bvh/r06-flex-multiframe-criterion/")
    record("D6 窗口提交对 src/tests/scripts/冻结 spec 零 diff",
           freeze.stdout.strip() == "", {"changed": freeze.stdout.split()[:6]})

    # ---------- E. 编码/卫生 ----------
    raw_manifest = (EVID / "manifest.txt").read_bytes()
    entries = [ln for ln in raw_manifest.decode("utf-8").splitlines() if ln and not ln.startswith("#")]
    mbad = []
    for line in entries:
        h, n, rel = line.split()
        raw = (EVID / rel).read_bytes()
        if sha(raw) != h or len(raw) != int(n):
            mbad.append(rel)
    record("E1 evidence manifest 首行 '#'、条目自核（16/16）", raw_manifest.startswith(b"#")
           and len(entries) == 16 and not mbad, {"entries": len(entries), "bad": mbad[:5]})
    prob = []
    for p, raw in text_files(EVID):
        bom = raw.startswith(b"\xef\xbb\xbf")
        try:
            text = raw.decode("utf-8")
        except UnicodeDecodeError:
            prob.append(f"{p.relative_to(EVID)}:not-utf8")
            continue
        if bom or "\r" in text or "\t" in text:
            prob.append(f"{p.relative_to(EVID)}:bom={bom},cr={'\r' in text},tab={'\t' in text}")
        if "\ufffd" in text:
            prob.append(f"{p.relative_to(EVID)}:U+FFFD")
    record("E2 evidence 文本文件 UTF-8/LF/无 BOM、裸 CR/TAB 0 命中、无 U+FFFD", not prob,
           {"problems": prob[:6]})
    fprob = []
    for p, raw in text_files(MULTI):
        text = raw.decode("utf-8")
        if raw.startswith(b"\xef\xbb\xbf") or "\r" in text or "\t" in text:
            fprob.append(p.relative_to(MULTI).as_posix())
    record("E3 fixtures 文本文件（capture/cam-poses/README）UTF-8/LF/无 BOM、裸 CR/TAB 0 命中",
           not fprob, {"problems": fprob})
    # fresh clone 字节安全（12 npz 应一致；文本 3 个受 core.autocrlf 影响 -> 见事实记录）
    if CLONE.is_dir():
        clone_bad = []
        for rel, (h, n) in listed.items():
            raw = (CLONE / rel).read_bytes()
            if sha(raw) != h or len(raw) != n:
                clone_bad.append(rel.split("/")[-1])
        npz_bad = [b for b in clone_bad if b.endswith(".npz")]
        record("E4 fresh clone（core.autocrlf=true）: 12 个 npz 全部字节一致（*.npz -text 规则生效）",
               not npz_bad, {"clone_mismatch": clone_bad})
        if clone_bad:
            obs("O1 fresh clone（core.autocrlf=true）文本夹具非字节稳定：3 个文本文件（capture.json/cam-poses.json/README.md）"
                "缺 eol 固定规则，checkout 后 LF→CRLF，与 files-sha256.json 记录不匹配；12 个 npz 字节安全",
                {"text_mismatch": clone_bad})
        else:
            record("E5 fresh clone 文本夹具字节安全", True, {"text_mismatch": []})
    else:
        record("E4 fresh clone（core.autocrlf=true）: 12 个 npz 全部字节一致（*.npz -text 规则生效）",
               False, {"reason": "clone missing: target/qag2-verify/clone"})
        obs("O1 fresh clone 未执行（clone 目录缺失）", {"reason": "clone missing"})

    # ---------- 证据自称逐项核对 ----------
    hdr_lines = (EVID / "artifacts/capture-log-header.txt").read_text(encoding="utf-8").splitlines()
    capture_log_lines = (EVID / "logs/capture.log").read_text(encoding="utf-8").splitlines()
    hdr_ok = hdr_lines[:10] == capture_log_lines[:10] and hdr_lines[10].startswith("# ---- end (exit_code=0)")
    record("E6 capture-log-header.txt 内容 = capture.log 前 10 行 + 合成 end 行（command/exit_code 原文）",
           hdr_ok, {"hdr": len(hdr_lines), "log": len(capture_log_lines)})
    readme = (EVID / "README.md").read_text(encoding="utf-8")
    manifest_rels = [ln.split()[2] for ln in (EVID / "manifest.txt").read_text(encoding="utf-8").splitlines()
                     if ln and not ln.startswith("#")]
    if "capture.log` 前 11 行" in readme and hdr_lines[:10] == capture_log_lines[:10]:
        obs("O2 README §6 把 capture-log-header.txt 写成「capture.log 前 11 行」，实际为 capture.log 前 10 行 + 合成 "
            "`# ---- end (exit_code=0) ----` 行（内容等价、仅计数口径差 1）",
            {"header_lines": len(hdr_lines), "log_prefix_lines": 10})
    if "并逐条进入 `manifest.txt`" in readme and not any(r.startswith("fixtures/") for r in manifest_rels):
        obs("O3 README §3 写「（fixtures 每文件 sha256）并逐条进入 manifest.txt」，实际 manifest 16 条全部是证据目录内条目；"
            "15 个落库文件 sha256/bytes 只列在 artifacts/files-sha256.json（已复核 15/15 一致）",
            {"manifest_entries": len(manifest_rels), "fixtures_in_manifest": 0})
    record("E6b 落库文件 sha 清单存在且完整（files-sha256.json 15/15，= B2 复核所据）",
           len(files_sha.get("files", [])) == 15, {"entries": len(files_sha.get("files", []))})
    checks_raw = (EVID / "artifacts/capture-checks.json").read_text(encoding="utf-8")
    record("E7 artifacts/capture-checks.json = capture.json.capture_checks 原文",
           json.loads(checks_raw) == capture.get("capture_checks"),
           {"equal": json.loads(checks_raw) == capture.get("capture_checks")})
    script_txt = (EVID / "artifacts/script-sha256.txt").read_text(encoding="utf-8")
    st_ok = all(SCRIPT_SHA[name] in script_txt and str(len((REPO/"scripts/authoring"/name).read_bytes())) in script_txt
                for name in SCRIPT_SHA)
    record("E8 artifacts/script-sha256.txt 三脚本 sha+bytes 与期望值一致（provenance 原值入证据）", st_ok,
           {"file": "artifacts/script-sha256.txt"})
    g2_card = json.loads((REPO / G2_CARD).read_text(encoding="utf-8"))
    g2_addenda = g2_card.get("addenda", [])
    record("E9 G-2 卡 verified + addenda 记证据补充（38a7767c）+ 窗口卡 checks/forbid_claim 保留",
           g2_card.get("status") == "verified" and len(g2_addenda) >= 1
           and any("三脚本 sha" in a.get("reason", "") for a in g2_addenda)
           and any(c.get("kind") == "forbid_claim" for c in g2_card.get("checks", [])),
           {"status": g2_card.get("status"), "addenda": len(g2_addenda)})
    runs_ok = {g: json.loads((RUNS / f"{g}.json").read_text(encoding="utf-8"))["passed"]
               for g in ("fmt", "clippy", "clippy-cuda", "unit")}
    unit = json.loads((RUNS / "unit.json").read_text(encoding="utf-8"))["tests"]
    record("E10 四门禁记录齐全（fmt/clippy/clippy-cuda/unit，unit 1321 过 0 败）",
           all(runs_ok.values()) and unit["passed"] == 1321 and unit["failed"] == 0,
           {"gates": runs_ok, "unit": unit})
    raw_status = git("status", "--porcelain").stdout.strip()
    own = (".normify/changes/2026-11-05-render-r06-flex-multiframe-capture-g2-check.json",
           "docs/development-evidence/render-bvh/r06-flex-multiframe-capture-g2-check/",
           ".normify/runs/2026-11-05-render-r06-flex-multiframe-capture-g2-check/")
    foreign = [ln for ln in raw_status.splitlines()
               if ln.strip() and not any(o in ln for o in own)]
    record("E11 工作树除本复核证据/卡/gates 外 clean（无未预期改动）", not foreign,
           {"dirty": raw_status[:200], "foreign": foreign[:5]})

    # ---------- F. 非声明 ----------
    banned = ["多帧验收通过", "动画一致性已覆盖", "上游等价"]
    evid_text = "\n".join(p.read_text(encoding="utf-8", errors="replace")
                          for p in EVID.rglob("*") if p.is_file() and p.suffix in (".md", ".json", ".txt", ".log", ".csv"))
    hits = [t for t in banned if t in evid_text]
    claim = "采集有效" in evid_text and "可进入验证窗" in evid_text
    record("F1 非声明：三个禁用短语 0 命中，且含「采集有效 → 可进入验证窗」口径",
           not hits and claim, {"hits": hits, "claim_phrase": claim})
    record("F2 收口口径不含「多帧已通过/多帧判据通过」肯定式",
           "多帧判据通过" not in evid_text and "多帧已通过" not in evid_text,
           {"ok": True})

    # ---------- 汇总 ----------
    failed = [it["name"] for it in RESULTS if not it["ok"]]
    payload = {"result": "PASS" if not failed else "FAIL", "total": len(RESULTS),
               "passed": len(RESULTS) - len(failed), "fails": failed, "checks": RESULTS,
               "observations": OBSERVATIONS,
               "head": git("rev-parse", "HEAD").stdout.strip()}
    if "--json" in sys.argv:
        out = pathlib.Path(sys.argv[sys.argv.index("--json") + 1])
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n",
                       encoding="utf-8", newline="\n")
    print(f"\nRESULT: {payload['result']} ({payload['passed']}/{payload['total']} pass, {len(failed)} fail)")
    return 0 if not failed else 1


if __name__ == "__main__":
    raise SystemExit(main())
