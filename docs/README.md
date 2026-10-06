# mjwarp-rs 项目文档

创建日期：2026-10-06。
状态：设计与P1探针开发。

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
契约签名尚未进入源码。
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
| 第三方依赖 | 可选cudarc与冻结CubeCL；Cargo.lock固定版本 |
| 程序入口 | `src/main.rs` |
| 程序行为 | 提供`probe`与`resources`命令；默认不加载GPU依赖 |
| 库入口 | `src/lib.rs`；只导出探针，不提供物理引擎 |
| 驱动级GPU探针 | Windows通过；缓冲、事件、边界与图重放 |
| CubeCL内核路线 | Windows双路各通过74组配置；含全局扫描、f32原子及图更新 |
| 内部资源租约 | 三路各通过5种规模；支持偏移视图、完成队列、句柄提前释放与宿主失败隔离 |
| GPU物理 | 尚未实现 |
| GPU批量渲染 | 尚未实现 |
| CPU play与查看器 | 不属于本仓库职责 |

依据：[包配置](../Cargo.toml)、[程序入口](../src/main.rs)、[驱动基线](gpu-probe.md)、[双路报告](cubecl-probe.md)、[能力报告](windows-kernel-probe.md)、[租约报告](windows-resource-probe.md)。

骨架版本不代表正式验收。
文档不宣称已有引擎API。
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
