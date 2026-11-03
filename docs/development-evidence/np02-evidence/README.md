# NP02 证据清单与缺口登记（Windows 单平台）

任务：`2026-10-24-np02-evidence-windows`（清单）+ `2026-10-24-np02-missing-capture`（缺失补采）。日期：2026-10-24。执行人：QA。
工具：`scripts/np02-evidence/np02-evidence.ps1`（主流程，PowerShell 7 只读扫描，**不启动 Python**）+ `scripts/np02-evidence/np02-lib.ps1`（路径/哈希、回执实体复核、源扫描、编译点收集）+ 规格 `scripts/np02-evidence/np02-spec.json`。
输入：产品源码根 + 已提交证据（含 `frozen-inventory/receipts.json` 主索引与 `np02-evidence/receipt.json` 补采索引）；输出：`inventory.json`（逐项清单）与 `gap.json`（缺口分类）。

## 结论（只写 Windows 单平台已实现路径）

- Windows 单平台**已实现路径 88 条**：手写 CUDA 内核 **84**（含 2 条派生内核）+ 冻结 PTX 链 **3** + MathDx Solver LTO 路线 **1**。
- **有证据 88/88（100%），缺失 0**：其中源身份一致 77 条、按用户 2026-10-24 裁决 B 归档的**已知漂移 9 条**（历史版本证据，不重采不重基线）、派生（证据由基核承担）2 条。
- 回执 **701/701** 文件哈希通过（688 份历史索引 + 13 份本批补采）；PTX 实体在仓并通过哈希 **650** 份，51 份不在仓（历史捕获产物依赖 `target/`，见缺口）。
- 缺口分类 32 条：已知漂移 9、未实现子系统 7、部分覆盖 3、Linux deferred 4、未判定/不适用 5、部分缺失（历史实体）2、派生 2（不重复计数见 `gap.json`）。
- **本清单不构成 NP02 验收完成结论，也不代表双平台**；缺口未清零（9 条已知漂移、20 个 NP01 内部字段、51 份历史实体不在仓），不宣称双平台结论。

## 本批补采（`2026-10-24-np02-missing-capture`）

此前缺失的 8 条内核已用产品现有捕获管线补采（`src/runtime/transfer/kernel/ptx.rs::record_physics_ptx`，`MJWARP_PHYSICS_PTX_DIR` 指定捕获根，`cargo test --locked --features cuda-probe --lib <filter> -- --ignored --test-threads=1`）：

| 内核 | 捕获结果 | 证据状态 |
| --- | --- | --- |
| `passive_force_add_fluid` | 1 份，passed=true | verified（参数化/拼接编译单元） |
| `fluid_force_clear` | 2 份，passed=true | verified（参数化/拼接编译单元） |
| `mass_multiply` | 3 份，passed=true | verified（参数化/拼接编译单元） |
| `mass_multiply_validate` | 3 份，passed=true | verified（参数化/拼接编译单元） |
| `resident_sparse_gather` | 1 份，passed=true | verified（源哈希 = 当前 const 源） |
| `resident_sparse_validate` | 1 份，passed=true | verified（源哈希 = 当前 const 源） |
| `sparse_solve_fused` | 1 份，passed=true | verified（源哈希 = 当前 const 源） |
| `sparse_solve_validate` | 1 份，passed=true | verified（源哈希 = 当前 const 源） |

- 三次定向 GPU 测试合计 **14 过 0 败**（passive_force 5 过 15.11s、mass_matrix::multiply 7 过 17.30s、mass_solve::sparse::resident 2 过 1.87s），GPU 独占，未跑全量 G01。
- 入库 13 组回执 + PTX/CU 实体：`docs/development-evidence/np02-evidence/missing-capture/receipts/`；索引与逐项结果：`docs/development-evidence/np02-evidence/receipt.json`。
- 未用旧名回退、未伪造：补采回执的 `sourceSha256` 与当前源按 LF 规范化后的哈希一致（4 条参数化单元以「捕获 .cu 完整包含当前 const 源文本」判定身份）。

## 方法（可复现）

