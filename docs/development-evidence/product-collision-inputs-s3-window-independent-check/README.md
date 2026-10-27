# S3 验证窗 attempt1 独立复核（QA）

- 任务：`2026-10-27-product-collision-inputs-s3-window-independent-check`（复核对象 `2026-10-27-product-collision-inputs-s3-window`）
- 复核提交：窗口主提交 `45e07485`（22 文件：21 证据 + 窗口任务记录）+ 补正 `fa704a6e`（4 证据文件，`Normify-Fixes`）
- 复核时 HEAD：`fa704a6e`；窗口产物原件在 `target/s3-product-wiring/attempt1/`，未重开门、未开 GPU、未重跑
- 结论：**pass / 阻断 0 / 观察 7（2 一般 + 5 建议）**；所有关键数字均由 QA 自写脚本从 `samples.csv`/`guard.log`/`facts.txt`/
  `sha-guards.txt`/实体日志独立重算，未抄 Arch/Chief 的数值
- 边界：本复核只核 attempt1 的执行事实与证据一致性，**不宣称窗口结果、覆盖、精度、等价或任何验收结论**

## 1. 总体判定

8 项硬要求全部成立：10/10 实体 exit=0；新 target 3 case `nacon=4=ref`、`bit_equal=12/12`、dist/pos/frame ULP=0、
geom `[1,0]/[1,0]/[0,1]`、failures=0；S1 快照 32/32×3；9 回归 failures=0（stage2 只记录 17 dump）；
133/133 SHA_OK 重新哈希全 match；10 冻结 exe 窗后复查 10/10；`0c9497e4..HEAD` 无代码改动；
采样三口径 1051/1051/1055、free_min 11920>floor 9904、util 峰值 31、streak 0、residual 0；
零 Rust 编译事实成立；提交只含证据 + 窗口任务记录。

有两条「一般」观察需要更正记录或证据文件：**① 入库的 `logs/attempt1-runner-stdout.log` 是 UTF-16LE（带 BOM）**
且每行后多一空行，与 README/交付记录「全部证据 UTF-8 无 BOM、LF」的声明不符；**② 尖峰时间「launcher 退出后
约 0.7 s」的表述与产物不符**（尖峰在 launcher 日志写完前 86 ms，按产物只能支撑「退出/收尾边界 ±0.1 s」）。
两条都不影响峰值/结论，但 `window-receipt.json` 与 README 的相应文字应更正。

## 2. 采样自算（Chief 追加点 A）

自写 `verify_samples.py` 直接解析 `target/s3-product-wiring/attempt1/samples.csv`（867 行数据 + 表头）与
`guard.log`/`baseline.txt`，结果：

| 项 | QA 重算 | receipt 声明 | 一致 |
| --- | --- | --- | --- |
| samples | **867** | 867 | ✓ |
| baseline（guard_baseline.used_mib） | **3091** | 3091 | ✓ |
| used_peak | **4142** | 4142 | ✓ |
| delta vs guard baseline | **1051** | 1051 | ✓ |
| delta vs 首样本 | **1051** | 1051 | ✓ |
| delta vs 采样最小（3087） | **1055** | 1055 | ✓ |
| free_min | **11920** | 11920 | ✓ |
| util_max | **31** | 31 | ✓ |
| util≥80 连续 | **0** | 0 | ✓ |
| residual | **0**（guard_end `residual_context_pids=` 空、`process_exited=True`） | 0 | ✓ |
| floor = min(10240, 12976−3072) | **9904**；open free 12976 ≥ 10240 | 9904 / 12976 | ✓ |
| 峰值排除后 max / sustained delta | **3109 / 18** | 3109 / 18 | ✓ |
| samples>3120 | **1** | 1/867 | ✓ |
| 尖峰 | **seq 776 @ 17:03:36.7225390+08:00，4142 MiB** | 同 | ✓ |
| 采样间隔 | 中位 **0.2759 s**（min 0.2721 / max 0.2901），标称 0.25 s | 「0.25 s」 | 标称口径（见 O4） |

（`logs/samples-summary.txt`、`verify_samples.json`；24 项检查 0 问题。）

## 3. 尖峰定位与预算偏差口径（Chief 追加点 A）

