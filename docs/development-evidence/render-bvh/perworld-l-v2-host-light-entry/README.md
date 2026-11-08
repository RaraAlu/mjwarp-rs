# perworld L v2 host 逐 world 灯入口（Card H；`.cu`/81 槽零改动；零 GPU）

- 任务：`2026-11-08-render-perworld-l-v2-host-light-entry`（owner Backend、p60、P8-渲染与BVH）
- 定位：已签 spec v3 §10.3 的 **host-only per-world 灯位姿接口**（先接口、后 G-L2 实体）；本卡**不跑 GPU 端到端**，
  渲染数值（判据 A/B/C）由 G-L2 实体承担。零 GPU、不采集、不开窗、不建 CUDA。
- 基线修订：`2e207e32`（本卡 start 过渡；所有 before 值取自该提交）。
- 来源：`docs/development-evidence/render-bvh/perworld-l-v2-light-criterion/spec.md`（sha `bb064fe0…`/39,379 B，S-1 已签生效）

## 1. 接口（一句话）与机制

`render_per_world_with_lights(renderer, context, fields, model, scene_poses, light_poses: &WorldLightPoses, camera_poses, rays)`：
**每次 world 发射前**把该 world 的 `LightDataPose{xpos,xdir}` 逐位透传到内核 1-D 实参
`light_xpos_in`/`light_xdir_in`（`render_megakernel.cu` 37/38 槽）。

- 内核灯数组仍是 **1-D vec3**；per-world 发射时内核 `worldid` 恒 0 → 重传即等价于该 world 的灯语义。
- **零 `.cu` 改动**：`git diff --stat -- src/render/kernel/cuda` 为空，全部 `.cu` sha256 前后一致
  （`artifacts/kernel-identity.json`）。
- **81 槽签名/内核身份不变**：`MEGAKERNEL_SLOTS` 仍 81 项（`artifacts/kernel-identity.json`；
  单测 `slot_40_is_bool_scalar_in_frozen_signature` 断言 `len == 81`，见 `logs/unit-tests.log`）。
- 不新增 2-D 灯数组、不改单发射结构、不引入多相机新路径（停手条款核查见 §7）。

## 2. 数据口径（写死）

- `WorldLightPoses` 承载 `nworld × nlight` 的 `LightDataPose`（world-major：`world * nlight + light`）；
  `xpos`/`xdir` 为 **f32 位型**，直接取采集侧 `state.npz` 的 `light_xpos`/`light_xdir` 原值
  （`capture_render_perworld.py::state_arrays`，`d.light_xpos/light_xdir` 转 f32）。
- **不做 FK 推导、不做 f64 重算**；`xdir` 已是上游 `normalize` 结果 → **逐位透传、不重新归一化**
  （重归一化可能改末位，破坏与参考的位级对齐）。
- 默认单 world 路径的 `light_data_poses` 语义不变：仍要求 `light_mode==0` / `light_bodyid==0`，
  归一化沿用 `crate::math::fma::fma_f32` 序列；golden 位型回归见 §5。

## 3. 改动文件与行数（git-blob 换行口径）

| 文件 | before | after | 说明 |
| --- | --- | --- | --- |
| `src/render/kernel/world_light.rs` | 0（新） | **371** | `WorldLightPoses` + `WorldLightError` + 逐 world 展平/选择 + 单测 |
| `src/render/kernel/per_world.rs` | 238 | **338** | 共享 impl + `render_per_world_with_lights` + world 数校验 + 单测 |
| `src/render/kernel/megakernel.rs` | 456 | **497** | `RenderPass.lights` + `render_world_with_lights` + 灯选择分支（默认路径不变） |
| `src/render/kernel/mod.rs` | 477 | **479** | `mod world_light;` + 导出 |
| `src/render/kernel/megakernel_arrays.rs` | 411 | 411 | **未改**（`light_data_poses` 保持原样） |
| `.normify/modules/mjwarp/render/kernel.md` | — | — | 模块源登记：新增 `world_light.rs`（`normify module source --add`） |

