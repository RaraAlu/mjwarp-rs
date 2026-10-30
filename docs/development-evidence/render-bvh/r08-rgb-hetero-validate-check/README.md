# R08-RGB 逐相机掩码验证窗（attempt=1，常规 util<40 阈值）· QA 独立复核

任务卡：`2026-10-30-render-r08-rgb-hetero-validate-check`（Owner QA）
被核对象：`2026-10-29-render-r08-rgb-hetero-validate`（Backend，**verified**）
pin revision：**`c19ca943`**（窗口证据 + guards 产物都在该提交）；复核卡 `revision.before` = `337abc14`（卡注册提交）
窗口基线：`33089ff9`。**只读**：未跑设备执行、未改证据；允许的纯编译门/四门禁已自跑（`logs/`）。

---

## 1. 结论

**verdict：pass（窗口执行事实与全部数值判据独立成立）/ 阻断 0 / 需修 1（证据卫生：manifest 2/31 行与 committed blob 不符）/ 观察 8。**

| 复核点 | 我的独立结果 | |
| --- | --- | --- |
| guards 身份 | `e81d69a4…` / **28,970 B** / suite `render-r08-rgb-hetero` / attempt=1 / **77 = 9 exe + 68 输入** | ✓ |
| aggregate | 按 `aggregate_algorithm` **独立重算 = `3ad8b352…`**（与登记一致；见 §2 排序语义说明） | ✓ |
| 77 条目 | 9 exe 的 `live == guards == stash` 全等；68 输入 sha 全对现盘；`sha-guards.txt` **77 行 SHA_OK** | ✓ |
| `-GuardsOnly` | `guard.log` 有 `GUARDS_OK=77/77` + stash 9/9；**但 `GUARDS_ONLY_DONE` 未单独留痕**（O2） | ✓（附观察） |
| 9 实体 | `entity-results.csv` 9/9 `exit=0`（1 new + 8 回归）；各日志 `test result: ok. N passed; 0 failed`，N 与 README 表一致 | ✓ |
| 新实体判据 | 8 用例逐条：三态 RGB、depth/seg 交叉、4 个矩阵用例全 0、cam1 三通道全关、隔离/重排、动态 vs 预计算、两跑 sha 相等、空选择 no-op | ✓ |
| 8 回归 | 8/8 `test result: ok`、0 failed、判据行无非零漂移（仅既有跨配方对照行非零，见 O5） | ✓ |
| 零编译 | deps `1839 → 1839`、max_write 两次同为 `10/30/2026 02:53:25`（早于 12:05 窗口）、`build_processes` 空 | ✓ |
| monitor | 我重算 **60 samples / longest_clean=60 / util 2–8 / free_min 13,452 / ge40=0 / ge60=0 / filtered=0 / build=0**；`exception_load_bearing=false`（常规窗口） | ✓ |
| samples/floor | **177 samples**、used_peak 2,883（seq 79）、used_min 2,605、free_min **13,179**、util_max 7、streak≥80=0、delta 276 ≤ 3,072；三个 floor 口径（10,240 / 10,383 / 10,374）均未破 | ✓ |
| 历史四轮 | 4 个 CSV 原样保留（354 samples，longest 0/37/7/37），与 target 副本**逐字节一致**；round4 失败记录 `window_commands_executed=0` | ✓ |
| 常规口径 | 三处一致 `not_a_normal_threshold_pass=false`、`authoring_exception_used=false`、命令无 `-OpenUtilLimit`/`-MaxUtil` | ✓ |
| 证据卫生 | 32 文件 committed blob **全部 LF 无 BOM**；receipt `/task_id` 正确；**manifest 2/31 行不符（需修）** | 见 §10 |

---

## 2. guards 身份与 aggregate 重算

- `scripts/validate/render-bvh/guards-r08-rgb-hetero.json`：committed blob **28,970 B / sha256 `e81d69a4daa0268777eff698ecf7621dee333076915f805c2e447cc9d0c34f3a`**（与卡面一致）。
- 条目 **77 = 9 frozen exe + 68 read-only inputs**；`entity_order` 与实体表顺序一致；68 输入组成我独立复现（8 hfield capture + 2 hfield xml + 42 legacy（38+4 skybox）+ 16 scene-shadow capture）。
- **aggregate 独立重算**：按 `aggregate_algorithm`（77 条按 repo 相对路径排序、sha256 hex 串接、无分隔再 sha256）用 PowerShell（与冻结器同语义）重算 = **`3ad8b3522314222629f1cf8e5a9703a9c53ad75433373d0c06f402803a517f1c`** ✓。
  排序语义备注：PowerShell `Sort-Object` 为文化敏感排序；用 Python 默认 ordinal 排序会得到不同值，故复核必须按工具语义算（这不是证据缺陷）。