- runner **在实体进程结束后才写实体日志**（`run-validation-attempt1.ps1` L253 写日志 → L256 `ENTITY_DONE`），
  所以日志 mtime ≥ 实体退出时刻：`nvrtc_launcher` 日志 mtime **17:03:36.808**；尖峰样本 17:03:36.7225390 在其
  **前 86 ms**。stage2 启动下界（日志 mtime−wall / window_end−wall）= **17:03:37.658 / 17:03:38.443**，
  尖峰比最早下界早 **0.936 s**。
- 因此可支撑的说法是「尖峰位于 **launcher 退出/日志收尾边界 ±0.1 s 内、stage2 启动之前**」，
  而不是「launcher 退出后约 0.7 s」（见 O2）；归因仍为**推断**（NVRTC/cuModule 临时分配释放滞后），
  未证实到具体分配点，与证据一致。
- 预算偏差口径 ✓：`window-receipt.json` L205-243、`receipt.json` 的 `s3.window.attempt1.budget_deviation`、
  交付记录对应段均写「预算偏差：实测 1051 MiB 单样本瞬态（270–320 预算的约 3.4 倍）」「归因推断（未证实到具体分配点）」；
  四个文件 `在预算内` 命中 **0**，`预算偏差` 合计 **7 处**（window-receipt/receipt/delivery 各 2、README 1）。
- guard_end/peak 不混用 ✓：窗口峰值只引 `guard_aggregate.used_peak_mib=4142` / `delta_peak_mib=1051`；
  `guard_complete.peak_used_mib=3091` 明确标注「仅结束相、不得作窗口峰值」，`guard_complete.samples=1` 标注为
  guard 状态离散断言计数（权威 867 来自 `guard_aggregate.*` + `samples.csv`）。`facts.txt`/`guard.log` 两处
  `guard_aggregates` 行与我重算一致。

## 4. 守卫、冻结与来源（Chief 追加点 B）

- **133 行 `SHA_OK` 逐条重新哈希**：QA 自己读 `sha-guards.txt`，对 10 个 exe（`target/s3-product-wiring-freeze/`）与
  123 个只读输入（repo 相对路径）逐个重算 sha256 与字节数 → **133/133 match，0 不符**；`facts.txt` 里的 133 行
  `SHA_OK` 与 `sha-guards.txt` **内容完全相同**（集合相等）。
- **窗后 10 exe 复查**：当前 freeze 目录 10 个 exe 的 sha/字节与守卫、`frozen-shas.txt`、receipt 四方一致（10/10）。
- **来源按 `frozen-shas.txt` source 列**：`s3-build` **1** + `s4-build` **1** + `m3-freeze(M3-signed)` **8** = 10；
  与 M3 已签冻结清单逐字节比对 **9/9**（8 个 m3-freeze 命中 `target/m3-freeze/frozen-shas.txt`；launcher 是 S4
  重建版，命中 `target/s4-freeze/frozen-shas.txt`，与旧 M3 launcher 不同属预期）。新 target 由 `0c9497e4` 源码
  16:49:31 重建（`--all-features`），无既有签名副本可比。
- **runner**：`run-validation-attempt1.ps1` 33221 B / sha256 `c7a387a4…`，与 signature 记录一致；窗后未编辑；
  `feature_profile=all-features`、`env_mujoco_dll`/`env_cuda_bin` 在 facts 中。
- `provenance.json` sha `3ea5c863…` 未变（133 条守卫内含它）；被守卫输入未被窗口提交改动（交集 0）。

## 5. 10 实体与新 target 数值

- 10 个实体 `exit=0`（facts `exit_codes` 10 项全 0，`failed_entities=0`；runner stdout 10 行 `ENTITY_DONE ... exit=0`；
  10 份实体日志均 `test result: ok … 0 failed`）。
- 新 target（`logs/attempt1-g10_p5_product_collision_wiring_cuda.log`）：`mesh-box`/`mesh-cylinder` 对 M1（`c83baffe…`）、
  `mesh-mesh` 对 M3（`96e7895d…`）——各 `nacon=4 = ref_count`、`bit_equal=4/4`、`max_dist/pos/frame ULP=0`、
  `contact.geom=[1,0]/[1,0]/[0,1]`、`failures=0`；`e2e_summary cases=3 bit_equal=12/12 failures=0`；
  S1 快照 `s1_fields=32/32 × 3`、`mismatches=0`；`test result: ok. 2 passed; 0 failed`（40.83 s）。
