+++
uid = "a66a7856"
id = "mjwarp.render"
tags = ["planned", "engine", "render"]
fingerprint = "58e9e2a044b104e536946fc4e6a3896c79a4d08f9bf78e8144fdafffa55f5774"

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

[[source]]
path = "src/render/bvh/mod.rs"

[[source]]
path = "src/render/bvh/aabb.rs"

[[source]]
path = "src/render/bvh/builder.rs"

[[source]]
path = "src/render/bvh/refit.rs"

[[source]]
path = "src/render/bvh/query.rs"

[[source]]
path = "src/render/bvh/cuda/mod.rs"

[[source]]
path = "src/render/bvh/cuda/scene.cu"

[[source]]
path = "src/render/bvh/mesh.rs"

[[source]]
path = "src/render/bvh/mesh_query.rs"

[[source]]
path = "src/render/bvh/cuda/mesh.cu"

[[source]]
path = "src/render/camera.rs"

[[source]]
path = "src/render/rays.rs"

[[source]]
path = "src/render/camera/cuda/build_rays.cu"

[[source]]
path = "src/render/texture.rs"

[[source]]
path = "src/render/texture/cuda.rs"

[[source]]
path = "src/render/texture/sample.cu"

[[source]]
path = "src/render/context.rs"

[[source]]
path = "src/render/context/params.rs"

[[source]]
path = "src/render/context/output.rs"

[[source]]
path = "src/render/context/model.rs"

[[source]]
path = "src/render/context/owner.rs"

[[source]]
path = "src/render/context/lifecycle.rs"

[[source]]
path = "src/render/kernel/mod.rs"

[[source]]
path = "src/render/kernel/launch.rs"

[[source]]
path = "src/render/context/camera_output.rs"

[[source]]
path = "src/render/context/world/mod.rs"

[[source]]
path = "src/render/context/world/errors.rs"

[[source]]
path = "src/render/context/world/layout.rs"

[[source]]
path = "src/render/context/world/state.rs"

[[source]]
path = "src/render/kernel/world_camera.rs"

[[source]]
path = "src/render/kernel/world_arrays.rs"

[[source]]
path = "src/render/context/lighting.rs"

[[source]]
path = "src/render/kernel/cuda/render_shadow.cu"

[[source]]
path = "src/render/context/hfield.rs"

[[source]]
path = "src/render/context/hfield_inputs.rs"

[[source]]
path = "src/render/kernel/megakernel_uploads.rs"

[[source]]
path = "src/render/kernel/cast_mesh.rs"

[[source]]
path = "src/render/bvh/hfield_mesh.rs"

[[source]]
path = "src/render/context/texture_inputs.rs"

[[source]]
path = "src/render/kernel/cast.rs"

[[source]]
path = "src/render/kernel/megakernel.rs"

[[source]]
path = "src/render/kernel/megakernel_aa.rs"

[[source]]
path = "src/render/kernel/megakernel_arrays.rs"

[[source]]
path = "src/render/kernel/output.rs"

[[source]]
path = "src/render/kernel/renderer.rs"

[[source]]
path = "src/render/kernel/cuda/render.cu"

[[source]]
path = "src/render/kernel/cuda/render_common.cu"

[[source]]
path = "src/render/kernel/cuda/render_math.cu"

[[source]]
path = "src/render/kernel/cuda/render_megakernel.cu"

[[source]]
path = "src/render/kernel/cuda/render_shade.cu"

[[source]]
path = "src/render/kernel/cuda/aa_resolve.cu"

[[source]]
path = "src/render/context/tests.rs"
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
