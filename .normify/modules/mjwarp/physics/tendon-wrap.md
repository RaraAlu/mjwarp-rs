+++
uid = "f9ad621e"
id = "mjwarp.physics.tendon-wrap"
fingerprint = "a4a7d8110a01751f52b6f7289ba120e449c3e14c34a05572a33469811e9abf19"

[name]
zh = "球柱绕行CUDA辅助"
en = "Sphere and cylinder CUDA wrapping"

[description]
zh = """
GPU f32球柱绕行。保留初始长度FMA修复。运动字段通过原生误差红线。原生超差仍保留。完整T4仍待验收。
"""
en = """
GPU f32 wrapping retains the initial-norm FMA fix. Motion blocks pass the native-error policy; native tolerance exceedances remain, and full T4 is pending.
"""

[[source]]
path = "src/physics/tendon_wrap.rs"

[[source]]
path = "tests/frame393_trace.rs"

[[source]]
path = "fixtures/mjwarp-geom-tendon/trace393.py"

[[source]]
path = "fixtures/mjwarp-geom-tendon/trace393.json"

[[source]]
path = "docs/frame393-first-divergence.md"

[[source]]
path = "docs/inside-norm-order-fix.md"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::tendon_wrap::TENDON_WRAP_CUDA"

[apis.description]
zh = """
内部CUDA绕行公式。
"""
en = """
Internal CUDA wrapping source.
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

## 两处初始长度修复（2026-10-11）

产品新增inside_norm。
它先舍入x平方，再融合y平方。
inside仅用它计算l0与l1。
通用n2、f32与迭代阈值不变。
本轮没有改动两级容差。
第393帧六个接触坐标逐位一致。
其MJWarp力臂差降至8.94e-8。
原生力臂差增至2.8593e-4。
基础样本1新增两坐标1 ULP差。
因此不宣称原生误差修复。
也不宣称全样本逐位一致。

双模式局部回归各35过1败。
G01相关四目标各22项通过。
运动肌腱仍有9次字段块待验收。
缺批准二级容差仍阻止准入。
所有配置本轮只调用一次。
旧393诊断此前已调用三次。
本轮没有再次运行该诊断。
完整T4、Linux与性能本轮未测。
产品测试没有启动Python。
Clippy全目标全特性与格式通过。
文档工具检查230源码与77文档。
工具报告0处不匹配。

证据：C:/Rust/mjwarp-rs/docs/inside-norm-order-fix.md
变更：changes/2026-10-11-inside-norm-order-fix.json
完整T4主变更继续开放。

## 修复前首差追踪（2026-10-11）

本轮追踪3世界中的世界2。
肌腱4采用内侧圆柱绕行。
22个绕行输入分量逐位一致。
局部与投影端点也逐位一致。
首差位于inside的l0与l1。
两路长度各差1 ULP。
PTX显示FMA收缩顺序相反。
定点回放只固定两个初始长度。
回放六个接触坐标逐位一致。
本轮没有修改产品CUDA。
本轮没有调整两级容差。
原生MuJoCo仍为第一参考。
第二级缺容差仍记待验收。

完整上游插桩保留三字段结果。
wrap_xpos、ten_J、ten_length零失配。
本库孤立终点与常驻终点一致。
上游孤立调用未复现完整链。
因此采用完整链插桩作参考。
离线上游调用3次，后两次通过。
Rust诊断调用3次，每次2过。
两类诊断均已达到调用上限。
完整T4与Linux本轮未复测。
产品测试只读取冻结JSON。
产品测试不启动Python。

证据：C:/Rust/mjwarp-rs/docs/frame393-first-divergence.md
变更：changes/2026-10-11-frame393-first-divergence.json

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

## 修复前并列诊断（2026-10-11）

已运行冻结MJWarp同机对照。
输入与状态保持相同f32量化。
3与513世界共采样37组状态。
比较16浮点字段与3整数字段。
整数失配为零。
同步内核已跟随上游开启FMA。
第310帧力臂逐位一致。
第393帧最大力臂差2.532e-4。
诊断不使用原生统一容差。
测试通过不表示数值验收通过。
完整T4与NP指标仍开放。
性能与Linux复验仍待补齐。
离线Python只负责作者生成。
产品测试不启动Python。

已审查旧指纹对应源码差异。
旧内部f64已在拉取代码中移除。
本次不改绕行公式或容差。
源码仍用f32中间量与牛顿迭代。
证据：C:/Rust/mjwarp-rs/docs/windows-mjwarp-geom-tendon-comparison.md。

## 历史记录（旧参考顺序失效）

## T4定义更正（2026-10-11）

T4表示一个验收阶段。
先完成G01代码。
再在Windows运行本机GPU验收。
范围覆盖完整G01行为。
同时覆盖全链与低层入口。
不能缩成单条肌腱测试。
显卡型号不定义验收阶段。
本机使用4070 Ti SUPER。
主判据仍对照冻结MJWarp。
原生参考只作辅助诊断。
debug与release均需实际证据。

此前圆柱测试仅属局部诊断。
每种模式各7过1败。
该结果不代表T4整体结论。
本次未运行新的全量验收。
缺少完整对照时保留未验收。
本节替代此前错误的T4定义。
历史日志保留原始内容。

T4 is the Windows local-GPU acceptance stage after G01 implementation. It covers complete G01 and raw stage entry points, not an NVIDIA GPU model or a particular tendon. The earlier inside-cylinder native-reference results are subset diagnostics only, not the verdict for full T4 acceptance. Frozen MJWarp remains the primary oracle; this clarification does not constitute a new test run.

## 当前实现与验收边界

当前绕行公式采用GPU f32。
内侧投影与迭代使用float。
本次核对了实际CUDA源码。
本次未刷新旧源码指纹。
主验收仍需冻结MJWarp对照。
原生容差只作辅助诊断。
禁止靠内部f64修补原生误差。
禁止靠放宽容差宣称通过。

## 内侧圆柱局部诊断

此处测试内侧圆柱肌腱。
肌腱编号不定义T4验收。
T4范围覆盖Windows G01。
本机GPU：
```text
NVIDIA GeForce RTX 4070 Ti SUPER
```

源码基线：
```text
791bcc334e2742a88919b064b6e96ff19ec0bb5e
```

原生对照debug为7过1败。
原生对照release为7过1败。
两种模式均在第310帧超限。
力臂误差约为容限的9.78倍。
这些结果仅描述原生参考差异。
MJWarp并列对照尚未执行。
当前不得判定上游等价成败。

保留本次原生证据：
C:/Rust/mjwarp-rs/target/cylinder-t4-acceptance/07f8cce0decf479a99a914459b68bb51/report.json

## 后续验收

遵循根节点的逐字段验收约束。
先冻结上游模型与状态哈希。
对齐量化、精度与编译策略。
分别记录绝对差、相对差与ULP。
先固定判据，禁止事后放宽。
补齐上游对照后再关闭变更。

变更：changes/2026-10-11-mjwarp-parallel-acceptance.json

## English scope

The inspected CUDA wrapping source currently uses f32/float. Its old source fingerprint remains stale until the pulled source changes receive a proper architecture review.
The inside-cylinder diagnostic uses tendon index 4 on the local RTX 4070 Ti SUPER. This tendon index is not the definition of T4 acceptance. Both native-reference test modes passed seven tests and failed one at source frame 310. This is diagnostic evidence, not an MJWarp parity verdict.
Use frozen MJWarp side-by-side results as the primary acceptance oracle, with matched inputs, quantization, precision/compiler policies and pre-frozen per-field criteria. Upstream comparison is pending.
