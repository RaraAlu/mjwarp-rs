+++
uid = "b1ed0f18"
id = "mjwarp.physics.resident"
fingerprint = "83196453707eae8d5355900ed7e74989c751687bec33abbac4f5b251140d68a2"

[name]
zh = "常驻运动学与G01"
en = "Resident kinematics and G01"

[description]
zh = """
实现G01全链与低层入口。数值先对照MuJoCo。超差后对照MJWarp。
"""
en = """
Complete G01 chain and raw entries. Native MuJoCo is the first numerical reference; frozen MJWarp is second after native tolerance failure.
"""

[[source]]
path = "src/physics/resident.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::KinematicsPlan"

[apis.description]
zh = """
常驻运动学辅助接口。
"""
en = """
Resident strict-subset kinematics helper.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::KinematicsData"

[apis.description]
zh = """
常驻运动学辅助接口。
"""
en = """
Resident strict-subset kinematics helper.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::KinematicsSnapshot"

[apis.description]
zh = """
常驻运动学辅助接口。
"""
en = """
Resident strict-subset kinematics helper.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::G01State"

[apis.description]
zh = """
G01等价状态或阶段入口。
"""
en = """
G01 equivalent state or stage entry.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::G01SleepState"

[apis.description]
zh = """
G01等价状态或阶段入口。
"""
en = """
G01 equivalent state or stage entry.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::fwd_kinematics"

[apis.description]
zh = """
G01等价状态或阶段入口。
"""
en = """
G01 equivalent state or stage entry.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::kinematics"

[apis.description]
zh = """
G01等价状态或阶段入口。
"""
en = """
G01 equivalent state or stage entry.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::com_pos"

[apis.description]
zh = """
G01等价状态或阶段入口。
"""
en = """
G01 equivalent state or stage entry.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::camlight"

[apis.description]
zh = """
G01等价状态或阶段入口。
"""
en = """
G01 equivalent state or stage entry.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::flex"

[apis.description]
zh = """
G01等价状态或阶段入口。
"""
en = """
G01 equivalent state or stage entry.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::tendon"

[apis.description]
zh = """
G01等价状态或阶段入口。
"""
en = """
G01 equivalent state or stage entry.
"""

[[deps]]
kind = "reference"
to = "mjwarp.model.attached"
to_api = "rpc:mjwarp_rs::model::AttachedModelInput"

[[deps]]
kind = "reference"
to = "mjwarp.physics.core"
to_api = "rpc:mjwarp_rs::physics::KinematicsOutput"

[[deps]]
kind = "reference"
to = "mjwarp.physics.core"
to_api = "rpc:mjwarp_rs::physics::ComPositionOutput"

[[deps]]
kind = "reference"
to = "mjwarp.physics.attached"
to_api = "rpc:mjwarp_rs::physics::AttachedKinematicsOutput"

[[deps]]
kind = "call"
to = "mjwarp.runtime.transfer.kernel"
to_api = "rpc:mjwarp_rs::runtime::transfer::kernel::SynchronousKernel"

[[deps]]
kind = "call"
to = "mjwarp.runtime.transfer.core"
to_api = "rpc:mjwarp_rs::runtime::TransferSession"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
to_api = "rpc:mjwarp_rs::diagnostics::TransferError"

[[deps]]
kind = "reference"
to = "mjwarp.model.parameters"
to_api = "rpc:mjwarp_rs::model::KinematicsParameters"

[[deps]]
kind = "reference"
to = "mjwarp.model.mocap"
to_api = "rpc:mjwarp_rs::model::MocapModelInput"

[[deps]]
kind = "reference"
to = "mjwarp.model.camlight"
to_api = "rpc:mjwarp_rs::model::CamLightModelInput"

[[deps]]
kind = "reference"
to = "mjwarp.physics.camlight"
to_api = "rpc:mjwarp_rs::physics::CamLightOutput"

[[deps]]
kind = "reference"
to = "mjwarp.model.fixed-tendon"
to_api = "rpc:mjwarp_rs::model::FixedTendonModelInput"

[[deps]]
kind = "reference"
to = "mjwarp.physics.fixed-tendon"
to_api = "rpc:mjwarp_rs::physics::FixedTendonOutput"

[[deps]]
kind = "reference"
to = "mjwarp.model.spatial-tendon"
to_api = "rpc:mjwarp_rs::model::SpatialTendonModelInput"

[[deps]]
kind = "reference"
to = "mjwarp.physics.spatial-tendon"
to_api = "rpc:mjwarp_rs::physics::SpatialTendonOutput"

[[deps]]
kind = "reference"
to = "mjwarp.physics.tendon-wrap"
to_api = "rpc:mjwarp_rs::physics::tendon_wrap::TENDON_WRAP_CUDA"

[[deps]]
kind = "reference"
to = "mjwarp.model.tendon"
to_api = "rpc:mjwarp_rs::model::TendonModelInput"

[[deps]]
kind = "reference"
to = "mjwarp.physics.tendon"
to_api = "rpc:mjwarp_rs::physics::TendonOutput"

[[deps]]
kind = "reference"
to = "mjwarp.model.flex"
to_api = "rpc:mjwarp_rs::model::FlexPositionModelInput"

[[deps]]
kind = "reference"
to = "mjwarp.physics.flex"
to_api = "rpc:mjwarp_rs::physics::FlexPositionOutput"

[[deps]]
kind = "reference"
to = "mjwarp.model.sleep"
to_api = "rpc:mjwarp_rs::model::TendonWakeModelInput"

