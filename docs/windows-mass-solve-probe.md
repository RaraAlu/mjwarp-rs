# Windows质量矩阵求解GPU探针

日期：2026-10-07。
范围：G03正定刚体辅助子集。
本轮只验收Windows。
U060、U065与完整G03仍待实现。
矩阵乘法U073也仍待实现。
本轮不冻结正式内核路线。

## 文件与接口

```text
src/physics/mass_solve.rs            反向LDL与多右端项
src/physics/mass_matrix.rs           共享设备矩阵阶段
src/physics/mod.rs                   共享精度与刚体阶段
src/runtime/transfer.rs              四类类型化复制
src/runtime/transfer/kernel.rs       同类型固定内核ABI
src/diagnostics/mod.rs               坏主元位置与数值
tests/mass_solve_probe.rs            固定参考与残差
tests/batch_transfer.rs              f64位模式与字段复制
scripts/test-windows-mass-solve.ps1
fixtures/mass-solve/reference.cpp
fixtures/mass-solve/{mixed-joints,inertial-tree,rotated-tree,zero-dof,massless-tree,armature-chain}.json
fixtures/mass-solve/manifest.json
```

```rust
pub fn probe_mass_solve(
    session: &TransferSession,
    model: &InertialModelInput,
    worlds: usize,
    qpos: &[f32],
    rhs_count: usize,
    rhs: &[f32],
) -> Result<MassSolveOutput, TransferError>;
```

模型仍采用二十一字段子集。
模型参数仍共享一份。
状态采用`[world,nq]`布局。
右端项采用`[world,rhs,nv]`布局。
右端项数量必须大于零。
输入状态与右端项必须有限。
GPU按世界串行计算。
一次分解服务同世界多个右端项。
每次调用仍上传模型并编译。
接口不承诺生产性能。
接口不使用缓存或图捕获。
接口需要NVIDIA驱动与NVRTC。
默认构建不需要GPU工具链。
本轮不新增第三方依赖。

## 精度与设备链

公开模型、状态与右端项仍为f32。
公开物理结果也保持f32。
内部GPU采用f64计算。
Rust只执行检查与精度转换。
Rust不执行物理公式或求解。
GPU依次计算刚体、质心与矩阵。
GPU继续计算反向LDL与求解。
各阶段先核对完成与回读。
下一阶段复用同一设备缓冲。
宿主不重新上传变换或矩阵。
矩阵检查件不参与设备求解。
求解器保留未窄化的设备矩阵。
输出窄化后再次核对有限性。
接口同步返回独立宿主结果。
结果不暴露设备地址。

旧刚体与矩阵探针仍使用f32。
它们保留既有接口与容限。
GPU共享同一套阶段公式。
内部按标量类型选择精度。
f64版本也选择双精度三角函数。
固定ABI保持四缓冲与四标量。
三个浮点缓冲采用同一类型。
运行时新增原样f64复制。
连续字段也支持八字节f64。
模型字段组仍保持既有类型。
`upload_f64_batch`仍执行窄化。
新原样复制不改变该入口语义。
外部资源ABI仍只支持既有f32原型。

### 首轮失败与修复

首轮f32求解未通过参考比较。
小惯量放大了矩阵量化误差。
仅用f64分解不能恢复丢失信息。
测试专用f64计算确认此边界。
临时调查不属于产品回退。
调查日志保留在以下目录：

```text
target/solve-checks/initial-gpu-solve.log
target/solve-checks/quantization.log
```

修复采用完整内部GPU f64链。
修复不放宽比较容限。
固定参考也保持原始哈希。
调查后删除临时诊断用例。
正式用例仍覆盖全部六模型。
混合模型的小主元继续接受检查。

## 分解与结果

GPU采用反向LDL分解。
分解关系为`M = L^T D L`。
L采用隐含单位对角。
D采用严格正的对角。
GPU先解`L^T`，再除D，再解L。
GPU不执行主元选取或正则化。

| 字段 | 每世界标量数 | 布局 |
| --- | ---: | --- |
| `ld` | `nv*nv` | 行优先；下三角存L，对角存D，上三角为零 |
| `diagonal_inverse` | `nv` | D的倒数 |
| `solution` | `rhs_count*nv` | 连续`[rhs,nv]` |

`world()`检查世界索引。
结果支持会话销毁后继续借用。
零自由度返回三个空字段。
零右端项返回零解。
混合自由度不共享同一单位。
D沿用质量矩阵对应对角单位。
D倒数采用对应逆单位。
L元素采用相应坐标比值单位。
解单位取决于右端项约定。
力与力矩右端项对应广义加速度。
分解布局不等同上游qLD布局。
字段子集不携带完整能力元数据。
快照不能核验完整模型的能力。
调用方须显式确认子集范围。

