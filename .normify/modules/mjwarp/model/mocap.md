+++
uid = "8e723dab"
id = "mjwarp.model.mocap"
fingerprint = "397c76e4e546fd7fcfda4a3372e087ab01b19c236963dd97ee404a305db45f80"

[name]
zh = "mocap拓扑检查"
en = "Mocap topology checks"

[description]
zh = """
检查共享mocap映射。推导静态geom掩码。仍限G01辅助子集。
"""
en = """
Checked shared mocap mappings and static geometry mask for the G01 helper subset.
"""

[[source]]
path = "src/model/mocap.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::MocapModelInput"

[apis.description]
zh = """
检查mocap模型子集。
"""
en = """
Checked mocap model subset.
"""

[[deps]]
kind = "reference"
to = "mjwarp.model.attached"
to_api = "rpc:mjwarp_rs::model::AttachedModelInput"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
to_api = "rpc:mjwarp_rs::diagnostics::InputError"
+++
