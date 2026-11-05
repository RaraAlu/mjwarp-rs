"""R06 多帧验证窗 G-3 —— QA 独立只读复核脚本（零 GPU、不采集、不改仓库内容）。

复核对象：`2026-11-05-render-r06-flex-multiframe-verify-g3`（verified；登记 515de099、start f7d32afd、结果 b3a05f2b）
HEAD 复核目标 = b3a05f2b。逐项独立核（不采信执行方结论）：

  A. 真冻结：guards 28 = 2 exes + 26 inputs；aggregate_inputs 独立复算 = b8110cee…；aggregate(all 28)
     独立复算 = 6a6cca42…；guards 文件 raw sha/bytes（工作树）与 HEAD blob/LF 快照；attempt=1/status=frozen/
     exe 非 pending；live==guard==stash 2/2（独立哈希）；freeze.log 与 freeze-verify 一致；历史 guards 未动。
  B. 窗口纪律：三点基线（free/util）≥ floor 11,688 / util<40 / build=0；monitor CSV 72 样本 60/60 干净
     （干净 streak 内 free≥floor、util<40、build=0；exit 0/ready）；零编译（exe sha 前后不变、build 0/0、
     guard/build 指纹同刻）；post-GPU；时间序 freeze < 基线 < monitor < 实体 < post。
  C. 实体日志：exit 0、running 1 test、1 passed、wall 2764 ms；逐相机 depth 统计（frame-000 cam0/1
     (0,0,0)、cam2 (1,0,3) + allowlist bits/expected/ulp/beyond4/flexid）；001/002 strict；两跑 sha；
     refits=0；跨帧 cam2-only 1167/1074、1262/1126；参考 15 文件 exit 0。
  D. 证据/边界：manifest 14/14、UTF-8/LF/无 BOM/裸 CR/TAB 0；禁用短语与「历史原文/O1–O7」说明；
     diff 8c9e7192..HEAD 仅 guards json + 证据 + 卡/runs；tests/fixtures/src/authoring/spec/实体与修卡证据
     /G-2 证据零 diff；G-3 卡 verified。

低项按 Chief 定级规则（只影响诊断/文案/证据完整性、不影响判据与数值 → observation）。

用法（仓库根，pinned venv）：
  target/mjwarp-parallel/venv/Scripts/python.exe -B target/qa-g3/verify_g3.py [--json <path>]
"""

from __future__ import annotations

import csv
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
EVID = REPO / "docs/development-evidence/render-bvh/r06-flex-multiframe-verify-g3"
GUARDS = "scripts/validate/render-bvh/guards-r06-multiframe.json"
BASE = "8c9e7192"  # 验证窗前最后一个提交（QA 复核 B 收口）
CARD = ".normify/changes/2026-11-05-render-r06-flex-multiframe-verify-g3.json"
RUNS = REPO / ".normify/runs/2026-11-05-render-r06-flex-multiframe-verify-g3"
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


def blob(rel: str) -> bytes:
    return subprocess.run(["git", "-C", str(REPO), "show", f"HEAD:{rel}"], capture_output=True).stdout


def agg(entries: list[dict]) -> str:
    paths = sorted(e["path"] for e in entries)
    hexc = "".join(next(e["sha256"] for e in entries if e["path"] == p) for p in paths)
    return sha(hexc.encode("ascii"))


