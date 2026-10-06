# CubeCL双路探针

日期：2026-10-06。
本轮只验证P1局部能力。
本轮不冻结引擎主路线。

## 当前结果

| 路线 | 实现 | Windows状态 |
| --- | --- | --- |
| 原生PTX | 手写PTX，驱动JIT | GPU回归通过 |
| CubeCL C++ | 同一Rust内核，C++，NVRTC | 六种长度、最大元素与3轮图重放通过 |
| CubeCL LLVM | 同一Rust内核，pliron，LLVM | MSVC构建通过；六种长度、最大元素与3轮图重放通过 |

Linux仍等待GitHub推送。
本轮不推送项目。
物理与渲染仍未实现。

## 编译与运行边界

Cargo固定CubeCL提交。
Cargo同时固定pliron提交。
发行crate采用不同快照。
本轮不混用两个来源。

```text
CubeCL: 1f73b9f63de50a17398c1d5278e2a5f11612c7e1
pliron: 81155d96b8797d149de1d88219c431a68a75ef33
tracel-llvm-bundler: 23.1.0-3
llvm-sys: 231.0.0
```

两路调用同一个`affine`定义。
Rust内核计算`y = 2x + 1`。
两路显式构造目标编译器。
它们不借用默认后端选择。
C++特性不引入LLVM编译器。
LLVM特性只启用NVPTX目标。
默认构建不启用这两项。
程序拒绝`auto`与`cpu`路线。
缺少特性时报告对应名称。
程序不静默更换编译路线。

| 参数 | 固定契约 |
| --- | --- |
| `--backend` | `native-ptx`、`cubecl-cpp`、`cubecl-llvm` |
| 两路Rust定义 | `src/runtime/cubecl.rs::affine` |
| C++编译器 | `CppCompiler<Cuda>` |
| LLVM编译器 | `PlironCompiler`，`LlvmTarget::Nvptx` |
| 地址类型 | `AddressType::U32` |
| 线程块 | 128 × 1 × 1 |
| 执行模式 | `ExecutionMode::Checked` |
| 元数据 | 两个连续u32长度，共8字节 |
| 内核参数 | 输入指针、输出指针、元数据指针 |
| Grid constant | 两路均禁用 |
| NVRTC选项 | C++17，当前GPU的`compute_SM`，禁用FMA融合与fast math |
| LLVM选项 | 当前GPU的SM，按驱动API选择PTX版本 |
| 输出尾部 | 16个守卫元素 |

该ABI只用于这个探针。
它不冻结模型设备布局。
探针不创建完整设备能力表。
探针不启用自动调优。
探针不验收CubeCL运行时。

驱动适配沿用既有资源管理。
两路先编译，再加载与预热。
适配随后捕获生成的内核。
捕获期间不编译或分配。
每轮重放更换输入内容。
两路保留元数据至图释放。
错误路径也等待执行完成。
图重放不证明节点参数更新。
图重放不关闭外部租约契约。

## Windows私有工具

脚本只改子进程环境。
脚本不改全局PATH。
本轮保留GNU默认工具链。
本轮另安装MSVC Rust工具链。
本轮不启动Python。

```powershell
pwsh -NoProfile -File scripts/prepare-cuda-probe.ps1
pwsh -NoProfile -File scripts/prepare-gnu-probe.ps1
pwsh -NoProfile -File scripts/run-cubecl-probe.ps1 -Backend cpp
pwsh -NoProfile -File scripts/run-cubecl-probe.ps1 -Backend cpp -GpuTests
pwsh -NoProfile -File scripts/run-cubecl-probe.ps1 -Backend cpp -Elements 1048576
```

CUDA脚本采用官方清单。
脚本校验各包的SHA256。
脚本保留原始许可文件。
脚本不调用下载的`nvcc`。
NVCC包提供头文件与libdevice。
清单记录在私有工具目录。

```text
target/toolchains/cuda-12.8.1/components.json
NVRTC: 12.8.93
CUDA runtime headers: 12.8.90
CCCL: 12.8.90
NVCC headers/libdevice: 12.8.93
```

