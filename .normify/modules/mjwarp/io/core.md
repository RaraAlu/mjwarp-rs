+++
uid = "2ab8f906"
id = "mjwarp.io.core"
state = "planned"
tags = ["planned", "engine"]
fingerprint = "9c4c377c18d4fabfe79f9bf35ad6e2992f1122dec8e095e1d8aa4c932befb7ab"

[name]
zh = "原生准入与字段交换"
en = "Native admission and field exchange"

[description]
zh = """
管理可信原生模型与快照。支持MSVC与Linux GNU。完整产品契约仍待完成。
"""
en = """
Own trusted native models and snapshots on Windows MSVC and Linux GNU; full production IO contracts remain planned.
"""

[[source]]
path = "src/io/mod.rs"

[[source]]
path = "src/io/collision.rs"

[[source]]
path = "src/io/collision_native.rs"

[[source]]
path = "include/mjwarp_collision_probe.h"

[[source]]
path = "native/model_probe_collision.cpp"

[[source]]
path = "src/io/collision_parameters.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::io::state::get_data_into"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U031；M07；P3/P9。
"""
en = """
Planned local function contract for upstream io.get_data_into (U031). Matrix: M07. Phase: P3/P9. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::io::state::put_data"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U033；M07；P3。
"""
en = """
Planned local function contract for upstream io.put_data (U033). Matrix: M07. Phase: P3. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::io::model::put_model"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U034；M01～M05；P2/P3。
"""
en = """
Planned local function contract for upstream io.put_model (U034). Matrix: M01～M05. Phase: P2/P3. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::io::state::reset_data"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U035；M08；P3/P7。
"""
en = """
Planned local function contract for upstream io.reset_data (U035). Matrix: M08. Phase: P3/P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::io::state::reset_data_keyframe"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U036；M08；P3/P7。
"""
en = """
Planned local function contract for upstream io.reset_data_keyframe (U036). Matrix: M08. Phase: P3/P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::io::state::get_state"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U071；M07；P3/P9。
"""
en = """
Planned local function contract for upstream support.get_state (U071). Matrix: M07. Phase: P3/P9. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::io::state::set_state"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U074；M07；P3/P9。
"""
en = """
Planned local function contract for upstream support.set_state (U074). Matrix: M07. Phase: P3/P9. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::io::state::State"

[apis.description]
zh = """
计划保留类型契约。保留冻结上游语义。 U097；M07；P3/P9。
"""
en = """
Planned type contract for upstream types.State (U097). Matrix: M07. Phase: P3/P9. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::io::NativeModelProbe"

[apis.description]
zh = """
读取可信已编译MJB。独占原生模型与动态库。不替代put_model。
"""
en = """
Optional trusted x86_64 MJB admission on Windows MSVC and Linux GNU, with version/symbol checks and owned model/library lifetime. No MJCF compilation or complete put_model.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::io::NativeCoreSnapshot"

[apis.description]
zh = """
独占四项代表字段。 原生资源释放后仍有效。 完整ModelInput仍待实现。
"""
en = """
Implemented owned snapshot of qpos0, body_mass, body_parentid and jnt_type. Not a complete ModelInput or GPU model.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::io::NativePrimitiveGeometrySnapshot"

[apis.description]
zh = """
已实现平面与球十三字段宿主小原型。原生f64/i32转换为f32/i32。不提供FFI捕获、上传或GPU碰撞。完整Model契约仍待实现。
"""
en = """
Implemented plane/sphere host-only thirteen-field prototype, with native f64/i32 to f32/i32 conversion; no FFI capture, upload, or GPU collision; full Model contract remains planned.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::io::NativeModelProbe::primitive_geometry_snapshot"

[apis.description]
zh = """
已实现十三项原生几何字段独占捕获。复用3.12模型拥有者。复制i32/f64，不调用CPU物理或GPU。完整P2仍未完成。
"""
en = """
Implemented owned thirteen-field geometry capture from the existing 3.12 owner; i32/f64 copying only, with no physics or GPU execution; full P2 remains incomplete.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::io::NativePrimitiveGeometrySnapshot::into_host_batch"

[apis.description]
zh = """
先校验原生ABI与一次f32量化，再绑定独占宿主周期；不捕获、上传或计算物理。
"""
en = """
Validate native ABI and quantize once, then bind owned host parameter cycles; no capture, upload, or physics.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::io::NativeModelProbe::sparse_mass_snapshot"

