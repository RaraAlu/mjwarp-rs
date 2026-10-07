# Windows几何与site运动学探针

日期：2026-10-07。
范围：G01附着位姿辅助子集。
本轮只验收Windows。
完整U018与U062仍待实现。
本轮不冻结正式内核路线。

## 文件与接口

```text
src/model/attached.rs                       六字段检查与独占输入
src/physics/attached.rs                     GPU附着变换与只读结果
tests/attached_kinematics_probe.rs           固定参考与失败边界
scripts/test-windows-attached-kinematics.ps1 哈希、测试与证据报告
fixtures/attached-kinematics/                XML、MJB与静态原生参考
```

```rust
pub fn probe_attached_kinematics(
    session: &TransferSession,
    model: &AttachedModelInput,
    worlds: usize,
    qpos: &[f32],
) -> Result<AttachedKinematicsOutput, TransferError>;
```

输入组合已有刚体模型子集。
新增模型独占六项字段。
所有参数仍共享一份。
qpos采用`[world,nq]`布局。
公开输入与输出保持f32。
六项字段来自结构化输入。
原生FFI暂不导出这六项。
本轮不新增第三方依赖。

| 输入字段 | 长度 | 语义 |
| --- | --- | --- |
| geom_bodyid | ngeom | 几何所属体索引 |
| geom_pos | 3*ngeom | 体坐标系内位置，米 |
| geom_quat | 4*ngeom | 体坐标系内姿态，wxyz |
| site_bodyid | nsite | site所属体索引 |
| site_pos | 3*nsite | 体坐标系内位置，米 |
| site_quat | 4*nsite | 体坐标系内姿态，wxyz |

构造器校验长度与有限性。
体索引必须包含在模型内。
世界体索引零保持合法。
局部四元数必须接近单位。
平方范数容限固定为2e-6。
构造器不改写四元数。
字段保留负零位模式。
私有字段仅提供只读借用。
输出布局另检查组合容量。

## GPU数据链

GPU先计算已有刚体位姿。
新内核复用同一设备缓冲。
宿主不重新上传体位姿。
新内核读取世界位置与姿态。
GPU执行局部旋转与平移。
GPU生成几何与site世界矩阵。
每线程独占一个世界的输出。
内核适配继续使用固定ABI。
所有缓冲保持同一会话。
同步完成后才发布宿主结果。
结果包含已有刚体只读视图。
结果独立于模型与会话寿命。

| 输出字段 | 每世界长度 |
| --- | --- |
| geom_xpos | 3*ngeom |
| geom_xmat | 9*ngeom |
| site_xpos | 3*nsite |
| site_xmat | 9*nsite |

矩阵按行展开。
每世界步长如下：

```text
12 * (ngeom + nsite)
```

缓冲首尾各保留四项守卫。
输出槽先填充NaN。
回读检查守卫与全部有限性。
任何失败都拒绝发布部分结果。
数值错误不自动隔离健康会话。
测试随后验证健康调用。

## 兼容边界

公式依据冻结Warp源码。
它复用刚体四元数辅助公式。
它覆盖世界体与静态子树。
它允许零geom或零site。
两个集合均空时保留刚体结果。
它允许零自由度与批量世界。
它保留原有四类关节范围。

冻结Warp跳过静态geom更新。
它在状态创建时初始化静态值。
本探针每次重算全部geom。
因此本探针不复刻缓存副作用。
它不接收调用方已有输出缓冲。
它不提供完整make_data生命周期。
严格输入不替代等价put_model。

本探针不接入mocap。
它不更新相机与光源。
它不更新柔性体与肌腱。
它不实现休眠与唤醒。
它不提供常驻执行计划。
每次调用仍上传模型并编译。
它不承诺生产性能。
Linux与清洁部署仍待验收。

依据：[冻结运动学源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py)。

## 独立参考与测试

参考采用官方MuJoCo 3.12.0。
候选版本不改变Warp基线。
独立C++程序调用原生运动学。
它不调用Rust或GPU公式。
产品构建与测试只读静态参考。
制备程序不进入产品构建。
产品脚本不启动Python。

五组样本分别覆盖以下分支：

- 四类关节与偏心旋转。
- 同体多附着与旋转子体。
- 世界体与静态祖先。
- 零自由度与空附着集合。
- 仅geom与仅site。

每组固定八项状态。
种子固定为1789。
原生输入先窄化为f32。
原生输出保持双精度。
绝对与相对容限均为2e-5。
测试不放宽已有刚体容限。
脚本核对十六项固定哈希。
宿主参考测试也核对这些哈希。

跨块测试采用以下世界数：

```text
1, 255, 256, 257, 513
```

解析测试独立检查两项变换。
它检查世界体几何矩阵。
它检查偏心ball旋转后site。
测试销毁模型与GPU会话。
测试随后读取独立宿主结果。

失败测试覆盖以下边界：

- 零世界与错误qpos长度。
- 非有限状态与零状态四元数。
- 有限附着输入引发输出溢出。
- 第一世界健康、第二世界失败。
- 失败后健康会话继续运行。

本次不声明完整G01验收。
原生候选不替代冻结Warp证据。

## 复验命令

GNU检查先使用仓库私有MinGW。
GPU检查采用另装MSVC工具链。
默认GNU工具链保持不变。
GPU执行需要NVIDIA与NVRTC。

