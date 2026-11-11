# QA 复核 W2/U7U8 全条件断言窗 attempt 2 PASS（r3 冻结）+ 主卡收口（`4e3abe9c`）

- 复核对象：主卡 `2026-11-12-w2-u7u8-full-condition-assert`（**verified**，收口 **`4e3abe9c`**）；
  提交链 `c36458b6`（登记 in_progress）→ `1166e627`（§1 收口 3 低 + 5 观察）→ `0a99f72e`（limits 升 pass/fail + r2 重冻）
  → `75e296fb`（pair 归一化修正 + CPU 断言）→ `1bd85287`（r3 窗 PASS 证据）→ `4e3abe9c`（卡收口 + r2 目录收据镜像）。
- 窗口：**attempt 2（r3 冻结）最小窗 1 实体**；窗口基线 HEAD `2ccc182b`；现 HEAD `4e3abe9c`。
- 性质：只读复核（不重开窗、不跑 GPU 窗口；仅本地重跑 CPU 断言复核）。零窗复核。
- 结论：**复核通过（0 阻断，0 严重、0 一般 + 1 低 + 3 观察）**。
- 窗口结论：**在该窗口径（r3 冻结、冻结参考、comparator 口径）下 U7/U8 pass_rule 全条件成立**
  （`failures=0`、两 case `bit_equal` 全行、`contact.geom=[1,0]`、`pos_order_equal=true`）——我按原件逐字段独立复算成立。

## 1. r3 窗口执行（独立复算）

- **1/1 实体 exit 0 ✅**：`results.json` `exit_code=0`、`elapsed_s=41.25`（≤300 s）；`facts.txt` 单条 `RESULT ... exit=0`；
  实体日志 `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 40.99s`；
  `.err` **0 B**；`SUMMARY entities=1 failed=0 exit_codes=0`（r2 的 exit 捕获/计数两 bug 已修复并自测）。
- **sha 守卫 18/18 ✅**：`sha-guards.txt` 实为 **18 条 = 1 exe + 15 fixture + 2 参考**，我逐条重算磁盘哈希 **0 不符 0 缺失**；
  `guards-r3.tsv` 为同源 18 条（表头+18 行）。r3 branch exe 在 `target/a2-freeze-r3/` 实测
  **41,782,272 B / `ac823b58…`**，与 receipt/refreeze 一致。
- **floor ✅**：`FLOOR gate: free=13440 MiB >= 10240 MiB`；`POST free=13440`；`GPU pre/post :: 2 %, 2622 MiB, 13440 MiB, 25, 596.36`。
- **monitor 60/60 ✅**：`monitor-pre-window-r3.csv` 实为 60 行（03:12:50–03:17:51、5 s 间隔），`streak=1..60`、`clean` 全 True、
  `util_max=4`、`free_min=13440`、`compute_filtered`/`build` 全 0——与 receipt `monitor` 字段一致。
- **三点基线 ✅**：`logs/three-point-baseline-r3.txt` 为 Chief 提供的 raw（P1/P2/P3：util 2/4/2 %、used 2,614 MiB、
  free 13,448 MiB、25°C、596.36）；时间戳 03:17:59–03:18:02 在 monitor 结束（03:17:51）之后、窗口采样（03:18:36）之前，顺序自洽。
- **窗内 0.25 s 采样（A2-3 L2 收口）✅**：`window-samples.csv` 实为 **138 行**，`interval_ms` 排除首行初始化 0 值后
  **min 290.6 / max 361.3 ms（137/137 全部在区间内）**、`util_percent max 4`、`memory_free_mib min 11,793`，
  与 receipt `window_sampling` 逐数一致；样本时间 03:18:36–03:19:17，窗口实际执行。
- **零 Rust 编译 ✅**：`PRECHECK ... no cargo/rustc/cl/nvcc process` + `POST zero-rust-compile: PASS`；
  我另扫 `target/` 在窗口时段（03:18:30–03:20:00）的 7 个改动文件全部是 `target/a2-window-r3/` 下 runner 输出，
  **无 `.rlib/.rmeta/.pdb/.obj/.d/.lib/.exe/.dll` 构建产物**；r3 branch exe mtime **03:07:57**（早于窗口），
  其余 9 个 exe mtime 01:47（r2 复制，未重建）。**POST 过滤后残留 compute 0**。
