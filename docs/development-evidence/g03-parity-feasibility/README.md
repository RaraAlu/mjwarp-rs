# G03 parity 可行性预检（只读）

任务：`2026-10-23-g03-parity-feasibility`。对象：`2026-10-16-next-g03-parity`（p84，proposed，本预检不 start、不关闭）。
日期：2026-10-23。只读核查，未改产品代码、测试、容差、fixtures 与脚本，未运行 GPU 测试或 G01 全脚本。

## 结论

**阻塞：不能按现状启动 g03-parity。** 它的验收含「Tile 与正式编译路线 + 双模式平台 + NP01–NP04 全验收」，而对应前置全部停留在评审/缺失状态：

1. **nvJitLink 不可用**：仓内 CUDA 12.8.1 只有 4 个组件（无 `cuda_nvjitlink`、无 `nvJitLink_120_0.dll`/头文件）；`cudarc` 0.18.2（及本地 0.19.10）没有 nvJitLink 绑定；许可与部署未裁决。
2. **MathDx 只有离线产物**：`target/` 下有 LTO/fatbin/cubin 与离线链接工具，但 SDK 头文件/库/许可原文都不在仓内；既有链接能力依赖 pinned Python venv 的 `warp.dll`，产品链禁止 Python。
3. **M7 三条路线仅评审**：W13（nvJitLink 动态绑定）有条件批准、实施延后；W14（tile 发射约束）冻结但实施延后；W15（Warp 头文件原样复制）选路线但落地任务未登记。当前产品无 tile 内核，`LaunchDims` 无共享内存上限校验，`NvrtcSource` 明确拒绝内存头文件（"待W15"）。
4. **NP01–NP04 全域证据缺失**：四类指标均未验收，NP01/NP02 需先做，NP03/NP04 还需 Linux GPU（`gpu:linux-rtx5080`）与同精度性能/显存记录。

但 **G03 数值缺口本身已收口**：矩阵链对齐（`d5b02d5`）后 `mass_solve_probe` 6 过 0 败、NP03 三项全绿；G01 双模式全脚本 340 过 0 败（含文档化排除项，见 `docs/development-evidence/g01-fast-loop/`）。因此 g03-parity 的剩余工作是「Tile/正式编译路线 + 精度四类证据」，不是数值修复。

## 1. nvJitLink 组件与绑定

证据：`cuda-nvjitlink.json`。

| 项 | 实测 | 结论 |
| --- | --- | --- |
| CUDA 12.8.1 组件 | `components.json` 仅 `cuda_nvrtc 12.8.93`、`cuda_cudart 12.8.90`、`cuda_cccl 12.8.90`、`cuda_nvcc 12.8.93`（许可标签均为 `CUDA Toolkit`） | 无 `cuda_nvjitlink` |
| bin 目录 | `cudart64_12.dll`、`nvrtc64_120_0.dll`、`nvrtc64_120_0.alt.dll`、`nvrtc-builtins64_128.dll`、`nvcc/nvlink/ptxas` 等；**无 `nvJitLink_120_0.dll`** | 缺运行库 |
| include | 无 `nvJitLink*.h` | 缺头文件 |
| 系统 CUDA | `C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA` 不存在 | 无系统安装可借用 |
| cudarc 0.18.2 | 16 个模块目录，无 `nvjitlink`；唯一 `NVJITLINK` 字样是 `cufft/sys/mod.rs` 的 `CUFFT_NVJITLINK_FAILURE` 错误码常量 | 无绑定 |
| cudarc 0.19.10（本地缓存） | 同样只出现 cuFFT 错误码常量 | 无绑定 |
| 许可 | `LICENSES/` 只有 Apache-2.0；`THIRD_PARTY_NOTICES.md` 无 CUDA/MathDx 条目 | CUDA Toolkit EULA 未登记 |

缺什么、谁处理：
- 缺 `cuda_nvjitlink` 组件与版本钉死：按 `scripts/prepare-cuda-probe.ps1` 的组件模式补拉并记录摘要（W13 前置 1）。
- 缺 Rust 绑定：W13 结论是「自建 `extern "C"` + 动态加载」，不启用 bindgen、不新增 C/C++ 组件；属 W13 实施任务。
- 缺许可与部署裁决：nvJitLink 属 CUDA Toolkit 组件，许可是 CUDA Toolkit EULA；随二进制再分发、还是运行期依赖用户 CUDA，需用户/产品裁决（W13 前置 2；`docs/unified-launcher.md` 待决清单同项）。

## 2. MathDx/SDK

证据：`mathdx-artifacts.json`。

