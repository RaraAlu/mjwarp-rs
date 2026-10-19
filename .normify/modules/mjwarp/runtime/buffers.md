+++
uid = "60a503a6"
id = "mjwarp.runtime.buffers"
state = "planned"
fingerprint = "9ba0d09b5115aa8812429a1825f3591a9989e04455441cb538639175e1afcbdc"

[name]
zh = "内部缓冲租约"
en = "Internal buffer leases"

[description]
zh = """
管理内部缓冲与范围租约。 产品目标接口仍属计划。
"""
en = """
Own typed internal buffer ownership, checked range views, layout versions and alias leases. Production target APIs remain planned.
"""

[[source]]
path = "src/runtime/lease.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::allocate_buffer"

[apis.description]
zh = """
检查字节数与容量。返回拥有所有权的缓冲。此项属于Rust目标契约。
"""
en = """
Check sizes and capacity and return owned buffers. Planned Rust design contract; not an implemented API or frozen final signature.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::copy_buffer"

[apis.description]
zh = """
检查设备与长度。保留暂存资源至完成。此项属于Rust目标契约。
"""
en = """
Validate device and length, retaining staging resources until completion. Planned Rust design contract; not an implemented API or frozen final signature.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::run_resource_probe"

[apis.description]
zh = """
现有局部探针入口。 不提供完整引擎API。 rpc仅作契约标记。
"""
en = """
Existing working prototype entry point, not a complete engine API. RPC is an abstract contract notation, not a network service.
"""
+++
