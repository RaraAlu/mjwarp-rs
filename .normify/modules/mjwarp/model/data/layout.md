+++
uid = "36c788cb"
id = "mjwarp.model.data.layout"
fingerprint = "f8bd29758d2beb50d875d8b465ba5a2ed3c2b7158ecdfe99fba11a7ae478083e"

[name]
zh = "设备数据布局"
en = "Device data layout"

[description]
zh = """
设备批量布局契约：世界数、每世界元素数与元素字节。受保护缓冲首尾各四槽，精确跨度为八槽加世界批量长度。宿主类型已实现。
"""
en = """
Device batch layout contract: worlds, elements per world and element bytes. Protected buffers keep four guard slots per end; exact span is eight slots plus the world batch. Host types implemented.
"""

[[source]]
path = "src/model/data/layout.rs"

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
