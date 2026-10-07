# Windows原生模型输入探针

日期：2026-10-07。
范围：可信已编译输入。
本轮继续只做Windows。
本轮不冻结正式GPU路线。
本轮不冻结生产原生ABI。

本报告保留四字段阶段证据。
后续已新增十二项运动学字段。
新增证据见[运动学字段报告](windows-kinematic-fields.md)。
下文数量只描述四字段阶段。

## 本轮实现

Rust接口已有原生输入探针。
C++桥接拥有模型与DLL。
入口先检查版本与必需符号。
入口只读取已编译MJB。
快照独占四项代表字段。
原生释放后快照仍有效。
GPU传输读取该独占快照。
GPU执行真实上传与回读。
本轮不提供完整put_model。
本轮不提供完整ModelInput。
GPU物理与渲染仍待实现。

```text
C:/Rust/mjwarp-rs/
  build.rs
  include/mjwarp_native_probe.h
  native/model_probe.cpp
  src/model/mod.rs
  src/io/mod.rs
  src/diagnostics/mod.rs
  tests/native_model_probe.rs
  tests/native/mock_mujoco.cpp
  fixtures/native-probe/
  scripts/prepare-native-probe.ps1
  scripts/test-windows-native.ps1
```

## 候选依赖

冻结上游要求MuJoCo至少3.12。
本轮选取3.12.0作探针候选。
它不证明全部字段兼容。
LRsLab尚需共同冻结正式版本。
依据：[冻结上游依赖](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/pyproject.toml)。

| 项目 | 本轮值 |
| --- | --- |
| 原生版本 | MuJoCo 3.12.0 |
| 原生提交 | `13827e9ee56f097f57acf69ae52b078f9839682d` |
| 头部与DLL版本值 | `3012000` |
| 官方平台包 | `mujoco-3.12.0-windows-x86_64.zip` |
| 包SHA256 | `ffe071c2747dd9513a1c59e7d2428bb678d887f9edec9eb9674b4288a248a8e9` |
| 可选构建依赖 | `cc = 1.6.0` |
| cc许可 | MIT或Apache-2.0 |
| 原生许可 | Apache-2.0；原包保留第三方说明 |

