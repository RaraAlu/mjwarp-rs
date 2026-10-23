# W13 落地记录：nvJitLink 自建动态绑定

任务：`2026-10-23-w13-nvjitlink-binding`。
日期：2026-10-24。
评审依据：`docs/development-evidence/layers-w13.md`。
本轮只落地 Windows；Linux 按 2026-10-24 决定并入集中回测。
本轮不切换产品内核，不改 f32 路径、容差与内核字符串。

## 结论

- Rust 侧手写 C ABI 加动态加载，已落地于 `src/runtime/launcher/jitlink.rs`。
- 钉死 nvJitLink **12.9.86**（CUDA 12.9.1 清单），其余 CUDA 组件仍为 12.8.1。
- 同一冻结输入下，新绑定产物与离线证据只差 1 字节。
- 该字节是行号表中的编译目录盘符，还原后摘要一致。
- 设备输出与离线证据逐字节一致。

## 版本裁决：为何不用 12.8.93

用户裁决 A 原写“须与 CUDA 12.8.1 清单匹配”。
实测 12.8.93 无法链接冻结 MathDx LTO。
用户随后要求“检查兼容性后再作决定”。
兼容性诊断如下（输入、选项与发射均相同）：

| 组合 | 链接结果 | cubin 摘要 | 设备输出 |
| --- | --- | --- | --- |
| NVRTC 12.8.93 + nvJitLink 12.8.93 | `potrf7.lto` 被拒：`ERROR 4 in nvvmAddNVVMContainerToProgram, may need newer version of nvJitLink library` | 无 | 无 |
| NVRTC 12.8.93 + nvJitLink 12.9.86 | 通过 | `1086284b…` | 与证据逐字节一致 |
| NVRTC 12.9.86 + nvJitLink 12.9.86（仅诊断） | 通过 | `1086284b…`（与上行相同） | 与证据逐字节一致 |

事实依据：

- 冻结 MathDx LTO 由 Warp 1.15.0 生成；`warp.dll` 内含 `Cuda compilation tools, release 12.9, V12.9.86`。
- LTOIR 只能由同版或更新的 nvJitLink 读取；12.8 读取 12.9 产物失败。
- 12.9.86 读取 NVRTC 12.8 的 LTOIR 成功，向后兼容已实测。
- 两版头文件 ABI 只差导出后缀（`_12_8`/`_12_9`），签名与枚举逐项相同。
- 12.9.86 DLL 只导入 `KERNEL32`、`ADVAPI32`、`ole32`、`SHELL32` 系统库。
- 12.9.86 正是冻结 Warp 内置链接器版本，更贴近冻结上游。
- NVRTC 与 cudart 仍为 12.8.1；现有内核产物与精度证据不受影响。

决定：钉 12.9.86。诊断用的 NVRTC 12.9.86 只放在 `target/w13/`，不进入产品链。

## 冻结 ABI

| 项 | 冻结值 |
| --- | --- |
| 库文件 | Windows `nvJitLink_120_0.dll`；Unix 名称 `libnvJitLink.so.12`（未验收） |
| 版本 | `nvJitLinkVersion` 必须返回 `(12, 9)`，在取版本化符号前核对，否则报“版本不符” |
| 函数 | `__nvJitLink{Create,Destroy,AddData,Complete,GetLinkedCubinSize,GetLinkedCubin,GetErrorLogSize,GetErrorLog,GetInfoLogSize,GetInfoLog}_12_9` |
| 后缀 | 由 `NVJITLINK_VERSION` 推导；头文件无后缀名只是内联包装，DLL 不导出 |
| 值类型 | 句柄为不透明指针；结果与输入类型按 C `int`；长度为 `size_t` |
| 输入类型 | 只开放 LTOIR（3）与 FATBIN（4） |
| 选项 | 固定 `-dlto`、`-arch=sm_<arch>`，`arch` 限 50–129 |
| 错误 | 统一映射为 `LauncherError::Linker`，附 nvJitLink 错误日志 |

## 拒绝面

| 情形 | 行为 | 覆盖 |
| --- | --- | --- |
| 库缺失 | “库缺失” | 单元测试 |
| 相对路径 | “须为绝对路径”，不走系统搜索 | 单元测试 |
| 符号缺失 | “符号缺失”，以 `kernel32.dll` 实测 | 单元测试（Windows） |
| 版本不符 | “版本不符”；12.8、12.10、13.0、11.9 均拒绝 | 单元测试 |
| 链接错误 | 返回结果码与错误日志；垃圾 LTOIR 实测拒绝 | GPU 机测试 |
| 空产物 | “链接产物为空” | 单元测试 |
| 非 CUBIN ELF | 要求 ELF64、小端、`e_machine=190` | 单元测试 |
| 超限输入 | 空列表、空输入、超过 64 项、单项超过 1 GiB、名称为空或含 NUL | 单元测试 |