- **stash**：`target/render-bvh/r08-rgb-hetero-window/frozen-exes/` 9 个副本，逐 exe `bytes+sha256` 与冻结记录一致；9 个 live exe 亦与 guards 记录一致（窗后 exe 复查 9/9 与我独立核对一致）。
- R05 / R02 guards 与 4 个既有 runner/freezer **零改动**（相对 `33089ff9` 的 diff 为空）。

## 3. `-GuardsOnly` 与 stash

- `window-command.txt` 逐字 4 条：monitor（`-Required 60 -MaxMinutes 30`）→ freeze（`-Attempt 1`）→ run-window `-GuardsOnly` → run-window（stage B）；**无 `-OpenUtilLimit`、无 `-MaxUtil`**。
- `guard.log`：`PRECHECK_OK open=2607/13455/2/31;…`、`GUARDS_OK=77/77`、`WINDOW_RUN_DONE`；PRECHECK 三点与 `baseline.txt` **同一多重集合**（格式不同，我归一化后一致）。
- **观察 O2**：`-GuardsOnly` 那次运行的 `GUARDS_ONLY_DONE` **没有单独留痕**——两次运行都写 `attempt1/guard.log`，body 运行覆盖了 GuardsOnly 的输出；77/77 与 stash 9/9 由 body 运行同一校验逻辑证明，README/receipt 的「GUARDS_ONLY_DONE」宣称缺少对应文件。

## 4. 窗口事实（9 实体）

- `entity-results.csv` 9 行：`render_r08_cameras`(new) exit 0 / 11,644 ms / 8 passed；8 回归 exit 0 / wall 1,312–9,378 ms，逐实体 passed 2/5/5/6/6/3/2/1 与 README 表一致。
- `facts.txt`：`sha_guards=77/77 mismatches=0`、`post_window_exe_recheck=9/9 mismatches=0`、`residual_processes=0`、`entity_count=9`、`entity_new=1 entity_regression=8`、`attempt_valid_rounds=1`。
- 窗口时间段 `12:05:31.632–12:06:27.460`，`body_wall_s=52.7` 与 samples 时间窗自洽；`window-command.txt` 4 条与 facts/guard.log 一致。

## 5. 新实体逐条判据（`logs/render_r08_cameras.log`，8 用例全 pass）

- **异构 RGB**：`cam0_on rgb_adr=[0,-1]` / `cam1_on [-1,0]` / `all_off [-1,-1]`；三态 sha 分别 `f62f4d73…`、`1325755c…`、`b09e2eec…`（与 README/receipt 一致）。
- **异构 depth/seg 交叉**：`depth_adr=[0,-1]`、`seg_adr=[-1,0]`，逐相机对参考成立 + 未启用相机拒绝（含 `MegakernelRenderer` 负偏移回归）。
- **矩阵 4 用例**：`all`/`names_all` sha `f9fe1b9e…`（120,000 px）、`mask_first` `cfc1a4b6…`（76,800）、`indices_reversed` `73891435…`；`rgb_beyond_1_lsb=0`、`depth_mismatch=0/max_depth_ulp=0`、`seg_mismatch=0` 全部为 0。
- **cam1 三通道全关（QA O1）**：cam0 三通道对参考、cam1 rgb/depth/seg 全部拒绝（覆盖内核 all-off early-return）。
- **隔离/重排**：subset_cam0 76800/0、subset_cam1 0/43200、reorder 76800/43200，均「逐位相同」。
- **动态 vs 预计算**：逐位相同，sha `b09e2eec…`。
- **两跑确定性**：run0 == run1 == `f9fe1b9e…`（且等于矩阵 all 的 sha）。
- **空选择 no-op**：mega/depth-seg 均返回 Ok no-op、缓冲未分配、无可读输出。

## 6. 8 回归零漂移

