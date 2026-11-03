# R06-S2 宿主 flex 通道：9 数组 + `bvh_nflexgeom` 实参填充与两处 2-D 形状

任务：`2026-11-03-render-r06-s2-flex-host-channel`。日期：2026-11-04（空闲段）。**零 GPU、零窗口、
不生成参考、不改设备行为**。本片是 R06 柔性体渲染第二实现片：只建**显式宿主通道**（类型 + 显式校验 +
上传 + 实参形状），model/context 的真实数据源归 R06-S2b，设备 cast/BVH 分支归 S3。

## 1. 交付物

| 文件 | 行数 | 作用 |
| --- | --- | --- |
| `src/render/kernel/flex_channel.rs` | 296 | `FlexHostInputs`（9 个 flex 数组 + nworld/nflex/nflexvert）、`validate`/`validate_counts` 显式校验、槽形状函数、`upload_flex_arrays`、u64→u32 字对承载 |
| `src/render/kernel/flex_channel/tests.rs` | 338 | 8 个 `#[cfg(test)]` 单测（空/单/多 flex、长度/世界数/映射/单调拒绝、形状口径、字对） |
| `src/render/kernel/megakernel.rs` | 379 | `#[path]` 声明 + `pub use flex_channel::FlexHostInputs` + `render_with_flex(..., Option<&FlexHostInputs>)` + `RenderPass` 抽参（守 clippy 参数上限）；**原 `render` 签名与行为不变** |
| `src/render/kernel/megakernel_uploads.rs` | 464 | `UploadInputs.flex` 通道入口、`MegakernelUploads.flex: FlexUploads`；原 9 处 1 元素占位上传改为通道上传 |
| `src/render/kernel/megakernel_args.rs` | 254 | 9 个 flex 槽按通道计数给形状；`bvh_nflexgeom` 标量 = `flex_geom_flexid.len()` |
| `.normify/modules/mjwarp/render/kernel.md` | — | 模块源登记：新增 `flex_channel.rs`、`flex_channel/tests.rs`，并补登 S1a 漏登的 `megakernel_args.rs` |

拆分原因（500 行硬规则）：`flex_channel.rs` 完整版 628 行超限，单测按 `src/model/flex/tests.rs` 先例拆到
子文件（`#[cfg(test)] #[path = "flex_channel/tests.rs"] mod tests;`）；`megakernel.rs` 314 → 379 行（450 死线内）。

## 2. 上游语义映射（pinned `71da24d`）

来源：`render_util.py` L490-512（flex 家族构建）/ L740-748（context 字段）、`render.py` L1520-1583
（`_render_megakernel` 实参顺序）。9 个 flex 形参全部已在 S0 对表的 81 槽内（见
`docs/development-evidence/render-bvh/r06-flex-precheck/`）。

| 槽（0-based） | 上游形参 | 元素类型 | 本片形状 | 数据来源（本片/后续） |
| --- | --- | --- | --- | --- |
| 19 | `flex_vertadr` | I32 | `[max(nflex,1)]` | 通道（S2b 从 MjModel） |
| 20 | `flex_edge` | Vec2I | `[max(nflexedge,1)]` | 通道（S2b） |
| 21 | `flex_radius` | F32 | `[max(nflex,1)]` | 通道（S2b；MjModel 为 mjtNum → f32） |
| 38 | `flexvert_xpos_in` | Vec3 | **2-D** `[nworld, max(nflexvert,1)]` | 通道（S2b 从 flex 物理输出） |
| 42 | `bvh_nflexgeom` | Scalar(I32) | `len(flex_geom_flexid)` | 通道（派生） |
| 56 | `flex_bvh_id` | U64 | `[max(nflex,1)]` | 通道（S3 填 BVH 句柄，本片由调用方供 0） |
| 57 | `flex_group_root` | I32 | **2-D** `[nworld, max(nflex,1)]` | 通道（上游初值 -1，S3 填 flex BVH group root） |
| 65 | `flex_rgba` | Vec4 | `[max(nflex,1)]` | 通道（S2b；MjModel float） |
| 66 | `flex_geom_flexid` | I32 | `[max(bvh_nflexgeom,1)]` | 通道（S2b 按 flex_dim/edgeadr/edgenum 派生） |
| 67 | `flex_geom_edgeid` | I32 | `[max(bvh_nflexgeom,1)]` | 通道（同上；2D/3D 图元为 -1） |

上游 flex 语义（S3 判据依据，本片只接线不消费）：1-D flex 每边一个胶囊图元（`flexid` 重复、`edgeid` 为边号），
2D/3D flex 每个一个盒图元（`edgeid = -1`）；命中走 `ray_flex_with_bvh`；颜色 `flex_rgba[flexid]`、seg 编码
`vec2i(flex_id, ObjType.FLEX)`。

## 3. 形状与「不构成 ABI 变更」

- **81 槽槽数、顺序、元素类型逐项不变**：`src/runtime/launcher/render/args.rs::MEGAKERNEL_SLOTS` 零改动；
  `RenderArrayValue::contiguous` 的 ndim/shape 是宿主侧数组描述，签名校验只比对元素类型
  （`RenderArgKind::Array(RenderElem)`），不含秩。
- 唯一变化：`flex_group_root` 从 1-D `[1]` 改 2-D `[nworld, max(nflex,1)]`、`flexvert_xpos` 从 1-D `[1]` 改
  2-D `[nworld, max(nflexvert,1)]`（上游 `wp.array2d`）。**槽数/槽类型不变 ⇒ 不构成 ABI 变更**；
  S0 对表见 `docs/development-evidence/render-bvh/r06-flex-precheck/README.md` §2。
