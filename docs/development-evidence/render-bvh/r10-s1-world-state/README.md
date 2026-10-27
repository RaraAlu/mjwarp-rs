# R10-S1：多 world 状态与布局（宿主侧，无 GPU、无窗口）

- 任务：`2026-10-28-render-bvh-r10-s1-world-state`（p74，P8-渲染与BVH，owner Backend）
- 上游冻结：`71da24d956378a87a703b6e1442b13aec0c4ac29`
- 本片只做**宿主侧**：无 GPU、无窗口、无新捕获；不改 kernel / 设备 `.cu` / `camera_output.rs`(499) /
  `model/render.rs`(500)。

## 1. 语义口径

上游 `render_util.create_render_context` 的状态带前导 `nworld` 维：相机位姿为
`cam_xpos[nworld, ncam, 3]` / `cam_xmat[nworld, ncam, 9]`，输出缓冲形状为 `[nworld, pixels]`
（world-major）。本片口径：

- **双 world = 同模型多 state**：world 维只复制状态（每个 world 一份位姿/模型状态），
  **不解析第二个模型**；相机下标与分辨率在所有 world 间一致。
- **输出区段**：在既有 `OutputLayout`（单 world 的 `*_adr` / `*_pixels`）上按 `nworld` 乘法展开，
  每 world 区段为 `[world * pixels, (world + 1) * pixels)`，world 间互不重叠；
  `nworld == 1` 时与单 world 布局逐位一致（`0..pixels`）。

## 2. 交付

| 文件 | 变更 | 行数 | 说明 |
| --- | --- | --- | --- |
| `src/render/context/world.rs` | 新增 | 499 | `WorldError`、`CameraPose`、`WorldCameraPoses`、`WorldResolutions`、`WorldStates<T>`、`WorldLayout` |
| `src/render/context.rs` | +1 | 427 | 只加 `pub mod world;`（父模块声明） |
| `src/render/mod.rs` | +4 | 378 | 顶层再导出 6 个类型 |
| `tests/render_multi_world_layout.rs` | 新增 | 397 | 5 条单测：world 隔离、nworld=1 回归、错误面、位姿、分辨率 |

未改（行数不变）：`src/render/context/camera_output.rs` 499、`src/model/render.rs` 500、
`src/render/kernel/launch.rs` 500、`megakernel.rs` 498、`cast.rs` 494、`src/render/kernel/cuda/*.cu`
与设备源；`DEVICE_SOURCES`=28、产品入口=6、`CollisionArgs`=504 B、判据/参考/夹具零改动。

## 3. per-world API

- `WorldCameraPoses`：`nworld × ncam` world-major（`world * ncam + camera`）；`get(world, camera)`、
  `world_slice(world)`；`new` 校验条目数 = `nworld × ncam`。
- `WorldResolutions`：`nworld × ncam` 分辨率表，逐项校验等于 `plan.resolutions`；不符返回
  `ResolutionMismatch { world, camera, expected, actual }`。
- `WorldStates<T>`：per-world 模型/场景状态容器（每 world 一份 `Option<T>` + 激活位）；
  读取顺序固定为 world 下标 → 激活位 → 状态是否存在。
- `WorldLayout`：`world_range(channel, world)`、`world_view(channel, world, data)`（切区段并校验
  缓冲长度）、`camera_range(channel, world, camera)`（world 偏移 + 相机 adr）、
  `buffer_elements(channel) = nworld × 单 world 像素数`、`channel_pixels(channel)`。

## 4. 错误面（全部读取/分配前返回，不 panic）

| 条件 | 错误 |
| --- | --- |
| `nworld == 0` | `WorldCountZero` |
| world 下标越界 | `WorldOutOfRange { world, worlds }` |
| world 未激活 | `WorldInactive { world }` |
| 该 world 状态缺失 | `StateMissing { world }` |
| 激活掩码长度 ≠ `nworld` | `ActiveMaskMismatch { expected, actual }` |
| per-world 状态条目数 ≠ `nworld × ncam` | `CameraCountMismatch { expected, actual }` |
| 相机下标越界 | `CameraOutOfRange { camera, cameras }` |
| 某 world 某相机分辨率与 plan 不符 | `ResolutionMismatch { world, camera, expected, actual }` |
| 该相机该通道未启用（`adr == -1`） | `ChannelDisabled { channel, camera_index }` |
| world 区段末端超出缓冲 | `BufferRange { world, end, actual }` |
| 计数溢出 | `CountOverflow { field }` |
| 单 world 布局解析失败 | `Layout(ContextError)` |

## 5. 单测（`tests/render_multi_world_layout.rs`，5 条，定向 log 见 `logs/`）

1. `world_views_are_disjoint_and_do_not_read_neighbours`：2 相机（4×3 + 2×2 = 16 像素/世界）× 3 world
   （整缓冲 48 元素）的 rgb/depth/seg 哨兵缓冲；逐 world `world_view` 只返回自己的 16 元素、
   区段两两不相交（`seen` 断言）、`camera_range` 在世界偏移之后（0..12 / 12..16 / 32..44 / 44..48）；
   短缓冲 `47` → `BufferRange{world:2,end:48,actual:47}`；world 3 → `WorldOutOfRange`。
