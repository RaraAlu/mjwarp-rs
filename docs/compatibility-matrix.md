# mjwarp-rs 兼容与验收矩阵

日期：2026-10-06。
状态：首轮实施清单。

所有项目当前均未实现。
此处指完整等价项目。
G01已有常驻辅助增量。
mocap证据见[增量报告](windows-resident-mocap.md)。
相机光源见[增量报告](windows-resident-camlight.md)。
固定肌腱见[增量报告](windows-resident-fixed-tendon.md)。
固定长度与力臂仍属子集。
空间肌腱见[增量报告](windows-resident-spatial-tendon.md)。
空间子集支持site与pulley。
它提供长度、力臂与包裹字段。
两类肌腱各用局部编号。
混合入口保留原生全局编号。
编号证据见[混合报告](windows-resident-tendon.md)。
球柱绕行见[增量报告](windows-resident-geom-tendon.md)。
GPU已支持球柱与内侧绕行。
几何尺寸已支持独立周期。
G06速度与限位仍待实现。
辅助证据不关闭G01。
下表不代表已有功能。
首版契约核对关键字段与限制。
P0仍需完整展开参数与枚举值。
P0仍需冻结全量组合清单。
声明清单覆盖895项直接注解。
枚举表达式仍需核对原生取值。
阶段与入口分支仍需Rust测试。
细节见[首版模块契约](module-contracts.md)。
等价偏差见[完整对标复审](initial-release-review.md)。
声明见[字段盘点](upstream-contract-inventory.md)。
调用见[阶段契约](physics-stage-contracts.md)。
限制见[组合清单](supported-combinations.md)。

## 功能基线

```text
upstream: google-deepmind/mujoco_warp
revision: 71da24d956378a87a703b6e1442b13aec0c4ac29
upstream_package_field: 3.15.0
checked_on: 2026-10-06
```

版本字段不代表稳定发行。
功能范围使用提交摘要锁定。
模型语义版本仍需P0单独固定。
不能直接沿用旧训练文档版本。

依据：[包元数据](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/pyproject.toml)。

## 完整对标的定义

核心覆盖冻结版GPU物理能力。
核心覆盖数据与运行时能力。
核心覆盖查询与批量渲染能力。
项目提供Rust等价接口。
接口名称不必逐字照搬。
默认值与有效调用范围必须等价。
副作用、警告与截断分别验收。
严格门面不能替代等价入口。
Python与JAX桥接不进入产品。

上游也提供CPU调试路径。
本项目仍以GPU能力作为分母。
CPU play与前端归LRsLab。
应用能力不计引擎完成率。
完整发行包与GPU范围不同。
本轮仍等待发行边界确认。

依据：[上游定位](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/README.md)。

## 上游公开接口盘点

数量：99个显式重导出符号。
不计版本字符串与隐式属性。
本节记录接口映射的起点。
它不是Rust现有API清单。

依据：[上游公开入口](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/__init__.py)。

```text
bvh: refit_bvh, refit_splat_bvh
collision_driver: collision, nxn_broadphase, sap_broadphase
collision_primitive: primitive_narrowphase
collision_sdf: sdf_narrowphase
constraint: make_constraint
derivative: deriv_smooth_vel
forward: step, discrete, euler, forward, fwd_acceleration, fwd_actuation,
  fwd_kinematics, fwd_position, fwd_velocity, implicit, rungekutta4, step1, step2
history: init_ctrl_history, init_sensor_history, read_ctrl, read_sensor,
  reset_history
inverse: inverse
io: get_data_into, make_data, put_data, put_model, reset_data,
  reset_data_keyframe
island: island
passive: passive
ray: ray, rays
render: render
render_util: create_render_context, get_depth, get_rgb, get_segmentation
sensor: energy_pos, energy_vel, sensor_acc, sensor_pos, sensor_vel
set_const: set_const, set_const_0, set_const_fixed, set_const_spring,
  set_length_range
smooth: camlight, com_pos, com_vel, crb, factor_m, flex, kinematics, rne,
  rne_postconstraint, solve_m, subtree_vel, tendon, transmission
solver: solve
support: contact_force, get_state, jac, mul_m, set_state, xfrc_accumulate
types: Model, Data, BiasType, BroadphaseFilter, BroadphaseType, Callback,
  ConeType, Constraint, Contact, CtrlChart, CtrlInput, DisableBit, DynType,
  EnableBit, GainType, GeomType, IntegratorType, JointType, ObjType, Option,
  OverflowType, RenderContext, SolverType, State, Statistic, TrnType
```