- **无失败轮/无重试/无放宽 ✅**：facts 仅 1 条 RESULT、无 fail-stop 行；`fail_stops=[]`；attempt=2 明确。

## 2. U7/U8 全条件（按原始输出与 comparator 逐字段核）

- **原始日志机证 ✅**：`logs/g10_p5_branch_closure_cuda.log` 中
  `case=mesh-limits-cone-box nacon=1 ref_count=1 bit_equal=1/1 max_dist_ulp=0 max_pos_ulp=0 max_frame_ulp=0 contact.geom=[1,0] failures=0`
  与 `case=mesh-limits-prism-box nacon=4 ref_count=4 bit_equal=4/4 ... contact.geom=[1,0] failures=0`；
  `u7-u8-full.json` 的 `comparator` 块与 `_meta.both_cases_observed=true`、`fail_lines=[]` 一致。
- **数值独立复算 ✅**：U7 device/ref dist 均 `-1.0000002e-2`（0 ULP）、pos 三元逐轴相同；U8 dist 4×`-9.999998e-3`（0 ULP）、
  pos 四行与参考**逐行相同**（`pos_order_equal=true`，不再轮换）；`max_*_ulp=0` ≤ 判据 4 ULP。
  r2 的 `geom=[0,1]`、frame 符号翻转、pos 行序轮换在本窗全部消失。
- **fail-stop 6 条口径 ✅**：任一 comparator failures≠0 即停；本窗 0 失败，未触发。
- **限制如实 ✅**：receipt `limitations` 明写 `efc_address/overflow` 参考未捕获（只做设备侧不变量，与 M2/M3 同口径）、
  `flex/elem/vert` 不在 pass_rule 11 字段、本窗仅 branch 实体（其余 9 exe 未重跑）；README §6 同口径。

## 3. 修正范围与冻结一致性

- **修正仅 tests ✅**：`75e296fb` 只改 `tests/g10_p5_branch_closure_cuda.rs` 与 `tests/support/branch_closure_case.rs`；
  `pair_meta()` 实现与上游 `_add_geom_pair` 同构（type1>type2 交换、相等保序），`build_meta` 使用归一化 pair；
  新增 CPU 断言 `pair_normalization_matches_upstream_reference`（11 case 对参考 `contact.geom` 逐行断言 + U7/U8 `(1,0,6,7)`）。
  **我本地重跑** `cargo test --locked --features cuda-probe --test g10_p5_branch_closure_cuda` → **4 passed / 2 ignored**
  （含 `pair_normalization_matches_upstream_reference ... ok`）。
- **product src 零改动 ✅**：`git diff ef8da38f..HEAD -- src` 为空、`.cu` 零改动；`src` digest 我按 addenda 口径重算 =
  **`266bbf94…cb44`**（与 receipt/r2/r3/refreeze 四处一致）；comparator `mesh_multicontact_compare_support.rs` **495 行且未被改动**。
- **只重建 branch exe ✅**：r3 refreeze 10 行与磁盘全等；branch `ac823b58…`/41,782,272（旧 r2 branch `6d2b39fc…`/41,771,008），
  其余 9 行与 r2 清单 sha/bytes **9/9 全等**；`counts={"rows":10,"rebuilt":1,"copied_unchanged":9,"old_freeze_match":9,"copy_match_total":10}`。
- **证据 manifest ✅**：r3 **17/17 逐条重算 0 错**、不列自身、磁盘 18 文件全覆盖。

## 4. r2 FAIL 证据（attempt 1 失败记录，append-only）

- **verdict=FAILED / attempt=1 ✅**：`r2-fail/receipt.json` `verdict=FAILED`、`attempt=1`、`failures_total=21`
  （`by_case`：U7 **4** + U8 **17**，逐条明细在 `u7-u8-failures.json`）；`failure-record.md` 逐条列出 frame 符号/geom 对序/pos 轮换；
  `.err` panic 行 `U7/U8 pass_rule 全条件与 S3 上游 GPU 参考不一致（21 条）` 在位。
- **bug 如实 ✅**：`entity_exit_code=null` + `exit_code_capture_bug`（`-PassThru` 无 `-Wait` 读不到 ExitCode；`cmd /c exit 3` 复现）+
  `summary_count_bug`（`SUMMARY entities=1 failed=4` 实为 1，OrderedDictionary 键数）——两项都以「工具缺陷」记录，未改判据。
