# P1无头GPU探针基线

本页保留此前驱动基线。
以下数量描述此前轮次。
最新进展见[双路探针](cubecl-probe.md)。
C++路线已执行Windows探针。
LLVM路线现已通过Windows探针。
最新能力见[Windows报告](windows-kernel-probe.md)。

日期：2026-10-06。
状态：Windows驱动探针通过。
P1完整准入仍未完成。

## 本轮范围

本轮采用`normify-dev`。
设计预检取得零错误。
本轮只落地两个模块的探针。
本轮不创建其余空壳模块。
本轮不迁移任何物理算法。
本轮不冻结主内核路线。
本轮不提交或推送Git。

用户提供远程Linux机器。
用户要求先开展Windows。
Linux验证等待项目推送。
本轮没有接入该机器。

## CubeCL两路评估

先核对既有源码基线。

```text
repository: tracel-ai/cubecl
revision: 1f73b9f63de50a17398c1d5278e2a5f11612c7e1
```

| 路线 | 显式选择 | 编译方式 | 本轮证据 |
| --- | --- | --- | --- |
| CUDA C++ | `CudaCompiler::new(CudaBackend::Cpp)` | CubeCL生成C++，NVRTC生成设备代码 | 源码核对；未编译或运行 |
| LLVM/PTX | `CudaCompiler::new(CudaBackend::Llvm)` | pliron与LLVM生成PTX | 源码核对；未编译或运行 |

`cpp`仅改变默认路线。
它不移除LLVM依赖。
上游同时编译两个后端。
因此不能只按feature选库。
正式探针须显式指定后端。

