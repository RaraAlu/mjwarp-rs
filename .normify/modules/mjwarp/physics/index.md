+++
uid = "28396e89"
id = "mjwarp.physics"
state = "planned"
tags = ["planned", "engine", "physics"]
fingerprint = "pending"
source = []

[name]
zh = "物理阶段"
en = "Physics stages"

[description]
zh = """
G01入口已实现。原生MuJoCo优先。超差后采用MJWarp原生误差红线。余量为5%。完整T4仍待验收。
"""
en = """
Native MuJoCo remains the first oracle. Only native failures compare candidate and MJWarp errors against the same native baseline, with an approved 5% relative error redline. Full T4 remains pending.
"""
+++

## 现行原生误差红线（2026-10-11）

用户在本轮测试前批准5%余量。
当前规则为native_error_v2。
第一参考始终为原生MuJoCo。
原生达标直接放行该项NP03。
仅原生超差后启用同条件MJWarp。
两路误差都相对同一原生参考。
每字段块取最大归一化误差。
分母沿用原生绝对加相对容差。
本库不劣于MJWarp时对照放行。
误差略高但未越5%红线也放行。
红线等于MJWarp误差乘1.05。
等于边界允许通过。
5%不作用于物理输出值。
本轮没有放宽原生容差。
本轮也没有修改产品物理内核。

越过红线严格禁行。
缺少必要参考仍待验收。
身份、条件或非有限异常禁行。
零数值证据不能通过汇总。
独立诊断明确不作准入依据。
NP01、NP02与NP04继续独立。
报告升级schema 2。
它明确记录准入标志与原因。
mjwarp误差现在也指向原生基准。
旧值差报告不能沿用新语义。

GNU宿主规则20项通过。
MSVC双模式专项各35过0败。
其中G01各7过，几何肌腱各8过。
宿主规则各20过，不算物理证据。
各配置本轮只调用一次。
运动肌腱18块原生放行。
另9块对照放行，没有红线内放行。
393帧本库分数12.236763。
MJWarp分数12.240589。
所以该字段放行，但仍原生超差。
本轮改变规则，不改变物理结果。

完整T4、Linux与性能未重测。
旧393诊断已满三次，不再运行。
质量求解仍缺同条件第二参考。
完整T4主变更继续开放。
产品测试没有启动Python。
全特性全目标Clippy与格式通过。
文档工具检查230源码、78文档。
工具报告0处不匹配。
本轮不激活计划架构。
布局层级、API和依赖保持原样。

证据：C:/Rust/mjwarp-rs/docs/native-error-redline-acceptance.md
契约：C:/Rust/mjwarp-rs/docs/numerical-precision-contract.md
变更：changes/2026-10-11-acceptance-verdict-labels.json

## 以下保留历史阶段证据

历史数值与失败记录保持原样。
若历史门禁冲突，以顶部规则为准。

## 历史值差门禁（现行规则见顶部）

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
未审查源码不刷新历史指纹。

Native MuJoCo is the first numerical reference. A native-tolerance pass is sufficient for the corresponding NP03 check; no MJWarp numerical pass is additionally required. Only a native-tolerance failure invokes frozen MJWarp as the second reference. A second-reference pass also passes NP03. Both failing means failure; missing required references or frozen tolerances means unverified. NP01, NP02 and NP04 remain independent. Automated gates now cover G01 main/raw fields, motion tendon blocks and G03 auxiliary solves. Missing second references or approved tolerances remain pending and block acceptance. The user chose mechanism-first implementation without freezing secondary tolerances; physical kernels and native tolerances are unchanged.

补测：C:/Rust/mjwarp-rs/docs/windows-two-reference-acceptance.md
契约：C:/Rust/mjwarp-rs/docs/numerical-precision-contract.md
变更：changes/2026-10-11-mjwarp-parallel-acceptance.json

## 基础源码落地

2026-10-07：补齐基础行为。
HistoryLayout检查三维范围。
时间阈值由调用方明确传入。
等于阈值的间隔保持合法。
辅助检查不改写历史槽位。
MC-38与MC-39只取得局部证据。
GPU历史与物理阶段仍待实现。
本轮不激活产品模块。

证据：C:/Rust/mjwarp-rs/docs/foundation-architecture.md。

## Windows刚体GPU子集

七项刚体结果已有真实GPU证据。
四种关节保留偏心旋转。
参数共享一份，状态按世界展开。
静态原生参考独立生成。
产品链不运行CPU运动学。
同步输出检查哨兵与有限性。
完整运动学与其他分支仍待实现。
U018、U062与完整G01继续计划。
本轮不激活产品物理模块。

证据：C:/Rust/mjwarp-rs/docs/windows-kinematics-probe.md。

## Windows质心GPU子集

子树质量与质心均在GPU计算。
GPU复用既有运动学缓冲。
cinert与cdof取得独立参考证据。
零质量遵循冻结Warp语义。
原生CPU阈值回退不适用。
微小正质量另用解析参考。
接口仍限制一组共享模型。
U057与完整G01继续计划。
不激活完整产品物理模块。

证据：C:/Rust/mjwarp-rs/docs/windows-com-position-probe.md。

## Windows刚体质量矩阵子集

GPU复用质心阶段设备结果。
GPU累加十项复合惯量。
矩阵采用稠密对称宿主视图。
自由度armature只加到对角线。
独立原生与解析参考验证子集。
不叠加肌腱与执行器惯量。
矩阵入口不提供分解与求解。
稀疏布局继续保持计划。
完整crb与G02仍保持计划。
本轮不激活产品物理模块。

证据：C:/Rust/mjwarp-rs/docs/windows-mass-matrix-probe.md。

## Windows正定质量求解子集

GPU内连续执行f64刚体阶段。
反向LDL产生L、D与倒数。
多右端项共用同一分解。
主元失败保留世界与自由度。
公开结果仍提供只读f32视图。
完整物理核心继续保持计划。

证据：C:/Rust/mjwarp-rs/docs/windows-mass-solve-probe.md。
