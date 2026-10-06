# CubeCL双路探针

日期：2026-10-06。
本轮只验证P1局部能力。
本轮不冻结引擎主路线。

## 当前结果

| 路线 | 实现 | Windows状态 |
| --- | --- | --- |
| 原生PTX | 手写PTX，驱动JIT | GPU回归通过 |
| CubeCL C++ | 同一Rust内核，C++，NVRTC | 六种长度、最大元素与3轮图重放通过 |
| CubeCL LLVM | 同一Rust内核，pliron，LLVM | 构建阻塞；尚未编译或运行 |

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

## LLVM构建阻塞

本机缺少MSVC与Windows SDK。
用户授权安装C++生成工具。
本轮下载微软官方安装器。
本轮核对有效微软签名。
执行策略拦截管理员提权。
本轮没有完成MSVC安装。
本轮没有执行LLVM编译器。
本轮没有安装LLVM二进制包。
Rust自带LLVM不替代链接包。

```text
installer: C:/Rust/mjwarp-rs/target/toolchains/msvc/vs_BuildTools.exe
signature: Valid; Microsoft Corporation
installed Rust target: stable-x86_64-pc-windows-msvc
default Rust target: stable-x86_64-pc-windows-gnu
actual build error: linker `link.exe` not found
```

用户需完成管理员安装。
以下命令安装所需两项。
请在管理员PowerShell执行。
命令不自动重启机器。

```powershell
& 'C:/Rust/mjwarp-rs/target/toolchains/msvc/vs_BuildTools.exe' --quiet --wait --norestart --nocache --add Microsoft.VisualStudio.Component.VC.Tools.x86.x64 --add Microsoft.VisualStudio.Component.Windows11SDK.22621
```

依据：[微软安装参数](https://learn.microsoft.com/en-us/visualstudio/install/use-command-line-parameters-to-install-visual-studio?view=visualstudio)。

完成安装后运行LLVM路线。
脚本读取VS开发环境。
脚本显式选用MSVC Rust目标。
上游bundler再下载LLVM包。
上游bundler负责摘要校验。
本轮不伪造安装标记。
主路线仍等待实际链接验收。

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
| LLVM包 | 本轮未安装；bundler目标23.1.0-3 | 待安装后核对包内通知 |
| cudarc | 0.18.2 | MIT OR Apache-2.0 |
| libloading | 0.8.8 | ISC |
| buildid、option-ext | 1.0.5、0.2.0 | MPL-2.0 |
| CUDA组件 | 官方12.8.1清单 | CUDA Toolkit许可；CCCL另含开源通知 |
| GNU构建工具 | binutils 2.47-3及配套包 | GPL/LGPL等；不链接产品 |
| VS Build Tools | 安装器签名通过；未安装 | 微软许可；不分发安装器 |

本轮核对Cargo许可字段。
锁文件固定全部Rust依赖。
新增依赖含MPL文件级义务。
本轮没有改写这些依赖源码。
发布前仍须整理完整通知。
本项目自身许可仍待用户确定。
元数据清单不等于法务验收。

## 未完成项

- LLVM实际构建与GPU执行。
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
本轮未执行全特性检查。
LLVM阻塞会影响该检查。
以上结果不代表部署验收。
