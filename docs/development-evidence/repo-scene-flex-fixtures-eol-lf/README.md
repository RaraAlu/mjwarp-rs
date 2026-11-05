# 治理(工程): scene-flex 多帧文本夹具固定 eol=lf（fresh clone 下 sha/bytes 不漂移）

- 任务：`2026-11-05-repo-scene-flex-fixtures-eol-lf`（owner Backend、P11-发布准入、p60；登记见卡记录）
- 触发：QA 复核 G-2 采集窗的**治理 O1**——`fixtures/render-bvh/scene-flex/multiframe/` 下 3 个文本夹具
  （`capture.json` / `cam-poses.json` / `README.md`）在 `core.autocrlf=true` 的 fresh clone 中被 LF→CRLF，
  其 sha256/bytes 与采集时点 `files-sha256.json` 不一致；12 个 `.npz` 已由 `*.npz -text -diff` 保护、不受影响。
- 边界：**只改 `.gitattributes`（1 条规则 + 依据注释）+ 本证据目录**；**不 renormalize、不改任何既有 blob**；
  不动 `.git/info/attributes`、不动其它规则；无 Rust 改动。

## 1. 结论

- 在 `.gitattributes` 中 `*.npz -text -diff` **之前**追加
  `fixtures/render-bvh/scene-flex/** text eol=lf`（附中文依据注释）；
- fresh clone（`core.autocrlf=true`）验证：修正前 3 个文本夹具为 CRLF（`9,240 / 3,967 / 1,745` B，
  sha 与记录不一致）；规则应用后回到 LF（**`8,948 / 3,805 / 1,729` B**），**sha256/bytes 与 G-2
  `files-sha256.json` 逐条一致**；12 个 `.npz` 全程逐字节一致；clone 内 `git status` 为空。
- npz 行为不变：规则在 `*.npz -text -diff` 之前，后者后匹配优先 → npz 仍 `text: unset` / `diff: unset`。

## 2. 改动（diff 见 `artifacts/gitattributes.diff`，仅 +4 行）

```text
# R06 多帧文本夹具（capture.json / cam-poses.json / README.md）：manifest 记 sha256+字节数，
# fresh checkout（core.autocrlf=true）须按字节一致；本规则置于 *.npz -text -diff 之前（npz 仍为二进制）。
fixtures/render-bvh/scene-flex/** text eol=lf
<空行>
```

- 插入位置：`*.npz -text -diff`（原第 291 行）**之前**；既有行逐字未改；
- **未** `git add --renormalize`、**未**重写任何 blob（`git status` 不出现 `fixtures/` 内容变化）；
- 历史先例：`2026-11-05-repo-evidence-gitattributes-rule`（同文件、`docs/development-evidence/** -text -diff`）、
  `2026-10-25-evidence-bin-eol-hygiene`（同文件、二进制 `-text` 规则、fresh clone 验证方法）。

## 3. 属性对照（`logs/attr-before.txt` / `logs/attr-after.txt`）

| 路径 | 修正前 text/eol/diff | 修正后 text/eol/diff |
| --- | --- | --- |
| `.../multiframe/capture.json` | unspecified / unspecified / unspecified | **set / lf** / unspecified |
| `.../multiframe/cam-poses.json` | unspecified / unspecified / unspecified | **set / lf** / unspecified |
| `.../multiframe/README.md` | unspecified / unspecified / unspecified | **set / lf** / unspecified |
| `.../multiframe/frame-000/state.npz` | unset / unspecified / unset | **unset / ignored / unset**（npz 仍二进制） |

## 4. fresh clone 验证（方法 + 结果）

`scripts/verify_clone.py`（自包含，可复跑）：

1. `git -c core.longpaths=true -c core.autocrlf=true clone --no-local <repo> D:\eol-clone`（干净 clone，`status=0`）；
2. **before**：clone 于规则落地前提交（`a7c401b4`）→ 3 文本夹具 CRLF、sha/bytes 与 `files-sha256.json` 不一致
   （复现 QA O1），12 npz 一致；
3. **after**：把工作树 `.gitattributes`（与最终提交**同一字节**，sha 记入 `clone-compare.json`）复制进 clone，
   做 **clone-local 临时提交**（throwaway，不进仓/不推送），删除并重新 `git checkout` 3 个文本夹具以按已提交规则物化；
4. 结果：3 文本夹具回到 LF、**sha256/bytes 与 `files-sha256.json` 逐条一致**；12 npz 逐字节一致；
   clone `git status --porcelain` **为空**；`check-attr` 见 §3。

机器可读结果：`artifacts/clone-compare.json`（before/after 逐文件 sha/bytes/CRLF 计数 + verdict + clone-local 提交号）。
落库后再以**真实最终提交**做一次 fresh clone 复核，日志见后续 `Normify-Fixes` addendum（`logs/clone-postcommit.json`）。

## 5. 边界与非声明

- scope 仅 `.gitattributes` + 本证据目录（+ 卡/runs）；`unchanged src/`, `tests/`, `fixtures/`, `docs/evidence/` 全过；
- 不 renormalize、不改任何既有 blob、不改既有规则/`.git/info/attributes`；
- O2/O3（G-2 证据 README 措辞）按 Chief 口径记已知、不返工；
- 非声明：本卡只保证 fresh checkout 下 scene-flex 文本夹具按字节稳定；不改变任何采集/判据结论。

## 6. 证据清单

| 路径 | 说明 |
| --- | --- |
| `README.md` | 本文件 |
| `receipt.json` | 机器可读回执（`/task_id` 供 normify `json` 检查） |
| `artifacts/gitattributes.diff` | `.gitattributes` 改动 diff（+4 行，raw bytes） |
| `artifacts/clone-compare.json` | fresh clone before/after 逐文件 sha256/bytes/CRLF 与 verdict |
| `logs/attr-before.txt` / `logs/attr-after.txt` | `git check-attr text eol diff` 前后对照（3 文本 + npz） |
| `logs/clone-verify.log` | 验证脚本控制台输出（含 exit_code） |
| `scripts/verify_clone.py` | 自包含 fresh clone 验证脚本（可复跑） |
| `manifest.txt` | 本目录 sha256 + bytes + 证据目录内相对路径（首行 `#` 表头，不自列） |

## 7. 复现

```powershell
# 需网络无关的本地 clone；每次自建 D:\eol-clone
target/mjwarp-parallel/venv/Scripts/python.exe -B docs/development-evidence/repo-scene-flex-fixtures-eol-lf/scripts/verify_clone.py
```
