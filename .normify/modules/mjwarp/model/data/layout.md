+++
uid = "36c788cb"
id = "mjwarp.model.data.layout"
state = "planned"
fingerprint = "pending"
source = []

[name]
zh = "设备数据布局"
en = "Device data layout"

[description]
zh = """
设备数据布局契约：批量布局、守卫槽与精确跨度。
计划模块，不表示已有实现或最终签名。
"""
en = """
Planned device-data layout contract: batch layouts, guard slots and exact spans.
Planned module; not an implemented API or final signature.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::data::DeviceDataLayout"

[apis.description]
zh = """
计划中的设备数据布局描述符。
只表达布局与容量契约，不表示已实现类型。
"""
en = """
Planned device-data layout descriptor.
States the layout and capacity contract only; not an implemented type.
"""
+++