- **失败窗其余事实 ✅**：18/18 守卫（r2 branch `6d2b39fc…`）逐条重算 0 不符；134 样本（288.7–361.2 ms、util_max 42、free_min 13439）；
  monitor attempt-1 CSV 142 行、util_max 94、streak_last 37（即当时未开窗那轮）；floor 10240（free 13440）。
- **append-only ✅**：r2-fail 目录 13 文件与 `63b95170` 的 blob **逐个相同**（0 改写）；r2-fail manifest 12/12 重算 0 错。

## 5. 主卡收口检查与镜像裁断

- **卡检查逐条解析 ✅**：主卡 18 项检查 = 4 gate + `unchanged src/,fixtures/,scripts/,cap32 A2-1 证据目录/` + **13 条 json**；
  我按路径逐条解析实际文件，`/src_digest`、`/verdict`、`/u7_failures`、`/u8_failures`、`/u7_pass`、`/u8_pass`、
  `/sha_guards_ok`、`/zero_rust_compile`、`/task_id`、`/freeze_dir`、`/counts/old_freeze_match`、`/source_tree_digest/combined`
  与 `A2-2 卡 /status` **全部满足（0 失败）**；其中窗口字段经 `...-window-r2/receipt.json` 镜像读取，
  我核镜像 11 个字段与 `...-window-r3/receipt.json` **逐字段相等**（verdict/failures/u7/u8 counts/sha_guards/zero_compile/src_digest/exe sha/bytes）。
- **门禁/unit ✅**：主卡 4 run 全 passed；unit **1410 passed / 0 failed / 402 ignored**（新增 CPU 断言 +1）；gate run at 03:21:24–43
  晚于 r3 证据提交 03:21:03、早于卡收口 03:21:59，顺序自洽。
- **镜像裁断（Chief 问题 4）**：**镜像可接受，不需 abandon 主卡另开收口卡**。理由：
  ① 镜像目录 `...-window-r2/` 的 `README.md` 与 `receipt.json` 都明写「**不是对 r2 exe（attempt 1）的 PASS 声明**」，
  且 receipt 顶部 `attempt1.verdict=FAILED` + `attempt1_fail_record` 指针 + `full_evidence_dir`→r3 醒目；
  ② README「目录历史」4 条把 r2 就绪→attempt1 FAILED→修正→attempt2 PASS 串起来；
  ③ 镜像值与 r3 权威 receipt 逐字段相等（我复算）。
  **残余风险与建议（观察）**：目录名含 `r2` 而 `verdict=PASS` 属 attempt 2，纯机器读者可能误读 → 建议给镜像 receipt 增
  机器可读 `verdict_scope`（如 `"attempt 2 (r3 freeze); attempt 1 (r2) = FAILED"`）与 `mirrors_sha256`（指向 r3 receipt 的 sha256），
  未来窗口证据目录用轮次无关命名（如 `-window-min/`）；这些属改进项，不影响本次判定。
- **§1 收口复核 ✅**：L1 生成 `logs/window-runner-console.utf8.log`（UTF-8/LF/无 BOM，43 行）+ `console-vs-facts.diff.log`，
  **原损坏 console 字节未改**（blob 与 `bd231ace` 相同）；L2 在 r2/r3 runner 实装窗内 0.25 s 采样并披露 A2-3 未落地；
  L3 identity addenda 末尾 append 精确表述（`new` 以 `old` 字节为前缀）；O1–O5 逐条处置。A2-3 window 目录 19 条旧记录 sha
  **全部未变**、manifest **24/24 重算 0 错**、新文件均 UTF-8 无 NUL/CR/BOM；identity manifest **12/12** 重算 0 错。

## 6. 非声明（全称扫描）

- 我扫 r2-fail / r3 / r2 三目录 **38 文件** `policy_scan` **0 命中**；正禁语（整族通过/缺口已清/NP01/NP02 通过/与上游等价等）
  **仅出现在「不宣称/不作/不提供」否定句**（含跨行续句，已逐条看上下文）。
- 本窗 **只**支持「该窗口径下 U7/U8 pass_rule 全条件成立（failures=0、bit-exact）」；**不宣称** mesh 容量缺口已清、
  与上游等价（over-32）、G10@P5 整族通过、NP01/NP02 门禁通过、性能结论；**不作位级等价声明、不提供位级等价保证**。

## 7. 证据文件

`README.md`、`receipt.json`、`logs/independent-check.txt`、`manifest.txt`。