P0为每个符号登记Rust映射。
多个符号可共用一个Rust接口。
合并不能丢失物理语义。
名称覆盖不等于功能覆盖。

本轮提供[逐项映射](upstream-api-map.md)。
映射登记99个公开符号。
映射也登记30个测试模块。
模块路径均表示未来布局。
实现归属与签名仍需评审。
内部类型与内核仍需盘点。
99个符号不等于完整功能分母。

## GPU物理矩阵

| ID | 功能族 | 实施要点 | 核心验收 | 阶段 |
| --- | --- | --- | --- | --- |
| G01 | 运动学 | 位姿、关节、自由度、质心 | 解析解、四元数、索引 | P4 |
| G02 | 空间动力学 | CRB、RNE、质量矩阵 | 对称性、正定性、偏置力 | P4 |
| G03 | 质量矩阵求解 | 因子分解、乘法、线性求解 | 稠密与稀疏残差 | P4/P6 |
| G04 | 被动力 | 阻尼、弹簧及上游被动力项 | 力值、符号、能量变化 | P4 |
| G05 | 执行器 | 增益、偏置、动态、传动 | 限幅、激活状态、控制输入 | P4/P7 |
| G06 | 肌腱与传动 | 长度、速度、雅可比、限位 | 多体引用与力传递 | P4/P7 |
| G07 | 宽相碰撞 | NXN、两类SAP、过滤 | 候选完整性与容量边界 | P5 |
| G08 | 基本几何碰撞 | 上游几何对与接触规则 | 深度、法线、接触数量 | P5 |
| G09 | 凸体与网格 | GJK、EPA、CCD等上游路径 | 退化几何与多接触 | P5 |
| G10 | 地形与SDF碰撞 | 地形、体积SDF与用户距离梯度钩子 | 地形接触、梯度与越界 | P5 |
| G11 | 柔性碰撞 | 上游flex碰撞及CCD能力 | 自碰撞与容量诊断 | P7 |
| G12 | 约束构建 | 接触、关节、等式、肌腱 | 雅可比、偏置与索引 | P6 |
| G13 | 摩擦锥 | 金字塔与椭圆锥 | 摩擦方向与约束残差 | P6 |
| G14 | CG求解器 | 梯度、预条件与收敛 | 残差、迭代与失败状态 | P6 |
| G15 | Newton求解器 | Hessian、分解与线搜索 | 稠密、稀疏与退化场景 | P6 |
| G16 | Euler与RK4 | 上游积分语义 | 单步误差与步长收敛 | P6 |
| G17 | 隐式积分 | IMPLICIT与IMPLICITFAST | 速度导数与稳定性 | P6 |
| G18 | 离散积分 | DISCRETE步进映射 | 有效度量与约束一致性 | P6 |
| G19 | 分阶段推进 | forward、step1、独立step2 | 导入中间量与调用时序 | P4/P6 |
| G20 | 逆向动力学 | inverse与逆向中间量 | 正逆向回代与力平衡 | P7 |
| G21 | 平滑速度导数 | deriv_smooth_vel | 解析导数与有限差分 | P7 |
| G22 | 柔性体动力学 | 上游已实现的flex能力 | 拉伸、弯曲及耦合场景 | P7 |
| G23 | 传感器 | 位置、速度、加速度阶段 | 类型、时序与单位 | P7 |
| G24 | 历史与延迟 | 控制与传感历史、时钟 | 环形索引与局部重置 | P7 |
| G25 | 岛与休眠 | 岛构建、休眠与唤醒 | 接触唤醒与活跃自由度 | P7/P10 |
| G26 | 能量与外力 | 能量、外力累加与接触力 | 能量、参考系与力矩 | P4/P7 |
| G27 | 常量计算 | set_const与长度范围 | 输入转换与动态更新 | P2/P7 |
| G28 | 查询 | jac、ray、rays、contact_force | 单体、批量与无命中 | P7/P8 |

