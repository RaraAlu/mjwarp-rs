+++
uid = "cd53f287"
id = "mjwarp.physics.mass-solve"
tags = ["implemented-subset", "gpu", "rigid-mass-solve", "windows"]
fingerprint = "bf9057c4c2dcc966163ad9f978cae292603ea669a5060e83addc041f3edc2108"

[name]
zh = "刚体质量矩阵求解"
en = "Rigid mass-matrix solve"

[description]
zh = """
实现GPU f32质量分解与求解辅助。新增CRB到三路混合CSR常驻桥接。数值与故障通过，状态测试失败修后未执行。Tile与完整G03仍未验收。
"""
en = """
Bounded f32 sparse/mixed factors and solves with resident CRB-to-CSR bridge; numeric and private fault subsets pass, repaired lifecycle unexecuted; Tile/full G03 unaccepted.
"""

[[source]]
path = "src/physics/mass_solve.rs"

[[source]]
path = "src/physics/mass_solve/resident.rs"

[[source]]
path = "src/physics/mass_solve/sparse.rs"

[[source]]
path = "src/physics/mass_solve/sparse/resident.rs"

[[source]]
path = "src/physics/mass_solve/sparse/resident/solve.rs"

[[source]]
path = "src/physics/mass_solve/sparse/dense.rs"

[[source]]
path = "src/physics/mass_solve/sparse/dense/small.rs"

[[source]]
path = "src/physics/mass_solve/sparse/mixed.rs"

[[source]]
path = "src/physics/mass_solve/sparse/mixed/solve.rs"

[[source]]
path = "src/physics/mass_solve/sparse/mixed/solve/resident.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::probe_mass_solve"

[apis.description]
zh = """
同步GPU f32分块质量求解。不替代完整G03阶段。
"""
en = """
Synchronous bounded f32 per-tree factorization and multi-RHS solve; validate inputs and reject invalid pivots without CPU fallback.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::MassSolveOutput"

[apis.description]
zh = """
独立持有同步宿主结果。世界视图检查索引。
"""
en = """
Owned finite f32 factors, diagonal inverses and multi-RHS solutions with checked world views and counts; independent of session lifetime.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::MassSolveWorld"

[apis.description]
zh = """
按块借用因子、倒数与解。blocks明确上三角或反向LDL。
"""
en = """
Read-only row-major factors with explicit per-tree blocks: upper Cholesky or reverse LDL; singleton factors and Cholesky inverse slots are zero.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::MassFactorKind"

[apis.description]
zh = """
区分对角、Cholesky与反向LDL。
"""
en = """
Explicit diagonal, upper Cholesky and reverse LDL semantics.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::MassFactorBlock"

[apis.description]
zh = """
记录树区间与因子格式。
"""
en = """
Contiguous DOF interval and its factor format.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::MassSolvePlan"

[apis.description]
zh = """
常驻分解与右端项严格辅助。完整G03仍未验收。
"""
en = """
Resident factor-cache and independent RHS-stream subset; full G03 and sparse equivalence unaccepted.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::MassSolveData"

[apis.description]
zh = """
常驻分解与右端项严格辅助。完整G03仍未验收。
"""
en = """
Resident factor-cache and independent RHS-stream subset; full G03 and sparse equivalence unaccepted.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SparseMassFactorPlan"

[apis.description]
zh = """
真实CSR分层分解辅助。不提供融合回代与非紧凑混合分解。
"""
en = """
True CSR layered factor subset, without fused solve or noncompact mixed factors.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SparseMassFactorData"

[apis.description]
zh = """
真实CSR分层分解辅助。不提供融合回代与非紧凑混合分解。
"""
en = """
True CSR layered factor subset, without fused solve or noncompact mixed factors.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SparseMassFactorOutput"

[apis.description]
zh = """
真实CSR分层分解辅助。不提供融合回代与非紧凑混合分解。
"""
en = """
True CSR layered factor subset, without fused solve or noncompact mixed factors.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SparseMassFactorWorld"

[apis.description]
zh = """
真实CSR分层分解辅助。不提供融合回代与非紧凑混合分解。
"""
en = """
True CSR layered factor subset, without fused solve or noncompact mixed factors.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SparseMassPlan"

[apis.description]
zh = """
常驻CRB到CSR设备衔接与分层分解辅助。完整G03仍未验收。
"""
en = """
Resident CRB-to-CSR device composition and sparse-factor subset; no complete G03 or fused solve admission.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SparseMassData"

