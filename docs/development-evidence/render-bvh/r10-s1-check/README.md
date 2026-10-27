# R10-S1 独立复核（QA）：多 world 状态与布局（宿主侧）

- 任务：`2026-10-28-render-bvh-r10-s1-check`（复核对象 `2026-10-28-render-bvh-r10-s1-world-state`）
- 提交：`4cf1159c`（14 文件 +3506；`world.rs` 新 499 行、`context.rs` +1=427、`render/mod.rs` +4=378、`tests/render_multi_world_layout.rs` 397）→ `14ff2536`（verified + 四门禁 runs + 模块源 27）
- 结论：**pass / 阻断 0 / 观察 3（1 一般 + 2 建议）**；只读复核，未重渲染、未开 GPU、未改代码

## 1. 总体判定

per-world 语义、`nworld=1` 与既有布局的位级一致性、3 world 哨兵隔离、11 类错误面、`nworld × ncam`
位姿/「同模型多 state」语义全部由我自写 harness 独立验证通过；`context.rs` 确认只 +1 行
`pub mod world;`、`mod.rs` 只 +4 行导出（无渲染逻辑）；行数棘轮与红线全部符合；四门禁我独立复跑
**1022/0/382**、定向 **5 passed**，与 Backend 一致。三条观察均不阻断本片：F1 是空相机表下的
`chunks(0)` panic（空相机选择是本仓已支持场景，建议 S2 接线前修），F2/F3 是极大 `worlds` 的分配型
panic 与错误优先级口径。

## 2. per-world 语义（QA 自写实现重算）

我的 harness 用**自己按 plan 重算**的单 world 通道布局（独立于 `OutputLayout` 实现）对
`[[5,4],[3,2],[1,7]]` + depth/seg 通道异构 的 plan 验证：

- 三通道像素数 = **33 / 27 / 13**，`*_adr` 与我的重算逐项相等（含 `-1` 未启用项）；
- `nworld=1`：`WorldLayout.layout()` 与 `OutputLayout::resolve` **逐字段相等**；`world_range` = `0..pixels`；
  `buffer_elements` = 单 world 像素数；
- `nworld=1` 逐相机 `camera_range` 切片与既有 `CameraOutputViews::rgb_packed` **逐位一致**（同 plan/布局/随机缓冲）；
- `nworld=3`：`world_offset = world × pixels`、`world_range = [w·P, (w+1)·P)`、world 内逐相机区段 =
  `w·P + adr` 起 `pixels` 个，全部与我的重算一致；`buffer_elements = 3 × P`。

## 3. 隔离与错误面（QA 自写哨兵）

- 2 相机 4×3 + 2×2 = **16 像素/world × 3 world** 哨兵缓冲：逐 world `world_view` 只返回自己 16 个元素，
  区段 `[0,16)/[16,32)/[32,48)` 两两不相交；逐 world 相机区段 `0..12 / 12..16 / 32..44 / 44..48`；
  短缓冲 47 → `BufferRange { world: 2, end: 48, actual: 47 }`；world 3 → `WorldOutOfRange { world: 3, worlds: 3 }`。
- 错误面（我逐类触发，均不 panic、`Display` 非空且含索引/上限）：`WorldCountZero`（4 个入口）、
  `WorldOutOfRange`（range/view/get/world_slice）、`WorldInactive`（range/view/insert）、`StateMissing`、
  `ActiveMaskMismatch`（layout/states）、`CameraCountMismatch`（pose/resolution 表）、`CameraOutOfRange`
  （get/camera_range）、`ResolutionMismatch`、`ChannelDisabled`、`BufferRange`、`Layout(ContextError)`
  （2 × u32::MAX² 的单 world 布局溢出 → 包装为 `WorldError::Layout`）；`CountOverflow` 见 F2。
- 空相机表（ncam=0）：`WorldLayout` 行为明确（cameras=0、range `0..0`、`world_view` 为空、`camera_range` →
  `CameraOutOfRange`）；`WorldResolutions` 见 F1。

## 4. 位姿/状态语义（同模型多 state）

- `WorldCameraPoses`：2 world × 2 相机 world-major，`world_slice` 与 `get(w,c)` 映射一致；world 间相机数
  相同（同一模型）、仅位姿 state 不同；`new(0,..)` 拒绝。
