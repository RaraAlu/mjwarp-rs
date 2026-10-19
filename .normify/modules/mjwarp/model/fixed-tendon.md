+++
uid = "a654e982"
id = "mjwarp.model.fixed-tendon"
fingerprint = "91852c20872076f589bcf48272dad318f2fa0e061838615e99802215cddb70da"

[name]
zh = "固定肌腱字段"
en = "Fixed tendon fields"

[description]
zh = """
检查关节型固定肌腱。保留已检查的原生CSR。拓扑与系数共享。
"""
en = """
Checked joint-only fixed tendons with validated native CSR rows and shared topology/coefficient fields.
"""

[[source]]
path = "src/model/fixed_tendon.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::FixedTendonFields"

[apis.description]
zh = """
固定肌腱辅助接口。
"""
en = """
Fixed tendon helper interface.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::FixedTendonModelInput"

[apis.description]
zh = """
固定肌腱辅助接口。
"""
en = """
Fixed tendon helper interface.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::FixedTendonRows"

[apis.description]
zh = """
固定肌腱辅助接口。
"""
en = """
Fixed tendon helper interface.
"""

[[deps]]
kind = "reference"
to = "mjwarp.model.camlight"
to_api = "rpc:mjwarp_rs::model::CamLightModelInput"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
to_api = "rpc:mjwarp_rs::diagnostics::InputError"
+++
