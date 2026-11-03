# NP01 精度类型映射清单（产品侧）

任务：`2026-10-23-precision-type-mapping`。对象：`docs/numerical-precision-contract.md` 的 NP01 门禁（类型映射 100%、精度偏离 0）。
日期：2026-10-23。只做证据/文档：未改产品代码、测试、容差与 fixtures；未跑 GPU；不做 NP02–NP04。

## 结论

- 已建 **30 条产品路径**的逐路径/逐内核类型映射清单（`inventory.json`），每条标「已有证据 / 待补」。
- 现状（2026-10-23 W15 路径补全后）：**29 条部分**（已有探针/逐位或头文件路径证据，缺逐字段类型表）、**1 条待补**（`tendon`，纯索引合并，头树无直接函数对应）。
- 产品 CUDA 源（0 处 `double`）当前为全 f32；库级 `sinf/cosf` 慢路径的 f64 归约属契约已登记例外，按单列处理。
- **NP01 仍未验收**：本清单是输入与缺口登记，不是通过结论。完整类型映射率 100%、偏离归零尚未达成。
- 2026-10-23 更新（`2026-10-23-precision-source-paths`）：W15 头文件原样复制（`include/warp-headers/`，36 个）已落地，已补「产品内核 → 上游 Warp 头文件实现」路径证据；**待补 10 → 1**（余 `tendon`），30 条仍全部缺逐字段类型表，见文末「W15 头文件路径补全」。
- 上游映射基于 pinned checkout `target/mjwarp-parallel/upstream/mujoco_warp/_src`（HEAD=`71da24d…`）；函数级已核对者标 `confidence=函数级`，其余为文件级待补。上游头文件实现路径见 `warp-headers.json`。

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
| 12 | passive_force_clear/sum/gravity/fluid | f32；中间/累加/输出：f32；double 0 处 double | passive.py：_passive（文件级） | 部分（W15 头文件路径已对；上游函数定位与逐字段表仍缺） |
| 13 | fixed_tendon | f32；中间/累加/输出：f32；double 0 处 double | smooth.py：_tendon_armature、transmission | 部分（W15 头文件路径已对；缺逐字段表） |
| 14 | spatial_site / spatial_moment / spatial_wrap | f32；中间/累加/输出：f32；double 0 处 double | smooth.py, util_misc.py：transmission、（wrap 路径见 util_misc） | 部分（W15 头文件路径已对；缺逐字段表） |
| 15 | tendon_wrap（26 个内联 kernel） | f32（局部投影、牛顿迭代）；中间/累加/输出：f32；double 0 处 double（但库级 sinf/cosf 慢路径属已知例外，见契约） | util_misc.py：（wrap 数学路径） | 部分 |
| 16 | tendon_values / tendon_indices | f32；中间/累加/输出：f32；double 0 处 double | smooth.py：transmission | 待补 |
| 17 | tendon_velocity_validate/compute | f32；中间/累加/输出：f32；double 0 处 double | passive.py, smooth.py：（文件级映射） | 部分 |
| 18 | tendon_passive_validate/clear/compute | f32；中间/累加/输出：f32；double 0 处 double | passive.py：（文件级映射） | 部分（W15 头文件路径已对；上游函数定位与逐字段表仍缺） |
| 19 | flex_positions / flex_invalidate | f32；中间/累加/输出：f32；double 0 处 double | smooth.py：_flex_vertices | 部分 |
| 20 | flex_edges | f32；中间/累加/输出：f32；double 0 处 double | smooth.py：_flex_edges | 部分 |
| 21 | flex_faces | f32；中间/累加/输出：f32；double 0 处 double | smooth.py：_flex_face_kinematics | 部分 |
| 22 | flex_hessian | f32；中间/累加/输出：f32；double 0 处 double | smooth.py, math.py：（文件级映射） | 部分（W15 头文件路径已对；上游函数定位与逐字段表仍缺） |
| 23 | flex_edge_passive_validate/clear/compute | f32；中间/累加/输出：f32；double 0 处 double | passive.py：_flex_gather_kernel | 部分（W15 头文件路径已对；缺逐字段表） |
| 24 | fluid_force_validate/body/project/clear | f32；中间/累加/输出：f32；double 0 处 double | passive.py, derivative.py：_fluid_force | 部分（W15 头文件路径已对；缺逐字段表） |
| 25 | applied_force_validate/delta/publish | f32；中间/累加/输出：f32；double 0 处 double | passive.py, smooth.py：_apply_face_forces（文件级） | 部分 |
| 26 | kinetic_energy_validate/reduce32 / device_energy / borrowed multiply energy | f32；中间/累加/输出：f32；double 0 处 double | math.py, smooth.py：（文件级映射） | 部分 |
| 27 | attached_kinematics / initialize_attached | f32；中间/累加/输出：f32；double 0 处 double | smooth.py：_site_local_to_global | 部分 |
| 28 | camlight_rigid / camlight_com | f32；中间/累加/输出：f32；double 0 处 double | smooth.py：_light_local_to_global | 部分（W15 头文件路径已对；缺逐字段表） |
| 29 | sleep_copy / tendon_wake | f32/int（树标记、树计数、周期）；中间/累加/输出：f32；double 0 处 double | sleep.py：_sleep_cycle、_wake_tree | 部分 |
| 30 | 字段批打包 | f32/整型字段批；中间/累加/输出：f32；double 0 处 double | io.py, types.py：（文件级映射） | 部分（W15 头文件路径已对；上游函数定位与逐字段表仍缺） |

