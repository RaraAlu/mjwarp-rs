# R06-S3 前置：`FlexHostInputs` 组装与 `render_with_flex` 真实调用方接线（宿主侧）

任务：`2026-11-04-render-r06-s3pre-flexhost-wiring`。日期：2026-11-04（空闲段）。
**零 GPU 计算、零窗口、零参考生成**；`src/render/kernel/cuda/` 与全部 `.cu` 零改动；不新增 ABI 槽。

## 1. 交付物

| 文件 | 行数 | bytes | 作用 |
| --- | --- | --- | --- |
| `src/render/context/flex_host.rs` | 487 | 17566 | `FlexHostData`（调用方 data 切片入参）、`FlexHostAssembly::build/inputs`、`FlexWiringError`、6 个单测 |
| `src/render/context.rs` | 459 | 16410 | 只加 `#[cfg(feature = "cuda-probe")] pub mod flex_host;` 与 `pub use flex_host::{FlexHostAssembly, FlexHostData, FlexWiringError};` |

接线下游（S3 前置）调用形态：

```rust
let assembly = FlexHostAssembly::build(
    nworld,
    &positions,          // FlexPositionFields（flex_vertadr/vertnum/vertbodyid）
    &edges,              // FlexEdgeFields（flex_edge）
    &render_fields,      // FlexRenderFields（dim/radius/rgba/edgeadr/edgenum）
    FlexHostData { flexvert_xpos, flex_bvh_ids, flex_group_root },
)?;
match &assembly {
    Some(assembly) => renderer.render_with_flex(context, fields, model, poses, rays, Some(&assembly.inputs()))?,
    None => renderer.render(context, fields, model, poses, rays)?,          // 无 flex：既有路径
}
```

## 2. 语义与边界

- **无 flex**（`FlexRenderFields::nflex == 0`）→ `build` 返回 `Ok(None)`，调用方走既有 `render()`；
  **有 flex** → `Ok(Some)` + `inputs()`；两条路径都只是宿主实参装配，设备侧零改动。
- **依赖边界**：policy 禁 `mjwarp.render → mjwarp.physics`，故组装层**不接收** physics 的 `FlexRenderView`；
  调用方把 S2b 只读视图的 `flexvert_xpos` 切片和 S3 前置的 `flex_bvh_ids` / `flex_group_root` 缓冲经
  `FlexHostData` 传入。`render → model` 允许，三份 model 输入直传。
- S3 前置阶段 `flex_bvh_ids` 由调用方给 0、`flex_group_root` 给 -1 占位（S3 本体填真值，本片不构造 BVH）。
- 校验失败返回 `FlexWiringError`（`Length` / `Count` / `Model` / `Channel`），**不 panic、不静默回退**。

## 3. 三层校验

1. `FlexRenderFields::validate()`（S2b）：dim∈1..3、radius 非负有限、rgba 有限、edgeadr 连续且总量正确。
2. 本层同源/拓扑（`check_sources`）：`flex_vertadr` 长度=nflex、`flex_vertbodyid` 长度=nflexvert、
   `flex_edge` 长度=2×nflexedge；`flex_vertadr` 链首项 0 且逐 flex 累加 = `nflexvert`；每条边的局部顶点号
   < 对应 `flex_vertnum`；`flexvert_xpos`=3×nworld×nflexvert、`flex_bvh_ids`=nflex、`flex_group_root`=nworld×nflex。
3. `FlexHostInputs::validate(nworld)`（S2 通道）：长度/世界数/顶点范围/geom 映射第二层复核。

## 4. 单测（6 个，`flex-host-unit-tests.txt` 原始输出）

`cargo test --locked --lib flex_host` → **6 passed / 0 failed**：

- `no_flex_returns_none_deterministically`（无 flex → `None`，两次调用一致）
- `single_flex_assembles_full_channel`（9 数组/计数/映射逐项落位 + `validate` 通过）
- `multi_flex_multi_world_assembles_mapping`（多 flex/nworld=2；1D 每边一图元、2D 每 flex 一图元）
- `length_mismatches_are_rejected_explicitly`（xpos/bvh_id/group_root 三处长度拒绝）
- `topology_mismatches_are_rejected_explicitly`（顶点链断裂、边端点越界拒绝）
- `assembly_is_bit_identical_across_calls`（同输入两次组装逐位一致）

## 5. 「无 flex 位级不变」的三条机器可判口径（Chief 批准口径）

1. `render_with_flex(None)` 与 `render()` 等价（S2 实现：`render` 内部即 `render_impl(…, flex: None)`；本片不改 kernel）；
2. `git diff` 不含 `src/render/kernel/cuda/`、任何 `.cu`、`fixtures/`（本片 diff 只含上述 2 文件 + 卡 + 证据）；
3. 单测 `no_flex_returns_none_deterministically`（`None` + 两次一致）与 `assembly_is_bit_identical_across_calls`。
   **不加 GPU 级两次运行位级证据**（留 S3 窗口）。

## 6. 门禁

fmt / clippy / clippy-cuda / unit（记录在 `.normify/runs/2026-11-04-render-r06-s3pre-flexhost-wiring/`）；
unit 只增不减；`unchanged` = `src/render/kernel/cuda/` 与 `fixtures/` 零改动。

## 7. 非声明

- S3 本体（设备 flex BVH、`render_flex.cu`、参考采集、验证窗口）不在本片；本片不碰设备面、不产生参考。
- 不宣称 flex 渲染可用、不做任何数值/精度结论；零 GPU、零窗口。
- 未做：真实 BVH/group-root 构造、设备 cast/seg、GPU 级位级回归（留 S3）。
