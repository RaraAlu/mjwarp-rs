# R05-S3b hfield 验证窗 attempt=2 · QA 独立复核

任务卡：`2026-10-28-render-bvh-r05-s3b-hfield-validate-check`（priority 57，Owner QA）
被核对象：`2026-10-28-render-bvh-r05-s3b-hfield-validate`（**verified**）
被核提交：`674426f5`（窗口与监测原始产物入库）→ `9717a8a4`（README/receipt/manifest）→ `634d0e4a`（收口）
复核基线 HEAD：`634d0e4a`；复核期 HEAD 漂到 `06cba487`（见 O9）。**只读**：未开 GPU 窗、未重跑窗口/实体、未跑 `freeze-guards.ps1`、未动 `guards.json`/stash。

---

## 1. 结论

**verdict：pass（窗口事实 / 判据数字 / 冻结 / 回归全部独立复核一致）/ 阻断 0 / 观察 10（其中 O1、O2 为「必修（证据记录层面）」，不需重跑窗口、不开 GPU）。**

我从原始产物自己重算/重跑得到的结论：

1. **例外合法性成立**（不是常规门槛通过）：常规 `util<40` 监测跑满 354 样本仍未达 60 连续干净 → `decision=STOP`、`window_commands_executed=0`，记录原样保留；例外 `validation_only_exception_util_lt_60` 的 5 要素齐全，`not_a_normal_threshold_pass=true`、`authoring_exception_used=false` 三处（README/receipt/facts）一致。**60 门槛的机器级证据在 `monitor/monitor-pre-window-attempt2-exception.csv`**：5 个 `util≥40`（40/41/41/40/40）样本被记为 `clean=True` 且 streak 连续 → 证明生效门槛是 60 而不是 40（若按 40 这些样本必须 unclean）。
2. **窗口事实一致**：9/9 实体 exit 0（wall 与 `facts.txt` 逐条对上）；`guard.log` = `PRECHECK_OK…` → `GUARDS_OK=73/73` → `WINDOW_RUN_DONE`；采样 171 样本、util 11–43、free_min 13035、floor 10233 未破、increment 270 ≤ 3072、`util_streak_ge_80=0`、`residual_processes=0`；`guard_aggregate.json` 与 `facts.txt` 10 项字段全等；deps 快照 1836 exe / max write 04:08:28（我现在盘上复核仍是 1836 / 04:08:28，无窗后重链）。
3. **判据数字全 0**（我从 hfield 日志逐行解析）：两组 × 两相机，RGB `mismatched=0 / max_byte_diff=0 / beyond_1_lsb=0`、depth `mismatch=0 / max_ulp=0 / beyond=0`、seg `mismatch=0`；depth_zero 29776（cam0）/37050（cam1）；两次运行 sha 由 3 条 `assert_eq`（rgb/depth/seg）钉住且 `test result: ok. 2 passed; 0 failed`。
4. **参考对齐**：6 个 npz 的现盘 sha = 各自 `capture.json.files_sha256` = S3a 采集证据 manifest 记录；两组 `captured_at` 同值 `2026-10-29T00:00:00+08:00`。
5. **冻结一致**：`guards.json` = `e2444a96…`/27 670 B/attempt=2，aggregate 复算 `7d24f036…`（用证据自带 `recompute-aggregate.ps1` 与 superseded 归档各跑一次，均 `matches_recorded=True`）；9 exe **live == guards == stash**（sha 与 bytes 逐项）；`sha-guards.txt` 73/73 行全部对现盘重算命中；`run-window.ps1` `31ac28bf…`/18 187、`freeze-guards.ps1` `b01abff2…`/7 803、`monitor-until-clean.ps1` `788d3505…`/2 509。
6. **完整性与零外溢**：`attempt1/` 原样保留（mtime 02:47–02:58，早于 attempt2 窗 04:56；独立目录）；64 项只读输入与 attempt1 归档 guards **64/64 逐项相同**；旧 guards `210f8208…` 归档文件 sha 复算一致并在 `materials/freeze-attempt2.txt` §2 标 superseded；`d1f7f851..HEAD` 只动证据与任务卡（`src`/`tests`/`fixtures`/`.cu`/`scripts` 零改动）；manifest 40 行 = 40/40 committed blob = 40/40 现盘，不自引用；37 个证据文件 LF/UTF-8/无 BOM，禁词 0。