- 9 回归：multicontact（314/314、391/392 pos ULP=1、80/80）、primitive（3×80/80）、plane（6 组合）、
  sphere_capsule（4 case）、branches（6 分支 + 4 swap）、m2（4/4+4/4）、m3（4/4）、launcher（314/314、3122/3122、
  80/80、hfield-capsule 275/275）、stage2（record-only）——日志均 `0 failed`。
- stage2 只记录：日志 17 行 dump，每行「两次一致 true」→ **17/17**（仅记录，不作通过结论）。

## 6. 零编译、范围、证据一致性（Chief 追加点 C）

- 零 Rust 编译：`deps_exe_before=after=1812`、`deps_max_write` 前后同为 `2026-10-27T16:50:14`、
  precheck `cargo/rustc/cl/nvcc/link absent`、heavy processes 0、`zero_rust_compile=True`；
  compute-apps 前后均 29 raw / **0 filtered**。（窗内 NVRTC 为进程内运行时编译，与「零 Rust 编译」不冲突。）
- 范围：`45e07485` 22 文件（21 证据 + 窗口任务 JSON）、`fa704a6e` 4 证据文件；两者合计 **0 个** `src/ tests/ native/
  include/ fixtures/ Cargo.*` 路径；`git diff 0c9497e4..HEAD -- <代码路径>` = **0 文件**；被守卫输入与窗口提交交集 0。
- 证据一致性：交付记录 22 个文件 sha/bytes **0 不符**（lines 见 O1 的 runner-stdout 例外）；顶层 receipt
  `task_id`（S1）、`s2.task_id`、`s3.task_id`、`s3.window.attempt1.task_id` 全部正确未被覆盖；
  `window-receipt.json /task_id` = 窗口任务；日志 UTF-8 无 BOM **16/17**（1 个 UTF-16，见 O1）；
  `window_pass_claimed=false`；「窗口通过」仅出现在否定句；6 条禁词 0 命中。
- 补正 `fa704a6e`：只改 4 个证据文件的「预算偏差」中文标签/否定句字面，**未删除任何被引用数字**
  （4142/1051/3091/11920/9904/12976/867 等）；trailer `Normify-Fixes: …-s3-window` 正确。
- O1 记录：窗口 receipt `dev_self_test_O1.note` 与 README §10.5 均写明此前约 42 s GPU 自测是「非验证窗」，
  未被当作窗口结论。

## 7. 观察（每条含「修/接受」建议）

### O1（一般）｜`logs/attempt1-runner-stdout.log` 不是 UTF-8/LF 忠实副本

该文件实际是 **UTF-16LE 带 BOM（FF FE）**，且**每行后多一个空行**（原始 183 行 → 入库 366 行；交付记录写 367 行），
与 README §10.4 和交付记录 `evidence_format.line_endings` 的「全部文本证据 UTF-8 无 BOM、LF/CR=0」声明不符。
内容可解码阅读、数值不受影响（数值权威源是 facts/samples/guards），但作为「原始 transcript 的归一化副本」不合格，
且行数声明与解码结果不一致。**建议修**：重新导出为 UTF-8 无 BOM、LF、不插空行的副本，更新交付记录 sha/bytes/lines
与 evidence_format 声明（或明确记为编码例外）。**可接受**：若坚持保留 UTF-16 原样，则必须改声明并在 receipt 记例外
（当前声明与实际不符，属于必须更正的记录项）。

### O2（一般）｜尖峰时间「launcher 退出后约 0.7 s」与产物不符

依据见 §3：launcher 日志写入 17:03:36.808（写日志发生在进程退出之后），尖峰样本在其前 86 ms；
stage2 最早启动下界 17:03:37.658。**建议修**：把 README §10.2、`window-receipt.json`
`budget_deviation.attribution.basis`、`receipt.json` 同段改为「位于 launcher 退出/日志收尾边界（约 ±0.1 s）内、
早于 stage2 启动约 ≥0.9 s」，归因继续标「归因推断」。**可接受**：峰值与归因方向不变，仅时间措辞需更正。

