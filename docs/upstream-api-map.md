# MJWarp上游映射草案

核对日期：2026-10-06。
状态：首版契约与映射草案。

## 基线与统计口径

```text
repository: google-deepmind/mujoco_warp
revision: 71da24d956378a87a703b6e1442b13aec0c4ac29
public_exports: 99
source_modules_for_exports: 21
test_modules: 30
```

本轮只读取上游源码。
本轮不执行上游Python。
入口显式重导出99个符号。
其中26个来自类型模块。
其余73个来自功能模块。
统计不含版本字符串。
统计不含隐式模块属性。

依据：[公开入口](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/__init__.py)。

测试统计使用以下口径：

```text
mujoco_warp/**/*_test.py
```

上游树包含30个匹配文件。
测试模块数不等于用例数。
本轮尚未逐条展开测试。

依据：[冻结代码树](https://github.com/google-deepmind/mujoco_warp/tree/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src)。

## 映射规则

下表只提出实现归属。
模块路径表示未来布局。
它们不是当前Rust API。
所有对应能力仍待实现。
原始签名见[声明盘点](upstream-contract-inventory.md)。
Rust签名仍需原型评审。
多个符号可以合并接口。
合并不得丢失物理语义。
映射保留默认值与阶段副作用。
等价入口不强制严格门面检查。
矩阵ID沿用[兼容矩阵](compatibility-matrix.md)。

U编号跟随本次入口顺序。
后续追加记录，不复用编号。
变更映射时保留上游符号。
类型阶段涵盖解析与执行。
类型映射不等于功能验收。
首版契约修正部分实现归属。
Data与缓冲布局归模型层。
物理历史与常量刷新归物理层。
物理回调不进入通用运行时。
公开BVH refit归渲染层。
算法与数据容器分别管理。
依据见[首版模块契约](module-contracts.md)。
偏差处理见[完整对标复审](initial-release-review.md)。

## 99个公开符号

| 编号 | 上游模块 | 上游符号 | 计划Rust模块 | 矩阵ID | 实施阶段 |
| --- | --- | --- | --- | --- | --- |
| U001 | `forward` | `step` | `physics::forward` | G16～G19 | P6 |
| U002 | `types` | `Model` | `model` | M01、M02、M05 | P2/P3 |
| U003 | `types` | `Data` | `model::data` | M06、M07、M08 | P3 |
| U004 | `bvh` | `refit_bvh` | `render::bvh` | R01、R05、R06 | P8 |
| U005 | `bvh` | `refit_splat_bvh` | `render::bvh` | R01、R07 | P8 |
| U006 | `collision_driver` | `collision` | `physics::collision` | G07～G11 | P5/P7 |
| U007 | `collision_driver` | `nxn_broadphase` | `physics::collision` | G07～G11 | P5/P7 |
| U008 | `collision_driver` | `sap_broadphase` | `physics::collision` | G07～G11 | P5/P7 |
| U009 | `collision_primitive` | `primitive_narrowphase` | `physics::collision` | G08 | P5 |
| U010 | `collision_sdf` | `sdf_narrowphase` | `physics::collision` | G10 | P5 |
| U011 | `constraint` | `make_constraint` | `physics::constraint` | G12、G13 | P6 |
| U012 | `derivative` | `deriv_smooth_vel` | `physics::derivative` | G21 | P7 |
| U013 | `forward` | `discrete` | `physics::integrator` | G18 | P6 |
| U014 | `forward` | `euler` | `physics::integrator` | G16 | P6 |
| U015 | `forward` | `forward` | `physics::forward` | G19 | P4/P6 |
| U016 | `forward` | `fwd_acceleration` | `physics::forward` | G03、G26 | P4/P6 |
| U017 | `forward` | `fwd_actuation` | `physics::forward` | G05 | P4/P7 |
| U018 | `forward` | `fwd_kinematics` | `physics::forward` | G01、G06 | P4/P7 |
| U019 | `forward` | `fwd_position` | `physics::forward` | G01～G03、G07～G13 | P4/P5/P6 |
| U020 | `forward` | `fwd_velocity` | `physics::forward` | G02、G04 | P4 |
| U021 | `forward` | `implicit` | `physics::integrator` | G17 | P6 |
| U022 | `forward` | `rungekutta4` | `physics::integrator` | G16 | P6 |
| U023 | `forward` | `step1` | `physics::forward` | G19 | P4/P6 |
| U024 | `forward` | `step2` | `physics::forward` | G19 | P6 |
| U025 | `history` | `init_ctrl_history` | `physics::history` | G24、M08 | P7 |
| U026 | `history` | `init_sensor_history` | `physics::history` | G24、M08 | P7 |
| U027 | `history` | `read_ctrl` | `physics::history` | G24、M08 | P7 |
| U028 | `history` | `read_sensor` | `physics::history` | G24、M08 | P7 |
| U029 | `history` | `reset_history` | `physics::history` | G24、M08 | P7 |
| U030 | `inverse` | `inverse` | `physics::inverse` | G20 | P7 |
| U031 | `io` | `get_data_into` | `io::state` | M07 | P3/P9 |
| U032 | `io` | `make_data` | `model::data` | M06 | P3 |
| U033 | `io` | `put_data` | `io::state` | M07 | P3 |
| U034 | `io` | `put_model` | `io::model` | M01～M05 | P2/P3 |
| U035 | `io` | `reset_data` | `io::state` | M08 | P3/P7 |
| U036 | `io` | `reset_data_keyframe` | `io::state` | M08 | P3/P7 |
| U037 | `island` | `island` | `physics::island` | G25 | P7/P10 |
| U038 | `passive` | `passive` | `physics::passive` | G04 | P4 |
| U039 | `ray` | `ray` | `physics::query` | G28 | P7/P8 |
| U040 | `ray` | `rays` | `physics::query` | G28 | P7/P8 |
| U041 | `render` | `render` | `render` | R01～R10 | P8 |
| U042 | `render_util` | `create_render_context` | `render` | R01～R04 | P8 |
| U043 | `render_util` | `get_depth` | `render` | R01～R04 | P8 |
| U044 | `render_util` | `get_rgb` | `render` | R01～R04 | P8 |
| U045 | `render_util` | `get_segmentation` | `render` | R01～R04 | P8 |
| U046 | `sensor` | `energy_pos` | `physics::sensor` | G23、G26 | P7 |
| U047 | `sensor` | `energy_vel` | `physics::sensor` | G23、G26 | P7 |
| U048 | `sensor` | `sensor_acc` | `physics::sensor` | G23、G26 | P7 |
| U049 | `sensor` | `sensor_pos` | `physics::sensor` | G23、G26 | P7 |
| U050 | `sensor` | `sensor_vel` | `physics::sensor` | G23、G26 | P7 |
| U051 | `set_const` | `set_const` | `physics::constants` | G27 | P2/P7 |
| U052 | `set_const` | `set_const_0` | `physics::constants` | G27 | P2/P7 |
| U053 | `set_const` | `set_const_fixed` | `physics::constants` | G27 | P2/P7 |
| U054 | `set_const` | `set_const_spring` | `physics::constants` | G27 | P2/P7 |
| U055 | `set_const` | `set_length_range` | `physics::constants` | G27 | P2/P7 |
| U056 | `smooth` | `camlight` | `physics::smooth` | G01、R08 | P4/P8 |
| U057 | `smooth` | `com_pos` | `physics::smooth` | G01 | P4 |
| U058 | `smooth` | `com_vel` | `physics::smooth` | G02 | P4 |
| U059 | `smooth` | `crb` | `physics::smooth` | G02 | P4 |
| U060 | `smooth` | `factor_m` | `physics::smooth` | G03 | P4/P6 |
| U061 | `smooth` | `flex` | `physics::flex` | G22 | P7 |
| U062 | `smooth` | `kinematics` | `physics::smooth` | G01 | P4 |
| U063 | `smooth` | `rne` | `physics::smooth` | G02 | P4 |
| U064 | `smooth` | `rne_postconstraint` | `physics::smooth` | G02、G23 | P4/P7 |
| U065 | `smooth` | `solve_m` | `physics::smooth` | G03 | P4/P6 |
| U066 | `smooth` | `subtree_vel` | `physics::smooth` | G02 | P4 |
| U067 | `smooth` | `tendon` | `physics::tendon` | G06 | P4/P7 |
| U068 | `smooth` | `transmission` | `physics::actuator` | G05、G06 | P4/P7 |
| U069 | `solver` | `solve` | `physics::solver` | G14、G15 | P6 |
| U070 | `support` | `contact_force` | `physics::query` | G26、G28 | P7 |
| U071 | `support` | `get_state` | `io::state` | M07 | P3/P9 |
| U072 | `support` | `jac` | `physics::query` | G28 | P4/P7 |
| U073 | `support` | `mul_m` | `physics::smooth` | G03 | P4 |
| U074 | `support` | `set_state` | `io::state` | M07 | P3/P9 |
| U075 | `support` | `xfrc_accumulate` | `physics::support` | G26 | P4 |
| U076 | `types` | `BiasType` | `model::actuator` | G05 | P2/P4/P7 |
| U077 | `types` | `BroadphaseFilter` | `model::collision` | G07 | P2/P5 |
| U078 | `types` | `BroadphaseType` | `model::collision` | G07 | P2/P5 |
| U079 | `types` | `Callback` | `physics::callback` | M10 | P7 |
| U080 | `types` | `ConeType` | `model::constraint` | G13 | P2/P6 |
| U081 | `types` | `Constraint` | `model::data` | G12 | P3/P6 |
| U082 | `types` | `Contact` | `model::data` | G08～G11 | P3/P5/P7 |
| U083 | `types` | `CtrlChart` | `model::actuator` | G05、G24 | P2/P7 |
| U084 | `types` | `CtrlInput` | `model::actuator` | G05、G24 | P2/P7 |
| U085 | `types` | `DisableBit` | `model::options` | G01～G28 | P2/P7 |
| U086 | `types` | `DynType` | `model::actuator` | G05 | P2/P4/P7 |
| U087 | `types` | `EnableBit` | `model::options` | G20、G25、G26 | P2/P7 |
| U088 | `types` | `GainType` | `model::actuator` | G05 | P2/P4/P7 |
| U089 | `types` | `GeomType` | `model::collision` | G08～G11、R05 | P2/P5/P8 |
| U090 | `types` | `IntegratorType` | `model::options` | G16～G18 | P2/P6 |
| U091 | `types` | `JointType` | `model::topology` | G01 | P2/P4 |
| U092 | `types` | `ObjType` | `model::index` | M01、M02、G23 | P2/P7 |
| U093 | `types` | `Option` | `model::options` | G07、G14～G18、M11 | P2/P6 |
| U094 | `types` | `OverflowType` | `diagnostics` | M11 | P3/P7 |
| U095 | `types` | `RenderContext` | `render` | R01 | P8 |
| U096 | `types` | `SolverType` | `model::options` | G14、G15 | P2/P6 |
| U097 | `types` | `State` | `io::state` | M07 | P3/P9 |
| U098 | `types` | `Statistic` | `model` | M02 | P2 |
| U099 | `types` | `TrnType` | `model::actuator` | G05、G06 | P2/P4/P7 |

## 30个测试模块

下表只登记迁移入口。
文件名均属于上游。
本仓库没有这些Python测试。
逐条核对输入、断言与容限。
Rust测试承接有效测试意图。
不直接搬运上游执行框架。

| 编号 | 上游测试文件 | 矩阵ID | 实施阶段 | 首轮审查任务 |
| --- | --- | --- | --- | --- |
| T01 | `broadphase_test.py` | G07 | P5 | 核对宽相与过滤 |
| T02 | `bvh_test.py` | R01、R05、R06、R07 | P8 | 核对渲染加速结构 |
| T03 | `collision_driver_test.py` | G07～G11 | P5/P7 | 核对候选与接触 |
| T04 | `collision_gjk_test.py` | G09 | P5 | 核对凸体与退化 |
| T05 | `collision_primitive_core_test.py` | G08 | P5 | 核对基本几何对 |
| T06 | `constraint_test.py` | G12、G13 | P6 | 核对约束与摩擦 |
| T07 | `derivative_test.py` | G17、G21 | P6/P7 | 核对导数与隐式项 |
| T08 | `flex_test.py` | G11、G22 | P7 | 核对柔性体组合 |
| T09 | `forward_test.py` | G05、G16～G19 | P4/P6/P7 | 核对调用阶段 |
| T10 | `history_test.py` | G24、M08 | P7 | 核对延迟与重置 |
| T11 | `inverse_test.py` | G20 | P7 | 核对正逆向回代 |
| T12 | `io_jax_test.py` | M07、A07 | P3/P8 | 逐项分离桥接语义 |
| T13 | `io_test.py` | M01～M09 | P2/P3/P7 | 核对转换与状态 |
| T14 | `island_test.py` | G25 | P7 | 核对岛构建 |
| T15 | `jax_test.py` | A07、A08 | P8/P10 | 逐项分离桥接语义 |
| T16 | `math_test.py` | G01～G03 | P4 | 核对基础不变量 |
| T17 | `passive_test.py` | G04 | P4 | 核对被动力 |
| T18 | `ray_test.py` | G28 | P7/P8 | 核对命中与边界 |
| T19 | `render_test.py` | R01～R10 | P8 | 核对批量图像 |
| T20 | `render_util_test.py` | R01～R04 | P8 | 核对布局与读回 |
| T21 | `sensor_test.py` | G23、G26 | P7 | 核对阶段与单位 |
| T22 | `set_const_test.py` | G27 | P2/P7 | 核对常量刷新 |
| T23 | `sleep_test.py` | G25 | P7/P10 | 核对休眠与唤醒 |
| T24 | `smooth_test.py` | G01～G06 | P4/P7 | 核对动力学中间量 |
| T25 | `solver_test.py` | G14、G15 | P6 | 核对收敛与失败 |
| T26 | `support_test.py` | G03、G26、G28、M07 | P4/P7/P9 | 核对查询与状态 |
| T27 | `types_test.py` | M01、M02、M06 | P2/P3 | 核对类型与字段 |
| T28 | `unroll_test.py` | A01 | P1 | 保留算法测试意图 |
| T29 | `util_misc_test.py` | G05、M02 | P2/P4/P7 | 逐项追踪调用方 |
| T30 | `util_pkg_test.py` | A01、A11 | P0/P1/P11 | 替换版本与部署检查 |

JAX桥接不进入本产品。
混合测试仍需逐条审查。
保留通用状态与同步意图。
不要机械丢弃整个测试文件。
内部工具测试追踪实际调用方。
替换Python专用版本机制。

## 公开入口之外的范围

99个符号不是完整功能分母。
内部类型与内核同样影响语义。
P0还需盘点以下内部类型：

| 上游类型 | 相关矩阵 | 待展开内容 |
| --- | --- | --- |
| `SensorType` | G23 | 传感类型与阶段 |
| `EqType` | G12 | 等式类型与参数 |
| `ConstraintType` | G12、G13 | 约束分类与布局 |
| `ContactType` | G08～G11 | 接触标志与字段 |
| `ProjectionType` | R08 | 投影类型与相机参数 |
| `CamLightType` | G01、R08、R09 | 相机与光源跟随规则 |
| `SleepPolicy`、`SleepState` | G25 | 休眠策略与状态 |
| geom插件及SDF用户钩子 | G10、M03、M10 | 属性、槽位、距离与梯度 |

依据：[类型定义](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py)。

还需展开Model与Data字段。
按输入、输出与工作区分类。
区分共享参数与世界参数。
登记字段尺寸、精度与所有权。
登记原生模型版本条件。
不要只迁移默认选项。

## 调用时序与负向范围

完整step先执行forward。
然后按选项执行积分。
forward穿插三个传感阶段。
位置阶段还包含碰撞与约束。
休眠会改变唤醒与碰撞时序。
分步接口单独验收。
不要假定分步等同完整step。
RK4模式下step2采用Euler。
等价step2不要求step1票据。
调用方可以导入已准备中间量。

以下示意省略条件分支：

```text
step
  forward
    位置计算 / 碰撞 / 约束 / 位置传感
    速度计算 / 被动力 / 速度传感
    控制回调 / 执行器 / 加速度
    约束求解 / 后约束量 / 加速度传感
  指定积分器
```

依据：[调度源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py)。

负向测试也属于复刻范围。
提取模型上传的拒绝分支。
提取积分器的组合限制。
提取休眠与柔性体组合限制。
同时登记警告与降级语义。
不要把上游限制误当缺陷。
不要将警告路径写成全量支持。

依据：[模型上传](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py)、[积分配置检查](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py)。

## 本轮完成与剩余任务

本轮核对了入口与文件数量。
本轮完成实现归属草案。
首版契约调整17项归属记录。
首版契约登记24项测试任务。
本轮复审补充6项等价测试任务。
本轮细化补充12项测试任务。
声明清单覆盖895项直接注解。
它同时保留31类枚举表达式。
它保留73项功能导出的签名。
阶段与组合另见细化清单。
本轮没有实现任何引擎接口。
本轮没有完成数值或GPU验收。

P0剩余工作：

- [ ] 逐条读取30个测试模块。
- [ ] 核对枚举原生值与能力开关。
- [ ] 冻结默认值与阶段副作用。
- [ ] 核对用户SDF与回调扩展。
- [ ] 补齐字段单位、布局与副作用。
- [ ] 固定碰撞几何对与组合。
- [ ] 固定原生模型语义版本。
- [ ] 登记参考样本与资产许可。
- [ ] 冻结数值与性能门槛。

任务依赖见[复刻执行清单](replication-plan.md)。
