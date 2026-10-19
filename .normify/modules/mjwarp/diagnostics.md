+++
uid = "1dd6441d"
id = "mjwarp.diagnostics"
state = "planned"
tags = ["planned", "engine", "diagnostics"]
fingerprint = "e2f923fba63ec28f50773dffbadcb4e6250008ddded4bdb3fe21c0596c8c13fa"

[name]
zh = "诊断与有效性"
en = "Diagnostics and validity"

[description]
zh = """
定义错误与有效性。区分警告、截断和迭代。保留原始类别。
"""
en = """
Plan errors and output validity, preserving warnings, truncation, iteration limits and original categories.
"""

[[source]]
path = "src/diagnostics/mod.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::diagnostics::OverflowType"

[apis.description]
zh = """
计划保留类型契约。保留冻结上游语义。 U094；M11；P3/P7。
"""
en = """
Planned type contract for upstream types.OverflowType (U094). Matrix: M11. Phase: P3/P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::diagnostics::Error"

[apis.description]
zh = """
区分输入与异步错误。不保证异步状态回滚。此项属于Rust目标契约。
"""
en = """
Separate input and asynchronous errors without promising asynchronous rollback. Planned Rust design contract; not an implemented API or frozen final signature.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::diagnostics::Diagnostics"

[apis.description]
zh = """
保留原始告警与计数。不隐式重置失败状态。此项属于Rust目标契约。
"""
en = """
Preserve warnings and counts without implicit state repair. Planned Rust design contract; not an implemented API or frozen final signature.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::diagnostics::PhysicsReport"

[apis.description]
zh = """
报告物理输出有效性。区分容量和迭代上限。此项属于Rust目标契约。
"""
en = """
Report physics output validity, capacity and iteration status separately. Planned Rust design contract; not an implemented API or frozen final signature.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::diagnostics::RenderReport"

[apis.description]
zh = """
报告图像输出有效性。保留像素与通道诊断。此项属于Rust目标契约。
"""
en = """
Report image output validity with pixel and channel diagnostics. Planned Rust design contract; not an implemented API or frozen final signature.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::diagnostics::InputError"

[apis.description]
zh = """
基础辅助接口已有源码。 保留基础输入错误详情。 不代表产品模块完成。
"""
en = """
Implemented foundation helper. Structured errors for foundation helpers; not the planned production physics/render reports.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::diagnostics::TransferError"

[apis.description]
zh = """
基础传输辅助已有源码。 区分输入、驱动与隔离错误。
"""
en = """
Structured foundation transfer error, preserving original inputs and driver failures; not a production report.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::diagnostics::NativeProbeError"

[apis.description]
zh = """
登记原生版本与准入错误。 不承诺恶意输入隔离。
"""
en = """
Implemented native probe errors for ABI, version, path, MJB header, allocation and bridge stages; not hostile-input isolation.
"""
+++

## P1局部实现

ProbeError报告探针错误。
整数错误保留完整u32值。
求解错误报告解或残差。
校验拒绝非有限结果。
InvalidGraph报告图布局错误。
图更新失败保留CUDA阶段与码。
物理与渲染诊断仍未实现。
五项原有API仍属于计划。
本轮不激活该模块。

证据：C:/Rust/mjwarp-rs/docs/windows-kernel-probe.md。

## 资源失败诊断

ResourceError保留具体类别。
范围错误记录偏移与容量。
布局错误记录实际与期望版本。
区分别名、权限与上下文错误。
完成失败保留首次阶段与码。
清理成功不抹去失败状态。
隔离拒绝资源继续复用。
锁中毒与编号耗尽明确报错。
仅NOT_READY表示事件待定。
本轮未注入真实设备故障。
产品恢复策略仍待设计。
物理与渲染诊断仍未实现。
本轮不激活该模块。

证据：C:/Rust/mjwarp-rs/docs/windows-resource-probe.md。

## 产物诊断原型

Artifact保留阶段与错误详情。
区分缺失、摘要、ABI与布局错误。
NvrtcPresent拒绝伪装仅驱动运行。
损坏缓存不隐式修复或换路。
JSON摘要不证明PTX代码安全。
物理与渲染诊断继续保持计划。
本轮不激活该模块。

证据：C:/Rust/mjwarp-rs/docs/windows-artifact-probe.md。

## 外部资源诊断原型

拒绝非法描述符与布局。
拒绝不支持的外部内存类型。
重复导入报告独立错误类别。
驱动错误仍保留阶段与原始码。
生产安全门面仍待设计。
本轮不激活该模块。

证据：C:/Rust/mjwarp-rs/docs/windows-external-probe.md。

## 基础源码落地

2026-10-07：补齐基础行为。
InputError保留字段与索引。
容量与非有限输入明确报错。
ProbeError保持原有类别。
生产物理与渲染报告仍待实现。
本轮不激活产品模块。

证据：C:/Rust/mjwarp-rs/docs/foundation-architecture.md。

## Windows同步字段传输

保留输入与CUDA阶段错误。
无法证明完成时保守保留资源。
仅驱动子进程另作烟测。
真实GPU故障不注入。
故障完成策略采用宿主模拟。
本轮继续保持计划态。

证据：C:/Rust/mjwarp-rs/docs/windows-transfer-probe.md。

## 原生模型输入探针

候选版本采用MuJoCo 3.12.0。
Windows探针暂用MSVC。
四项字段已有实际行为。
正式输入契约仍待冻结。
本轮保留产品计划态。

证据：C:/Rust/mjwarp-rs/docs/windows-native-model-probe.md。

## 运动学输入诊断

InvalidTopology保留字段索引。
具体原因区分父体与地址错误。
浮点转换错误保留字段名。
复制前拒绝全量指针错误。
完整物理报告仍待实现。

证据：C:/Rust/mjwarp-rs/docs/windows-kinematic-fields.md。

## 惯性输入诊断

NegativeValue记录字段与索引。
微小原生负参数不能借下溢通过。
桥接错误保留copy_inertial阶段。
产品报告继续保持计划态。

证据：C:/Rust/mjwarp-rs/docs/windows-inertial-fields.md。

## 求解主元诊断

InvalidPivot保留世界与自由度。
错误保留实际f64主元值。
倒数溢出记录字段与全局索引。
主元失败不隔离健康CUDA会话。
完整物理报告继续保持计划。

证据：C:/Rust/mjwarp-rs/docs/windows-mass-solve-probe.md。