[apis.description]
zh = """
常驻CRB到CSR设备衔接与分层分解辅助。完整G03仍未验收。
"""
en = """
Resident CRB-to-CSR device composition and sparse-factor subset; no complete G03 or fused solve admission.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SparseMassOutput"

[apis.description]
zh = """
常驻CRB到CSR设备衔接与分层分解辅助。完整G03仍未验收。
"""
en = """
Resident CRB-to-CSR device composition and sparse-factor subset; no complete G03 or fused solve admission.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SparseMassWorld"

[apis.description]
zh = """
常驻CRB到CSR设备衔接与分层分解辅助。完整G03仍未验收。
"""
en = """
Resident CRB-to-CSR device composition and sparse-factor subset; no complete G03 or fused solve admission.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SparseMassSolvePlan"

[apis.description]
zh = """
128线程稀疏融合回代辅助。缓存因子与独立RHS流；非紧凑混合和完整NP未验收。
"""
en = """
Bounded 128-thread fused sparse solve with cached factors and independent RHS streams; noncompact mixed blocks and full NP remain unaccepted.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SparseMassSolveData"

[apis.description]
zh = """
128线程稀疏融合回代辅助。缓存因子与独立RHS流；非紧凑混合和完整NP未验收。
"""
en = """
Bounded 128-thread fused sparse solve with cached factors and independent RHS streams; noncompact mixed blocks and full NP remain unaccepted.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SparseMassSolveOutput"

[apis.description]
zh = """
128线程稀疏融合回代辅助。缓存因子与独立RHS流；非紧凑混合和完整NP未验收。
"""
en = """
Bounded 128-thread fused sparse solve with cached factors and independent RHS streams; noncompact mixed blocks and full NP remain unaccepted.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SparseMassSolveWorld"

[apis.description]
zh = """
128线程稀疏融合回代辅助。缓存因子与独立RHS流；非紧凑混合和完整NP未验收。
"""
en = """
Bounded 128-thread fused sparse solve with cached factors and independent RHS streams; noncompact mixed blocks and full NP remain unaccepted.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SparseMassDensePlan"

[apis.description]
zh = """
GPU采集非紧凑块对称原矩阵。保留f32位型与守卫，不计算Cholesky因子。
"""
en = """
GPU symmetric noncompact-block matrix gather with f32 bit preservation and guards; no Cholesky factors.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SparseMassDenseData"

[apis.description]
zh = """
GPU采集非紧凑块对称原矩阵。保留f32位型与守卫，不计算Cholesky因子。
"""
en = """
GPU symmetric noncompact-block matrix gather with f32 bit preservation and guards; no Cholesky factors.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SparseMassDenseOutput"

[apis.description]
zh = """
GPU采集非紧凑块对称原矩阵。保留f32位型与守卫，不计算Cholesky因子。
"""
en = """
GPU symmetric noncompact-block matrix gather with f32 bit preservation and guards; no Cholesky factors.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SparseMassSmallPlan"

[apis.description]
zh = """
仅Small2..6前缀U辅助。Tile拒绝；宿主证书修后未执行，完整混合及NP未验收。
"""
en = """
Small2..6 U-prefix prototype only; reject Tile, repaired host certificate unexecuted, full mixed G03/NP unaccepted.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SparseMassSmallData"

[apis.description]
zh = """
仅Small2..6前缀U辅助。Tile拒绝；宿主证书修后未执行，完整混合及NP未验收。
"""
en = """
Small2..6 U-prefix prototype only; reject Tile, repaired host certificate unexecuted, full mixed G03/NP unaccepted.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SparseMassSmallOutput"

[apis.description]
zh = """
仅Small2..6前缀U辅助。Tile拒绝；宿主证书修后未执行，完整混合及NP未验收。
"""
en = """
Small2..6 U-prefix prototype only; reject Tile, repaired host certificate unexecuted, full mixed G03/NP unaccepted.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SparseMassMixedFactorPlan"

[apis.description]
zh = """
常驻CSR三路混合分解。仅Sparse/Small/Compact。Tile仍拒绝。不提供混合求解。
"""
en = """
Resident CSR Sparse/Small/Compact factor assembly; Tile and mixed solve remain unsupported.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SparseMassMixedSolvePlan"