枚举值以冻结版类型为准。
测试还需覆盖能力开关组合。
不能只覆盖默认求解器。

依据：[上游类型定义](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py)。

## 模型与数据矩阵

| ID | 功能族 | 实施要点 | 核心验收 | 阶段 |
| --- | --- | --- | --- | --- |
| M01 | 已编译模型接入 | 原生模型视图与版本检查 | 输入完整性与缺字段诊断 | P2 |
| M02 | GPU模型转换 | 原生字段映射与GPU布局 | 常量、精度与索引转换 | P2 |
| M03 | 资产转换 | 原生资产提取与GPU预处理 | 单位、路径与内容摘要 | P2/P7/P8 |
| M04 | 结构化模型输入 | 等价模型视图与输入规范 | 与原生输入的转换一致性 | P2 |
| M05 | 模型上传 | 公共参数与世界参数 | 广播、所有权与布局 | P3/P10 |
| M06 | 状态构造 | 世界数、维度、工作区 | 默认估算、覆盖与容量检查 | P3 |
| M07 | 状态交换 | 上传、下载、状态签名与额外快照 | 保留掩码、截断与报告 | P3/P9 |
| M08 | 局部重置 | 世界掩码、关键帧 | 隔离、历史与时钟 | P3/P7 |
| M09 | 参数更新 | 按世界修改物理参数 | 常量刷新与执行图规则 | P7/P10 |
| M10 | Rust回调 | 类型回调与内核钩子 | 时序、状态与同步契约 | P7/P10 |
| M11 | 错误状态 | 溢出、NaN、迭代上限 | 诊断与失败隔离 | 全阶段 |

LRsLab负责模型编译。
引擎只接收已编译输入。
原生版本需匹配冻结功能集。
Rust转换层不得遗漏GPU字段。
完整GPU能力需要完整输入语义。

## 渲染矩阵

| ID | 功能族 | 实施要点 | 核心验收 | 阶段 |
| --- | --- | --- | --- | --- |
| R01 | 渲染上下文 | 世界、相机、资产、BVH | 全部默认参数与生命周期 | P8 |
| R02 | RGB输出 | 颜色、材质与纹理 | 布局、色彩与图像容限 | P8 |
| R03 | 深度输出 | 原始深度、缩放公式与无命中值 | 解析平面、标定与缩放边界 | P8 |
| R04 | 分割输出 | 对象ID与像素映射 | 遮挡、背景与ID稳定性 | P8 |
| R05 | 网格与地形 | 上游渲染几何支持 | 法线、变换与资产 | P8 |
| R06 | 柔性体渲染 | 动态顶点与BVH维护 | 动画与几何一致性 | P8 |
| R07 | 高斯泼溅 | splat资产与加速结构 | 透明混合与更新 | P8 |
| R08 | 异构多相机 | 分辨率、视场与内参 | 偏移表与输出隔离 | P8 |
| R09 | 光照与阴影 | 上游光源与阴影能力 | 几何场景与固定参数 | P8 |
| R10 | 批量渲染调度 | 相机与世界并行 | 独立缓冲与完成令牌 | P8/P10 |

渲染是正式对标范围。
项目不能用窗口截图替代它。
原生查看器不替代GPU批量输出。
原生离屏接口不替代批量验收。

