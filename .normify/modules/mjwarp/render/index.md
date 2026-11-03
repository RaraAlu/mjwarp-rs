+++
uid = "a66a7856"
id = "mjwarp.render"
tags = ["planned", "engine", "render"]
fingerprint = "8a41ca99799be71419fe79ca309edb983d3d1cdfb88c2c59e392bd35c6cc4a75"

[name]
zh = "GPU批量渲染"
en = "Batched GPU rendering"

[description]
zh = """
提供无头批量渲染。维护渲染BVH。输出像素与完成状态。
"""
en = """
Plan headless batched rendering, rendering BVHs and image outputs without changing dynamic physics state.
"""

[[source]]
path = "src/render/mod.rs"
+++

## 基础源码落地

2026-10-07：补齐基础行为。
ImageBatchLayout检查单相机布局。
深度与RGB各占四字节。
分割ID与类型共占八字节。
禁用输出拒绝访问。
单相机布局不冻结相机列表。
BVH与GPU批量渲染仍待实现。
本轮不激活产品模块。

证据：C:/Rust/mjwarp-rs/docs/foundation-architecture.md。

## A2 布局归属

2026-10-22：渲染布局改依赖设备数据定义。
ImageBatchLayout只持有设备数据布局。
批量布局宿主类型已迁入model::data。
旧model::BatchLayout路径保留兼容。
本归属不改变渲染功能状态。
BVH与GPU批量渲染仍待实现。

证据：C:/Rust/mjwarp-rs/docs/development-evidence/changes/2026-10-18-layers-a2.json。
## 模块拆分落地核对（2026-11-03，Arch）

本卡 acceptance 已由 2026-10-25 的两笔架构提交落地并核对：

- `7de14d30` render split 步 1（3 新叶 + 11 API/6 依赖原文迁移，源留容器）
- `e98595ec` render split 步 2（53 源归位：kernel 22 / bvh 11 / context 19，容器仅留 `src/render/mod.rs` 门面）

核对（只读）：

- `mjwarp.render` 容器 `index.md` 自身 `[[apis]]` = 0（不再直接声明 API）；
- 子模块持有原 API：`bvh` 2 条、`context` 8 条、`kernel` 1 条 `[[apis]]` 条目；
- `mjwarp.render.kernel` 叶子已登记 22 个 source，含 `src/render/kernel/{mod,launch,cast,output,megakernel*}.rs` 与全部渲染 `.cu`；
- `normify validate` = **错误 0 / 警告 5**，均为本卡之前既有基线（`mjwarp.io.render`、`mjwarp.model.render-model` 两条 leaf-empty，`structure/leaf-too-coarse`、两条 layout/order），**无 render 新增警告**；
- 不改产品代码/测试/夹具/已验收证据；本轮仅补核对记录。
