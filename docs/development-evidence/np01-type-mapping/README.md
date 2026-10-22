# NP01 精度类型映射清单（产品侧）

任务：`2026-10-23-precision-type-mapping`。对象：`docs/numerical-precision-contract.md` 的 NP01 门禁（类型映射 100%、精度偏离 0）。
日期：2026-10-23。只做证据/文档：未改产品代码、测试、容差与 fixtures；未跑 GPU；不做 NP02–NP04。

## 结论

- 已建 **30 条产品路径**的逐路径/逐内核类型映射清单（`inventory.json`），每条标「已有证据 / 待补」。
- 现状：**20 条部分**（已有探针或逐位证据，缺逐字段类型表）、**10 条待补**（缺证据或上游定位）。
- 产品 CUDA 源（0 处 `double`）当前为全 f32；库级 `sinf/cosf` 慢路径的 f64 归约属契约已登记例外，按单列处理。
- **NP01 仍未验收**：本清单是输入与缺口登记，不是通过结论。完整类型映射率 100%、偏离归零尚未达成。
- 上游映射基于 pinned checkout `target/mjwarp-parallel/upstream/mujoco_warp/_src`（HEAD=`71da24d…`）；函数级已核对者标 `confidence=函数级`，其余为文件级待补。W15 落地后补「上游源码复制」路径。

## 方法

1. `extract-kernels.mjs` 枚举 `src/physics/**/*.rs|cu` 中全部 `__global__` 内核签名，统计参数类型与 `double`/`fma`/`atomicAdd`/`__shared__`，输出 `raw-kernels.json`（49 个含内核文件、产品域 0 处 double）。
2. 编译配方取 `src/runtime/launcher/recipe.rs` 的 `warp-1.15-physics`：`fmad=true`、`fast-math=false`、`--std=c++17`、`compute_70`、f32 物理检查。
3. 冻结 PTX 三条单列：`src/runtime/warp_kinematics.rs`、`warp_com.rs`（PTX_SHA256 `4179acd0…`）、`warp_gravity.rs`（`d06154da…`）。
4. 上游函数用 pinned checkout 逐个 grep 核对；未能函数级定位的标「文件级」并列入待补。
5. 「已有证据」只列可复核路径/命令；「待补」写清缺什么，不把待补写成通过。

## 清单摘要

