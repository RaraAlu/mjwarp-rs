+++
uid = "2697aa91"
id = "mjwarp.model.core"
state = "planned"
tags = ["planned", "engine"]
fingerprint = "38edc23a7750441eadfccaa325056f8c8f349df936716b7fc752e2624185dec3"

[name]
zh = "模型布局与输入契约"
en = "Model layout and input contracts"

[description]
zh = """
职责叶子登记真实实现。 完整产品契约仍待完成。
"""
en = """
Responsibility leaf with real foundation/kinematic evidence. Full production contracts remain incomplete.
"""

[[source]]
path = "src/model/mod.rs"

[[source]]
path = "src/model/collision.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::Model"

[apis.description]
zh = """
计划保留类型契约。保留冻结上游语义。 U002；M01、M02、M05；P2/P3。
"""
en = """
Planned type contract for upstream types.Model (U002). Matrix: M01、M02、M05. Phase: P2/P3. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::data::Data"

[apis.description]
zh = """
计划保留类型契约。保留冻结上游语义。 U003；M06、M07、M08；P3。
"""
en = """
Planned type contract for upstream types.Data (U003). Matrix: M06、M07、M08. Phase: P3. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::data::make_data"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U032；M06；P3。
"""
en = """
Planned local function contract for upstream io.make_data (U032). Matrix: M06. Phase: P3. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::actuator::BiasType"

[apis.description]
zh = """
计划保留类型契约。保留冻结上游语义。 U076；G05；P2/P4/P7。
"""
en = """
Planned type contract for upstream types.BiasType (U076). Matrix: G05. Phase: P2/P4/P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::collision::BroadphaseFilter"

[apis.description]
zh = """
计划保留类型契约。保留冻结上游语义。 U077；G07；P2/P5。
"""
en = """
Planned type contract for upstream types.BroadphaseFilter (U077). Matrix: G07. Phase: P2/P5. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::collision::BroadphaseType"

[apis.description]
zh = """
计划保留类型契约。保留冻结上游语义。 U078；G07；P2/P5。
"""
en = """
Planned type contract for upstream types.BroadphaseType (U078). Matrix: G07. Phase: P2/P5. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::constraint::ConeType"

[apis.description]
zh = """
计划保留类型契约。保留冻结上游语义。 U080；G13；P2/P6。
"""
en = """
Planned type contract for upstream types.ConeType (U080). Matrix: G13. Phase: P2/P6. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::data::Constraint"

[apis.description]
zh = """
计划保留类型契约。保留冻结上游语义。 U081；G12；P3/P6。
"""
en = """
Planned type contract for upstream types.Constraint (U081). Matrix: G12. Phase: P3/P6. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::data::Contact"

[apis.description]
zh = """
计划保留类型契约。保留冻结上游语义。 U082；G08～G11；P3/P5/P7。
"""
en = """
Planned type contract for upstream types.Contact (U082). Matrix: G08～G11. Phase: P3/P5/P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::actuator::CtrlChart"

[apis.description]
zh = """
计划保留类型契约。保留冻结上游语义。 U083；G05、G24；P2/P7。
"""
en = """
Planned type contract for upstream types.CtrlChart (U083). Matrix: G05、G24. Phase: P2/P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::actuator::CtrlInput"

[apis.description]
zh = """
计划保留类型契约。保留冻结上游语义。 U084；G05、G24；P2/P7。
"""
en = """
Planned type contract for upstream types.CtrlInput (U084). Matrix: G05、G24. Phase: P2/P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::options::DisableBit"

[apis.description]
zh = """
计划保留类型契约。保留冻结上游语义。 U085；G01～G28；P2/P7。
"""
en = """
Planned type contract for upstream types.DisableBit (U085). Matrix: G01～G28. Phase: P2/P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::actuator::DynType"

[apis.description]
zh = """
计划保留类型契约。保留冻结上游语义。 U086；G05；P2/P4/P7。
"""
en = """
Planned type contract for upstream types.DynType (U086). Matrix: G05. Phase: P2/P4/P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::options::EnableBit"

[apis.description]
zh = """
计划保留类型契约。保留冻结上游语义。 U087；G20、G25、G26；P2/P7。
"""
en = """
Planned type contract for upstream types.EnableBit (U087). Matrix: G20、G25、G26. Phase: P2/P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::actuator::GainType"

[apis.description]
zh = """
计划保留类型契约。保留冻结上游语义。 U088；G05；P2/P4/P7。
"""
en = """
Planned type contract for upstream types.GainType (U088). Matrix: G05. Phase: P2/P4/P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::collision::GeomType"