分解依据：[冻结稀疏LDL](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py#L1085-L1241)。
独立参考依据：[原生分解与求解](https://github.com/google-deepmind/mujoco/blob/3.12.0/src/engine/engine_core_smooth.c#L1819-L1962)。
许可见[第三方说明](../THIRD_PARTY_NOTICES.md)。

## 严格边界

输入沿用刚体与质心预检。
布局检查平方、乘积与字节容量。
f64分配采用八字节容量检查。
内核标量索引仍限制为i32范围。
状态记录包含代码、自由度与数值。
主元必须严格为正且有限。
D倒数不得超过f32最大值。
最终f32结果必须全部有限。
GPU不夹紧主元或添加抖动。
求解接口不接受半正定系统。
原矩阵探针仍接受半正定系统。

坏主元返回`InvalidPivot`。
错误保留世界、自由度与数值。
主元诊断保留原始f64值。
一个失败世界阻止整个结果发布。
多个失败世界先报告最小世界索引。
同世界按反向分解顺序报告。
倒数溢出与窄化溢出单独报错。
倒数错误索引为`world*nv+dof`。
输出错误索引保留内部打包布局。
每世界先含三项状态标量。
随后依次存因子、倒数与解。
它不等于单个结果切片索引。
输出守卫检查先于失败状态。
共享守卫诊断仍采用f32近似。
未初始化的部分结果不得发布。
数值拒绝不隔离健康会话。
后端失败沿用传输隔离规则。
完成不确定时保留缓冲与内核。
测试不注入真实GPU硬件故障。

本子集不支持稀疏或分块调度。
它不处理休眠、mocap或柔性体。
它不叠加肌腱与执行器惯量。
它不实现LU或非对称求解。
它不提供通用矩阵输入接口。
它不替代完整功能准入检查。
它不承诺任意条件数的前向误差。
它不声明完整U060或U065等价。

## 固定参考与容限

六模型复用质量矩阵样本。
每模型包含八组固定状态。
状态种子固定为1789。
右端项种子固定为9701。
生成器先把输入量化至f32。
原生输出保留double精度。
生成器调用原生分解与求解。
生成器不调用候选公式。
产品测试只读取静态结果。
产品测试不生成参考。

首次GPU求解前固定以下容限：

```text
abs(actual-reference) <= 2e-4 + 2e-4*abs(reference)
||M*x-b||inf / (||M||inf*||x||inf+||b||inf) <= 2e-5
||L^T*D*L-M||inf / ||M||inf <= 2e-6
```

零分母采用绝对误差检查。
残差与重构使用独立原生M。
检查采用测试专用f64运算。
这些运算不进入产品求解链。
每项因子比较仍采用上述容限。
测试检查正主元与倒数关系。
测试检查上三角精确为零。
批量规模覆盖1、255、256。
批量规模也覆盖257与513。
右端项数量覆盖1、2、3与5。
测试覆盖位模式与八字节复制。
测试覆盖负主元与部分世界失败。
测试覆盖零与非有限主元。
测试核对f64主元低位信息。
测试覆盖倒数与窄化溢出。
健康会话随后仍通过求解。

解析链采用以下右端项：

```text
b = [9.75, 5.125, 14.25]
x = [1, 2, 3]
D = [479/108, 27/16, 19/4]
L[1,0] = 28/27
```

该证据不依赖候选参考公式。

## 复验与独立制备

```powershell
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
./scripts/test-windows-mass-solve.ps1
./scripts/test-windows-mass-solve.ps1 -Release
./scripts/test-windows-mass-solve.ps1 -AllFeatures
./scripts/test-windows-mass-solve.ps1 -AllFeatures -Release
./scripts/test-windows-transfers.ps1
./scripts/test-windows-mass-matrix.ps1 -AllFeatures
./scripts/test-windows-mass-matrix.ps1 -AllFeatures -Release
./scripts/test-windows-native.ps1 -AllFeatures -RestrictedRuntime
./scripts/test-windows-native.ps1 -AllFeatures -Release -RestrictedRuntime
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release -- --ignored --test-threads=1
```

GNU命令需要仓库私有MinGW。
GPU命令采用另装MSVC工具链。
Rust默认工具链仍保持GNU。
上述命令使用同一PowerShell进程。
原生脚本同时准备模拟DLL。
总GPU回归需要这些环境变量。
脚本先核对十九项输入哈希。
全feature也核对原生DLL。
报告记录真实HEAD与工作区。
报告记录平台、GPU与驱动。
NVRTC清单只代表目录文件。
它不证明实际加载路径。
本轮不提供清洁机器部署证据。

以下命令只用于独立制备。
它需要VS x64编译器环境。

```powershell
$m = (Resolve-Path target/toolchains/mujoco-3.12.0/package).Path
$env:PATH = "$m\bin;" + $env:PATH
New-Item -ItemType Directory -Force target/solve-reference | Out-Null
cl /nologo /EHsc /W4 /WX /std:c++17 /I"$m\include" fixtures/mass-solve/reference.cpp /Fo:target/solve-reference/reference.obj /Fe:target/solve-reference/reference.exe /link /LIBPATH:"$m\lib" mujoco.lib
target/solve-reference/reference.exe fixtures/native-probe/mixed-joints.mjb target/solve-reference/mixed-joints.json
target/solve-reference/reference.exe fixtures/native-probe/inertial-tree.mjb target/solve-reference/inertial-tree.json
target/solve-reference/reference.exe fixtures/kinematics/rotated-tree.mjb target/solve-reference/rotated-tree.json
target/solve-reference/reference.exe fixtures/native-probe/zero-dof.mjb target/solve-reference/zero-dof.json
target/solve-reference/reference.exe fixtures/com-position/massless-tree.mjb target/solve-reference/massless-tree.json
target/solve-reference/reference.exe fixtures/mass-matrix/armature-chain.mjb target/solve-reference/armature-chain.json
```

## 本轮验证

平台采用Windows x64。
系统版本为`10.0.26200.0`。
GPU采用RTX 4070 Ti SUPER。
驱动版本为596.36。
MSVC Rust版本为1.99.0。
NVRTC目录采用CUDA 12.8.1。
默认GNU工具链保持不变。

| 普通测试配置 | 调试宿主 | 发布宿主 | 每配置文档 |
| --- | ---: | ---: | ---: |
| 默认GNU | 138 | 138 | 6 |
| 原生MSVC | 144 | 144 | 6 |
| CUDA MSVC | 155 | 155 | 6 |
| 全feature MSVC | 165 | 165 | 6 |

普通测试仍忽略设备用例。
全feature显式测试各通过75项。
调试与发布均无失败或忽略。
四组clippy与格式检查均通过。
全feature文档构建拒绝警告。
该构建也通过检查。

| 求解配置 | 求解设备用例 | 原生世界比较 |
| --- | ---: | ---: |
| CUDA调试 | 5 | 1363 |
| CUDA发布 | 5 | 1363 |
| 全feature调试 | 6 | 1411 |
| 全feature发布 | 6 | 1411 |

每轮另通过两项分解边界测试。
每轮另通过一项f64 ABI测试。
每轮通过解析链与奇异拒绝检查。
四轮均沿用首次冻结容限。
四轮最大误差均保持一致：

```text
max absolute error:             4.9863528374771704e-5
max normalized residual:        1.2638390364138244e-8
max normalized reconstruction:  5.6782745647879574e-8
```

旧刚体脚本各通过四项测试。
它另检查三项内核适配。
每轮比较1347个原生世界。
旧质心脚本各通过六项测试。
每轮比较1347个原生世界。
旧矩阵脚本各通过六项测试。
每轮比较1379个原生世界。
三组脚本均通过调试与发布。
旧矩阵最大误差保持不变。
其值为`1.7808917590400597e-5`。
原有矩阵容限仍保持不变。
原生限制运行各通过22项。
仅驱动传输通过十项测试。
它包含原样f64位模式与世界复制。
该子进程看不到NVRTC。

独立生成器通过`/W4 /WX`。
重复制备匹配六组JSON哈希。
产品脚本核对十九项输入哈希。
Git暂存字节也匹配十九项哈希。
这些证据不来自零用例。

### 收尾修复

首次总回归漏设模拟DLL。
五项原生用例因此报错。
准备夹具后重跑全组并通过。
旧脚本仍期待两项内核适配。
新增ABI用例使实际计数变为三。
本轮同步脚本的准确计数。
首轮文档构建发现两个坏链接。
本轮给布局标记添加代码格式。
复验保留拒绝警告设置。

### 日志与版本证据

```text
target/mass-solve-probe/d668e846dd67474a851b62d956399cac/
target/mass-solve-probe/b6305d89dcf44391b0d48d0eae3dcc39/
target/mass-solve-probe/1b140d53d8eb482bb9d58de8aea2817f/
target/mass-solve-probe/069007f91df04c3aa171e425afd8b1cd/
target/transfer-probe/ff778036f5864f448c826a0add66e408/
target/solve-checks/
```

四份求解报告记录提交前工作区。
`workingTreeDirty`均为`true`。
它们记录以下真实基线：

```text
21ad9dd997808b933b437988e53bf3a310806d9c
```

## 架构收尾

架构现含28节点与22叶子。
架构登记196项契约与83条依赖。
旧193项契约与归属全部保留。
本轮新增三项求解辅助契约。
本轮只激活实际求解叶子。
七个产品模块继续保持计划。
完整物理核心也继续保持计划。
架构保留二十个计划态节点。
六个容器各有阅读布局。
本轮不修改十二条架构规则。
提交前检查取得零错误。
此时仍有一项临时警告。
它提示两项变更同时推进。
提交后关闭本轮辅助变更。
随后再核对警告与源码漂移。

完整G01、G02与G03仍待验收。
Linux继续暂缓。
