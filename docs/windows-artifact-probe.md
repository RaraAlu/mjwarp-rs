# Windows产物缓存探针

日期：2026-10-06。
范围：P1局部运行时原型。
本轮只开展Windows工作。
本报告不声明物理验收。
本报告不冻结编译主路线。

## 实现文件

```text
src/
  main.rs                    缓存CLI与显式信任
  diagnostics/mod.rs         产物与编译库诊断
  runtime/
    mod.rs                   P1入口与结果
    cache.rs                 格式、键、摘要与文件交换
    cuda.rs                  共享驱动执行器
    cuda/artifacts.rs        编译、加载与固定ABI适配
tests/artifact_probe.rs      实际GPU产物复用
scripts/test-windows-deployment.ps1
                             独立受限进程烟测
```

新文件承担真实行为。
本轮不铺设其他模块空壳。
运行时只依赖诊断模块。
物理与渲染源码仍未实现。

## 缓存契约

产物采用严格JSON结构。
serde拒绝未知与重复字段。
sha2计算SHA256摘要。
文件名采用完整键摘要。
内容包含键、阶段与摘要。
摘要覆盖规范化载荷。
摘要包含PTX及入口声明。

缓存键记录以下信息：

| 字段 | 当前内容 |
| --- | --- |
| 格式 | schema 1 |
| 编译来源 | 明确路线与内核名 |
| 源码及依赖 | 源码、Cargo.toml与Cargo.lock摘要 |
| 配方 | P1版本、Checked、U32、128线程 |
| C++ | C++17、禁用FMA与快速数学、NVRTC12.8 |
| LLVM | 驱动选择PTX版本，禁用grid constants |
| 原生 | 内嵌PTX6.0、sm_70 |
| 设备目标 | SM主次版本、驱动API版本 |
| 宿主 | OS与CPU架构 |

源码摘要包含以下文件：

```text
src/runtime/mod.rs
src/runtime/cubecl.rs
src/runtime/cuda.rs
src/runtime/cuda/artifacts.rs
src/runtime/probe.ptx
src/runtime/cache.rs
Cargo.toml
Cargo.lock
```

源码变化会保守失效缓存。
元素数与重放数不进入键。
同键产物可复用不同规模。
设备索引不进入编译目标键。
相同SM及驱动API可共用产物。
程序不自动清除旧键文件。

原生路线只支持affine。
CubeCL两路各支持八项内核。
全局扫描固定保存三个阶段。
阶段包含主扫描、总和与偏移。
程序检查阶段数、顺序与名称。
程序检查声明ABI与线程数。
程序检查共享区声明上限。

| 边界 | 拒绝条件 |
| --- | --- |
| 文件 | 超过8MiB |
| 单阶段PTX | 空内容、超过2MiB或包含NUL |
| 阶段 | 缺失、重复、乱序或未知入口 |
| ABI声明 | 原生三参数与CubeCL元数据不匹配 |
| 线程 | 非128线程 |
| 共享区 | 超过128KiB；原生声明非零 |
| 完整性 | SHA256不匹配 |
| 键 | 源码、配方、平台或设备不匹配 |

文件读取采用有界缓冲。
程序也检查读取后的长度。
写入先创建同目录临时文件。
程序同步临时文件再执行替换。
程序不先删除旧产物。
失败清理只删除临时文件。
此流程不承诺断电持久性。
同键并发写入保留完整产物。
本原型不提供跨进程单次编译。

## 来源与安全边界

摘要只检查完整性。
摘要不认证GPU代码来源。
攻击者可以重算自带摘要。
声明ABI不证明真实PTX ABI。
本原型不解析任意PTX语义。
本原型不沙箱化GPU代码。

`run_cached_probe`要求unsafe。
调用方须信任目录与全部PTX。
调用方须保证固定内核ABI。
调用方须防止产物篡改。
内核不得越界或制造数据竞争。
CLI要求显式`--trust-cache`。
不要对外部下载产物使用此项。
GPU测试只消费自产临时产物。

## 命令行为

| 命令 | 行为 |
| --- | --- |
| `cache-build` | 命中时不调用编译器；缺失时编译指定路线 |
| `cache-build --refresh` | 显式重建并替换；失败保留旧产物 |
| `cache-run` | 只读取已有产物；绝不编译或换路 |
| `--require-no-nvrtc` | 真实探测NVRTC；发现库时明确失败 |

损坏缓存不会触发自动重建。
缺失缓存运行明确报告missing。
关闭前端后可消费其PTX。
缓存运行仍报告原始路线。
普通`probe`仍检查路线特性。
程序不静默回退原生或CPU。
图捕获前完成产物加载与预热。
运行复用原有缓冲与事件检查。
运行复用真实图节点更新。
执行器先释放图，再释放缓冲。
模块随函数所有者保持存活。

