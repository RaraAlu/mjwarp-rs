# Windows刚体运动学探针

日期：2026-10-07。
范围：G01刚体运动学子集。
本轮只验收Windows。
本轮不冻结正式内核路线。
完整U018与U062仍待实现。

## 本轮源码

```text
src/physics/mod.rs                 输入预检与GPU刚体探针
src/runtime/transfer/kernel.rs     内部同步内核适配
tests/kinematics_probe.rs          固定参考与GPU回归
scripts/test-windows-kinematics.ps1  哈希、测试与报告
fixtures/kinematics/reference.cpp  独立参考制备程序
fixtures/kinematics/rotated-tree.xml
fixtures/kinematics/rotated-tree.mjb
fixtures/kinematics/{mixed-joints,inertial-tree,rotated-tree,zero-dof}.json
fixtures/kinematics/manifest.json
```

Rust提供主接口与检查。
CUDA C++只执行GPU公式。
本轮不新增第三方依赖。
默认构建不需要GPU工具链。
探针需要NVIDIA驱动与NVRTC。
该探针不使用既有PTX缓存。
它不具备仅驱动部署能力。
CubeCL两路仍保留原探针。
此内核不决定生产主路线。

## 接口与范围

```rust
pub fn probe_kinematics(
    session: &TransferSession,
    model: &InertialModelInput,
    worlds: usize,
    qpos: &[f32],
) -> Result<KinematicsOutput, TransferError>;
```

参数共享一份模型。
状态采用`[world,nq]`布局。
每个世界独立执行。
每次调用重新上传输入。
接口同步返回宿主结果。
体顺序遵循已检查父子拓扑。
每个线程串行遍历一个世界。
此调度不提供性能承诺。
`world()`提供只读结果切片。
它检查世界索引与字段边界。
结果保留自身宿主所有权。

| 输出 | 每世界标量数 | 语义 |
| --- | --- | --- |
| `xpos` | `3*nbody` | 世界体位置；米 |
| `xquat` | `4*nbody` | 世界体姿态；wxyz |
| `xmat` | `9*nbody` | 世界体旋转；按行展开 |
| `xipos` | `3*nbody` | 世界惯性质心；米 |
| `ximat` | `9*nbody` | 世界惯性旋转；按行展开 |
| `xanchor` | `3*njnt` | 世界关节锚点；米 |
| `xaxis` | `3*njnt` | 世界关节轴向量 |

关节覆盖free与ball。
关节覆盖slide与hinge。
free姿态直接读取当前状态。
ball姿态叠加局部旋转。
slide与hinge减去`qpos0`。
GPU归一化可用状态四元数。
偏心旋转保持锚点约束。
静态祖先照常传递体姿态。
惯性姿态叠加`body_iquat`。

算法依据[冻结Warp刚体源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py)。
数学约定沿用[冻结数学源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/math.py)。
许可记录见[源码说明](../THIRD_PARTY_NOTICES.md)。

### 严格探针边界

世界体保持零位置与单位姿态。
世界惯性系也保持该形式。
模型四元数须接近单位长度。
平方模长误差不能超过`2e-6`。
slide与hinge轴也采用该阈值。
状态四元数平方模长限于`[1e-12,1e12]`。
其余状态标量必须有限。
布局检查长度与索引容量。
接口允许零自由度模型。
接口拒绝零世界数量。

输入子集不包含mocap状态。
它不模拟休眠与柔性体。
它不更新geom、site或相机。
它不计算质心树或自由度映射。
它不计算质量矩阵与时间积分。
它不验证完整模型功能准入。
调用方明确选择上述子集语义。
严格探针不替代等价入口。
G01与MC系列仍需完整验收。

## 内存与失败边界

物理层不依赖io或render。
运行时不解释物理字段。
内部内核采用固定八参数ABI。
前三缓冲仅供内核读取。
第四缓冲承接输出。
后四参数均采用u32。
内部unsafe调用证明访问范围。
安全公开接口不接收任意代码。

