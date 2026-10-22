+++
uid = "f0f12206"
id = "mjwarp.model.data.batch"
fingerprint = "013dec0cd6e7d59624f3caf4b29237866eecee0d7117b971d487420b75aa3cc3"

[name]
zh = "设备数据批量"
en = "Device data batching"

[description]
zh = """
设备数据批量语义：世界批量下标、共享参数周期与连续设备字段。参数按世界序号对周期取模，拓扑与映射只存一份。宿主与设备字段类型已实现。
"""
en = """
Device-data batching semantics: world indexes, shared parameter periods and contiguous device fields. Parameters sample by world modulo period; topology is stored once. Host and device field types implemented.
"""

[[source]]
path = "src/model/data/batch.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::data::WorldBatch"

[apis.description]
zh = """
已实现的世界批量下标：检查世界数并拒绝越界序号。
"""
en = """
Implemented world batch: validates world count and rejects out-of-range indexes.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::data::FieldBatch"

[apis.description]
zh = """
已实现的共享参数周期：批量下标等于世界序号对周期取模。
"""
en = """
Implemented shared parameter period: batch index equals world modulo period.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::data::DeviceBatch"

[apis.description]
zh = """
设备批量字段：由`model::data`持有，`io::DeviceBatch`保留为公开兼容转发路径。
"""
en = """
Device batch field owned by model::data; io::DeviceBatch remains a public compatibility forwarding path.
"""
+++

## 批量契约

世界批量独立推进，单世界可局部写入。
字段批量按周期共享参数，下标等于世界序号对周期取模。
拓扑与映射只存一份，不随世界复制。
设备批量保留其上传会话；跨会话复制一律拒绝。