[apis.description]
zh = """
计划保留类型契约。保留冻结上游语义。 U089；G08～G11、R05；P2/P5/P8。
"""
en = """
Planned type contract for upstream types.GeomType (U089). Matrix: G08～G11、R05. Phase: P2/P5/P8. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::options::IntegratorType"

[apis.description]
zh = """
计划保留类型契约。保留冻结上游语义。 U090；G16～G18；P2/P6。
"""
en = """
Planned type contract for upstream types.IntegratorType (U090). Matrix: G16～G18. Phase: P2/P6. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::index::ObjType"

[apis.description]
zh = """
计划保留类型契约。保留冻结上游语义。 U092；M01、M02、G23；P2/P7。
"""
en = """
Planned type contract for upstream types.ObjType (U092). Matrix: M01、M02、G23. Phase: P2/P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::options::Option"

[apis.description]
zh = """
计划保留类型契约。保留冻结上游语义。 U093；G07、G14～G18、M11；P2/P6。
"""
en = """
Planned type contract for upstream types.Option (U093). Matrix: G07、G14～G18、M11. Phase: P2/P6. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::options::SolverType"

[apis.description]
zh = """
计划保留类型契约。保留冻结上游语义。 U096；G14、G15；P2/P6。
"""
en = """
Planned type contract for upstream types.SolverType (U096). Matrix: G14、G15. Phase: P2/P6. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::Statistic"

[apis.description]
zh = """
计划保留类型契约。保留冻结上游语义。 U098；M02；P2。
"""
en = """
Planned type contract for upstream types.Statistic (U098). Matrix: M02. Phase: P2. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::actuator::TrnType"

[apis.description]
zh = """
计划保留类型契约。保留冻结上游语义。 U099；G05、G06；P2/P4/P7。
"""
en = """
Planned type contract for upstream types.TrnType (U099). Matrix: G05、G06. Phase: P2/P4/P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::HostModelView"

[apis.description]
zh = """
借用已编译模型。检查原生版本与所有者。此项属于Rust目标契约。
"""
en = """
Borrow a compiled model with native-version and owner checks. Planned Rust design contract; not an implemented API or frozen final signature.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::ModelInput"

[apis.description]
zh = """
持有已检查的字段与资产。不保留悬空原生指针。此项属于Rust目标契约。
"""
en = """
Own validated fields and assets without dangling native pointers. Planned Rust design contract; not an implemented API or frozen final signature.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::BatchLayout"

[apis.description]
zh = """
基础辅助接口已有源码。 检查连续批量字段范围。 不代表产品模块完成。
"""
en = """
Implemented foundation helper. Checked dense batched host field layout; not Model, native ABI or GPU allocation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::NativeModelInfo"

[apis.description]
zh = """
探针DTO检查候选ABI。 计数使用64位有符号数。 完整Model仍待实现。
"""
en = """
Implemented probe-only DTO and count checks for candidate MuJoCo 3.12.0; not the full Model layout.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::PrimitiveGeometryFields"

[apis.description]
zh = """
已实现平面与球十三字段宿主小原型。原生f64/i32转换为f32/i32。不提供FFI捕获、上传或GPU碰撞。完整Model契约仍待实现。
"""
en = """
Implemented plane/sphere host-only thirteen-field prototype, with native f64/i32 to f32/i32 conversion; no FFI capture, upload, or GPU collision; full Model contract remains planned.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::PrimitiveGeometryInput"

[apis.description]
zh = """
已实现平面与球十三字段宿主小原型。原生f64/i32转换为f32/i32。不提供FFI捕获、上传或GPU碰撞。完整Model契约仍待实现。
"""
en = """
Implemented plane/sphere host-only thirteen-field prototype, with native f64/i32 to f32/i32 conversion; no FFI capture, upload, or GPU collision; full Model contract remains planned.
"""

[[deps]]
kind = "call"
to = "mjwarp.runtime.buffers"
from_api = "rpc:mjwarp_rs::model::data::make_data"
to_api = "rpc:mjwarp_rs::runtime::allocate_buffer"

[deps.label]
zh = "持有设备状态缓冲"
en = "Owned device buffers"

[[deps]]
kind = "reference"
to = "mjwarp.math"
from_api = "rpc:mjwarp_rs::model::Model"
to_api = "rpc:mjwarp_rs::math::mul_quat"

[deps.label]
zh = "定义公共数学布局"
en = "Math layout"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::model::data::Data"
to_api = "rpc:mjwarp_rs::diagnostics::Error"

[deps.label]
zh = "声明布局及容量错误"
en = "Layout errors"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::model::BatchLayout"
to_api = "rpc:mjwarp_rs::diagnostics::InputError"

[deps.label]
zh = "基础辅助的真实依赖"
en = "Foundation helper"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::model::NativeModelInfo"
to_api = "rpc:mjwarp_rs::diagnostics::NativeProbeError"

[deps.label]
zh = "原生探针布局错误"
en = "Native admission helper"
+++

2026-10-14：沿P0允许的小原型补平面/球宿主13字段输入。保留原UID及模块状态。无设备上传/物理执行/原生指针捕获；完整P2及G08未验收。
