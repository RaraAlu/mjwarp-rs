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
| 计算精度 | 全部物理路径跟随冻结上游；内部精度也须一致 |
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

精度硬指标见[精度契约](numerical-precision-contract.md)。
第一数值参考为原生MuJoCo。
原生通过即通过该项数值门禁。
仅原生超差后对照MJWarp。
两路都计算相对原生的误差。
不劣于MJWarp时对照放行。
略高但在5%红线内也放行。
越线则严格禁行。
两级机制已接入G01与G03辅助。
本轮所需第二参考已经补齐。
最新规则见[红线验收](native-error-redline-acceptance.md)。
此前完整复测各474过3败。
上述失败属于历史记录。
10月12日已补齐分块双参考。
随后定位分解首差并修复。
新求解专项双模式各8过0败。
每模式仍有20块对照放行。
专项解除G03辅助数值阻断。
记录见[G03首差修复](windows-g03-first-difference.md)。
随后追加完整产品验证。
双模式各483过、0败。
每模式65733块原生放行。
另29块采用上游对照放行。
数值失败与待验收均为零。
两类满额独立诊断没有重跑。
NP01与NP02仍缺全路径证据。
NP04仍缺公平稳态性能对照。
完整T4仍不放行。
最新记录见[T4追加验证](windows-t4-g03-retest.md)。
后续新增独立宿主证据门禁。
24个入口及双模式产物已核对。
性能条件检查拒绝不公平记录。
真实稳态性能仍未测。
详见[证据门禁](t4-evidence-gates.md)。
随后核对78项原生入口。
32项浮点转换保持f32。
三项布尔适配单独记录。
局部量与设备打包仍待审。
详见[入口精度映射](t4-input-precision.md)。
随后追踪15项状态声明。
同时核对十个打包器存储。
派生缓存与布尔编码单列。
详见[状态精度追踪](t4-state-packing.md)。

后续已展开23项批量参数。
质心内部计算另映射15组。
新增五项宿主测试通过。
旧满额验证没有重跑。
详见[逐字段与内部量](t4-field-internal.md)。

随后展开其余八个打包器。
本轮登记82条字段与派生路由。
固定肌腱内部量另映射四组。
新增五项测试通过，旧项不重跑。
详见[其余打包路由](t4-packing-routes.md)。

随后补齐23条整数拓扑路由。
附着相机内部量另映射16组。
八项输出追踪到宿主视图。
新增五项用例通过，旧项未重跑。
详见[拓扑与位姿](t4-topology-frames.md)。

随后映射17组刚体内部量。
七项输出已追踪到宿主视图。
新增六项用例通过，旧项未重跑。
当时发现零四元数源码分歧。
详见[刚体内部量](t4-rigid-internal.md)。

现已核实上游实际产物。
完整入口零分支已修复。
定向608项比较全部一致。
旧完整产物已触发源码漂移。
完整T4仍未验收。
详见[零四元数核验](t4-zero-quaternion.md)。