（逐条明细、内核名、证据路径见 `inventory.json` 的 `rows` 字段。）

## 缺口优先级

1. **P0：1 条待补路径（`tendon`）**——纯索引合并内核，头树无直接函数对应；补逐字段类型表与上游函数定位。另有 9 条由待补升级为部分（头文件路径已对），其上游函数定位/逐字段表仍缺。
2. **P1：29 条部分路径的逐字段表**——每个输入的量化位置、中间量、累加类型与输出转换点；已有探针证据的路径优先（kinematics、mass_solve、rne_acc、gravity）。升级后的 9 条也在此列。
3. **P2：冻结 PTX 与手写源逐字段对照**——三条冻结 PTX 摘要已知，缺逐字段类型/精度对照。
4. **P3：上游源码路径已补**（W15 头文件，见文末）；剩余为逐字段 f32 映射与「上游生成内核/编译器库」覆盖（NP02 范围）。
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

## W15 头文件路径补全（2026-10-23）

任务：`2026-10-23-precision-source-paths`。依据：W15 `b1de2db` 把 Warp v1.15.0（clone `ffa038b1`）36 个头文件原样复制到 `include/warp-headers/`，逐字节清单见 `docs/development-evidence/w15-verbatim-headers/headers.json`。

### 方法

- 上游侧：从 pinned checkout（`target/mjwarp-parallel/upstream/mujoco_warp/_src`，HEAD=`71da24d…`）读对应文件；优先按 `upstream.functions` 抽函数体提取 `wp.*` 调用（函数级，19 行），函数名不可定位时退回整文件（文件级，11 行）并如实标注。
- 产品侧：从 `productFiles`（含 `**`）读手写 CUDA 源，提取 Warp 原语/数学原语（quat/mat/vec/spatial/tile/atomic/math）映射到头文件定义。
- 头文件侧：在 36 个文件中按定义行（`inline`/`CUDA_CALLABLE`/`using`/`struct`/`class`）定位符号，记录文件与行号；`mesh.h`/`intersect.h` 中的使用点不会被当成定义。
- CUDA 内建不冒充 Warp 头树实现：标量 `atomicAdd` 映射到 Warp `atomic_add` 重载族；`__fadd_rn`/`__fmul_rn`/`__fmaf_rn`/`__syncthreads` 不登记为 Header 实现，只在 `warp-headers.json` 的检出项里保留。
- 输出：`warp-headers.json`（逐行头文件/符号/行号/来源/函数级或文件级），并回写 `inventory.json` 的 `warpHeaders`、`warpHeaderCoverage` 与 `evidence` 两栏。

### 结果

- 30 条路径中 **28 条**得到头文件路径证据，涉及 9 个头文件：`array.h`、`builtin.h`、`cuda_crt.h`、`mat.h`、`quat.h`、`spatial.h`、`tile.h`、`tile_cholesky.h`、`vec.h`。
- **补齐 9 条**：由「待补」升级为「部分」（头文件路径已对）；**剩余待补 1 条**：`tendon`（上游 `transmission` 只检出运行时 API，产品为纯索引合并，头树无直接函数对应）。
- `sleep_wake` 无头文件证据（上游 `_sleep_cycle`/`_wake_tree` 只调用运行时 API、产品为整型搬运），保留原「部分」状态并在 `warpHeaderCoverage.note` 注明原因。
- **30 条仍全部缺逐字段类型表**（NP01 未验收）；本次只补「上游实现路径」，不改变类型映射结论。

