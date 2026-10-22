+++
uid = "2cf3e9e5"
id = "mjwarp.model.data"
fingerprint = "154cd981d5fd09ccf12df5e30d9de02209922bd238470dfae5ea8fcf97273e33"

[name]
zh = "设备数据"
en = "Device data"

[description]
zh = """
按层属于计算物理层的设备数据归属根。目录位置不变，已实现布局、批量与生命周期三个叶子契约。设备批量字段归此处，io保留兼容转发。
"""
en = """
Compute-physics-layer home for device data ownership. The directory stays in place; layout, batching and lifecycle leaf contracts are implemented. Device batch fields live here; io keeps compatibility forwarding.
"""

[[source]]
path = "src/model/data/mod.rs"
+++

## 归属边界

本层只新增共享契约类型，不整体搬迁目录。
物理路径使用本层身份与生命周期记录。
输入适配仍归数据交互层，外部原生ABI与内核容差不变。
旧公开路径保留，兼容层不复制设备资源。
