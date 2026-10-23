# NP02 缺失内核补采（Windows 单平台）

任务：`2026-10-24-np02-missing-capture`。日期：2026-10-24。执行人：QA。
上游任务：`2026-10-24-np02-evidence-windows`（清单报告 8 条缺失）。用户 2026-10-24 裁决 B：补采缺失 8 条；38 条漂移按裁决 B 归档（本轮复测实际为 9 条真实漂移，见 `np02-evidence/README.md`）。

## 结论

- 8 条缺失内核全部用**产品现有捕获管线**补采成功，每条 `passed=true`，PTX/CU 实体哈希与回执逐份一致：`passive_force_add_fluid`、`fluid_force_clear`、`mass_multiply`、`mass_multiply_validate`、`resident_sparse_gather`、`resident_sparse_validate`、`sparse_solve_fused`、`sparse_solve_validate`。
- 三次定向 GPU 测试合计 **14 过 0 败**，墙钟约 **34 秒**：`physics::passive_force` 5 过 15.11s、`physics::mass_matrix::multiply` 7 过 17.30s、`physics::mass_solve::sparse::resident` 2 过 1.87s。
- NP02 清单更新为 **88/88 有证据（100%）、缺失 0**；未用旧名回退、未伪造、未改产品/测试/容差/fixtures。
- 不宣称 NP02 验收完成或双平台结论；9 条真实漂移按裁决 B 归档、不重采不重基线。

## 捕获管线与方法

- 捕获实现：`src/runtime/transfer/kernel/ptx.rs::record_physics_ptx`（产品编译路径内，`#[cfg(test)]` 侧由 `MJWARP_PHYSICS_PTX_DIR` 触发），每份回执记录 `entry`、`sourceSha256`、`ptxSha256`、`precisionGate=f32-physics`、`passed`、`nvrtcVersion`、`fmad=true`、`useFastMath=false`、`options=["--std=c++17","--gpu-architecture=compute_70"]`，并落盘同名 `.cu`/`.ptx`/`.json`。
- 命令（GPU 独占，逐条 `--test-threads=1`）：

```powershell
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features cuda-probe --lib physics::passive_force -- --ignored --test-threads=1
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features cuda-probe --lib physics::mass_matrix::multiply -- --ignored --test-threads=1
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features cuda-probe --lib physics::mass_solve::sparse::resident -- --ignored --test-threads=1
```

- 环境：`CUDA_PATH=target/toolchains/cuda-12.8.1`（入 PATH）；`MJWARP_MUJOCO_ROOT=target/toolchains/mujoco-3.12.0/package`、`MJWARP_MUJOCO_DLL=...\bin\mujoco.dll`；报告类探针环境 `MJWARP_MUL_M_REPORT`、`MJWARP_RESIDENT_MIXED_REPORT`、`MJWARP_MIXED_FACTOR_REPORT`、`MJWARP_MIXED_SOLVE_REPORT` 指向 `target/np02-capture/reports/`；捕获根 `MJWARP_PHYSICS_PTX_DIR=target/np02-capture/final`。
- 复跑日志：`capture-log.txt`（三次 cargo 的尾部输出）。

## 采集结果（逐条）

| 内核 | 来源测试 | 份数 | passed | PTX 摘要 | 判定 |
| --- | --- | --- | --- | --- | --- |
| `passive_force_add_fluid` | `physics::passive_force::tests_fluid` | 1 | true | 见回执 | verified（参数化/拼接单元：`.cu` 包含当前 const 源） |
| `fluid_force_clear` | `physics::passive_force::tests_fluid` | 2 | true | 同上 | verified（参数化/拼接单元） |
| `mass_multiply` | `physics::mass_matrix::multiply::tests` | 3 | true | 同上 | verified（参数化单元，`#define MASS_NV/MASS_DENSE` 前导） |
| `mass_multiply_validate` | 同上 | 3 | true | 同上 | verified（参数化单元） |
| `resident_sparse_gather` | `physics::mass_solve::sparse::resident::tests` | 1 | true | 同上 | verified（源哈希 = 当前 const 源） |
| `resident_sparse_validate` | 同上 | 1 | true | 同上 | verified（源哈希 = 当前 const 源） |
| `sparse_solve_fused` | `physics::mass_solve::sparse::resident::solve::tests` | 1 | true | 同上 | verified（源哈希 = 当前 const 源） |
| `sparse_solve_validate` | 同上 | 1 | true | 同上 | verified（源哈希 = 当前 const 源） |

## 产物

- 回执 + 实体（13 组 `.json`/`.ptx`/`.cu`）：`docs/development-evidence/np02-evidence/missing-capture/receipts/`。
- 补采索引与逐项字段（含 `task_id`）：`docs/development-evidence/np02-evidence/receipt.json`；扫描器把该索引并入主索引复核（701 份）。
- 原始捕获目录保留在 `target/np02-capture/final`（未入库，`target` 已忽略）。

## 身份判定口径（重要）

- 编译源哈希按 **LF 规范化** 计算：rustc 读取源码时把 CRLF 规范化为 LF（字符串字面量同样生效），工作区因 `core.autocrlf` 检出为 CRLF；`sparse_solve_*`/`resident_sparse_*` 因此与回执 `sourceSha256` 直接相等。
- 参数化/拼接编译单元（`passive_force_add_fluid`、`fluid_force_clear`、`mass_multiply*`）的编译源含 `#define` 前导或多源拼接，回执 `sourceSha256` 不等于单条 const 哈希；判定要求捕获 `.cu` 实体哈希 = 回执 `sourceSha256` **且** 完整包含当前 const 源文本。
- 该口径同时纠正了上一轮清单中 29 条 CRLF 假阳性漂移（38 → 9 条真实漂移），详见 `np02-evidence/README.md`。

## 边界

- 只补采缺失 8 条；未改产品源码/测试/容差/fixtures；未跑全量 G01；GPU 仅用于上述三次定向测试（约 34 秒）。
- 不宣称 NP02 验收完成、不宣称双平台；9 条真实漂移与 51 份历史实体不在仓继续按原口径登记。
