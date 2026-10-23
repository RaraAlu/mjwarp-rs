# 内核来源路线交付独立复核（快验口径）

任务：`2026-10-24-kernel-source-route-review`。复核对象：提交 `687754c`（任务 `2026-10-24-kernel-source-route-mathdx-wire`，已 verified）。
复核运行 HEAD：`2dee442`（`687754c` 之后仅有 NP01 证据/脚本与任务登记提交，无 `src`/`tests`/`Cargo.*` 改动，见「源码同一性」）。
日期：2026-10-24。执行人：QA。平台：Windows，NVIDIA GeForce RTX 4070 Ti SUPER（驱动 596.36，SM89）；`cargo +1.99.0-x86_64-pc-windows-msvc`。

## 结论

- **交付声明成立**：`RouteKey` 字段与因子敏感性、拒绝面、「不切物理配方」（默认不启用、既有路径与内核字符串/容差/f32 未改、既有配方产物键基线 `b515f1eb…` 复跑一致）、两环境 GPU 探针各 1 过 0 败且四项冻结摘要一致、新文件 ≤500 行与模块挂载、无新增 C/C++/Python、SDK 二进制只在 `target/`、T4 源码身份未漂移，均已复核通过。
- **本批未跑全量 G01 debug/release 双模式**：用户 2026-10-24 决定停止逐次全量 G01（快验迭代，完整双模式只在真正里程碑由 Chief 点名时跑），因此本复核只用 `scripts/g01-fast.ps1` 受影响 target + 定向测试 + 两环境探针（秒级 GPU）。**不得把本条解读为 G01 已在 `687754c` 之后重新验证。**
- **最近一次全量双模式全绿 HEAD 为 `4f362f2`**（结果见 `docs/three-layer-acceptance.md` §4.11：host-msvc 823、all-probes 352/0、86 result 行、11 项文档化排除）。`687754c` 之后的 G01 计数变化（预期 host-msvc +12 单测、all-probes +1 探针）**未实测**，本批不宣称。
- 未改产品/测试/容差/排除清单/G01 脚本；11 项排除清单未动。

## 复核手段与实测数字

命令均在 `D:\Rust\mjwarp-rs` 下执行，`scripts/g01-fast.ps1` 自带 G01 工具链环境（`MJWARP_MUJOCO_ROOT`/`MJWARP_MUJOCO_DLL`、`CUDA_PATH=target\toolchains\cuda-12.8.1`）。

| 复核项 | 命令 | 实测 |
| --- | --- | --- |
| 定向单测（12 项） | `g01-fast.ps1 -Target mjwarp_rs -Test launcher::route -NoBuild` | **12 过 0 败**（1 target，0.55s）；`summary.json` 记录 `acceptanceTool=test-windows-g01.ps1（未执行）` |
| GPU 探针 · G01 环境变量集 | `g01-fast.ps1 -Target kernel_source_route_probe -IgnoredOnly -Nocapture -NoBuild` | **1 过 0 败**（2.3s） |
| GPU 探针 · 普通环境 | 去掉 `MJWARP_MUJOCO_*`、保留 `CUDA_PATH` + PATH，直接运行同一 debug 构建的 `kernel_source_route_probe-08fce4d3913838a4.exe --ignored --nocapture` | **1 过 0 败**（2.10s，exit 0） |

两环境探针元数据逐字段一致，且与交付证据 `docs/development-evidence/kernel-source-route/{probe.log,probe-g01-env.log}` 相同：

```json
{"route":"MathDxLto","route_key_sha256":"27c33a4ebd30c6b8ce8f4602abb4560121545575b7d9147880bb1e2f68b01010","arch":89,"nvrtc_version":[12,9],"nvjitlink_version":[12,9],"libmathdx_version":[0,3,1],"call_ltoir_bytes":3632,"call_ltoir_sha256":"a323db38…","solver_ltoir_bytes":1034872,"solver_ltoir_sha256":"98c01ce6…","solver_fatbin_bytes":506960,"solver_fatbin_sha256":"c00d6412…","cubin_bytes":2086544,"cubin_sha256":"de660b5d…","device_output_bytes":104528,"device_output_sha256":"51e1a99b…","launches":8,"repeat_build_identical":true}
```