```powershell
cargo fmt --check
cargo test --locked
cargo test --locked --release
cargo clippy --locked --all-targets -- -D warnings
scripts/test-windows-attached-kinematics.ps1
scripts/test-windows-attached-kinematics.ps1 -Release
scripts/test-windows-attached-kinematics.ps1 -AllFeatures
scripts/test-windows-attached-kinematics.ps1 -AllFeatures -Release
```

独立制备见[样本说明](../fixtures/attached-kinematics/README.md)。

## 本轮GPU验证

平台采用Windows x64。
系统版本为`10.0.26200.0`。
GPU采用RTX 4070 Ti SUPER。
驱动版本为596.36。
MSVC Rust版本为1.99.0。
NVRTC目录采用CUDA 12.8.1。
默认GNU工具链保持不变。

| 配置 | 附着设备用例 | 内核适配用例 | 比较世界数 |
| --- | ---: | ---: | ---: |
| CUDA调试 | 4 | 3 | 1323 |
| CUDA发布 | 4 | 3 | 1323 |
| 全feature调试 | 4 | 3 | 1323 |
| 全feature发布 | 4 | 3 | 1323 |

四轮均无失败或忽略。
四轮沿用固定容限。
最大绝对误差保持一致：

```text
3.991163102234907e-7
```

每轮比较包含以下记录：

- 五组原生参考共40个世界。
- 跨块批量共1282个世界。
- 失败后恢复比较一个世界。

解析与溢出边界另行执行。
这些检查不充当原生世界比较。
空附着样本只验收空结果边界。
项目不把它视为完整物理验收。

全feature显式回归各通过79项。
调试与发布均无失败或忽略。
该回归包含原有设备探针。
原生受限回归各通过22项。
两个构建模式均完成受限运行。

| 证据 | 本机报告目录 |
| --- | --- |
| CUDA调试 | `target/attached-kinematics-probe/7f5ba1d15f9b43119554b971b3a1ecaf/` |
| CUDA发布 | `target/attached-kinematics-probe/17bea00713124d868e5eb7f44e299dfa/` |
| 全feature调试 | `target/attached-kinematics-probe/0423a22b8d844f7ab823bf5d24eff224/` |
| 全feature发布 | `target/attached-kinematics-probe/ba92cf1ba0e744b8885d70c5cdea536e/` |
| 原生调试与受限运行 | `target/native-probe/5e34eb64d20546a2a644afeb16033596/` |
| 原生发布与受限运行 | `target/native-probe/53e45ad8d660419fa32588827ec331a6/` |
| 全量回归日志 | `target/attached-reference/gpu-all-{debug,release}.log` |

报告记录提交前的真实基线：

```text
e9173d78ddceccb45c2ceeb2fd5e1563a61b7243
```

报告同时标记工作区修改。
NVRTC清单只代表目录文件。
它不证明实际加载路径。
本机成功不代表清洁部署。

生成器通过`/W4 /WX`检查。
五份JSON重复哈希匹配。
三份MJB重复哈希也匹配。
十六项Git暂存哈希匹配。
文档链接与脚本语法检查通过。

## 宿主与工程检查

| 普通测试配置 | 调试宿主 | 发布宿主 | 每配置文档 |
| --- | ---: | ---: | ---: |
| 默认GNU | 145 | 145 | 7 |
| 原生MSVC | 151 | 151 | 7 |
| CUDA MSVC | 163 | 163 | 7 |
| 全feature MSVC | 173 | 173 | 7 |

普通测试仍忽略设备用例。
这些数量不代表物理验收。
四组clippy拒绝警告并通过。
格式检查也通过。
全feature文档构建拒绝警告。
该构建也通过检查。
宿主日志保留在以下目录：

```text
target/attached-reference/host-*.log
target/attached-reference/clippy-*.log
target/attached-reference/doc.log
```

一次并行复验出现产物缺失。
随后采用串行方式完整复验。
本轮修复两个新增lint问题。
修复不改动物理公式与容限。

工程复验还执行以下命令：

```powershell
cargo +stable-x86_64-pc-windows-msvc test --locked --features native-model-probe
cargo +stable-x86_64-pc-windows-msvc test --locked --features native-model-probe --release
cargo +stable-x86_64-pc-windows-msvc test --locked --features cuda-probe
cargo +stable-x86_64-pc-windows-msvc test --locked --features cuda-probe --release
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release -- --ignored --test-threads=1
scripts/test-windows-native.ps1 -AllFeatures -RestrictedRuntime
scripts/test-windows-native.ps1 -AllFeatures -Release -RestrictedRuntime
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-targets --features native-model-probe -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-targets --features cuda-probe -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-targets --all-features -- -D warnings
$env:RUSTDOCFLAGS = '-D warnings'
cargo +stable-x86_64-pc-windows-msvc doc --locked --all-features --no-deps
```

## 架构收尾

本轮新增两个辅助叶子。
架构含30节点与24叶子。
旧196项契约继续保留。
本轮新增五项辅助契约。
架构共登记201项契约。
依赖数量为91。
六个容器继续保留阅读布局。
十二条架构规则保持不变。
七个产品模块继续保持计划。
二十个计划态节点继续保留。
本轮只激活附着辅助叶子。

提交前检查取得零错误。
此时保留一项临时协作警告。
本轮收尾随后关闭该变更。
完整G01与Linux继续保持待办。