```powershell
# 以下命令沿用准备好的私有环境。
# C++构建需要NVRTC12.8与头文件。
cargo +stable-x86_64-pc-windows-msvc run --locked --features cubecl-cpp-probe -- cache-build --cache target/probe-cache --backend cubecl-cpp --kernel global-scan

# 仅驱动构建可以消费可信产物。
cargo +stable-x86_64-pc-windows-msvc run --locked --features cuda-probe -- cache-run --cache target/probe-cache --backend cubecl-cpp --kernel global-scan --elements 16385 --trust-cache

# 脚本独立启动受限PATH子进程。
pwsh -NoProfile -File scripts/test-windows-deployment.ps1
```

NVRTC探测使用当前进程加载器。
已加载NVRTC时检查也会失败。
因此脚本创建新的消费进程。
普通开发终端不证明无NVRTC。

## Windows部署烟测

脚本构建两个发行程序。
生产程序启用全部探针特性。
消费程序只启用cuda-probe。
两个构建使用不同产物目录。
脚本先复制exe再启动验证。
脚本不改Rust默认工具链。
脚本不安装或删除开发组件。
每次烟测保留独立证据目录。

消费子进程采用以下约束：

- 工作目录只包含消费exe。
- PATH只含Windows系统目录。
- 清除CUDA开发目录环境变量。
- 清除LLVM与Tracel环境变量。
- 设置`CUDA_CACHE_DISABLE=1`。
- 检查NVRTC加载器确实不可见。
- 独立记录退出码、输出与错误。
- 超时后终止子进程树。

脚本也检查依赖树与PE导入。
消费依赖树不含CubeCL前端。
依赖树也不含pliron与LLVM。
PE导入不含NVRTC及cudart。
PE导入仍含Windows系统API。
PE导入仍含VC与UCRT运行库。
本机提供VCRUNTIME140.dll。
清洁部署须另外准备该运行库。

