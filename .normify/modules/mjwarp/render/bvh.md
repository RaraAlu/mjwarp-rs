+++
uid = "fcd8ff58"
id = "mjwarp.render.bvh"
tags = ["engine", "render", "bvh", "gpu"]
fingerprint = "pending"
source = []

[name]
zh = "渲染BVH与网格查询"
en = "Render BVH and mesh queries"

[description]
zh = "渲染BVH与网格查询"
en = "Render BVH and mesh queries"

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