**唯一实质缺口是证据记录的措辞/签名记录（O1/O2）**：README/receipt 把 `baseline.txt` 的 37/40/40 写成「precheck 采样」，而 `guard.log` 的 precheck 实际是 37/37/37；例外监测的实际命令行（含 `-MaxUtil 60`）未逐字留痕，且 README/receipt/runner-ascii-check/window-plan 仍记旧 monitor sha `ec2b959c…/2 486`（实际运行版 `788d3505…/2 509`）。两处都是文字修正，不牵动窗口有效性。

---

## 2. 例外合法性（重点，A 组）

### 2.1 常规阈值失败记录原样保留（我从 CSV 重算）

`monitor/monitor-pre-window-attempt2-final.csv`（354 行 + 表头）与 `-final-result.txt`：

| 项 | 记录值 | 我重算 |
| --- | --- | --- |
| samples | 354 | **354** |
| longest_clean | 45（< 60） | **45** |
| required | 60 | 60 |
| util_min / util_max | 1 / 49 | **1 / 49** |
| util≥40 样本 | 33 | **33** |
| util≥60 样本 | 0 | **0** |
| free_min | 13 163 MiB | **13 163** |
| decision | STOP | STOP |
| window_commands_executed | 0 | 0（该轮没有执行任何窗口命令） |
| build / compute_filtered 峰值 | — | 0 / 0 |

### 2.2 例外记录五要素（例外 CSV/result 我重算）

`monitor/monitor-pre-window-attempt2-exception-result.txt`：`mode=validation_only_exception_util_lt_60`、`approved_by=Chief 2026-10-29T04:5x（预批，条件批准）`、`reason=persistent desktop GPU graphics contexts; filtered compute-apps=0; build=0; bitwise criteria unchanged`、`original_normal_threshold_attempts_retained=…（samples=354, longest_clean=45, decision=STOP）`、`not_a_normal_threshold_pass=true`、`signature_unchanged=true`、`guards_sha256=e2444a96…`、`guards_bytes=27670`。

我重算例外 CSV（60 行）：samples **60**、longest_clean **60**、util **36–41**、util≥40 **5**、util≥60 **0**、free_min **13 271**、build 0、compute_filtered 0。

### 2.3 `-OpenUtilLimit 60` 与 60 门槛的机器级证据（**口径修正，见 O1**）

- `attempt2/window-command.txt` 逐字含 `-OpenUtilLimit 60`（`…run-window.ps1 -Attempt 2 -OutDir target\render-bvh\r05-s3b-window\attempt2 -OpenUtilLimit 60`）✓。
- **但 `guard.log` 的 `PRECHECK_OK open=2757/13305/37/30;…37/30;…37/30` 三个 precheck 样本 util 全是 37，没有 ≥40 样本**（`run-window.ps1` L111–129：precheck 用 3 个新 `GpuSample` 与 `-OpenUtilLimit` 比较；`baseline` 是 L131–135 另取 3 个样本写进 `facts.txt`）。
- `attempt2/baseline.txt` 的三行 = **37 / 40 / 40** —— 这正是 README §9.1 与 `receipt.attempt2_window.precheck_util_samples` 里写成「precheck 三样本」的那组数；它不是 precheck 采样。
- 因此**不能用 precheck 证明 40→60 门槛生效**（按默认 40，37/37/37 也会 PASS）。**真正的机器级证据在例外监测 CSV**：5 个 `util≥40` 样本被记 `clean=True`（`04:50:55` 40、`04:52:32` 41、`04:54:24` 41、`04:54:55` 40、`04:55:45` 40，streak 连续到 60）⇒ 生效门槛 60 成立。

### 2.4 不得表述为常规阈值通过

`not_a_normal_threshold_pass=true` 在 README §9.1、`receipt.attempt2_window`、`facts.txt` 例外块三处都有；`authoring_exception_used=false` 三处一致；全证据禁词扫描 0 命中（词表取自任务卡 forbid_claim，脚本内用码点构造，证据里不出现词表字面）。README §9 标题明确「validation-only 例外，例外实际生效」。✓

---

## 3. 窗口事实（B 组）

| 项 | 记录 | 我的核对 |
| --- | --- | --- |
| `entity-results.csv` | 9 行，exit 全 0 | 9 行 exit 全 0；wall 3812/1314/7845/8563/4568/9123/3571/8051/3041 与 `facts.txt` 的 `entity <name> exit=… wall_ms=…` **逐条一致** |
| `guard.log` | `PRECHECK_OK…` / `GUARDS_OK=73/73` / `WINDOW_RUN_DONE` | 三行齐全、顺序正确 |
| `facts.txt` | sha_guards=73/73 mismatches=0；post_window_exe_recheck=9/9；zero_rust_compile=True；deps 1836→1836；max write 04:08:28；floor 10233；free_min 13035；samples 171；util_max 43；residual 0；attempt=2 | 全部复算一致（§5 给出 exe 复核：73/73 行对现盘重算命中，等价于窗后 9/9 无漂移；deps 现盘仍 1836 exe / max 04:08:28） |
| `samples.csv`（171 行） | — | peak 3027 @seq52、used_min 2754、free_min 13035、util 11–43、`util_streak_ge_80`=0、delta 270（首样本与首基线同值）、floor 10233 未破、limit 3072 未超 |
| `guard_aggregate.json` | — | 与 `facts.txt` 的 samples/peak/peak_seq/used_min/free_min/util_max/floor/baseline/aggregate/sha_guards **10/10 全等** |

