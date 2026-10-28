# R05 前置：渲染侧 hfield 数据通道（方案 A，薄适配器）

任务：`2026-10-28-render-bvh-r05-hfield-channel`（P8 渲染与 BVH；p57；owner Backend）
判据树 HEAD：`9a32bf87990e4e79a14b49e1246dc912550bc7b7`（R05-S1 加固收口后）
阶段：**方案 A 实现 + 7 条宿主单测 + 模块源登记完成**；纯宿主，无 GPU、无窗口、无渲染数值结论。

## 1. 背景与裁决（为什么走方案 A）

R05-S1 已确认：渲染侧 `RENDER_FIELDS`（116 项）没有 hfield 通道，`src/model/render.rs` 是 500 行红线文件，`field("hfield_*")` 恒 `None`。因此 S2（设备侧 hfield BVH/相交）需要一个可用的 hfield 输入来源。

**发现**：模型侧已有两条**已验收**通道——
- `VolumeModelInput`（`src/model/volume_input.rs`，G1/G10 链路）：`hfield_data() -> &[f32]`、`hfield_size() -> &[f64]`、**`hfield_size_f32()`（唯一 f64→f32 转换点，`model::volume::num_to_f32`）**、`ints("hfield_nrow/ncol/adr")`；`new()` 已校验字段数/类型/长度/`adr+span<=nhfielddata`；
- `CollisionInputs`（`src/model/collision_inputs.rs` L110-114）：同源 hfield 字段 + `nhfield/nhfielddata` 计数。

**裁决（Arch 2026-10-28，方案 A）**：复用 `VolumeModelInput` 做**薄适配器**，零 native、零 `RENDER_FIELDS`、零红线、零 ABI。

**方案 B（未采用）的代价**：拆 `src/model/render.rs`（500 → ~382）+ 新 `src/model/render/fields.rs`（~126）；`CountUnit` +2、`RenderModelCounts` +2、`RENDER_FIELDS` 116→121（追加在末尾 id 116-120）、`io/render.rs` 计数/静态断言 +8–12、native `model_probe_render.cpp` render_stage 加 5 case + `mjwarp_render_info` 加 2 字段（**native ABI 变更，需重冻**）、`RenderModelCounts::validate` +2、所有硬编码 116 的测试/证据同步——约 10 文件、+40–60 行、触碰红线与 native ABI。

## 2. 实现

新增 `src/render/context/hfield_inputs.rs`（**70 行**）：

| 项 | 内容 |
| --- | --- |
| `VolumeHfieldInputs<'a>` | 持有 `size: Vec<f32>`（f64→f32 转换结果），借用 `nrow/ncol/adr: &[i32]`、`data: &[f32]` |
| `from_volume(&VolumeModelInput) -> Result<Option<Self>, NativeProbeError>` | `nhfield==0` → `Ok(None)`（无 hfield 合法）；否则 `hfield_size_f32()` + `ints(...)` + `hfield_data()`；**错误直传**、不复制 data、不读 `nworld` |
| `as_inputs() -> HfieldInputs<'_>` | 五字段全 `Some`，可直接交给 [`resolve_hfield`] |
| `resolve() -> Result<Option<HfieldParams<'_>>, HfieldError>` | 一步 `resolve_hfield(&self.as_inputs())` 组合 |

**生命周期说明**：`HfieldInputs::size` 是 `&[f32]`，而体积通道 `hfield_size` 是 f64、转换结果为 owned `Vec<f32>`；因此适配器返回持有该结果的 `VolumeHfieldInputs`，再用 `as_inputs()` 得到 `HfieldInputs`。**不改 S1 已验收的 `HfieldInputs` API**（不为 `size` 引入 `Cow`）。

**挂载**：`src/render/context/params.rs` 以 `#[path = "hfield_inputs.rs"] pub mod hfield_inputs;`（+4 行，与 `lighting.rs`/`hfield.rs` 同构）；**未改 `src/render/mod.rs`**（channel 卡 scope 只含 `src/render/context/`；S2 若需要顶层导出 `pub use context::params::hfield_inputs::VolumeHfieldInputs;` 只加 1 行，届时另行处理）。外部路径：`mjwarp_rs::render::context::params::hfield_inputs::VolumeHfieldInputs`。

## 3. 语义边界（显式验证，不静默改写）