2. `single_world_path_matches_existing_layout_and_camera_views`：`nworld = 1` 时 world 布局与既有
   `OutputLayout` 完全相等、区段为整缓冲；逐相机 `camera_range` 切片与既有 `CameraOutputViews`
   的 `rgb_packed` **逐位一致**（同 plan/布局/缓冲）。
3. `error_surface_is_diagnostic_without_panicking`：`nworld=0`、掩码长度、未激活 world 的 range/view、
   world 越界、通道未启用、位姿条目数/越界、分辨率不符、状态容器缺失/未激活/越界，全部返回对应
   错误且 `Display` 文本非空。
4. `camera_poses_are_nworld_by_ncam_same_model_multi_state`：2 world × 2 相机位姿 world 切片内容与
   `get(w,c)` 映射一致；同模型（相机数一致）但位姿 state 不同；`nworld=0` 拒绝。
5. `resolutions_replicate_plan_and_reject_mismatch`：`3 × 2` 分辨率表逐项等于 plan、`get(2,1)` 正确；
   `nworld=0` 拒绝。

## 6. 门禁与定向（本片最终树）

| 门禁 | 结果 | wall_ms | 日志 |
| --- | --- | --- | --- |
| `cargo fmt --check` | exit 0 | 1573 | `logs/fmt-check.log`（空输出） |
| `cargo clippy --all-targets --locked -- -D warnings` | exit 0 | 506 | `logs/clippy.log` |
| `cargo clippy --all-targets --locked --features cuda -- -D warnings` | exit 0 | 386 | `logs/clippy-cuda.log` |
| `cargo test --locked` | exit 0；**1022 passed / 0 failed / 382 ignored**（126 个 result 行） | 66291 | `logs/unit.log` |
| `cargo test --locked --test render_multi_world_layout`（定向） | **5 passed / 0 failed** | 178 | `logs/directed-render-multi-world-layout.log` |

窗口内 GPU 端到端（真实 `nworld > 1` 渲染）不在本片，属 R10-S2（1 验证窗、无新捕获）。

## 7. 证据格式与自检

- 文本证据 UTF-8 无 BOM、LF/CR=0；日志为命令原样输出。
- 机器可读例外：`logs/unit.log`（>500 行）、`receipt.json`、`manifest.json`。
- 禁词自检：任务卡 `forbid_claim` 三项（本目录不复写字面）命中 0；四个措辞约束词命中 0。

## 8. 边界与后续

- 本片不接 GPU、不开窗口、不做新捕获；不改 kernel / 设备源 / 判据 / 参考 / 容差 / 夹具。
- `camera_output.rs` 的逐相机视图仍只支持 `nworld == 1`（S1 遗留拒绝，本片不改）；多 world 的
  逐相机视图与设备侧整合在 R10 后续片。
- 未接产品入口调用链（R10-S2 GPU 端到端另片）。

## 9. R10-S1 观察收口（拆分 + F1–F3，2026-10-28）

- 收口任务：`2026-10-28-render-bvh-r10-s1-obs-closure`；收据 `obs-closure-receipt.json`（含 `/task_id`），
  门禁/自检数字见收据 `gates` 与 `self_checks`。本段**只改文件组织与三处防御**，公开 API 与渲染语义不变。
- **拆分（零行为变化）**：`src/render/context/world.rs`(499) → `src/render/context/world/mod.rs`(22) +
  `errors.rs`(171) + `state.rs`(199) + `layout.rs`(175)；`context.rs` 的 `pub mod world;` 声明不变（指向目录），
  `render/mod.rs` 再导出路径不变。`nworld = 1` 与 `OutputLayout` 逐字段相等、逐相机切片与
  `CameraOutputViews::rgb_packed` 逐位一致（既有回归单测原样通过）。
- **F1（修，真 panic）**：`WorldResolutions::new` 在 `cameras == 0` 时不再走 `chunks(0)`；空相机选择直接返回
  空表（非空输入报 `CameraCountMismatch { expected: 0 }`）。新单测
  `empty_camera_selection_is_accepted_without_panic`（`nworld = 3` + 空选择）。
- **F2（修）**：`WorldStates::new` / `WorldLayout::resolve` 的全激活掩码与状态槽位改走
  `try_reserve_exact`；`usize::MAX` 级 `worlds` 返回 `CountOverflow { field: "worlds" }` 而非 capacity
  overflow panic（`CountOverflow` 公共路径可达）。新单测
  `huge_world_count_returns_count_overflow_without_panic`（`catch_unwind` 断言不 panic）。
- **F3（修）**：`WorldLayout::resolve` 的 `nworld == 0` 判定提到 `OutputLayout::resolve` **之前**（错误优先级
  写死）；新单测 `nworld_zero_has_priority_over_layout_errors` 用「OutputLayout 会失败的 plan」锁定：
  `nworld = 0` 返回 `WorldCountZero`、`nworld = 2` 才返回 `Layout(..)`。
- 收口门禁（最终树）：fmt 1577 ms、clippy 402 ms、clippy-cuda 411 ms、unit **1025 passed / 0 failed /
  382 ignored**（63516 ms，126 result 行）；定向 **8 passed / 0 failed**（187 ms，5 条既有 + 3 条新增）。
- 红线：`camera_output.rs`(499)、`model/render.rs`(500)、kernel 三文件（500/498/494）、设备 `.cu`、
  `DEVICE_SOURCES`=28、入口=6、`CollisionArgs` 504 B、判据/参考/夹具零改动；拆分后各文件 ≤500。
