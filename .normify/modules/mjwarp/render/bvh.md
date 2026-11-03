+++
uid = "fcd8ff58"
id = "mjwarp.render.bvh"
tags = ["engine", "render", "bvh", "gpu"]
fingerprint = "e7d56c6ce3c78eba2408a073b2a40cb4764f272f0a541b1a239797dc0aa61e18"

[name]
zh = "渲染BVH与网格查询"
en = "Render BVH and mesh queries"

[description]
zh = "渲染BVH与网格查询"
en = "Render BVH and mesh queries"

[[source]]
path = "src/render/bvh/aabb.rs"

[[source]]
path = "src/render/bvh/builder.rs"

[[source]]
path = "src/render/bvh/cuda/mesh.cu"

[[source]]
path = "src/render/bvh/cuda/mod.rs"

[[source]]
path = "src/render/bvh/cuda/scene.cu"

[[source]]
path = "src/render/bvh/hfield_mesh.rs"

[[source]]
path = "src/render/bvh/mesh.rs"

[[source]]
path = "src/render/bvh/mesh_query.rs"

[[source]]
path = "src/render/bvh/mod.rs"

[[source]]
path = "src/render/bvh/query.rs"

[[source]]
path = "src/render/bvh/refit.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::render::bvh::refit_bvh"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U004；R01、R05、R06；P8。
"""
en = """
Planned local function contract for upstream bvh.refit_bvh (U004). Matrix: R01、R05、R06. Phase: P8. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::render::bvh::refit_splat_bvh"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U005；R01、R07；P8。
"""
en = """
Planned local function contract for upstream bvh.refit_splat_bvh (U005). Matrix: R01、R07. Phase: P8. Not implemented; RPC is only an abstract contract notation.
"""
+++