8/8 日志 `test result: ok. N passed; 0 failed`；对每条日志的判据行做非零扫描（排除既有跨配方对照行）**无非零漂移**。
**观察 O5**：`render_kernel_depth_seg` 的日志含 `s7b1[s0b-fast-math]` **跨配方对照行**（`depth_mismatch=19074/4064`、`max_ulp=2/4` 等）——这是既有参考的跨配方差异记录，不是本窗漂移；验收判据行是 `s7b1[recipe]`（`depth_mismatch=0 max_ulp=0`）。我按 recipe 行判 0 漂移。

## 7. 资源、floor 与 monitor

- **monitor（第五轮，本轮）**：60 samples / **longest_clean=60** / ready=True / exit 0 / `12:00:05.796–12:05:05.753`；util 2–8、free_min 13,452、ge40=0、ge60=0、filtered_max=0、build_max=0；`monitor/recompute.json` 与我重算一致，且 `exception_load_bearing=false`（证明门槛是常规 40、非 60）。
- **窗口 samples**：177 行，used_peak 2,883（seq 79）、used_min 2,605、free_min **13,179**、util_max **7%**、`util_streak_ge_80=0`；`delta_vs_guard_baseline=276 ≤ 3072`。
- **floor 三口径**（都不破）：`facts` 的 run-window `floor_mib=10240`（OpenFreeMiB 默认）、`baseline_free 13455−3072=10383`、`monitor/pre-window-snapshot-attempt5.txt` 的 `baseline_min_free 13446−3072=10374`；窗内 free_min 13,179 远高于三者。
- **零编译**：deps 计数 1839→1839、max_write 同为 `10/30/2026 02:53:25`（**早于窗口开始 12:05**）；`machine-load.txt` 中 build_processes 空、无 cargo/rustc 行（该文件无时间戳字段，见 O3）。

## 8. 四轮历史原样保留（R1 风险已消除）

| 轮 | samples | longest_clean | util 范围 | ge40 | filtered_max |
| --- | --- | --- | --- | --- | --- |
| round1 | 354 | **0** | 30–45 | 83 | 1（外部 CUDA context） |
| round2 | 354 | **37** | 22–100 | 140 | 0 |
| round3 | 354 | **7** | 11–99 | 287 | 0 |
| round4 | 354 | **37** | 35–44 | 40 | 0 |

- 四个 CSV 与 `target/render-bvh/r08-rgb-hetero-window/history/` 对应副本**逐字节一致**；round4 的快照与证据副本一致；`monitor-round4-failure.txt` 记 `ready=False / window_commands_executed=0 / 无 freeze 产物`，`monitor-round4-recompute.json` 与我重算一致（354/37/False）。
- 本轮（第五轮）用独立文件名 `monitor/monitor-pre-window.csv`，未覆盖任何历史轮 ✓。

## 9. 常规窗口口径（无例外，三处一致）

`not_a_normal_threshold_pass=**false**`、`authoring_exception_used=false`、`exception_used=false`、`open_util_limit=null`（receipt.signature）、`facts.authoring_exception_used=false`、window-command 无 `-OpenUtilLimit`/`-MaxUtil`；monitor CSV 中 **clean 样本的 util 全部 < 40**（阈值=40，非 60）。

## 10. 证据卫生（32 文件）

- 32 个证据文件 committed blob **全部 LF、无 BOM**；`receipt.task_id` 正确；`not_a_normal_threshold_pass`/`authoring_exception_used` 在 README/receipt/facts **三处自洽**。
- **需修（P1）**：`manifest.txt` 31 行中 **2 行与 committed blob 不符**：
  - `guard_aggregate.json`：manifest 记 `1699 B / d7fa6428…`，blob 实为 `1654 B / b1812cc6…`；
  - `monitor/pre-window-snapshot-attempt5.txt`：manifest 记 `1874 B / 09924751…`，blob 实为 `1872 B / f2677caa…`。
  根因：manifest 按**工作区字节（CRLF）**生成，committed blob 被 git 归一为 LF（两文件工作区 LF 归一后与 blob 完全一致）。修法：按 committed blob 重算这 2 行后重提交 manifest（或先把两文件归一化 LF）；**不需重跑窗口**。详见 `materials/manifest-mismatch-detail.txt`。
- 禁词：README §9 否定式非声明句里出现「上游+等价」连写 1 处（记录，O7）；normify 自身 check 通过。

## 11. 范围（相对窗口基线 `33089ff9`）

