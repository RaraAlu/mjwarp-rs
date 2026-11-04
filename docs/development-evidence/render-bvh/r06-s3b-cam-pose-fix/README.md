# R06-S3b D5 修复：参考 data 相机位姿 provenance + 用例取数修正（CPU 位级断言）

任务：`2026-11-04-render-r06-s3b-cam-pose-source-fix`（owner Backend，A 类修法）。
触发：attempt 3 的 `cam1 rgb max_byte_diff=255` 经诊断（`docs/development-evidence/render-bvh/r06-s3b-cam1-diagnosis/`）
判定为 **harness 相机位姿来源缺陷（D5）**：用例从 model 的 `cam_pos0/cam_mat0`（mode-3 TARGETBODY 下为单位阵）派生位姿，
而参考采集/渲染用 **data** 位姿（`d.cam_xpos/d.cam_xmat`，look-at 乘积）；cam-track/cam-flex 朝向差 85.517°/75.562°。
本片只修 harness 输入 + 补 provenance：**参考 npz 零改动、判据/容差零改动、`src/` 零 diff**。

## 1. 位姿补录运行事实（Chief GO，forward-only 短 CUDA）

同一 `MJWARP_CAPTURE_AT=2026-11-04T11:58:13+08:00`、同一 pinned 环境两次 forward-only 运行：

| 运行 | 命令 | 时长 | GPU 前 → 后 | 结果 |
| --- | --- | --- | --- | --- |
| run1（产出） | `…venv/Scripts/python.exe -B scripts/authoring/capture_render_flex.py --cam-poses-only` | **1.74 s** | free 14,313 MiB / 23% / 27 °C → 14,312 MiB / 2% / 27 °C | cam-poses.json 产出 |
| run2（脚本行压缩后复跑校验） | 同上 | **2.20 s** | free 14,306 MiB / 4% / 27 °C → 14,306 MiB / 1% / 28 °C | 6 state 位级相等、5 文件零变化；**`state_bitwise_equal`/sha 复验通过** |

两次均：无渲染、不产参考图；进程退出即释放 GPU（总 GPU 时间约 4 s，远低于 2 分钟上限）。run2 原因：首次产出后为过
500 行棘轮对脚本做**纯行压缩**（无数据路径改动），复跑使 provenance 与入库脚本版本一致。

| 项 | 值 |
| --- | --- |
| 输出 | `fixtures/render-bvh/scene-flex/reference/base/cam-poses.json`（3,987 B，sha256 `9b6eabf233c8cdcb8dd4fc086d994b4fce8d65169979a7978c1a5b8b6d8b425e`） |
| 内容 | 3 台相机 data 位姿 **f32 位型**（`cam_xpos_bits`/`cam_xmat_bits`）+ 十进制值 + 相机名/mode/targetbodyid + 环境（pinned 3.15 editable、CUDA 12.9/driver 596.36、GPU 4070 Ti SUPER） |
| 同帧自证 | 同一次 forward 重算 6 个 state 数组与 `state.npz` **逐位相等**（`qpos/geom_xpos/geom_xmat/light_xpos/light_xdir/flexvert_xpos` 全 true） |
| 零字节保护 | 两次运行前后既有 5 文件 sha256 **完全一致**：`rgb.npz 5ed50287…`、`depth.npz c3ee4adc…`、`seg.npz f3b00fa8…`、`state.npz 7f2fff50…`、`capture.json 4713e2c7…` |
| 一致性 | 位姿矩阵与诊断证据同源：cam1 xmat max abs Δ=1.693267、cam2=0.968418，朝向差 85.517°/75.562°（CPU 用例逐位消费后打印） |

运行前/后对照命令：`nvidia-smi --query-gpu=memory.free,utilization.gpu,temperature.gpu`；5 文件 `sha256` 由脚本在运行前后各自计算并断言相等（`existing_files_unchanged=true`）。

## 2. 脚本改动（`scripts/authoring/capture_render_flex.py`，additive；497→491 行）

