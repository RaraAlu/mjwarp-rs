+++
uid = "824ac735"
id = "mjwarp.model.topology"
state = "planned"
tags = ["planned", "engine"]
fingerprint = "658583fe0a50b1faf93cd45b508dce01ac3dab88ea9b8389e33f81fb9e6ae21b"

[name]
zh = "运动学字段与拓扑"
en = "Kinematic fields and topology"

[description]
zh = """
职责叶子登记真实实现。 完整产品契约仍待完成。
"""
en = """
Responsibility leaf with real foundation/kinematic evidence. Full production contracts remain incomplete.
"""

[[source]]
path = "src/model/topology.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::topology::JointType"

[apis.description]
zh = """
四种关节值已有源码。 坐标宽度匹配原生布局。 U091与G01物理验收仍待完成。
"""
en = """
Implemented four numeric joint tags and qpos/dof widths for U091. G01 physics and complete equivalence remain unverified; no production completion claim.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::KinematicFields"

[apis.description]
zh = """
十二字段的未检查输入。 浮点使用连续f32布局。
"""
en = """
Owned raw twelve-field kinematic input, flattened f32/i32 arrays. Shared parameter batch one; not full ModelInput.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::KinematicModelInput"

[apis.description]
zh = """
校验拓扑、地址与有限值。 验证后仅导出只读借用。
"""
en = """
Immutable validated twelve-field kinematic subset with parent ordering, ownership, joint partitions, exact coordinate widths and finite-value checks; not full ModelInput.
"""

[[deps]]
kind = "call"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::model::KinematicModelInput"
to_api = "rpc:mjwarp_rs::diagnostics::InputError"

[deps.label]
zh = "报告拓扑输入错误"
en = "Implemented kinematic subset"
+++
