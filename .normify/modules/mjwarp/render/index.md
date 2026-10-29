+++
uid = "a66a7856"
id = "mjwarp.render"
tags = ["planned", "engine", "render"]
fingerprint = "8cea9f4843b2b039598cf2f1184d9c7dbb6313ae790818646339ac6945d4f18c"

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
