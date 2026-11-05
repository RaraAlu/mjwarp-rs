"""O1 治理复核：`.gitattributes` 为 scene-flex 文本夹具固定 eol=lf（fresh clone 字节安全）。

前置：治理卡 `2026-11-05-repo-scene-flex-fixtures-eol-lf` 已落库（cce3e2e3 + 落库后复核 c7eb4737）。
只读；fresh clone 落 `target/o1-verify/clone`（不落仓）。

判定（Chief 6 项口径）：
  A. `.gitattributes` 只新增目标规则（1 行规则 + 2 行注释 + 1 空行 = 4 行；无删除/修改既有行）；
     规则行号在 `*.npz -text -diff` 之前（npz 仍 -text）。
  B. 不 renormalize / 不改 blob：15 个夹具 blob id 在改动父提交与 HEAD 逐一相同；改动提交不含
     fixtures/src/tests/scripts/冻结 spec。
  C. 属性生效：3 文本夹具 text=set/eol=lf；12 npz text=unset/diff=unset。
  D. 本机工作树 15 个夹具 sha256/bytes 与 G-2 `files-sha256.json` 15/15 一致。
  E. fresh clone（core.autocrlf=true，HEAD）：15/15 sha/bytes 一致（3 文本 LF 8,948/3,805/1,729 B；
     12 npz 逐字节不变）；clone 干净；G-2 证据目录 manifest 在 clone 中仍 16/16。
  F. 对照（必要性）：同一 clone checkout 改动父提交（autocrlf=true）→ 3 文本 CRLF、sha 不一致（复现 O1），
     npz 仍一致。
  G. 治理证据：manifest 11/11 自核、UTF-8/LF/无 BOM、无裸 CR/TAB；artifacts/gitattributes.diff 与实际 diff 一致；
     clone-postcommit.json verdict 全 true；receipt task_id 正确。

用法（仓库根，pinned venv）：
  target/mjwarp-parallel/venv/Scripts/python.exe -B target/o1-verify/verify_o1_review.py [--json <path>]
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
G2_EVID = REPO / "docs/development-evidence/render-bvh/r06-flex-multiframe-capture-g2"
GOV_EVID = REPO / "docs/development-evidence/repo-scene-flex-fixtures-eol-lf"
FILES_SHA = json.loads((G2_EVID / "artifacts/files-sha256.json").read_text(encoding="utf-8"))
CLONE = REPO / "target/o1-verify/clone"
RULE = "fixtures/render-bvh/scene-flex/** text eol=lf"
TEXT_FILES = ["fixtures/render-bvh/scene-flex/multiframe/capture.json",
              "fixtures/render-bvh/scene-flex/multiframe/cam-poses.json",
              "fixtures/render-bvh/scene-flex/multiframe/README.md"]
RESULTS: list[dict] = []


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def git(*args: str, cwd: pathlib.Path | None = None) -> subprocess.CompletedProcess:
    return subprocess.run(["git", "-C", str(cwd or REPO), *args], capture_output=True,
                          encoding="utf-8", errors="replace")


def record(name: str, ok: bool, detail=None) -> None:
    RESULTS.append({"name": name, "ok": bool(ok), "detail": detail})
    print(f"[{'PASS' if ok else 'FAIL'}] {name}: {detail}")


def main() -> int:
    # 定位 .gitattributes 改动提交（HEAD 链上最后一个改动它的提交）及其父提交
    log = git("log", "-2", "--format=%H", "--", ".gitattributes").stdout.split()
    change, parent = log[0], git("rev-parse", f"{log[0]}^").stdout.strip()
    record("A1 定位 .gitattributes 改动提交与父提交", bool(change) and bool(parent),
           {"change": change[:12], "parent": parent[:12]})

    diff = git("diff", "-U0", f"{parent}..{change}", "--", ".gitattributes").stdout
    added = [ln[1:] for ln in diff.splitlines() if ln.startswith("+") and not ln.startswith("+++")]
    removed = [ln[1:] for ln in diff.splitlines() if ln.startswith("-") and not ln.startswith("---")]
    rule_lines = [ln for ln in added if ln.strip() == RULE]
    other = [ln for ln in added if ln.strip() and not ln.startswith("#") and ln.strip() != RULE]
    record("A2 .gitattributes 只新增目标规则（1 规则 + 2 注释 + 1 空行），无删除/修改既有行",
           rule_lines == [RULE] and not removed and not other and len(added) == 4,
           {"added": added, "removed": removed})
    lines = (REPO / ".gitattributes").read_text(encoding="utf-8").splitlines()
    idx_rule = [i + 1 for i, ln in enumerate(lines) if ln.strip() == RULE]
    idx_npz = [i + 1 for i, ln in enumerate(lines) if ln.strip() == "*.npz -text -diff"]
    record("A3 规则位于 `*.npz -text -diff` 之前（npz 仍被 -text 覆盖）",
           bool(idx_rule) and bool(idx_npz) and idx_rule[0] < idx_npz[0],
           {"rule_line": idx_rule, "npz_line": idx_npz})
    raw_attr = (REPO / ".gitattributes").read_bytes()
    record("A4 .gitattributes UTF-8/无 BOM、无 TAB/裸 CR、行尾 LF",
           raw_attr.decode("utf-8") is not None and not raw_attr.startswith(b"\xef\xbb\xbf")
           and b"\t" not in raw_attr and b"\r" not in raw_attr)

    # B. 不 renormalize：15 夹具 blob id 不变；提交范围
    rels = [item["path"] for item in FILES_SHA["files"]]
    def blob_id(rev: str, rel: str) -> str:
        out = git("rev-parse", f"{rev}:{rel}").stdout.strip()
        return out if len(out) == 40 else "MISSING"
    changed_blobs = [r for r in rels if blob_id(parent, r) != blob_id("HEAD", r)]
    record("B1 15 个夹具 blob id 在父提交与 HEAD 相同（无 renormalize / 无 blob 重写）",
           not changed_blobs, {"changed": changed_blobs[:5]})
    scope = git("diff", "--name-only", f"{parent}..{change}").stdout.split()
    bad_scope = [p for p in scope if p.startswith(("fixtures/", "src/", "tests/", "scripts/",
                                                  "docs/development-evidence/render-bvh/r06-flex-multiframe-criterion/"))]
    allowed = all(p == ".gitattributes" or p.startswith((".normify/", "docs/development-evidence/repo-scene-flex-fixtures-eol-lf/"))
                  for p in scope)
    record("B2 改动提交范围 = .gitattributes + 卡/runs/证据，无 fixtures/src/tests/scripts/冻结 spec",
           not bad_scope and allowed, {"scope": scope})

    # C. 属性（当前工作树）
    cur_attrs = {}
    for rel in TEXT_FILES + ["fixtures/render-bvh/scene-flex/multiframe/frame-000/state.npz"]:
        cur_attrs[rel.split("multiframe/")[1]] = git("check-attr", "text", "diff", "eol", "--", rel).stdout.replace("\n", " | ").strip()
    text_ok = all("text: set" in cur_attrs[r.split("multiframe/")[1]] and "eol: lf" in cur_attrs[r.split("multiframe/")[1]]
                  for r in TEXT_FILES)
    npz_attr = cur_attrs["frame-000/state.npz"]
    record("C1 3 文本夹具 text=set / eol=lf；npz text=unset / diff=unset",
           text_ok and "text: unset" in npz_attr and "diff: unset" in npz_attr,
           {"attrs": cur_attrs})

    # D. 本机工作树 15/15
    bad = []
    for item in FILES_SHA["files"]:
        raw = (REPO / item["path"]).read_bytes()
        if sha(raw) != item["sha256"] or len(raw) != item["bytes"]:
            bad.append(item["path"])
    record("D1 本机工作树 15 个夹具 sha256/bytes 与 G-2 files-sha256.json 15/15 一致", not bad,
           {"bad": bad[:5]})

    # E. fresh clone（autocrlf=true，HEAD）
    CLONE.parent.mkdir(parents=True, exist_ok=True)
    if not CLONE.is_dir():
        cp = git("clone", "--no-checkout", "--quiet", str(REPO), str(CLONE))
        print("clone exit:", cp.returncode)
    else:
        git("fetch", "--quiet", "origin", cwd=CLONE)
    git("config", "core.longpaths", "true", cwd=CLONE)
    git("config", "core.autocrlf", "true", cwd=CLONE)
    head = git("rev-parse", "HEAD").stdout.strip()
    ck = git("-c", "core.autocrlf=true", "checkout", "-f", head, cwd=CLONE)
    print("checkout HEAD exit:", ck.returncode)
    # 强制重新物化：避免上一次对照检查留下的父提交 CRLF 工作树因 blob 相同而不被重写
    for item in FILES_SHA["files"]:
        q = CLONE / item["path"]
        if q.exists():
            q.unlink()
    git("-c", "core.autocrlf=true", "checkout", "-f", head, "--", *[i["path"] for i in FILES_SHA["files"]], cwd=CLONE)
    print("clone HEAD now:", git("rev-parse", "--short", "HEAD", cwd=CLONE).stdout.strip())
    clone_bad, rows = [], []
    for item in FILES_SHA["files"]:
        raw = (CLONE / item["path"]).read_bytes()
        ok = sha(raw) == item["sha256"] and len(raw) == item["bytes"]
        rows.append({"file": item["path"].split("multiframe/")[1], "bytes": len(raw),
                     "crlf": raw.count(b"\r\n"), "match": ok})
        if not ok:
            clone_bad.append(item["path"])
    text_bad = [r for r in clone_bad if not r.endswith(".npz")]
    npz_bad = [r for r in clone_bad if r.endswith(".npz")]
    record("E1 fresh clone 后 3 文本夹具 sha/bytes 与证据一致（LF 8,948/3,805/1,729 B）",
           not text_bad, {"text_rows": [r for r in rows if not r["file"].endswith(".npz")]})
    record("E2 fresh clone 后 12 个 npz 字节不变（*.npz -text 仍生效）", not npz_bad,
           {"npz_mismatch": [r.split("multiframe/")[1] for r in npz_bad]})
    record("E3 fresh clone 后 15/15 夹具 sha/bytes 全部一致", not clone_bad, {"15/15": not clone_bad})
    clone_status = git("status", "--porcelain", cwd=CLONE).stdout.strip()
    record("E4 clone 工作树干净（autocrlf=true 下无待提交差异）", clone_status == "",
           {"status": clone_status[:200]})
    entries = [ln for ln in (G2_EVID / "manifest.txt").read_text(encoding="utf-8").splitlines()
               if ln and not ln.startswith("#")]
    bad_e = []
    for line in entries:
        h, n, rel = line.split()
        raw = (CLONE / "docs/development-evidence/render-bvh/r06-flex-multiframe-capture-g2" / rel).read_bytes()
        if sha(raw) != h or len(raw) != int(n):
            bad_e.append(rel)
    record("E5 fresh clone 中 G-2 证据 manifest 仍 16/16 自洽", not bad_e, {"bad": bad_e[:5]})

    # F. 对照：clone checkout 父提交（autocrlf=true）→ 复现 O1
    # 注：15 个夹具 blob 在父提交与 HEAD 相同，直接 checkout 不会重写工作树文件；
    # 必须先删除再物化，否则测不到规则前后的 eol 转换差异。
    paths = [item["path"] for item in FILES_SHA["files"]]
    ck2 = git("-c", "core.autocrlf=true", "checkout", "-f", parent, cwd=CLONE)
    print("checkout parent exit:", ck2.returncode)
    for rel in paths:
        q = CLONE / rel
        if q.exists():
            q.unlink()
    git("-c", "core.autocrlf=true", "checkout", "-f", parent, "--", *paths, cwd=CLONE)
    before_bad = []
    for item in FILES_SHA["files"]:
        raw = (CLONE / item["path"]).read_bytes()
        if sha(raw) != item["sha256"] or len(raw) != item["bytes"]:
            before_bad.append(item["path"])
    before_text = [r for r in before_bad if not r.endswith(".npz")]
    before_npz = [r for r in before_bad if r.endswith(".npz")]
    record("F1 对照（规则前父提交）：3 文本夹具 CRLF、sha 不一致（复现 O1）",
           len(before_text) == 3, {"text_mismatch": [r.split("multiframe/")[1] for r in before_text]})
    record("F2 对照：12 npz 在规则前也一致（差异只来自文本夹具）", not before_npz,
           {"npz_mismatch": [r.split("multiframe/")[1] for r in before_npz]})
    git("-c", "core.autocrlf=true", "checkout", "-f", head, cwd=CLONE)  # 先把 HEAD 移回
    for rel in paths:
        q = CLONE / rel
        if q.exists():
            q.unlink()
    git("-c", "core.autocrlf=true", "checkout", "-f", head, "--", *paths, cwd=CLONE)  # 先删再物化，复原到 HEAD 规则

    # G. 治理证据
    raw_m = (GOV_EVID / "manifest.txt").read_bytes()
    entries_g = [ln for ln in raw_m.decode("utf-8").splitlines() if ln and not ln.startswith("#")]
    bad_g = []
    for line in entries_g:
        h, n, rel = line.split()
        raw = (GOV_EVID / rel).read_bytes()
        if sha(raw) != h or len(raw) != int(n):
            bad_g.append(rel)
    record("G1 治理证据 manifest 首行 #、11/11 自核", raw_m.startswith(b"#") and len(entries_g) == 11 and not bad_g,
           {"entries": len(entries_g), "bad": bad_g[:5]})
    prob = []
    for p in sorted(GOV_EVID.rglob("*")):
        if not p.is_file():
            continue
        raw = p.read_bytes()
        try:
            t = raw.decode("utf-8")
        except UnicodeDecodeError:
            prob.append(f"{p.relative_to(GOV_EVID)}:not-utf8")
            continue
        if raw.startswith(b"\xef\xbb\xbf") or "\r" in t or "\t" in t:
            prob.append(f"{p.relative_to(GOV_EVID)}:bom/cr/tab")
    record("G2 治理证据 UTF-8/LF/无 BOM、裸 CR/TAB 0 命中", not prob, {"problems": prob[:6]})
    art_diff = (GOV_EVID / "artifacts/gitattributes.diff").read_bytes().decode("utf-8")
    diff_full = git("diff", f"{parent}..{change}", "--", ".gitattributes").stdout
    record("G3 artifacts/gitattributes.diff 与实际 default-context diff 一致",
           art_diff.replace("\r\n", "\n").strip() == diff_full.replace("\r\n", "\n").strip(),
           {"artifact_lines": len(art_diff.splitlines()), "actual_lines": len(diff_full.splitlines())})
    post = json.loads((GOV_EVID / "artifacts/clone-postcommit.json").read_text(encoding="utf-8"))
    txt = json.dumps(post, ensure_ascii=False)
    verdict_ok = "true" in txt.lower() and ("false" not in txt.lower() or False)
    record("G4 clone-postcommit.json verdict 全 true（真实 HEAD 复核）",
           all(v is True for v in _walk_bools(post)) if _walk_bools(post) else False,
           {"json_keys": list(post.keys())[:8]})
    gov_card = json.loads((REPO / ".normify/changes/2026-11-05-repo-scene-flex-fixtures-eol-lf.json").read_text(encoding="utf-8"))
    record("G5 治理卡 verified + addenda 记录落库后复核（c7eb4737）",
           gov_card.get("status") == "verified" and any("fresh clone" in a.get("reason", "") for a in gov_card.get("addenda", [])),
           {"status": gov_card.get("status"), "addenda": len(gov_card.get("addenda", []))})

    failed = [it["name"] for it in RESULTS if not it["ok"]]
    payload = {"result": "PASS" if not failed else "FAIL", "total": len(RESULTS),
               "passed": len(RESULTS) - len(failed), "fails": failed, "checks": RESULTS,
               "change_commit": change, "parent_commit": parent, "clone_head": head}
    if "--json" in sys.argv:
        out = pathlib.Path(sys.argv[sys.argv.index("--json") + 1])
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n",
                       encoding="utf-8", newline="\n")
    print(f"\nRESULT: {payload['result']} ({payload['passed']}/{payload['total']} pass, {len(failed)} fail)")
    return 0 if not failed else 1


def _walk_bools(node) -> list[bool]:
    out: list[bool] = []
    if isinstance(node, dict):
        for v in node.values():
            out.extend(_walk_bools(v))
    elif isinstance(node, list):
        for v in node:
            out.extend(_walk_bools(v))
    elif isinstance(node, bool):
        out.append(node)
    return out


if __name__ == "__main__":
    raise SystemExit(main())