| # | 产品路径 | 类型要点 | 上游（71da24d…） | 状态 |
| --- | --- | --- | --- | --- |
| 1 | xpos / xquat / xmat / xipos / ximat / xanchor / xaxis / subtree_com / cinert / crb / cdof / matrix_dense_block | f32（const float*：qpos/qvel、body/geom 参数、树结构常量）；中间/累加/输出：f32；double 0 处 double | smooth.py：_compute_body_matrices、_kinematics_branch、_site_local_to_global | 部分（探针逐位证据齐；缺逐字段表） |
| 2 | com_pos | f32（质量、xipos、xmat 等）；中间/累加/输出：f32；double 0 处 double | smooth.py：com_pos | 部分 |
| 3 | com_vel / cdof_dot | f32（xpos/xquat/xmat、qvel、cdof 等）；中间/累加/输出：f32；double 0 处 double | smooth.py：com_vel | 部分 |
| 4 | crb / cinert 组合 | f32（惯量 10 分量、xmat/xipos、body 参数）；中间/累加/输出：f32；double 0 处 double | smooth.py：crb、_crb_accumulate | 部分 |
| 5 | mass_multiply / borrowed_mass_multiply / resident 乘 | f32（M、向量、累加器）；中间/累加/输出：f32；double 0 处 double | smooth.py：（文件级映射；具体点积/矩阵函数待逐函数核对） | 部分 |
| 6 | rigid_mass_solve / sparse dense/small/mixed/resident / substitute | f32（矩阵、因子、rhs）；中间/累加/输出：f32；double 0 处 double | math.py, block_cholesky.py, smooth.py：solve_LD、cholesky/tile 求解（MathDx cuSOLVER 分支） | 部分（数值准入已过；缺逐字段表） |
| 7 | rne_bias | f32（cdof、qvel、cacc、惯量等）；中间/累加/输出：f32；double 0 处 double | smooth.py：rne | 部分 |
| 8 | rne_cacc / rne_cfrc / rne_backward / rne_qfrc | f32（惯量、cdof、qvel、qacc）；中间/累加/输出：f32；double 0 处 double | smooth.py：rne、_rne_cacc_world | 部分（数值口径已裁决；缺逐字段表） |
| 9 | gravity_compensation | f32（rigid/com、raw 参数、gravity）；中间/累加/输出：f32；double 0 处 double | passive.py, smooth.py：_gravity_force | 部分 |
| 10 | joint_passive | f32（qpos/qvel、5 项参数、周期）；中间/累加/输出：f32；double 0 处 double | passive.py：（文件级映射） | 部分 |
| 11 | smooth_force | f32；中间/累加/输出：f32；double 0 处 double | passive.py：（文件级映射） | 部分 |
| 12 | passive_force_clear/sum/gravity/fluid | f32；中间/累加/输出：f32；double 0 处 double | passive.py：_passive（文件级） | 待补 |
| 13 | fixed_tendon | f32；中间/累加/输出：f32；double 0 处 double | smooth.py：_tendon_armature、transmission | 待补 |
| 14 | spatial_site / spatial_moment / spatial_wrap | f32；中间/累加/输出：f32；double 0 处 double | smooth.py, util_misc.py：transmission、（wrap 路径见 util_misc） | 待补 |
| 15 | tendon_wrap（26 个内联 kernel） | f32（局部投影、牛顿迭代）；中间/累加/输出：f32；double 0 处 double（但库级 sinf/cosf 慢路径属已知例外，见契约） | util_misc.py：（wrap 数学路径） | 部分 |
| 16 | tendon_values / tendon_indices | f32；中间/累加/输出：f32；double 0 处 double | smooth.py：transmission | 待补 |
| 17 | tendon_velocity_validate/compute | f32；中间/累加/输出：f32；double 0 处 double | passive.py, smooth.py：（文件级映射） | 部分 |
| 18 | tendon_passive_validate/clear/compute | f32；中间/累加/输出：f32；double 0 处 double | passive.py：（文件级映射） | 待补 |
| 19 | flex_positions / flex_invalidate | f32；中间/累加/输出：f32；double 0 处 double | smooth.py：_flex_vertices | 部分 |
| 20 | flex_edges | f32；中间/累加/输出：f32；double 0 处 double | smooth.py：_flex_edges | 部分 |
| 21 | flex_faces | f32；中间/累加/输出：f32；double 0 处 double | smooth.py：_flex_face_kinematics | 部分 |
| 22 | flex_hessian | f32；中间/累加/输出：f32；double 0 处 double | smooth.py, math.py：（文件级映射） | 待补 |
| 23 | flex_edge_passive_validate/clear/compute | f32；中间/累加/输出：f32；double 0 处 double | passive.py：_flex_gather_kernel | 待补 |
| 24 | fluid_force_validate/body/project/clear | f32；中间/累加/输出：f32；double 0 处 double | passive.py, derivative.py：_fluid_force | 待补 |
| 25 | applied_force_validate/delta/publish | f32；中间/累加/输出：f32；double 0 处 double | passive.py, smooth.py：_apply_face_forces（文件级） | 部分 |
| 26 | kinetic_energy_validate/reduce32 / device_energy / borrowed multiply energy | f32；中间/累加/输出：f32；double 0 处 double | math.py, smooth.py：（文件级映射） | 部分 |
| 27 | attached_kinematics / initialize_attached | f32；中间/累加/输出：f32；double 0 处 double | smooth.py：_site_local_to_global | 部分 |
| 28 | camlight_rigid / camlight_com | f32；中间/累加/输出：f32；double 0 处 double | smooth.py：_light_local_to_global | 待补 |
| 29 | sleep_copy / tendon_wake | f32/int（树标记、树计数、周期）；中间/累加/输出：f32；double 0 处 double | sleep.py：_sleep_cycle、_wake_tree | 部分 |
| 30 | 字段批打包 | f32/整型字段批；中间/累加/输出：f32；double 0 处 double | io.py, types.py：（文件级映射） | 待补 |

（逐条明细、内核名、证据路径见 `inventory.json` 的 `rows` 字段。）

## 缺口优先级

1. **P0：10 条待补路径**（如 passive_force、tendon、tendon_passive、flex_hessian、fluid_force、camlight、field_batch 等）——逐字段类型表 + 上游函数定位。
2. **P1：20 条部分路径的逐字段表**——每个输入的量化位置、中间量、累加类型与输出转换点；已有探针证据的路径优先（kinematics、mass_solve、rne_acc、gravity）。
3. **P2：冻结 PTX 与手写源逐字段对照**——三条冻结 PTX 摘要已知，缺逐字段类型/精度对照。
4. **P3：W15 落地后补上游源码复制路径**——`tile*.h` 等条件复制文件与逐字段 f32 映射；本任务不依赖 W15。
5. **例外单列**：库级 `sinf/cosf` 慢路径的 f64 归约（契约已登记），不得当作产品 f64。

## 证据判读

- `inventory.json`：清单主体（`rows` 每条含 productFiles/constants/kernels/types/upstream/evidence/status）。
- `raw-kernels.json`：机械采集结果（内核签名参数统计、配方、冻结 PTX 摘要），用于复核清单。
- `extract-kernels.mjs`：只读复现脚本，输出 `raw-kernels.json`。
- 本清单只覆盖**产品侧**；上游源码复制路径（W15）与 NP02–NP04 不在本任务。

## 复现

```powershell
node docs/development-evidence/np01-type-mapping/extract-kernels.mjs
```

## 边界

- 不宣称 NP01–NP04 通过；不宣称完整产品精度验收。
- 未改产品代码、断言、容差、fixtures；未运行 GPU；未改 `docs/compatibility-matrix.md`（正被 Backend 的 U038-3D 占用，待其关闭后由后续文档任务刷新）。
- 与 Backend 的 W15 范围（`src/runtime/launcher`、`include/warp-headers`、`LICENSES`）不重叠。