[[deps]]
kind = "reference"
to = "mjwarp.physics.sleep"
to_api = "rpc:mjwarp_rs::physics::SleepTreeOutput"

[[deps]]
kind = "reference"
to = "mjwarp.physics.flex-edge"
to_api = "rpc:mjwarp_rs::physics::FlexEdgeOutput"

[[deps]]
kind = "reference"
to = "mjwarp.model.flex"
to_api = "rpc:mjwarp_rs::model::FlexEdgeFields"

[[deps]]
kind = "reference"
to = "mjwarp.physics.flex-face"
to_api = "rpc:mjwarp_rs::physics::FlexFaceOutput"

[[deps]]
kind = "reference"
to = "mjwarp.model.flex"
to_api = "rpc:mjwarp_rs::model::FlexFaceFields"

[[deps]]
kind = "reference"
to = "mjwarp.physics.flex-hessian"
to_api = "rpc:mjwarp_rs::physics::FlexHessianOutput"

[deps.label]
zh = "矩阵快照"
en = "Hessian snapshot"

[[deps]]
kind = "reference"
to = "mjwarp.model.flex"
to_api = "rpc:mjwarp_rs::model::FlexHessianFields"

[deps.label]
zh = "矩阵输入"
en = "Hessian input"
+++

## 2026-10-14 CRB设备借用

新增私有质心设备借用。
借用要求新鲜com票据。
新增所需前置的回读检查。
检查刚体与质心守卫和有限性。
它不要求未执行的附着结果。
旧G01公开语义与物理CUDA保持。
本轮仅新常驻CRB入口使用。
source文件身份出现真实变化。
旧冻结清单不自动继承。
RNE两类三次预算原样保留。
完整G02与精度仍未验收。
证据：C:/Rust/mjwarp-rs/docs/resident-crb.md。

## 当前数值参考顺序（2026-10-11）

第一参考标准为原生MuJoCo。
原生通过即通过该项数值门禁。
此时不要求MJWarp再次通过。
仅原生超差后启用冻结MJWarp。
第二参考通过也通过数值门禁。
两路均超差则判失败。
缺少所需参考或容差则待验收。
本规则不要求双路同时通过。
两类通过具有相同准入效力。
报告另记实际采用的参考。
可选上游诊断不追加否决权。

两路按字段分别冻结容差。
不得根据失败结果放宽容差。
三方核对有效模型与状态。
版本与量化差异须单独登记。
结构错误不进入容差回退。
非有限值直接判失败。
内部精度仍跟随冻结上游。
NP01、NP02与NP04继续独立检查。
NP03改为两级数值判定全覆盖。
失败与待验收数量均须归零。

T4表示Windows本机GPU验收。
它覆盖完整G01与低层入口。
它不表示显卡型号或肌腱编号。
双模式均须记录实际测试证据。
产品测试不启动Python。
两级机制已接入G01主链与低层。
运动肌腱与G03辅助也已接入。
其余辅助仍保留原有门禁。
用户选择缺容差时记待验收。
本轮不采用上游通用容差。
本轮不改写历史测试结果。
完整T4仍保持未验收。
本轮逐项复审后刷新指纹。

Native MuJoCo is the first numerical reference. A native-tolerance pass is sufficient for the corresponding NP03 check; no MJWarp numerical pass is additionally required. Only a native-tolerance failure invokes frozen MJWarp as the second reference. A second-reference pass also passes NP03. Both failing means failure; missing required references or frozen tolerances means unverified. NP01, NP02 and NP04 remain independent. Automated gates now cover G01 main/raw fields, motion tendon blocks and G03 auxiliary solves. Missing second references or approved tolerances remain pending and block acceptance. The user chose mechanism-first implementation without freezing secondary tolerances; physical kernels and native tolerances are unchanged.

补测：C:/Rust/mjwarp-rs/docs/windows-two-reference-acceptance.md
契约：C:/Rust/mjwarp-rs/docs/numerical-precision-contract.md
变更：changes/2026-10-11-mjwarp-parallel-acceptance.json

## G01实现

for_g01建立等价计划。
导入先预检全部字段。
低层入口读取已准备设备字段。
低层入口不要求新鲜度票据。
严格计划保留原有就绪检查。
fwd_kinematics串联全部分支。
它保留静态几何与世界位姿。
柔体先清除Hessian标志。
限位肌腱按条件唤醒树。
它不计算碰撞或质量阶段。
G01输入不表示完整M02与M07。

证据：C:/Rust/mjwarp-rs/docs/windows-g01-acceptance.md。

## 本轮源码复审

设备模型参数固定采用f32。
私有速度前置链复用位置上传。
它核对模型身份与工作区尺寸。
刚体与质心内核按顺序运行。
它们保持同会话缓冲与守卫。
这些辅助不改变G01调度。
外部G01接口与严格入口保持不变。

## 2026-10-14 新混合桥接测试证据

新增cfg(all(test,cuda-probe))只读state钩子。
MassMatrix转接Kinematics实际设备qpos状态。
产品方法/计算/ABI不改，旧测试不重放。
qpos state无四槽守卫，桥接测试误判后静态修复。
修后状态用途未执行，3/3预算冻结。
新numeric桥接及private源故障独立通过。
不继承旧G01/CRB完整验收，原UID活动态保持。
证据：C:/Rust/mjwarp-rs/docs/resident-mixed-mass.md。