依据：[编译器选择](https://github.com/tracel-ai/cubecl/blob/1f73b9f63de50a17398c1d5278e2a5f11612c7e1/crates/cubecl-cuda/src/compiler.rs)、[依赖配置](https://github.com/tracel-ai/cubecl/blob/1f73b9f63de50a17398c1d5278e2a5f11612c7e1/crates/cubecl-cuda/Cargo.toml)。

本轮另下载发行源码作对照。
对照版本为`0.11.0-pre.4`。
它标注不同Git来源。
本轮不将两者视为相同快照。
本轮不将CubeCL加入产品依赖。

```text
crate: cubecl-cuda 0.11.0-pre.4
crate_vcs: 9d1314c070b998008959db021abcc23228ab1eb8
license: MIT OR Apache-2.0
llvm_bundler: 23.1.0-3
```

发行源码链接原生LLVM。
构建脚本还编译C++垫片。
bundler下载平台LLVM包。
它校验包与解压内容摘要。
源码读取未发现Python调用。
这不替代完整依赖链验收。

本机没有找到以下入口：

- `nvcc`。
- `clang`。
- `llvm-config`。
- 标准CUDA安装目录。
- `CUDA_PATH`等环境变量。

本轮没有安装系统工具链。
本轮未尝试构建CubeCL。
缺少工具只提示环境缺口。
它不能证明某路线不兼容。
两路Windows验收均保持待执行。

## 已实现的驱动基线

驱动加载手写PTX。
驱动JIT生成机器码。
探针不调用NVRTC或LLVM。
它不证明Rust内核编译可用。
它也不代表CubeCL执行成功。

| 文件 | 实际职责 |
| --- | --- |
| `src/lib.rs` | 导出探针模块 |
| `src/main.rs` | 解析命令与打印结果 |
| `src/runtime/mod.rs` | 检查配置与核对输出 |
| `src/runtime/cuda.rs` | 管理CUDA缓冲、事件与图 |
| `src/runtime/probe.ptx` | 执行有索引守卫的f32变换 |
| `src/diagnostics/mod.rs` | 报告参数与CUDA错误 |
| `tests/gpu_probe.rs` | 执行显式GPU测试 |

默认构建不启用GPU依赖。
默认探针返回功能缺失错误。
缺少驱动或设备也返回错误。
程序不静默切换到CPU。
CPU只核对已下载的GPU结果。

内核计算`y = 2x + 1`。
输入采用确定的二进制分数。
结果逐元素进行精确比较。
尾部保留16个守卫元素。
GPU测试覆盖六种长度。
它们包含块边界与非整块。

```text
elements: 1, 127, 128, 129, 257, 4097
block_threads: 128
guard_elements: 16
guard_value: -12345.0
ptx_version: 6.0
ptx_target: sm_70
probe_min_driver_api: 12000
probe_max_elements: 1048576
probe_max_replays: 1000
```

这些上限仅约束探针。
它们不冻结引擎设备要求。

## 同步与资源契约

上传与执行采用两个流。
执行流等待上传完成事件。
下载流等待内核完成事件。
探针同步返回，不导出指针。
函数保留模块与全部缓冲。
图释放前等待执行流完成。
错误路径同样执行等待。
适配层为每处unsafe注明前提。

捕获前完成加载与预热。
捕获期间不分配设备缓冲。
捕获期间不编译新内核。
捕获暂时暂停自动事件跟踪。
它避免等待图外的跟踪事件。
私有适配随后恢复跟踪。
重放采用显式跨流事件。
本轮修正过捕获隔离错误905。

每轮重放改写输入内容。
每轮重放先重置输出守卫。
地址、容量与内核配置保持不变。
探针核对每轮的新结果。
此行为不等于图节点参数更新。
本轮未验证外部资源租约。
本轮未验证令牌泄漏与隔离。
完整P3异步契约仍待实现。

## 依赖与许可

| 依赖 | 锁定版本 | 许可 | 范围 |
| --- | --- | --- | --- |
| cudarc | 0.18.2 | MIT OR Apache-2.0 | 可选驱动适配 |
| libloading | 0.8.8 | ISC | 动态库加载 |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 | Linux条件配置；本轮未构建Linux |
| windows-targets | 0.53.5 | MIT OR Apache-2.0 | Windows导入库 |
| windows_x86_64_gnu | 0.53.1 | MIT OR Apache-2.0 | 本机GNU目标导入库 |
| windows-link及其他目标包 | 见Cargo.lock | MIT OR Apache-2.0 | 条件目标记录；本机不构建其他目标 |
| NVIDIA驱动 | 596.36 | NVIDIA专有许可 | 用户系统预装；仓库不分发 |

本轮核对crate许可声明。
本轮核对实际依赖树。
锁文件记录版本与校验摘要。
运行命令采用`--locked`。
发布前仍需整理许可通知。

首试采用新版cudarc。
它引入新版libloading。

| 首试组合 | 版本 |
| --- | --- |
| cudarc | 0.19.10 |
| libloading | 0.9.0 |

本机GNU缺少完整汇编工具。
构建首先找不到`dlltool`。
补入Rust工具路径后仍失败。
`dlltool`无法启动汇编助手。
0.8.9版也需要此链。

本轮改用cudarc `0.18.2`。
加载器固定0.8.8版。
该组合使用预编译导入库。
它在现有GNU环境构建成功。
本轮不修改全局PATH。
本轮不安装系统工具。
此锁定仅服务探针。
它不冻结正式引擎依赖。
更新锁文件须重做GNU验收。

`nvrtc`仅提供PTX类型。
探针不调用其编译函数。
本轮构建脚本不启动Python。
本轮测试和运行也不启动Python。
本轮未模拟独立清洁部署机。
现有机器通过不等于部署完成。

## Windows执行记录

```text
rustc: 1.99.0 (b940084d7 2026-09-28)
cargo: 1.99.0 (5f94df478 2026-08-27)
host: x86_64-pc-windows-gnu
GPU: NVIDIA GeForce RTX 4070 Ti SUPER
SM: 8.9
driver_release: 596.36
driver_api: 13020
PCI: 00000000:01:00.0
```

驱动API版本不是发行号。
SMI中的CUDA号不是SDK安装证据。

```powershell
cargo run --locked --features cuda-probe -- probe
cargo test --locked
cargo test --locked --features cuda-probe
cargo test --locked --features cuda-probe --test gpu_probe -- --ignored --test-threads=1
cargo fmt --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo tree --locked --features cuda-probe
cargo build --locked --release --features cuda-probe
.\target\release\mjwarp-rs.exe probe
cargo run --locked --features cuda-probe -- probe --elements 1048576 --replays 10
cargo doc --locked --all-features --no-deps
```

| 检查 | 实际结果 |
| --- | --- |
| 默认Rust测试 | 11通过；不执行GPU测试 |
| CUDA功能Rust测试 | 10通过；2个GPU测试默认忽略 |
| 显式GPU测试 | 2通过；覆盖六种长度与错误设备 |
| 默认探针 | 257元素；3轮换内容重放通过 |
| 最大元素探针 | 1048576元素；10轮换内容重放通过 |
| Release构建与运行 | 通过；默认探针同样通过 |
| 命令失败边界 | 缺feature与错误设备返回1；零元素返回2 |
| f32与守卫 | 精确比较通过；尾部保持原值 |
| 双流与图 | 上传、下载、完成事件及重放通过 |
| 格式与clippy | 通过 |
| Rust文档生成 | 通过 |
| Linux | 未执行；等待项目推送 |
| CubeCL C++与LLVM | 未执行；只完成源码与环境评估 |
| GPU物理与渲染 | 未实现，未验收 |

## Normify源码证据

本轮刷新根与两个探针模块。
工具计算真实源码指纹。
所有模块仍保持计划态。
原有141项目标契约保持不变。
本轮不激活完整引擎模块。

结构校验取得零错误。
工具仍提示六项警告。
五项提示其余源码尚未落地。
一项提示运行时叶子较粗。
本轮保留这些真实提示。
本轮不拆空模块消除警告。

仓库没有Git首个提交。
刷新只更新指纹与时间。
工具保留原revision占位。
同步工具报告Git失败。

```text
sync/git-failed
```
因此Git漂移审计仍未通过。
用户提交后可重新执行同步。
探针变更取得`verified`状态。
该状态不代表P1准入完成。
工具保留空Git前后修订号。

## P0收口范围

本轮只固定探针局部契约。
它明确f32、索引与缓冲上限。
它明确同步返回与资源保留。
它登记探针依赖与GNU限制。
P0-08只取得阶段证据。

以下事项继续保持待决：

- 完整模型字段与设备布局。
- MuJoCo原生输入ABI。
- BVH借用与跨层查询。
- 回调捕获与设备资源租约。
- 全量失败组合与数值容限。
- CPU调试与发行工具边界。
- 双平台正式工具链与部署。

## 下一步准入

1. 补齐Windows原生编译环境。
2. 按冻结提交构建CubeCL探针。
3. 显式分别选择两个编译器。
4. 检查Rust编译、原子与共享内存。
5. 检查图捕获、缓存和外部缓冲。
6. 检查无SDK部署的实际动态库。
7. 在远程Linux执行同组验收。
8. 双平台验收后再冻结主路线。

驱动探针成功不关闭P1。
本轮不改变GPU功能分母。