[apis.description]
zh = """
三路常驻混合缓存消费。支持多RHS。Tile与CRB桥接仍拒绝。
"""
en = """
Resident three-route merged-factor consumer with independent RHS batches; no Tile or CRB bridge.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SparseMassMixedSolveData"

[apis.description]
zh = """
三路常驻混合缓存消费。支持多RHS。Tile与CRB桥接仍拒绝。
"""
en = """
Resident three-route merged-factor consumer with independent RHS batches; no Tile or CRB bridge.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SparseMassMixedSolveOutput"

[apis.description]
zh = """
三路常驻混合缓存消费。支持多RHS。Tile与CRB桥接仍拒绝。
"""
en = """
Resident three-route merged-factor consumer with independent RHS batches; no Tile or CRB bridge.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SparseMassMixedSolveWorld"

[apis.description]
zh = """
三路常驻混合缓存消费。支持多RHS。Tile与CRB桥接仍拒绝。
"""
en = """
Resident three-route merged-factor consumer with independent RHS batches; no Tile or CRB bridge.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SparseMassMixedResidentPlan"

[apis.description]
zh = """
CRB到混合CSR常驻桥接。数值与故障通过，状态测试修后未执行。Tile与完整G03未验收。
"""
en = """
Resident CRB-to-mixed-CSR bridge; numeric and private faults pass, repaired lifecycle unexecuted; no Tile or complete G03 acceptance.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SparseMassMixedResidentData"

[apis.description]
zh = """
CRB到混合CSR常驻桥接。数值与故障通过，状态测试修后未执行。Tile与完整G03未验收。
"""
en = """
Resident CRB-to-mixed-CSR bridge; numeric and private faults pass, repaired lifecycle unexecuted; no Tile or complete G03 acceptance.
"""

[[deps]]
kind = "call"
to = "mjwarp.physics.mass-matrix"
from_api = "rpc:mjwarp_rs::physics::probe_mass_solve"

[deps.label]
zh = "复用设备质量矩阵"
en = "Reuse checked device mass"

[[deps]]
kind = "call"
to = "mjwarp.runtime.transfer.kernel"
from_api = "rpc:mjwarp_rs::physics::probe_mass_solve"
to_api = "rpc:mjwarp_rs::runtime::transfer::kernel::SynchronousKernel"

[deps.label]
zh = "提交同步内核"
en = "Synchronous kernel"

[[deps]]
kind = "call"
to = "mjwarp.runtime.transfer.core"
from_api = "rpc:mjwarp_rs::physics::probe_mass_solve"
to_api = "rpc:mjwarp_rs::runtime::TransferSession"

[deps.label]
zh = "上传多个右端项"
en = "Upload multiple RHS"

[[deps]]
kind = "reference"
to = "mjwarp.model.inertial"
from_api = "rpc:mjwarp_rs::physics::probe_mass_solve"
to_api = "rpc:mjwarp_rs::model::InertialModelInput"

[deps.label]
zh = "读取刚体辅助模型"
en = "Rigid model subset"

[[deps]]
kind = "call"
to = "mjwarp.physics.core"
from_api = "rpc:mjwarp_rs::physics::probe_mass_solve"

[deps.label]
zh = "核对守卫与数值"
en = "Guard and finite checks"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::physics::probe_mass_solve"
to_api = "rpc:mjwarp_rs::diagnostics::TransferError"

[deps.label]
zh = "报告世界与坏主元"
en = "World and pivot failure"

[[deps]]
kind = "reference"
to = "mjwarp.physics.mass-matrix"
from_api = "rpc:mjwarp_rs::physics::MassSolvePlan"
to_api = "rpc:mjwarp_rs::physics::MassMatrixPlan"

[[deps]]
kind = "reference"
to = "mjwarp.physics.mass-matrix"
from_api = "rpc:mjwarp_rs::physics::MassSolveData"
to_api = "rpc:mjwarp_rs::physics::MassMatrixData"

[[deps]]
kind = "reference"
to = "mjwarp.model.inertial"
from_api = "rpc:mjwarp_rs::physics::SparseMassFactorPlan"
to_api = "rpc:mjwarp_rs::model::SparseMassInput"

[[deps]]
kind = "call"
to = "mjwarp.runtime.transfer.kernel"
from_api = "rpc:mjwarp_rs::physics::SparseMassFactorPlan"
to_api = "rpc:mjwarp_rs::runtime::transfer::kernel::SynchronousKernel"

[[deps]]
kind = "reference"
to = "mjwarp.physics.mass-matrix"
from_api = "rpc:mjwarp_rs::physics::SparseMassPlan"
to_api = "rpc:mjwarp_rs::physics::MassMatrixPlan"

[[deps]]
kind = "reference"
to = "mjwarp.physics.mass-matrix"
from_api = "rpc:mjwarp_rs::physics::SparseMassData"
to_api = "rpc:mjwarp_rs::physics::MassMatrixData"
+++

## 2026-10-14 CRB到CSR设备衔接

新增SparseMassPlan/Data/Output/World。
构造核对nv及父链，拒绝同维异拓扑。
组合原CRB与稀疏分解，保持同会话。
GPU验证源全部有限值及双端守卫。
宿主只回读诊断状态和目标守卫。
GPU逐行复制实际稠密结果到CSR。
不回读后再上传任何物理输入。
源/状态/目标故障拒绝并可恢复。
归属检查先于失效，输入拒绝保留缓存。
位置/矩阵刷新废弃采集和因子。
旧内部CSR不得绕过新鲜采集再分解。
新集成首轮三过7.06秒。
新私有故障首轮一过0.82秒。
新目的各1/3，成功不重复。
1424800槽实际设备复制to_bits相等。
1476280因子倒数独立满足原生容差。
重构2e-6实际通过，不宣称求解残差。
两个证明不外推完整冻结流水线等价。
无上游诊断调用，旧395007只属历史。
新桥接缓冲最多10172字节。
九批3340ms包含宿主比较及重构。
不与旧123ms直接比较，无公平性能结论。
旧稀疏与CRB CUDA字节保持。
旧成功/满额/作者均零调用。
原UID、前置、资源与完整NP缺口保持。
证据：C:/Rust/mjwarp-rs/docs/resident-sparse-mass.md。

## 2026-10-14 新稀疏实际验证

新整体集成第二次四过4.28秒。
首轮诊断启动错误中止，保留失败。
整体与独立新输入冻结诊断各2/3。
私有故障单元首轮一过0.26秒。
非零耦合65链、67分支及紧凑混合。
世界1/2/257、两状态、九批。
1476280标量满足原容差。
候选和冻结分别通过原生门禁。
原f64重构2e-6实际通过。
395007项标量位型不同，不等于二进制位数。
不声称全上游零偏离或完整NP通过。
不改变原子顺序追零位差。
直接设备读实证矩阵写位型隔离。
输入与输出前守卫注入拒绝并恢复。
首轮共享256槽误启1024线程。
诊断改为256线程，不改覆盖或物理算式。
首轮只进入一测试、完成零项，计划四项。
私有单元另实证纯紧凑非空拒绝。
计划之外的满额与旧成功选择零调用。
新成功选择不追加测试。
求解与残差没有执行，不给对应准入。
九批阶段123毫秒，新产品缓冲4505584字节。
不含独立诊断/JIT/驱动，无公平性能结论。
Normify0错误9粒度警告，不关闭主变更。

## 2026-10-14 新CSR分层路径

新增SparseMassFactorPlan/Data及独立快照。
上传真正CSR，不调用旧稠密辅助。
冻结分层反序同步执行更新。
直接div.rn.f32后独立mul/neg/原子加。
末阶段直接除法计算对角倒数。
物理全链f32，f64只作宿主诊断。
支持稀疏与紧凑混合，拒绝非紧凑混合。
纯紧凑无稀疏尾区时明确拒绝。
只发布CSR尾区与倒数，无稠密前缀或解。
写入失效、同计划、整批主元拒绝及守卫。
旧>64稠密辅助分歧仍保留，不借新证据放行。
新耦合数值与位差验收另记真实日志。
原生作者保守3/3冻结，不再执行。
正式G03、全NP01至04、Linux与性能开放。
主前置、资源锁与原模块UID保持。
证据：C:/Rust/mjwarp-rs/docs/resident-sparse-factor.md。

## 2026-10-14 常驻分解与求解

复用常驻CRB设备矩阵。
新增MassSolvePlan与MassSolveData。
位置、矩阵、分解与回代分阶段。
右端项更新只废弃解。
因子采用独立只读设备缓存。
破坏原矩阵后仍能正确回代。
第二次单元实测因子位型保持。
再次分解拒绝坏主元并整批失效。
健康矩阵恢复后允许重新计算。
物理输入、中间量与输出均f32。
旧完整CUDA常量保持LF字节不变。
实际两段生成源码匹配编译回执。
换行规范化不触发新编译捕获。
新集成首轮两过，509616标量。
13503字段块原生放行，21块对照放行。
原容差、原参考与全部旧覆盖保持。
新单元保守3/3冻结，不再调用。
前两次各三过，末次宿主一过。
集成1/3成功后不重复。
旧满额与已过CRB均零调用。
冻结六模型最大块12维。
解析65维对角样例只证辅助行为。
冻结稀疏采用直接除法和分层原子减。
现辅助倒数再乘并串行更新。
舍入与算序差异阻断稀疏等价。
动态融合稀疏回代产物仍待核对。
主排程和资源锁保持，不启动正式P4。
完整G03、NP01至NP04仍未收口。
旧RNE1042失败及两类3/3保持。
Linux、release与公平性能未测。
证据：C:/Rust/mjwarp-rs/docs/resident-mass-solve.md。

## 分块辅助边界

保留模块UID与现有依赖。
产品物理链全部采用f32。
单自由度块只存质量倒数。
至64维块存上三角Cholesky。
更大块保留反向LDL辅助。
公开blocks字段说明格式。
ld保留名称，不保留旧统一语义。
Cholesky倒数槽填零。
多右端项复用同一分解。
交错树区间直接拒绝。
坏主元阻止整批发布。
会话在数值拒绝后仍可复用。
本辅助没有tile调度。
多自由度紧凑优化仍待实现。
小型分支树仍有路由差异。
验收读取冻结双参考。
原生派生因子只服务测试。
产品测试不启动Python。

## 2026-10-12首差修复

状态1的矩阵与上游逐位一致。
首差位于分解首项。
7至64维采用rsqrt.approx乘主元。
乘减保留PTX融合许可。
不加禁止融合的舍入修饰符。
2至6维保留标量sqrt路径。
回代算法与既有容差不变。
状态1同矩阵因子逐位一致。
同矩阵状态7末项仍差1ULP。
双模式求解集成各8过0败。
双模式求解单元各10过0败。
每模式1411次世界比较。
7027块原生放行，20块对照放行。
失败0块，待验收0块。
本次专项NP03通过。
不宣称全样本逐位等价。
不宣称完整G03、T4或NP04通过。
证据：docs/windows-g03-first-difference.md。


## 2026-10-14 稀疏融合回代

新增SparseMassSolvePlan/Data及独立快照。
每世界每RHS独占128线程块。
反层原子减、对角乘、正层原子减。
每层同步，物理全链f32。
读回解耦旧CRB与CSR矩阵。
RHS-only复用缓存因子。
重新采集或分解仍拒绝坏源。
新离线f64 LU只消费冻结原生M。
作者1/3；整体2/3，保留首次2过1败。
第二次只续跑归属项，1过。
数值函数首轮621920标量通过。
原2e-4双容差与2e-5残差保持。
首轮解位型断言属于错误确定性假设。
第二次只证明未选解数值正确。
私有2过1/3，RHS及因子位型保持。
128ABI1过1/3，不重复成功用途。
原395007历史不绑定本轮。
上游融合产物/NP/双平台仍开放。
CUDA Clippy4次全过但保守重叠超限。
明确登记流程偏差并冻结。
主变更和正式前置/资源锁保持。
证据：C:/Rust/mjwarp-rs/docs/resident-sparse-solve.md。


## 2026-10-14 非紧凑块GPU稠密化

新增SparseMassDensePlan/Data/Output。
它准备原矩阵，不发布Cholesky因子。
i32索引/f32输入输出，仅位型复制。
缺项写正零，源负零保留。
先GPU验证完整CSR源及守卫。
再按前缀地址采集设备原矩阵。
宿主只在采集前回读诊断状态。
就绪/外来计划/世界写入/快照寿命保持。
旧SparseFactor与Solve混合拒绝边界不改。
第四模型65+7参考可用于M布局，不可作U因子。
两实际缓存tile11/12不覆盖tile7。
目标宿主1过/GPU2过，跨配置累计2/3冻结。
私有1过0.39秒，1/3冻结。
2187640主槽+147世界编辑槽逐位通过。
缺项正零实际2046720，原预估2079480错误。
原contract字节保留，闭式纠正不改样本。
阶段122ms含比较，缓冲4358752字节非峰值。
无作者/Python/Clippy/旧GPU目标执行。
完整混合分解/MathDx/NP/双平台仍开放。
证据：C:/Rust/mjwarp-rs/docs/sparse-mass-dense.md。


## 2026-10-14 Small前缀分解

新增SparseMassSmallPlan/Data/Output。
只发布Small2..6前缀U，不生成LDL或D。
复用GPU Dense输入，Tile仍拒绝。
按size分组，static模板与64线程启动。
sqrt.rn/rcp.rn，mul/sub保留融合许可。
全部物理输入/中间/累减/输出f32。
DOF索引保持整数，主元拒绝整批。
缓存旧CSR投毒不阻止新因子读回。
世界256合法写入要求新鲜gather。
解析作者1/3成功后冻结。
新整体3/3冻结：编译失败、零用例、末轮3过1败。
两个GPU与哈希子项通过，不是整体通过。
55120因子槽满足原2e-4与重构2e-6。
宿主rawf64 qpos身份断言失败。
修正按旧qpos f32位型契约，修后未执行。
执行前后源与全部失败记录保留。
新私有两过1/3，新64ABI一过1/3。
旧Clippy与全部满额/成功目的零调用。
Small3..6冻结产物/SASS、MathDx与完整NP开放。
不提供完整混合factor/solve准入。
证据：C:/Rust/mjwarp-rs/docs/small-mass-prefix.md。

## 2026-10-14 常驻三路混合分解

新增SparseMassMixedFactorPlan/Data/Output/World。
只Sparse+Small+Compact，要求Sparse尾，Tile仍拒绝。
主CSR缓冲设备复制进入Small，再GPU合并U/CSR/Dinv。
CSR尾非紧凑Small保持原M，全nv倒数跟随冻结factor_m顺序。
不接收SDK CUBIN，不绕过现有精度门禁。
旧公开Sparse/solve拒绝及旧GPU算子不变。
新混合路由/布局、GPU数值缓存/失败边界仍待实测。
旧失败/成功预算保持冻结，本用途预登记最多3调用。
完整G03/七维产品/NP与双平台仍缺。
证据：C:/Rust/mjwarp-rs/docs/mixed-mass-factor.md。

## 新混合增量实测收尾

主CSR独占；gather捕获同代Sparse/Small源。
额外CSR快照防止两半代次分歧。
旧主M投毒仍复用旧双缓存，fresh gather拒绝。
新目标两GPU函数2过18.38秒，原容差不变。
六批1236040因子/倒数槽，maxabs3.43254025e-6。
重构max5.87321693e-7；18200正零下三角。
29120非稀疏CSR与10920倒数位型通过。
世界256/三坏主元/拒绝保留/快照寿命通过。
新私有缓存故障一过0.91秒/258过滤。
两类producer/merged/source守卫拒绝恢复。
保守合并宿主与新GPU调用共3/3，全部冻结。
unused import修复后私有CUDA编译无warning。
宿主路由/布局2过发生快照修复前；修后default配置未追加。
八数据缓冲11794880字节为容量算术统计，非峰值。
15713ms含宿主重构，不称纯GPU公平性能。
本轮未接通CRB混合桥接、Tile或混合solve。
旧公开Sparse拒绝/原CUDA/SDK成功与失败预算不变。
完整G03/NP01–04及Linux/release/T4仍开放。

## 2026-10-14 三路混合缓存消费

新增MixedSolve四类型与多RHS常驻入口。
仅消费已完成合并因子，Tile继续拒绝。
Small两次div.rn、mul/sub保留融合许可。
Compact D*y，Sparse只flag-1的UP/diag/DOWN。
新128ABI使用含prefix的因子stride。
矩阵写失效全部，RHS写只失效解。
读回不读取旧主M或生产者CSR。
普通86/拥挤390静态独立参考装配1/3成功冻结。
不重跑旧LU作者，只复用其原输出并制造已知新解。
当前新host2过0.01秒/191过滤，用途1/3冻结。
GPU数值/状态与私有故障仍待实测。
完整G03/NP/Tile与CRB桥接继续开放。
证据：C:/Rust/mjwarp-rs/docs/mixed-mass-solve.md。

## 新混合求解实测收尾

新MixedSolve消费完成合并因子，不读旧主M/生产者。
矩阵合法写全失效，RHS写只失效解并保持因子bits。
普通86/拥挤390，14批141592解槽/47656零RHS槽。
原abs/rel2e-4/maxabs2.6702880859375e-5。
原逐树2e-5/maxresidual3.7332696249326277e-7。
131Sparse/129Small块跨128路径通过。
新GPU2过4.67秒/私有缓存1过1.10秒无warning。
主M/CSR/Small producer投毒缓存独立消费，完成Merged异常整批拒绝。
RHS/输出/status守卫及世界256/快照寿命通过。
产品新host/GPU/private合计3/3冻结，不重放旧用途。
独立参考装配1/3与新128ABI1/3成功冻结。
308ms含诊断与拷贝，11缓冲容量12352436字节非峰值。
29实际entry捕获源/精度门禁passed；div.rn18/mul-sub各53/fma.rn0。
Small1/2旧sm89.ptx只读补证，3..6/SASS全对应仍缺。
旧错误PTX文件名查找前提已纠正，无旧重编译。
原SparseSolve/MixedFactor/SDK/Small失败/Clippy账本不动。
完整G03/NP/Tile与CRB桥接及双平台仍未验收。

## 2026-10-14 GPU CRB到混合常驻桥接

新增MixedResident Plan/Data，复用原MassMatrix与MixedSolve。
只写qpos/RHS，GPU fullsource验证/CSR拷贝并同代冻结。
不经旧纯Sparse入口，Tile继续拒绝，不需七维前置。
21物理输入及中间/累加/结果f32，CSR metadata i32。
旧计算/ABI/门禁不改，仅新增256验证/复制入口。
合法qpos全失效，RHS只失效解，非法/外来计划保留。
辅助组合68/69来自两冻结独立根，非新native编译模型。
CRB/M原2e-5，因子/解2e-4，逐树残差2e-5不改。
参考1失败/2成功2/3冻结，纯PS数组括号修复。
失败Tee文件不存在，保存原script/contract与工具输出转录。
新host2过0.10秒/193过滤，产品组合1/3冻结。
剩余新GPU双函数及私有缓存单函数待验。
全部旧成功/失败/满额/SDK/Clippy/作者不追加。
证据：C:/Rust/mjwarp-rs/docs/resident-mixed-mass.md。

## 2026-10-14 桥接本次实测收尾

新产品合计3/3冻结，default host2过0.10秒。
GPU数值1过/状态1败5.64秒，private1过1.76秒。
12批2827048 CRB/M、1143268 CSR位槽、1181572因子、108256解。
原CRB/M2e-5与factor/solve2e-4、逐树2e-5不改。
最大误差CRB/M6.67572e-6、factor3.76685e-6、solve4.63754e-6。
最大逐树残差5.59803e-8，数值成功冻结。
状态误套qpos四槽守卫，实际state无守卫。
只修未选区间/选中位型与长度，修后未执行。
私有源/CSR/status双端与NaN、缓存寿命/恢复通过。
38实际entry捕获源/PTX零偏差，原门禁通过。
1406ms含诊断，17145572bytes仅已知子集。
Kinematics显存统计缺少，非总/峰值或公平性能。
两个cfg测试钩子不改旧生产体与计算/ABI/门禁。
所有旧目标/Clippy/SDK/作者零调用，预算不重置。
参考失败1/成功2冻结，原失败转录/脚本/合同保留。
整体结果4过1败，不宣称全部生命周期通过。
原UID/状态/正式前置资源锁不改，主变更不关闭。
完整G03/NP/Tile/双平台T4继续开放。
证据：C:/Rust/mjwarp-rs/docs/resident-mixed-mass.md。
