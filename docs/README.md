# mjwarp-rs 项目文档

创建日期：2026-10-06。
状态：基础层与P1至P4局部开发。

本项目对标MJWarp。
它提供Rust GPU物理接口。
它主要服务LRsLab。
它不代替上层训练框架。
项目只交付引擎层能力。
LRsLab负责应用调用与play。
LRsLab负责窗口与交互前端。
Rust负责主API与宿主逻辑。
C/C++组件与内核可以复用。
项目不要求全部源码使用Rust。

## 已确认的要求

| 项目 | 要求 |
| --- | --- |
| 主API与宿主逻辑 | Rust |
| 物理内核 | 允许Rust与C/C++，GPU执行 |
| Python | 构建、测试、运行不依赖 |
| 原生组件 | 允许接入C/C++物理与渲染 |
| 模型输入 | 接收已编译模型并转换上传 |
| 首版GPU | NVIDIA |
| 首版系统 | Windows与Linux |
| GPU正式范围 | 冻结版MJWarp完整支持集 |
| 上层服务对象 | LRsLab，定位类似mjlab |
| GPU批量渲染 | 属于引擎对标范围，不含窗口 |
| CPU play与交互前端 | 全部归LRsLab，不属本crate |
| 后续GPU | 预留跨厂商后端 |

内部原型可以分阶段。
首个正式版不能缩减范围。
项目不承诺上游未支持项。
等价范围也覆盖默认与副作用。
严格门面不能代替等价入口。
GPU对标不表示发行包相同。
CPU调试与工具边界仍待确认。

## 阅读顺序

1. [项目定位](project-positioning.md)
2. [架构与接口](architecture.md)
3. [首版模块契约](module-contracts.md)
4. [字段与接口声明](upstream-contract-inventory.md)
5. [物理阶段契约](physics-stage-contracts.md)
6. [组合与边界清单](supported-combinations.md)
7. [完整对标复审](initial-release-review.md)
8. [兼容与验收矩阵](compatibility-matrix.md)
9. [详细实施计划](implementation-plan.md)
10. [复刻执行清单](replication-plan.md)
11. [上游映射草案](upstream-api-map.md)
12. [GPU探针与路线评估](gpu-probe.md)
13. [CubeCL双路探针](cubecl-probe.md)
14. [Windows内核能力探针](windows-kernel-probe.md)
15. [Windows资源租约探针](windows-resource-probe.md)
16. [Windows产物缓存探针](windows-artifact-probe.md)
17. [Windows外部资源ABI探针](windows-external-probe.md)
18. [基础源码与架构收口](foundation-architecture.md)
19. [Windows字段交换探针](windows-transfer-probe.md)
20. [Windows原生模型输入探针](windows-native-model-probe.md)
21. [Windows运动学字段增量](windows-kinematic-fields.md)
22. [Windows惯性字段增量](windows-inertial-fields.md)
23. [Windows刚体运动学探针](windows-kinematics-probe.md)
24. [Windows质心GPU探针](windows-com-position-probe.md)
25. [Windows质量矩阵探针](windows-mass-matrix-probe.md)
26. [Windows质量求解探针](windows-mass-solve-probe.md)
27. [Windows附着运动学探针](windows-attached-kinematics-probe.md)
28. [Windows常驻运动学底座](windows-resident-kinematics.md)
29. [Windows运动学字段批量](windows-resident-parameters.md)
30. [Windows运动学mocap子集](windows-resident-mocap.md)
31. [Windows相机光源子集](windows-resident-camlight.md)
32. [Windows固定肌腱子集](windows-resident-fixed-tendon.md)
33. [Windows空间肌腱子集](windows-resident-spatial-tendon.md)
34. [Windows球柱绕行子集](windows-resident-geom-tendon.md)
35. [运动视频逐帧原生复核](windows-geom-tendon-frame-review.md)
36. [Windows混合肌腱编号](windows-resident-tendon.md)
37. [Windows柔体位置子集](windows-resident-flex-position.md)
38. [Windows肌腱唤醒子集](windows-resident-tendon-wake.md)
39. [Windows柔体边子集](windows-resident-flex-edge.md)

定位文档明确产品边界。
架构文档定义模块契约。
首版契约细化数据与调用边界。
兼容矩阵管理功能完成度。
实施计划定义阶段交付。
执行清单拆分任务与依赖。
上游映射登记接口与测试入口。

## 当前复刻准备

