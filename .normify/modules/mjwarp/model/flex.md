+++
uid = "cf104c21"
id = "mjwarp.model.flex"
fingerprint = "840d1dad3a29882bbb5d1ef02d7c408baf228b57eda67f81e0273b633602dde1"

[name]
zh = "柔体运动学输入"
en = "Flex kinematics input"

[description]
zh = """
检查位置、边、面与拉伸矩阵。自动派生线性壳体面。
"""
en = """
Checked position, edge, face and stretch fields with canonical linear shell face derivation.
"""

[[source]]
path = "src/model/flex.rs"

[[source]]
path = "src/model/flex_hessian.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::FlexPositionFields"

[apis.description]
zh = """
共享柔体位置字段。
"""
en = """
Shared flex position fields.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::FlexPositionModelInput"

[apis.description]
zh = """
检查柔体位置输入。
"""
en = """
Checked flex position input.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::FlexEdgeFields"

[apis.description]
zh = """
共享边与稀疏行字段。
"""
en = """
Shared edge topology and native sparse rows.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::FlexFaceFields"

[apis.description]
zh = """
检查完整线性壳体面映射。
"""
en = """
Canonical complete linear shell face mapping.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::FlexHessianFields"

[apis.description]
zh = """
共享21系数与单元映射。
"""
en = """
Shared 21-coefficient material and element mappings.
"""

[[deps]]
kind = "reference"
to = "mjwarp.model.tendon"
to_api = "rpc:mjwarp_rs::model::TendonModelInput"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
to_api = "rpc:mjwarp_rs::diagnostics::InputError"

[[deps]]
kind = "reference"
to = "mjwarp.model.inertial"
to_api = "rpc:mjwarp_rs::model::InertialModelInput"
+++