- 新增 `--cam-poses-only` 模式（走现有 main 流程到 `mjw.forward` 后分支）：只做 `put_data → mjw.forward → synchronize`，
  **不建 render context、不渲染、不写 npz**；写 `cam-poses.json` 前后断言既有 5 文件零字节变化，否则 fail。
- 新增 `flex_cam_pose_record()`（data 位姿 f32 位型 + 十进制）、`state_bitwise_equal()`（6 数组与 `state.npz` 逐位比对）、
  `write_cam_poses()`（provenance 文件 + 相机名/mode/targetbodyid + 环境 + 命令 + captured_at）。
- 全量 capture 路径同样在 `mjw.forward` 后（**任何 render 调用之前**）落 provenance，并在写完 npz 后附
  `state_matches_saved_npz` 自证；`--repeat 2` 的既有字节一致性检查不变。
- 兼容性：默认 `--group base` 行为不变；`--dry-run` 不触发 CUDA（已跑通，CPU preflight `report_ok/state_ok`）；
  既有输出文件格式/字段不变（行压缩为纯格式改动，JSON 由 `json.dumps` 生成不受缩进行数影响）。

## 3. 用例改动（`tests/render_flex_validate.rs`，499→499 行）

- `camera_poses()` 改读 `cam-poses.json` 的 **f32 位型**（`f32::from_bits`），不再读 `model cam_pos0/cam_mat0`；
  两处调用点（CPU 前置用例、GPU 窗口用例）同步。
- **CPU 前置断言（新增硬要求，普通 `cargo test` 可跑）**：逐相机断言 `用例传入 == 参考位姿`，且
  「位型编码」与「十进制编码」两条独立路径逐项一致（`assert_eq!`，不是 ULP/近似）；形状 ncam/3·ncam/9·ncam 全部断言。
- 三方对照打印（诊断口径，`-第 3 列` f64 点积 acos）：

| cam | data xpos | model-static xpos | 朝向差 | xmat max abs Δ |
| --- | --- | --- | --- | --- |
| 0 cam-persp | `[1.2, -1.1, 0.8]` | 相同 | ≈0（0.025° 为 f32 点积舍入噪声） | 0.000000 |
| 1 cam-track | `[-0.8, 0.9, 0.7]` | 相同 | **85.517°** | 1.693267 |
| 2 cam-flex | `[-0.35, -1.15, 1.05]` | 相同 | **75.562°** | 0.968418 |

  后两行与诊断证据逐项一致 ⇒ 用例现在消费的正是参考渲染所用的 data 朝向。
- 500 行棘轮：新增断言与既有 `#[rustfmt::skip]` 字段列表的纯格式压缩相抵，文件仍为 499 行；`cargo fmt --check` 通过。

## 4. 验证

- `cargo test --locked --features cuda-probe --test render_flex_validate render_context_preflight_matches_capture_recipe -- --nocapture`：**1 passed / 0 failed**（打印上表；消费新 provenance 文件）。
- `--dry-run`（CPU-only，不碰 CUDA）：EXIT=0，环境/pin/preflight 全 ok。
- 四门禁：fmt / clippy / clippy-cuda / unit——见 `.normify/runs/2026-11-04-render-r06-s3b-cam-pose-source-fix/`。
- 零窗口/零参考图/零判据改动；`src/` 零 diff（卡面 `unchanged src/`）；attempt 4 仍需 Chief GO。

## 5. 边界与遗留

- 本片只修 harness 输入与 provenance；**不宣称 flex 渲染通过**，不评估 cam1/cam2 数值（attempt 4 窗口做）。
- guards 扩条目（inputs 13→14 / total 15→16、冻结器 `$inputPaths` 与 count 断言）归 r2 卡
  `2026-11-04-render-r06-s3b-flex-guards-pose-entry-r2`，真冻结仍在 attempt 4 窗口内现场执行。
- `capture.json` 未加相机清单（QA backlog 建议），本片按最小改法不动既有字段（零字节要求）；相机清单已写入
  `cam-poses.json.cameras`（names/cam_mode/cam_targetbodyid），QA 后续可直接引用。
