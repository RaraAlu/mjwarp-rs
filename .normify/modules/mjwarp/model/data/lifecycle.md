+++
uid = "6b66bd50"
id = "mjwarp.model.data.lifecycle"
state = "planned"
fingerprint = "pending"
source = []

[name]
zh = "设备数据生命周期"
en = "Device data lifecycle"

[description]
zh = """
设备数据生命周期契约：会话所有权、身份核对与失效传播。
计划模块，不表示已有实现或最终签名。
"""
en = """
Planned device-data lifecycle contract: session ownership, identity checks and invalidation flow.
Planned module; not an implemented API or final signature.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::data::DataLifecycle"

[apis.description]
zh = """
计划中的设备数据生命周期句柄。
只表达所有权与失效契约，不表示已实现类型。
"""
en = """
Planned device-data lifecycle handle.
States ownership and invalidation contract only; not an implemented type.
"""
+++