消费仍动态加载NVIDIA驱动。
PTX仍需驱动执行JIT编译。
此缓存不是驱动二进制缓存。
驱动JIT组件也属于部署依赖。
[NVIDIA驱动文档](https://docs.nvidia.com/cuda/cuda-driver-api/cuda_driver_api/group__CUDA__MODULE.html)说明PTX加载与JIT错误。

本机仍安装MSVC及SDK。
本机仍保存私有CUDA组件。
受限进程不等于独立清洁机器。
系统加载器仍能访问系统库。
本报告只证明受限进程烟测。
本报告不证明完整部署准入。

## 依赖与工具链

| 项目 | 本轮记录 |
| --- | --- |
| OS | Windows NT 10.0.26200.0，x64 |
| GPU | RTX 4070 Ti SUPER；SM8.9 |
| NVIDIA驱动 | 596.36；CUDA驱动API13020 |
| Rust | 1.99.0；GNU默认、MSVC显式调用 |
| MSVC | 14.44.35207；PE工具14.44.35229.0 |
| Windows SDK | 10.0.26100.0 |
| cudarc | 0.18.2，维持冻结版本 |
| CubeCL | `1f73b9f63de50a17398c1d5278e2a5f11612c7e1` |
| LLVM bundle | `tracel-llvm-23.1.0-3` |
| NVRTC | 私有组件12.8.93；运行时检查12.8 |
| CUDA头文件与CCCL | 私有组件12.8.90 |
| serde | 1.0.229；MIT OR Apache-2.0 |
| serde_json | 1.0.151；MIT OR Apache-2.0 |
| sha2 | 0.10.9；MIT OR Apache-2.0 |

新增三项依赖均采用可选特性。
默认构建不加载这些依赖。
锁文件只新增根包依赖引用。
本轮不升级已有锁定版本。
许可记录来自锁定包元数据。
正式分发仍须整理许可声明。
本轮未改变冻结上游范围。

缓存键尚有以下限制：

- NVRTC检查只覆盖主次版本。
- 编译器补丁摘要未进入键。
- 外部头文件内容未进入键。
- LLVM原生bundle摘要未进入键。
- 工具替换仍需显式刷新。
- 烟测报告另记NVRTC DLL摘要。
- 主路线与发行包仍待冻结。

## 已执行验证

默认宿主测试48项通过。
GNU C++宿主测试65项通过。
MSVC全特性宿主66项通过。
MSVC LLVM隔离宿主65项通过。
MSVC CUDA隔离宿主62项通过。
缓存单元测试包含13项。
CLI单元测试包含6项。

MSVC调试GPU集成12项通过。
其中缓存4项、旧回归8项。
三路缓存覆盖157组配置。
缓存共499轮重放。
缓存共611次节点更新。
另验缺失及损坏产物诊断。
GNU C++缓存GPU集成3项通过。
MSVC LLVM隔离缓存3项通过。
MSVC发行宿主66项通过。
MSVC发行GPU集成12项通过。
格式与五种Clippy组合均通过。
全特性rustdoc拒绝警告并通过。
PowerShell脚本语法检查通过。
默认宿主检查不执行GPU。
零GPU用例不表示物理验收。

受限进程烟测实际通过。
运行证据目录如下：

```text
C:/Rust/mjwarp-rs/target/deployment-probe/3931d8db16e84694a7e5640de04d2555/
  report.json
  consumer-dependencies.txt
  consumer-imports.txt
  producer/mjwarp-rs.exe
  driver/mjwarp-rs.exe
  cache/                     17份JSON、21个PTX阶段
  *.stdout.txt
  *.stderr.txt
```

报告记录107次独立子进程。
生产程序生成17份产物。
消费程序验证17次构建命中。
消费程序执行68组GPU配置。
配置规模为1、129、16385及上限。
消费共204轮图重放。
消费共240次节点更新。
各次GPU运行均确认NVRTC缺失。
另外五次预期失败均符合诊断。
失败覆盖关闭前端、缺失、损坏。
失败也覆盖可见NVRTC检查。

首次脚本命中PS自动变量冲突。
本轮改名后完整重跑通过。
旧失败报告继续保留false状态。
本轮不运行Python审计工具。
文档检查只覆盖本次变更。
检查范围包含四份文档。
路径核对覆盖八个实现文件。
本地链接43项检查通过。
全仓文档审计仍待开展。

本轮实际执行以下命令：

```powershell
# 默认GNU；按脚本导入私有GNU工具。
cargo test --locked --offline
cargo clippy --locked --offline --all-targets -- -D warnings

# GNU C++隔离；先设置CUDA_PATH及bin。
cargo +stable-x86_64-pc-windows-gnu test --locked --offline --features cubecl-cpp-probe
cargo +stable-x86_64-pc-windows-gnu test --locked --offline --features cubecl-cpp-probe --test artifact_probe -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-gnu clippy --locked --offline --features cubecl-cpp-probe --all-targets -- -D warnings

# MSVC命令先导入VsDevCmd环境。
cargo +stable-x86_64-pc-windows-msvc test --locked --offline --all-features
cargo +stable-x86_64-pc-windows-msvc test --locked --offline --all-features --tests -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc clippy --locked --offline --all-features --all-targets -- -D warnings
cargo +stable-x86_64-pc-windows-msvc test --locked --offline --release --all-features
cargo +stable-x86_64-pc-windows-msvc test --locked --offline --release --all-features --tests -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc test --locked --offline --features cubecl-llvm-probe
cargo +stable-x86_64-pc-windows-msvc test --locked --offline --features cubecl-llvm-probe --test artifact_probe -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc clippy --locked --offline --features cubecl-llvm-probe --all-targets -- -D warnings
cargo +stable-x86_64-pc-windows-msvc test --locked --offline --features cuda-probe
cargo +stable-x86_64-pc-windows-msvc clippy --locked --offline --features cuda-probe --all-targets -- -D warnings
$env:RUSTDOCFLAGS = '-D warnings'
cargo +stable-x86_64-pc-windows-msvc doc --locked --offline --all-features --no-deps

pwsh -NoProfile -File scripts/test-windows-deployment.ps1
cargo fmt --check
git diff --check
```

Clippy实际覆盖五种组合。
默认、C++、LLVM、CUDA及全特性。

## Normify证据

变更ID如下：

```text
2026-10-06-windows-artifact-cache
```

运行时新增四个源码映射。
运行时当前映射19个文件。
根模块记录可选缓存依赖。
诊断模块记录产物失败类别。
工具刷新实际源码指纹。
工具校验取得零错误。
工具保留六项既有警告。
一项提示运行时叶子过粗。
五项提示计划源码尚未落地。
八个架构节点继续保持计划。
目标契约及冻结上游保持不变。
本轮不激活产品模块。

## 下一批Windows工作

- 补独立清洁部署机器证据。
- 收口完整编译工具指纹。
- 收口外部资源ABI与租约。
- 决议已编译模型的输入ABI。
- 明确设备故障恢复边界。
- 补最小无窗口图像输出。

P0模型与数值契约仍待收口。
P1完整准入仍未完成。
Linux工作继续暂缓。
正式准入仍要求双平台。