## 并发与生命周期

- `NvJitLink` 持有库与函数表，可跨线程共享。
- 每次 `link_cubin` 内创建句柄，返回前由守卫销毁。
- 句柄不离开调用，不实现 `Send`，不跨线程共享。
- 同一实例的链接调用经互斥锁串行。
- 库在实例析构时卸载；函数指针不比库存活更久。
- 加载只按绝对路径；Windows 依赖只从库目录与 System32 解析。

## 部署与许可

- `scripts/prepare-cuda-probe.ps1` 另读 12.9.1 清单，钉 `libnvjitlink` 12.9.86。
- 压缩包摘要 `ee7175da9628d47ccc92dce6d28d57ca77633e79079a2aee90e2a645edcd1384`，版本不符即报错。
- DLL 摘要 `998494a8eff0d8382b41ba443d668e3eaea7b67852256ca2982c841cd5bed47a`。
- 组件随附许可原文另存 `target/toolchains/cuda-12.8.1/licenses/`，摘要写入 `components.json`。
- 运行期只从显式目录或 `CUDA_PATH/bin` 加载；未设置时报错，不回退。
- 仓库不提交 SDK 二进制，只在 `target/` 忽略目录落地。
- 许可原文：`LICENSES/CUDA-Toolkit-EULA-12.9.86.txt`，摘要 `72c22161fc1ebf242443d45158a7dd382d68597b00eeb7a7f539be5c6114be33`。
- 该文本与 12.8.1 组件随附文本不同（`e2c71bab…`）；后者本轮未登记。
- 本轮不做法律结论；再分发边界仍属待决。
- 新增直接依赖 `libloading 0.8.8`（ISC），原已由 cudarc 引入，锁文件未增新包。

## 无 Python 边界

- 绑定与测试不调用 `warp.dll`，不读取 venv。
- 调用源 `scripts/native-mathdx7-call.cu` 不含任何头文件。
- 测试按离线证据钉死摘要核对三份输入后才使用。
- 构建与测试命令不含 Python 解释器。

## 对照样本

| 项 | 值 |
| --- | --- |
| 输入 | `potrf7.lto`（`98c01ce6…`）、`potrf7_fatbin.lto`（`c00d6412…`）、调用源（`abfc8c77…`，LF） |
| 调用 LTOIR | NVRTC 12.8，3632 字节，`1415be10…` |
| 链接顺序 | 调用 LTOIR、MathDx LTOIR、FATBIN（与 Warp 相同） |
| cubin | 2086544 字节，`1086284ba5d1bd525d7320faee63978f6db79f955599d02232128da783433fe1` |
| 证据 cubin | 2086544 字节，`de660b5d5095516549c053bf51e13c05b9bc55a3dfe5cfeb0a0833c068be55ae` |
| 唯一差异 | 偏移 951680（0 起），行号表编译目录 `D:\Rust\mjwarp-rs` 对 `C:\Rust\mjwarp-rs` |
| 还原该字节后 | 摘要等于 `de660b5d…` |
| 重复链接 | 两次产物摘要相同 |
| 设备输出 | 104528 字节，`51e1a99b11990d259411de58816a6344cec85f224c6a541fa088030749126d3f`，与证据相同 |
| 覆盖 | 8 次发射：6 批正定（世界数 1、2、257 各两样本）加 2 次坏主元 |
| 守卫 | 输入整段不变；输出与状态两端守卫不变 |

精度映射：绑定只产出机器码，不改产品 f32 路径。
设备输入、输出与共享量均为 f32，与证据逐位一致。
cubin 摘要差异只在调试目录字符串，不影响机器码。

## 验证

平台：Windows，NVIDIA GeForce RTX 4070 Ti SUPER，驱动 596.36，SM89。

| 命令 | 结果 |
| --- | --- |
| `cargo test --locked --features cuda-probe --lib jitlink` | 8 项通过 |
| `cargo test --locked --features cuda-probe --test nvjitlink_probe -- --ignored --test-threads=1` | 2 项通过；设备阶段 6.80 ms；单批最大设备内存按布局计为 101796 字节 |
| `normify gate` | 结果见本目录 `receipt.json` 与任务门禁记录 |

诊断命令（`target/w13/`，不入库）：12.8+12.8、12.8+12.9、12.9+12.9 三组，结果见上表。

## 未完成与影响

- Linux 绑定未验收，并入 `2026-10-24-linux-batch-retest`。
- 产品编译链尚无 LTOIR 输出接口；测试内自建 NVRTC 调用生成。
- MathDx（L5）仍在仓外；接入产品另立任务。
- 新增产品源码在 `cuda-probe` 下，不改既有发射路径。
- 按任务口径，T4 源码身份重钉与 G01 双模式复跑须由 QA 另立任务。
- 运行期依赖用户自装 CUDA 的部署方式未采用。
