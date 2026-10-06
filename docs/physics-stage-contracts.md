# mjwarp-rs 物理阶段细化契约

整理日期：2026-10-06。
版本：0.1阶段草案。
状态：源码核对，尚未实现。

本文件细化阶段输入与副作用。
本文件不设计新的物理算法。
本文件不提供可调用Rust接口。
全部语义对齐冻结上游。
资源安全约束继续保持强制。
严格门面继续采用独立入口。

相关文档：
[模块契约](module-contracts.md)、
[声明盘点](upstream-contract-inventory.md)、
[组合限制](supported-combinations.md)。

## 1. 调用方责任

调用方可以直接组合物理阶段。
阶段入口不强制Simulator门面。
调用方也可以导入已准备的Data。
库检查上下文、容量与资源寿命。
调用方负责物理中间量一致性。
严格门面可以检查物理新鲜度。
严格检查不能替代等价入口。

以下读写集合只列主要字段。
内核迁移仍需逐字段核对。
休眠与柔性路径可能写入更多字段。
本表不构成完整访问白名单。

## 2. Option构造与转换默认值

声明默认与转换默认不同。
多数Option字段读取原生模型。
Rust不能把未赋值字段默认清零。
参数覆盖不改变冻结默认值。

| 字段 | 冻结上游构造规则 | 关联矩阵 |
| --- | --- | --- |
| `timestep/gravity/wind/magnetic/density/viscosity` | 从原生选项转换为设备批量数组 | M02、G04 |
| `tolerance` | 转换后采用`max(input, 1e-6)` | M02、G14、G15 |
| `impratio_invsqrt` | 原生存在impratio时采用`1 / sqrt(max(impratio, mjMINVAL))` | M02、G13 |
| `broadphase` | 默认NXN | G07 |
| `broadphase_filter` | 默认`PLANE | SPHERE | OBB`；不默认包含AABB | G07 |
| `graph_conditional` | 默认true | A06 |
| `run_collision_detection` | 默认true | G07～G12、M09 |
| `run_rne_postconstraint` | 默认false；传感需求仍可触发后约束计算 | G02、G23 |
| `warn_overflow` | 默认ALL；false映射0；true映射ALL；整数保留位掩码 | M11 |
| `contact_sensor_maxmatch` | 优先读取同名numeric；缺省采用64 | G23、M02 |

批量参数入口只检查Model字段。
它不直接接受Option字段路径。
Option数组的扩展另查更新路径。
Rust不能假定每个星号都含W维度。

模型转换还折入执行器阻尼。
关节和肌腱路径使用gear平方。
多项式阻尼也进入对应参数。
移植时不能再重复累加同一阻尼。

依据：[选项与阻尼转换](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L422-L480)、[告警属性](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L982-L990)。

## 3. 阶段输入与主要副作用

### 位置与速度阶段

| 入口 | 主要输入 | 主要输出及副作用 |
| --- | --- | --- |
| `fwd_kinematics` | qpos、mocap与模型拓扑 | 身体、几何、相机、光源、flex与肌腱位置；休眠肌腱可触发唤醒 |
| `fwd_position` | 位置状态及选项；可含调用方接触 | 质量矩阵、可选分解、接触、约束、传动；休眠可改变活动集与岛 |
| `fwd_velocity` | qvel及位置中间量 | 执行器和肌腱速度、空间速度、被动力、偏置力；DISCRETE更新有效度量偏移 |

位置阶段factorize默认true。
完整forward显式传入false。
分步step1使用默认true。
这两条路径不能共用错误默认。

位置阶段也构建约束。
它不等于纯运动学更新。
调用方只需位姿时可选运动学入口。
位置阶段保留休眠双碰撞时序。
第二遍碰撞追加新唤醒接触。
它不清空第一遍的共享接触计数。

