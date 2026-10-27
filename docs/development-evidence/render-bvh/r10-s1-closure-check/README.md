# R10-S1 观察收口独立复核（QA）：拆分零行为变化 + F1/F2/F3

- 任务：`2026-10-28-render-bvh-r10-s1-closure-check`（复核对象 `2026-10-28-render-bvh-r10-s1-obs-closure`）
- 提交：`0398c23c`（拆分 `world.rs` 499 行 → `world/` 4 文件 + F1/F2/F3 + 3 条新单测）→ `879cd012`（verified + 四门禁 runs + 模块源 30）
- 结论：**pass / 阻断 0 / 观察 0**；只读复核，未重渲染、未开 GPU、未改代码

## 1. 总体判定

拆分与三项修复全部按裁决落地，且**零行为变化**：我复用 S1 复核时的独立 harness（per-world 布局重算、
`nworld=1` 与 `OutputLayout` 逐字段相等、逐相机切片与 `CameraOutputViews::rgb_packed` 位级一致、
3-world 哨兵隔离、11 类错误面、位姿/状态语义）在新树上重跑，全部通过；公开 API 路径/再导出不变；
F1（空相机选择不再 panic）、F2（极大 worlds 返回 `CountOverflow{field:"worlds"}` 而非 capacity overflow）、
F3（`nworld==0` 优先于布局错误）由我自写用例独立复现；四门禁独立复跑 **1025/0/382**、定向 **8 passed**。

## 2. 拆分零行为变化

- 拆分后：`world/mod.rs` **22**、`world/errors.rs` **171**、`world/state.rs` **199**、`world/layout.rs` **175**，
  `world.rs` **已删除**（无残留）；目录内 4 文件均 ≤500。
- 公开 API：`mod errors`（私有）+ `pub mod layout; pub mod state;` + `pub use errors::WorldError;`
  `pub use layout::WorldLayout; pub use state::{CameraPose, WorldCameraPoses, WorldResolutions, WorldStates};`
  —— `render::context::world::{...}` 六个类型路径不变；`render/mod.rs`（378）与 `context.rs`（427）
  **未改**，顶层 `pub use context::world::{...}` 保持。
- 行为：我的 harness 在新树上重跑原有 5 类用例（布局算术、`nworld=1` 位级、隔离、错误面、位姿/状态）
  全部通过；产品定向测试 `render_multi_world_layout` = **8 passed**（5 既有 + 3 新增）。

## 3. F1（修）空相机选择不 panic

- 代码：`WorldResolutions::new` 在 `plan.resolutions.is_empty()` 时直接返回空表（不再 `chunks(0)`）；
  非空输入因期望条目数 0 返回 `CameraCountMismatch { expected: 0, actual }`。
- 我独立验证（`catch_unwind`）：`WorldResolutions::new(empty_plan, 3, vec![])` → **Ok 空表**、不 panic；
  `vec![[1,2]]` → `CameraCountMismatch{expected:0,actual:1}`；`get(0,0)` → `CameraOutOfRange{0,0}`；
  空相机选择 + `nworld=3` 的 `WorldLayout` 区段 `0..0`、`buffer_elements=0`。
- 旧 panic 已不可达（同一入口不再 panic）。

## 4. F2（修）极大 worlds 返回 CountOverflow

- 代码：`active_mask_all_true` / `WorldStates::assemble` 改用 `try_reserve(_exact)`，失败映射为
  `CountOverflow { field: "worlds" }`。
- 我独立验证（`catch_unwind`）：`WorldLayout::resolve(plan, usize::MAX)`、`WorldStates::new(usize::MAX)`
  均 **不 panic** 且返回 `CountOverflow { field: "worlds" }`；正常规模（worlds=2/4）仍可用。
- 补充：`WorldStates::with_active(usize::MAX, vec![])` 先在掩码长度校验处返回
  `ActiveMaskMismatch{expected: usize::MAX, actual: 0}`（可诊断、不分配、不 panic）。
  `world_offset` / `world_buffer_elements` 的溢出分支仍属防御（触发需 ~1 GiB 级 active 掩码分配 +
  极大像素，本片未实分配验证）——不影响 F2 的「不 panic + `worlds` 字段可达」结论。

## 5. F3（修）nworld==0 优先于布局错误

- 代码：`WorldLayout::resolve` 把 `worlds == 0 → WorldCountZero` 提到 `OutputLayout::resolve` 之前。
- 我独立验证：构造「`OutputLayout::resolve` 必失败」的 plan（`[[u32::MAX,u32::MAX],[1,1]]`）：
  `nworld=0` → **`WorldCountZero`**；`nworld=2` → **`Layout(ContextError)`**；`WorldStates::new(0)` →
  `WorldCountZero`。优先级已锁死。

## 6. 红线与证据

- `git diff 248b33ff..HEAD` 对 `camera_output.rs`(499)、`model/render.rs`(500)、`kernel/*`、`bvh/*`、
  `fixtures/*`、`Cargo.*` **为空**；`DEVICE_SOURCES`=28、入口=6、`CollisionArgs` 504 B 未动
  （随 unit 断言）；`context.rs`(427)、`mod.rs`(378) 未改。
- 新行数：`world/mod.rs` 22、`errors.rs` 171、`state.rs` 199、`layout.rs` 175、测试 478；`world.rs` 不存在。
- 模块源 **27 → 30**（`world/mod.rs`、`layout.rs`、`state.rs` 登记；`world.rs` 移除）；
  `normify validate --fingerprints` 错误 0（5 条既有告警）。
- 证据 `obs-closure-receipt.json` `/task_id` 正确（141 行，`split`/`fixes` 记录完整）；证据文本 LF 无 BOM、禁词 0。

## 7. 门禁独立复跑（收口最终树）

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets --locked -- -D warnings` | exit 0 |
| `cargo clippy --all-targets --locked --features cuda -- -D warnings` | exit 0 |
| `cargo test --locked` | exit 0：**1025 passed / 0 failed / 382 ignored**（126 result 行） |
| `cargo test --locked --test render_multi_world_layout` | **8 passed / 0 failed** |

与归档 closure 日志重数一致（unit 1025/0/382、定向 8 passed）。

## 8. 观察

无（0 条）。F1/F2/F3 均已修并由我独立复现验证；拆分零行为变化与红线核对全部通过。

## 9. 边界与不宣称

只读复核：未重渲染、未开 GPU、未开窗口、未改代码/判据/参考/容差。`world_offset`/`world_buffer_elements`
的溢出分支未做实分配验证（见 §4 补充），属防御路径记录；本片不宣称 GPU 端到端（R10-S2 另片）。
