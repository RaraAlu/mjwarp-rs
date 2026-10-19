+++
uid = "a4fa0d47"
id = "mjwarp.physics.tendon"
fingerprint = "6e84ed485555e06d72e5901d56cc8691791d82b6da33a170fdb04fc05cc6eefa"

[name]
zh = "全局肌腱输出"
en = "Global tendon output"

[description]
zh = """
GPU合并两类肌腱。原生顺序输出长度与包裹。
"""
en = """
GPU assembly of tendon subsets into native-order outputs.
"""

[[source]]
path = "src/physics/tendon.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::TendonOutput"

[apis.description]
zh = """
全局肌腱结果接口。
"""
en = """
Global tendon output interface.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::TendonWorld"

[apis.description]
zh = """
全局肌腱结果接口。
"""
en = """
Global tendon output interface.
"""

[[deps]]
kind = "reference"
to = "mjwarp.model.tendon"
from_api = "rpc:mjwarp_rs::physics::TendonOutput"
to_api = "rpc:mjwarp_rs::model::TendonRows"

[[deps]]
kind = "reference"
to = "mjwarp.model.core"
from_api = "rpc:mjwarp_rs::physics::TendonOutput"
to_api = "rpc:mjwarp_rs::model::BatchLayout"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::physics::TendonOutput"
to_api = "rpc:mjwarp_rs::diagnostics::InputError"
+++