- **设备侧零改动**：`git diff --stat -- src/render/kernel/cuda` 为空（0 文件 0 行）⇒ `MEGAKERNEL_SOURCE` /
  PTX 不变；内核仍把 flex 槽当 ABI 占位不读取（设备 cast 分支 S3）。

## 4. 空通道（零 flex）等价占位

`None` 或空通道（`nflex == 0` 且全数组为空）走以下占位，零 flex 场景设备输出不变：

| 槽 | 数据 | 形状 | 与 S7b2 的关系 |
| --- | --- | --- | --- |
| `flex_vertadr` | `[0]` | `[1]` | 相同 |
| `flex_edge` | `[0,0]` | `[1]` | 相同 |
| `flex_radius` | `[0.0]` | `[1]` | 相同 |
| `flexvert_xpos` | `3*nworld` 个 0 | `[nworld,1]` | 仅 descriptor 维度 1-D→2-D（内核不读） |
| `flex_bvh_id` | `[0,0]`（u32 字对） | `[1]` | 相同 |
| `flex_group_root` | `nworld` 个 0 | `[nworld,1]` | 仅 descriptor 维度 1-D→2-D（内核不读） |
| `flex_rgba` | `[0;4]` | `[1]` | 相同 |
| `flex_geom_flexid` / `flex_geom_edgeid` | `[0]` | `[1]` | 相同 |
| `bvh_nflexgeom` 标量 | `0` | — | 相同 |

## 5. 显式校验面（不 panic、不静默回退）

| 字段 | 规则 | 错误 |
| --- | --- | --- |
| — | 通道 `nworld` 必须等于发射侧 `resolved.nworld` | `InvalidArray("flex_nworld")` |
| — | 非空通道 `nflex >= 1`、`nflexvert >= 1` | `InvalidArray("flex_nflex")` / `("flex_nflexvert")` |
| `flex_vertadr` | 长度 = `nflex`；首项 0、非降、每项 ∈ `[0, nflexvert)` | `InvalidArray("flex_vertadr")` |
| `flex_edge` | 长度为偶数；每端点 ∈ `[0, nflexvert)` | `InvalidArray("flex_edge")` |
| `flex_radius` | 长度 = `nflex` | `InvalidArray("flex_radius")` |
| `flex_rgba` | 长度 = `4*nflex` | `InvalidArray("flex_rgba")` / `Overflow("flex_rgba")` |
| `flex_bvh_id` | 长度 = `nflex` | `InvalidArray("flex_bvh_id")` |
| `flexvert_xpos` | 长度 = `3*nworld*nflexvert` | `InvalidArray("flexvert_xpos")` / `Overflow(...)` |
| `flex_group_root` | 长度 = `nworld*nflex` | `InvalidArray("flex_group_root")` / `Overflow(...)` |
| `flex_geom_flexid`/`flex_geom_edgeid` | 等长；flexid ∈ `[0, nflex)`；edgeid = -1 或 ∈ `[0, nflexedge)` | `InvalidArray("flex_geom_flexid")` / `("flex_geom_edgeid")` |
| （不校验） | `vertadr[f] + vertnum[f] == nflexvert` 累加、1D/2D/3D ↔ `edgeid == -1` 对应 | 通道不携带 `flex_vertnum`/`flex_dim`/`flex_edgenum`；归 S2b/S3 |

## 6. 单测（8 个，模块内 `#[cfg(test)]`）

`cargo test --locked --lib flex_channel` → **8 passed / 0 failed**：

- `empty_flex_channel_validates_as_zero_flex`（空 flex = `None` 等价，计数全 0）
- `single_flex_channel_validates`（单 flex：1 边 + 1 个 1D flex 图元）
- `multi_flex_multi_world_channel_validates`（多 flex / nworld=2 / 混合 `edgeid=-1` 图元）
- `world_count_mismatch_is_rejected`
- `length_mismatches_are_rejected_explicitly`（8 个字段逐个长度坏值）
- `range_and_monotonic_violations_are_rejected`（vertadr 起点/越界/回退、边端点、映射 id）
- `slot_shapes_follow_max_one_and_two_dimensional_upstream`（`max(1)` 与 2-D 口径）
- `bvh_ids_are_carried_as_little_endian_word_pairs`

## 7. 门禁（实际结果）

见 `.normify/runs/2026-11-03-render-r06-s2-flex-host-channel/{fmt,clippy,clippy-cuda,unit}.json`；
本节数字与 run json 一致：

- fmt：通过
- clippy（`--all-targets --locked -- -D warnings`）：通过
- clippy-cuda（`--features cuda`）：通过
- unit（`cargo test --locked`）：**1276 passed / 0 failed / 398 ignored**（S1a 基线 1268 → +8，只增不减）
- `git diff --stat -- src/render/kernel/cuda` 空、`fixtures/` 零改动（见 §3/§8）

## 8. 边界与非声明

- **本片不宣称** R06 能力、不宣称 flex 渲染可用、不宣称任何数值/精度验收；无 GPU 计算、无窗口、未生成参考。
- model/context 真实 flex 数据源（原生 render 字段 + io/model 接线 + flex 物理输出取顶点世界位姿）归
  **R06-S2b**（`2026-11-04-render-r06-s2b-flex-data-source`）；设备 cast/BVH 与 seg 编码归 **S3**。
- 逐 world 入口（`render_world` / `render_per_world`）本片不接 flex 通道（`None`，留 L v2/K）。
- `flex_vertnum` / `flex_dim` / `flex_edgenum` 不在 81 槽内，本通道不携带，故对应语义不校验（见 §5）。
- 证据文件为 UTF-8 LF、无 BOM；manifest 只列本目录文件（不自引用）。
