# 世界空间外力累加辅助

日期：2026-10-14。
范围：P4、U075/G26辅助子集。
本轮新增空间外力累加(xfrc_accumulate)。
它不执行旧位置或力生产者。

## 计划与接口

P4明确要求外力功能。
U075登记xfrc_accumulate。
U016在四力合并后调用它。
依据见[实施计划](implementation-plan.md#p4-空间数学与基础动力学)。
映射见[上游入口](upstream-api-map.md)。

```text
src/physics/applied_force.rs
```

```rust
AppliedForcePlan
AppliedForceData
AppliedForceOutput
AppliedForceGeometry::{Xipos, SubtreeCom, Cdof}
```

计划接收parent、root与dofbody索引片。
世界parent必须为零。
父体必须先于子体。
该顺序排除父链循环。
根索引必须在范围内。
根索引也须匹配实际父链。
DOF体引用必须在范围内。
这些检查不证明完整原生模型。

调用方上传世界几何与六分量力。
调用方显式写入当前广义力。
`accumulate`真正累加当前值。
重复调用再次累加，不重置种子。
`replace`覆盖当前值。
世界写入保留未选输入位型。
非法输入或外来计划保留结果。
快照不依赖数据、计划或会话寿命。
接口不提供CPU物理回退。

## 冻结语义与精度

冻结提交保持如下基线：

```text
71da24d956378a87a703b6e1442b13aec0c4ac29
```

`support.py:260–326`定义该算子。
Warp vec.h定义dot与cross模板。
spatial_vector实际采用vec6别名。
本轮只读取这些文件。
本轮不运行Python或参考作者。

| 量 | 映射与顺序 |
| --- | --- |
| xipos | 世界惯性质心位置，f32 |
| subtree_com | 世界根子树质心，f32 |
| cdof | `[angular, linear]`，f32 |
| ft | `[force, torque]`，世界分量，f32 |
| jac | `[linear, angular]`，f32 |
| offset | xipos减root对应subtree_com，f32 |
| cross、dot与accum | f32 |
| delta、current与staged | f32 |
| parent、root与dofbody | i32 |
| add标记 | bool转私有u32参数，不冒充原ABI同型 |
| 参考 | f64虚功诊断，不反馈产品 |

每DOF按bodyid升序遍历。
遍历从该DOF所属体开始。
它沿真实父链筛选子树。
它忽略世界体的外力。
六项ft数值全零时跳过该体。
正零与负零都满足该判断。
本轮不使用原子累加。

```text
offset = xipos[body] - subtree_com[root[body]]
cross_term = cross(cdof.angular, offset)
accum += dot6([cdof.linear, cdof.angular], ft)
         + dot3(cross_term, ft.force)
```

六维dot从正零顺序累加。
三维dot保持三乘积左结合。
cross保持原三项乘减表达式。
产品保留C++的FMA许可。
产品不私自改变融合侧。
产品不添加内部f64分支。
原NVRTC12.8与compute70保持。
fmad开启，fast_math关闭。
原256同步ABI与f32门禁保持。

全部体、DOF与根偏移采用64位。
宿主检查所有计数与加乘。
它检查守卫容量与工作项上限。
元数据长度不超过i32上限。
空DOF仍属于严格入口拒绝范围。

## 事务发布与严格边界

GPU先验证源与索引。
它验证完整几何和ft字段有限性。
该要求也覆盖未使用字段。
它比上游零ft早退更严格。
本入口不冒充完整兼容门面。

GPU先计算独立delta。
宿主只检查其有限性与守卫。
检查不反馈物理计算。
GPU再计算当前力加delta。
replace路径只复制delta。
GPU写入独立staged缓冲。
只有全批结果合格才交换缓冲。
错误不会部分改变current。
最终加法保持一次f32 RN舍入。
中间缓冲也始终保持f32。

add路径要求当前力有限。
replace只检查当前缓冲守卫。
replace不把旧NaN载荷当输入。
结果溢出必须阻止发布。
旧快照仍保留独立宿主值。

## 独立参考与本次验证

本轮手写四体位型参考。
它包含多根、子体与世界体。
三维角、线与力矩分量各不相同。
非根COM故意采用不同坐标。
参考区分根索引与体索引。

```text
delta: [4, -12, -40, +0]
added: [14, -32, -39.75, +0]
```

宽批参考另采用独立虚功表达式：

```text
linear dot force + angular dot(torque + offset cross force)
```

它不共享GPU的jac重排代码。
它使用f32输入的f64诊断值。
二进制有理数结果均精确可表示。
主批位型容差预冻结为零。
本轮不改变任何旧容差或覆盖。

| 覆盖 | 实际结果 |
| --- | --- |
| nbody | 8、273 |
| nv | 1、65、513 |
| 世界规模 | 1、2、257 |
| 状态 | 2 |
| 模式 | add、replace |
| 主批 | 72 |
| qfrc比较槽 | 1204320 |
| delta比较槽 | 1204320 |
| 两字段合计 | 2408640 |
| 主批位差 | 0 |

两字段分别逐槽比较。
合计数不冒充单一输出数量。
各世界采用不同力与种子。
273体与513DOF跨256边界。
模型包含多根与重复DOF体引用。
测试也覆盖世界体力忽略。

新状态测试覆盖以下行为：

- 重复add真实累加。
- replace恢复纯投影结果。
- 三种几何的世界256写入。
- ft与当前力的世界256写入。
- 未选输入与守卫位型保持。
- 非法长度、越界与非有限值拒绝。
- 外来计划拒绝与独立快照寿命。
- 全负零ft跳过且delta写正零。

私有测试另注入力与几何故障。
它检查全部六个Data缓冲守卫。
它也检查元数据守卫和坏引用。
中间溢出与最终溢出均阻止发布。
每次失败均核对current原位型。
旧NaN可通过replace覆盖。
故障修复后可以继续发布。

## 融合诊断与证据缺口

已有缓存缺少完整_apply_ft入口。
其他dot/cross产物不能替代它。
本轮不重编译旧冻结产物。
本轮不伪造冻结设备对照。

额外样本隔离三维dot融合侧。
它预登记两个可能的编译位型。
它仅诊断C++收缩选择。
新abs/rel2e-7在执行前冻结。
它不放宽旧原生容差。

实际输出为正零`00000000`。
独立f64值为`-2^-46`。
绝对误差1.4210854715202004e-14。
该样本没有上游位型通过结论。
它也不进入主批零位差声明。
完整冻结生成核与NP证据仍缺。

## 实际调用与资源

| 新调用 | 结果 |
| --- | --- |
| default lib新模块 | 2过，0.00秒，201过滤 |
| CUDA lib新gpu_双函数 | 2过，5.12秒，280过滤 |

编译分别耗时3.67与5.87秒。
新用途合计2/3，成功后冻结。
本轮不追加第三次调用。
本轮不执行任何旧生产者或测试。
Clippy、SDK与参考作者均未调用。

主阶段计时4747毫秒。
它包含计划JIT与宿主诊断。
它也包含分配、传输与新状态验证。
自有计划与Data容量8119492字节。
该数包含六个f32 Data缓冲。
它也包含一个i32计划元数据缓冲。
它不表示峰值显存或公平性能。
它不统计驱动与JIT模块资源。
实测采用Windows调试模式。
GPU为RTX 4070 Ti SUPER。
驱动发行版本596.36。
Linux、发行模式与T4均未执行。

本轮保留三个入口回执。
它们对应一个唯一PTX。
实际源、PTX与回执哈希均匹配。
源不包含按维度替换的宏。
同名文件始终复用完全相同源。
回执数不冒充逐次JIT记录。
PTX含八项FMA与七项浮点乘法。
它另含六项减法与三项加法。
它不包含f64浮点运算或原子操作。

## 伴随与未完成项

Normify保留原树与UID。
core与root仍保持计划态。
主变更继续保持进行态。
原运行占用与正式前置保持。
本轮只扩展原physics.core源码。
本轮不修改model或runtime代码。
完整U075与fwd_acceleration仍缺。
qacc_smooth、DISCRETE与休眠求解仍缺。
完整G03与NP01至NP04仍开放。
旧mul_m尚未借用CRB设备M。
旧Sparse原源覆盖缺口仍保留。
旧bridge状态失败修后仍未执行。
全部旧成功、失败与满额用途冻结。
本轮没有推送、部署或发布。

参考见[固定位型](../fixtures/applied-force/reference.json)。
预跑见[合同](evidence/applied-force/contract.json)。
数据见[统计](evidence/applied-force/numeric.json)。
产物见[回执](evidence/applied-force/kernel-index.json)。
边界见[审计](evidence/applied-force/audit.json)。
来源见[映射](evidence/applied-force/map.json)。
