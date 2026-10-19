+++
uid = "006c4859"
id = "mjwarp.physics.mass-matrix"
tags = ["implemented-subset", "gpu", "rigid-mass-matrix", "windows"]
fingerprint = "9b00da6baa323bebd1a5d28ec1a061679a73cf8603f73c8153d8a050989d1c54"

[name]
zh = "刚体质量矩阵辅助"
en = "Rigid mass-matrix helper"

[description]
zh = """
刚体CRB与矩阵采用f32。 常驻消费者直接借用M。 完整G03仍待实现。
"""
en = """
f32 rigid CRB/M and direct borrowed live-M consumer; complete G03 remains pending.
"""

[[source]]
path = "src/physics/mass_matrix.rs"

[[source]]
path = "src/physics/mass_matrix/resident.rs"

[[source]]
path = "src/physics/mass_matrix/multiply.rs"

[[source]]
path = "src/physics/mass_matrix/multiply/borrowed.rs"

[[source]]
path = "src/physics/mass_matrix/multiply/borrowed/device_energy.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::probe_mass_matrix"

[apis.description]
zh = """
实现GPU质量矩阵子集。 不替代完整G02阶段。
"""
en = """
Implemented strict shared-model GPU CRB/dense mass matrix with DOF armature; no complete crb/G02 equivalence.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::MassMatrixOutput"

[apis.description]
zh = """
持有复合惯量与稠密矩阵。 不承诺矩阵可逆。
"""
en = """
Owned checked finite GPU output, nbody/nv/world counts and checked views; no factorization or invertibility guarantee.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::MassMatrixWorld"

[apis.description]
zh = """
借用世界复合惯量与矩阵。 稠密矩阵按行展开。
"""
en = """
Read-only per-world packed CRB and row-major symmetric dense matrix; f32 and mixed generalized-coordinate units.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::MassMatrixPlan"

[apis.description]
zh = """
常驻CRB严格辅助。完整G02仍待验收。
"""
en = """
Implemented strict resident CRB subset with explicit position and matrix stages; not full CRB/G02 acceptance.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::MassMatrixData"

[apis.description]
zh = """
常驻CRB严格辅助。完整G02仍待验收。
"""
en = """
Implemented strict resident CRB subset with explicit position and matrix stages; not full CRB/G02 acceptance.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::MassMultiplyPlan"

[apis.description]
zh = """
独立常驻GPU mul_m。支持Sparse/Dense、skip和替代M，不自动forward，完整G03/NP未验收。
"""
en = """
Standalone resident f32 mass multiplication, Sparse/Dense, skip and alternate matrix; no automatic forward, complete G03/NP unaccepted.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::MassMultiplyData"

[apis.description]
zh = """
独立常驻GPU mul_m。支持Sparse/Dense、skip和替代M，不自动forward，完整G03/NP未验收。
"""
en = """
Standalone resident f32 mass multiplication, Sparse/Dense, skip and alternate matrix; no automatic forward, complete G03/NP unaccepted.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::MassMultiplyOutput"

[apis.description]
zh = """
独立常驻GPU mul_m。支持Sparse/Dense、skip和替代M，不自动forward，完整G03/NP未验收。
"""
en = """
Standalone resident f32 mass multiplication, Sparse/Dense, skip and alternate matrix; no automatic forward, complete G03/NP unaccepted.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::MassMultiplyFormat"

[apis.description]
zh = """
独立常驻GPU mul_m。支持Sparse/Dense、skip和替代M，不自动forward，完整G03/NP未验收。
"""
en = """
Standalone resident f32 mass multiplication, Sparse/Dense, skip and alternate matrix; no automatic forward, complete G03/NP unaccepted.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::MassMatrixMultiplyPlan"

[apis.description]
zh = """
直接借用常驻设备矩阵。 视图认证本次计算世界。 跳过槽只保存历史结果。
"""
en = """
Direct same-plan live-device-M consumer with read-only source/data borrow; computed worlds only are current, skips remain historical.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::MassMatrixMultiplyData"

[apis.description]
zh = """
直接借用常驻设备矩阵。 视图认证本次计算世界。 跳过槽只保存历史结果。
"""
en = """
Direct same-plan live-device-M consumer with read-only source/data borrow; computed worlds only are current, skips remain historical.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::MassMatrixMultiplyView"

[apis.description]
zh = """
直接借用常驻设备矩阵。 视图认证本次计算世界。 跳过槽只保存历史结果。
"""
en = """
Direct same-plan live-device-M consumer with read-only source/data borrow; computed worlds only are current, skips remain historical.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::MassMatrixEnergyPlan"

