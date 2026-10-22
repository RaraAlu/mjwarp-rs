+++
uid = "f0f12206"
id = "mjwarp.model.data.batch"
state = "planned"
fingerprint = "pending"
source = []

[name]
zh = "设备数据批量"
en = "Device data batching"

[description]
zh = """
设备数据批量语义：世界批量、字段批量与共享参数周期。
计划模块，不表示已有实现或最终签名。
"""
en = """
Planned device-data batching semantics: world batches, field batches and shared parameter periods.
Planned module; not an implemented API or final signature.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::data::WorldBatch"

[apis.description]
zh = """
计划中的世界批量视图。
只表达批量索引与共享周期契约，不表示已实现类型。
"""
en = """
Planned world-batch view.
States batching indexes and shared-period contract only; not an implemented type.
"""
+++
