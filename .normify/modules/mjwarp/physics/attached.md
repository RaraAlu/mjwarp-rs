+++
uid = "90a67df6"
id = "mjwarp.physics.attached"
fingerprint = "942ee2dac799361ed28a97bba1b017e7557b74e884443311e71d09ffa82ea877"

[name]
zh = "附着位姿GPU探针"
en = "Attached frame GPU probe"

[description]
zh = """
仅实现几何与site辅助子集。不替代完整G01阶段。
"""
en = """
Probe-only geometry and site transforms; not a complete G01 stage.
"""

[[source]]
path = "src/physics/attached.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::probe_attached_kinematics"

[apis.description]
zh = """
几何与site辅助接口。
"""
en = """
Probe-only attached frame contract.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::AttachedKinematicsOutput"

[apis.description]
zh = """
几何与site辅助接口。
"""
en = """
Probe-only attached frame contract.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::AttachedKinematicsWorld"

[apis.description]
zh = """
几何与site辅助接口。
"""
en = """
Probe-only attached frame contract.
"""

[[deps]]
kind = "reference"
to = "mjwarp.model.attached"
to_api = "rpc:mjwarp_rs::model::AttachedModelInput"

[[deps]]
kind = "reference"
to = "mjwarp.physics.core"
from_api = "rpc:mjwarp_rs::physics::AttachedKinematicsOutput"
to_api = "rpc:mjwarp_rs::physics::KinematicsOutput"

[deps.label]
zh = "持有同步刚体结果"
en = "Own completed rigid result"

[[deps]]
kind = "call"
to = "mjwarp.runtime.transfer.kernel"
to_api = "rpc:mjwarp_rs::runtime::transfer::kernel::SynchronousKernel"

[[deps]]
kind = "call"
to = "mjwarp.runtime.transfer.core"
to_api = "rpc:mjwarp_rs::runtime::TransferSession"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
to_api = "rpc:mjwarp_rs::diagnostics::TransferError"

[[deps]]
kind = "call"
to = "mjwarp.model.core"
to_api = "rpc:mjwarp_rs::model::BatchLayout"
+++

## 本轮源码复审

已核对旧指纹后的两处改动。
刚体辅助不再选择标量模板。
设备模型固定保存f32参数。
附着缓冲与CUDA接口仍为f32。
几何与site输出契约保持不变。
本叶子不替代完整G01阶段。