1. **上游 pin 强校验**：`target/mjwarp-parallel/upstream` 的 `git rev-parse HEAD` 必须 = `71da24d956378a87a703b6e1442b13aec0c4ac29`；`target/research/warp` 必须 = `ffa038b1adf3927a8e893d4536f1a7562f17d749`（Warp 1.15.0）。不符即报错停手。
2. **回执逐份实体复核**：主索引 688 份 + 补采索引 13 份，逐份重算文件 SHA-256 并与索引 `fileSha256` 比对；再按 `<entry>-<ptxSha256>.ptx/.cu` 约定复核实体哈希（`sourceSha256`/`ptxSha256`）。
3. **产品 CUDA 源扫描**：递归扫描 `src/**/*.rs`（排除 `tests/`、`tests.rs` 与测试探针文件）的 `const ...: &str = r#"..."#` 与 `include_str!("*.cu")`，提取每个源单元内的 `__global__ void <name>`，并计算**该源文本**的 SHA-256。**行尾口径**：rustc 读取源码时把 CRLF 规范化为 LF（字符串字面量同样生效），故源哈希按 LF 规范化后计算；工作区因 `core.autocrlf` 检出为 CRLF 时不再误判为漂移。
4. **编译点补充**：扫描 `SynchronousKernel::compile(...)` 调用名，补充不由独立 const 提供的派生内核（规格 `derived_kernels` 显式登记派生关系与源码锚点），并排除测试探针名。
5. **证据关联**：entry ↔ 回执 ↔ PTX 实体 ↔ T4 冻结清单（30 变体/24 映射）↔ NP01 逐字段映射（191 行）↔ 覆盖清单上游函数/头文件。缺 entry 时先按模板实例（`small_prefix_factor_` → `factor_2..6`）与同源旧名回退，仍未命中判缺失；参数化/拼接编译单元（`#define` 前导、多源拼接）以「捕获 `.cu` 实体哈希 = 回执 sourceSha256 且完整包含当前 const 文本」判定身份。
6. **冻结链实体核对**：`src/runtime/warp_*.rs` 的 `PTX_SHA256`/`CU_SHA256` 常量与实体文件哈希逐一比对；模块内 `SMOOTH/PASSIVE/SUPPORT/TYPES_SHA256` 常量与 pinned checkout 对应 `.py` 文件哈希比对。
7. **MathDx 路线**：路线键与四摘要、调用 LTOIR 摘要须同时出现在普通环境与 G01 环境探针日志中。

## 计数表

| 项 | 值 |
| --- | --- |
| Windows 已实现路径 | **88**（内核 84 + 冻结链 3 + MathDx 路线 1） |
| 有证据 | **88**（100%，缺失 0） |
| ├ 源身份一致 | 77 |
| ├ 已知漂移（裁决 B 归档，不重采不重基线） | 9 |
| └ 派生（与基核同源，证据由基核承担） | 2 |
| 回执索引（主 + 补采） | 701 份，文件哈希 701/701 通过 |
| 回执 PTX 实体 | 650 份在仓且哈希通过；51 份实体不在仓（历史捕获） |
| 冻结 PTX 链 | 3 条 / entry 12 个，实体哈希与模块常量一致 |
| MathDx 路线 | 1 条，12/12 摘要命中两环境探针日志 |
| 未实现子系统 | 7 |
| 部分覆盖（NP01 缺口） | 3 |
| Linux deferred | 4 |
| 未判定/不适用 | 5 |

## 9 条已知漂移（用户 2026-10-24 裁决 B：归档，不重采不重基线）

回执与 PTX 实体可核对，但捕获时 `sourceSha256` 与当前源文本（LF 规范化）不一致，属历史版本证据：

`rigid_mass_matrix`、`rigid_kinematics`、`rigid_mass_solve`、`passive_force_clear`、`passive_force_clear_gravity`、`passive_force_sum`、`fluid_force_body`、`fluid_force_project`、`fluid_force_validate`。

说明：上一轮清单（`3dcc1f2`）报告 38 条漂移；本轮修正行尾口径后复测，其中 29 条为 CRLF 检出导致的**假阳性**（编译源 LF vs 工作区 CRLF），真实漂移为上述 9 条。两种口径都以回执与源哈希实测为准，本表为当前权威数字。

## 上游参考（落到具体来源）

- **MJWarp `71da24d956378a87a703b6e1442b13aec0c4ac29`**：每个内核行 `upstream.np01_module` 关联 `frozen-inventory/np02-coverage.json` 的模块/函数（如 kinematics → `_compute_body_matrices`/`_kinematics_branch`，mass_solve → `_block_cholesky` 等）与 W15 头文件路径（`include/warp-headers/*.h` 的符号+行号）；未被规则覆盖的内核行写明「未判定」原因。
- **Warp `v1.15.0`（`ffa038b1…`）**：3 条冻结链的生成内核符号共 12 个（如 `_kinematics_branch_95f8028e_cuda_kernel_forward`、`_subtree_com_acc_350f0560_cuda_kernel_forward`、`_gravity_force_7c63ac28_cuda_kernel_forward`），并核对模块常量与 pinned checkout 的 `smooth.py`/`types.py`/`passive.py`/`support.py` 哈希（全部一致）。
- 不使用「文件未改」替代产物核对：冻结链按 PTX/CU **实体文件哈希**核对；手写内核按回执 PTX 摘要 + 实体文件哈希核对。

