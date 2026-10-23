# 内核来源路线标准化与 MathDx LTO 接入编译链

任务：`2026-10-24-kernel-source-route-mathdx-wire`。日期：2026-10-24。
前置（均 verified）：W13 nvJitLink 绑定、LTOIR 输出接口、W14 tile、W15 头文件、libmathdx 0.3.1 与 pinned NVRTC 12.9.86 组件。
平台：Windows，NVIDIA GeForce RTX 4070 Ti SUPER，驱动 596.36，SM89。

## 结论

- 定义四类内核来源路线：`HandWrittenCuda`、`FrozenPtx`、`UpstreamSource`（W15 头文件）、`MathDxLto`；**新路线默认不启用**，只经显式 `RouteOptIn` 暴露。
- 产物键 `RouteKey` 纳入：路线、源 SHA-256、头文件集合摘要、include 路径、NVRTC 选项与顺序、arch、配方指纹（CUDA 路线）/求解器指纹（MathDx）、NVRTC/nvJitLink/libmathdx 版本；同输入稳定，任一因子变化键即变化（测试锁定）。
- MathDx LTO 接入 launcher：`build_mathdx_lto_kernel` 生成调用 LTOIR 与求解器 LTOIR/FATBIN，经 W13 `NvJitLink` 按「调用 LTOIR → MathDx LTOIR → FATBIN」链接为 CUBIN；产物携带路线、版本组合与四级摘要，`KernelArtifact::verify` 可复核键与内容一致。
- GPU 探针用新 API 复现冻结证据：LTOIR 1,034,872 B `98c01ce6…`、FATBIN 506,960 B `c00d6412…`、CUBIN 2,086,544 B `de660b5d…`、设备输出 104,528 B `51e1a99b…`、8 次发射、重复一致；普通环境与 G01 环境变量集各跑一次，结果相同。
- **不切物理配方**：默认路径不启用新路线；既有 `artifact.rs`/launch 路径、内核字符串、容差与 f32 均未改；既有配方产物键行为用基线测试锁定（`b515f1eb…`）。

## 接口（`src/runtime/launcher/route.rs` 303 行 + `route/artifact.rs` 222 行）

| 对象 | 说明 |
| --- | --- |
| `KernelSourceRoute` | 四类路线枚举；`ALL`、`as_str`、`enabled_by_default() == false` |
| `RouteOptIn` | 显式启用凭证；`none()` 默认、`enable(route)` |
| `require_route_enabled` / `require_mathdx_nvrtc` | 路线未启用/不符、NVRTC 非 12.9 的显式拒绝（不回退） |
| `RouteKey` | 确定性产物键；`for_cuda_route`（手写/W15）、`for_frozen_ptx`、`for_mathdx_lto`；`sha256()` |
| `build_mathdx_lto_kernel` | MathDx LTO 端到端入口（需显式 opt-in）；返回 `KernelArtifact` |
| `KernelArtifact` / `KernelArtifactMetadata` | 产物字节与路线/版本/各级摘要；`verify()` 复核键与内容 |

- MathDx 路线选项与顺序沿用冻结 13 项 NVRTC 选项（`--gpu-architecture=sm_<arch>` 起）；调用源 include 路径按序追加 `--include-path=`。
- 拒绝面：路线未启用/不符、libmathdx 版本不符、nvJitLink 版本不符、NVRTC 版本不符（不回退）、组件缺失（由加载器显式报错）、源/头文件摘要非法、空产物、非 CUBIN ELF、产物键与内容不一致；均有单元或 GPU 探针覆盖。

## 验证（本机实测）

| 命令 | 结果 |
| --- | --- |
| `cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features cuda-probe --lib launcher::route` | **12 过 0 败**（`unit-tests.log`） |
| 普通环境：`CUDA_PATH=target/toolchains/cuda-12.8.1`，PATH 加其 bin | **1 过 0 败**（`probe.log`） |
| G01 环境变量集：另加 `MJWARP_MUJOCO_ROOT`/`MJWARP_MUJOCO_DLL` | **1 过 0 败**（`probe-g01-env.log`） |
| 既有配方产物键基线（`ArtifactKey::new`，物理配方，NVRTC 12.8） | `b515f1ebfbf8127e13e3e76b737cf30625bb2cd8662213a5a7cc6f665dc16b29`（测试锁定） |

