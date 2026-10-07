# Windows质量矩阵GPU探针

日期：2026-10-07。
范围：G02刚体质量矩阵子集。
本轮只验收Windows。
完整U059与G02仍待实现。
U060分解与求解也仍待实现。
本轮不冻结正式内核路线。

## 文件与接口

```text
src/physics/mod.rs                  设备质心结果复用
src/physics/mass_matrix.rs          复合惯量与稠密矩阵
tests/mass_matrix_probe.rs          原生与解析参考
tests/support/rigid_reference.rs    共用只读模型输入
scripts/test-windows-mass-matrix.ps1
fixtures/mass-matrix/reference.cpp
fixtures/mass-matrix/armature-chain.xml
fixtures/mass-matrix/armature-chain.mjb
fixtures/mass-matrix/{mixed-joints,inertial-tree,rotated-tree,zero-dof,massless-tree,armature-chain}.json
fixtures/mass-matrix/manifest.json
```

```rust
pub fn probe_mass_matrix(
    session: &TransferSession,
    model: &InertialModelInput,
    worlds: usize,
    qpos: &[f32],
) -> Result<MassMatrixOutput, TransferError>;
```

Rust负责输入检查与主接口。
CUDA C++执行质量矩阵公式。
模型沿用二十一字段子集。
参数仅共享一份模型。
状态采用`[world,nq]`布局。
GPU先执行刚体与质心阶段。
各阶段先检查完整回读结果。
下一阶段复用同一设备缓冲。
宿主不重新上传体变换或惯量。
宿主不计算物理公式。
接口同步返回独立宿主结果。
每个线程串行处理一个世界。
此调度不承诺生产性能。
每次调用仍上传模型并编译。
接口不使用PTX缓存或图捕获。
接口需要NVIDIA驱动与NVRTC。
默认构建不需要GPU工具链。
本轮不新增第三方依赖。

## 结果语义

所有结果采用f32连续切片。
`world()`检查世界索引。
结果保留独立宿主所有权。
接口不暴露设备地址。

| 字段 | 每世界标量数 | 语义 |
| --- | --- | --- |
| `crb` | `10*nbody` | 子树复合惯量；打包沿用cinert |
| `matrix` | `nv*nv` | 完整对称稠密矩阵；按行展开 |

GPU反向累加子体惯量。
同根子树共用根质心坐标系。
GPU不把各根累加到世界体。
各根采用不同质心坐标系。
世界体复合惯量保持原值。
矩阵沿自由度祖先链计算。
非祖先耦合保持精确零值。
GPU复制对称位置的同一结果。
自由度armature只增加对角线。
damping不参与本阶段计算。

惯量打包顺序见[质心报告](windows-com-position-probe.md)。
矩阵元素不共享同一单位。
平移对平移采用kg。
旋转对旋转采用`kg*m^2`。
平移对旋转采用`kg*m`。
该单位与米制坐标约定一致。