### 原 10 条待补的前后变化

| 路径 | 原状态 | 新状态 | 头文件数 | 上游证据级别 |
| --- | --- | --- | --- | --- |
| passive_force | 待补 | 部分（W15 路径已对；上游函数定位与逐字段表仍缺） | 6 | 文件级 |
| fixed_tendon | 待补 | 部分（W15 路径已对；缺逐字段表） | 1 | 函数级 |
| spatial_tendon | 待补 | 部分（W15 路径已对；缺逐字段表） | 3 | 函数级 |
| tendon | 待补 | 待补（头树无直接函数对应） | 0 | — |
| tendon_passive | 待补 | 部分（W15 路径已对；上游函数定位与逐字段表仍缺） | 5 | 文件级 |
| flex_hessian | 待补 | 部分（W15 路径已对；上游函数定位与逐字段表仍缺） | 8 | 文件级 |
| flex_edge_passive | 待补 | 部分（W15 路径已对；缺逐字段表） | 2 | 函数级 |
| fluid_force | 待补 | 部分（W15 路径已对；缺逐字段表） | 7 | 函数级 |
| camlight | 待补 | 部分（W15 路径已对；缺逐字段表） | 3 | 函数级 |
| field_batch | 待补 | 部分（W15 路径已对；上游函数定位与逐字段表仍缺） | 4 | 文件级 |

### 文件与复现

- `warp-headers.json`：逐行头文件路径/符号/行号/来源/证据级别；
- `inventory.json`：已回写 `warpHeaders`、`warpHeaderCoverage` 与 `evidence.existing/missing`；
- `map-warp-headers.mjs`：只读复现脚本。

```powershell
node docs/development-evidence/np01-type-mapping/map-warp-headers.mjs
```

### 边界

- 这是**路径级证据**，不是逐字段类型映射；30 条路径的逐字段表、偏离归零仍未完成，不宣称 NP01 通过。
- 文件级行（11 条）的头文件证据是文件范围证据，上游具体函数定位仍标待补；`tendon` 仍待补。
- 未改产品代码、测试、容差、fixtures；未跑 GPU；未改 `include/warp-headers`（只读）。


## host 站点口径与 gjk/ccd/epa host 覆盖（2026-10-26）

任务 `2026-10-25-g10-p5-faithful-fma-fix-r2`（用户拍板 A：数据向最可靠的依据靠拢）后补登记，
**只登记口径，不改判据/容差/验收范围**。`inventory.json` 新增 `hostSiteRegistration`；
原 30 条产品 CUDA 内核行的分类与计数不变。

站点语义（host 侧乘加融合口径）：

| 站点 | host 语义 | 参考 |
| --- | --- | --- |
| `gjk_ccd_epa_host`（`src/physics/collision/gjk/`） | **非融合**：乘加独立、两次舍入；`dot(r,r)/length/normalize` 不做 FMA 收缩 | warp 1.15.0 CPU 后端 f32（`device:cpu`；生成 C++ mul/add 独立、`.o` FMA=0） |
| `render_light_dir_normalize`（`src/render/kernel/megakernel_arrays.rs`） | **fused**：精确 f32 FMA helper | render S7 参考（GPU `use_fast_math=true`） |
| `relmat_relative_matrix`（`src/physics/collision/relmat.rs`） | **fused**：精确 f32 FMA helper | warp `mat.h` `muladd→fmaf` |

device 侧口径：pinned GPU `ccd` PTX 的 `fma.rn` 融合（T4 采集 PTX `fma.rn.f32=1480`、`mul.rn/add.rn=0`），
不属 host 判据；两套口径并存，不宣称 host 与 device 数值等价。

补 3 条 **host 路径覆盖项**（原 30 条 CUDA 内核路径未覆盖）：`gjk_host`、`ccd_host`、`epa_host`；
状态如实为「部分（站点口径已登记；逐字段输入→输出类型表/逐表达式位级对照待补）」——**NP01 仍未验收**。
实现与三构建证据见 `docs/development-evidence/faithful-fma-fix/README.md`。

## NP01 门禁第一片：CUDA 源面快照 + 精度 token 断言（2026-11-03）

任务：`2026-11-03-precision-np01-cuda-type-gate`（CPU-only、零窗口）。本片只把 NP01 的**可判定部分**机械化，
不宣称 NP01 通过（逐字段类型表仍未补，见上面「缺口优先级」）。

