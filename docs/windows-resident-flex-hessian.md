# Windows柔体缓存失效

日期：2026-10-08。
本报告只验收G01失效子集。
完整G01仍保留计划态。

## 冻结契约

冻结提交保持不变：
`71da24d956378a87a703b6e1442b13aec0c4ac29`。

[smooth.flex](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py#L573-L576)
先清除全部Hessian标志。
它不筛选插值模式或休眠状态。
随后执行节点、顶点、边与面。

[types.Data](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L2465)
定义逐世界、逐柔体的bool字段。
[reset_data](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L2782-L2783)
将标志初始化为false。

[passive.flex_hessian](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/passive.py#L1704-L1767)
另行计算矩阵并标记true。
本轮不实现该生产阶段。
本轮不宣称原生矩阵等价。

## 已落地行为

- `src/physics/flex.rs`定义布尔视图。
- `FlexPositionOutput::nflex`返回数量。
- `FlexPositionWorld::flex_hessian_valid`提供只读切片。
- `src/physics/resident.rs`保留设备标志。
- 创建状态将全部标志初始化为零。
- 每次位置更新先清除全部标志。
- 清除与位置各执行一次同步内核。
- 阶段间不回读或上传标志。
- 标志不选择性保留睡眠世界。
- 模式0、1与-1遵循相同清除逻辑。
- 非位置阶段不改写标志。
- qpos与mocap写入废弃阶段就绪。
- 后续位置阶段执行全量清除。

设备使用独立守卫i32缓冲。
零与一分别编码false与true。
这不是Warp的bool设备ABI。
四项头守卫与四项尾守卫保留。
回读先检查守卫及规范编码。
它拒绝负数及大于一的编码。
失败不发布部分快照。
快照独立拥有宿主布尔数据。
世界视图新增公开字段。
手动构造视图须补充该字段。
这不是完整Data ABI交付。

旧入口仍不分配标志缓冲。
空柔体入口保留八项守卫。
空入口提供空布尔切片。
纯位置入口现有十一个结果缓冲。
可选边仍另占两个f32缓冲。
其中一个缓冲存放qvel输入。
可选面仍另占一个f32结果。
唤醒组合仍另占两个i32缓冲。

## 失败与严格边界

入口先检查计划身份与刚体就绪。
检查失败不清除标志。
这仍是严格辅助入口。
它不替代冻结完整阶段入口。
内核失败会废弃位置阶段就绪。
清除成功后的位置失败保留零值。
清除失败不发布旧快照。
常驻缓冲保留上下文所有权。
数据与快照允许晚于计划释放。

## 测试范围

Rust测试私下播种缓存标志。
测试不伪造Hessian矩阵。
公开API不允许标记缓存有效。

| 检查 | 范围 |
| --- | --- |
| 批量清除 | 1、2、5、513世界，四轮 |
| 精确布尔比较 | 2084组状态，6252个标志 |
| 混合模式 | 直接、线性与线性壳体 |
| 休眠组合 | 全部树休眠，保留闭环状态 |
| 非位置阶段 | 附着、质心、相机、边、面、肌腱、唤醒 |
| 错误编码 | -1、2、i32上下界 |
| 守卫检查 | 首尾四个边界位置 |
| 失败顺序 | 清除与位置的会话不匹配 |
| 输入与身份 | 跨计划、越界、非有限与未就绪 |
| 空输入 | 旧入口、空柔体与静态直接柔体 |
| 所有权 | 计划、会话、数据及快照释放顺序 |

既有三个柔体验收也检查标志。
它们分别比较位置、边与面。
数值容限仍保持原值。
本轮不生成或改写原生参考。
产品测试不启动Python。

## 验证记录

初次边界测试出现一项失败。
测试误将零顶点柔体视为合法。
模型校验正确拒绝该输入。
随后测试改用合法静态柔体。
后续定向测试四项全部通过。

平台采用Windows x86_64。
GPU采用RTX 4070 Ti SUPER。
驱动版本采用596.36。
Rust版本采用1.99.0。
工具链覆盖GNU与MSVC。
Cargo参数放在`--`之前。

| 实际命令 | 最终结果 |
| --- | --- |
| `cargo test --locked` | 184项宿主、12项文档通过 |
| `cargo +stable-x86_64-pc-windows-msvc test --locked --all-features` | 233项宿主、12项文档通过 |
| 上一命令加`--release` | 同样245项通过 |
| `cargo +stable-x86_64-pc-windows-msvc test --locked --all-features -- --ignored --test-threads=1 --nocapture` | 148项通过 |
| 上一命令加`--release` | 同样148项通过 |
| `scripts/test-windows-resident-kinematics.ps1 -AllFeatures` | 16项宿主、72项GPU通过 |
| 上一命令加`-Release` | 同样88项通过 |
| `cargo test --locked --features cuda-probe --lib flex_hessian_ -- --include-ignored --test-threads=1 --nocapture` | GNU调试四项通过 |
| 上一命令加`--release` | GNU发布四项通过 |
| `cargo clippy --locked --all-targets -- -D warnings` | GNU默认零警告 |
| `cargo clippy --locked --all-targets --features cuda-probe -- -D warnings` | GNU CUDA零警告 |
| `cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-targets --all-features -- -D warnings` | MSVC调试零警告 |
| 上一命令加`--release` | MSVC发布零警告 |
| `cargo +stable-x86_64-pc-windows-msvc doc --locked --all-features --no-deps` | Rustdoc通过 |
| `cargo fmt --check` | 格式通过 |
| `git diff --check` | 差异检查通过 |

全特性探针包含22项原生探针。
其余126项执行GPU检查。
两种构建模式均记录相同数量。
定向布尔检查各比较6252个标志。
检查结果全部精确等于false。
本轮没有调整物理误差容限。

本轮记录十七条最终命令。
所有命令均成功退出。
证据目录包含实际日志。

- 汇总：`target/flex-hessian-verification/final-results.json`。
- 调试：`target/resident-kinematics/a695027a5a65443b96a92cf501a73953/report.json`。
- 发布：`target/resident-kinematics/b580f6fa0fbd47d6ac640f059d35278a/report.json`。
- 检查脚本：`scripts/test-windows-resident-kinematics.ps1`。

## 文档与架构核对

本轮同步六处失效状态。
本轮同步四处缓冲数量。
本轮调整下一步实施顺序。
增量检查读取16份源码。
它读取10份文档。
它校验142条本地链接。
它校验14项固定参考哈希。
增量检查报告零错误。

独立文档钩子扫描整个仓库。
它读取170份源码与60份文档。
它仍报告八项路径提示。
人工核对确认全部属于误报。
其中一项引用外部上游路径。
三项引用已有PTX文件。
四项引用脚本配置的原生库。
增量检查确认本地文件存在。
本轮不改写这些历史证据。

现有架构叶节点继续保持活跃。
引擎与完整阶段继续保持计划态。
本轮没有新增依赖或架构叶。

## 未完成项

- Hessian矩阵计算与乘积仍待实现。
- 高阶柔体插值仍待实现。
- 完整Model/Data ABI仍待实现。
- 完整休眠、回调与阶段入口仍待实现。
- Linux本轮尚未执行。
- G01、G22与G25继续保留计划态。

参见[位置报告](windows-resident-flex-position.md)。
参见[边报告](windows-resident-flex-edge.md)。
参见[面报告](windows-resident-flex-face.md)。
