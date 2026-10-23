# NP02 证据清单与缺口登记（Windows 单平台）

任务：`2026-10-24-np02-evidence-windows`。日期：2026-10-24。执行人：QA。
工具：`scripts/np02-evidence/np02-evidence.ps1`（主流程，PowerShell 7 只读扫描，**不启动 Python**）+ `scripts/np02-evidence/np02-lib.ps1`（路径/哈希、回执实体复核、源扫描、编译点收集）+ 规格 `scripts/np02-evidence/np02-spec.json`。
输入：产品源码根 + 已提交证据；输出：`inventory.json`（逐项清单）与 `gap.json`（缺口分类）。

## 结论（只写 Windows 单平台已实现路径）

- Windows 单平台**已实现路径 88 条**：手写 CUDA 内核 **84**（含 2 条派生内核）+ 冻结 PTX 链 **3** + MathDx Solver LTO 路线 **1**。
- **有证据 80/88（90.91%）**：其中捕获源身份与当前源码一致 40 条、源身份漂移 38 条、派生（证据由基核承担）2 条。
- **缺失证据 8 条**（回执索引无该 entry，也无同源历史回执）：`passive_force_add_fluid`、`fluid_force_clear`、`mass_multiply`、`mass_multiply_validate`、`resident_sparse_gather`、`resident_sparse_validate`、`sparse_solve_fused`、`sparse_solve_validate`。
- 缺口分类 69 条：缺失 8、源身份漂移 38、派 2、未实现子系统 7、部分覆盖 3、deferred（Linux）4、未判定/不适用 5、部分缺失（历史实体/其他）2。
- **本清单不构成 NP02 验收完成结论，也不代表双平台**；缺口为 0 的表述不存在——Windows 范围内仍有 8 条缺失与 38 条源身份漂移。

## 方法（可复现）

1. **上游 pin 强校验**：`target/mjwarp-parallel/upstream` 的 `git rev-parse HEAD` 必须 = `71da24d956378a87a703b6e1442b13aec0c4ac29`；`target/research/warp` 必须 = `ffa038b1adf3927a8e893d4536f1a7562f17d749`（Warp 1.15.0）。不符即报错停手。
2. **回执逐份实体复核**：读 `docs/development-evidence/frozen-inventory/receipts.json`（688 份内核回执索引），逐份重算文件 SHA-256 与索引 `fileSha256` 比对；再按 `<entry>-<ptxSha256>.ptx` 约定检查 PTX 实体是否存在并重算哈希。
3. **产品 CUDA 源扫描**：递归扫描 `src/**/*.rs`（排除 `tests/`、`tests.rs` 与测试探针文件）的 `const ...: &str = r#"..."#` 与 `include_str!("*.cu")`，提取每个源单元内的 `__global__ void <name>`，并计算**该源文本**的 SHA-256（与回执 `sourceSha256` 同一口径：内联常量取引号内原文、`include_str!` 取文件字节）。
4. **编译点补充**：扫描 `SynchronousKernel::compile(...)` 调用名，补充不由独立 const 提供的派生内核（规格 `derived_kernels` 显式登记派生关系与源码锚点），并排除测试探针名。
5. **证据关联**：entry ↔ 回执 ↔ PTX 实体 ↔ T4 冻结清单（`fixtures/precision/t4-current-kernels.json`，30 变体/24 映射）↔ NP01 逐字段映射（191 行）↔ 覆盖清单上游函数/头文件（`frozen-inventory/np02-coverage.json`）。缺 entry 时先按模板实例（`small_prefix_factor_` → `factor_2..6`）与同源旧名回退，仍未命中判缺失。
6. **冻结链实体核对**：`src/runtime/warp_*.rs` 的 `PTX_SHA256`/`CU_sha256` 常量与实体文件哈希逐一比对；模块内 `SMOOTH/PASSIVE/SUPPORT/TYPES_SHA256` 常量与 pinned checkout 对应 `.py` 文件哈希比对。
7. **MathDx 路线**：路线键与四摘要、调用 LTOIR 摘要须同时出现在普通环境与 G01 环境探针日志中。

## 计数表

| 项 | 值 |
| --- | --- |
| Windows 已实现路径 | **88**（内核 84 + 冻结链 3 + MathDx 路线 1） |
| 有证据 | **80**（90.91%） |
| ├ 源身份一致（回执源哈希 = 当前源哈希） | 40 |
| ├ 源身份漂移（回执源哈希 ≠ 当前源哈希） | 38 |
| └ 派生（与基核同源，证据由基核承担） | 2 |
| 缺失证据 | **8** |
| 回执索引 | 688 份，文件哈希 688/688 通过 |
| 回执 PTX 实体 | 637 份在仓且哈希通过；51 份实体不在仓（历史捕获，见缺口） |
| 冻结 PTX 链 | 3 条 / entry 12 个，实体哈希与模块常量一致 |
| MathDx 路线 | 1 条，12/12 摘要命中两环境探针日志 |
| 未实现子系统 | 7 |
| 部分覆盖（NP01 缺口） | 3 |
| Linux deferred | 4 |
| 未判定/不适用 | 5 |