算法依据：[冻结CRB源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py#L940-L1005)。
空间乘法依据：[冻结惯量公式](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/math.py#L112-L121)。
许可说明见[第三方说明](../THIRD_PARTY_NOTICES.md)。

## 严格边界

输入沿用刚体与质心预检。
世界体质量与主惯量必须为零。
模型姿态与关节轴须接近单位长。
状态四元数须保持可用长度。
输入状态必须有限。
布局检查字节容量与内核索引。
矩阵平方容量不能溢出。
本轮支持静态祖先与四类关节。
本轮支持零自由度与零惯量。
零自由度结果只包含CRB。
零惯量结果可以保持半正定。
接口不承诺正定或可逆。
接口不执行矩阵分解或求解。

模型子集不包含肌腱或执行器。
探针不叠加对应附加惯量。
它不提供完整功能准入检查。
调用方明确选择上述子集语义。
它不处理mocap或休眠。
它不处理柔性体与逐世界参数。
它不计算RNE偏置力或时间积分。
它不提供上游稀疏M布局。
它不替代U059等价入口。
它不替代完整G02或G03。

## 失败与所有权

内核沿用内部八参数ABI。
所有缓冲保持同一会话。
会话锁串行化提交与同步。
前三缓冲只供内核读取。
输出首尾各保留四个哨兵。
有效区先填NaN。
返回前检查全部哨兵与数值。
任何失败都不发布部分结果。
有限单体惯量也可能累加溢出。
测试单独覆盖该阶段的溢出。
数值拒绝不隔离健康会话。
后端失败仍隔离会话。
完成不确定时仍保留设备资源。
保留范围包含内核模块与缓冲。
测试不注入真实设备故障。
引擎不得静默切换CPU。

## 独立参考

参考采用官方MuJoCo 3.12.0。
它仍属于原生候选版本。
它不改变冻结Warp基线。
参考程序独立调用原生库。
它调用`mj_kinematics`与`mj_comPos`。
它调用`mj_crb`与`mj_fullM`。
它不调用Rust候选算法。
生成器拒绝肌腱与执行器模型。
产品测试只读取静态JSON与MJB。
产品链不运行该参考程序。
产品链不启动Python。

原生依据：[候选CRB实现](https://github.com/google-deepmind/mujoco/blob/3.12.0/src/engine/engine_core_smooth.c#L1739-L1808)。

| 样本 | 体数 | nv | 主要覆盖 |
| --- | --- | --- | --- |
| mixed-joints | 6 | 11 | 四类关节与偏心旋转 |
| inertial-tree | 8 | 13 | 静态祖先、分支与多关节 |
| rotated-tree | 7 | 12 | 旋转惯性系与非单位状态 |
| zero-dof | 1 | 0 | 世界体与空矩阵 |
| massless-tree | 4 | 0 | 多根零质量静态子树 |
| armature-chain | 4 | 3 | 非零armature、耦合与独立根 |

每个模型包含八组固定状态。
种子固定为1789。
输入状态先量化至f32。
参考输出保留double精度。
比较容限在首轮GPU执行前固定：

```text
abs(actual-reference) <= 2e-5 + 2e-5*abs(reference)
```

测试不得放宽此容限。
批量规模覆盖1、255、256。
批量规模也覆盖257与513。
测试核对矩阵精确对称。
测试核对非祖先元素为零。
测试用f64检查样本正定性。
该检查仅验证这些物理样本。
它不把分解能力加入产品链。
所有半正定输入仍保持合法。

新增链的默认矩阵如下：

```text
6.25   1.75     0
1.75   1.6875   0
0      0        4.75
```

手算公式为`T = 0.5 * v^T * M * v`。
速度`[1,2,3]`对应动能31.375。
零惯量模型只输出armature对角线。
该模型不声称原生可编译等价。
GPU测试明确验证这两项解析值。

## 复验命令

```powershell
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
./scripts/test-windows-mass-matrix.ps1
./scripts/test-windows-mass-matrix.ps1 -Release
./scripts/test-windows-mass-matrix.ps1 -AllFeatures
./scripts/test-windows-mass-matrix.ps1 -AllFeatures -Release
./scripts/test-windows-native.ps1 -AllFeatures -RestrictedRuntime
./scripts/test-windows-native.ps1 -AllFeatures -Release -RestrictedRuntime
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-features --all-targets -- -D warnings
```

GNU命令需要仓库私有MinGW。
GPU命令采用另装MSVC工具链。
Rust默认工具链仍保持GNU。
脚本先核对14项固定输入哈希。
全feature模式也核对原生DLL。
报告记录真实HEAD与工作区状态。
报告记录平台、GPU与驱动。
NVRTC清单只记录目录内文件。
该清单不证明实际加载路径。
本轮不提供清洁机器部署证据。

## 独立制备命令

以下命令不属于产品链。
它需要VS x64编译器环境。

```powershell
$m = (Resolve-Path target/toolchains/mujoco-3.12.0/package).Path
$env:PATH = "$m\bin;" + $env:PATH
New-Item -ItemType Directory -Force target/mass-reference | Out-Null
cl /nologo /EHsc /W4 /WX /std:c++17 /I"$m\include" fixtures/mass-matrix/reference.cpp /Fo:target/mass-reference/reference.obj /Fe:target/mass-reference/reference.exe /link /LIBPATH:"$m\lib" mujoco.lib
target/mass-reference/reference.exe --mjb fixtures/native-probe/mixed-joints.mjb target/mass-reference/mixed-joints.json
target/mass-reference/reference.exe --mjb fixtures/native-probe/inertial-tree.mjb target/mass-reference/inertial-tree.json
target/mass-reference/reference.exe --mjb fixtures/kinematics/rotated-tree.mjb target/mass-reference/rotated-tree.json
target/mass-reference/reference.exe --mjb fixtures/native-probe/zero-dof.mjb target/mass-reference/zero-dof.json
target/mass-reference/reference.exe --mjb fixtures/com-position/massless-tree.mjb target/mass-reference/massless-tree.json
target/mass-reference/reference.exe --xml fixtures/mass-matrix/armature-chain.xml target/mass-reference/armature-chain.json target/mass-reference/armature-chain.mjb
```

## 本轮验证

本轮平台采用Windows x64。
系统版本为`10.0.26200.0`。
GPU采用RTX 4070 Ti SUPER。
驱动版本为`596.36`。
Rust版本为`1.99.0`。

| 配置 | 宿主通过 | 文档通过 | 默认忽略 |
| --- | ---: | ---: | ---: |
| GNU默认 | 133 | 6 | 0 |
| MSVC原生输入 | 139 | 6 | 17 |
| MSVC CUDA | 149 | 6 | 33 |
| MSVC全feature调试 | 159 | 6 | 66 |
| MSVC全feature发布 | 159 | 6 | 66 |

全feature显式运行66项测试。
调试版与发布版各通过66项。
两轮均无失败或遗留忽略项。
格式与四组clippy均通过。
全feature文档构建拒绝警告。
该构建也通过检查。
旧刚体与质心脚本均通过。
原生限制运行检查也均通过。

| 矩阵配置 | 设备通过 | 原生世界比较 |
| --- | ---: | ---: |
| CUDA调试 | 5 | 1331 |
| CUDA发布 | 5 | 1331 |
| 全feature调试 | 6 | 1379 |
| 全feature发布 | 6 | 1379 |

每轮另通过解析动能检查。
每轮另通过半正定边界检查。
六模型参考覆盖48组状态。
最大绝对误差为`1.7808917590400597e-5`。
四轮均保持原定比较容限。
独立生成器通过`/W4 /WX`。
重复生成结果匹配七项哈希。
这些证据不来自零用例。

矩阵报告位于以下目录：

```text
target/mass-matrix-probe/b58e8e3d327641d5ab4a6ebf5a940047/
target/mass-matrix-probe/2a8378825c114ec08106e69b7ff7e50e/
target/mass-matrix-probe/2e2c63e4eaac46ee9b6f01ae3b2ab38f/
target/mass-matrix-probe/a2772945742940c8802eb47ff95cdd11/
target/mass-checks/
```

四份报告记录提交前工作区。
`workingTreeDirty`均为`true`。
它们记录以下真实基线：

```text
b1229f2ea3f03867ab12224a6f034eaf07915d58
```

## 架构收尾

物理层新增两个职责叶子。
`core`保留66项既有契约。
这些契约继续保持计划态。
矩阵叶子新增三项辅助契约。
本轮只激活真实矩阵叶子。
七个产品模块继续保持计划。
本轮不修改十二条架构规则。

架构现含27节点与21叶子。
架构登记193项契约。
旧190项契约全部保留。
66项物理契约只迁移归属。
Rust公开路径保持不变。
架构登记77条依赖。
六个容器各有阅读布局。
提交前检查取得零错误。
当时仍有一项临时警告。
它提示两项变更同时推进。
本轮收尾关闭矩阵辅助变更。
收尾后再次检查警告与漂移。
完整核心提案继续保持开放。

完整G01、G02与G03仍待验收。
Linux继续暂缓。
