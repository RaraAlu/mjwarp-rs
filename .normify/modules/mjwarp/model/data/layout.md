+++
uid = "36c788cb"
id = "mjwarp.model.data.layout"
fingerprint = "f90686154ec5136e39871fefbf14a5fbd0e70f4740e549e3fa9fc22c52b11b68"

[name]
zh = "设备数据布局"
en = "Device data layout"

[description]
zh = """
设备批量布局契约：连续批量布局、世界数、每世界元素数与元素字节。受保护缓冲首尾各四槽，精确跨度为八槽加世界批量长度。宿主类型已实现。
"""
en = """
Device batch layout contract: contiguous batch layout, worlds, elements per world and element bytes. Protected buffers keep four guard slots per end; exact span is eight slots plus the world batch. Host types implemented.
"""

[[source]]
path = "src/model/data/layout.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::data::BatchLayout"

[apis.description]
zh = """
已实现的连续批量布局；A2起归属设备数据定义，`model::BatchLayout`保留兼容导出。
"""
en = """
Implemented contiguous batch layout; owned by device data since A2, with model::BatchLayout kept as compatibility export.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::data::DeviceDataLayout"

[apis.description]
zh = """
已实现的布局描述符：精确守卫跨度、单世界区间与宽输出最小长度校验。
"""
en = """
Implemented layout descriptor: exact guarded span, per-world range and minimum-length checks for wide outputs.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::data::GUARD_SLOTS"

[apis.description]
zh = """
首尾守卫槽总数：每端四槽，合计八槽。
"""
en = """
Total guard slots: four per end, eight in total.
"""
+++

## 布局契约

`DeviceDataLayout`复用已检查的`BatchLayout`。
它只描述布局，不分配设备缓冲。
守卫缓冲要求精确跨度，不足与超出都拒绝。
结果缓冲可宽于最小写入宽度，按最小长度校对。
世界数、元素数与元素字节的检查在构造期完成。

## A2 归属

`BatchLayout`已迁入本模块。
`model::BatchLayout`保留兼容导出，旧路径不删除。
渲染布局只依赖本模块的设备布局契约。
本归属不改变批量渲染功能状态。