门禁实现：`tests/precision_type_gate.rs`（3 个用例，随 `cargo test --locked` 执行）。

1. **源面快照棘轮**：`cuda-surface.json` 记录当前 42 个产品 `.cu` 文件与每个文件的 `__global__` 入口数
   （合计 **22**）。新增/删除 `.cu` 或入口数变化都会让测试失败，必须由人显式更新快照并同步本清单，
   防止新内核绕过类型登记。
2. **精度 token 断言**：产品 `.cu` 源的**代码区**（去掉行注释与块注释后）不得出现
   `double` / `__half` / `half2` / `bfloat16` / `tf32`。当前 8 处命中全部落在注释（讨论上游 double 分支、
   `1.0/6.0` 舍入说明）或 W15 守卫宏 `#define WP_NO_BFLOAT16`，按规则放行；代码里再出现即失败。
3. **清单形状断言**：`inventory.json` 至少 30 行，每行的 `types.{input,intermediate,accumulation,output,doubleInProduct}`
   非空、`productFiles` 非空；`hostSiteRegistration.hostCoverage` 至少 3 行（gjk/ccd/epa host）且同样带 5 个类型键，
   `siteSemantics` 至少 3 条。删行/删口径会失败。

**本片未覆盖（仍属 NP01 未验收项）**：

- `.rs` 内联 CUDA 源字符串（`KINEMATICS_CUDA` 等 50+ 处）的精度 token 扫描——下一片做；
- 30 条产品路径 + 3 条 host 路径的**逐字段类型表**（输入量化位置、中间量、累加类型、输出转换点）；
- 冻结 PTX 与手写源的逐字段对照；
- 任何数值容差/GPU 行为（本片零 GPU）。

复现：`cargo test --locked --test precision_type_gate`。
## NP01 门禁第二片：宿主 Rust 源精度 token + 允许清单活性（2026-11-04）

任务：`2026-11-03-precision-np01-host-source-gate`（CPU-only、零窗口）。第一片只覆盖产品 `.cu` 源面，
本片补宿主 Rust 侧，两片合起来覆盖「代码里出现 C 风格/设备宽度精度 token」这一类泄漏。

实现：`tests/precision_host_source_gate.rs`（3 个用例，随 `cargo test --locked` 执行）。

1. **宿主 token 断言**：`src/**/*.rs` 的代码区（剥掉 `//` 与 `/* */`）不得出现
   `double` / `__half` / `half2` / `bfloat16` / `tf32`。Rust 的 `f64` **不在**本门禁范围——
   契约允许参考/诊断与原生 ABI（`mjtNum`）转换使用 f64。当前全仓代码区命中 **5 处**，
   全部落在允许清单内。
2. **允许清单活性**：唯一放行的是精度门禁自身的 token 表（`src/render/kernel/mod.rs` 1 处、
   `src/runtime/transfer/kernel/ptx.rs` 1 处）与刻意注入禁用 token 的负对照测试
   （`tests/adapter.rs` 1 处、`tests/precision.rs` 2 处）。每个条目必须在当前源里仍有 ≥登记数命中，
   否则清单腐坏即失败。
3. **跨片一致性**：断言第一片的 `cuda-surface.json` 仍在且仍是 42 文件 / 22 入口，
   防止只删一片就以为覆盖完整。

**本片未覆盖**：`f64` 语义审计（哪些 f64 会反馈产品物理）、逐字段类型表、数值容差；不宣称 NP01 通过。

复现：`cargo test --locked --test precision_host_source_gate`。
## B1 逐字段表已交付（5/30 条产品路径，2026-11-04）

任务：`2026-11-03-precision-np01-field-tables`（B1 批）。在 `field-tables/` 下新增 5 条产品路径的逐字段类型表
（参数/中间量/累加/输出转换点/上游/证据）：`com_position`、`com_velocity`、`gravity_compensation`、
`rne_bias`、`rne_acc`。表结构、粒度边界、待补 25 条产品路径与 3 条 host 站点的分批计划见
[`field-tables/README.md`](field-tables/README.md)；`tests/precision_type_gate.rs` 新增覆盖门禁：
清单每行必须在 `field-tables/` 有表，或在测试的待批清单里显式登记，缺表即失败。

**NP01 仍未验收**：B1 只覆盖 5/30 条产品路径；逐表达式位级对照、`kinematics`（12 输出）与 mass_solve 多路径、
host 三条站点表仍未做。