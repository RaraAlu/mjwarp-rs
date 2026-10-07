# Windows质心GPU探针

日期：2026-10-07。
范围：G01刚体质心子集。
本轮只验收Windows。
完整U057与G01仍待实现。
本轮不选择正式内核路线。

## 源码与入口

```text
src/physics/mod.rs                  GPU质心与惯量映射
tests/com_position_probe.rs         静态原生与解析参考
tests/support/rigid_reference.rs    两项探针共用DTO
scripts/test-windows-com-position.ps1
fixtures/com-position/reference.cpp
fixtures/com-position/massless-tree.xml
fixtures/com-position/massless-tree.mjb
fixtures/com-position/{mixed-joints,inertial-tree,rotated-tree,zero-dof,massless-tree}.json
fixtures/com-position/manifest.json
```

```rust
pub fn probe_com_position(
    session: &TransferSession,
    model: &InertialModelInput,
    worlds: usize,
    qpos: &[f32],
) -> Result<ComPositionOutput, TransferError>;
```

接口复用二十一字段子集。
模型参数仅共享一份。
状态采用`[world,nq]`布局。
GPU先计算刚体运动学。
宿主检查该阶段全部输出。
质心内核读取同一设备缓冲。
宿主不重新上传体变换。
宿主不执行物理公式。
每个线程串行处理一个世界。
此调度不承诺生产性能。
每次调用仍上传并编译。
接口不使用PTX缓存或图捕获。
接口需要NVIDIA驱动与NVRTC。
默认构建不需要GPU工具链。
本轮不新增第三方依赖。

## 字段语义

结果采用f32与连续切片。
`world()`检查世界索引。
宿主结果保留独立所有权。
它不暴露设备地址。

| 输出 | 每世界标量数 | 语义与单位 |
| --- | --- | --- |
| `subtree_mass` | `nbody` | 子树质量；kg |
| `subtree_com` | `3*nbody` | 世界坐标子树质心；m |
| `cinert` | `10*nbody` | 每体根子树质心坐标惯量 |
| `cdof` | `6*nv` | 空间运动映射；角向量先行 |

cinert前六项采用以下顺序：
`xx,yy,zz,xy,xz,yz`。
这六项采用`kg*m^2`。
随后三项保存质量一阶矩。
这三项采用`kg*m`。
最后一项保存质量，单位kg。
GPU旋转主惯量并移轴。
GPU从拓扑推导根体索引。
直接世界子体充当根体。
其后代使用该根子树质心。
世界体惯量输出保持零。
GPU反向累加子树质量与矩。

free前三项映射世界平移轴。
free后三项映射体旋转轴。
ball映射三个体旋转轴。
slide只映射世界线向量。
hinge映射角轴与偏心线向量。
旋转线向量包含米制偏移。
armature与damping不影响本阶段。