| 产物 | 位置 | 规格 |
| --- | --- | --- |
| authoring cache LTO | `target/g03-block-reference/warp-cache/1.15.0/lto/*.lto` | 4 组 `*.lto` + `*_fatbin.lto`（约 1.04MB / 507KB 各） |
| 7 维 POTRF | `target/native-mathdx7/attempt-1/potrf7.lto`、`potrf7_fatbin.lto` | 1034872 / 506960 字节 |
| 7 维链接产物 | `target/native-mathdx7-link/attempt-2/mathdx7.cubin` | 2086544 字节 |
| 11 维 POTRF/POTRS | `target/mathdx11/out/**` | `potrf_11_11_1_89_32…cubin`、`potrs_11_11_1_89_32/64…cubin`（含 2.1MB 变体） |
| 离线工具 | `target/mathdx11/oracle.exe`、`link11.exe` | 3572/5576 行级 C++ 工具（离线） |
| pinned venv | `target/mjwarp-parallel/venv/.../warp/bin/warp.dll` | 365392896 字节 |

- 链接来源：`docs/native-mathdx7-link.md` 记录离线探针用 `LoadLibraryExW` 加载 pinned `warp.dll`，`GetProcAddress` 取 `wp_cuda_compile_program`，输入 LTOIR/FATBIN（类型 3/4），选项 `-dlto -arch=sm_89`，产出 CUBIN ELF。
- 边界：该能力依赖 Python venv 与 Warp 头文件，**AGENTS 禁止 Python 构建或运行链**；只能作离线参考。
- SDK 本身：仓内/本机未找到 MathDx 头文件、库或许可原文；文档自述「复用既有本地 SDK 许可、SDK 与生成二进制仅留 `target/`、不替用户判断发布许可」。
- 因此产品侧要用 MathDx LTO，必须同时满足：W13 绑定可用（替代 warp.dll）＋ 用户对 MathDx SDK 许可与部署的裁决 ＋ 版本/摘要冻结；W14 还要求接受「SDK 内部精度未验收」这一边界。

## 3. Tile 与正式编译路线（M7）

证据：`m7-status.json`；评审原文 `docs/development-evidence/layers-w13.md`、`layers-w14.md`、`layers-w15.md`。

| 评审 | 决策状态 | 冻结内容 | 当前实现 |
| --- | --- | --- | --- |
| W13 nvJitLink | **有条件批准，实施延后** | 自建动态绑定；Linux 另评审；前置 7 项（版本/许可/ABI/并发/对照样本/无 Python/Linux） | 未实现 |
| W14 Tile | **tile 约束冻结，实施延后** | 静态 `__shared__`、发射参数 `shared_mem_bytes=0`；块大小 `Fixed(32)`/`WarpGrid(256)` 不匹配即拒绝；每 block 一个 tile 单元；精度全 f32、SDK 内部精度未接受 | 未实现 |
| W15 头文件 | **路线批准（原样复制），落地任务未登记** | Warp `v1.15.0`（`ffa038b1…`）字节复制；必需/条件/排除三级；`cuda_fp16.h` 另固定；逐字段精度映射 | 未实现 |

代码现状（只读核对）：
- `src/runtime/launcher/launch.rs`：`LaunchDims` 只有 `shared_bytes` 字段与 getter，**无上限/opt-in 校验**；`BlockRule` 仅 `Fixed(u32)` 与 `WarpGrid`。
- `src/runtime/launcher/artifact/nvrtc.rs`：非空 headers 直接返回 `Unsupported("内存头文件编译待W15")`。
- `src/physics`：无 `__shared__`，产品路径没有 tile 内核；现有发射 `shared_mem_bytes` 全为 0（W14 基线仍成立）。
- M7 五个评审任务（tile/link/headers/proof/conditional）均 `verified`，note 均为「未实现」；W15 建议的 `2026-10-21-w15-verbatim-headers` **未登记**。

## 4. NP01–NP04 缺什么

依据：`docs/numerical-precision-contract.md`（硬性指标 L306–313、必检范围 L279–297、NP03 顺序 L326–338、当前状态 L458–531）、`docs/three-layer-acceptance.md` §6。四个 NP 任务当前均 `backlog`（P11，资源含 `gpu:windows-t4` + `gpu:linux-rtx5080`）。

| 指标 | 要求 | 当前证据 | 缺什么 |
| --- | --- | --- | --- |
| NP01 类型门禁 | 类型映射 100%、精度偏离 0 | 仅四类修复路径（f32 恢复）有记录；契约明示「其他路径仍需补齐精度证据」 | 全产品逐字段/逐内核类型映射表；上游 69+13+… 处 f64 的逐字段映射（W15 前置） |
| NP02 证据门禁 | 证据覆盖 100%、缺失 0 | 部分路径有源码/PTX/编译选项证据 | 冻结提交+符号+生成内核+编译选项的全量清单；动能「冻结完整生成入口」、一般空间雅可比等仍缺 |
| NP03 数值等价 | 两级判定覆盖 100%、失败/待验收 0 | G01 双模式 all-probes 340/0（含排除项）；G03 mass_solve_probe 6/0；RNE 重基线后 0 失败 | 全产品路径覆盖（不止 G01/G02/G03 探针）；Linux RTX 5080 双平台；契约提到「每模式 20 块仍需上游对照放行」 |
| NP04 同精度性能 | 条件与性能记录缺失 0 | 无 | 同精度批量对照；每项耗时、峰值显存；G01 报告明确 `process_gpu_memory_peak_measured=false`、`fair_performance_acceptance=false`；需 T4/Linux GPU |

