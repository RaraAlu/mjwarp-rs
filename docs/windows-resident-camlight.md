# Windows相机光源子集

日期：2026-10-07。
范围：G01常驻辅助增量。
本批不关闭完整G01。
本文保留相机光源批次证据。
后续加入[固定肌腱](windows-resident-fixed-tendon.md)。
当前常驻链已有五组输出。
本批不新增第三方依赖。
产品链不引入Python。

## 实际接口

模型新增共享拓扑检查。
浮点参数新增十项独立周期。
GPU复用已有刚体与质心结果。
快照新增四项只读世界视图。

```text
model::CamLightFields
model::CamLightModelInput
model::CamLightParameter
model::CamLightParameters
physics::KinematicsPlan::with_camlight
physics::KinematicsPlan::update_camlight
physics::KinematicsPlan::ncam
physics::KinematicsPlan::nlight
physics::KinematicsPlan::camlight_parameters
physics::KinematicsSnapshot::camlight
physics::CamLightOutput
physics::CamLightWorld
```

调用方提供已编译模型常量。
本接口不编译MJCF。
本接口不创建窗口或图像。
参数与拓扑在计划内保持只读。
旧构造器默认空相机光源。
旧手动三阶段回读仍有效。
非空集合要求第四阶段就绪。

## 输入与布局

模式、体编号与目标编号共享。
模式仅接受原生值0至4。
体编号必须落在模型范围。
非负目标必须落在体范围。
所有负目标均采用固定回退。
这些检查仍属严格辅助限制。
它们不等于完整等价入口。

每项浮点字段独立取模。
规则为`world % B_f`。
缺省字段采用原模型共享行。
空字段仍要求正周期。
周期无需整除世界总数。
构造器检查全部参数行。
未选用的行也必须合法。
相机四元数必须近似单位值。
光源方向允许零与非单位值。
初始姿态矩阵只检查有限性。
TRACK分支原样复制此矩阵。
参数覆盖不自动重算跟踪常量。

| 字段 | 每行标量数 |
| --- | --- |
| `cam_pos` | `3*ncam` |
| `cam_quat` | `4*ncam` |
| `cam_poscom0` | `3*ncam` |
| `cam_pos0` | `3*ncam` |
| `cam_mat0` | `9*ncam` |
| `light_pos` | `3*nlight` |
| `light_dir` | `3*nlight` |
| `light_poscom0` | `3*nlight` |
| `light_pos0` | `3*nlight` |
| `light_dir0` | `3*nlight` |

当前常驻链共有23项参数。
原有13项接口保持不变。
相机光源另用独立参数集合。

## 冻结模式语义

| 模式 | 位置 | 相机姿态 | 光源方向 |
| --- | --- | --- | --- |
| FIXED | 体变换局部位置 | 体与局部姿态相乘 | 体旋转局部方向并归一化 |
| TRACK | 体位置加初始世界偏移 | 复制初始世界矩阵 | 归一化初始世界方向 |
| TRACKCOM | 子树质心加初始质心偏移 | 复制初始世界矩阵 | 归一化初始世界方向 |
| TARGETBODY | 体变换局部位置 | 指向目标体位置 | 指向目标体位置并归一化 |
| TARGETBODYCOM | 体变换局部位置 | 指向目标子树质心 | 指向目标子树质心并归一化 |

目标模式遇负编号时回退。
相机采用固定位置与姿态。
光源采用固定位置与方向。
此光源分支不归一化方向。
因此它保留非单位方向长度。

相机采用全局Z轴构造视轴。
矩阵按行展开，视轴作为列。
重合目标产生零矩阵。
极点目标保留退化横轴。
接口不添加CPU正交回退。
归一化使用sqrt与分量除法。
Warp 1.15.0将kEps设为零。
浮点平方下溢时返回零向量。
极小非零方向仍可能归一化。
零质量质心沿用既有Warp分支。
该质心不会回退到体位置。

