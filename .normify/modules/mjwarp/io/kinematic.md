+++
uid = "c7e40532"
id = "mjwarp.io.kinematic"
state = "planned"
tags = ["planned", "engine"]
fingerprint = "9fd187755a7933902f2ddab453134c15b77034e4df6efb74a3d8836b5a831a10"

[name]
zh = "运动学字段转换与上传"
en = "Kinematic conversion and upload"

[description]
zh = """
职责叶子登记真实实现。 完整产品契约仍待完成。
"""
en = """
Responsibility leaf with real foundation/kinematic evidence. Full production contracts remain incomplete.
"""

[[source]]
path = "src/io/kinematic.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::io::NativeKinematicSnapshot"

[apis.description]
zh = """
独占十二项原生字段。 严格转换后校验运动学输入。
"""
en = """
Owned Windows native twelve-field snapshot and strict f64 conversion; independent native owner lifetime. Not full compiled-model admission.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::io::DeviceKinematicModel"

[apis.description]
zh = """
同步上传十二字段组。 隐藏可写缓冲与设备地址。
"""
en = """
Owned read-only GPU group with synchronous completion before publication and validated readback. Shared parameters only; not full Model or put_model.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::io::kinematic::native::convert"

[apis.description]
zh = """
内部严格标量转换。按字段报告非有限值与溢出。
"""
en = """
Internal feature-gated f64 conversion helper; preserve field-labelled finite and overflow errors.
"""

[[deps]]
kind = "call"
to = "mjwarp.model.topology"
from_api = "rpc:mjwarp_rs::io::DeviceKinematicModel"
to_api = "rpc:mjwarp_rs::model::KinematicModelInput"

[deps.label]
zh = "借用已校验字段"
en = "Implemented kinematic subset"

[[deps]]
kind = "call"
to = "mjwarp.runtime.transfer.core"
from_api = "rpc:mjwarp_rs::io::DeviceKinematicModel"
to_api = "rpc:mjwarp_rs::runtime::TransferSession"

[deps.label]
zh = "执行同步GPU传输"
en = "Implemented kinematic subset"

[[deps]]
kind = "call"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::io::DeviceKinematicModel"
to_api = "rpc:mjwarp_rs::diagnostics::TransferError"

[deps.label]
zh = "保留传输失败"
en = "Implemented kinematic subset"

[[deps]]
kind = "call"
to = "mjwarp.model.core"
from_api = "rpc:mjwarp_rs::io::NativeKinematicSnapshot"
to_api = "rpc:mjwarp_rs::model::NativeModelInfo"

[deps.label]
zh = "检查原生计数"
en = "Implemented kinematic subset"

[[deps]]
kind = "call"
to = "mjwarp.model.topology"
from_api = "rpc:mjwarp_rs::io::NativeKinematicSnapshot"
to_api = "rpc:mjwarp_rs::model::KinematicModelInput"

[deps.label]
zh = "严格转换与校验"
en = "Implemented kinematic subset"

[[deps]]
kind = "call"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::io::NativeKinematicSnapshot"
to_api = "rpc:mjwarp_rs::diagnostics::NativeProbeError"

[deps.label]
zh = "保留桥接失败"
en = "Implemented kinematic subset"

[[deps]]
kind = "call"
to = "mjwarp.io.fields"
from_api = "rpc:mjwarp_rs::io::DeviceKinematicModel"
to_api = "rpc:mjwarp_rs::io::DeviceBatch"

[deps.label]
zh = "复用连续GPU字段"
en = "Reuse dense GPU fields"

[[deps]]
kind = "call"
to = "mjwarp.io.fields"
from_api = "rpc:mjwarp_rs::io::NativeKinematicSnapshot"
to_api = "rpc:mjwarp_rs::io::convert_f64_to_f32_into"

[deps.label]
zh = "复用严格浮点转换"
en = "Reuse strict conversion"

[[deps]]
kind = "call"
to = "mjwarp.io.fields"
from_api = "rpc:mjwarp_rs::io::NativeKinematicSnapshot"
to_api = "rpc:mjwarp_rs::io::fields::native_staging"

[deps.label]
zh = "分配原生字段暂存"
en = "Allocate native staging"
+++