- `WorldResolutions`：3 × 2 表逐项等于 `plan.resolutions`；条目数不符/逐项不符均可诊断。
- `WorldStates`：每 world 一份 `Option<T>` + 激活位；读取顺序 world → 激活 → 存在；`insert` 未激活被拒；
  `set_active` 后可写；越界/缺失/未激活分别报对应错误。

## 5. 接线与不变量

- `src/render/context.rs` **只 +1 行** `pub mod world;`（427 行）；`src/render/mod.rs` **只 +4 行**
  （1 行注释 + 3 行 `pub use context::world::{...}`），**无渲染逻辑**。
- 行数：`world.rs` **499**（⚠ 距 500 仅 **1 行**；**S2 若需扩它必须先拆**，已写入 receipt 风险）、
  `context.rs` 427、`render/mod.rs` 378、`tests/render_multi_world_layout.rs` 397、
  `camera_output.rs` 499、`model/render.rs` 500、`launch.rs` 500、`megakernel.rs` 498、`cast.rs` 494。
- `git diff 48dc7db8..HEAD` 对 `camera_output.rs` / `model/render.rs` / `kernel/*` / `bvh/*` / `fixtures/*` /
  `Cargo.*` **为空**；代码改动仅 `world.rs`、`context.rs`、`mod.rs`、新测试。
- `.normify/modules/mjwarp/render.md` 模块源 **27** 项含 `world.rs`；`normify validate --fingerprints`
  错误 0（5 条既有告警）；收口任务 `normify check` 通过。

## 6. 门禁独立复跑（收口最终树）

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets --locked -- -D warnings` | exit 0 |
| `cargo clippy --all-targets --locked --features cuda -- -D warnings` | exit 0 |
| `cargo test --locked` | exit 0：**1022 passed / 0 failed / 382 ignored**（126 result 行） |
| `cargo test --locked --test render_multi_world_layout` | **5 passed / 0 failed** |

与归档一致（归档 unit 重数 1022/0/382/126）。证据 manifest 10 项对工作树 **0 不符**；证据文本
LF 无 BOM；3 条禁词组 0 命中；`receipt.json` `/task_id` 正确。

## 7. 观察（3 条，给修/接受建议）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| F1 | **一般** | `WorldResolutions::new(&plan, worlds, vec![])` 在 `plan.resolutions` 为空时 panic：`expected = worlds×0 = 0` 且表长 0 通过校验，随后 `resolutions.chunks(0)` 触发 `chunk size must be non-zero`。空相机选择是本仓既有支持场景（R08 空选择 no-op），nworld>1 + 空选择就会踩到（QA 已用 `catch_unwind` 复现：panic caught = true） | **修**（2 行级）：`cameras == 0` 时直接返回空表，或改按下标循环；建议在 S2 使用 `WorldResolutions` 前修掉 |
| F2 | 建议 | 极大 `worlds`（如 `usize::MAX`）在 `WorldLayout::resolve` / `WorldStates::new` / `with_active` 先分配 `vec![true; worlds]`，以 capacity overflow **panic** 而非 `CountOverflow`；因此 `world_offset`/`world_buffer_elements`/`world_states` 的 `CountOverflow` 在公共 API 路径实际不可达 | **修**或**接受**：加上界/`try_reserve` 返回 `CountOverflow`，或文档写明「worlds 必须可分配、CountOverflow 仅防御」 |
| F3 | 建议 | `WorldLayout::resolve` 先 `OutputLayout::resolve(plan)` 再查 `worlds == 0`：非法 plan + `nworld=0` 返回 `Layout(ContextError)` 而非 `WorldCountZero`；错误优先级未在 README 写明 | **接受**（真实 plan 合法时 `nworld=0` 正确拒绝）或**修**（先判 nworld==0 / 文档写优先级） |

**Arch 裁决（2026-10-28）**：三条全部判「修」。因 `world.rs` 已 499/500，先由收口片
`2026-10-28-render-bvh-r10-s1-obs-closure` 拆分（`world/mod.rs` + `world/layout.rs` + `world/state.rs`/errors）
再修 F1–F3（F1 空相机表不 panic；F2 `try_reserve`/上界返回 `CountOverflow`；F3 `nworld==0` 前置或锁优先级）；
本复核片只报 verdict，不在本片改代码。

## 8. 边界与不宣称

只读复核：未重渲染、未开 GPU、未开窗口、未重跑 GPU 端到端（R10-S2 另片）、未改代码/判据/参考/容差。
本片只核宿主侧多 world 状态/布局与错误面，不宣称 GPU 端到端或渲染级结论。