本轮开始整理P0任务。
本轮核对99个公开符号。
本轮核对30个测试模块。
映射表仍提供实现归属草案。
首版契约修正部分模块归属。
首版契约核对关键字段与限制。
复审修正7项文档语义偏差。
功能分母与发行边界仍待关闭。
复审补充6项等价测试任务。
细化清单盘点895项字段声明。
清单覆盖31类枚举及73项签名。
组合清单核对43组入口分支。
模块契约累计42项测试任务。
MC物理契约任务仍待实现。
字段单位、布局与证据仍待补齐。
全量内部组合仍待展开。
产品契约仍待完整实现。
五模块已有基础辅助接口。
八项数学公式已有宿主测试。
运行时架构拆成九个叶子。
刚体子集已有GPU数值证据。
质心与自由度映射也有证据。
质量矩阵已有GPU子集证据。
正定分解与求解也有GPU子集。
几何与site位姿也有GPU子集。
常驻计划复用六个设备子集。
局部入口使用七个结果缓冲。
混合入口另加GPU合并。
混合入口使用九个结果缓冲。
柔体入口另加位置子集。
纯位置入口使用十个结果缓冲。
GPU计算节点与顶点位置。
它支持直接与线性插值。
柔体位置字段仍共享。
可选边入口另加两个浮点缓冲。
GPU计算边长、雅可比与速度。
qvel支持独立世界写入。
边拓扑与稀疏行仍共享。
柔体面运动学仍待实现。
可选入口另加肌腱唤醒。
GPU刷新树标记与树计数。
范围与边距各用独立周期。
完整休眠仍待实现。
既有二十三项参数保留独立批量。
GPU支持相机与光源位姿。
GPU支持固定肌腱长度与力臂。
肌腱拓扑与系数仍共享。
空间肌腱支持site与pulley。
GPU更新长度、力臂与包裹字段。
GPU支持球柱与内侧绕行。
T4内侧精度修复已通过复验。
480源帧覆盖1920组原生状态。
复验保留原有容限。
几何尺寸支持独立周期。
两类肌腱各用局部编号。
混合入口保留原生全局编号。
GPU合并长度、力臂与包裹。
常驻状态支持mocap写入。
GPU初始化静态geom缓存。
更新仍重算全部site。
状态更新不再重复上传模型。
完整G01阶段仍待实现。
连续字段已有真实GPU交换。
同步接口覆盖上传与世界复制。
Windows驱动探针已经执行。
CubeCL C++已执行Windows探针。
CubeCL LLVM也通过局部探针。
本机MSVC与SDK复验通过。
Windows补齐八项能力探针。
双路通过分层全局扫描。
双路通过f32原子累加。
图节点更新切换真实输出。
三路通过内部资源租约探针。
队列保留真实缓冲所有者。
提交失败保留错误状态。
三路支持可信PTX缓存复用。
仅驱动子进程通过68组配置。
清洁部署仍须独立机器证据。
三路通过外部资源ABI原型。
驱动核对传统分配实际范围。
队列保留外部所有者与事件。
原型不冻结生产张量接口。
本轮不开展Linux工作。
这些记录不代表引擎完成率。

应用调用方案不放入本仓库。
本工作区另存[应用方案](../../LRsLab/docs/engine-integration.md)。

## 当前代码状态

以下事实来自当前代码。
其余能力均属于实施目标。

