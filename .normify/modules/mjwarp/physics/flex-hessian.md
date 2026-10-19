+++
uid = "b6e07c32"
id = "mjwarp.physics.flex-hessian"
fingerprint = "c64918c0f7ed0b5fe3d799dd5892029af8981ebff403485f74bc39844b591588"

[name]
zh = "柔体海森矩阵子集"
en = "Flex Hessian subset"

[description]
zh = """
GPU缓存直接柔体拉伸矩阵。仅支持21系数布局。
"""
en = """
GPU cached direct-flex stretch Hessians with 21 coefficients; not the full elasticity stage.
"""

[[source]]
path = "src/physics/flex_hessian.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::FlexHessianOutput"

[apis.description]
zh = """
海森矩阵宿主快照。
"""
en = """
Owned Hessian snapshot.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::FlexHessianWorld"

[apis.description]
zh = """
顶点与边的矩阵视图。
"""
en = """
World vertex and edge Hessian blocks.
"""

[[deps]]
kind = "reference"
to = "mjwarp.model.flex"
to_api = "rpc:mjwarp_rs::model::FlexHessianFields"

[deps.label]
zh = "矩阵字段"
en = "Hessian fields"

[[deps]]
kind = "reference"
to = "mjwarp.model.flex"
to_api = "rpc:mjwarp_rs::model::FlexEdgeFields"

[deps.label]
zh = "边映射"
en = "Edge mappings"

[[deps]]
kind = "reference"
to = "mjwarp.model.core"
to_api = "rpc:mjwarp_rs::model::BatchLayout"

[deps.label]
zh = "批量布局"
en = "Batch layout"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
to_api = "rpc:mjwarp_rs::diagnostics::InputError"

[deps.label]
zh = "输入检查"
en = "Input checks"
+++