`git diff 33089ff9..c19ca943` = **35 文件**：证据目录 32 + `guards-r08-rgb-hetero.json` 1 + `.normify` 卡 2；**无第三类**。`src/**`、`tests/**`、`fixtures/**`、`Cargo.*` 零改动；其它 scripts（freeze/run-window/monitor、R05/R02 guards）零改动。`c19ca943..337abc14` 仅新增本复核卡文件。

## 12. 门禁（我自跑）

fmt / clippy / clippy-cuda **exit 0**；`cargo test --locked` **exit 0**：**136 result 行 / 1106 passed / 0 failed / 392 ignored**。主窗口卡 checks 仅 `json`（无 gate），故无 `.normify/runs/<id>` 目录，属预期（O6）。
说明：我的门禁构建发生在窗口**之后**，不影响窗口 `zero_rust_compile` 的判断（后者由窗口内 deps 快照 + max_write 证明）。

## 13. limits

- 只读复核：未跑任何设备执行；数值结论只覆盖本窗 attempt=1 的 9 实体。
- 我的 cargo 构建在窗口后执行，`target/` 当前状态已不是窗口时状态（zero_compile 用 facts 快照判，不依赖当前 target）。
- aggregate 重算按工具（PowerShell Sort-Object）语义；改变排序语义会得到不同 aggregate，重算时必须沿用 `aggregate_algorithm` 的实现口径。

## 14. 观察（需修 1 + 观察 8）

| # | 级别 | 观察 | 建议 |
| --- | --- | --- | --- |
| P1 | **需修（轻微）** | `manifest.txt` 2/31 行与 committed blob 不符（工作区 CRLF 字节 vs LF blob），违反卡面「manifest 与 committed blob 逐个一致」 | 按 blob 重算 2 行后重提交（或两文件归一 LF）；不需重跑 |
| O1 | 建议 | `guard_aggregate.json` 的 `suite` 字段 = `r05-s3b-hfield-validate`（`run-window.ps1:342` 硬编码），与 guards 文件 `render-r08-rgb-hetero` 不一致 | 把 suite 参数化（或从 guards/命令推导），否则跨套件证据会误导 |
| O2 | 记录 | `-GuardsOnly` 的 `GUARDS_ONLY_DONE` 未单独留痕（两次运行同写 `attempt1/guard.log`，body 覆盖） | 让 runner 分文件名落盘（如 `guard-guards-only.log`），或在 facts 记 `guards_only=true+sha` |
| O3 | 记录 | `machine-load.txt` 无时间戳字段，不能独立锚定「窗口体内无编译」；需与 facts 的 deps 快照 + max_write 交叉 | 接受（我已交叉核）；建议快照加时间戳 |
| O4 | 记录 | README §7 如实披露首次快照写入失败（相对路径 bug），仅 min 13453 留存；floor 改用 13446 口径 | 透明记录，接受 |
| O5 | 记录 | `render_kernel_depth_seg` 日志含 s0b-fast-math 跨配方对照行（非零）；判据行 `[recipe]` 全 0 | 接受（既有记录）；报告里别把对照行当漂移 |
| O6 | 记录 | 主窗口卡 checks 仅 json、无 `.normify/runs/<id>` | 接受（卡面如此设计） |
| O7 | 记录 | README §9 否定式非声明里「上游+等价」连写 1 处 | 接受（normify check 通过）；若要求字面 0 可改写 |
| O8 | 记录 | 四轮失败历史 CSV 与 target 副本逐字节一致、round4 失败记录完整（`window_commands_executed=0`） | 确认（本复核前置风险 R1 已消除） |

---

## 15. 本卡文件

| 文件 | 内容 |
| --- | --- |
| `recompute.py` / `recompute.json` | **18 项**独立重算（guards/aggregate/77 条/stash/命令/9 实体/新实体判据/回归/零编译/samples/floor/monitor/历史/证据/manifest/范围/receipt/卡状态），problems=1（即 P1） |
| `logs/recompute.log` | 上述脚本 stdout |
| `logs/fmt.log`、`logs/clippy.log`、`logs/clippy-cuda.log`、`logs/unit.log` | QA 自跑四门禁原始输出 |
| `logs/gates-raw.txt` | 四门禁 exit 码 |
| `materials/window-recompute.txt` | guards/aggregate/monitor/samples/floor/零编译/历史轮的重算明细 |
| `materials/manifest-mismatch-detail.txt` | manifest 2 行不符的逐项对照（listed vs blob vs worktree）与修法 |
| `manifest.txt` | 本卡文件 sha256/字节/路径（不含自身） |
