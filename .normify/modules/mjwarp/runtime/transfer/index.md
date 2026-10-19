+++
uid = "2d98835f"
id = "mjwarp.runtime.transfer"
tags = ["runtime", "transfer", "prototype", "windows-verified"]
fingerprint = "c4d7881e9a05b3a3cbe9703f374a777bef584caba2b5684f31780d0abf09f835"

[name]
zh = "同步传输与内核"
en = "Synchronous transfers and kernels"

[description]
zh = """
管理内部同步GPU执行。 分别封装复制与内核ABI。
"""
en = """
Own internal synchronous GPU transfers and unsafe typed kernel execution. Children separate field ownership/completion from kernel compilation and launch.
"""

[[source]]
path = "src/runtime/transfer.rs"

[[source]]
path = "docs/windows-transfer-probe.md"
+++

## 已实现职责

容器组织同步复制与固定内核ABI。
core管理四类标量与缓冲所有权。
kernel管理可信内核与同步提交。
支持f32、f64、i32与u32复制。
零长度字段不申请设备内存。
成功返回前确认复制完成。
未知完成时保留真实资源。
模块不解释模型或物理字段。
接口不暴露裸地址与上下文。
本轮不注入真实设备故障。
生产运行时继续保持计划。
双平台与清洁部署仍待验收。

证据：C:/Rust/mjwarp-rs/docs/windows-transfer-probe.md。
证据：C:/Rust/mjwarp-rs/docs/windows-mass-solve-probe.md。