## 冻结 PTX 链（3 条，全部实体哈希一致）

| 链 | 模块 | 实体 | PTX SHA-256 | entry |
| --- | --- | --- | --- | --- |
| `warp-independent-kinematics-com-cdof` | `src/runtime/warp_kinematics.rs` | `fixtures/gravity-upstream-abi/independent-inputs/smooth.ptx` | `4179acd0…`（= 模块常量） | 7 |
| `warp-independent-com` | `src/runtime/warp_com.rs` | 同上 `smooth.ptx` | `4179acd0…` | 4 |
| `warp-generated-gravity` | `src/runtime/warp_gravity.rs` | `fixtures/gravity-upstream-abi/generated/module.ptx` | `d06154da…`（= 模块常量） | 1 |

`.cu` 实体哈希同样与模块 `CU_SHA256` 常量一致；生成回执 `receipt.json` 记录的上游源哈希与模块常量一致。

## MathDx Solver LTO 路线（1 条）

- 入口 `build_mathdx_lto_kernel`（`src/runtime/launcher/route/artifact.rs`），版本组合 NVRTC 12.9.86 / nvJitLink 12.9.86 / libmathdx 0.3.1，13 项冻结 LTOIR 选项 + `--include-path=`。
- 证据：`docs/development-evidence/kernel-source-route/` 与 `kernel-source-route-independent-check/`；路线键 `27c33a4e…`、调用 LTOIR 3632 B、求解器 LTOIR 1,034,872 B、FATBIN 506,960 B、CUBIN 2,086,544 B、设备输出 104,528 B、8 次发射；12/12 摘要命中两环境探针日志。
- 默认不启用、不切物理配方；该路线不属 MJWarp/Warp 内核集合。

## 未实现子系统（7，不计入 Windows 缺失）

源自 NP01 缺口清单（`docs/development-evidence/np01-field-mapping/diff.json`），逐条带上游文件与产品路径：

1. 渲染/BVH（`render.py`、`render_util.py`、`bvh.py`；r01-p8 与 R02–R10 顺延）
2. 地形/SDF 碰撞（`hfield`、`collision_sdf`；g10-p5 顺延）
3. 凸体/网格 GJK、EPA、convex 支持映射（G08/G09 仅原语窄相）
4. 柔体碰撞（flex collision）
5. 约束求解（constraint、efc 行地址、约束一致性）
6. 求解器管线（solver：CG/Newton、收敛判定）
7. 岛屿/休眠分组（island；仅 sleep 树级切片）

部分覆盖 3 条：IO/CLI（仅状态导出与入口）、block_cholesky 完整块分解（仅 dense/small 切片）、smooth/passive/forward/inverse/sensor/history 完整语义（逐切片实现）。

## 不适用与未判定（如实登记）

- `tendon_wrap`（`src/physics/tendon_wrap.rs`）：只有 `__device__` 辅助函数，无独立 `__global__` 条目；由 `resident/plan_kernels.rs` 与 `SPATIAL_TENDON_CUDA` 拼接编译为 resident 路径内核 → 不适用（非独立内核）。
- `rne_acc`：产品由 `rne.cu` 的 `rne_cacc`/`rne_qfrc`/`rne_backward` 组合实现，无同名 `__global__` → 不适用（无可独立计数内核）。
- `field_batch`：FIELD_BATCH 是传输/计划包装，不产生独立 CUDA 内核与 PTX 产物 → 不适用（非内核路径）。
- **未判定 2 项**：① 上游 Warp 生成内核（codegen/builtin 头文件级）逐符号覆盖未展开；② Warp 编译器库内部精度边界无逐项证据。均写明原因，不计入 Windows 缺失。
- **部分缺失**：51/701 份回执的 PTX 实体不在仓（历史捕获依赖 `target/` 产物），在仓 650 份逐份校验通过；与 `docs/three-layer-acceptance.md` §5.2 的 11 项 all-probes 排除同源，本批不重采集。

## 复现

```powershell
cd D:\Rust\mjwarp-rs
target\toolchains\powershell-7.6.7\bin\pwsh.exe -NoProfile -ExecutionPolicy Bypass `
  -File scripts\np02-evidence\np02-evidence.ps1 `
  -ProductRoot D:\Rust\mjwarp-rs -OutDir docs\development-evidence\np02-evidence
```