- **`nhfield==0`**：`from_volume` 返回 `Ok(None)`，与 S1 `resolve_hfield` 的「空输入 `Ok(None)`」一致；
- **`nrow/ncol` 零值口径差异**：体积通道只拒绝**负数**（`nrow/ncol/adr >= 0`，`volume_input.rs::check_ranges`），`nrow=0` **可构造**；S1 校验层对 `<= 0` 拒绝（`HfieldError::NonPositiveRowsCols`）。本适配器不改写该口径，测试 `boundary_errors_are_rejected_by_volume_construction` 显式验证：体积层接受 `nrow=0` → 适配 → `resolve()` 返回 `NonPositiveRowsCols{index:0,nrow:0,ncol:3}`；
- `adr` 越界（`adr+span > nhfielddata`）由体积通道构造期拒绝，适配器不会见到非法区段。

## 4. 文件与行数（LF 口径）

| 文件 | 行数 | 说明 |
| --- | --- | --- |
| `src/render/context/hfield_inputs.rs`（新） | **70** | 方案 A 适配器 |
| `src/render/context/params.rs` | **197**（+4） | `#[path]` 挂 `hfield_inputs` 模块 + 文档注 |
| `tests/render_hfield_inputs.rs`（新） | **218** | 7 条宿主单测（含 G10 夹具同源数组比对） |
| `.normify/modules/mjwarp/render.md` | — | 模块源登记（35→36）+ 指纹刷新（done） |

`src/render/context/hfield.rs`（372）与 `src/render/mod.rs`（384）本片未改。全部文件 ≤500 行。

## 5. 测试（7/7 通过）

`cargo test --locked --test render_hfield_inputs` → **7 passed / 0 failed**（`logs/test.log`）

| 用例 | 断言 |
| --- | --- |
| `adapts_volume_hfield_into_inputs` | 五字段适配齐全（size/nrow/ncol/adr/data） |
| `resolve_full_chain_matches_s1_views` | 全链路视图：size/nrow=3/ncol=3/adr=0/span=9/`data_of` 内容 |
| `no_hfield_returns_none` | `nhfield=0` → `None` |
| `g10_fixture_arrays_match_volume_adapter` | 读 `fixtures/g10-p5/hfield-f32/hfield-box-cpu.json` 的 arrays → 构造体积输入 → 适配：`size` 与 `num_to_f32(夹具 size)` 逐位一致、`nrow/ncol/adr/data` 与夹具同源数组一致（**只作输入一致性，不作渲染数值结论**） |
| `size_conversion_is_single_f64_to_f32` | 0.1 位型 `0x3DCC_CCCD`，与 `model::volume::num_to_f32` 一致（唯一转换点） |
| `boundary_errors_are_rejected_by_volume_construction` | `adr` 越界在体积层拒绝；`nrow=0` 体积层接受但 S1 层 `NonPositiveRowsCols`（边界显式） |
| `adapter_is_stable_and_copy_free` | 两次适配逐字段相同；`data` 指针与 `model.hfield_data()` 相同（未复制）；视图数=nhfield（单份无 world 维度） |

## 6. 红线与不变量（零改动）

`render.cu` **499** / `launch.rs` **500** / `cast.rs` **494** / `model/render.rs` **500** / `camera_output.rs` **499** / `megakernel.rs` **490** / `io/render.rs` **453**：均未改动（`logs/lines.log`）；native `model_probe_render.cpp` 未改；`DEVICE_SOURCES=28`、入口 6、`CollisionArgs` 504 B、81 槽 ABI、`fixtures/precision` 零改动。

## 7. 模块源登记

- `normify module source mjwarp.render --add src/render/context/hfield_inputs.rs`：**35 → 36**；
- `normify validate --fingerprints`：指纹过期由 `normify done` 刷新，收口后错误 0（O1 同类缺口主动预防）。

## 8. 未覆盖项（必须随证据保留）

- **产品路径「谁在 context 创建时提供 `VolumeModelInput`」留 S2 决定**（本片只交付适配器；是否由 `Session`/调用方在创建 render context 时一并提供体积输入，S2 定接口）；
- **设备侧 hfield BVH 与内核相交未实现**（S2）；`hfield_bvh_id` 槽位语义未变（仍为既有占位）；
- **上游 2D 数值/等价未验证**：本片只验证输入一致性，零渲染数值结论；
- `CollisionInputs` 通道本次未用（`VolumeModelInput` 足以满足本片需求；若 S2 需要与 collision 侧共享同一快照，再评估）。