依据：[冻结Warp质心源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py#L613-L771)。
根体定义见[原生拓扑源码](https://github.com/google-deepmind/mujoco/blob/3.12.0/src/user/user_model.cc#L2661-L2666)。
许可说明见[第三方说明](../THIRD_PARTY_NOTICES.md)。

## 严格边界

输入沿用刚体探针检查。
世界体保持零位置与单位姿态。
世界体质量与主惯量必须为零。
模型四元数保持近似单位长度。
slide与hinge轴也须归一化。
状态四元数需保持可用长度。
状态标量必须有限。
接口支持静态祖先与零自由度。
接口拒绝零世界与布局溢出。
接口不处理mocap或休眠。
接口不处理柔性体与逐世界参数。
它不计算质量矩阵与积分。
它不替代完整Model或Data。
它不替代U057等价入口。

## 零质量参考差异

冻结Warp只检查质量是否为零。
非零时它计算加权平均。
零质量时它保留零加权矩。
原生3.12检查最小质量阈值。
低于阈值时它选取`xipos`。
两者在此边界不等价。

依据：[原生质心实现](https://github.com/google-deepmind/mujoco/blob/3.12.0/src/engine/engine_core_smooth.c#L226-L323)。

新增样本包含四个静态体。
三个体具有非零惯性质心。
全部质量与惯量均为零。
JSON保留原生CPU的实际输出。
测试用解析零值核对Warp。
测试不把该边界计入原生等价。
微小正质量测试采用`1e-20`。
两体质心分别采用以下值：
`[2,4,6]`与`[2,0,7]`。
手算根子树质心为`[2,2,6.5]`。
该测试证明探针不采用阈值回退。
它不声称原生CPU提供同值。

## 内存与失败

接口沿用内部八参数内核ABI。
各缓冲保持同一会话。
会话锁串行化提交与同步。
两个阶段均检查首尾哨兵。
有效区先填NaN。
返回前检查每项输出有限性。
有限输入也可能导致数值溢出。
探针拒绝整个非有限结果。
拒绝不发布部分世界结果。
数值拒绝不隔离健康设备会话。
世界零质量也保留原浮点运算。
探针不清零溢出的世界惯量。
后端失败仍隔离会话。
完成不确定时仍保留实际资源。
资源包括设备缓冲与内核模块。
测试不注入真实设备故障。
接口不得静默切换CPU。

## 复验命令

```powershell
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
./scripts/test-windows-com-position.ps1
./scripts/test-windows-com-position.ps1 -Release
./scripts/test-windows-com-position.ps1 -AllFeatures
./scripts/test-windows-com-position.ps1 -AllFeatures -Release
./scripts/test-windows-kinematics.ps1 -AllFeatures
./scripts/test-windows-kinematics.ps1 -AllFeatures -Release
./scripts/test-windows-native.ps1 -AllFeatures -RestrictedRuntime
./scripts/test-windows-native.ps1 -AllFeatures -Release -RestrictedRuntime
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-features --all-targets -- -D warnings
```

GNU命令需要仓库私有MinGW。
GPU命令使用另装MSVC工具链。
Rust默认工具链仍保持GNU。
脚本核对全部固定输入哈希。
全feature模式也核对原生DLL。
产品脚本不生成模型或参考。
产品脚本不启动Python。
NVRTC清单只记录目录内文件。
该清单不证明实际加载路径。

## 独立参考制备

以下步骤不属于产品链。
它需要VS x64编译器环境。
参考采用MuJoCo 3.12.0。

```powershell
$m = (Resolve-Path target/toolchains/mujoco-3.12.0/package).Path
$env:PATH = "$m\bin;" + $env:PATH
New-Item -ItemType Directory -Force target/com-reference | Out-Null
cl /nologo /EHsc /W4 /WX /std:c++17 /I"$m\include" fixtures/com-position/reference.cpp /Fo:target/com-reference/reference.obj /Fe:target/com-reference/reference.exe /link /LIBPATH:"$m\lib" mujoco.lib
target/com-reference/reference.exe --mjb fixtures/native-probe/mixed-joints.mjb target/com-reference/mixed-joints.json
target/com-reference/reference.exe --mjb fixtures/native-probe/inertial-tree.mjb target/com-reference/inertial-tree.json
target/com-reference/reference.exe --mjb fixtures/kinematics/rotated-tree.mjb target/com-reference/rotated-tree.json
target/com-reference/reference.exe --mjb fixtures/native-probe/zero-dof.mjb target/com-reference/zero-dof.json
target/com-reference/reference.exe --xml fixtures/com-position/massless-tree.xml target/com-reference/massless-tree.json target/com-reference/massless-tree.mjb
```

原生程序独立计算参考。
它不调用Rust候选算法。
产品测试只读取静态结果。
容限在首轮GPU运行前固定：

```text
abs(actual-reference) <= 2e-5 + 2e-5*abs(reference)
```

参考量化输入状态至f32。
参考保留double输出。
种子固定为1789。
五个模型各含八组状态。
四个既有模型覆盖四类关节。
偏心旋转与静态祖先沿用旧样本。
批量规模覆盖1、255、256。
批量规模也覆盖257与513。

## 本轮验证

平台采用Windows x64 MSVC。
系统版本为`10.0.26200.0`。
GPU采用RTX 4070 Ti SUPER。
驱动版本为596.36。
Rust版本为1.99.0。
原生DLL哈希如下：
`79b61d22b4d230a00bd31930fc6943f8ea8c6174a1bf4a6bf90f2459dbf8df1b`。

| 配置 | 宿主测试 | 文档测试 | 显式设备及原生回归 |
| --- | --- | --- | --- |
| 默认GNU | 130通过 | 6通过 | 不执行 |
| native-only MSVC | 136通过 | 6通过 | 原生17；受限17 |
| CUDA-only MSVC | 145通过 | 6通过 | 质心调试/发行各5 |
| 全feature MSVC调试 | 155通过 | 6通过 | 全量60；质心6 |
| 全feature MSVC发行 | 155通过 | 6通过 | 全量60；质心6 |

全量60项包含设备与原生测试。
它包含此前54项回归。
新增六项检验质心子集。
旧运动学脚本调试/发行各6项。
两轮均保持1347世界比较。
原生全feature两轮各22项。
受限原生两轮也各22项。
基础与探针测试不证明完整物理。

四种feature配置均通过Clippy。
Clippy拒绝全部警告。
fmt与警告拒绝rustdoc均通过。
PowerShell语法检查通过。
独立C++参考通过`/W4 /WX`。
五组JSON重复生成哈希一致。
新增MJB重复生成哈希也一致。

质心比较统计如下：

| 配置 | 测试数 | 原生比较世界 | 解析零质量世界 | 解析微小质量世界 |
| --- | --- | --- | --- | --- |
| CUDA调试/发行各轮 | 5 | 1315 | 8 | 1 |
| 全feature调试/发行各轮 | 6 | 1347 | 16 | 1 |

最大绝对误差如下：
`4.158170539447781e-6`。
四轮均取得该值。
容限始终保持`2e-5 + 2e-5*abs(reference)`。
解析边界不计入原生比较。

报告路径如下：

```text
target/com-position-probe/9fc5b1f0ab0447b0baa9f22782a1acd2/report.json  CUDA调试
target/com-position-probe/ece466a82b2149ad9e8898bbf42ecd19/report.json  CUDA发行
target/com-position-probe/aca1ba928f8645e881f78104db341ecd/report.json  全feature调试
target/com-position-probe/5d161f3f40ea49abb774b31950e060bd/report.json  全feature发行
target/kinematics-probe/356fea8998434dd993cddca70e4d6876/report.json
target/kinematics-probe/0b763769cffc429d820c647f89b8f9d1/report.json
target/native-probe/bb4ed949d76347898c31a3155a72d268/report.json
target/native-probe/42bf57bd2a1d48769232e62d3fb499d4/report.json
target/native-probe/fdf219b028394e77bc202ef825bee287/report.json
target/native-probe/a5491a2205104451bbaa44bd7965a2f6/report.json
target/com-checks/*.log
```

四轮报告记录开发基准HEAD：
`88f82b3084d72bd6b8afe6fbde1fbfb0dc1df410`。
四轮也记录`workingTreeDirty=true`。
该值准确表示提交前验证。
干净提交后仍需复跑烟测。
烟测另存真实HEAD与干净状态。

## 架构收尾

本轮新增三个实际辅助契约。
本轮保留此前187项契约。
架构合计190项契约。
其中184项使用rpc协议标识。
该标识不表示网络服务。
结构保留25节点与20叶子。
依赖数增加至72。
12项规则保持原样。
七个产品模块继续保持计划态。
指纹只从真实源码计算。
收尾使用当前变更登记。
本轮不关闭旧全量字段变更。

完整G01与Linux仍待验收。