依据：[冻结camlight源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py#L773-L938)。
归一化见[Warp向量源码](https://github.com/NVIDIA/warp/blob/v1.15.0/warp/native/vec.h#L1033-L1040)。
零阈值见[Warp常量](https://github.com/NVIDIA/warp/blob/v1.15.0/warp/native/builtin.h#L527)。
冻结依赖要求见[上游配置](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/pyproject.toml#L27-L33)。

## 常驻设备链

计划一次上传四组模型字段。
空相机光源不上传第四组。
计划一次编译所需同步内核。
状态创建时申请四组输出缓冲。
空相机光源仍保留输出守卫。
静态geom初始化保持原有语义。

完整辅助更新按以下顺序执行：

```text
rigid -> attached -> com -> camlight
```

相机光源采用两段内核。
第一段读取刚体设备结果。
它计算固定、跟踪与体目标。
它保存质心目标的局部位置。
第二段读取质心设备结果。
它调整质心跟踪与质心目标。
第二段只读取本世界已有输出。
同步完成保证两段读写顺序。
阶段间不上传或回读物理结果。
更新不重新分配或编译内核。
本批不修改内部运行时ABI。

qpos与mocap写入废弃全部结果。
刚体更新也废弃全部派生结果。
质心更新废弃相机光源就绪。
附着更新不废弃相机光源结果。
相机光源更新先检查体与质心。
两段成功后才记录阶段就绪。
驱动失败沿用运行时隔离机制。
输入错误保留已完成的结果。
回读检查四组守卫与有限值。
失败不发布部分宿主快照。
数据继续持有设备资源所有者。
计划释放不破坏已完成回读。

## 静态参考与边界

样本位于`fixtures/camlight/`。
样本含6个体与2个自由度。
样本含1个mocap体。
相机与光源各有8项。
样本覆盖世界体与五种模式。
两类目标均覆盖负编号回退。
样本固定八组f32输入状态。
原生参考保留f64结果。
固定种子为1789。
manifest冻结四个文件哈希。
产品测试只读取静态参考。
开发期C++生成器另行执行。

世界数覆盖1、2、5、513。
原生参考记录2084组世界比较。
独立参数记录1042组世界比较。
两类比较均覆盖四项结果。
绝对与相对容限均为2e-5。
参数周期采用以下值：

```text
2,3,5,7,11,3,5,7,2,11
```

边界验收另检查以下行为：

- 重合目标与全局Z轴极点。
- 零方向、极小方向与下溢。
- 负目标保留非单位方向。
- 零质量子树与零自由度。
- 仅相机、仅光源与空集合。
- 空字段周期i32::MAX。
- 首尾世界与跨块局部写入。
- 多状态与历史快照隔离。
- 跨计划身份拒绝与就绪检查。
- 计划释放后的设备回读。
- 四项结果的非有限值拒绝。
- 首尾四标量守卫的拒绝。

退化边界采用冻结公式检查。
原生退化回退不充当Warp参考。
守卫测试只改写私有测试缓冲。
产品接口不开放此类写入。

## Windows实测

平台：Windows x86_64 MSVC。
系统：10.0.26200.0。
GPU：RTX 4070 Ti SUPER。
驱动：596.36。
Rust：1.99.0。
工具链：CUDA 12.8.1。
参考原生候选：3.12.0。
正式依赖路线仍待冻结。

实际命令与结果见下节。

## 验证命令

```powershell
cargo test --locked
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --features cuda-probe --all-targets -- -D warnings
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-features --all-targets -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-features --all-targets --release -- -D warnings
cargo +stable-x86_64-pc-windows-msvc doc --locked --all-features --no-deps
scripts/test-windows-resident-kinematics.ps1 -AllFeatures
scripts/test-windows-resident-kinematics.ps1 -AllFeatures -Release
```

原生回归先预备DLL与环境。
本批复用已有原生mock产物。
脚本核对原生DLL与参考哈希。
常驻脚本不执行原生参考生成。
常驻脚本不调用Python。

GNU默认测试通过162项。
GNU文档测试通过8项。
MSVC两路各通过194项宿主测试。
MSVC两路各通过8项文档测试。
默认列表各忽略99项探针。
显式执行后各通过99项探针。
这些探针包含GPU与原生测试。
两路均无失败与残留忽略项。
四路Clippy均通过。
MSVC文档构建通过。

两次常驻脚本各通过26项。
每次包含3项宿主哈希检查。
每次另执行23项GPU测试。
其中6项验证相机光源行为。
两次GPU测试均无忽略项。

| 配置 | 原生世界比较 | 原生最大绝对误差 | 参数世界比较 | 参数最大绝对误差 |
| --- | --- | --- | --- | --- |
| debug | 2084 | `1.1379931486033001e-6` | 1042 | `6.788120066048009e-7` |
| release | 2084 | `1.1379931486033001e-6` | 1042 | `6.788120066048009e-7` |

本批保留两份完整常驻证据：

```text
target/resident-kinematics/f28995aa1fab47a489a6d9e84ee81698/
target/resident-kinematics/a1d4269c1f8c41828b608a9dade56044/
```

每份目录包含日志与report.json。
report记录平台、GPU与驱动。
report记录基础提交与脏树状态。
报告不声称执行Linux验收。

本批人工核对相关文档。
本批检查99条本地文档链接。
本批不运行Python审计工具。
本批不声称完成全库文档审计。

回归日志保留以下目录：

```text
target/camlight-verification/
```

## 架构收尾

Normify开启独立增量记录。
本批仅激活两个辅助叶子。
完整模型与物理阶段保持计划态。
源码同步未发现未登记文件。
最终校验得到零错误与零警告。
架构构建通过。
本批不关闭历史变更记录。

## 剩余G01范围

本批只实现相机光源位姿子集。
完整Model与Data仍待实现。
模型参数刷新仍待实现。
肌腱与flex位置仍待实现。
休眠相关副作用仍待实现。
等价导入与完整阶段仍待实现。
Linux与双平台验收仍待执行。
本批不统计完整引擎完成率。