---

## 4. 判据（C 组，我逐行解析 hfield 日志）

`attempt2/logs/render_hfield_reference.log`（2 tests，`test result: ok. 2 passed; 0 failed; 0 ignored; 3 filtered out`）：

| 组 | 相机 | 像素 | RGB mismatched / max_byte_diff / beyond_1_lsb | depth mismatch / max_ulp / beyond | seg mismatch | depth_zero |
| --- | --- | --- | --- | --- | --- | --- |
| hfield-only | cam0 | 76 800 | 0 / 0 / 0 | 0 / 0 / 0 | 0 | **29 776** |
| hfield-only | cam1 | 43 200 | 0 / 0 / 0 | 0 / 0 / 0 | 0 | **37 050** |
| hfield-primitive | cam0 | 76 800 | 0 / 0 / 0 | 0 / 0 / 0 | 0 | **29 776** |
| hfield-primitive | cam1 | 43 200 | 0 / 0 / 0 | 0 / 0 / 0 | 0 | **37 050** |

两次运行 sha：`tests/render_hfield_reference.rs` L335–353 对 rgb/depth/seg 各一条 `assert_eq`（同一 context 重复发射），测试通过 ⇒ 两次 sha 相同；日志打印的 run sha（两组各 rgb/depth/seg）与 receipt 记录一致。

参考对齐：`fixtures/render-bvh/scene-hfield/capture-hfield/{hfield-only,hfield-primitive}/*.npz` 6 个文件现盘 sha = `capture.json.files_sha256` = S3a 采集证据 manifest 记录；两组 `captured_at` 同为 `2026-10-29T00:00:00+08:00`。判据常量（`MAX_DEPTH_ULP=4` 等）位于 `tests/support/render_bvh/hfield_scene.rs`，本次链未改（§6）。

---

## 5. 冻结（D 组）

| 项 | 期望 | 我实测 |
| --- | --- | --- |
| `scripts/validate/render-bvh/guards.json` | `e2444a96…` / 27 670 B / attempt=2 / aggregate `7d24f036…` | **全对**（`sha256=e2444a96…`，bytes=27 670，attempt=2，aggregate 复算=记录值） |
| 9 exe live == guards == stash | 9/9 | **9/9**：每个实体 `guards.sha256 == guards.stash_sha256`、live 文件 sha/bytes 与 guards 一致、stash 文件 sha/bytes 与 guards 一致 |
| `sha-guards.txt` | 73 行 | **73 行**，键集 = guards 的 73 项，逐行对现盘重算 sha+bytes 全中（9 exe + 64 输入） |
| `freeze-guards.ps1` | `b01abff2…` / 7 803 | **`b01abff2…` / 7 803**（LF 154 行、CR 0） |
| `run-window.ps1` | `31ac28bf…` / 18 187 | **`31ac28bf…` / 18 187** |
| `monitor-until-clean.ps1` | `788d3505…` / 2 509 | **`788d3505…` / 2 509**（README/receipt 记的是旧值，见 O2） |
| exe 冻结前最后构建 | 04:08:28 | 9 exe mtime 最大 `04:08:28`，均早于 guards `generated_at=04:10:37.668` |

aggregate 口径（我按证据自带脚本复算，并验证 superseded）：`recompute-aggregate.ps1` 对当前 guards 输出 `matches_recorded=True`；对 `target/render-bvh/r05-s3b-window/guards-attempt1-superseded.json` 输出 `105ad0ab… matches_recorded=True`。注意：用**序数排序**（Python `sorted()`）复算会得 `89895a621b…`（≠ 记录值），该差异已被 `materials/aggregate-sort-note.txt` 明确写为「不作为复算口径」——我复现了这两个值，口径说明成立。

分组聚合（按 guards 内 64 项，三种排序口径结果一致）：capture 8 文件/5 784 264 B/`d405ae21…`；legacy 38 文件/25 320 515 B/`b5cb50af…`；scene-shadow 16 文件/11 561 363 B/`cd990c20…`。