- 设备阶段耗时：本次 7.19 ms（普通）/ 8.21 ms（G01 环境），与交付证据 6.92 ms/7.13 ms 同量级（单次采样，非性能结论）。
- 探针工作目录 `C:\Rust\mjwarp-rs`（libmathdx 将 CWD 写入 LTOIR 的前序边界仍在；`MATHDX_EVIDENCE_CWD` 仅测试内切目录）。

## 源码同一性（复核 HEAD 与交付提交）

`git diff --name-only 687754c..HEAD -- src tests fixtures Cargo.toml Cargo.lock scripts` 只命中 `scripts/np01-field-map/*`（NP01 只读映射工具，与编译链无关）；`src`/`tests`/`Cargo.*` 无改动。故在 `2dee442` 上复跑等于在交付源码上复跑。

## 逐项核对

### 1. `RouteKey`（`src/runtime/launcher/route.rs`）

字段（11 项，与声明一致）：`route`、`source_sha256`、`header_sha256`、`include_paths`、`options`（有序）、`arch`、`recipe_fingerprint`（CUDA 路线）、`solver_fingerprint`（MathDx 路线）、`nvrtc`、`nvjitlink`、`libmathdx`；`new()` 校验源/头文件摘要为 64 位十六进制，`sha256()` 对序列化键取摘要。
专用构造：`for_cuda_route`（拒绝非 CUDA 路线、`recipe.validate()`、arch 取自 `ArchTarget::Compute`）、`for_frozen_ptx`（无版本字段）、`for_mathdx_lto`（`solver.validate()`、调用源与求解器 arch 必须一致、13 项冻结 LTOIR 选项 + include 路径按序追加）。
测试覆盖：同输入键稳定（`cuda_route_keys_are_stable_and_factor_sensitive`、`frozen_ptx_route_key_is_stable`）、因子敏感（源/头文件/选项顺序/配方/求解器/arch/三级版本任一变化键即变，`headers_and_option_order_change_key`、`mathdx_route_key_tracks_versions_and_solver`）、摘要非法拒绝（`generic_key_rejects_invalid_digests`）。

### 2. 拒绝面

| 拒绝场景 | 覆盖位置 | 实测 |
| --- | --- | --- |
| 路线未启用（默认 `RouteOptIn::none()`） | `route_opt_in_is_required_and_checked`；探针入口 `build_mathdx_lto_kernel` 先 `require_route_enabled` | 报「路线未启用：MathDxLto」 |
| 路线不符（CUDA 构造收到 MathDx 路线等） | 同上测试 + `cuda_route_rejects_non_cuda_routes` | 显式报错，无静默回退 |
| NVRTC 非 12.9（不回退） | `nvrtc_version_gate_rejects_other_versions` | 报「NVRTC版本不符…不回退」 |
| 架构不一致 | `mathdx_route_rejects_arch_mismatch` | 报「架构不一致：调用源sm_…，求解器sm_…」 |
| 键/内容篡改、空产物、非 CUBIN ELF | `artifact_verify_accepts_consistent_and_rejects_tampering` | `verify()` 逐项报「产物键与内容不一致」「空产物」，`check_cubin` 报「产物不是CUBIN ELF」「不是64位小端ELF」 |
| libmathdx/nvJitLink 版本不符、组件缺失 | `build_mathdx_lto_kernel` 版本门（本任务单元测试覆盖版本门，组件缺失沿用 W13/libmathdx 加载器显式报错） | 未见静默降级路径 |

### 3. 不切物理配方

