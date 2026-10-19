+++
uid = "da95e316"
id = "mjwarp.runtime.transfer.kernel"
tags = ["runtime", "prototype"]
fingerprint = "7297d905cb485b8ec19184da2ebe4a4bf052824098c55b960b8e2a93c9ef9740"

[name]
zh = "同步内核适配"
en = "Synchronous kernel adapter"

[description]
zh = """
管理内部同步内核ABI。检查物理f32 PTX。FMA跟随冻结上游。
"""
en = """
Private synchronous kernel ABI with f32 physics PTX gating and frozen Warp-compatible FMA fusion.
"""

[[source]]
path = "src/runtime/transfer/kernel.rs"

[[source]]
path = "src/runtime/transfer/kernel/kinetic_energy.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::transfer::kernel::SynchronousKernel"

[apis.description]
zh = """
内部同步内核ABI。柔体边另用三个浮点输入。调用方证明访问范围。
"""
en = """
Private synchronous CUDA ABIs, including three-read-only-f32-input flex edges. Caller proves shader widths and bounds.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::transfer::kernel::SynchronousKernel::launch_kinetic_energy32"

[apis.description]
zh = """
内部32线程动能ABI。 检查四会话与精确容量。 调用方证明实际内核契约。
"""
en = """
Private32-thread kinetic ABI, four sessions and exact capacities checked; caller proves actual kernel contract.
"""

[[deps]]
kind = "call"
to = "mjwarp.runtime.transfer.core"
from_api = "rpc:mjwarp_rs::runtime::transfer::kernel::SynchronousKernel"
to_api = "rpc:mjwarp_rs::runtime::TransferSession"

[deps.label]
zh = "复用会话与完成检查"
en = "Owned session completion"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::runtime::transfer::kernel::SynchronousKernel"
to_api = "rpc:mjwarp_rs::diagnostics::TransferError"

[deps.label]
zh = "报告内核与会话错误"
en = "Kernel and session errors"
+++

## 2026-10-14 RNE前向私有ABI

新增launch_rne_cacc。
六指针包含四个f32只读输入。
输出独占并保留守卫。
开关转换为u32零或一。
上游诊断仍采用单字节bool。
本ABI不直接混用上游数组结构。
启动检查会话、非空与世界数。
同一会话锁序列化并同步返回。
编译精度、融合与选项保持。
新增方法改变本文件来源哈希。
旧冻结T4清单保留原始哈希。
本轮不继承旧整体验收。
新RNE实编译产物单独记录。
证据：C:/Rust/mjwarp-rs/docs/rne-acceleration.md。

## 2026-10-14 三角控制域实核

本轮未发现产品三角缺陷。
本轮不改产品公式或编译选项。
源码局部与累加继续采用f32。
两侧面PTX各含四条库归约链。
库内部采用整数-f64-f32转换。
该f64量影响物理输出。
它不属于纯诊断计算。
不能称全部编译指令采用f32。
有限小坐标也能触发慢归约。
独立宿主目标两次，后轮四过。
首轮文本断言失败如实保留。
旧满额选择均未执行。
当前机器码及数值仍未取证。
当前位差未知，前轮2078保留。
原容差、覆盖与21系数范围保持。
主排程与原模块状态保持。
证据：C:/Rust/mjwarp-rs/docs/t4-face-trig-control.md。

## 2026-10-13编译证据测试挂接

仅新增cfg(test)模块挂接。
测试复用实际PTX门禁与记录器。
测试不创建GPU会话。
实际生产编译选项及ABI保持不变。
十个含刚体模板的变体重新编译。
另20个旧变体核实后沿用。
30份产物通过原PTX精度门禁。
物理和性能验收仍未重跑。
证据：C:/Rust/mjwarp-rs/docs/t4-rigid-recapture.md。

## 当前精度策略（2026-10-11）

物理编译入口强制检查PTX。
产品全链保持f32。
FMA跟随Warp 1.15默认开启。
fast-math继续关闭。
库级角度约简允许两种拼写。
常量与寄存器链必须精确匹配。
门禁仍拒绝物理f64与低精度。
回执记录源码、PTX与编译选项。
GPU融合运算测试已通过。
内核适配专项7项通过。
ABI宽度测试独享门禁豁免。
产品编译入口没有此豁免。
此项不表示完整NP验收。

已审查旧指纹至当前的差异。
范围仅限本文件。
旧ABI与隔离语义保持不变。
新增精度门禁和编译证据。
证据：C:/Rust/mjwarp-rs/docs/windows-mjwarp-geom-tendon-comparison.md。

## 历史记录

仅登记内部同步内核适配。
它不解释物理字段。
它不接收外部裸指针。
unsafe调用方证明ABI与访问。
编译错误保持传输健康。
后端失败复用会话隔离。
完成不确定时保留内核模块。
真实GPU核对正常与拒绝路径。
测试仅模拟完成不确定状态。
本轮不注入真实GPU故障。

证据：C:/Rust/mjwarp-rs/docs/windows-kinematics-probe.md。

## 历史同型宽度扩展

固定ABI采用一个i32缓冲。
该批三个缓冲采用同一标量。
可信调用方选择f32或f64。
unsafe调用方证明源码与宽度。
编译完成后不借用源码字符串。
设备测试保留f64低位差异。
通用安全内核入口仍不开放。

证据：C:/Rust/mjwarp-rs/docs/windows-mass-solve-probe.md。

## 混合类型内部ABI

三个缓冲类型独立选择。
球柱整数段读取f32暂存。
它直接写入i32对象与地址。
GPU另验证f64参数混合路径。
可信调用方显式固定实际宽度。
所有权与完成隔离保持不变。

证据：C:/Rust/mjwarp-rs/docs/windows-resident-geom-tendon.md。


## 2026-10-14 受限128线程入口

新增内部launch_sparse_solve。
元数据i32，RHS/因子/解f32。
每世界每RHS一个128线程块。
检查四会话、维度与受检容量。
私有调用方证明元数据访问范围。
旧通用256线程launch逐字不改。
新ABI首轮1过0.22秒，1/3。
14拒绝及768位置/共享屏障通过。
编译策略和原门禁保持不变。
共享文件来源漂移另列新映射。
旧RNE/T4回执保持原绑定。
不继承完整机器码或NP验收。
证据：C:/Rust/mjwarp-rs/docs/resident-sparse-solve.md。


## 2026-10-14 Small64内部ABI

新增launch_small_factor受限入口。
每线程一个世界/常量Small块。
固定64线程，工作域按组计数。
检查四会话、四容量与维度整除。
元数据三元组由私有调用方证明。
原128和256方法逐字不变。
新64ABI首轮一过0.22秒，1/3。
14拒绝与128位置/共享槽同步通过。
共享源新指纹不继承旧验收。
编译选项与f32门禁不改变。
证据：C:/Rust/mjwarp-rs/docs/small-mass-prefix.md。

## 混合128只读前缀ABI

新增launch_mixed_solve。
因子stride包含稠密前缀/CSR/逆对角。
检查四会话、四容量、组数与32位界限。
每世界每RHS独占一个128线程块。
私有调用方证明路由/对角/更新/Small元数据。
旧128/256/64方法与编译门禁保持。
新ABI独立14拒绝/774位置/共享128屏障待测。

## 混合128ABI实测收尾

新单函数首轮1过0.22秒/262过滤。
14会话/维度/容量拒绝，774输出位置与128共享屏障。
此独立purpose1/3成功冻结，无旧ABI重放。
新方法因子stride含prefix，旧128/256/64与编译门禁保持。
共享文件指纹改变，历史T4回执保留原绑定，不继承新全NP。


## 2026-10-14 默认32动能消费者

U047/G23/G26新增严格f32辅助，P4能量原型不冒充P7完整sensor调度。
qvel/Mqvel声明同model/state，一次成对上传，world256保留未选位型。
外部标签不认证Mqvel=M(state)qvel；完整CRB→M→Mqvel设备接线仍缺。
Warp1.15默认32，register lane+reg*32，首项初始化，无效尾槽不加；shuffle16/8/4/2/1。
NV1..1024受限支持，GPU数值只测到513，1024只有宿主边界。
势能位型复制，状态只标动能；严格有限势能边界不同于完整上游入口。
源/保护槽/溢出不部分发布，完整暂存有限后交换，快照独占标签与结果。
新host2过0.00秒/203过滤，CUDA新consumer/private/ABI3过1.81秒/284过滤，无编译警告。
新combined2/3成功冻结，第三次不调用，旧producer/测试/作者/SDK/Clippy0调用。
48批4160世界；kinetic与potential各4160槽零位差，手写证书另12世界。
独立正对角质量与dyadic速度f64诊断；nv31/33树序列差、负零1/7/65与minsubnormal513。
新32ABI核对32lane与257world块、四会话和21拒绝；旧内核方法只加mod声明。
17实际entry/9唯一CU/PTX哈希一致，非逐JIT；nv33/65各FMA1，其余产品nv0，各shuffle5/ballot1，无f64/atomic。
完整冻结energy_vel generatedentry仍缺，不借其他模板放行NP。
458预规范化身份零漂移；其中20旧MJB的UTF8模式不是raw身份，保留原清单。
另做测试后20MJB原始基线Gitblob/磁盘核对全同，不冒充预跑快照；consumer不读MJB。
449旧Git过滤blob不变，另2文件只加module/reexport，旧容差/coverage/Cargo/gates保持。
1414ms含JIT/传输/host诊断，自有四float+一int容量1060000bytes，非peak/fair。
完整U047、NP01..NP04、Linux/release/T4、生产者借用与旧失败用途仍开放/冻结。
证据C:/Rust/mjwarp-rs/docs/kinetic-energy.md。