依据：[上游批量渲染](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/README.md#batch-rendering)。

## 运行时与数据接口矩阵

| ID | 功能族 | 核心验收 | 阶段 |
| --- | --- | --- | --- |
| A01 | Rust到GPU执行 | Rust或原生内核的双平台执行 | P1 |
| A02 | NVIDIA设备管理 | 能力检查与缺失设备报错 | P1/P3 |
| A03 | 缓冲与池化 | 边界、生命周期与资源释放 | P3 |
| A04 | 多流与事件 | 读写依赖与并发安全 | P3/P10 |
| A05 | JIT或预编译缓存 | 缓存身份与失效重建 | P1/P10 |
| A06 | 执行图 | 普通执行与图重放等价 | P10 |
| A07 | 设备数据视图 | 复制路径、布局与同步契约 | P8/P10 |
| A08 | 零拷贝优化 | 上下文、所有权与跨流事件 | P10 |
| A09 | 通用调用方接口 | 控制、状态、重置、参数更新 | P9/P10 |
| A10 | 记录与性能工具 | Rust无头测速与状态导出 | P10/P11 |
| A11 | 双平台发布 | 构建、仿真、渲染、清洁部署 | 全阶段/P11 |
| A12 | 跨厂商预留 | 后端契约不绑定CUDA模型布局 | P3/P11 |

零拷贝属于优化验收项。
安全接入路径必须先完成。
若未达零拷贝，不得宣称已支持。
P0确定首版零拷贝门槛。

## 应用验收项的归属

C01至C12转入LRsLab文档。
它们不进入本crate的验收分母。
CPU play要求没有取消。
LRsLab负责策略、前端与调用。
本矩阵保留61项引擎功能族。
公开接口盘点仍保留99个符号。
应用迁出不缩减GPU对标范围。

## 上游限制与明确排除

上游限制以冻结版源码为准。
下表区分排除项与保留项。
项目不扩展上游未支持项。

| 项目 | 边界 |
| --- | --- |
| IMPLICITFAST midpoint | 不支持该midpoint特性 |
| 普通IMPLICITFAST | 仍属于对标范围 |
| PGS与noslip | 上游尚未支持 |
| 执行器与传感器PLUGIN | 上游尚未支持 |
| body插件 | 上传检查拒绝 |
| geom插件与用户SDF | 保留上游属性与距离梯度扩展 |
| Flex | 只覆盖上游已实现能力 |
| 全局物理自动求导 | 上游尚未提供 |
| 平滑速度导数 | 仍属于对标范围 |
| Python与JAX桥接 | Rust产品不提供 |
| CPU开发调试后端 | 上游提供；当前范围不交付，等待确认 |
| 自研GPU接口的MuJoCo C ABI | 不承诺对外二进制兼容 |
| 完整MuJoCo生态 | 不属于当前立项目标 |
| CPU play、策略加载与回放应用 | LRsLab负责，不纳入本crate |
| 窗口、查看器与交互前端 | LRsLab负责，不纳入本crate |
| MJCF编译与场景编排 | LRsLab负责，不纳入本crate |
| 学习框架专用适配 | LRsLab负责，不纳入本crate |

依据：[上游兼容边界](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/README.md#mujoco-api-compatibility)。

该限制表描述GPU对标范围。
GPU加载仍需独立能力检查。
C/C++接入不属于排除项。
所有产品路径均不得依赖Python。

## 完成记录格式

每项实现维护以下记录。
当前不填写虚构的测试结果。

| 字段 | 要求 |
| --- | --- |
| 功能ID | 引用本表的稳定ID |
| 上游对应项 | 提交、符号、参数及枚举值 |
| 默认与副作用 | 默认值、推导规则、阶段更新与计数 |
| Rust映射 | 实际接口与实现位置 |
| 模型与输入 | 样本身份、初始状态与种子 |
| 数值指标 | 精度、误差公式与容限 |
| 失败行为 | 错误码、诊断与资源释放 |
| 测试证据 | 命令、平台、GPU与报告 |
| 性能证据 | 冷启动、稳态、同步与显存 |
| 状态 | 未实现、实现中、已验收 |

验收必须覆盖双平台。
功能实现不等于性能达标。
CPU/GPU一致不证明上游等价。
先冻结分母，再计算完成率。
清单未冻结时不发布完成率。
矩阵未展开完不能宣称完整。