- `git show --name-status 687754c`：仅新增 `route.rs`、`route/artifact.rs`、`route/tests.rs`、`tests/kernel_source_route_probe.rs`、证据/门禁记录；既有源码只改 `src/runtime/launcher/mod.rs`（`mod route;` 与导出，均 `#[cfg(feature = "cuda-probe")]` 门控）。既有 `artifact/nvrtc.rs`、PTX/CUBIN 路径、内核字符串、容差、f32 未改。
- 12 项定向单测中 `existing_recipe_artifact_key_baseline_is_locked` 复跑通过，既有配方产物键基线 **`b515f1ebfbf8127e13e3e76b737cf30625bb2cd8662213a5a7cc6f665dc16b29`** 与声明一致。
- 默认特性构建未引入新模块（`route` 仅在 `cuda-probe` 下编译）；交付门禁记录 `.normify/runs/2026-10-24-kernel-source-route-mathdx-wire/` 的 fmt（exit 0，1362ms）、clippy、clippy-cuda、unit（480 过 0 败）与声明一致。

### 4. 文件行数与模块挂载

- 实测行数（`.rs` 文件 LF 计数、`[System.IO.File]::ReadAllLines`、`git show --numstat` 三者一致）：`route.rs` **323**、`route/artifact.rs` **232**、`route/tests.rs` **363**、`tests/kernel_source_route_probe.rs` **275**，均 ≤500。
- 模块挂载：`.normify/modules/mjwarp/runtime/index.md` 已列 `src/runtime/launcher/route.rs`、`route/tests.rs`、`route/artifact.rs`、`tests/kernel_source_route_probe.rs`（指纹同步）。
- **文档建议项（不影响结论）**：交付证据 README/receipt 与提交信息标注为 303/222/361/267，比实测少 20/10/2/8 行（疑为按 `Get-Content` 计数所得，该方式在本机把部分行合并）。本批复核按实测记录，不改产品/测试，也不为此重写交付证据；如后续需要口径一致，可按 `normify fix` 只改行数字段。

### 5. 边界核对

- 无新增 C/C++（`git show --name-status` 无相关文件）、无 Python/venv 调用；探针只用 Rust + cudarc + pinned 组件。
- libmathdx 0.3.1 与 NVRTC 12.9.86 组件只在 `target/toolchains/`（`git check-ignore` 命中 `.gitignore:/target`），未入库。
- T4 源码身份未漂移：`git diff --name-only 66256c9..HEAD -- src/physics src/runtime/resident src/runtime/mass_solve src/runtime/mass_matrix` 为空；本交付改动位于 `src/runtime/launcher` 与 `tests/`（cuda-probe 门控），不在 `fixtures/precision/t4-current-kernels.json` 的锚点集合内。

## 未覆盖与未满足项（如实记录）

- **未跑全量 G01 debug/release 双模式**（用户 2026-10-24 节奏决定）：14 阶段数字、all-probes 新计数（预期 352→约 353）、host-msvc 新计数（预期 823→约 835）、墙钟、rustdoc 阶段均**未在本批实测**；最近一次全量双模式结果为 HEAD `4f362f2`（§4.11）。
- 未验证 Linux（按用户决定终止）；未宣称 MathDx 完整验收、上游等价或物理配方切换；性能无结论（仅单次毫秒级采样）。
- 排除清单未动（11 项：W-D5 3、历史捕获身份 6、env/外部产物 1、U038-3D 已知债务 1）；本批既未新增也未删除。

## 复现

```powershell
cd D:\Rust\mjwarp-rs
target\toolchains\powershell-7.6.7\bin\pwsh.exe -NoProfile -ExecutionPolicy Bypass -File scripts\g01-fast.ps1 `
  -Target mjwarp_rs -Test launcher::route
target\toolchains\powershell-7.6.7\bin\pwsh.exe -NoProfile -ExecutionPolicy Bypass -File scripts\g01-fast.ps1 `
  -Target kernel_source_route_probe -IgnoredOnly -Nocapture
# 普通环境探针（去掉 MJWARP_MUJOCO_*，仅保留 CUDA_PATH + PATH，直接跑同一 debug 构建的测试二进制）
```