顺序约束（契约）：先 NP01/NP02，再 NP03/NP04；debug 与 release 都要覆盖；Linux 未跑必须如实报告。

## 5. 依赖顺序（建议）

1. **用户/产品裁决**（阻塞项）：MathDx SDK 许可与部署方式；nvJitLink 许可与部署方式（随构建缓存固定组件 vs 运行期依赖用户 CUDA）；是否接受 W13 有条件路线；Linux GPU（`linux-rtx5080`）可用性。
2. **W13 实施**：按前置 7 项落地 nvJitLink 动态绑定（含版本钉死、许可登记、ABI/并发冻结、对照样本、无 Python 验证）。
3. **W15 实施**：`2026-10-21-w15-verbatim-headers` 登记并落地（原样复制、摘要、`--include-path`、产物键、拒绝面、CUDA SDK 头、许可、逐字段精度映射）。
4. **W14 实施**：tile ABI 与发射器（共享内存声明、上限/opt-in、块规则一致性、拒绝面、GPU 测试）。
5. **选定内核来源路线**（W15 源码编译 vs 冻结 LTO 产物）并接通 MathDx LTO；核对 Small/稀疏实际算序，给出逐位对照。
6. **NP01/NP02 全产品证据**（类型映射 + 生成内核证据）。
7. **NP03 全产品双平台数值准入**；**NP04 同精度性能**（耗时/显存、T4 与 Linux RTX 5080）。
8. 以上完成后才启动 `2026-10-16-next-g03-parity`；其验收再收口双模式平台与 NP01–NP04。

建议另立任务（未登记，仅建议名）：`2026-10-23-w13-nvjitlink-binding`、`2026-10-23-w15-verbatim-headers`、`2026-10-23-w14-tile-launcher`、`2026-10-23-g03-kernel-route-lto`、`2026-10-23-np01-02-evidence`、`2026-10-23-np03-04-dual-platform`。

## 6. 其他发现

- `docs/windows-mass-solve-probe.md` 仍写「最新结果仍有数值门禁失败」，但 G03 NP03 已由 `d5b02d5` 收口（6 过 0 败，`docs/development-evidence/g03-matrix-chain-alignment/README.md`）。属文档陈旧，本预检未修改，建议后续随 g03-parity 文档任务刷新。
- `docs/unified-launcher.md` 待决清单仍列「nvJitLink 的许可与部署方式」「产品内核来源：手写移植或冻结 PTX」；W15 已选原样复制路线，清单未同步，建议一并刷新。
- 前置任务 `2026-10-16-done-mixed / done-small-mathdx / done-sparse / done-rne` 均 `verified`；g03-parity 的依赖链本身已就绪。

## 复现（全部只读）

```powershell
# CUDA 组件与 DLL
Get-Content target/toolchains/cuda-12.8.1/components.json
Get-ChildItem target/toolchains/cuda-12.8.1/bin | Select-Object Name,Length
Get-ChildItem target/toolchains/cuda-12.8.1/include -Filter 'nvJitLink*'

# cudarc 绑定
Get-ChildItem "$env:USERPROFILE\.cargo\registry\src\index.crates.io-*\cudarc-0.18.2\src" -Directory
Select-String -Path "$env:USERPROFILE\.cargo\registry\src\index.crates.io-*\cudarc-0.18.2\src\*\*\*.rs" -Pattern nvJitLink

# MathDx / LTO 产物
Get-ChildItem target/g03-block-reference/warp-cache/1.15.0/lto
Get-ChildItem target/mathdx11/out -Recurse -Filter *.cubin | Select-Object FullName,Length

# 代码现状
Select-String -Path src/runtime/launcher/launch.rs -Pattern 'shared_bytes'
Select-String -Path src/runtime/launcher/artifact/nvrtc.rs -Pattern '头文件'
```

## 边界

- 只做预检：未实现任何修复，未新增依赖，未改产品代码/测试/容差/fixtures/脚本，未运行 GPU 测试或 G01 全脚本。
- 不宣称 G03、NP01–NP04、release 或 Linux 验收通过；四类指标结论沿用验收文档现状。
- 未做许可法律结论，只列待裁决项与证据缺口。
- 本预检不 start、不关闭 `2026-10-16-next-g03-parity`；它仍为 proposed。
