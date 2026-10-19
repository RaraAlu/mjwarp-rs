+++
uid = "8e52a332"
id = "mjwarp.runtime.completion"
state = "planned"
fingerprint = "2a32c574be86aa6e9664d468125fab80e84f1d567302205da03a42b16d0d76f8"

[name]
zh = "完成队列"
en = "Completion queue"

[description]
zh = """
保留在途所有者与完成状态。 产品目标接口仍属计划。
"""
en = """
Own completion tokens, in-flight owners, event retirement and conservative failure quarantine. Production target APIs remain planned.
"""

[[source]]
path = "src/runtime/completion.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::submit"

[apis.description]
zh = """
检查在途资源租约。返回提交及完成状态。此项属于Rust目标契约。
"""
en = """
Validate in-flight leases and return submission completion state. Planned Rust design contract; not an implemented API or frozen final signature.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::wait"

[apis.description]
zh = """
等待提交完成。报告设备或异步失败。此项属于Rust目标契约。
"""
en = """
Wait for completion and report device or asynchronous failure. Planned Rust design contract; not an implemented API or frozen final signature.
"""

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::runtime::submit"
to_api = "rpc:mjwarp_rs::diagnostics::Error"

[deps.label]
zh = "声明设备与异步错误"
en = "Async device errors"
+++
