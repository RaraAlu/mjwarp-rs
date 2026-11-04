# attempt 4 窗口计划（R06-S3b④ flex 渲染验证，D5 修复后重验）

任务：`2026-11-04-render-r06-s3b-flex-validate-attempt4`（owner Arch，唯一执行会话 **arch-group**）。
材料性质：开窗前计划，本文件落库后 HEAD 即窗口基线；窗口开启仍需 **Chief 单独 GO**（四前提）。
本轮重验对象：attempt 3 的 VALID_ROUND 判据性红（cam1 rgb max_byte_diff=255）已被诊断与 errata 定性为 **harness D5 相机位姿来源缺陷污染**（`82307565` / `3d5f841b`），D5 与 guards 扩条目已收口，按既有窗口口径重验。

## 1. 前置状态（全部 satisfied）

| 项 | 值 |
| --- | --- |
| D5 修复卡 | `2026-11-04-render-r06-s3b-cam-pose-source-fix` **verified**，提交 `ab8da091` |
| guards 扩条目卡 r2 | `2026-11-04-render-r06-s3b-flex-guards-pose-entry-r2` **verified**，提交 `733037c5` |
| 旧 guards 扩条目卡 | `...-guards-pose-entry` **abandoned**（files 漏冻结器，已被 r2 取代） |
| attempt3 定性 | VALID_ROUND 判据性红，判据面被 D5 污染，**不作产品结论**；flex 数值与两跑 sha 从未被评估 |
| 参考目录 | `fixtures/render-bvh/scene-flex/reference/base/`，**6 文件**（新增 `cam-poses.json`） |
| 判据/容差 | 零放宽，沿用 R05/R02 render 家族口径 |

## 2. 参考夹具 sha256（窗口前基线，开窗后须复核）

| 文件 | bytes | sha256 |
| --- | --- | --- |
| `rgb.npz` | 2362306 | `5ed502872b869bb1f37659ec22816f9f1c04405c1666f3d366b28fd298678207` |
| `depth.npz` | 787918 | `c3ee4adca6cf9874f40ab87b7f72a16b17f26647e45758342895919b725d8deb` |
| `seg.npz` | 1575106 | `f3b00fa82df1e54d4c89309d820212d9808038e00ae25f25841f2b40a6277bef` |
| `state.npz` | 1748 | `7f2fff501bfa98f41fb8d4b689337e9e62e1ec54d98c576dc0a6e8b880b61769` |
| `capture.json` | 16166 | `4713e2c7e403d47e1a23aed064e8f565336287839bfd0f96fb9242a1b1f10289` |
| `cam-poses.json`（新增） | 3987 | `9b6eabf233c8cdcb8dd4fc086d994b4fce8d65169979a7978c1a5b8b6d8b425e` |

既有 5 文件与 attempt 3 参考**逐字节相同**（D5 补录时已自证运行前后 sha 零变化）；`cam-poses.json` 为 3 台相机 `cam_xpos/cam_xmat` 的 f32 位型 provenance，两次 forward-only 均与同帧 6 个 `state.npz` 数组逐位相等（1.74 s / 2.20 s）。
D5 后用例侧已改读参考 data 位姿，CPU 前置断言逐 f32 位型相等，并打印三方对照：cam1 朝向差 **85.517°**、cam2 **75.562°**（与诊断证据一致）。

## 3. 守卫期望（本窗核心数字）

- guards 文件：`scripts/validate/render-bvh/guards-r06-s3b-flex.json`（4,137 B，sha256 `42981f8d…a95da5`）
- 冻结器：`freeze-guards-r06-s3b-flex.ps1`（9,379 B，sha256 `056102ab…f180892`）
- `frozen_exes = 2`、`guard_inputs = **14**`、`counts.total = **16**`
- `aggregate_inputs_sha256 = ec6fa22239c492c620a6d7dba9191b1d7f07b041ff4c27976b2de2a8f6998e77`
- **现场 `-Attempt 1` 真冻结期望：`GUARDS_OK=16/16 exes=2 inputs=14`**（不是 15/15）；`-GuardsOnly` 期望 `GUARDS_STRUCTURE_OK … inputs=14`
- 等价独立核验：live exe sha == guards 2/2、输入 sha/bytes 14/14、stash 副本 2/2