### O3（建议）｜邻居采样写错一格

`window-receipt.json` `sampler.observation_OW1.neighbor_samples_used_mib=[3092,3091]`、
`budget_deviation.peak.neighbors_used_mib=[3092,4142,3091]`；实测 seq 773–778 为
`3091,3092,3092,4142,3092,3091`，即峰前后都是 **3092**（3091 是峰后第 2 个样本）。README §10.2 写的
「3092 → 4142 → 3092」是对的。**建议修**：两处改为 `[3092,3092]` / `[3092,4142,3092]`；**可接受**：不影响峰值。

### O4（建议）｜「单样本 0.25 s」是标称间隔

实测采样间隔中位 **0.2759 s**（min 0.2721 / max 0.2901，含 nvidia-smi 查询耗时）。**建议修**：写成
「0.25 s 目标间隔（实测中位 ≈0.28 s）」；**可接受**：作为标称采样周期表述。

### O5（建议）｜机器可读例外清单不完整

交付记录 `machine_readable_exceptions` 列了 samples.csv/sha-guards.txt/facts.txt/window-receipt.json，
但 **`receipt.json` 719 行 >500 未列**；另外 O1 的 UTF-16 副本也未标例外。**建议修**：补列 receipt.json，
并把 runner-stdout 的编码例外一并记录；**可接受**：JSON 本身即机器可读，仅格式声明问题。

### O6（建议）｜`runner_outer_wall_s=252.1` 缺产物支撑

`window-receipt.json` 的 `runner_outer_wall_s=252.1` 在 facts/runner stdout 均无对应键（只有 window_start/end 与
实体 wall）；QA 可复核的是 body_wall 238.5 s（start/end 差 = 238.480）。**建议修**：在 facts 落 runner 起止时间，
或在记录里标注该字段为派生值；**可接受**：不影响任何窗口事实。

### O7（建议）｜预算偏差命中计数口径

`预算偏差` 在 window-receipt/receipt/delivery 各 **2 处**、README **1 处**（合计 7 处），`在预算内` **0 处**；
若按「单文件 ≥3」理解 Arch 的口径则不成立。**建议修**：在记录里写明统计口径（全文合计 ≥3 已成立）；
**可接受**：语义无歧义，纯计数口径说明。

## 8. 证据完整性

- 独立复算脚本与原始输出（本目录）：`verify_samples.py`（24 项）、`verify_guards.py`（29 项）、
  `verify_facts.py`（43 项）、`verify_evidence.py`（18 项 + 已知 findings）、`spike_localization.py`、
  `encoding_scan.py` 与对应 `.json`/`logs/`。
- 复核目录内文件：UTF-8 无 BOM、LF；README ≤500 行；机器可读 JSON 在 receipt 中标注例外。
- 我的复核目录与窗口证据目录分离；未改动任何窗口证据、代码、判据、参考、容差。
- 窗口原始产物（`target/s3-product-wiring/attempt1/`、`window/`、`*freeze/`）只读引用，未改动。

## 9. 复现命令（QA 侧，只读）

```
target/render-bvh-precheck/venv312/Scripts/python.exe verify_samples.py
target/render-bvh-precheck/venv312/Scripts/python.exe verify_guards.py
target/render-bvh-precheck/venv312/Scripts/python.exe verify_facts.py
target/render-bvh-precheck/venv312/Scripts/python.exe verify_evidence.py
target/render-bvh-precheck/venv312/Scripts/python.exe spike_localization.py
target/render-bvh-precheck/venv312/Scripts/python.exe encoding_scan.py
```

## 10. 边界与不宣称

本复核只读：未重开窗口、未开 GPU、未重跑任何实体、未改代码/判据/参考/容差。窗口数值只在开发机 attempt1 产物
与本机重算之间核对；本复核**不宣称**窗口结论、任何覆盖状态、精度目标、host↔device 一致、上游或双端等价。
`runner_outer_wall_s`、NVRTC 编译耗时拆分等派生字段仅按记录引用（见 O6）。
