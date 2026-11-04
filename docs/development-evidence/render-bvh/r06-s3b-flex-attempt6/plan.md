# attempt 6 窗口计划（R06-S3b flex 渲染验证，签名 allowlist 已进断言）

任务：`2026-11-04-render-r06-s3b-flex-validate-attempt6`（owner Arch，唯一执行会话 **arch-chat**）。
性质：开窗前计划；窗口开启需 **Chief 单独 GO**（前置见 §6）。本文件入库后，窗口以现场 HEAD 为准。

## 1. 前置状态

| 项 | 值 |
| --- | --- |
| allowlist 断言实现 | r2 卡 `2026-11-04-render-r06-s3b-flex-allowlist-assert-fix-r2` **verified**，提交 `8c15ad75`（主用例 495 行 + 支持模块 156 行，unit 1320/0） |
| 判据口径 | 除签名 allowlist 外全部 bit-first；allowlist 仅 `cam2/(x=134,y=128)/flexid0` 允许 `ULP≤4 且 beyond4=0`，`beyond4≠0` 即失败 |
| D8/f32 修复 | `2026-11-04-render-r06-s3b-flex-f32-order-fix-r2` verified，`d35cab68`（`scene.cu` 显式 fma 顺序） |
| attempt5 | `6e9aa585` **FAILED_ROUND**：用例断言严于签名 allowlist（观测 `(1,0)` = 允许范围），非数值红 |
| 当前 guards | `attempt=1 status=frozen`、`16/16 exes=2 inputs=14`、`aggregate_inputs=ec6fa222…`、`aggregate_all=de4a4279…`、文件 sha `70d4cbc6…` |
| 参考夹具 | `fixtures/render-bvh/scene-flex/reference/base/` 6 文件（含 `cam-poses.json` `9b6eabf2…`），**本窗不得改动** |

## 2. 窗口内 canonical rebuild（只做一次）

```
cargo test --locked --features cuda-probe --no-run --test render_flex_validate
```
- 命令只跑**一次**；跑完确认 build 进程 0。
- **立刻**（同一时刻）记录：exe 绝对路径、`sha256`、`bytes`、`LastWriteTime`、构建命令与输出尾部；写入 `canonical-rebuild.txt`。
- 若同刻快照与后续任何时刻不一致，以**同刻快照**为准并在证据注明「其它时刻不可复现」。
- 冻结后**零编译/零 relink**；冻结后任何编译 ⇒ 本窗 **INVALID_ROUND**（记录不采用）。

## 3. 重新真冻结（`-Attempt 2`）

```
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate/render-bvh/freeze-guards-r06-s3b-flex.ps1 -Attempt 2
```
- **必须用 `-Attempt 2`**：guards 现为 `attempt=1 status=frozen`，`-Attempt 1` 会因 `exe entry already frozen` 失败。
- 期望：`GUARDS_OK=16/16 exes=2 inputs=14`、`attempt=2 status=frozen`、2 exe 不再 pending。
- 记录 `aggregate_inputs_sha256` / `aggregate_sha256` **before（§1 值）→ after（冻结输出）**，并与独立复算对照（Ordinal 路径序，不用 `Sort-Object` culture 序）。
- 等价核验：live exe sha/bytes == guards == stash（2/2）；14 条输入 live sha/bytes 全等；`aggregate_inputs` 复算一致。

## 4. 开窗基线与 monitor

- 三点基线（5 分钟内，间隔 2.5 s）：`nvidia-smi --query-gpu=memory.free,utilization.gpu,temperature.gpu --format=csv,noheader,nounits`；`--query-compute-apps` 无 CUDA 计算任务；build 0；树 clean。
- `floor = max(11,688, min(free) − 3,072)`，逐点 free ≥ floor 且 util < 40；命令与输出贴群、落 `pre-window.csv`。
- `monitor-until-clean.ps1 -Required 60 -MaxMinutes 30 -OutCsv <证据目录>/monitor-official.csv`（必须显式 `-OutCsv`，避免默认目录缺失；D6 已修但留档用显式路径）。`util 2–7` 量级、`free_min ≥ floor` 才继续。

## 5. 实体（单次、唯一 ignored 用例）

```
cargo test --locked --features cuda-probe --test render_flex_validate -- --ignored --test-threads=1 --nocapture
```
- PATH 前置 `target/toolchains/cuda-12.8.1/bin`（否则 `NvrtcUnavailable`）。
- 日志必须出现 `running 1 test`；出现 `0 tests`/`filtered` 假绿 ⇒ **INVALID_ROUND**。
- 判定：rgb bit-first（`mismatched=0 / max_byte_diff=0 / beyond_1_lsb=0`）；depth bit-first，**唯一例外** `cam2/(134,128)/flexid0` 允许 `ULP≤4 且 beyond4=0`（`beyond4≠0` 即失败），其余像素 `mismatched=0 / beyond=0`；seg 精确 0 差异；(flexid,9) 参考 654 像素覆盖；两跑 sha256 相等。
- 结果按相机/实体逐项列 `(mismatched, beyond4, max_ulp, bits)`；allowlist 像素明细单列（实际 bits、ULP、beyond4、seg）。

## 6. 前置与边界

- 前置（缺一不开）：QA allowlist 复核 PASS；本材料入库并核过；canonical rebuild 同刻指纹已记；真冻结 16/16/attempt=2 + aggregate before/after；三点基线逐点 ≥ 11,688；**Chief attempt6 token**。
- 单一执行者 **arch-chat**；Backend/QA 只读，不占 GPU、不写仓。
- 失败即停：不重试、不放宽、不改判据、不做二次微跑、不重冻到过；原文归档 `attempts/attempt-6-*` 并群报。
- 非声明：不宣称上游等价/parity pass/flex 通过/R06-S3b 整体通过；allowlist 仅限本次 attempt6。

## 7. 证据落点

`docs/development-evidence/render-bvh/r06-s3b-flex-attempt6/`：`plan.md`（本文件）、`canonical-rebuild.txt`、`pre-window.csv`、`monitor-official.csv`、`freeze-attempt2.log`、`guards-after.json`、`entity-run.log`、`allowlist-pixel.json`、`two-run-sha.txt`、`README.md`（结论与边界）、`manifest.txt`（`#` 表头、LF、无 BOM）。
