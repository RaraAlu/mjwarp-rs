"""R06 多帧专题收口治理 —— QA 一致性只读复核脚本（CPU-only、零 GPU、不改仓库内容）。

复核对象：`2026-11-05-render-r06-multiframe-closeout`（verified；登记 8f376ae0、start 73f33bd6、实现 6af1d83d），
HEAD 复核目标 = 6af1d83d，基线 = 收口治理前 `d8fc8eeb`。

核对：
  A. 两卡仅 note+updated_at 变更；status 保持 backlog/proposed；title/acceptance/depends_on 等其余字段逐字未动。
  B. 两卡 note 含 G-2/G-3 覆盖 + 覆盖边界 + spec sha + 证据指针（5f2bd0bb/38a7767c/b3a05f2b/d8fc8eeb）；
     roadmap note 含「保持 proposed、不宣称整族通过」。
  C. O1 backlog 卡 status=backlog/p15 且含 guards eol raw-vs-blob sha 口径；窗口/收口范围未改 .gitattributes。
  D. 收口证据 manifest 2/2 自核、首行 #、UTF-8/LF/无 BOM、裸 CR/TAB 0；receipt task_id/known_observations O1-O3；
     README 覆盖边界/非声明/证据索引；证据指针提交存在；spec sha 1d9fa5ce…/10,506 B 未变。
  E. 边界：d8fc8eeb..6af1d83d 仅 7 文件（两卡 + closeout 卡 + backlog 卡 + 3 证据文件）；
     unchanged src/tests/fixtures/scripts 与既有证据目录；无越界声明（精确禁词只在 closeout 卡自身词表/否定句）。

用法（仓库根，pinned venv）：
  target/mjwarp-parallel/venv/Scripts/python.exe -B target/qa-closeout/verify_closeout.py [--json <path>]
"""

from __future__ import annotations

import hashlib
import json
import pathlib
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
EVID = REPO / "docs/development-evidence/render-bvh/r06-multiframe-closeout"
BASE = "d8fc8eeb"
TARGET = "6af1d83d"
TRACK = ".normify/changes/2026-11-04-render-r06-flex-multiframe.json"
ROADMAP = ".normify/changes/2026-10-11-roadmap-r06-p8.json"
CLOSEOUT = ".normify/changes/2026-11-05-render-r06-multiframe-closeout.json"
BLOG = ".normify/changes/2026-11-05-render-r06-multiframe-guards-eol-backlog.json"
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


def card(rel: str, rev: str = "HEAD") -> dict:
    return json.loads(blob(rel, rev).decode("utf-8"))


