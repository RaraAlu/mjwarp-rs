# 平滑力四项常驻合并

日期：2026-10-14。
范围：U016/G03/G26辅助子集。
本轮实现平滑力(qfrc_smooth)合并。
本轮不执行完整加速度阶段。

## 计划依据与实现

P4要求forward阶段中间量。
U016对应`fwd_acceleration`。
依据见[实施计划](implementation-plan.md#p4-空间数学与基础动力学)。
契约见[物理阶段](physics-stage-contracts.md)。

```text
src/model/smooth_force.rs
src/physics/smooth_force.rs
```

新输入提供以下类型：

```rust
SmoothForceInput
```

它接收body_treeid与dof_bodyid。
它另接收ntree计数。
它保留原顺序与重复体引用。
它允许静态树编号-1。
它只检查维数与索引边界。
它不证明完整原生拓扑。

新物理入口提供以下类型：

```rust
SmoothForcePlan
SmoothForceData
SmoothForceOutput
SmoothForceOptions
SmoothForceField::{Passive, Bias, Actuator, Applied}
```

计划保留拓扑并编译模板。
数据上传拓扑与树状态。
数据独立持有四项力与树状态。
调用方显式更新各项力字段。
调用方也可单独更新世界字段。
合法写入废弃本合并结果。
非法输入或外来计划保留结果。
`update`计算全批合并结果。
`readback`只发布整批有限快照。
快照不依赖计划或会话寿命。
接口不提供CPU物理回退。

## 冻结算序与精度

本地冻结上游保持以下提交：

```text
71da24d956378a87a703b6e1442b13aec0c4ac29
```

`forward.py:1973–2035`定义该合并。
`types.py`定义五项f32力字段。
Warp原生头定义f32加减。
本轮只读取这些源码。
本轮不启动Python或参考作者。

GPU严格采用以下顺序：

```text
first  = sub.rn.f32(passive, bias)
second = add.rn.f32(first, actuator)
smooth = add.rn.f32(second, applied)
```

每步单独舍入为f32。
产品不提升为f64。
产品不重排或一次性求和。
大数消减样本明确区分两者。
一次f64和得到零时，样本仍要求一。
连续半ULP样本要求逐步偶数舍入。
活动分支保留计算得到的负零。

| 量 | 映射 |
| --- | --- |
| passive、bias、actuator、applied | f32 |
| packed力缓冲与三步局部量 | f32 |
| qfrc_smooth输出 | f32 |
| body_treeid、dof_bodyid、tree_awake | i32 |
| 选项 | 两个bool子集，不冒充原生位掩码同型 |
| 诊断 | 固定位型与宿主逐步f32证书；不反馈产品 |
| 编译 | NVRTC12.8，compute70 |
| 选项 | fmad开启，fast_math关闭 |

本轮保留原256同步ABI。
本轮保留原f32精度门禁。
浮点输入按field/world/dof连续。
结果按world/dof连续。
元数据保留完整体映射。
它另存逐世界tree_awake。
每个设备缓冲都有前后四槽守卫。
宿主加乘与容量采用checked。
工作项上限保持`u32::MAX-255`。
元数据长度不超过i32上限。

## 休眠与错误边界

选项采用如下逻辑：

```text
enable_sleep = sleep_enabled && !island_disabled
body = dof_bodyid[dof]
tree = body_treeid[body]
if enable_sleep && tree >= 0 && tree_awake[world, tree] == 0:
    smooth = +0.0f32
    return
```

休眠分支必须写入正零。
它不保留旧结果位型。
它不采用mul_m的skip语义。
静态树-1始终计算四项力。
岛禁用时关闭本休眠分支。
树状态只按是否等于零分支。
上游类型文档标注0与1。
本轮额外测试-11、2等非零值。
这些值不表示完整休眠状态机。

GPU先检查映射与全部守卫。
它在解引用前检查体与树索引。
它检查活动DOF的力值有限性。
它不读取休眠DOF的力数据。
休眠NaN或中间溢出不阻止写零。
活动NaN、Inf或结果溢出必须拒绝。
坏输入或结果不发布部分快照。
ntree为零时允许全静态DOF。
空DOF仍属于严格入口拒绝范围。

## 独立样本与定向实测

本轮预先手写十八组位型参考。
参考独立存储输入与预期位型。
它没有调用候选或参考作者。
它不是一次f64总和的输出。
它也不是完整原生模型结果。
参考覆盖以下边界：

- 二进制有理数正常加减。
- 大数消减与首步舍入。
- 连续半ULP逐步舍入。
- 活动正零与负零。
- 最小次正规数与正负次正规链。
- 最小正规数减最大次正规数。
- 最大有限数与有限消减。

算子位型容差预冻结为零。
本轮没有放宽任何旧容差。
本轮没有减少任何旧覆盖。
上游forward_test采用5e-4。
它来自5e-5乘十的MuJoCo对照。
本轮不借它放宽位型门槛。
新证书只放行本算子样本。
它不等于完整NP验收。

| 实际覆盖 | 结果 |
| --- | --- |
| nv | 1、65、513 |
| nbody、ntree | 273、129 |
| 世界规模 | 1、2、257 |
| 样本轮换 | 2 |
| SLEEP/ISLAND组合 | 4 |
| 主批数 | 72 |
| 主输出槽 | 1204320 |
| 位差 | 0 |
| 休眠正零槽 | 54481 |
| 活动负零槽 | 63732 |
| 静态活动槽 | 114400 |

各世界采用不同的样本轮换。
测试不把世界输入隐式广播。
513自由度跨越256线程边界。
129树状态也采用独立世界行。
十八组样本均有活动分支覆盖。

新状态测试逐一更新四项字段。
它验证世界256的真实写入。
它检查未选输入与守卫位型。
树状态写入也保留未选整数位型。
它验证休眠清零与非零重新活动。
它检查非法长度、越界与NaN。
它也检查外来计划与快照寿命。
该测试不包含qpos或旧RHS生产者。
它不替旧桥接状态测试放行。

私有测试另核对以下故障：

- 力、整数元数据、结果与status守卫。
- 坏体编号与坏树编号。
- 活动力的NaN与Inf。
- 活动分支的有限输入中间溢出。
- 休眠坏载荷不读取且写正零。
- 故障修复后的恢复与整批发布。

## 调用预算与产物

| 新调用 | 结果 |
| --- | --- |
| default lib新model与physics | 4过，0.00秒，197过滤 |
| CUDA lib新gpu_双函数 | 2过，2.73秒，276过滤 |

编译分别耗时3.35与6.02秒。
新用途合计2/3，成功后冻结。
本轮不追加第三次调用。
准备失败仍须计入同用途预算。
本轮没有运行旧测试或Clippy。
旧作者与SDK调用数均为零。

主阶段计时2463毫秒。
它包含计划JIT、分配与传输。
它也包含读回、诊断与新状态测试。
四个Data缓冲容量2773732字节。
该数包含三个f32与一个i32缓冲。
它不表示峰值显存或公平性能。
它不包含驱动与JIT模块资源。
实测采用Windows调试模式。
GPU采用RTX 4070 Ti SUPER。
驱动发行版本596.36。
Linux、发行模式与T4均未执行。

本轮保留四份入口回执。
它们对应两个唯一PTX。
实际源与PTX哈希全部匹配。
每份PTX含一次sub.rn与两次add.rn。
它们不包含浮点FMA或f64运算。
模板只有有效休眠选项变化。
运行维数不进入生成源文本。
同名回执复用完全相同源。
回执数不冒充JIT调用次数。

## 伴随与未完成项

Normify保留原树与UID。
model.sleep保持活动态。
core与root继续保持计划态。
主变更保持进行态。
正式前置与资源锁保持。
架构校验不代替产品验收。
本次架构校验零错误。
它另报告十二项粗叶警告。
刷新后源码指纹没有漂移。

本轮只交付四力合并辅助。
它不提供xfrc_accumulate。
它不提供qacc_smooth。
它不实现DISCRETE或休眠求解。
完整fwd_acceleration仍未实现。
完整G03与NP01至NP04仍开放。
旧bridge状态失败修后未执行。
旧bridge与mul_m预算继续冻结。
mul_m仍未借用CRB设备M。
旧Sparse86/390原源覆盖缺口保持。
旧六回执不是逐次完整捕获。
本轮不修旧记录器或补编译。
本轮没有推送、部署或发布。

参考见[固定位型](../fixtures/smooth-force/reference.json)。
预检见[合同](evidence/smooth-force/contract.json)。
统计见[实测](evidence/smooth-force/numeric.json)。
产物见[回执索引](evidence/smooth-force/kernel-index.json)。
边界见[审计](evidence/smooth-force/audit.json)。
来源见[映射](evidence/smooth-force/map.json)。