全部 `.rs` ≤500 行；`artifacts/src.diff` 为完整 diff，`artifacts/line-counts.txt` 记录 sha256 前后。

## 4. 默认路径不变与拒绝面

- `render_per_world` 公开签名/行为不变：内部抽共享 `render_per_world_impl(..., lights=None)`，
  v1（geom-only）路径逐字节等价。
- `light_mode!=0` / `light_bodyid!=0` 的拒绝**保留在默认路径**（`light_data_poses` 原样）：
  per-world 灯入口自带逐 world 灯位姿，不经该推导；拒绝面**不全局放开**
  （golden 负例：`default_path_still_rejects_tracked_lights`）。
- `nsamples>1`（AA×perworld）仍 `LauncherError::Unsupported`（`reject_multi_sample` 不变）。
- per-world 灯入口新增校验（全部返回可诊断错误、不 panic）：
  `WorldLightPoses::new`（worlds≥1 / lights≥1 / 条目数==worlds×lights / checked_mul 溢出）、
  `get`/`world_slice` 越界、入口 `world` 数不匹配（`InvalidArray("per_world_light_worlds")`）、
  灯数不匹配（`InvalidArray("per_world_light_count")`）。

## 5. CPU 自证（单测，零 GPU）

日志：`logs/unit-tests.log`（三条命令 exit=0）。清单：

| 测试 | 覆盖 |
| --- | --- |
| `world_light::keeps_world_major_light_poses_and_round_trips_bits` | world-major 存取 / xpos·xdir 位型 round-trip / `world_slice` |
| `world_light::rejects_count_mismatch_zero_sides_and_overflow_without_panic` | 计数不匹配（expected/actual）、零 world、零灯、`checked_mul` 溢出 |
| `world_light::rejects_out_of_range_world_and_light` | world/light 下标越界（`WorldOutOfRange`/`LightOutOfRange`） |
| `world_light::flattens_target_world_in_light_order_with_verbatim_bits` | 展平顺序 = world 内灯序；f32 位型逐位保持 |
| `world_light::select_world_lights_rejects_light_count_mismatch_and_bad_world` | 灯数不匹配 → `InvalidArray("per_world_light_count")`；world 越界 → `Driver(...)`；合法通过 |
| `world_light::default_path_light_data_poses_keep_golden_bits` | 默认路径 golden：`light_data_poses` 的 xpos/xdir 位型 == 既有 `light_data_check` 基准（`fixtures/render-bvh/scene-a/s7b/rgb.json`，pinned 3.15 forward） |
| `world_light::default_path_still_rejects_tracked_lights` | `light_mode!=0` / `light_bodyid!=0` 在默认路径仍被拒 |
| `per_world::validates_per_world_light_world_count_without_panic` | 入口 world 数不匹配拒绝 / 匹配通过（纯校验） |
| `per_world::{rejects_multi_sample_per_world_entry, splits_leaf_slots_per_world_and_requires_equal_geom_order, replicates_target_world_poses_and_restores_caller_mapping}` | v1 geom-only 路径既有单测**保持通过**（回归） |

结果：`world_light` **8 passed / 0 failed**；`per_world` **6 passed / 1 ignored**（既有 GPU 用例 ignore）；
`slot_40_is_bool_scalar_in_frozen_signature` **1 passed**（`MEGAKERNEL_SLOTS.len()==81`）。

## 6. `.cu` 零 diff 与内核身份

`artifacts/kernel-identity.json`：基线 `2e207e32` 与工作树逐文件 sha256 对照，`src/render/kernel/cuda/**`
**全部 identical，`git diff` 为空**；`MEGAKERNEL_SLOTS` 声明长度 **81**；内核入口
`render_megakernel` 与 81 槽签名未改动。

## 7. 边界与停手条款核查

- `unchanged fixtures/`、`unchanged scripts/`、`unchanged` spec 目录
  （`docs/development-evidence/render-bvh/perworld-l-v2-light-criterion/`）——`artifacts/boundaries.json` 实测 true。
