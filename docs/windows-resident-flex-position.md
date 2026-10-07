# Windows柔体位置子集

日期：2026-10-07。
范围：G01严格辅助增量。
完整G01仍待实现。
完整G22也仍待实现。

## 实现范围

GPU计算节点与顶点位置。
直接顶点读取所附体的位姿。
非居中顶点应用局部坐标。
居中顶点直接复制体位置。
插值顶点读取八个邻接节点。
权重采用三线性基函数。
网格支持多个单元。
查找与局部坐标沿用钳制。
节点先应用所附体的位姿。
零局部节点直接复制体位置。
全部柔体字段保持共享。
既有参数仍支持独立周期。
公开物理输入与结果采用f32。

| 接口 | 职责 |
| --- | --- |
| `FlexPositionFields` | 共享原生位置字段 |
| `FlexPositionModelInput` | 检查字段与混合模型 |
| `with_flex_positions()` | 创建可选柔体计划 |
| `update_flex_positions()` | 更新位置子集 |
| `snapshot.flex_positions()` | 访问可选快照 |
| `FlexPositionOutput` | 独占宿主结果 |
| `FlexPositionWorld` | 单世界只读视图 |

新入口包装已检查的混合模型。
它保留全局肌腱编号。
旧入口不增加设备结果缓冲。
旧快照的柔体结果返回`None`。
新快照返回`Some`。
空柔体仍提供空世界视图。
新入口最多使用十个结果缓冲。
它新增一个柔体浮点缓冲。
每世界先存节点再存顶点。
两个字段都按xyz顺序展开。

```text
payload = 3*nflexnode + 3*nflexvert
flexnode_xpos: [world, node, 3]
flexvert_xpos: [world, vertex, 3]
```

## 检查与调度

检查器先校验全部字段尺寸。
它再检查有限性与体引用。
节点与顶点范围必须连续。
所有范围必须完整覆盖。
直接柔体不能携带节点。
线性网格要求正单元数。
节点数必须匹配网格。
居中字段不能丢弃局部偏移。
查找坐标必须安全转换为整数。
检查器不强制坐标落在单位立方。
编译器可能留下微小越界值。
GPU保留上游钳制语义。
检查器拒绝高阶与壳插值。
布局检查先于设备分配。

位置更新只依赖刚体结果。
它不要求附着与质心就绪。
完整辅助更新自动调用该子集。
每个线程独占一个世界。
同一线程先更新全部节点。
该线程随后更新全部顶点。
阶段间不回读宿主结果。
GPU计算不触发CPU回退。
模型上传与内核编译只执行一次。

状态写入废弃全部派生结果。
刚体重算也废弃柔体位置。
附着与质心重算不废弃它。
错误输入保留已有结果。
计划身份检查拒绝跨模型复用。
回读检查双端守卫与有限性。
失败不发布部分宿主快照。
数据继续保留设备所有者。
计划释放不破坏已有结果。
快照独占自己的宿主缓冲。

## 原生参考

目录见[样本说明](../fixtures/flex-position/README.md)。
MuJoCo版本为3.12.0。
开发者独立运行C++参考工具。
顶点参考调用原生柔体函数。
节点参考调用原生变换函数。
原生Data不公开节点位置。
产品测试只读取静态参考。
产品链不运行模型编译器。
产品链不引入Python。
清单锁定XML、MJB与JSON。
清单也锁定参考工具与DLL。

| 样本 | 数量 |
| --- | --- |
| 柔体 | 4 |
| 节点 | 20 |
| 顶点 | 80 |
| 确定性状态 | 12 |
| 每状态位置标量 | 300 |

样本包含两类直接柔体。
样本也包含两类线性柔体。
各类型覆盖居中与局部偏移。
首个线性网格包含两个单元。
样本含球关节、滑动与转动。
样本含独立mocap运动。
参考工具明确改写节点偏移。
XML需经该工具才能复现。
样本不要求弹性力。
原生编译器提示四条警告。

GPU测试覆盖1、2、5、513世界。
四轮更新比较2084组状态。
每种构建比较625200个位置值。
绝对与相对容限均为`2e-5`。
本轮不放宽既有容限。
测试另检查刚体与肌腱结果。
测试覆盖空集合与直接入口。
测试覆盖阶段依赖与状态失效。
测试覆盖身份与生命周期。
守卫测试检查首尾世界。
它拒绝NaN与正负无穷。

## 验证命令

```powershell
cargo test --locked
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release
scripts/test-windows-resident-kinematics.ps1 -AllFeatures
scripts/test-windows-resident-kinematics.ps1 -AllFeatures -Release
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release -- --ignored --test-threads=1
```

## 已执行结果

本机运行Windows原生验收。
两种构建都通过原容限。
最大位置误差均约`3.80e-7`。
本机GPU型号见下表。
驱动版本为596.36。
Rust版本为1.99.0。
CUDA工具目录版本为12.8.1。
系统版本为10.0.26200.0。

| 验证 | 结果 |
| --- | --- |
| GPU | NVIDIA RTX 4070 Ti SUPER |
| GNU默认宿主与文档测试 | 188通过 |
| MSVC全feature调试测试 | 231通过 |
| MSVC全feature发布测试 | 231通过 |
| 全部忽略探针，调试 | 128通过 |
| 全部忽略探针，发布 | 128通过 |
| 常驻验收脚本，调试 | 61通过 |
| 常驻验收脚本，发布 | 61通过 |
| 四路Clippy | 通过，拒绝全部警告 |
| rustfmt与文档构建 | 通过 |
| 位置直接对照 | 每模式2084组 |
| 位置标量对照 | 每模式625200项 |

MSVC宿主测试含12项文档测试。
常驻脚本含9项宿主测试。
它另含52项GPU测试。
忽略探针包含GPU与原生测试。
它们不表示完整物理验收。
完整命令与日志保存在下列目录。

```text
target/flex-position-verification/
```

本批新增依赖数量为零。
开发者另做范围内文档检查。
该检查读取10个源码文件。
它读取10个文档文件。
它核对120条本地链接。
它核对四个参考哈希。
审计钩子提示五处路径。
人工核对确认五处均为误报。
上游源码路径不属于本仓库。
三处PTX引用实际存在。
原生链接库来自配置目录。
本批不修改无关历史文档。

## 剩余边界

本轮只实现柔体位置子集。
它不提供柔体边与Jacobian。
它不提供弹性力与碰撞。
它不提供面姿态与高阶插值。
它不提供休眠与唤醒副作用。
完整原生字段转换仍待实现。
正式阶段ABI仍待冻结。
Linux真实验收仍待执行。
完整G01与U061继续保持计划。
下一步补齐休眠副作用。

[冻结位置源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py#L211-L317)。
[冻结线性权重](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/support.py#L1004-L1013)。