适配器拒绝跨会话缓冲。
它拒绝空内核参数缓冲。
它复用会话操作锁与完成检查。
后端失败隔离整个会话。
完成不确定时保留资源所有者。
保留范围包含模块与设备缓冲。
编译失败不隔离健康传输。
测试只模拟完成不确定状态。
测试不注入真实GPU故障。

输出首尾各保留四个哨兵。
有效区预填NaN。
成功返回前检查全部输出。
越界与非有限值不发布结果。
输入拒绝也不发布部分结果。
接口不静默切换CPU。

## 独立参考与容限

参考采用官方MuJoCo 3.12.0。
它仍属于原生候选版本。
它不改变冻结Warp基线。
独立程序调用`mj_kinematics`。
原生程序不调用Rust算法。
产品构建与测试不运行该程序。
产品运行不调用CPU运动学。
生产测试只读取静态JSON。

原生语义依据[候选刚体实现](https://github.com/google-deepmind/mujoco/blob/3.12.0/src/engine/engine_core_smooth.c)。

| 样本 | 体数 | 关节数 | nq/nv | 覆盖 |
| --- | --- | --- | --- | --- |
| mixed-joints | 6 | 4 | 13/11 | 四关节与偏心hinge |
| inertial-tree | 8 | 6 | 15/13 | 静态祖先、分支与多关节 |
| rotated-tree | 7 | 5 | 14/12 | 非单位状态与旋转体/惯性系 |
| zero-dof | 1 | 0 | 0/0 | 世界体与空状态 |

四个模型各含八组固定状态。
种子固定为1789。
参考先量化状态到f32。
参考结果保留double精度。
批量规模覆盖1、255、256。
批量规模也覆盖257与513。

容限在首轮GPU执行前固定：

```text
abs(actual-reference) <= 2e-5 + 2e-5*abs(reference)
```

参考制备文件登记SHA-256。
测试脚本先核对所有哈希。
原生适配链也对照相同参考。
该链只提取已编译模型字段。
它不编译XML或计算CPU姿态。

### 参考制备命令

以下命令只用于静态制备。
先进入MSVC x64开发环境。
产品脚本不调用这些命令。

```powershell
$root=(Resolve-Path target/toolchains/mujoco-3.12.0/package).Path
cl.exe /nologo /std:c++17 /EHsc /W4 /WX /I"$root/include" fixtures/kinematics/reference.cpp /Fo:target/kinematics-checks/reference.obj /Fe:target/kinematics-checks/reference.exe /link /LIBPATH:"$root/lib" mujoco.lib
$env:PATH="$root/bin;"+$env:PATH
target/kinematics-checks/reference.exe --xml fixtures/kinematics/rotated-tree.xml fixtures/kinematics/rotated-tree.json fixtures/kinematics/rotated-tree.mjb
foreach($name in @('mixed-joints','inertial-tree','zero-dof')) {
    target/kinematics-checks/reference.exe --mjb "fixtures/native-probe/$name.mjb" "fixtures/kinematics/$name.json"
}
```

## Windows验证

平台采用Windows 11 x64。
系统构建号为26200。
GPU采用RTX 4070 Ti SUPER。
驱动版本为596.36。
Rust版本为1.99.0。
CUDA编译器采用NVRTC 12.8。
默认GNU工具链保持不变。

| 特性与配置 | 宿主测试 | 文档测试 | 本轮显式GPU测试 |
| --- | ---: | ---: | ---: |
| 默认GNU调试 | 126 | 6 | 不执行 |
| native-model-probe MSVC | 132 | 6 | 不执行 |
| cuda-probe MSVC | 140 | 6 | 调试/发布各5 |
| 全feature MSVC调试 | 150 | 6 | 6 |
| 全feature MSVC发布 | 150 | 6 | 6 |

全量显式回归各通过54项。
其中本轮新增六项。
两项检查内部内核适配。
四项检查GPU运动学。
调试与发布均取得相同结果。
后续求解增量新增f64 ABI用例。
当前脚本检查三项内核适配。
增量证据见[求解报告](windows-mass-solve-probe.md)。
旧原生测试各通过22项。
受限原生运行也各通过22项。
原生单feature烟测通过17项。
其受限运行也通过17项。
默认跳过不计入GPU通过量。

四样本共包含32组原生状态。
CUDA专用烟测比较1315个世界。
全feature烟测比较1347个世界。
两种配置均核对七项结果。
最大绝对误差为`4.584066739532489e-7`。
该误差不代表全域误差上界。
固定容限始终保持不变。
独立参考重复生成保持同哈希。
格式、clippy与rustdoc均通过。
clippy与rustdoc拒绝所有警告。
文档相对链接检查通过。

### 实际执行命令

MSVC命令显式选择工具链。
所有Cargo命令均采用`--locked`。

```powershell
cargo test --locked
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo +stable-x86_64-pc-windows-msvc test --locked --features native-model-probe
cargo +stable-x86_64-pc-windows-msvc test --locked --features cuda-probe
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release
cargo +stable-x86_64-pc-windows-msvc clippy --locked --features native-model-probe --all-targets -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --features cuda-probe --all-targets -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-features --all-targets -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-features --release --all-targets -- -D warnings
$env:RUSTDOCFLAGS='-D warnings'
cargo +stable-x86_64-pc-windows-msvc doc --locked --all-features --no-deps
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release -- --ignored --test-threads=1
scripts/test-windows-native.ps1 -AllFeatures -RestrictedRuntime
scripts/test-windows-native.ps1 -AllFeatures -Release -RestrictedRuntime
scripts/test-windows-native.ps1 -RestrictedRuntime
scripts/test-windows-kinematics.ps1
scripts/test-windows-kinematics.ps1 -Release
scripts/test-windows-kinematics.ps1 -AllFeatures
scripts/test-windows-kinematics.ps1 -AllFeatures -Release
```

全量回归先运行原生脚本。
该脚本提供可信DLL与替身。
构建环境沿用既有制备脚本。
检查日志保留在本机target。
`target/kinematics-checks/*.log`记录完整矩阵。

| 证据 | 本机报告目录 |
| --- | --- |
| CUDA调试 | `target/kinematics-probe/0bece1dddbed4de38139823ada1d1de4/` |
| CUDA发布 | `target/kinematics-probe/9fdcc3ccbd564feda26feca43a56be45/` |
| 全feature调试 | `target/kinematics-probe/1dcf9429259a47a5ba3f08f55bc459a0/` |
| 全feature发布 | `target/kinematics-probe/cce3cafb371e40b096deea1680f099a8/` |
| 原生调试与受限运行 | `target/native-probe/393736c4676441768cd35b2efe0738bc/` |
| 原生发布与受限运行 | `target/native-probe/6aa3278f28b84d0b8ded44a0b1c2464d/` |
| 原生单feature与受限运行 | `target/native-probe/9cec7f5c74a949bda827c3ccedfd4237/` |

报告记录GPU、驱动与源码状态。
报告也记录NVRTC库摘要。
摘要只列出私有目录库存。
它不认证实际动态加载来源。
上述矩阵来自提交前工作树。
每份报告明确记录dirty状态。

### 已纠正的尝试

初轮误用GNU工具链交叉编译。
该命令未执行测试。
改用已安装MSVC工具链后通过。
本轮也核对了实际线程配置。
适配器显式使用256线程。
它不依赖cudarc的1024默认值。

## 架构收尾

结构预检取得零错误。
本轮保留十二项原有规则。
本轮保留183项原有契约。
本轮新增三项公开辅助。
本轮新增一项内部ABI。
契约总数增至187项。
同步辅助拆成core与kernel。
源码路径对应这两项职责。
运行时层级细分至四层。
物理层保持完整计划状态。
历史计划变更也保持原范围。
本轮只激活已实现辅助叶子。
最终图状态以Normify回执为准。

## 下一步

补齐质心与自由度映射。
扩展mocap与其他运动学输出。
设计常驻设备结果与执行计划。
补齐正式模型准入与失败契约。
继续保留完整阶段计划状态。
Linux验收继续暂缓。
