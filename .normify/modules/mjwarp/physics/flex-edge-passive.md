+++
uid = "5067644f"
id = "mjwarp.physics.flex-edge-passive"
tags = ["G04", "U038", "NP01", "NP02", "cuda"]
fingerprint = "ed25efa24c5d92a3feb70510fd7bb3361dc8c2631935bfadaaf62f147358e3c2"

[name]
zh = "柔体边被动力"
en = "Flex edge passive force"

[description]
zh = """
消费常驻边长、速度与雅可比，在GPU计算柔体边弹簧及阻尼广义力。
"""
en = """
Consumes resident edge length, velocity and Jacobian to compute flex-edge spring and damper generalized forces on GPU.
"""

[[source]]
path = "src/physics/flex_edge_passive.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::FlexEdgePassivePlan"

[apis.description]
zh = """
柔体边被动力执行计划。
"""
en = """
Flex-edge passive plan.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::FlexEdgePassiveInput"

[apis.description]
zh = """
柔体边被动力静态输入。
"""
en = """
Flex-edge passive input.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::FlexEdgePassiveSnapshot"

[apis.description]
zh = """
弹簧与阻尼广义力结果。
"""
en = """
Passive-force snapshot.
"""

[[deps]]
kind = "dataflow"
to = "mjwarp.physics.resident"
from_api = "rpc:mjwarp_rs::physics::FlexEdgePassivePlan"
to_api = "rpc:mjwarp_rs::physics::KinematicsPlan"

[deps.label]
zh = "读取常驻边结果"
en = "Read resident edge data"

[[deps]]
kind = "dataflow"
to = "mjwarp.physics.flex-edge"
from_api = "rpc:mjwarp_rs::physics::FlexEdgePassivePlan"
to_api = "rpc:mjwarp_rs::physics::FlexEdgeOutput"

[deps.label]
zh = "沿用边结果布局"
en = "Reuse edge result layout"

[[deps]]
kind = "reference"
to = "mjwarp.model.flex"
from_api = "rpc:mjwarp_rs::physics::FlexEdgePassiveInput"
to_api = "rpc:mjwarp_rs::model::FlexEdgeFields"

[deps.label]
zh = "读取已校验边拓扑"
en = "Read validated topology"

[[deps]]
kind = "call"
to = "mjwarp.runtime.transfer.kernel"
from_api = "rpc:mjwarp_rs::physics::FlexEdgePassivePlan"
to_api = "rpc:mjwarp_rs::runtime::transfer::kernel::SynchronousKernel"

[deps.label]
zh = "执行同步CUDA内核"
en = "Launch CUDA kernels"
+++

冻结上游被动力边分支。

模块跳过刚性边。
模块支持独立禁用弹簧与阻尼。
模块保留上游f32原子累加。
模块不代表完整被动力阶段。
