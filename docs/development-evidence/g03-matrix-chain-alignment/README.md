# G03矩阵链非收缩与求解算序对齐

任务：`2026-10-22-g03-matrix-chain-alignment`（前序登记 `97e4c69`；被放弃任务 `2026-10-22-g03-kinematics-matrix-fma-alignment`）。
依据：`2026-10-22-g03-upstream-matrix-authoring`（`ad6c2e7`）——上游矩阵冻结参考与 MathDx potrf 天然校验 0/121 已成立；
首差定位第一阶段差异在 `kinematics.xmat`，按首差循环归零过程中确认残余收缩还位于 mass-matrix 与 mass-solve 模块。

## 结论

三处 f32 算序已对齐冻结上游，导出链逐位一致：

| 验证项 | 结果 |
| --- | --- |
| 离线首差工具（xpos/xquat/xmat/xipos/ximat/xanchor/xaxis/subtree_com/cinert/crb/cdof/matrix_dense_block） | `first_stage=None`，全部 0 位差 |
| MathDx potrf(产品矩阵) vs 冻结上游 factor | `status=0`，**0/121** 位差，max_abs 0 |
| 产品 GPU 解 vs 冻结上游 solution（case4/world4 三条 rhs） | **0/33** 位差，max_abs 0 |
| `tests/mass_solve_probe`（all-features，ignored，单线程） | 6 过 0 败；NP03 汇总 factors 240 / native-snapshots 240 / rhs-counts 152 全部 `failed=0` |

三项 NP03 准入缺口（case4/world4 factors、native-snapshots、rhs-counts）已全部转绿。

## 实现改动

| 文件 | 改动 |
| --- | --- |
| `src/physics/mod.rs` | `KINEMATICS_CUDA` 增加 `umul/uadd/usub/ufma` 显式 PTX 助手与 `matrix_from`：四元数乘积先落局部、再按上游 `_compute_body_matrices` 的非收缩顺序组装 `xmat`；`ximat` 保持与上游 inertial_frames 一致的自然写法。 |
| `src/physics/mass_matrix.rs` | `MASS_MATRIX_CUDA` 的 `rigid_mass_matrix` 按上游 `_M` 的 mul/fma/sub 顺序逐项转录：惯量向量 6 分量、点积链式 fma、`j==d` 主对角加法；对称写入不变。 |
| `src/physics/mass_solve/cuda.rs` | tile 块（>6 维）求解改为上游 MathDx 列扫描：先 forward `U^T y=b`、再 backward `U x=y`；对角 `rcp.rn` 倒数乘，离对角 `fma.rn` 单步更新；≤6 维标量块保留既有路径（其原顺序已与上游一致）。 |

不改 ABI、类型、容差、准入余量、测试断言与 fixtures。

## 算序反推依据

离线工具 `target/g03-precheck-dump`（临时分析包，非产品代码）以冻结上游 factor + 冻结 rhs + 冻结上游 solution 做全组合枚举，
唯一 `diffs=0` 组合为：列扫描、先 forward 后 backward、更新融合为 fma、对角取倒数后乘。
产品内核据此实现后，GPU 解与冻结上游解逐位一致（0/33），验证了反推口径。

## T4 源码身份影响

产品源码变化导致 T4 固定源码身份到期（`lib` 两项目录重采集测试报 `unmapped pre-rotation variant` / retained 漂移）。
按 Chief 指示另立 `2026-10-22-t4-source-hash-repin-6`（只改哈希数值与内联字面量），完成后跑全量非 ignored 回归与 G01 debug 全脚本；
相关结果记入 repin-6 证据，不在本任务宣称全量通过。

## 复现命令

```powershell
# 离线链（产品 GPU 只经 cargo；无 Python）
cargo +1.99.0-x86_64-pc-windows-msvc run --offline --manifest-path target/g03-precheck-dump/Cargo.toml --bin first_difference -- <repo> docs/development-evidence/g03-upstream-matrix/upstream-matrix.json docs/development-evidence/g03-matrix-chain-alignment/first-difference-after-fix.json
cargo +1.99.0-x86_64-pc-windows-msvc run --offline --manifest-path target/g03-precheck-dump/Cargo.toml --bin dump_matrix -- <repo> target/g03-fma/product-matrix.f32
target/mathdx11/oracle.exe target/mathdx11/out/factor-cdd46856c1496194/potrf_11_11_1_89_32_1_1_5_x_x_0.cubin factor target/g03-fma/product-matrix.f32 target/g03-fma/potrf-product.out mathdx11_factor
cargo +1.99.0-x86_64-pc-windows-msvc run --offline --manifest-path target/g03-precheck-dump/Cargo.toml --bin verify_factor -- docs/development-evidence/g03-upstream-matrix/upstream-matrix.json target/g03-fma/potrf-product.out
cargo +1.99.0-x86_64-pc-windows-msvc run --offline --manifest-path target/g03-precheck-dump/Cargo.toml --bin dump_solution -- <repo> target/g03-fma/product-solution-world4.f32
# 准入
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --test mass_solve_probe -- --ignored --test-threads=1
```

## 未验收边界

- 未跑全量非 ignored 回归、G01 全脚本、release 与 Linux：T4 源码身份重钉（repin-6）完成后执行。
- 未宣称 NP01–NP04、精度契约或性能验收通过；离线 authoring 只复用已批准 pinned venv 冻结产物，不进产品链。
