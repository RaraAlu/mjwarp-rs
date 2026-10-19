+++
uid = "541f82db"
id = "mjwarp.runtime.transfer.core"
tags = ["runtime", "transfer", "prototype", "windows-verified"]
fingerprint = "111c62d8b3453dcf191b8a531ebd8caf179c01a982c56a6ff050d7c776ae4ca2"

[name]
zh = "同步字段底层"
en = "Synchronous field primitives"

[description]
zh = """
管理类型化GPU复制。 隐藏地址与上下文。 GPU错误隔离整个会话。
"""
en = """
Own checked typed CUDA transfers with opaque context/buffer lifetimes and conservative session quarantine on backend failures.
"""

[[source]]
path = "src/runtime/transfer.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::TransferElement"

[apis.description]
zh = """
封闭支持四类标量。复制保留实际宽度与位模式。
"""
en = """
Sealed f32/f64/i32/u32 transfer element with checked byte capacity and exact bit-preserving copies; no external ABI freeze.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::TransferSession"

[apis.description]
zh = """
同步复制基础辅助已落地。 不冻结生产接口或外部ABI。
"""
en = """
Implemented owned synchronous CUDA transfer helper; no production runtime/backend/external ABI freeze.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::TransferBuffer"

[apis.description]
zh = """
同步复制基础辅助已落地。 不冻结生产接口或外部ABI。
"""
en = """
Implemented owned synchronous CUDA transfer helper; no production runtime/backend/external ABI freeze.
"""

[[deps]]
kind = "call"
to = "mjwarp.runtime.driver"
from_api = "rpc:mjwarp_rs::runtime::TransferSession"

[deps.label]
zh = "复用驱动准入"
en = "Driver admission"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::runtime::TransferBuffer"
to_api = "rpc:mjwarp_rs::diagnostics::TransferError"

[deps.label]
zh = "传输失败诊断"
en = "Transfer errors"
+++

## 实际同步传输范围

封闭类型支持f32与f64。
封闭类型也支持i32与u32。
缓冲按实际元素宽度检查容量。
复制保留各类原始位模式。
接口隐藏地址、流与上下文。
未知完成仍保留实际资源。
真实故障不注入。

证据：C:/Rust/mjwarp-rs/docs/windows-transfer-probe.md。
证据：C:/Rust/mjwarp-rs/docs/windows-mass-solve-probe.md。