## 8 条缺失证据（明细）

| 内核 | 源 | 说明 |
| --- | --- | --- |
| `passive_force_add_fluid` | `src/physics/passive_force.rs` | 回执索引只有 clear/clear_gravity/sum（5 份），无 add_fluid；新增内核未重采集 |
| `fluid_force_clear` | `src/physics/fluid_force/cuda.rs` | 回执只有 body/project/validate（3 份），无 clear |
| `mass_multiply`、`mass_multiply_validate` | `src/physics/mass_matrix/multiply/cuda.rs` | 回执索引只有 `borrowed_mass_multiply/validate`（借用路径），非借用路径的两个 entry 无回执 |
| `resident_sparse_gather`、`resident_sparse_validate` | `src/physics/mass_solve/sparse/resident.rs` | 回执有 `sparse_dense_*` 与 `resident_mixed_*`，当前 resident 稀疏 entry 名无回执 |
| `sparse_solve_fused`、`sparse_solve_validate` | `src/physics/mass_solve/sparse/resident/solve/cuda.rs` | 回执有 `mixed_solve_*`，当前 sparse solve entry 名无回执 |

上述 8 条均为**产品真实编译点**（`SynchronousKernel::compile` 调用存在），已删除/改名可能性由「同源旧名回退」排除（同源哈希回执 0 条命中），故按缺失登记，不当作通过。

## 源身份漂移 38 条

回执存在、PTX 实体可核对，但捕获时 `sourceSha256` 与当前源文本哈希不一致：与 `frozen-inventory` 结论「产品源 20 个中 10 个漂移」同源问题，本清单给出**内核名粒度**的逐条记录（见 `inventory.json` 中 `evidence_state = source-drift`）。重采集/重基线需另行裁决（本任务不跑 GPU、不改 fixtures）。

## 上游参考（落到具体来源）

- **MJWarp `71da24d956378a87a703b6e1442b13aec0c4ac29`**：每个内核行 `upstream.np01_module` 关联 `frozen-inventory/np02-coverage.json` 的模块/函数（如 kinematics → `_compute_body_matrices`/`_kinematics_branch`/`_site_local_to_global`，mass_solve → `_block_cholesky` 等）与 W15 头文件路径（`include/warp-headers/*.h` 的符号+行号）；未被规则覆盖的内核行 `upstream.confidence` 明确写「未判定」原因。
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
- 证据：`docs/development-evidence/kernel-source-route/` 与 `kernel-source-route-independent-check/`；路线键 `27c33a4e…`、调用 LTOIR 3632 B、求解器 LTOIR 1,034,872 B、FATBIN 506,960 B、CUBIN 2,086,544 B、设备输出 104,528 B、8 次发射；12/12 摘要同时命中普通环境与 G01 环境探针日志。
- 默认不启用、不切物理配方；该路线不属 MJWarp/Warp 内核集合，上游为 MathDx SDK 0.3.1 + 冻结 7 维 POTRF 对照。

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
- `field_batch`：FIELD_BATCH 是传输/计划包装（`src/runtime/transfer`、resident plan），不产生独立 CUDA 内核与 PTX 产物 → 不适用（非内核路径）。
- **未判定 2 项**：① 上游 Warp 生成内核（codegen/builtin 头文件级）逐符号覆盖未展开；② Warp 编译器库内部精度边界无逐项证据。均写明原因，不计入 Windows 缺失。
- **部分缺失**：51/688 份回执的 PTX 实体不在仓（历史捕获依赖 `target/` 产物），已入库 637 份逐份校验通过；与 `docs/three-layer-acceptance.md` §5.2 的 11 项 all-probes 排除同源，本任务不重采集。

## 复现

```powershell
cd D:\Rust\mjwarp-rs
target\toolchains\powershell-7.6.7\bin\pwsh.exe -NoProfile -ExecutionPolicy Bypass `
  -File scripts\np02-evidence\np02-evidence.ps1 `
  -ProductRoot D:\Rust\mjwarp-rs -OutDir docs\development-evidence\np02-evidence
```

约束：脚本只读；不启动 Python；不改产品源码/测试/容差/fixtures；不跑 GPU。上游目录缺失或 pin 不符时直接报错停手，不猜测。

## 边界

- 只覆盖 Windows 单平台**已实现**路径；未实现子系统与 Linux（deferred）单列在 `gap.json`，不计入 Windows 缺失，但不得据此宣称覆盖完整。
- 本清单不替代 NP01 逐字段映射、NP03 数值等价、NP04 性能或 T4 冻结验收；不宣称 NP02 验收完成或双平台结论。
- 数字均来自脚本实测与已提交证据；未采集项标为缺失/未判定，不写成通过。
- 未改 `src`/`tests`/`fixtures`/`Cargo.*`；门禁 `unchanged` 校验同任务提交。