随后补齐三种刚体编译产物。
另补七份附着相机复用产物。
当前集合保留24入口、30变体。
三十份产物均过原精度门禁。
本轮未重跑物理或满额验证。
详见[刚体增量编译](t4-rigid-recapture.md)。
随后映射16组柔体内部量。
六项输出追踪到宿主视图。
未用稀疏尾槽存在清零差异。
完整精度验收仍未完成。
详见[柔体后续切片](t4-packing-routes.md#柔体后续切片)。
随后补齐21组面与矩阵内部量。
四项输出与缓存标志均已追踪。
面生成缓存已完成静态核验。
矩阵生成及求值差异仍待审。
详见[面与矩阵切片](t4-field-internal.md#柔体面与矩阵切片)。
随后直接运行冻结面产物。
1033664项比较均在原容差内。
四元数仍有98625项位差。
本轮仅修正诊断启动配置。
完整精度及性能门禁仍开放。
详见[面产物实测](t4-field-internal.md#面产物定向实测)。
随后隔离算子与编译器因素。
本组编译器与目标均无位差。
修复已证实的表达与融合差异。
当前仍有2078项位差。
严格逐位门禁仍失败。
满额诊断不再重跑。
详见[算子隔离](t4-field-internal.md#极分解算子与编译器隔离)。
随后核对矩阵生成前置。
三项独立宿主检查通过。
实际矩阵生成产物仍缺失。
原子次序与零符号仍待实测。
详见[生成前置](t4-field-internal.md#矩阵生成前置与累加风险)。
随后静态比较既有面中间码。
十九项名义运算图一致。
三处舍入限定仍有差异。
实际机器融合与剩余首差待证。
详见[舍入限定](t4-field-internal.md#剩余位差的静态舍入核查)。
随后捕获本轮实际加载机器码。
上游三处末项确实执行融合。
产品仍保留分离乘减。
本轮没有执行物理内核。
2078项逐样本归因仍待验证。
详见[机器码捕获](t4-field-internal.md#实际加载机器码与末项融合)。
记录见[红线T4复测](windows-t4-redline-retest.md)。
记录见[两级补测](windows-two-reference-acceptance.md)。
第393帧绕行首差已有定位。
证据见[首差追踪](frame393-first-divergence.md)。
两处初始长度已固定乘加顺序。
第393帧接触点对齐MJWarp。
但原生力臂误差反而增大。
下述报告保留旧门禁结果。
证据见[局部修复](inside-norm-order-fix.md)。
内部升精度不能替代上游等价。
四类已知路径已恢复f32。
原生对照仍有超差。
类型修复见[f32修复](linux-f32-precision-restoration.md)。
历史原生参考验收不关闭此缺口。

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
40. [Windows柔体面子集](windows-resident-flex-face.md)
41. [Windows柔体缓存失效](windows-resident-flex-hessian.md)
42. [Windows柔体拉伸矩阵](windows-resident-flex-stretch.md)
43. [Windows原生柔体位置转换](windows-native-flex-position.md)
44. [Windows G01全阶段验收](windows-g01-acceptance.md)
45. [Linux G01全阶段验收](linux-g01-acceptance.md)
46. [Linux常驻空间速度子集](linux-resident-com-velocity.md)
47. [Linux常驻RNE偏置力子集](linux-resident-rne-bias.md)
48. [Linux单精度修复进展](linux-f32-precision-restoration.md)
49. [Windows肌腱上游并列诊断](windows-mjwarp-geom-tendon-comparison.md)
50. [Windows完整T4补测](windows-t4-recheck.md)
51. [Windows两级判定补测](windows-two-reference-acceptance.md)
52. [第393帧绕行首差](frame393-first-divergence.md)
53. [内侧绕行乘加顺序修复](inside-norm-order-fix.md)
54. [原生基准与5%误差红线](native-error-redline-acceptance.md)
55. [Windows红线T4复测](windows-t4-redline-retest.md)
56. [Windows完整T4追加验证](windows-t4-g03-retest.md)
57. [T4精度与性能证据门禁](t4-evidence-gates.md)

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
纯位置入口使用十一个结果缓冲。
GPU计算节点与顶点位置。
它支持直接与线性插值。
柔体位置字段仍共享。
可选边入口另加两个浮点缓冲。
GPU计算边长、雅可比与速度。
qvel支持独立世界写入。
边拓扑与稀疏行仍共享。
GPU已计算线性壳体面位姿。
原生柔体位置转换已落地。
十二项字段可接入常驻辅助。
转换仍拒绝二次插值。
证据见[转换报告](windows-native-flex-position.md)。
可选面入口另加一个结果缓冲。
面四元数采用xyzw顺序。
柔体Hessian失效已落地。
GPU已缓存拉伸投影矩阵。
三角形与四面体采用21系数。
矩阵乘法仍待实现。
证据见[拉伸报告](windows-resident-flex-stretch.md)。
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
此前T4复验采用双精度绕行。
当前f32绕行复验仍超原生容差。
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
完整G01入口已实现。
Windows双模式验收通过。
证据见[G01报告](windows-g01-acceptance.md)。
Linux双模式完整门禁通过。
上述记录属于历史验收。
当前完整产品数值复验通过。
双模式各483过、0败。
独立精度与性能门禁仍待收口。
证据见[T4追加验证](windows-t4-g03-retest.md)。
证据见[Linux报告](linux-g01-acceptance.md)。
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
Linux复验只覆盖既有范围。
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
| 原生模型输入探针 | Windows MSVC与Linux GNU可选读取可信MJB；独占快照与GPU回读；不提供完整put_model |
| 运动学字段增量 | 四类关节与地址校验；十二字段独占快照及只读GPU字段组；参数批量长度固定为一 |
| 惯性字段增量 | 九字段及最近祖先校验；组合二十一字段上传与回读；不计算动力学 |
| 原生柔体位置转换 | 独占十二项位置字段；严格转换后接入常驻GPU辅助；不替代完整put_model |
| 刚体运动学探针 | GPU计算七项字段；四类关节、偏心旋转与批量世界；不替代完整G01 |
| 质心GPU探针 | GPU计算子树质量、质心、cinert与cdof；零质量遵循冻结Warp；不替代U057 |
| 质量矩阵GPU探针 | GPU计算CRB与对称稠密矩阵；支持自由度armature；不提供分解或求解 |
| 质量求解GPU探针 | GPU f32分块分解与多右端项；专项双参考通过；不替代完整G03 |
| 附着运动学GPU探针 | 六项几何与site字段检查；复用刚体设备结果计算四项位姿；不替代完整G01 |
| 常驻运动学底座 | 一次上传与编译；严格计划保留就绪检查；G01计划提供完整链与等价低层调用；支持预检导入与当前字段回读 |
| 完整G01 | 原生78字段转换；完整运动学调度与五个低层入口；本机双模式完整数值复验通过；NP03失败与待验收归零；独立精度和性能门禁仍待验收 |
| G02刚体子集 | 常驻cvel、cdof_dot与qfrc_bias；全链f32；本机原生子集通过；本轮未复测Linux；完整G02仍待实现 |
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
