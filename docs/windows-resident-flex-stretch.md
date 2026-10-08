# Windows柔体拉伸矩阵

日期：2026-10-08。
本报告验收拉伸辅助子集。
完整G01仍保留计划态。
完整G22与G25仍待实现。
本轮不新增CPU物理后端。

## 冻结契约

冻结提交保持不变：

```text
71da24d956378a87a703b6e1442b13aec0c4ac29
```

[冻结passive模块](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/passive.py)
定义拉伸块与缓存阶段。
[冻结types模块](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py)
按原生版本选择21或24系数。
本轮只实现21系数路线。
原生样本采用MuJoCo3.12.0。

活动分支要求直接插值。
它要求非刚性二维或三维柔体。
它要求非负材料地址。
它要求首个材料系数非零。
非活动分支清零后标记有效。
缓存阶段不筛选休眠树。
已有有效标志跳过矩阵改写。
位置阶段仍清除全部标志。

上游将负张力截断为零。
矩阵因此采用拉伸投影。
压缩时它不等于普通能量海森。
本轮不实现Newton求解器海森。

## API与设备布局

| 接口 | 行为 |
| --- | --- |
| `FlexHessianFields` | 共享单元、21系数、静长与边映射 |
| `FlexPositionModelInput::with_hessian` | 先检查边与单元，再启用矩阵 |
| `TendonWakeModelInput::with_flex_positions` | 保留矩阵、边、面与唤醒组合 |
| `KinematicsPlan::update_flex_hessian` | 读取常驻位置、边长与有效标志 |
| `KinematicsSnapshot::flex_hessian` | 仅提供已完成的可选矩阵快照 |
| `FlexHessianOutput::world` | 返回顶点六项与边九项切片 |

调用方先启用边计算。
字段检查拒绝错误材料跨度。
检查也拒绝非法单元与边映射。
有向边允许反向端点顺序。
替换边时重新检查单元映射。
拓扑与材料只共享一份。
矩阵另占一个守卫f32缓冲。
纯位置的十一缓冲保持不变。
节点与顶点仍使用独立区段。
矩阵内核跳过节点区段。

GPU按世界顺序遍历单元。
一个线程独占一个世界。
它累加共享边的三乘三块。
它用边块恢复顶点对角块。
它不依赖浮点原子累加。
当前实现不承诺性能等价。

矩阵与标志使用两段同步。
第二段只在首段成功后执行。
两段之间不上传或回读。
标志表示计算缓存状态。
显式回读另查守卫与有限性。
非有限结果阻止整个快照。
失败不发布部分矩阵。

常规update不计算矩阵。
它只保持G01缓存失效语义。
位置刷新后矩阵视图返回None。
调用方须显式计算新矩阵。
qvel与质心刷新保留几何缓存。
矩阵阶段要求位置与边就绪。
跨计划与未就绪调用保留状态。
计算或标记失败隐藏矩阵快照。
旧入口仍返回None。
空入口提供空矩阵切片。
数据与快照保留资源所有权。

## 独立参考与测试

静态样本见[参考说明](../fixtures/flex-stretch/README.md)。
MuJoCo提供模型与顶点几何。
原生字段不提供缓存矩阵。
C++工具独立组装稠密导数。
它抽取顶点块与有向边块。
该参考不复制GPU稀疏累加。
产品测试只读取冻结参考。
产品测试不启动Python。
产品测试不调用模型编译器。

| 检查 | 范围 |
| --- | --- |
| 原生几何与稠密导数 | 十二组固定状态 |
| 世界批量 | 1、2、5、513世界，四轮 |
| 数值对比 | 2084组，3044724个矩阵标量 |
| 几何偏移 | 非空节点区段、直接三角形与四面体 |
| 共享与方向 | 相邻单元累加、有向边转置 |
| 停用分支 | 刚性、一维、插值、缺材料、零首系数 |
| 缓存 | 重复位级相等、逐柔体部分有效 |
| 分析解 | 单位三角形、压缩张力截断 |
| 故障 | 首尾守卫、NaN、正负无穷、算术溢出 |
| 两段失败 | 计算与标记的会话不匹配 |
| 组合 | 边、面与休眠唤醒同时保留 |
| 边界与所有权 | 空入口、旧入口、身份、就绪与释放顺序 |

