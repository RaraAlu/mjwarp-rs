+++
uid = "c8f4a621"
id = "mjwarp.physics.flex"
fingerprint = "3172b45127798fc15c915f69c986a70b1d5174c8144648b95352ebf66aa27ae8"

[name]
zh = "柔体位置子集"
en = "Flex position subset"

[description]
zh = """
GPU更新柔体位置与缓存标志。不提供Hessian计算。
"""
en = """
GPU flex positions and Hessian validity invalidation, without a Hessian producer.
"""

[[source]]
path = "src/physics/flex.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::FlexPositionOutput"

[apis.description]
zh = """
位置与布尔标志宿主快照。
"""
en = """
Owned positions and checked bool validity snapshot.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::FlexPositionWorld"

[apis.description]
zh = """
只读位置与Hessian标志。
"""
en = """
Read-only world positions and Hessian validity.
"""

[[deps]]
kind = "reference"
to = "mjwarp.model.core"
to_api = "rpc:mjwarp_rs::model::BatchLayout"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
to_api = "rpc:mjwarp_rs::diagnostics::InputError"
+++