- 零 GPU：本卡未建 CUDA context、未跑任何 GPU 渲染（单测全为 CPU 纯函数）。
- **停手条款核查结论**：实现中**未发现**必须改内核的迹象——不需要 2-D 灯数组/单次发射（方案 K）/AA×perworld/
  多相机新内核路径；灯位姿重传只发生在 host 侧既有 1-D 实参上传，`.cu` 与 81 槽未动 → **不触发停手**。

## 8. 非声明

本卡只提供 host 接口与数据面；**不宣称**渲染通过、不宣称灯位姿已覆盖、不宣称 per-world L v2 验收、
不宣称与上游实现等价、不宣称任何渲染数值结论。GPU 端到端渲染数值（判据 A/B/C）由 **G-L2 实体**承担。

## 9. 证据清单

| 路径 | 说明 |
| --- | --- |
| `README.md` | 本文件 |
| `receipt.json` | 机器可读回执（`/task_id` 供 normify `json` 检查） |
| `manifest.txt` | sha256 + bytes 自核清单（首行 `#`、`# base dir:`、不自列） |
| `artifacts/interface.txt` | 接口一句话 + 数据口径摘要 |
| `artifacts/src.diff` | 本卡 src 完整 diff（含新文件） |
| `artifacts/line-counts.txt` | 行数/sha256 前后对照 |
| `artifacts/kernel-identity.json` | `.cu` 零 diff + `MEGAKERNEL_SLOTS`=81 证据 |
| `artifacts/boundaries.json` | 变更面与 `unchanged fixtures/, scripts/, spec` 实测 |
| `logs/unit-tests.log` | 三条定向单测命令原始输出（exit=0） |

## 10. O1 修正（QA 复核中观察；fix 提交，基线既有缺陷非本卡引入）

- **问题**：`render_per_world_impl` 的失败恢复被 `outcome.is_ok()` 门控——失败路径预算了
  `Ok(original)` 但从不 `apply`，即失败时**未恢复**调用前位姿；与注释/回执的「失败尽力恢复」不符。
- **修法**：收尾抽为纯函数 `finish_per_world(outcome, restore, apply)`，**两条路径都 apply**：
  - 成功路径：施加调用方位姿；`restore`/`apply` 失败即首错（行为与修前一致）。
  - 失败路径：施加 `original` 恢复位姿；恢复阶段错误**只作附注**（首错原样返回，不 panic、
    不把恢复错误升为首错）。
- **新增 CPU 单测（4 条，`per_world` 由 6 passed 增至 10 passed / 1 ignored）**：
  `failure_path_restores_original_and_keeps_first_error`（失败时恢复被施加、状态回 original、返回首错）、
  `failure_path_restore_error_does_not_mask_first_error`（恢复失败不掩盖首错且仍尝试恢复）、
  `success_path_applies_caller_poses_unchanged`（成功路径同步不变）、
  `success_path_apply_error_is_returned_as_first_error`（成功路径施加失败即首错）。
- **行数**：`src/render/kernel/per_world.rs` 338 → **409**（≤500）；其余文件未改。
- **边界**：仅 `per_world.rs` + 本证据目录；`.cu` 零 diff、81 槽/内核身份不变；`unchanged fixtures/, scripts/, spec`。
- **fix 轮证据**：`artifacts/o1-fix.json`（bug/修法/单测/边界/内核身份复核）、`logs/o1-fix-unit-tests.log`
  （`per_world` 10 passed / 0 failed / 1 ignored，exit=0）。
- `receipt.json` 属实现提交时的历史证据（verified），**不回改**；O1 修正以本 README 小节 + 卡 addenda 记录。
  §3–§6 与 `artifacts/boundaries.json`、`kernel-identity.json`、`line-counts.txt`、`src.diff`、`logs/unit-tests.log`
  均为 **Card H 实现提交时快照**，本 fix 不回改。