探针元数据（两环境一致；工作目录 `C:\Rust\mjwarp-rs`）：

```json
{"route":"MathDxLto","route_key_sha256":"27c33a4ebd30c6b8ce8f4602abb4560121545575b7d9147880bb1e2f68b01010","arch":89,"source_sha256":"abfc8c777014708ed9b33c7a80798e8d2160f102f49a21d01fbf203911968853","header_sha256":"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855","nvrtc_version":[12,9],"nvjitlink_version":[12,9],"libmathdx_version":[0,3,1],"call_ltoir_bytes":3632,"call_ltoir_sha256":"a323db381e0793b03837c95179965eb90aeb44c110eae042f16ede1d587da953","solver_ltoir_bytes":1034872,"solver_ltoir_sha256":"98c01ce64f55d31af5d4eb001c1e9fc4c8456173a9f0bbe97be479c804a981eb","solver_fatbin_bytes":506960,"solver_fatbin_sha256":"c00d6412780b18f67e11a1d93fc34d51db9cdd687f26ec7bc926efbff5ddbd1f","cubin_bytes":2086544,"cubin_sha256":"de660b5d5095516549c053bf51e13c05b9bc55a3dfe5cfeb0a0833c068be55ae","device_output_bytes":104528,"device_output_sha256":"51e1a99b11990d259411de58816a6344cec85f224c6a541fa088030749126d3f","launches":8}
```

- 重复构建：两次 `build_mathdx_lto_kernel` 的键摘要与元数据完全一致（`repeat_build_identical=true`）。
- 路线未启用：`RouteOptIn::none()` 调用显式报「路线未启用：MathDxLto」。
- 环境依赖沿用前序结论：libmathdx 将工作目录写入 LTOIR，探针在冻结证据工作目录 `C:\Rust\mjwarp-rs`（`MATHDX_EVIDENCE_CWD` 可覆盖）生成并在结束/panic 时恢复；产品 API 不做 chdir；NVRTC 从 pinned 组件显式解析。

## 边界与影响

- 不切换任何物理配方；新路线默认不启用，仅显式 API；既有 PTX/cubin 编译路径、内核字符串、容差与 f32 未改。
- 既有配方产物键基线锁定为 `b515f1eb…`；`artifact.rs` 等既有文件未改（本任务只加新文件 + `launcher/mod.rs` 挂载导出）。
- G01 影响（待 QA 复跑确认）：all-features host-msvc 会新增本模块 12 项单元测试；all-probes 的 `--ignored` 会自动纳入新增 1 项 GPU 探针（已在 G01 环境变量集下验证通过）。未改 G01 脚本与排除清单。
- 新文件（303/222/361/267 行）均 ≤500 行并已 `normify module source` 挂到 `mjwarp.runtime`；不新增 C/C++、不跑 Python；SDK 二进制只在 `target/`。
- Linux 未涉及；T4 源码身份：改动在 `src/runtime/launcher` 与 `tests/`（cuda-probe 门控），不在 T4 锚点集合（physics/resident/mass-solve）内，未漂移。
- 未宣称 MathDx 完整验收、上游等价或物理配方切换；本任务只交付路线标准化与管线能力。

## 复现

```powershell
target\toolchains\powershell-7.6.7\bin\pwsh.exe -NoProfile -File scripts\prepare-cuda-probe.ps1
$env:CUDA_PATH = 'D:\Rust\mjwarp-rs\target\toolchains\cuda-12.8.1'
$env:PATH = "$env:CUDA_PATH\bin;$env:PATH"
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features cuda-probe `
  --test kernel_source_route_probe -- --ignored --test-threads=1 --nocapture
```