补采复现（GPU 独占，约 35 秒）：

```powershell
$env:CUDA_PATH = 'D:\Rust\mjwarp-rs\target\toolchains\cuda-12.8.1'
$env:PATH = "$env:CUDA_PATH\bin;$env:PATH"
$env:MJWARP_MUJOCO_ROOT = 'D:\Rust\mjwarp-rs\target\toolchains\mujoco-3.12.0\package'
$env:MJWARP_MUJOCO_DLL = "$env:MJWARP_MUJOCO_ROOT\bin\mujoco.dll"
$env:MJWARP_PHYSICS_PTX_DIR = 'D:\Rust\mjwarp-rs\target\np02-capture\final'
$env:MJWARP_MUL_M_REPORT = 'D:\Rust\mjwarp-rs\target\np02-capture\reports\mass-multiply.json'
$env:MJWARP_RESIDENT_MIXED_REPORT = 'D:\Rust\mjwarp-rs\target\np02-capture\reports\resident-mixed.json'
$env:MJWARP_MIXED_FACTOR_REPORT = 'D:\Rust\mjwarp-rs\target\np02-capture\reports\mixed-factor.json'
$env:MJWARP_MIXED_SOLVE_REPORT = 'D:\Rust\mjwarp-rs\target\np02-capture\reports\mixed-solve.json'
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features cuda-probe --lib physics::passive_force -- --ignored --test-threads=1
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features cuda-probe --lib physics::mass_matrix::multiply -- --ignored --test-threads=1
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features cuda-probe --lib physics::mass_solve::sparse::resident -- --ignored --test-threads=1
```

约束：扫描脚本只读；不启动 Python；不改产品源码/测试/容差/fixtures；不跑全量 G01。上游目录缺失或 pin 不符时直接报错停手，不猜测。

## 边界

- 只覆盖 Windows 单平台**已实现**路径；未实现子系统与 Linux（deferred）单列在 `gap.json`，不计入 Windows 缺失，但不得据此宣称覆盖完整。
- 9 条已知漂移按用户 2026-10-24 裁决 B 归档（历史版本证据，不重采不重基线）；不宣称漂移清零。
- 本清单不替代 NP01 逐字段映射、NP03 数值等价、NP04 性能或 T4 冻结验收；不宣称 NP02 验收完成或双平台结论。
- 未改 `src`/`tests`/`fixtures`/`Cargo.*`；门禁 `unchanged` 校验同任务提交。

## NP02 门禁第一片：清单静态棘轮 + 证据指针可解析（2026-11-04）

任务：`2026-11-03-precision-np02-evidence-gate`（CPU-only、零窗口）。本片不重跑扫描工具
（它需要 `target/mjwarp-parallel` 上游 checkout，仓瘦身后不在仓），只把**已提交清单**钉成机器门禁，
防止证据与缺口在无人注意时退化。

实现：`tests/precision_evidence_gate.rs`（3 个用例，随 `cargo test --locked` 执行）。

1. **计数棘轮**：`inventory.json` 的 `counts` 必须与测试内快照完全一致——已实现路径 88、内核 84、
   冻结链 3、MathDx 路线 1、有证据 88、缺失 0、证据覆盖 100%、回执 701（文件哈希通过 701）、
   PTX 实体通过 650、缺口 32、未实现 7、未判定 5；`rows` 88 条、`gaps` 32 条。数字滑动必须显式改测试。
2. **指针可解析**：每行的 `source.path`（派生源行写说明文字，按非路径跳过）与 `artifact.evidence_paths`
   每一项都必须在仓内存在；当前可解析指针 **158 条**，少于该数即失败。
3. **词表与 pin**：`evidence_state` ∈ {verified, source-drift, derived}；`HandWrittenCuda` 行的
   `compile.precision_gate` 必须为 `f32-physics`；MJWarp pin `71da24d…`、Warp pin `ffa038b1…` 不得漂移；
   `platform.os` 必须仍是 Windows（本清单只覆盖单平台，不得被写成双平台）。

**本片未覆盖**：重跑 np02 工具与回执文件哈希重算、PTX 实体重算、双平台；21 条已知缺口
（9 条已知漂移、7 个未实现子系统、20 个 NP01 内部字段、51 份历史实体不在仓）仍是未验收项，
本片不宣称 NP02 通过。

复现：`cargo test --locked --test precision_evidence_gate`。