[apis.description]
zh = """
独立六项CSR整数快照。完整模型转换仍待实现。
"""
en = """
Capture an owned six-field CSR integer snapshot; no complete put_model admission.
"""

[[deps]]
kind = "reference"
to = "mjwarp.model.core"
from_api = "rpc:mjwarp_rs::io::model::put_model"
to_api = "rpc:mjwarp_rs::model::Model"

[deps.label]
zh = "引用模型与输入布局"
en = "Model layout"

[[deps]]
kind = "call"
to = "mjwarp.physics.core"
from_api = "rpc:mjwarp_rs::io::state::reset_data"
to_api = "rpc:mjwarp_rs::physics::history::reset_history"

[deps.label]
zh = "组织物理历史重置"
en = "History reset"

[[deps]]
kind = "call"
to = "mjwarp.runtime.buffers"
from_api = "rpc:mjwarp_rs::io::model::put_model"
to_api = "rpc:mjwarp_rs::runtime::allocate_buffer"

[deps.label]
zh = "转换并上传设备缓冲"
en = "Device upload"

[[deps]]
kind = "reference"
to = "mjwarp.math"
from_api = "rpc:mjwarp_rs::io::model::put_model"
to_api = "rpc:mjwarp_rs::math::mul_quat"

[deps.label]
zh = "沿用数学转换约定"
en = "Math conventions"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::io::state::get_data_into"
to_api = "rpc:mjwarp_rs::diagnostics::Diagnostics"

[deps.label]
zh = "报告截断与输入诊断"
en = "Export diagnostics"

[[deps]]
kind = "reference"
to = "mjwarp.model.core"
from_api = "rpc:mjwarp_rs::io::NativeModelProbe"
to_api = "rpc:mjwarp_rs::model::NativeModelInfo"

[deps.label]
zh = "原生小型DTO"
en = "Native admission helper"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::io::NativeModelProbe"
to_api = "rpc:mjwarp_rs::diagnostics::NativeProbeError"

[deps.label]
zh = "原生准入错误"
en = "Native admission helper"

[[deps]]
kind = "call"
to = "mjwarp.io.kinematic"
from_api = "rpc:mjwarp_rs::io::NativeModelProbe"
to_api = "rpc:mjwarp_rs::io::NativeKinematicSnapshot"

[deps.label]
zh = "复制运动学快照"
en = "Inertial subset helper"

[[deps]]
kind = "call"
to = "mjwarp.io.inertial"
from_api = "rpc:mjwarp_rs::io::NativeModelProbe"
to_api = "rpc:mjwarp_rs::io::NativeInertialSnapshot"

[deps.label]
zh = "复制惯性快照"
en = "Inertial subset helper"

[[deps]]
kind = "call"
to = "mjwarp.io.fields"
from_api = "rpc:mjwarp_rs::io::NativeModelProbe"
to_api = "rpc:mjwarp_rs::io::fields::native_staging"

[deps.label]
zh = "分配原生字段暂存"
en = "Native staging allocation"

[[deps]]
kind = "reference"
to = "mjwarp.model.parameters"
from_api = "rpc:mjwarp_rs::io::NativePrimitiveGeometrySnapshot::into_host_batch"
to_api = "rpc:mjwarp_rs::model::PrimitiveGeometryHostBatch"

[deps.label]
zh = "绑定宿主独立周期"
en = "Bind independent host cycles"
+++

## 本轮源码复审

已核对原生路径与FFI增量。
Windows继续传入UTF-16。
Linux保留原始路径字节。
两类路径均要求绝对路径。
路径拒绝嵌入空字节。
模型先释放，再卸载原生库。
G01快照独占78项读字段。
本轮保留八项计划产品契约。
快照不替代完整put_model。
本轮不激活完整IO模块。


2026-10-14：沿P0允许的小原型补平面/球宿主13字段输入。保留原UID及模块状态。无设备上传/物理执行/原生指针捕获；完整P2及G08未验收。


2026-10-14补独立120字节目标DTO和13字段复制入口。旧78字段源与ABI保留；原生编译入口改扩展单元，旧完整构建验收不转授新二进制。保留原UID与planned。
