+++
uid = "545c0310"
id = "mjwarp.runtime.driver"
state = "planned"
fingerprint = "3bea86aac98c06320b14e8ff6c89b02ee03806253b783772dff38bac349344aa"

[name]
zh = "驱动与执行图"
en = "Driver and graphs"

[description]
zh = """
管理驱动、内核与执行图。 产品目标接口仍属计划。
"""
en = """
Own CUDA driver initialization, native kernel execution, graph capture, replay and node updates. Production target APIs remain planned.
"""

[[source]]
path = "src/runtime/cuda.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::device"

[apis.description]
zh = """
检查设备与上下文。不支持时明确报错。此项属于Rust目标契约。
"""
en = """
Check device and context capabilities without CPU fallback. Planned Rust design contract; not an implemented API or frozen final signature.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::capture_graph"

[apis.description]
zh = """
先预热并分配工作区。捕获期禁止编译和扩容。此项属于Rust目标契约。
"""
en = """
Warm up and allocate before capture; forbid capture-time compilation and growth. Planned Rust design contract; not an implemented API or frozen final signature.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::replay_graph"

[apis.description]
zh = """
检查布局、地址与配置。拒绝失效图。此项属于Rust目标契约。
"""
en = """
Validate layouts, addresses and configuration; reject stale graphs. Planned Rust design contract; not an implemented API or frozen final signature.
"""
+++
