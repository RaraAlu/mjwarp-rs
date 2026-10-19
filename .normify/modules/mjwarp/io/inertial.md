+++
uid = "b18de452"
id = "mjwarp.io.inertial"
fingerprint = "f8717deae5160925726ee157f869764ae4ea8046ef9253bef11357d643a59a84"

[name]
zh = "惯性字段转换与上传"
en = "Inertial conversion and upload"

[description]
zh = """
独占原生惯性快照。同步上传只读字段组。
"""
en = """
Owned inertial native snapshot and synchronous read-only 21-field GPU group; not full put_model or model identity.
"""

[[source]]
path = "src/io/inertial.rs"

[[source]]
path = "src/io/inertial/sparse.rs"

[[source]]
path = "include/mjwarp_sparse_probe.h"

[[source]]
path = "native/model_probe_sparse.cpp"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::io::NativeInertialSnapshot"

[apis.description]
zh = """
独占原生惯性与运动学字段。严格转换有限标量。
"""
en = """
Owned native 21-field subset and strict f64 conversion.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::io::DeviceInertialModel"

[apis.description]
zh = """
上传完成后返回字段组。同步回读并重新校验。
"""
en = """
Synchronous complete-group upload, retained session ownership and validated readback.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::io::NativeSparseMassSnapshot"

[apis.description]
zh = """
独占六项原生i32 CSR字段并严格转换。不运行物理或GPU。
"""
en = """
Owned six-field native i32 CSR capture and checked conversion; no physics or GPU execution.
"""

[[deps]]
kind = "call"
to = "mjwarp.model.inertial"
from_api = "rpc:mjwarp_rs::io::NativeInertialSnapshot"
to_api = "rpc:mjwarp_rs::model::InertialModelInput"

[deps.label]
zh = "转换并校验惯性"
en = "Inertial subset helper"

[[deps]]
kind = "call"
to = "mjwarp.io.kinematic"
from_api = "rpc:mjwarp_rs::io::NativeInertialSnapshot"
to_api = "rpc:mjwarp_rs::io::NativeKinematicSnapshot"

[deps.label]
zh = "组合运动学快照"
en = "Inertial subset helper"

[[deps]]
kind = "call"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::io::NativeInertialSnapshot"
to_api = "rpc:mjwarp_rs::diagnostics::NativeProbeError"

[deps.label]
zh = "保留桥接错误"
en = "Inertial subset helper"

[[deps]]
kind = "call"
to = "mjwarp.model.inertial"
from_api = "rpc:mjwarp_rs::io::DeviceInertialModel"
to_api = "rpc:mjwarp_rs::model::InertialModelInput"

[deps.label]
zh = "借用惯性输入"
en = "Inertial subset helper"

[[deps]]
kind = "call"
to = "mjwarp.io.kinematic"
from_api = "rpc:mjwarp_rs::io::DeviceInertialModel"
to_api = "rpc:mjwarp_rs::io::DeviceKinematicModel"

[deps.label]
zh = "组合GPU字段"
en = "Inertial subset helper"

[[deps]]
kind = "call"
to = "mjwarp.runtime.transfer.core"
from_api = "rpc:mjwarp_rs::io::DeviceInertialModel"
to_api = "rpc:mjwarp_rs::runtime::TransferSession"

[deps.label]
zh = "同步上传与回读"
en = "Inertial subset helper"

[[deps]]
kind = "call"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::io::DeviceInertialModel"
to_api = "rpc:mjwarp_rs::diagnostics::TransferError"

[deps.label]
zh = "保留传输错误"
en = "Inertial subset helper"

[[deps]]
kind = "call"
to = "mjwarp.io.fields"
from_api = "rpc:mjwarp_rs::io::NativeInertialSnapshot"
to_api = "rpc:mjwarp_rs::io::convert_f64_to_f32_into"

[deps.label]
zh = "复用严格浮点转换"
en = "Inertial subset helper"

[[deps]]
kind = "call"
to = "mjwarp.io.fields"
from_api = "rpc:mjwarp_rs::io::DeviceInertialModel"
to_api = "rpc:mjwarp_rs::io::DeviceBatch"

[deps.label]
zh = "复用连续字段传输"
en = "Inertial subset helper"

[[deps]]
kind = "call"
to = "mjwarp.io.fields"
from_api = "rpc:mjwarp_rs::io::NativeInertialSnapshot"
to_api = "rpc:mjwarp_rs::io::fields::native_staging"

[deps.label]
zh = "分配原生字段暂存"
en = "Allocate native staging"

[[deps]]
kind = "call"
to = "mjwarp.model.inertial"
from_api = "rpc:mjwarp_rs::io::NativeSparseMassSnapshot"
to_api = "rpc:mjwarp_rs::model::SparseMassInput"
+++

## 2026-10-14 原生CSR六字段

独立32/80字节schema，不改原96字节ABI。
复制六项i32，原G01编号及78字段保持。
不可变原生拥有者保留DLL，Vec快照独占。
全部指针与计数预检后才复制，零长跳过。
Rust严格检查长度再转换原CSR校验器。
两新用途保守各2/3，首轮路径准备失败。
第二次三过0.20秒，C++1190断言。
六模型330整数，最大nv13，不证大块原生捕获。
整数极值字节相等只证明ABI，不证明合法拓扑。
无物理/GPU/Python/作者与旧目标执行。
旧源码身份保持，新构建入口漂移单列。
证据：C:/Rust/mjwarp-rs/docs/native-sparse-mass.md。

## 本轮局部验收

本叶子仅登记窄辅助契约。
二十一字段取得Windows证据。
测试覆盖正常与失败边界。
完整产品模块继续保持计划。
本叶子不声明物理等价。
本叶子不声明Linux验收。

证据：C:/Rust/mjwarp-rs/docs/windows-inertial-fields.md。