## 4. 执行顺序（单执行者 arch-group，单 GPU 任务）

1. **开窗前卫生**：`normalize-evidence-eol.ps1 -DryRun` → 实跑；`check-evidence-bom.ps1`（基线 128 non-.ps1 WARN + 4 manifest 违规，新增即停）；`git status --porcelain --untracked-files=no` 空；build 进程 0。
2. **三点基线（5 分钟内）**：`nvidia-smi --query-gpu=memory.free,utilization.gpu,temperature.gpu` ×3 + `--query-compute-apps` 无 CUDA 计算任务；逐点 free ≥ floor 且 util < 40。
   `floor = max(11,688, min(free) − 3,072)`（11,688 为 attempt 3 生效值/Chief 留痕 11,618 之上取严；以本窗现场三点现算为准）。
   **逐点手核 free ≥ floor**；`run-window.ps1` 的 `-OpenFreeMiB` 默认 10,240 与 monitor 内硬编码的 10,240 都只是下界（比本窗 floor 松 1,448 MiB），**不得**以「runner 没 abort」当作 floor 未破；开窗时显式传 `-OpenFreeMiB 11688` 并逐样本人工核对。
3. **monitor**：`monitor-until-clean.ps1 -Required 60 -MaxMinutes 30`（默认 `MaxUtil=40`，不带 `-OpenUtilLimit`）；任一点 util ≥ 40 或 free < floor → 停。
4. **真冻结**：`freeze-guards-r06-s3b-flex.ps1 -Attempt 1` → 期望 16/16。
5. **guards 断言**：`-GuardsOnly` 静态自检（inputs=14）+ 等价独立核验。
6. **实体**（零放宽）：三台相机比对组（cam-flex / cam-persp / cam-track）+ `render_regression_suite`。
   - **真实执行命令（唯一合法入口，不得用相机名当过滤器）**：
     `cargo test --locked --features cuda-probe --test render_flex_validate -- --ignored --nocapture`
     三台相机是**同一个 ignored 用例** `flex_renders_match_captured_reference` 内部的比对组；窗口日志必须出现 `running 1 test`。若出现 0 用例匹配、或 `filtered out 13` 却返回 0 的“假绿”，一律视为**未执行**，停手留档、不报通过/不通过。
   - rgb bit-first：`mismatched=0 / max_byte_diff=0 / beyond_1_lsb=0`
   - depth bit-first：`mismatched=0 / max_ulp<=4 / beyond_0_ulp=0`
   - seg 精确：0 差异像素，含 `(flexid,9)`（cam-flex 参考侧 654 像素 = (0,9)403 + (1,9)251）
   - 两跑 sha256 相等；回归实体零漂移
7. **收尾**：EOL 规范化后提交证据（`attempts/attempt-4-{monitor csv,guards body,entity log,manifest}` + README），`normify gate` 四门禁 → `normify done` → 显式 pathspec 提交；释放写锁并群报。

## 5. 失败政策

任一判据红 / monitor 不达标 / 冻结不符 / guards 不符 → **停手、原文存档、报 Chief**；不重试、不重冻到过、不放宽阈值、不改参考。只有「放宽判据 / 改参考 / 对外发布声明」需用户签。

## 6. 时长与时段

monitor 5–10 分钟（硬顶 30）+ 冻结/自检 1–2 分钟 + 实体 60–120 s，整窗约 15–25 分钟。
时段纪律：12:00–13:50 空闲块内可执行；13:50 前未开窗则顺延到 18:00 后，不挤 14:00–18:00 峰值。

## 7. 非声明

不宣称 R06/NP01/NP02 验收通过；不放宽任何判据/容差；`nbuffer` 版本差异（B 类）不参与本窗判据。