---

## 6. 完整性与零外溢（E 组）

- `attempt1/` 原样保留：目录存在，14 个条目（含 `.abort`、`rca-trace.txt`、`rca-zero-length-buffers.txt`、`window-command.txt`），mtime 最大 `02:58:52`，早于 attempt2 窗口 `04:56:10`；attempt1 `facts.txt` 首行 `attempt=1`。attempt2 独立目录，未覆盖。
- 64 项只读输入 vs attempt1 归档 guards：`guards-attempt1-superseded.json` 与当前 `guards.json` 的 `guard_inputs` **64/64 项 path/sha256/bytes 全等**。
- 旧 guards `210f8208…`/27 670 B 归档在 `target/render-bvh/r05-s3b-window/guards-attempt1-superseded.json`，我重算 sha 一致；`materials/freeze-attempt2.txt` §2 明确标 superseded（原因：FMA 修复后 9 exe 重链）。
- 零外溢：`git diff --name-status d1f7f851..HEAD` 只有证据目录 + 任务卡；`git diff --stat d1f7f851..HEAD -- src tests fixtures scripts` **空**；更早的 `2c56cc1a..HEAD -- tests fixtures src/render/kernel/cuda` **空**（`src` 的 3 个文件改动来自另外两个已复核片：拆分 megakernel、FMA 修复）。
- manifest：40 行，逐行 `sha256+bytes` 对 **committed blob** 与 **现盘** 都命中（40/40 + 40/40），不自引用；覆盖 README/receipt/4 个签名脚本/`guards.json`/attempt2 产物/monitor/材料。
- 证据卫生：37 个已提交文件全部 UTF-8 无 BOM、无 NUL、CR=0；禁词扫描 0。

---

## 7. 观察（10 条；O1/O2 为「必修（证据记录）」）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | **必修｜一般** | **precheck 采样值错记**：README §9.1「本窗窗口 precheck 三样本 util = 37/40/40」与 `receipt.attempt2_window.precheck_util_samples=[37,40,40]` / `precheck_note` 把 `attempt2/baseline.txt` 的基线三样本当成 precheck；`guard.log` 的 precheck 实际是 **37/37/37**（无 ≥40 样本）。卡面 ④ 要求的「≥40 的 open 样本仍 PASS」不存在 | 修：改为「precheck 采样 37/37/37；baseline 37/40/40；60 门槛的机器级证据在 exception.csv 的 5 个 util≥40 且 clean=True 样本」；删掉「precheck 证明例外生效」的推论 |
| O2 | **必修｜一般** | **例外监测脚本签名记录与实际不符 + 命令行未留痕**：README §2/§4、`receipt.scripts[2]`/`pre_window_monitor`、`materials/runner-ascii-check.txt`、`materials/window-plan.txt` 记 monitor `ec2b959c…`/2 486（旧版），实际运行版 `788d3505…`/2 509（正确值只在 `manifest.txt` 与 `freeze-attempt2.txt` §6，且 §6 只给截断 sha）；例外监测的实际命令行（`… monitor-until-clean.ps1 … -MaxUtil 60`）没有逐字留痕 | 修：① 补例外监测逐字命令；② 把 monitor 完整 sha/bytes 写进 README/receipt 并标注旧值 superseded；③ 补 `-MaxUtil` 参数化 diff 摘要（提交 `01e36a54`：param 默认 40 + `($util -lt $MaxUtil)` 两处，我核过默认行为等价）作为卡面 ⑤ 的证据 |
| O3 | 建议 | **README 头部与 receipt 顶层仍是 authoring 阶段文案**：README「阶段：离线 authoring 完成；窗口未开、未跑 GPU、未跑 `--ignored`、零渲染数值结论」；receipt `authoring_only: true`、`window: false`、`limits:["no GPU entity was executed…"]`、`pending: ""` —— 与同文件 §9 / `attempt2_window`（9 实体已跑、GPU 已执行）矛盾 | 修：改成 attempt2 实际状态 |
| O4 | 建议 | **attempt1 冻结物身份记录未标 superseded**：README §2/§3/§4 与 `receipt.guards`/`pre_window_monitor`、`materials/guard-manifest.txt`（header aggregate `105ad0ab…`、exe sha `90d5becd…` 等）仍是 attempt1 冻结值；attempt2 值只在 `materials/freeze-attempt2.txt`、`receipt.refreeze_attempt2`、README §9.1 出现 | 修：在这些位置标「superseded（attempt1）」或换成 attempt2 值，避免签名身份误读 |
| O5 | 建议 | `materials/freeze-attempt2.txt` §8 写「窗口命令…`OpenUtilLimit` 默认 40、monitor `-MaxUtil` 默认 40，不带任何例外覆盖」，与实跑 `-OpenUtilLimit 60` + `-MaxUtil 60` 矛盾 | 修：注明 §8 已被 Chief 预批的 validation-only 例外取代（提交 D 之后） |
| O6 | 建议 | **旧 capture 组聚合不可复现**：在 64/64 输入逐项未变的前提下，attempt1 记录的 capture 聚合 `c6705a80…` 用 array/culture/ordinal 三种口径都复现不出（均得 `d405ae21…`）；`freeze-attempt2.txt` §5 已披露该点，但 README 未写 | 建议把该限制写进 README「已知限制」 |
| O7 | 记录｜接受 | `guards.json` 签名字节依赖本地 `.git/info/attributes`（`scripts/validate/render-bvh/guards.json -text`）：我实测按 fresh-clone 的 LF 归一化字节 sha = `25cd1524…` ≠ `e2444a96…`。即「committed blob == 现盘」成立（当前仓库），但该属性不随提交分发 | 接受（`freeze-attempt2.txt` §1 已披露，并建议 freeze 脚本改写 LF） |
| O8 | 记录｜接受 | 证据副本与 `target/` 原始件：`window-command.txt`/`exception-block.txt` 原始件带 UTF-8 BOM + CRLF、`facts.txt` CRLF，证据副本已归一为 LF/无 BOM（内容逐行一致）；`monitor/` 4 个文件与原始件逐字节相同 | 建议在 README 写明证据副本的归一化口径 |
| O9 | 记录｜接受 | 复核期 HEAD 漂移 `634d0e4a` → `06cba487`（R05-S2 hfield 设备通路附录），只动 `r05-s2-hfield-intersect` 证据与 S2 卡片，未触 S3b validate 路径 | 接受（后续派工可指定提交号） |
| O10 | 建议 | 例外批准时间记录为 `04:5x`（5 分钟粒度模糊） | 建议补精确时间戳（批准人/模式/理由/条件都已在） |

