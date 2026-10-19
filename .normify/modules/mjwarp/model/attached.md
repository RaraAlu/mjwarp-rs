+++
uid = "a6d23429"
id = "mjwarp.model.attached"
fingerprint = "3df23ed6812933e7912dd9a082e1b171034fb9f521577b3bb15f4beb7e5a0ecd"

[name]
zh = "附着字段检查"
en = "Attached frame fields"

[description]
zh = """
检查几何与site字段。严格与等价构造分离。
"""
en = """
Checked attached fields with separate strict and equivalent constructors.
"""

[[source]]
path = "src/model/attached.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::AttachedFields"

[apis.description]
zh = """
几何与site辅助接口。
"""
en = """
Probe-only attached frame contract.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::AttachedModelInput"

[apis.description]
zh = """
几何与site辅助接口。
"""
en = """
Probe-only attached frame contract.
"""

[[deps]]
kind = "call"
to = "mjwarp.model.inertial"
to_api = "rpc:mjwarp_rs::model::InertialModelInput"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
to_api = "rpc:mjwarp_rs::diagnostics::InputError"
+++