| 项目 | 当前状态 |
| --- | --- |
| Cargo包名 | `mjwarp-rs` |
| Cargo版本 | `0.1.0` |
| Rust edition | `2024` |
| 第三方依赖 | 可选cudarc、冻结CubeCL、serde、JSON、sha2与原生构建cc；锁定版本 |
| 程序入口 | `src/main.rs` |
| 程序行为 | 提供`probe`、`resources`、`external-resources`、`cache-build`与`cache-run`；默认不加载GPU依赖 |
| 库入口 | `src/lib.rs`；导出七模块基础接口与探针，不提供完整引擎 |
| 模型基础 | 检查布局与原生DTO；组合十二运动学与九惯性字段；完整模型ABI仍待实现 |
| 输入转换 | 先校验再转f32；支持连续字段GPU交换；不替代等价上传入口 |
| GPU字段交换 | f32、f64、i32与u32上传、回读与世界复制；独占缓冲、会话隔离与同步等待 |
| 原生模型输入探针 | Windows MSVC可选读取可信MJB；四字段独占快照与GPU回读；不提供完整put_model |
| 运动学字段增量 | 四类关节与地址校验；十二字段独占快照及只读GPU字段组；参数批量长度固定为一 |
| 惯性字段增量 | 九字段及最近祖先校验；组合二十一字段上传与回读；不计算动力学 |
| 刚体运动学探针 | GPU计算七项字段；四类关节、偏心旋转与批量世界；不替代完整G01 |
| 质心GPU探针 | GPU计算子树质量、质心、cinert与cdof；零质量遵循冻结Warp；不替代U057 |
| 质量矩阵GPU探针 | GPU计算CRB与对称稠密矩阵；支持自由度armature；不提供分解或求解 |
| 质量求解GPU探针 | 内部GPU f64计算反向LDL与多个右端项；公开结果为f32；不替代完整G03 |
| 附着运动学GPU探针 | 六项几何与site字段检查；复用刚体设备结果计算四项位姿；不替代完整G01 |
| 常驻运动学底座 | 一次上传与编译；独占世界状态；显式回读；二十三项参数独立取模；mocap世界写入；静态geom缓存；仍限严格子集 |
| 固定肌腱GPU子集 | 关节项加权qpos长度与稀疏力臂；共享系数与原生CSR；不提供空间绕行、速度或限位 |
| 空间肌腱GPU子集 | site、pulley与球柱路径；内侧绕行；独立尺寸周期；长度、稀疏力臂及动态包裹字段；仍用局部编号 |
| 混合肌腱GPU子集 | 保留原生全局编号与CSR；GPU合并长度、力臂及动态包裹；旧局部接口保持兼容 |
| 纯数学 | 八项f32四元数及空间代数公式；刚体探针覆盖部分GPU公式；完整数值验收仍待补齐 |
| 历史基础 | 检查三维布局与显式时间阈值；GPU历史仍待实现 |
| 图像基础 | 检查单相机批量像素、容量与输出开关；不生成像素 |
| 驱动级GPU探针 | Windows通过；缓冲、事件、边界与图重放 |
| CubeCL内核路线 | Windows双路各通过74组配置；含全局扫描、f32原子及图更新 |
| 内部资源租约 | 三路各通过5种规模；支持偏移视图、完成队列、句柄提前释放与宿主失败隔离 |
| PTX缓存与部署烟测 | 17份产物；仅驱动受限进程通过68组配置；清洁机器验收仍待执行 |
| 外部资源ABI原型 | Rust/C++核对96字节布局；三路核对实际分配与所有者移交；生产接口仍待决议 |
| GPU物理 | 刚体、附着、质心、矩阵与正定求解子集已有探针；完整物理阶段仍待实现 |
| GPU批量渲染 | 尚未实现 |
| CPU play与查看器 | 不属于本仓库职责 |

依据：[包配置](../Cargo.toml)、[程序入口](../src/main.rs)、[驱动基线](gpu-probe.md)、[双路报告](cubecl-probe.md)、[能力报告](windows-kernel-probe.md)、[租约报告](windows-resource-probe.md)、[缓存报告](windows-artifact-probe.md)、[外部ABI报告](windows-external-probe.md)、[基础报告](foundation-architecture.md)、[字段报告](windows-transfer-probe.md)、[原生输入报告](windows-native-model-probe.md)、[运动学字段报告](windows-kinematic-fields.md)。

骨架版本不代表正式验收。
惯性子集详见[增量报告](windows-inertial-fields.md)。
刚体子集详见[数值报告](windows-kinematics-probe.md)。
质心子集详见[质心报告](windows-com-position-probe.md)。
质量矩阵详见[矩阵报告](windows-mass-matrix-probe.md)。
正定求解详见[求解报告](windows-mass-solve-probe.md)。
附着位姿详见[附着报告](windows-attached-kinematics-probe.md)。
常驻设备链详见[底座报告](windows-resident-kinematics.md)。
字段批量详见[批量报告](windows-resident-parameters.md)。
mocap与缓存详见[增量报告](windows-resident-mocap.md)。
文档不宣称已有完整引擎。
GPU编译路线仍需原型验证。
原生依赖版本仍需冻结。
许可与部署清单仍需核对。

## 上游基线

项目冻结以下MJWarp快照。

```text
repository: google-deepmind/mujoco_warp
revision: 71da24d956378a87a703b6e1442b13aec0c4ac29
checked_on: 2026-10-06
```

依据：[冻结版源码](https://github.com/google-deepmind/mujoco_warp/tree/71da24d956378a87a703b6e1442b13aec0c4ac29)。

后续升级需单独评审。
不能用追踪main替代验收基线。

## 文档维护规则

代码实现后更新状态。
每项完成声明附测试证据。
设计接口不得冒充已实现接口。
正式发布需满足全部准入条件。

历史训练文档不约束本引擎。
项目允许复用原生C物理组件。
复用组件不替代GPU完整验收。
引擎发布不等待play或前端交付。