[apis.description]
zh = """
严格当前态设备动能。 实际源视图证明来源。 配对采用GPU内复制。
"""
en = """
Strict current-view kinetic device connection; actual source borrowing and device pair copies, no external state labels.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::MassMatrixEnergyData"

[apis.description]
zh = """
严格当前态设备动能。 实际源视图证明来源。 配对采用GPU内复制。
"""
en = """
Strict current-view kinetic device connection; actual source borrowing and device pair copies, no external state labels.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::MassMatrixEnergyView"

[apis.description]
zh = """
严格当前态设备动能。 实际源视图证明来源。 配对采用GPU内复制。
"""
en = """
Strict current-view kinetic device connection; actual source borrowing and device pair copies, no external state labels.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::MassMatrixEnergySnapshot"

[apis.description]
zh = """
严格当前态设备动能。 实际源视图证明来源。 配对采用GPU内复制。
"""
en = """
Strict current-view kinetic device connection; actual source borrowing and device pair copies, no external state labels.
"""

[[deps]]
kind = "call"
to = "mjwarp.runtime.transfer.kernel"
from_api = "rpc:mjwarp_rs::physics::probe_mass_matrix"
to_api = "rpc:mjwarp_rs::runtime::transfer::kernel::SynchronousKernel"

[deps.label]
zh = "提交同步内核"
en = "Synchronous kernel launch"

[[deps]]
kind = "reference"
to = "mjwarp.model.inertial"
from_api = "rpc:mjwarp_rs::physics::probe_mass_matrix"
to_api = "rpc:mjwarp_rs::model::InertialModelInput"

[deps.label]
zh = "读取刚体模型子集"
en = "Validated rigid model"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::physics::probe_mass_matrix"
to_api = "rpc:mjwarp_rs::diagnostics::TransferError"

[deps.label]
zh = "返回输入与设备错误"
en = "Input and device errors"

[[deps]]
kind = "call"
to = "mjwarp.runtime.transfer.core"
from_api = "rpc:mjwarp_rs::physics::probe_mass_matrix"
to_api = "rpc:mjwarp_rs::runtime::TransferSession"

[deps.label]
zh = "复制模型与状态"
en = "Model and state transfer"

[[deps]]
kind = "call"
to = "mjwarp.physics.core"
from_api = "rpc:mjwarp_rs::physics::probe_mass_matrix"

[deps.label]
zh = "复用内部质心阶段"
en = "Reuse checked COM stage"

[[deps]]
kind = "reference"
to = "mjwarp.physics.resident"
from_api = "rpc:mjwarp_rs::physics::MassMatrixPlan"
to_api = "rpc:mjwarp_rs::physics::KinematicsPlan"

[[deps]]
kind = "reference"
to = "mjwarp.physics.resident"
from_api = "rpc:mjwarp_rs::physics::MassMatrixData"
to_api = "rpc:mjwarp_rs::physics::KinematicsData"
+++

## 2026-10-14 CSR衔接源守卫证明

原matrix_input仍只检查ready。
新组合另用GPU扫描全部输出及守卫。
新增cfg(test)私有故障写入口。
源前后守卫与未选上NaN注入拒绝。
旧CRB内核与原成功测试保持。
新证据只属于CSR设备衔接目的。
证据：C:/Rust/mjwarp-rs/docs/resident-sparse-mass.md。

## 2026-10-14 常驻求解借用

新增物理私有的计划归属检查。
新增新鲜矩阵只读设备借用。
测试专用入口注入单世界坏矩阵。
产品不导出故障注入入口。
CRB计算与原CUDA常量保持不变。
旧CRB回执仍绑定bd37548。
不重跑原已成功的CRB测试。
新缓存测试实证回代不重新分解。
完整CRB等价仍遵循原缺口。

## 2026-10-14 常驻CRB严格组合

新增MassMatrixPlan与MassMatrixData。
复用原G01刚体与质心设备缓冲。
模型上传与编译只在计划创建时。
位置与矩阵拥有显式独立阶段。
矩阵消费新鲜cinert与cdof。
位置写入废弃前置与矩阵。
输入拒绝和外来计划保留结果。
只回读所需前置及结果守卫。
十系数按深度采用f32原子加。
父体为世界体时跳过累加。
_M分量1先舍入i1*v1。
六项点积从融合正零起步。
对角先写armature再原子加点积。
镜像复制原子操作后实际值。
四个次正规边界实测通过。
新集成首轮两过，单元首轮两过。
六模型八状态五世界规模。
每状态刷新两次，15240416项通过。
保留原生容差与全部旧覆盖。
新目标各1/3，成功后未重跑。
旧一次性内核与测试保持。
不继承旧整体验收或RNE结论。
稀疏ABI、休眠及完整G02未收口。
保留原UID006c4859和活动态。
证据：C:/Rust/mjwarp-rs/docs/resident-crb.md。

## 未验收工作区快照（2026-10-12）

本次审查实际未提交差异。
GPU先完成六项f32点积。
随后单独加对角armature。
新增回归隔离舍入顺序。
该GPU回归尚未执行。
本次没有产品通过结论。
指纹只记录当前文件字节。
G03辅助修复负责后续验收。
不以本快照激活完整G02。

## 本轮源码复审

矩阵辅助全链采用f32。
它复用同会话的质心缓冲。
GPU计算复合惯量与稠密矩阵。
armature只累加到对应对角。
求解入口复用原设备矩阵。
宿主检查不反馈设备计算。
内部不再提供f64标量分支。
本入口不提供分解与求解。
本叶子不替代完整G02阶段。

## 2026-10-14 新混合桥接测试证据

新增cfg(all(test,cuda-probe))只读state钩子。
MassMatrix转接Kinematics实际设备qpos状态。
产品方法/计算/ABI不改，旧测试不重放。
qpos state无四槽守卫，桥接测试误判后静态修复。
修后状态用途未执行，3/3预算冻结。
新numeric桥接及private源故障独立通过。
不继承旧G01/CRB完整验收，原UID活动态保持。
证据：C:/Rust/mjwarp-rs/docs/resident-mixed-mass.md。

## 2026-10-14 常驻GPU质量矩阵乘法

新增MassMultiply Plan/Data/Output/Format，对应U073/P4。
独立上传M/alternate/vec，不自动forward或推进时钟。
原CSR追加序无sort，镜像原地址；dense静态列循环。
物理f32正零acc乘加；meta i32，skip bool私有i32适配。
全checked元数据及mc/u32与worlds*n边界，原256 ABI不改。
skip保存已有res位型，独立alternate不改default。
两新调用default2过0.03秒/195过滤、CUDA2过9.97秒/270过滤。
新组合2/3成功冻结，零准备失败，不补第三次。
72批1485120槽最大误差1.3435145600482201e-6。
新执行前冻结abs/rel2e-5，仅独立诊断，不继承solve等价。
Frozen support_test MuJoCo5e-4不等于NP上游静态验收。
两格式skip/alternate/世界256/输入位型/快照通过。
Dense非对称跨树alternate按独立点积通过；私有全部守卫/NaN/Inf/恢复。
主阶段9645ms含JIT/分配传输/宿主诊断，六自有缓冲容量313664644bytes非峰值。
旧产物命名entry+PTX覆盖无用Sparse NV宏，保留六entry/三PTX非全JIT回执。
六实际source/PTX哈希核对通过；dense86/390 FMA86/390、Sparse65 FMA5，无floatf64。
预绑定94源与执行源不变，旧物理/ABI/门禁/Cargo/参考不改。
全部旧冻结测试/Clippy/SDK/作者零调用，旧bridge3/3/状态FAILED保持。
原UID/active不变，root/core planned与正式前置资源锁不动。
完整U073/G03/NP、CRB device-M借用/共享广播/nv0、双平台T4仍缺。
下一就绪qfrc_smooth合并须核fwd_acceleration/P4阶段中间量，不重放旧solve。
证据：C:/Rust/mjwarp-rs/docs/mass-multiply.md。


## 2026-10-14 常驻CRB/M直接借用消费者

新增MassMatrixMultiplyPlan/Data/View。计划借用实际MassMatrixPlan；只读context读取实际i32 parents检查CSR匹配，旧函数/CUDA不变。
每次update消费同原计划ready MassMatrixData原buffer，明确guard4+world*(10nb+nv²)+guard4，M无副本/复制/host反馈。
Sparse保留原追加顺序与下三角镜像地址，Dense静态nv列序；f32正零累加、融合许可与原门禁保持。
结果载荷D2D复制current到staged保留skip；全暂存有限/保护槽通过才交换，不部分发布。
View同时共享借用源与consumerData，Rust禁止视图期内qpos/M/vector写入；无外部整数标签或缓存M。
逐世界is_current仅本次计算为true，skip历史输出为false；fresh_snapshot拒绝任何skip，普通snapshot只作历史结果。
初次全skip、qpos新态全skip、vector改写部分skip与同plan另一source实例切换+skip已覆盖。
冷源/异计划/consumer跨token/非法向量与skip通过；错worlds和坏dense地址拒绝只有静态证据，不冒充GPU覆盖。
新host2过0.00秒/205过滤、GPU2过4.89秒/289过滤，combined2/3成功冻结不补第三次。
主72批210080槽，条件化于实际GPU f32 M的独立f64点积maxabs4.05125319957733154e-7，原abs/rel2e-5。
该参考只证明消费关系，不证明CRB/原生/冻结上游精度；旧CRB/mul_m/bridge/factor/SDK/作者/Clippy验收目的0。
实际旧生产者作为新连接依赖主19/private1共20次update，非20核启动，不声称producer0calls。
私有双格式102故障恢复，无部分发布；source guard/CRB/M/vector非有限与溢出、metadata静态范围拒绝。
主4065ms含JIT/host/传输/依赖；consumer5buffer219852bytes，不含producer，不是peak/fair。
19真实entry=12新consumer+7依赖，10唯一PTX；新Sparse各FMA5/Dense3/33/65，无f64/atomic。
225预身份原样封存，224同模式不变；新test名称大写M编译warning1，静态小写m改名正文相同，最终名版本未编译/执行，不重置冻结purpose。
补属性前217旧blob不变，4旧文件只新增声明/exports/只读context；属性文本另登记，产品源码没有后测修补。
原moduleUID/state/running claims/dependencies/resources保持，完整U073/G03/energy设备接线/NP、旧Sparse原源与Small/bridge失败继续开放冻结。
证据C:/Rust/mjwarp-rs/docs/borrowed-mass-multiply.md。


### 提交前完整祖先行修复

parents相等不能认证Compact/子集CSR来源。严格新packed新增完整祖先行拒绝，避免漏掉真实CRB耦合。
仅修改新消费者Rust构造与既有新host拒绝断言；六新CUDA源与全部旧body/tests/容差不变。
原host/GPU成功冻结2/3不再执行，原执行源与日志保留；final mainGPU body只改大小写测试名，另新host断言未运行。
实际cargo check --locked --features cuda-probe --lib成功2.46秒，只编译最终库，最终test版本仍未编译/执行。
该拒绝分支仅静态与库类型检查证据，不宣称动态已验收。旧预stage map c83e3e1属历史，最终map/HEAD另核。


## 2026-10-14 严格当前态设备动能接线

MassMatrixEnergyPlan/Data/View/Snapshot接通实际CRB→M→Mqvel→GPU pair→原scalar32。
能量计划借用实际乘积计划，双Arc身份/原matrix计划/ready/worlds/allcurrent全部检查，无外部状态标签。
能量View继续借用原乘积View与能量Data，合法qpos/vector/Mqvel可变写入在视图期内不成立；snapshot只作历史，势能仅显式透传不认证状态。
每world两次既有memcpy_dtod成对复制q/Mq，非零复制pair；M仍零复制，无正常物理Mqvel下载回传。
新原始source有限性/保护槽GPU验证，原CUDA字符串与32ABI逐字复用；完全暂存通过才swap，无部分发布。
新host2过0.00秒/207过滤（编译3.69），GPU2过4.58秒/293过滤（编译7.69），无warning，combined2/3成功freeze不补第三次。
72批6240 kinetic/potential世界各自零位差，420160配对槽精确，额外nv3双格式负零4世界，private56故障通过。
一热hot=(world+vector+pose)%nv逐world区别两向量；独立源Mjj正正规数指数减1作精确预期，不读候选Mqvel/energy。
不移植mul_m 2e-5作为能量容差；原U047固定输入与完整标准不变，一热只补接线；一般混合向量与M/原生/上游精度仍缺。
一个新GPU命令两新函数内source18+private1=19；Mqvel72+allskip2+partial2+negativezero2+private1=79，非独立test或核启动。
旧test selectors/Clippy/SDK/作者/cargo check/full0；共享旧tests仅编译不执行，原祖先CSR新拒绝断言不追认。
3762ms含JIT/transfer/host诊断/依赖，不含初次session；能量自有四float+一int138912bytes，不含M/Mqvel producer，非peak/fair。
28实际entry/16唯一PTX，source validator3+原scalar6+其他依赖19；nv33/65四份scalar源与PTX匹配旧U047原记录，完整冻结upstream生成entry仍缺。
228预绑定零漂移；补属性前217旧blob不变，5文件仅exports/privateconst/helpervisibility。执行源与当前源完全相同。
归档脚本首次counter matches与PS自动Matches冲突，只修静态计数重归档，无物理调用。
旧UID/state/main claims/deps/resources不变；完整energy_vel/sensor/G03/NP/一般向量、祖先拒绝动态验证、旧Sparse原源/失败及Linux/release/T4/fair性能仍开放。
证据C:/Rust/mjwarp-rs/docs/device-energy.md。