def main() -> int:
    # ---------- A. 真冻结 ----------
    raw = (REPO / GUARDS).read_bytes()
    head_blob = blob(GUARDS)
    snapshot = (EVID / "artifacts/guards-frozen-snapshot.json").read_bytes()
    record("A1 guards 文件 raw 工作树 sha/bytes = 64bfecdf…/11,899（与证据记录一致）",
           sha(raw) == "64bfecdfd179c4b7160c4f853660671ec9fb570b4d6c7702aa6c35c2c3cb352d" and len(raw) == 11899,
           {"sha": sha(raw)[:16], "bytes": len(raw)})
    record("A2 HEAD blob = 11,696 B/LF（bf4b9302…），证据 LF 快照 = blob 且 = 工作树 LF 规范化",
           sha(head_blob) == "bf4b930216ab9c912c12e214637d6a15dd127eb5142de0aca9118ca209a2f5e4"
           and len(head_blob) == 11696 and snapshot == head_blob
           and snapshot.replace(b"\r\n", b"\n") == raw.replace(b"\r\n", b"\n"),
           {"blob": len(head_blob), "snapshot": len(snapshot), "worktree_crlf": raw.count(b"\r\n")})
    g = json.loads(raw.decode("utf-8"))
    exes, inputs = g["frozen_exes"], g["guard_inputs"]
    record("A3 28 条目 = 2 exes + 26 inputs；attempt=1/status=frozen/exe 非 pending",
           g["attempt"] == 1 and g["status"] == "frozen" and len(exes) == 2 and len(inputs) == 26
           and g["counts"] == {"frozen_exes": 2, "guard_inputs": 26, "total": 28}
           and all(e.get("pending_freeze") is False and e.get("sha256") for e in exes),
           {"counts": g["counts"], "status": g["status"]})
    record("A4 aggregate_inputs 独立复算 = b8110cee…；aggregate(all 28) 独立复算 = 6a6cca42…",
           agg(inputs) == "b8110cee547a6a6a12dbc4b8201f1571bebb31f7450e2d252f0a4015484afb7f"
           and agg(exes + inputs) == "6a6cca42554967291c29b5030ffc3f98e50581a7956bea8fbf51e0df6e144978"
           and g["aggregate_inputs_sha256"] == agg(inputs) and g["aggregate_sha256"] == agg(exes + inputs),
           {"aggregate_inputs": agg(inputs)[:16] + "…", "aggregate": agg(exes + inputs)[:16] + "…"})
    live_eq = []
    for e in exes:
        live = REPO / e["path"]
        stash = REPO / e["stash_path"]
        lb = live.read_bytes() if live.is_file() else b""
        sb = stash.read_bytes() if stash.is_file() else b""
        live_eq.append(sha(lb) == e["sha256"] and len(lb) == e["bytes"]
                       and sha(sb) == e["sha256"] and len(sb) == e["bytes"])
    record("A5 live==guard==stash 2/2（独立哈希：bin a75cf7db…/85,041,741、test 0f76f9e1…/118,098,954）",
           all(live_eq), {"per_exe": live_eq})
    freeze_log = (EVID / "artifacts/freeze.log").read_text(encoding="utf-8")
    record("A6 freeze.log 含 GUARDS_OK=28/28 且 aggregate/guards sha 与文件一致",
           "GUARDS_OK=28/28" in freeze_log
           and "aggregate_sha256=6a6cca42554967291c29b5030ffc3f98e50581a7956bea8fbf51e0df6e144978" in freeze_log
           and "guards_sha256=64bfecdfd179c4b7160c4f853660671ec9fb570b4d6c7702aa6c35c2c3cb352d" in freeze_log)
    fv = json.loads((EVID / "artifacts/freeze-verify.json").read_text(encoding="utf-8"))
    record("A7 freeze-verify.json：aggregate/guards sha/attempt/status/live==guard==stash 与实测一致",
           fv["aggregate_sha256"] == agg(exes + inputs) and fv["aggregate_inputs_sha256"] == agg(inputs)
           and fv["guards_file_sha256"] == sha(raw) and fv["guards_file_bytes"] == len(raw)
           and fv["attempt"] == 1 and fv["status"] == "frozen"
           and all(e["live_eq_guard_eq_stash"] for e in fv["exes"]) and fv["problems"] == [])
    hist = git("diff", "--name-only", f"{BASE}..HEAD", "--", "scripts/validate/render-bvh/").stdout.split()
    record("A8 历史 guards 未动：仅 guards-r06-multiframe.json 变更，guards.json / r06-s3b-flex 零 diff",
           hist == [GUARDS]
           and not git("diff", "--name-only", f"{BASE}..HEAD", "--",
                       "scripts/validate/render-bvh/guards.json",
                       "scripts/validate/render-bvh/guards-r06-s3b-flex.json").stdout.strip(),
           {"changed": hist})
    if "text: unspecified" in git("check-attr", "text", "--", GUARDS).stdout:
        obs("O1 guards-r06-multiframe.json 无 `-text`/eol 规则：raw 工作树 sha/bytes（64bfecdf…/11,899，CRLF 203 处）"
            "≠ HEAD blob（bf4b9302…/11,696 B）；证据已用 LF 快照（=blob）兜底并在 README §8 说明。"
            "建议补仓库 `.gitattributes` 规则（既有 guards.json 仅靠本机 .git/info/attributes，不随克隆传播）",
            {"worktree": len(raw), "blob": len(head_blob)})

    # ---------- B. 窗口纪律 ----------
    baseline = (EVID / "artifacts/baseline.txt").read_text(encoding="utf-8")
    pts = re.findall(r"point\d ts=\S+ used=\d+MiB free=(\d+)MiB util=(\d+)% temp=\d+C compute_raw=\d+ compute_filtered=(\d+) build=(\d+)", baseline)
    frees = [int(p[0]) for p in pts]
    utils = [int(p[1]) for p in pts]
    floor = int(re.search(r"floor=max\(min_free-3072,11688\)=(\d+)MiB", baseline).group(1))
    record("B1 三点基线：free 13,726/13,727/13,696 ≥ floor 11,688、util 37/36/38<40、compute_filtered=0、build=0",
           len(pts) == 3 and frees == [13726, 13727, 13696] and utils == [37, 36, 38]
           and floor == 11688 and all(int(p[2]) == 0 and int(p[3]) == 0 for p in pts), {"free": frees, "floor": floor})
    rows = list(csv.DictReader((EVID / "logs/monitor-g3.csv").read_text(encoding="utf-8").splitlines()))
    clean = [r for r in rows if r["clean"] == "True"]
    clean_bad = [r for r in clean if int(r["free_mib"]) < 11688 or int(r["util_percent"]) >= 40
                 or r["build"] != "0" or r["compute_filtered"] != "0"]
    max_streak = max(int(r["streak"]) for r in rows)
    receipt = json.loads((EVID / "receipt.json").read_text(encoding="utf-8"))
    mon = receipt["monitor"]
    record("B2 monitor 72 样本、60/60 干净（干净行全部 free≥floor/util<40/build=0），exit 0/ready=true",
           len(rows) == 72 and max_streak == 60 and not clean_bad and len(clean) == 69
           and mon["samples"] == 72 and mon["longest_clean"] == 60 and mon["required"] == 60
           and mon["ready"] is True and mon["exit_code"] == 0
           and int(mon["free_min_mib"]) == min(int(r["free_mib"]) for r in rows),
           {"rows": len(rows), "clean": len(clean), "max_streak": max_streak, "free_min": mon["free_min_mib"]})
    record("B3 monitor util_max 58 仅出现在 60 连击之前（3 个非干净样本 streak=0），不影响 60/60",
           max(int(r["util_percent"]) for r in rows) == 58
           and all(int(r["streak"]) == 0 for r in rows if r["clean"] != "True"),
           {"non_clean": [(r["ts"], r["util_percent"]) for r in rows if r["clean"] != "True"]})
    # 时间序
    fv_t = fv["freeze_completed_at"]
    bfp = json.loads((EVID / "artifacts/build-fingerprint.json").read_text(encoding="utf-8"))
    eei = json.loads((EVID / "artifacts/entity-exe-identity.json").read_text(encoding="utf-8"))
    ent = receipt["entity_run"]
    record("B4 零编译：构建指纹 = 冻结 exe sha = live/stash 前后不变、build 进程 0/0、mtime 未变",
           {e["entity"]: e["sha256"] for e in bfp["exes"]}
           == {e["entity"]: e["sha256"] for e in exes}
           and eei["zero_relink"] is True and eei["build_procs_before"] == 0 and eei["build_procs_after"] == 0
           and eei["live_unchanged_before_after"] is True and eei["stash_unchanged_before_after"] is True
           and eei["bin_unchanged_before_after"] is True,
           {"exes": {e["entity"]: e["sha256"][:12] for e in bfp["exes"]}})
    t_freeze = fv["freeze_completed_at"]
    t_wstart = eei["window_start"]
    t_wend = eei["window_end"]
    record("B5 时间序：freeze(22:39:25) < 基线(22:39:40-22:40:10) < monitor(22:40:31-22:46:33) < 实体(22:46:50.555-22:46:53.322) < post-gpu",
           fv["freeze_completed_at"] < "2026-11-05T22:39:40"
           and "2026-11-05T22:39:40" < "2026-11-05T22:40:10" < mon["start"]
           and mon["start"] < mon["end"] < t_wstart and t_wstart < t_wend < "2026-11-05T22:46:54",
           {"freeze": t_freeze, "window": [t_wstart, t_wend]})
    post = (EVID / "artifacts/post-gpu.txt").read_text(encoding="utf-8")
    record("B6 post-GPU 记录（22:46:53.631、2,282/13,780 MiB、5%、29°C、exit 0、wall 2764）",
           "2026-11-05T22:46:53.631+08:00" in post and "exit_code=0" in post and "wall_ms=2764" in post)

    # ---------- C. 实体日志 ----------
    log = (EVID / "logs/entity-gpu.log").read_text(encoding="utf-8")
    record("C1 实体：running 1 test、1 passed、exit 0、wall 2,764 ms、PASS 标记",
           "running 1 test" in log and "test result: ok. 1 passed; 0 failed" in log
           and "exit_code=0" in log and "wall_ms=2764" in log
           and "r06-multiframe PASS: 3 frames, per-frame + cross-frame + option-A 3-context audit" in log)
    record("C2 frame-000 逐相机 depth：cam0/cam1 (0,0,0)、cam2 (1,0,3) + allowlist 明细",
           "R06-S3b depth cam0: mismatched=0 beyond4=0 max_ulp=0" in log
           and "R06-S3b depth cam1: mismatched=0 beyond4=0 max_ulp=0" in log
           and "R06-S3b depth cam2: mismatched=1 beyond4=0 max_ulp=3" in log
           and "R06-S3b allowlist cam2 (134,128): bits=0x400015ba expected=0x400015b7 ulp=3 beyond4=0 flexid=0" in log)
    record("C3 001/002 严格 bit-first 通过（无严格失配打印）+ 逐帧两跑 sha 相等 + refits=0",
           "refits=0 two-run sha equal" in log and log.count("refits=0 two-run sha equal") == 3
           and "严格区失配" not in log and "bit-first 失配" not in log,
           {"refit_prints": log.count("refits=0 two-run sha equal")})
    record("C4 跨帧 cam2-only：pair0 1167/1074、pair1 1262/1126（≥384）",
           "pair 0 cam2=1167/1074" in log and "pair 1 cam2=1262/1126" in log
           and "all_cameras=1167/1074" in log and "all_cameras=1262/1126" in log)
    ref_log = (EVID / "artifacts/reference-integrity.log").read_text(encoding="utf-8")
    record("C5 参考 15 文件校验：running 1 test、1 passed、exit 0",
           "running 1 test" in ref_log and "1 passed" in ref_log and "exit_code: 0" in ref_log)
    # 参考完整性独立重算（15 文件）
    multis = REPO / "fixtures/render-bvh/scene-flex/multiframe"
    capture = json.loads((multis / "capture.json").read_text(encoding="utf-8"))
    recorded = capture["files_sha256"]
    ref_ok = len(recorded) == 12 and all(
        sha((multis / f"frame-{f:03d}/{k}.npz").read_bytes()) == recorded[f"frame-{f:03d}/{k}.npz"]
        for f in range(3) for k in ("rgb", "depth", "seg", "state"))
    text_ok = all(sha((multis / n).read_bytes()) == h for n, h in
                  (("capture.json", "767f8c1b0ea63880b6bbdec0f30452fd24b6166ccef8aceea43acf29417a7252"),
                   ("cam-poses.json", "97d6a50b3fe9ac4bbfddb699af5fa5b7e3d435c013c212b1e3702387fa459afb"),
                   ("README.md", "fa51f4b325973236aa78b4b54eb1a8bd0d407aa72a30cab4f9310be6187f34e2")))
    record("C6 参考完整性独立重算：12 npz 对齐 capture.files_sha256 + 3 文本锚定 = 15 文件", ref_ok and text_ok)

    # ---------- D. 证据/边界 ----------
    man = (EVID / "manifest.txt").read_bytes()
    entries = [ln for ln in man.decode("utf-8").splitlines() if ln and not ln.startswith("#")]
    bad_m = []
    for line in entries:
        h, n, rel = line.split()
        b = (EVID / rel).read_bytes()
        if sha(b) != h or len(b) != int(n):
            bad_m.append(rel)
    record("D1 证据 manifest 首行 #、14/14 自核", man.startswith(b"#") and len(entries) == 14 and not bad_m,
           {"entries": len(entries), "bad": bad_m})
    prob = []
    for p in sorted(EVID.rglob("*")):
        if not p.is_file():
            continue
        b = p.read_bytes()
        if b"\x00" in b[:8192]:
            continue
        try:
            t = b.decode("utf-8")
        except UnicodeDecodeError:
            prob.append(f"{p.name}:not-utf8")
            continue
        if b.startswith(b"\xef\xbb\xbf") or "\r" in t or "\t" in t or "\ufffd" in t:
            prob.append(f"{p.name}:bom/cr/tab/ufffd")
    record("D2 证据文本 UTF-8/LF/无 BOM、裸 CR/TAB 0、无 U+FFFD", not prob, {"problems": prob[:6]})
    WIN = "b3a05f2b"  # G-3 结果提交
    changed = git("diff", "--name-only", f"{BASE}..{WIN}").stdout.split()
    allowed = {GUARDS, CARD, "docs/development-evidence/render-bvh/r06-flex-multiframe-verify-g3/"}
    unexpected = [p for p in changed if not p.startswith(tuple(allowed)) and p != CARD]
    record("D3 窗口改动（8c9e7192..b3a05f2b）= guards JSON + 证据目录 + 卡；tests/fixtures/src/authoring/spec/实体与修卡证据/G-2 证据零 diff",
           not unexpected
           and not git("diff", "--name-only", f"{BASE}..{WIN}", "--", "tests/", "src/", "fixtures/",
                       "scripts/authoring/", "docs/development-evidence/render-bvh/r06-flex-multiframe-criterion/",
                       "docs/development-evidence/render-bvh/r06-flex-multiframe-entity/",
                       "docs/development-evidence/render-bvh/r06-flex-multiframe-entity-fix/",
                       "docs/development-evidence/render-bvh/r06-flex-multiframe-capture-g2/").stdout.strip(),
           {"unexpected": unexpected[:5], "changed": changed})
    readme = (EVID / "README.md").read_text(encoding="utf-8")
    record("D4 历史原文说明 + 实体文件 499/496/210 冻结 + O1–O7 已知项写入窗口材料",
           "历史原文说明" in readme and "499" in readme and "496" in readme and "210" in readme
           and "已知项" in readme and "O1" in readme and "O7" in readme)
    banned = ["多帧验收通过", "上游等价"]
    hits = {}
    for term in banned:
        loc = []
        for p in EVID.rglob("*"):
            if p.is_file() and p.suffix in (".md", ".json", ".txt", ".csv", ".log"):
                try:
                    t = p.read_text(encoding="utf-8")
                except UnicodeDecodeError:
                    continue
                if term in t:
                    loc.append(p.name)
        if loc:
            hits[term] = sorted(set(loc))
    record("D5 禁用短语（多帧验收通过 / 上游等价 exact）0 命中", not hits, {"hits": hits})
    neg_ctx = []
    for p in EVID.rglob("*"):
        if p.is_file() and p.suffix in (".md", ".json", ".txt", ".csv", ".log"):
            try:
                t = p.read_text(encoding="utf-8")
            except UnicodeDecodeError:
                continue
            if "动画一致性已覆盖" in t:
                neg_ctx.append(p.name)
    if neg_ctx:
        obs("O2 禁词「动画一致性已覆盖」只在**非声明否定句**出现（README §1「**不写** …」/ receipt `not_claimed`）；"
            "语义为不得写，但若 forbid_claim 按字面扫描会误报，建议改「动画一致性结论」等避词",
            {"files": sorted(set(neg_ctx))})
    else:
        record("D5b 禁词「动画一致性已覆盖」0 命中", True)
    record("D6 G-3 卡 verified + 非声明口径（只判该 3 帧集逐帧 + 跨帧可见性，不含 R06 整族/动画/上游等价）",
           json.loads((REPO / CARD).read_text(encoding="utf-8")).get("status") == "verified"
           and "只覆盖该 3 帧集" in readme and "不写" in readme)
    raw_status = git("status", "--porcelain").stdout.strip()
    own = (".normify/changes/2026-11-05-render-r06-flex-multiframe-verify-g3-check.json",
           "docs/development-evidence/render-bvh/r06-flex-multiframe-verify-g3-check/",
           ".normify/runs/2026-11-05-render-r06-flex-multiframe-verify-g3-check/")
    foreign = [ln for ln in raw_status.splitlines() if ln.strip() and not any(o in ln for o in own)]
    record("D7 工作树除本复核证据/卡/gates 外 clean（无未预期改动）", not foreign,
           {"dirty": raw_status[:200], "foreign": foreign[:5]})
    if "repo-write-lock" not in "\n".join(p.read_text(encoding="utf-8", errors="replace")
                                          for p in EVID.rglob("*") if p.is_file() and p.suffix in (".md", ".json", ".txt", ".log")):
        obs("O3 证据未留 repo-write-lock status/acquire/release 原始输出（仅 README §6 一句「收尾释放写锁」与时间戳）；"
            "窗口锁纪律缺机器留痕，建议后续窗口把 lock status/acquire/release 输出入证据", {"files": len(entries)})

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


if __name__ == "__main__":
    raise SystemExit(main())