cc元数据与锁文件校验一致。
依据：[cc包元数据](https://crates.io/crates/cc/1.6.0)。

来源：[官方发行页](https://github.com/google-deepmind/mujoco/releases/tag/3.12.0)。
本项目不提交官方DLL或头文件。
脚本只下载并核验官方包。
脚本保留包内全部许可。
源码只调用三个原生符号。
代码不复制上游模型结构体。

默认构建无需原生头文件。
默认构建不调用C++编译器。
默认GNU工具链保持不变。
原生探针暂限Windows MSVC。
架构还要求x86_64。
本机未安装GNU C++编译器。
首次GNU原生试编明确失败。
本轮不引入跨编译器混链。
MSVC初次链接出现CRT警告。
代码随后改用动态CRT。
复验不再出现该警告。

## ABI与字段边界

Rust只声明96字节DTO。
C++检查大小、对齐与偏移。
Rust另作同样的编译期检查。
原生头文件另检查模型ABI。
原生计数采用有符号64位。
索引数组仍采用有符号32位。
原生浮点采用64位double。
指针采用64位。
依据：[原生类型声明](https://github.com/google-deepmind/mujoco/blob/3.12.0/include/mujoco/mjtype.h)。

| DTO内容 | 含义 |
| --- | --- |
| `schema` | 探针DTO版本；当前1 |
| `native_version` | 头部与DLL共同的版本值 |
| `*_bytes` | 指针、浮点、索引与计数字节 |
| `native_model_bytes` | 编译时原生结构大小；仅作证据 |
| `nq,nv,nu,na` | 坐标、速度、控制输入与激活计数 |
| `nbody,njnt,ngeom` | 体、关节与几何计数 |
| `nsensordata` | 传感输出计数 |

DTO检查拒绝负数与容量溢出。
它要求至少存在世界体。
它允许零自由度与零关节。
nu不表示执行器数量。
DTO不验证全部字段语义。

快照包含以下字段：

| 字段 | 原生类型 | 长度与语义 |
| --- | --- | --- |
| `qpos0` | double | nq；关节原生单位 |
| `body_mass` | double | nbody；质量kg |
| `body_parentid` | int32 | nbody；父体索引，包含世界体 |
| `jnt_type` | int32 | njnt；原生枚举值 |

字段依据：[原生模型声明](https://github.com/google-deepmind/mujoco/blob/3.12.0/include/mujoco/mjmodel.h)。
GPU辅助严格转换double为f32。
索引与枚举采用i32上传。
本轮不实现共享或周期参数批。
本轮不推导完整GPU模型布局。
本轮不生成完整模型身份。

## 所有权与失败边界

`load_trusted`明确使用unsafe。
调用者须信任DLL及其依赖。
调用者须保证原生解析安全。
版本检查不隔离恶意DLL。
头部检查不隔离恶意MJB。
原生库可能直接终止进程。
本轮不承诺全部失败路径无泄漏。
原生解析含分配与错误处理。
依据：[原生MJB加载实现](https://github.com/google-deepmind/mujoco/blob/3.12.0/src/engine/engine_io.c)。

Rust入口拒绝相对DLL路径。
Rust入口拒绝路径内NUL。
桥接限制依赖DLL搜索目录。
它只搜索DLL目录与系统目录。
桥接先检查版本，再检查符号。
必需符号齐备后才调用加载。
原生所有者独占模型及DLL。
所有者不提供Clone、Send或Sync。
Drop先删除模型，再卸载DLL。
DTO验证失败也执行该顺序。
字段复制先检查全部长度。
C++另检查容量与字段指针。
输入错误不会部分改写目标。
空字段不触发原生解引用。
Rust快照不保留任何原生指针。

## 静态样本边界

本项目另行制备固定MJB。
该过程独立于产品工具链。
本轮只运行两次官方compile。
样本目录保留自行编写的XML。
manifest记录来源、版本与哈希。
测试脚本先核验全部样本。
构建、测试与运行不启动compile。
产品链不引入Python。
LRsLab仍负责MJCF编译。
详见[样本来源](../fixtures/native-probe/README.md)。

## 执行方式

先安装MSVC与Windows SDK。
这些命令显式选择MSVC。
脚本不修改默认Rust工具链。
请使用独立PowerShell进程。
脚本仅调整该进程环境。

```powershell
pwsh -NoProfile -File scripts/prepare-native-probe.ps1
pwsh -NoProfile -File scripts/test-windows-native.ps1
pwsh -NoProfile -File scripts/test-windows-native.ps1 -Gpu
pwsh -NoProfile -File scripts/test-windows-native.ps1 -AllFeatures -RestrictedRuntime
pwsh -NoProfile -File scripts/test-windows-native.ps1 -AllFeatures -Release -RestrictedRuntime
```

全feature需要已有CUDA工具包。
它复用此前双路探针依赖。
Gpu只要求NVIDIA驱动。
受限运行先完成构建。
它再隐藏CUDA工具目录。
它也隐藏MSVC、Python与compile。
DLL路径仍指向可信原生包。
它不模拟洁净系统安装。
它不证明发行部署已经完成。

## 验证证据

本机继续使用RTX 4070 Ti SUPER。
系统采用Windows 11专业版。
系统版本为10.0.26200。
驱动版本保持596.36。
Rust继续采用1.99。
MSVC与SDK沿用已验收安装。

八项显式测试覆盖如下：

1. 代表字段及释放后快照。
2. 零自由度与空字段。
3. 四项目标长度错误。
4. 错误版本及三种缺失符号。
5. 缺失DLL及安全头部反例。
6. 删除模型与卸载DLL顺序。
7. DTO验证失败后的资源释放。
8. 原生释放后的四字段GPU回读。

七项宿主测试不需要GPU。
第八项要求真实GPU。
头部截断测试输出原生警告。
该警告属于预期反例。
它不表示架构或编译警告。
原生库还生成诊断日志。
Git忽略`MUJOCO_LOG.TXT`。

本轮实际测试结果如下：

| 矩阵 | 通过 | 失败 | 备注 |
| --- | --- | --- | --- |
| 默认GNU宿主 | 103 | 0 | 另有2项文档测试；无原生环境变量 |
| MSVC原生单feature宿主 | 106 | 0 | 另有2项文档测试；7项原生默认忽略 |
| MSVC全feature宿主debug | 123 | 0 | 另有2项文档测试；34项显式测试默认忽略 |
| MSVC全feature宿主release | 123 | 0 | 另有2项文档测试；34项显式测试默认忽略 |
| 全feature显式debug | 34 | 0 | 原有26项，加7项原生宿主与1项GPU |
| 全feature显式release | 34 | 0 | 同上；不将忽略项计入通过 |
| 原生单feature显式 | 7 | 0 | 不要求GPU |
| 原生加CUDA显式 | 8 | 0 | 四字段真实GPU上传及回读 |
| 全feature受限debug | 8 | 0 | 原生测试程序；无开发工具PATH |
| 全feature受限release | 8 | 0 | 同上；当前主机提供运行库 |

格式、三组clippy及文档通过。
三组clippy覆盖以下组合：

- 默认GNU。
- MSVC原生单feature。
- MSVC全feature。

实际命令另含以下项目：

```powershell
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo +stable-x86_64-pc-windows-msvc test --locked --features native-model-probe
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-targets --features native-model-probe -- -D warnings
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --tests -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release --tests -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-targets --all-features -- -D warnings
cargo +stable-x86_64-pc-windows-msvc doc --locked --all-features --no-deps
cargo fmt --check
```

上述MSVC命令先初始化工具环境。
环境还设置已核验的原生包路径。
全feature另设置已有CUDA路径。
脚本自动完成这些局部设置。

日志与JSON保留在：

```text
target/native-probe/<run-id>/tests.log
target/native-probe/<run-id>/restricted.log
target/native-probe/<run-id>/report.json
```

报告记录当时HEAD及工作区状态。
预提交日志验证当时工作区代码。
它不把基线HEAD当作新代码提交。

本轮保留以下受限证据：

| 运行 | run-id |
| --- | --- |
| 全feature debug最终复验 | `c3b6af2dd5a146dfb29ad87cf592a100` |
| 全feature release最终复验 | `04a03ae8b8124bbab4ea37ceeb9270c3` |
| 原生单feature | `285c8ed2e5e24055b0b4256673077400` |

全量日志另存于：

```text
target/native-probe/default-tests.log
target/native-probe/native-host-tests.log
target/native-probe/all-host-tests.log
target/native-probe/all-host-release.log
target/native-probe/all-gpu-debug.log
target/native-probe/all-gpu-release.log
```

## 部署依赖检查

本轮实际运行dumpbin。
原生DLL依赖以下运行库：

- `MSVCP140.dll`。
- `MSVCP140_ATOMIC_WAIT.dll`。
- `VCRUNTIME140.dll`。
- `VCRUNTIME140_1.dll`。
- Windows UCRT与系统DLL。

当前主机提供这些DLL。
受限PATH不移除系统运行库。
探针程序没有静态导入mujoco.dll。
桥接按显式路径动态加载它。
本轮不冻结再分发版本与安装方案。
依赖日志位于：

```text
target/native-probe/native-dll-dependencies.log
target/native-probe/test-exe-dependencies.log
```

## 架构收口

本轮保留16个架构节点。
四项新增辅助契约已有源码。
API契约总数现为174。
全部旧170项契约继续保留。
原141项计划契约也继续保留。
三项新依赖均遵守模块方向。
架构依赖总数现为35。
根层登记跨语言构建与测试。
IO保持原生准入与所有者职责。
model保持DTO与容量职责。
diagnostics保持错误职责。
七个产品顶层模块仍为计划态。
传输辅助叶子继续保持活跃态。
图校验取得零错误与零警告。
这些数量不证明完整物理等价。

## 后续仍需落实

- 与LRsLab冻结原生依赖。
- 扩展完整字段与资产转换。
- 收口模型身份与回调租约。
- 设计完整GPU Model与Data。
- 建立物理轨迹独立参考。
- 验收正式Windows部署。
- 用户启用后再验收Linux。

MC01与完整模型验收仍未完成。
本轮探针不改变产品完成状态。
