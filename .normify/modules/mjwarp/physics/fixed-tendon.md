+++
uid = "a394e90c"
id = "mjwarp.physics.fixed-tendon"
fingerprint = "3b900adca0166c3f7344238f5d97964ccabce933d434e0154fd176faf11bcec4"

[name]
zh = "固定肌腱GPU结果"
en = "Fixed tendon GPU output"

[description]
zh = """
GPU计算长度与稀疏力臂。仍限G01辅助子集。
"""
en = """
GPU lengths and sparse moments for the G01 fixed tendon helper.
"""

[[source]]
path = "src/physics/fixed_tendon.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::FixedTendonOutput"

[apis.description]
zh = """
固定肌腱只读结果。
"""
en = """
Read-only fixed tendon results.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::FixedTendonWorld"

[apis.description]
zh = """
固定肌腱只读结果。
"""
en = """
Read-only fixed tendon results.
"""

[[deps]]
kind = "reference"
to = "mjwarp.model.fixed-tendon"
to_api = "rpc:mjwarp_rs::model::FixedTendonRows"

[[deps]]
kind = "reference"
to = "mjwarp.model.core"
to_api = "rpc:mjwarp_rs::model::BatchLayout"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
to_api = "rpc:mjwarp_rs::diagnostics::InputError"
+++
