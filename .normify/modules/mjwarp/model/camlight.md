+++
uid = "c97e614a"
id = "mjwarp.model.camlight"
fingerprint = "260e80291d90d2040f61471b361c712bedfea2367b0ca00f9f2854effe336141"

[name]
zh = "相机光源字段"
en = "Camera light fields"

[description]
zh = """
检查共享拓扑与独立参数。等价构造保留有限非单位姿态。
"""
en = """
Checked camera/light topology and parameters with finite nonunit equivalent construction.
"""

[[source]]
path = "src/model/camlight.rs"

[[source]]
path = "src/model/camlight/model.rs"

[[source]]
path = "src/model/camlight/parameters.rs"

[[source]]
path = "src/model/camlight/tests.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::CamLightModelInput"

[apis.description]
zh = """
相机与光源辅助接口。
"""
en = """
Camera and light helper interface.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::CamLightParameters"

[apis.description]
zh = """
相机与光源辅助接口。
"""
en = """
Camera and light helper interface.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::CamLightFields"

[apis.description]
zh = """
相机与光源辅助接口。
"""
en = """
Camera and light helper interface.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::CamLightParameter"

[apis.description]
zh = """
相机与光源辅助接口。
"""
en = """
Camera and light helper interface.
"""

[[deps]]
kind = "reference"
to = "mjwarp.model.mocap"
to_api = "rpc:mjwarp_rs::model::MocapModelInput"

[[deps]]
kind = "reference"
to = "mjwarp.model.parameters"
to_api = "rpc:mjwarp_rs::model::ParameterBatch"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
to_api = "rpc:mjwarp_rs::diagnostics::InputError"
+++
