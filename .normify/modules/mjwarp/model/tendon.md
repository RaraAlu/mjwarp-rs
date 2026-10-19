+++
uid = "8b6221e7"
id = "mjwarp.model.tendon"
fingerprint = "1c03494a78485e2391ae19e12b2bfa4853e74457cdd87117bba9ad06354bb3cd"

[name]
zh = "混合肌腱编号"
en = "Mixed tendon numbering"

[description]
zh = """
检查原生混合字段。保持全局CSR与子集映射。
"""
en = """
Validate mixed tendon fields with global CSR and subset mapping.
"""

[[source]]
path = "src/model/tendon.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::TendonFields"

[apis.description]
zh = """
混合肌腱辅助接口。
"""
en = """
Mixed tendon helper interface.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::TendonModelInput"

[apis.description]
zh = """
混合肌腱辅助接口。
"""
en = """
Mixed tendon helper interface.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::TendonRows"

[apis.description]
zh = """
混合肌腱辅助接口。
"""
en = """
Mixed tendon helper interface.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::TendonSubset"

[apis.description]
zh = """
混合肌腱辅助接口。
"""
en = """
Mixed tendon helper interface.
"""

[[deps]]
kind = "reference"
to = "mjwarp.model.spatial-tendon"
from_api = "rpc:mjwarp_rs::model::TendonModelInput"
to_api = "rpc:mjwarp_rs::model::SpatialTendonModelInput"

[[deps]]
kind = "reference"
to = "mjwarp.model.fixed-tendon"
from_api = "rpc:mjwarp_rs::model::TendonModelInput"
to_api = "rpc:mjwarp_rs::model::FixedTendonModelInput"

[[deps]]
kind = "reference"
to = "mjwarp.model.camlight"
from_api = "rpc:mjwarp_rs::model::TendonModelInput"
to_api = "rpc:mjwarp_rs::model::CamLightModelInput"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::model::TendonModelInput"
to_api = "rpc:mjwarp_rs::diagnostics::InputError"
+++
