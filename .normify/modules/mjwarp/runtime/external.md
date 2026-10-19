+++
uid = "bc1e046f"
id = "mjwarp.runtime.external"
state = "planned"
fingerprint = "16fe28fb1793023856de80b6c92bffebddf82a3f34b1a5d7d438a25056de23aa"

[name]
zh = "外部资源导入"
en = "External resource import"

[description]
zh = """
检查外部描述符与重复导入。 产品目标接口仍属计划。
"""
en = """
Own fixed-width prototype descriptors and import identity/weak registration. CUDA adapter evidence lives in the runtime container. Production target APIs remain planned.
"""

[[source]]
path = "src/runtime/external.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::device_view"

[apis.description]
zh = """
声明布局与读写权限。关联所有者和完成事件。此项属于Rust目标契约。
"""
en = """
Expose typed, leased layouts with owners and completion events. Planned Rust design contract; not an implemented API or frozen final signature.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::run_external_resource_probe"

[apis.description]
zh = """
现有局部探针入口。 不提供完整引擎API。 rpc仅作契约标记。
"""
en = """
Existing working prototype entry point, not a complete engine API. RPC is an abstract contract notation, not a network service.
"""
+++