依据：[位置调度](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py#L1302-L1365)、[速度调度](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py#L1421-L1443)。

### 执行器、加速度与求解

| 入口 | 主要输入 | 主要输出及副作用 |
| --- | --- | --- |
| `fwd_actuation` | ctrl、act、历史、执行器长度与速度 | act_dot、actuator_force与qfrc_actuator；执行器回调可覆盖中间量 |
| `fwd_acceleration` | 应用外力、偏置力、被动力与执行器力 | qfrc_smooth与qacc_smooth；DISCRETE和休眠采用不同求解路径 |
| `solve` | 质量、约束、平滑加速度与热启动 | qacc、约束力及迭代诊断；稠密、稀疏与休眠路径分别验收 |

该阶段默认不分解矩阵。
完整forward显式传入true。
step2使用默认false。
导入Data必须携带所需分解量。
库不能隐式补做完整forward。

执行器缺失或禁用时清零输出。
该路径不触发执行器类型回调。
历史存在时读取延迟控制。
延迟读取不能覆盖原始ctrl。
执行器回调先于肌腱限幅与累加。

依据：[执行器路径](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py#L1867-L1937)、[加速度路径](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py#L2010-L2045)。

### 完整、分步与逆向入口

| 入口 | 行为边界 |
| --- | --- |
| `forward` | 检查组合；刷新正向量及三个传感阶段；不执行最终积分 |
| `step` | forward后执行指定积分器；积分不保证全部派生量同步刷新 |
| `step1` | 位置与速度阶段、对应传感及control回调；不执行最终积分 |
| `step2` | 执行器、加速度、求解、后约束传感与积分；不要求本库生成step1票据 |
| `inverse` | 读取qacc并更新逆向力及相关中间量；离散加速度转换后恢复qacc；不执行积分 |

RK4完整步使用RK4积分。
RK4分步step2仍使用Euler。
两条路径不能无条件判定等价。
INVDISCRETE不是通用积分器开关。
RK4的离散逆向路径明确拒绝。
该转换也不实现IMPLICIT路径。
普通IMPLICIT正向积分仍受支持。
Rust不能据此排除全部IMPLICIT。

依据：[完整与分步](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py#L2063-L2156)、[逆向入口](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/inverse.py#L178-L234)、[离散逆向限制](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/inverse.py#L80-L121)。

## 4. 人工接触注入

人工接触属于等价范围。
调用方可以关闭碰撞调度。
它随后填充共享接触缓冲。
位置阶段仍构建对应约束。
Rust不能强制重新检测碰撞。
Rust不能丢弃调用方接触。

推荐等价时序：

```text
run_collision_detection = false
更新所需位置中间量
写入contact及nacon
fwd_position或对应约束阶段
后续速度、执行器、加速度及求解
```

上图只表达调度责任。
每项场景仍需选取有效组合。
休眠与柔性组合另按上游限制。

| 数据 | 接入要求 |
| --- | --- |
| `nacon` | 共享池计数；报告容量溢出，保留上游计数含义 |
| `contact.worldid` | 每条接触使用有效世界索引 |
| `dist/pos/frame/dim` | 保留距离、位置、接触坐标系与约束维数 |
| `type/geom/flex/elem/vert` | 保留接触类别与有效对象引用；按对应路径核对哨兵 |
| `friction/solref/solreffriction/solimp` | 保留完整摩擦与约束参数 |
| `efc_address` | 约束阶段建立对应地址；调用方不伪造求解后的力 |

关闭碰撞调度不等于禁用接触。
CONTACT位约束物理接触路径。
显式collision检查禁用位。
该入口可能清零共享接触计数。
该选项只控制碰撞调度。
它不禁止调用方显式调用collision。

碰撞增量入口保留awake_prev。
默认None执行完整碰撞。
增量路径保留nacon并追加接触。
它仍清零宽相候选计数ncollision。
增量路径不重复执行flex碰撞。

依据：[选项说明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L937-L949)、[碰撞调度条件](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py#L1334-L1354)、[完整与增量碰撞](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/collision_driver.py#L927-L984)。

## 5. 回调参数与触发边界

回调不承诺每个step只触发一次。
RK4与休眠可能重复执行阶段。
调用方需要观察实际阶段顺序。
Rust接口保留所有七类回调。

| 回调 | 参数 | 触发位置与边界 |
| --- | --- | --- |
| `passive` | `(Model, Data)` | 被动力合并之后；SPRING与DAMPER同时禁用时提前返回 |
| `control` | `(Model, Data)` | forward及step1的速度阶段之后；ACTUATION禁用时跳过 |
| `act_dyn` | `(Model, Data)` | 执行器基础力内核之后；先于act_gain与act_bias |
| `act_gain` | `(Model, Data)` | act_dyn之后；先于act_bias |
| `act_bias` | `(Model, Data)` | act_gain之后；先于肌腱限幅与力累加 |
| `sensor` | `(Model, Data, Stage)` | 分别传入POS、VEL、ACC；SENSOR禁用时跳过 |
| `contactfilter` | `(Model, Data)` | 窄相及适用flex碰撞之后；增量碰撞也可触发 |

传感回调必须携带阶段参数。
不能把它改成统一双参数回调。
回调写入遵守资源租约。
自定义GPU内核保持明确同步。
图捕获仍需验证回调等价路径。

依据：[被动力提前返回](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/passive.py#L3390-L3401)、[被动力回调](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/passive.py#L3678-L3696)、[执行器回调](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py#L1916-L1921)、[位置传感回调](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/sensor.py#L963-L964)、[速度传感回调](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/sensor.py#L1509-L1510)、[加速度传感回调](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/sensor.py#L2843-L2848)。

## 6. 历史输入与读取

ctrlid实际表示执行器索引。
它不是全局控制通道编号。
控制维度取执行器通道数。
传感维度使用sensor_dim。
本节用S表示对象采样数。
D表示该对象的通道维度。

| 入口或参数 | 等价规则 |
| --- | --- |
| 控制初始化values | 接受`[W, S * D]`；也接受`[W, S, D]`并重排视图 |
| 传感初始化values | 当前只接受`[W, S * D]`；不暗增三维入口 |
| times | 参数必须传入；值可以为None；显式数组形状为`[S]` |
| 时间检查 | 相邻差值小于MJ_MINVAL时报错；不能只检查大于零 |
| times为None | 保留已有时间槽；更新值和游标；不默认为均匀重采样 |
| 控制用户槽 | 初始化保留原值 |
| 传感phase | 默认None保留用户槽；标量广播；数组必须为`[W]` |
| read_ctrl | 输出`[W, D]`；仅D为1时允许`[W]` |
| read_sensor | 输出`[W, D]` |
| 查询time | 每世界一个查询时间；读取时考虑对应延迟 |
| interp | 必要参数；负数采用模型模式；文档定义0、1、2插值模式 |
| 无历史对象读取 | 返回当前控制或当前传感值；初始化仍拒绝未分配历史 |

Optional注解不表示可以省略参数。
读取不改变物理时钟。
历史初始化不执行积分。
历史布局仍需核对周期参数差异。
不能只从批次0推断完整有效性。

依据：[控制读取](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/history.py#L846-L881)、[传感读取](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/history.py#L901-L967)、[控制初始化](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/history.py#L970-L1068)、[传感初始化](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/history.py#L1071-L1184)。

## 7. 查询与输出缓冲

| 入口 | 形状、默认值与输出 |
| --- | --- |
| ray | 每世界一条射线；输入`[1, 1]`或`[W, 1]`的vec3；返回三个`[W, 1]`数组 |
| rays | 输入`[1, N]`或`[W, N]`的vec3；调用方提供`[W, N]`输出 |
| ray默认过滤 | geomgroup=None采用六项-1；flg_static=true；bodyexclude=-1 |
| rays排除对象 | 使用`[N]`的逐射线bodyexclude；不是逐世界排除数组 |
| ray及rays的rc | 默认None仍提供非BVH查询路径；提供上下文时使用加速路径 |
| 射线无命中 | dist与geomid均为-1；不等同渲染原始深度的0 |
| jac | point使用全局坐标；body按世界提供；平移与旋转输出可以分别省略 |
| contact_force | 接触ID索引共享池；to_world_frame显式选择坐标系；调用方提供输出 |
| mul_m | 支持可选skip与替代M；不能简化成唯一固定质量矩阵入口 |

查询读取现有派生缓冲。
查询不自动执行完整forward。
查询不推进物理时钟。
图执行使用可复用输出缓冲。
上游分配便利入口仍保留等价语义。

依据：[射线入口与形状检查](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/ray.py#L1179-L1264)、[雅可比入口](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/support.py#L583-L611)、[接触力入口](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/support.py#L445-L473)。

## 8. 实施交付与剩余项

P2首先实现选项及字段转换。
P4至P7逐步落地阶段入口。
P8实现渲染及查询互操作。
P10验证图捕获与工作区复用。
各阶段均保留双平台验收。

首轮测试采用MC31至MC42。
它们目前只登记测试任务。
主要字段表不替代完整读写集合。
内部工作区shape仍需逐构造核对。
所有GPU阶段均尚未实现。
