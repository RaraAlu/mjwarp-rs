+++
uid = "91bf8a42"
id = "mjwarp.physics.flex-edge"
fingerprint = "ba17a6dbe8ff265130a6df533648de812fa9410afc1f8550b33c3cf76f262aab"

[name]
zh = "柔体边子集"
en = "Flex edge subset"

[description]
zh = """
GPU计算边长、雅可比与速度。
"""
en = """
GPU edge lengths, sparse Jacobians and velocities; not full flex.
"""

[[source]]
path = "src/physics/flex_edge.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::FlexEdgeOutput"

[apis.description]
zh = """
柔体边宿主只读结果。
"""
en = """
Owned flex edge readback.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::FlexEdgeWorld"

[apis.description]
zh = """
柔体边宿主只读结果。
"""
en = """
Owned flex edge readback.
"""

[[deps]]
kind = "reference"
to = "mjwarp.model.core"
to_api = "rpc:mjwarp_rs::model::BatchLayout"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
to_api = "rpc:mjwarp_rs::diagnostics::InputError"

[[deps]]
kind = "reference"
to = "mjwarp.model.flex"
to_api = "rpc:mjwarp_rs::model::FlexEdgeFields"
+++