依据：[NVIDIA组件清单](https://developer.download.nvidia.com/compute/cuda/redist/redistrib_12.8.1.json)。

GNU原环境缺少`dlltool`。
CubeCL依赖因此无法完整构建。
脚本补齐私有binutils工具。
它同时安装工具所需动态库。
脚本固定包版本与SHA256。
工具不链接或分发进引擎。

本轮另发现GNU链接缺口。
链接器丢弃跨crate注册项。
内核展开因此触发聚合异常。
保留注册后，257元素探针通过。
项目配置固定`link-dead-code=yes`。
回归测试检查切片接口注册。
该选项可能增加发行体积。
正式部署仍须评估链接方案。

依据：[MSYS2 binutils](https://packages.msys2.org/packages/mingw-w64-x86_64-binutils)、[linkme注册机制](https://github.com/dtolnay/linkme)。

## MSVC与LLVM复验

用户完成C++生成工具安装。
本轮核对VS组件记录。
本轮导入x64开发环境。
原脚本错误包裹路径引号。
脚本改用`call`命令。
脚本正确读取含空格路径。
本轮编译并链接C++程序。
程序调用Windows API。
程序返回成功并报告64位。
本轮不改Rust默认目标。

```text
VS Build Tools 2022: 17.14.41
VC tools directory: 14.44.35207
cl.exe: 19.44.35229.0
Windows SDK: 10.0.26100.0
installed Rust target: stable-x86_64-pc-windows-msvc
default Rust target: stable-x86_64-pc-windows-gnu
C++ smoke: target/msvc-check/smoke.cpp
C++ API: GetCurrentProcessId
C++ flags: /W4 /WX /EHsc
```

上轮缺少`link.exe`。
用户安装后解除该阻塞。
本轮实际编译LLVM路线。
257元素探针首先通过。
六种长度随后通过GPU回归。
最大元素探针同样通过。
两路共用MSVC构建也通过。

bundler提供独立LLVM包。
它不借用Rust内部LLVM。
本轮另核对下载包摘要。
本轮不伪造安装标记。

```text
LLVM: 23.1.0
bundle: 23.1.0-3, windows-x64
cache: C:/Users/zhang/AppData/Local/tracel/tracel-llvm-23.1.0-3
archive_sha256: d0a548eec377613e56aa61c500c5dee7c469d49e20b8ea076ee2441ea1c435ee
sidecar_content_sha256: 2bb32dedb99de5a764e056dfa3c98d6d60368aa3dc468171e009d9cd2b8fdc73
```

bundler核对包与内容摘要。
本轮独立复验下载包摘要。
SDK安装不证明清洁部署。
主路线仍等待双平台验收。

```powershell
pwsh -NoProfile -File scripts/run-cubecl-probe.ps1 -Backend llvm
pwsh -NoProfile -File scripts/run-cubecl-probe.ps1 -Backend llvm -GpuTests
```

## 依赖与许可

| 依赖 | 版本或来源 | 声明许可 |
| --- | --- | --- |
| CubeCL各crate | 冻结Git提交；0.11.0-pre.4 | MIT OR Apache-2.0 |
| pliron与pliron-llvm | 冻结Git提交；0.18.0 | Apache-2.0 |
| tracel-llvm-bundler | 23.1.0-3 | MIT OR Apache-2.0 |
| llvm-sys | 231.0.0 | MIT |
| LLVM包 | 23.1.0-3，Windows x64 | 包内Support通知声明Apache-2.0 WITH LLVM-exception；完整通知仍待整理 |
| cudarc | 0.18.2 | MIT OR Apache-2.0 |
| libloading | 0.8.8 | ISC |
| buildid、option-ext | 1.0.5、0.2.0 | MPL-2.0 |
| CUDA组件 | 官方12.8.1清单 | CUDA Toolkit许可；CCCL另含开源通知 |
| GNU构建工具 | binutils 2.47-3及配套包 | GPL/LGPL等；不链接产品 |
| VS Build Tools | 2022，17.14.41；编译与链接通过 | 微软许可；不分发安装器 |

本轮核对Cargo许可字段。
锁文件固定全部Rust依赖。
新增依赖含MPL文件级义务。
本轮没有改写这些依赖源码。
LLVM包只附局部Support通知。
发布前需补齐完整许可通知。
本项目自身许可仍待用户确定。
元数据清单不等于法务验收。

## 未完成项

- 两路原子与共享内存测试。
- 两路生成产物与缓存核对。
- 无完整SDK的部署测试。
- Linux同组GPU测试。
- 全部模型与数值契约。

本轮只取得Windows局部证据。
P1完整准入仍未完成。
所有Normify模块仍保持计划态。

## 已执行检查

本机采用RTX 4070 Ti SUPER。
设备报告SM 8.9。
驱动API报告13020。
本轮沿用驱动596.36。
GNU与MSVC Rust均为1.99.0。

### 上轮检查

以下保留安装前的证据。
LLVM失败只描述当时环境。

| 检查 | 实际结果 |
| --- | --- |
| `cargo test --locked` | 13通过；默认不执行GPU |
| `cargo test --locked --features cubecl-cpp-probe` | 15通过；3项GPU测试默认忽略 |
| `run-cubecl-probe.ps1 -Backend cpp -GpuTests` | 1通过；覆盖1、127、128、129、257、4097元素 |
| `run-cubecl-probe.ps1 -Backend cpp -Elements 1048576` | 精确结果、尾部守卫与3轮重放通过 |
| 原生GPU测试 | 2通过；覆盖六种长度与错误设备 |
| `cargo fmt --check` | 通过 |
| `cargo clippy --locked --all-targets --features cubecl-cpp-probe -- -D warnings` | 通过 |
| `cargo doc --locked --features cubecl-cpp-probe --no-deps` | 通过 |
| PowerShell脚本解析 | 3项通过 |
| `cargo +stable-x86_64-pc-windows-msvc check --locked --features cubecl-llvm-probe` | 失败；缺少`link.exe`，尚未检查本项目LLVM代码 |
| `run-cubecl-probe.ps1 -Backend llvm` | 预检拒绝；缺少MSVC与Windows SDK |

C++检查使用私有GNU工具路径。
上轮尚未执行全特性检查。

### 安装后复验

本轮采用MSVC开发环境。
GPU测试同时设置CUDA路径。
MSVC目标初缺Clippy。
本轮补装后检查通过。
本轮保留GNU默认工具链。

| 检查 | 实际结果 |
| --- | --- |
| C++ x64烟测；`cl.exe /W4 /WX /EHsc` | Windows头文件、链接与API调用通过 |
| `cargo +stable-x86_64-pc-windows-msvc test --locked` | 13通过；默认不执行GPU |
| `run-cubecl-probe.ps1 -Backend llvm` | 257元素通过；3轮图重放通过 |
| `run-cubecl-probe.ps1 -Backend llvm -GpuTests` | 1通过；覆盖六种长度 |
| `run-cubecl-probe.ps1 -Backend llvm -Elements 1048576` | 精确结果、守卫与3轮重放通过 |
| `cargo +stable-x86_64-pc-windows-msvc test --locked --all-features` | 15通过；4项GPU测试默认忽略 |
| `cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --test cubecl_probe -- --ignored --test-threads=1` | 2通过；同一MSVC构建验证两路 |
| `cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --test gpu_probe -- --ignored --test-threads=1` | 2通过；原生回归与错误设备 |
| `run-cubecl-probe.ps1 -Backend cpp -GpuTests` | 1通过；GNU路线回归 |
| `cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-targets --all-features -- -D warnings` | 通过 |
| `cargo fmt --check` | 通过 |
| PowerShell启动脚本解析 | 通过 |
| `git diff --check` | 通过 |

以上结果不代表部署验收。
