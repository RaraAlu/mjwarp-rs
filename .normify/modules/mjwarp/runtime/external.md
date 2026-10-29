+++
uid = "bc1e046f"
id = "mjwarp.runtime.external"
fingerprint = "72f001da95c07d6fd2b713e2e07215d99a74f0684b3245ab8eedb0e85910c730"

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

[[source]]
path = "src/runtime/external/view.rs"

[[source]]
path = "src/runtime/external/view/tests.rs"

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
