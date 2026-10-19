+++
uid = "80d3a725"
id = "mjwarp.model.spatial-tendon"
fingerprint = "4542b22ad8dd96c70f67bab56e131ec33428c0ea4e61e5ad374e09f60809232c"

[name]
zh = "空间肌腱字段子集"
en = "Spatial tendon field subset"

[description]
zh = """
检查site、滑轮与球柱路径。尺寸支持独立周期。混合入口另行映射编号。
"""
en = """
Checked spatial tendon paths with independent geometry-size batches; the mixed model maps these local rows to global indices.
"""

[[source]]
path = "src/model/spatial_tendon.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::SpatialTendonFields"

[apis.description]
zh = """
空间肌腱辅助接口。
"""
en = """
Spatial tendon helper interface.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::SpatialTendonModelInput"

[apis.description]
zh = """
空间肌腱辅助接口。
"""
en = """
Spatial tendon helper interface.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::SpatialTendonRows"

[apis.description]
zh = """
空间肌腱辅助接口。
"""
en = """
Spatial tendon helper interface.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::SpatialTendonGeometry"

[apis.description]
zh = """
球柱类型与独立尺寸周期。
"""
en = """
Geometry types and independent size periods.
"""

[[deps]]
kind = "reference"
to = "mjwarp.model.fixed-tendon"
to_api = "rpc:mjwarp_rs::model::FixedTendonModelInput"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
to_api = "rpc:mjwarp_rs::diagnostics::InputError"

[[deps]]
kind = "reference"
to = "mjwarp.model.parameters"
to_api = "rpc:mjwarp_rs::model::ParameterBatch"
+++