---

## 8. 边界

1. **未开 GPU、未重跑窗口/实体**：判据数字来自窗口日志（我逐行解析并与 receipt/README 对照），不等于我独立渲染验证；8 个回归实体只核 exit 0 + 日志 `test result: ok` + facts，未重跑。
2. 未跑 `freeze-guards.ps1`、未改 `guards.json`/stash/证据；只运行了证据自带的只读 `recompute-aggregate.ps1` 与我自己写的只读脚本。
3. aggregate 按证据记录的口径（PowerShell culture sort）复算；序数口径结果不同（`89895a62…`）且该差异已被文档披露。
4. 证据副本的 3 个文件因 BOM/CRLF 无法与原始件逐字节相同（内容已核一致，见 O8）。
5. 本卡不产生任何渲染数值验收结论；只核留档、判据计数、冻结与例外记录。

---

## 9. 复现命令（仓库根目录）

```text
# 我的三段只读复算（Python 3.12 pinned；读 git blob + target/ 原始件）
target\render-bvh-precheck\venv312\Scripts\python.exe -B docs\development-evidence\render-bvh\r05-s3b-hfield-validate-check\recompute.py
target\render-bvh-precheck\venv312\Scripts\python.exe -B docs\development-evidence\render-bvh\r05-s3b-hfield-validate-check\recompute-window.py
target\render-bvh-precheck\venv312\Scripts\python.exe -B docs\development-evidence\render-bvh\r05-s3b-hfield-validate-check\recompute-integrity.py
# 证据自带 aggregate 复算（只读）
powershell -NoProfile -ExecutionPolicy Bypass -File docs\development-evidence\render-bvh\r05-s3b-hfield-validate\materials\recompute-aggregate.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File docs\development-evidence\render-bvh\r05-s3b-hfield-validate\materials\recompute-aggregate.ps1 -GuardsPath target\render-bvh\r05-s3b-window\guards-attempt1-superseded.json
```

（三段脚本的 `problems` / 关键断言见 `recompute*.json`；日志在 `logs/`。）

---

## 10. 收口

- 本卡证据提交：本目录（README/receipt/manifest/3 份 recompute 脚本与结果/logs/materials）。
- 被核对象三段提交 `674426f5` + `9717a8a4` + `634d0e4a` 保持不动（本卡只读）。
- O1/O2 是文字/留痕修正项，不需重跑窗口或开 GPU；修与不修、是否按卡面 ④⑤ 记阻断，请 Arch 裁定。