绝对与相对容限均为2e-5。
本轮没有放宽容限。
四路最大误差均为1.54036e-5。
四路采用同一固定参考。

## 实际验证结果

| 路线 | 实际结果 |
| --- | --- |
| GNU默认 | 188项宿主与12项文档通过 |
| MSVC全特性Debug | 239项宿主与12项文档通过 |
| MSVC全特性Release | 239项宿主与12项文档通过 |
| MSVC显式探针Debug | 155项通过；22原生与133GPU |
| MSVC显式探针Release | 155项通过；22原生与133GPU |
| 常驻脚本双模式 | 各98项；19宿主与79GPU |
| GNU失效测试双模式 | 各4项；1宿主与3GPU |
| GNU拉伸集成双模式 | 各6项；2宿主与4GPU |
| GNU拉伸内部双模式 | 各2项GPU |
| Clippy四路 | 零警告 |
| Rustdoc、格式与差异 | 全部通过 |
| C++参考作者工具 | `/W4 /WX`编译与执行通过 |
| 检出烟测 | 四项静态哈希通过；一项宿主复核通过 |

最终运行器执行21条命令。
全部命令返回零退出码。
默认测试跳过显式GPU用例。
显式探针实际运行155项。
本轮不把零用例当验收。

完整证据目录：

```text
target/flex-stretch-verification/final-results.json
target/flex-stretch-verification/final-check.log
target/flex-stretch-verification/document-check.json
target/resident-kinematics/f2eae9b9c0b34c29a8a63d16cf91c4f5/report.json
target/resident-kinematics/6db5b98f1c3347438abebc6d85ba532e/report.json
```

运行环境如下：

| 项目 | 值 |
| --- | --- |
| 系统 | Windows 10.0.26200.0 |
| GPU | NVIDIA GeForce RTX 4070 Ti SUPER |
| 驱动 | 596.36 |
| Rust | GNU与MSVC均为1.99.0 |
| CUDA工具链 | 12.8.1 |
| 原生参考 | MuJoCo3.12.0 |

验收记录父提交与脏工作区。
父提交为`1564796`。
本轮随后执行智能提交。

文档钩子扫描178个源码。
钩子也扫描62篇文档。
钩子给出九项路径提示。
人工复核认定九项误报。
上游数学路径不指向本地。
本地PTX与配置库均存在。
局部复核读取22份源码配置。
复核还读取12篇相关文档。
149项链接与18项哈希通过。
局部复核没有发现错误。
样本文本固定使用LF。
MJB保持二进制格式。
本轮另行检出四项样本。
检出后哈希仍匹配清单。

架构检查没有发现错误。
检查保留一项粒度提示。
柔体输入模块管理两个文件。
本轮保留同一模块归属。
该归属避免接口互相依赖。

## 复核命令

```powershell
scripts/test-windows-native.ps1 -AllFeatures
cargo test --locked
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features -- --ignored --test-threads=1 --nocapture
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release -- --ignored --test-threads=1 --nocapture
scripts/test-windows-resident-kinematics.ps1 -AllFeatures
scripts/test-windows-resident-kinematics.ps1 -AllFeatures -Release
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --all-targets --features cuda-probe -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-targets --all-features -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-targets --all-features --release -- -D warnings
cargo +stable-x86_64-pc-windows-msvc doc --locked --all-features --no-deps
cargo fmt --check
git diff --check
```

GPU运行需要NVIDIA与NVRTC。
原生运行需要MSVC环境。
GNU矩阵测试不需要原生运行。

## 未覆盖范围

矩阵乘法仍待实现。
24系数与新材料仍待实现。
二次插值不属冻结支持集。
节点计算沿用冻结上游。
本子集不重建原生TFI节点。
完整弹性与被动力仍待实现。
完整阶段入口仍待收口。
Linux尚未运行本轮验收。
本轮不关闭G01、G22或G25。
