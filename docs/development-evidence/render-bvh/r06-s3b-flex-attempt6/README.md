# R06-S3b attempt6 窗口记录（VALID_ROUND：判据实现已含签名 allowlist，实体体 PASS）

任务：`2026-11-04-render-r06-s3b-flex-validate-attempt6`（owner Arch，唯一执行会话 arch-chat）。
窗口规则 v2 + 用户签名 allowlist（`3406f1ca`）+ plan.md（`0a176abf`）。**本文件只记录本次观测；不宣称上游等价 / parity pass / flex 渲染整体通过。**

## 1. 开窗前（三点基线，已贴群留痕）

| 点 | free MiB | util | temp |
| --- | --- | --- | --- |
| 1 | 14194 | 1% | 27°C |
| 2 | 14194 | 1% | 27°C |
| 3 | 14194 | 4% | 27°C |

- **floor = max(11,688, min_free−3,072 = 11,122) = 11,688 MiB**；逐点 free ≥ floor、util < 40 ✓；
- build 0、worktree 0 dirty、HEAD `1927d62e`、锁 FREE、`--query-compute-apps` 无我方 CUDA 计算任务。

## 2. 窗口内 canonical rebuild（同刻指纹）

1. 先按 plan 直接执行 `cargo test --locked --features cuda-probe --no-run --test render_flex_validate` → **`Finished … 0.16 s`（无重编，缓存命中）**，据此判定该产物是窗口前的构建实例（LastWriteTime 22:54:37），不能代表「窗口内重建」；
2. 为确保绑定**窗口内产物**，追加一次 `cargo clean -p mjwarp-rs`（23,735 文件 / 14.0 GiB）+ 同命令重建：**14.3 s**；
3. **同刻记录（23:03:33）**：
   - test exe：`target/debug/deps/render_flex_validate-30bd8d889931ab2f.exe`
   - **sha256 `e48c7268…`、bytes 107,854,859**、LastWriteTime `2026-11-04T23:03:33.9091015+08:00`
   - 产品 bin：`mjwarp-rs.exe` **sha256 `c5014d83…`、bytes 85,041,741**
   - 构建进程归零（build=0）。
- 原始输出：`canonical-rebuild.txt`。

## 3. 重新真冻结（`-Attempt 2`）

- `freeze-guards-r06-s3b-flex.ps1 -Attempt 2` → **`GUARDS_OK=16/16 exes=2 inputs=14`**，`attempt=2 status=frozen`；
- `aggregate_sha256`（全部条目）before → after：`de4a4279…`（attempt=1 frozen 记录）→ **`c5263d33…`**；`aggregate_inputs_sha256` **`ec6fa222…` 不变**（14 条输入内容未变）；
- `guards_sha256 = c72caea0…`、文件 7,508 B；快照 `guards-after.json`；
- 等价核验：exe **live==guard==stash 2/2**（`mjwarp-rs` `c5014d83…`、`render_flex_validate` `e48c7268…`）、`guard_inputs` **14/14** sha+bytes 全等；
- **冻结后零编译**：monitor 与实体体期间 cargo 均无重编（`build_after_monitor=0`、`build_after_entity=0`），未发生 INVALID_ROUND。

## 4. monitor（通过）

`monitor-until-clean.ps1 -Required 60 -MaxMinutes 30 -OutCsv <evidence>/monitor-official.csv`：
`samples=60 longest_clean=60 ready=True util_min=2 util_max=7 free_min=14414` → 60/60 连续干净，free_min 14,414 ≥ floor 11,688。

## 5. 实体体（单次，PASS）

命令（PATH 前置 `target/toolchains/cuda-12.8.1/bin`）：
`cargo test --locked --features cuda-probe --test render_flex_validate -- --ignored --test-threads=1 --nocapture`

- 日志含 **`running 1 test`**，结果 **`test result: ok. 1 passed; 0 failed; 0 ignored`（1.41 s）**（`17 filtered out` 为本文件的非 ignored 用例，属正常过滤，不是假绿）；
- D5 位姿自证（通过）：cam0 `0.025°`、cam1 `85.517°`、cam2 `75.562°`（用例已改读参考 data 位姿）；
- **两跑 sha256 相同**（用例打印）：`rgb=d84ecaed…`、`depth=98292153…`、`seg=c4be9853…`（`two-run-sha.txt`）。

## 6. 判定

- **PASS**：用例断言已按签名 allowlist 实现（`allowlist` 仅 `cam2/(134,128)/flexid0` 允许 `ULP ≤ 4` 且 `beyond4 = 0`，其余全部 bit-first；rgb/seg/两跑 sha 均在断言内），实体体通过。
- **采集完整性（相对 plan §5 的差距，如实记录）**：用例通过时**不打印**逐相机 `(mismatched, beyond4, max_ulp)` 与 allowlist 像素的 `bits/expected/ULP/beyond4/seg` 明细；本轮为单次窗口（失败即停/不重跑，且冻结后禁止编译，故无法在不违反纪律的前提下补打印）。因此：
  - 已有：PASS 结论、两跑 sha、D5 位姿自证、冻结指纹、monitor 全量 CSV；
  - **缺**：逐相机/allowlist 像素的数值明细（`allowlist-pixel.json` 本轮**未生成**）。
  - 若需明细，须在**下一轮**先给用例加「只在通过时打印统计」的只读输出（代码改动 → 重建 → 重冻 → 新 GO），本轮不做。
- **不重试、不重冻、不放宽**；本结论不宣称上游等价/parity pass/flex 渲染整体通过。

## 7. 证据清单

`canonical-rebuild.txt`（含 no-op 与 forced rebuild 两段 + 指纹）、`freeze-attempt2.log`、`guards-after.json`、`monitor-official.csv`、`entity-run.log`、`two-run-sha.txt`、本 README、`manifest.txt`。