def main() -> int:
    # ---------- A/B. 两卡 diff 与 note 内容 ----------
    diff = git("diff", "-U0", f"{BASE}..{TARGET}", "--", TRACK, ROADMAP).stdout
    removed = [ln[1:] for ln in diff.splitlines() if ln.startswith("-") and not ln.startswith("---")]
    added = [ln[1:] for ln in diff.splitlines() if ln.startswith("+") and not ln.startswith("+++")]
    record("A1 两卡 diff 仅 note 两条 + updated_at 两条（4 removed / 4 added 行）",
           len(removed) == 4 and len(added) == 4
           and sum(1 for ln in removed if ln.strip().startswith('"note"')) == 2
           and sum(1 for ln in removed if ln.strip().startswith('"updated_at"')) == 2,
           {"removed": len(removed), "added": len(added)})
    for rel, want_status in ((TRACK, "backlog"), (ROADMAP, "proposed")):
        old, new = card(rel, BASE), card(rel, "HEAD")
        keep = {k: v for k, v in new.items() if k not in ("note", "updated_at")}
        old_keep = {k: v for k, v in old.items() if k not in ("note", "updated_at")}
        record(f"A2 {rel.split('/')[-1]}：status 保持 {want_status} 且除 note/updated_at 外其余字段逐字未动",
               new["status"] == want_status and keep == old_keep
               and new["note"] != old["note"] and new["updated_at"] != old["updated_at"],
               {"status": new["status"]})
    track = card(TRACK)["note"]
    roadmap = card(ROADMAP)["note"]
    t_tokens = ["G-2 采集有效", "G-3 验证 PASS", "单场景 scene-flex", "3 帧（0/+0.10/+0.20 m）",
                "帧内位级 + 跨帧可见性", "动画通用性/多场景/长序列未覆盖", "1d9fa5ce",
                "5f2bd0bb", "38a7767c", "b3a05f2b", "d8fc8eeb"]
    t_missing = [t for t in t_tokens if t not in track]
    record("B1 tracking note 含覆盖边界 + spec sha + 全部证据指针", not t_missing, {"missing": t_missing})
    r_tokens = ["多帧子项已由 G-2/G-3 覆盖（3 帧集、单场景）", "保持 proposed", "不宣称整族通过",
                "5f2bd0bb", "38a7767c", "b3a05f2b", "d8fc8eeb"]
    r_missing = [t for t in r_tokens if t not in roadmap]
    record("B2 roadmap note 含覆盖口径 + 保持 proposed + 不宣称整族 + 证据指针", not r_missing, {"missing": r_missing})
    record("B3 证据指针 4 个提交均存在",
           all(git("rev-parse", f"{c}^{{commit}}").returncode == 0 for c in ("5f2bd0bb", "38a7767c", "b3a05f2b", "d8fc8eeb")))
    spec = (REPO / "docs/development-evidence/render-bvh/r06-flex-multiframe-criterion/spec.md").read_bytes()
    record("B4 冻结 spec 未变（1d9fa5ce…/10,506 B）",
           sha(spec) == "1d9fa5cef1ebc829747d63f648d3bab1ec935d8df8e3403bc52c2ac2aa7c40b6" and len(spec) == 10506)

    # ---------- C. O1 backlog ----------
    b = card(BLOG)
    bnote = b.get("note", "")
    record("C1 O1 backlog 卡 status=backlog / p15，note 记 guards eol raw-vs-blob sha 且「今晚不改 .gitattributes」",
           b.get("status") == "backlog" and b.get("schedule", {}).get("priority") == 15
           and "64bfecdf" in bnote and "bf4b9302" in bnote and "11,899" in bnote and "11,696" in bnote
           and ".gitattributes" in bnote, {"status": b.get("status")})
    record("C2 收口范围未改 .gitattributes（d8fc8eeb..6af1d83d 零 diff）",
           git("diff", "--name-only", f"{BASE}..{TARGET}", "--", ".gitattributes").stdout.strip() == "")

    # ---------- D. 收口证据 ----------
    m = (EVID / "manifest.txt").read_bytes()
    lines = [ln for ln in m.decode("utf-8").splitlines() if ln and not ln.startswith("#")]
    bad_m = []
    for line in lines:
        h, n, rel = line.split()
        bb = (EVID / rel).read_bytes()
        if sha(bb) != h or len(bb) != int(n):
            bad_m.append(rel)
    record("D1 收口证据 manifest 首行 #、2/2 自核", m.startswith(b"#") and len(lines) == 2 and not bad_m,
           {"entries": len(lines), "bad": bad_m})
    prob = []
    for p in sorted(EVID.rglob("*")):
        if not p.is_file():
            continue
        bb = p.read_bytes()
        try:
            t = bb.decode("utf-8")
        except UnicodeDecodeError:
            prob.append(f"{p.name}:not-utf8")
            continue
        if bb.startswith(b"\xef\xbb\xbf") or "\r" in t or "\t" in t or "\ufffd" in t:
            prob.append(f"{p.name}:bom/cr/tab/ufffd")
    record("D2 收口证据 UTF-8/LF/无 BOM、裸 CR/TAB 0、无 U+FFFD", not prob, {"problems": prob})
    rec = json.loads((EVID / "receipt.json").read_text(encoding="utf-8"))
    known = rec.get("known_observations", {})
    record("D3 receipt task_id 正确 + known_observations 含 O1/O2/O3",
           rec.get("task_id") == "2026-11-05-render-r06-multiframe-closeout"
           and all(k in known for k in ("O1", "O2", "O3")), {"known": list(known)})
    readme = (EVID / "README.md").read_text(encoding="utf-8")
    record("D4 README 含覆盖边界表、证据索引、O1–O3 已知项与非声明",
           "## 2. 覆盖边界（如实）" in readme and "## 3. 证据索引" in readme
           and "已知项" in readme and "## 5. 非声明" in readme)
    record("D5 收口卡 verified 且 checks 含 unchanged/json/forbid_claim",
           card(CLOSEOUT).get("status") == "verified"
           and {c["kind"] for c in card(CLOSEOUT).get("checks", [])} >= {"unchanged", "json", "forbid_claim"})

    # ---------- E. 边界与无越界声明 ----------
    scope = git("diff", "--name-only", f"{BASE}..{TARGET}").stdout.split()
    expected = {TRACK, ROADMAP, CLOSEOUT, BLOG,
                "docs/development-evidence/render-bvh/r06-multiframe-closeout/README.md",
                "docs/development-evidence/render-bvh/r06-multiframe-closeout/manifest.txt",
                "docs/development-evidence/render-bvh/r06-multiframe-closeout/receipt.json"}
    unchanged = git("diff", "--name-only", f"{BASE}..{TARGET}", "--", "src/", "tests/", "fixtures/",
                    "scripts/", "docs/development-evidence/render-bvh/r06-flex-multiframe-criterion/",
                    "docs/development-evidence/render-bvh/r06-flex-multiframe-capture-g2/",
                    "docs/development-evidence/render-bvh/r06-flex-multiframe-verify-g3/",
                    "docs/development-evidence/render-bvh/r06-flex-multiframe-entity/",
                    "docs/development-evidence/render-bvh/r06-flex-multiframe-entity-fix/").stdout.strip()
    record("E1 变更范围恰 7 文件；unchanged src/tests/fixtures/scripts 与既有证据目录", set(scope) == expected and not unchanged,
           {"scope": scope})
    banned = ["多帧验收通过", "动画一致性已覆盖", "上游等价"]
    hits = []
    for text, name in ((track, "tracking note"), (roadmap, "roadmap note"), (readme, "closeout README"),
                       (json.dumps(rec, ensure_ascii=False), "closeout receipt")):
        for term in banned:
            if term in text:
                hits.append((name, term))
    record("E2 两卡 note、收口 README/receipt 中无精确禁词正向声明", not hits, {"hits": hits})
    closeout_text = json.dumps(card(CLOSEOUT), ensure_ascii=False)
    only_card = all(term in closeout_text for term in banned)
    if only_card:
        obs("精确禁词仅出现在 closeout 卡自身的 `forbid_claim` 词表与否定句（不写…）中；两卡 note 与收口证据"
            "均无正向声明，属工具词表/否定语境，与 G-3 复核 O2 同型（记已知）", {"card_terms": banned})
    record("E3 工作树除本复核证据/卡/gates 外 clean",
           not [ln for ln in git("status", "--porcelain").stdout.splitlines()
                if ln.strip() and not any(o in ln for o in
                                          (".normify/changes/2026-11-05-render-r06-multiframe-closeout-check.json",
                                           "docs/development-evidence/render-bvh/r06-multiframe-closeout-check/",
                                           ".normify/runs/2026-11-05-render-r06-multiframe-closeout-check/"))])

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
