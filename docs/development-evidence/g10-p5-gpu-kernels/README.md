# G5 碰撞 GPU 内核（增量提交 · 第一批设备公共层 + 第二批 2a：GJK 距离/主循环）

任务：`2026-10-25-g10-p5-gpu-kernels`（范围 `src/physics/`、`src/runtime/`、`native/`、`include/`、
`build.rs`、`tests/g10_p5_gpu_kernels.rs`、本证据目录；资源 `gpu:windows-t4` 独占）。
上游基线 `71da24d956378a87a703b6e1442b13aec0c4ac29`。**本批无 GPU 运行**，只做设备源码/接线/CPU 侧编译验证。

## 1. 本批范围（增量提交 1/5）

| 文件 | 内容 |
| --- | --- |
| `src/physics/collision/cuda/mod.rs`（新） | 设备源码持有（`include_str!`）与两套编译模式约定 |
| `src/physics/collision/cuda/common.cu`（新） | 与 host `gjk/math.rs` 逐式对应的设备数学（dot/cross/length/normalize/sign/cw_mul/scale/neg、`mul_mat3_vec`、`transpose_mul_mat3_vec`、`relative_matrix`、`make_frame`） |
| `src/physics/collision/cuda/geom.cu`（新） | 设备几何视图 `d_geom`/`d_mesh`/`d_support_point`（对应 host `Geom`/`MeshView` 字段） |
| `src/physics/collision/cuda/support.cu`（新） | 支撑点设备版，对应 host `gjk/support.rs`：SPHERE/CAPSULE/ELLIPSOID/CYLINDER/BOX、MESH（穷举 + graph hillclimb + 热启动缓存）、HFIELD 六顶点（`dir.z<0?-2:-3`）、TRIANGLE、FLEX、margin 尾项、`support_pair`、`discrete_geoms` |
| `src/physics/collision.rs` | `#[cfg(feature="cuda-probe")] #[allow(dead_code)] pub mod cuda;` 接线 |

## 2. 编译模式定性（不得混用）

- **调试模式、非判据**：NVRTC `--fmad=false` 时设备表达式与 host 源码顺序逐位可对照，
  只用于定位「host→device 搬运偏差」；**不得写成 GPU 判定通过**。
- **正式判据**：对同配方上游 f32 GPU，按 pinned `ccd` PTX 的**全 fma 收缩**（显式 `mul_add`）编译，
  ≤4 ULP、位级优先；该判据在最终 GPU 窗口批才执行。
- 两套数字在证据中分开记录。

## 3. 本批验证（CPU 侧，无 GPU 计算）

- `nvcc -std=c++17 -Xcompiler /utf-8 -arch=sm_89 -c support.cu/geom.cu/common.cu`：通过
  （仅编译，无 GPU 上下文；源文件含中文注释，MSVC 宿主 / NVRTC 均按 UTF-8 处理）。
- `cargo check --locked --features cuda-probe --lib`：通过。
- `cargo clippy --locked --all-targets --features cuda-probe -- -D warnings`：通过。
- `normify gate 2026-10-25-g10-p5-gpu-kernels`：fmt/clippy/clippy-cuda/unit 全过（unit 以门禁记录为准）。
- 未启动任何 CUDA kernel、未占用 GPU 窗口。

## 4. 后续批次（2/3/4 已提交，剩余 5/6）

2. 设备 GJK phase/EPA（已提交 `polytope.cu`/`polytope_build.cu`/`epa.cu`/`ccd.cu`；宿主 launcher 归窗口阶段）；
3. SDF narrowphase 设备版（已提交 `sdf_octree.cu`/`sdf_math.cu`/`sdf_narrowphase.cu`）；
4. hfield prism CCD 设备版（已提交 `hfield.cu`，见 §9）；
5. 接触写入（`worldid`/顺序/`efc_address`/`OverflowType`/`naconmax`·`naccdmax` 截断）+ `nworld>1` world 维上传；
6. GPU 窗口对照（先报窗口：显存基线、预计时长、迭代次数、跑什么；跑完报回桌面基线 + 无本任务计算进程）。

## 5. 未覆盖 / unmet（全片）

- 设备 GJK/EPA、SDF、hfield 内核与宿主 launcher 尚未接入；GPU 对照未执行（无窗口记录）。
- `naconmax`/`naccdmax` 截断、`efc_address`、`OverflowType`、`nworld>1` 上传未实装。
- 正式判据（上游 f32 GPU ≤4 ULP）**尚未测量**，本批不构成任何精度结论；
  host 逐位对照只是调试手段。
- GPU 结果不外推性能/跨硬件结论（性能归 G6）；Linux/T4 双平台未验证。
- 禁止措辞：不写「原生放行」「精度达标」「上游完全等价」「完整物理通过」「双平台通过」。

## 6. 第二批 2b（增量提交 2/5）：设备 GJK phase / EPA / ccd

| 文件 | 内容 |
| --- | --- |
| `src/physics/collision/cuda/polytope.cu`（新） | `d_polytope`（调用方指针 + `max_face`/`max_horizon` 容量）、face 位域与 delete/invalidate 判决、`_attach_face`（`fmaf` 链对齐 pinned PTX）、`_epa_support`、`_rotmat`、`_ray_triangle`、`_replace_simplex3`、`_epa_witness`（含 hfield witness 修正） |
| `src/physics/collision/cuda/polytope_build.cu`（新） | `_polytope2/_polytope3/_polytope4` 与 host 状态码（-1/0/1/2/3–12）一一对应 |
| `src/physics/collision/cuda/epa.cu`（新） | `_get_edge`/`_add_edge`（重复边交换、horizon 容量 -1）、`_epa`（最近面下界、`upper_k=dot(face_pr,w)/lower`、`upper-lower<epsilon`、离散重复支撑点早退、`dist2∈[lower2,upper2]` 有效性、`OverflowType.EPA_HORIZON=1<<8`、witness 收尾） |
| `src/physics/collision/cuda/ccd.cu`（新） | `_inflate`（hfield side + margin 膨胀）、`gjk_phase`（球/胶囊 shrink、浅/深穿透两段）、`epa_phase`（dim 2/4 回退、`status!=0` 边界不穿透、`multiccd_idx` 规则）、`ccd`（`overflow|=OverflowType.CCD(1<<4)`） |
| `src/physics/collision/cuda/mod.rs` | 追加 `POLYTOPE_CUDA`/`POLYTOPE_BUILD_CUDA`/`EPA_CUDA`/`CCD_CUDA` 源码常量 |

- 缓冲容量：polytope 由调用方按 `epa_iterations` 分配（vert `10+2I`、face `6+5I`、horizon `MJ_MAX_EPAHORIZON=24`）；
  **宿主 launcher（NVRTC 编译入口 + 批量 launch/回读）与 GPU 侧批量对照归 GPU 窗口阶段**，
  本批不含运行时接线，也不产生任何 GPU 侧数字。
- 验证（CPU 侧，无 GPU 计算）：`nvcc -std=c++17 -Xcompiler /utf-8 -arch=sm_89 -c` 对
  `polytope.cu`/`polytope_build.cu`/`epa.cu`/`ccd.cu` 全部通过；`cargo check/clippy --features cuda-probe` 通过。
- 结论口径：**仅「与 host 逐式对应 + 编译通过」**；正式判据（上游 f32 GPU + pinned ctxt 全 fma ≤4 ULP）未测量。
## 7. 2c：`relative_matrix` 转置约定（G5 设备侧修正，未改 G3/G4）

- **上游语义**（`mujoco_warp/_src/collision_sdf.py`）：`params.rel_mat = wp.transpose(rot1) * rot2`，
  随后 `x1 = rel_mat * x2 + rel_pos`；行优先展开即 `rel[row][col] = Σ_k a[k][row] * b[k][col]`（`aᵀb`）。
- **设备侧已按上游修正**：`cuda/common.cu::relative_matrix` 改为行优先 `aᵀb` 直接写入 `out[row*3+col]`；
  `src/physics/collision/relmat.rs` 提供同一语义的 host 参考实现与 **CPU 单测**
  （`a=Rz(0.7)`、`b=Rx(0.4)`，期望值用独立构造的 `aᵀ·b` 展开，不依赖被测 helper）。
  单测同时复现 QA 的观察：旧 host 构造结果 = 上游结果的**转置**，最大绝对差约 1.24。
- **只读核查结论（未修改 G3/G4 已 verified 代码）**：
  - `src/physics/collision/sdf/descent.rs:171` 定义该 helper，**:198 在 `gradient_descent` 产品路径调用**
    （`rel_mat: relative_matrix(&rot1, &rot2)`，供 `x1 = rel_mat*x2 + rel_pos`）：对 `rot1 ≠ I` 是转置语义，
    属**潜在产品偏差**；G3 现有数值 case（`sdf-box`）geom1 恒为单位旋转（`a=I` 时 `aᵀb=bᵀa=b`），未被暴露。
    建议 Arch 裁决开「G3 约定修正」小片（改 3 行 + 重跑 G3 判据回归），本次未动。
  - `src/physics/collision/hfield/driver.rs:55` 定义该 helper，但**产品 driver 不调用**（仅 `tests.rs:196`
    在测试里把 geom2 变换到 hfield 帧时使用）；`hfield_pair_contacts` 接收的是调用方已变换到 hf 帧的 geom2，
    故 G4 产品路径不经该 helper；QA 观测到的 rot-box/rot-capsule 与上游一致来自「调用方预变换」而非该 helper。
- 口径：设备侧以**上游语义**为准，不为与 host 逐位一致而复制错误约定；正式判据仍是 GPU 窗口阶段的
  上游 f32 GPU 对照。
## 8. 第三批（增量提交 3/5）：SDF narrowphase 设备版

| 文件 | 内容 |
| --- | --- |
| `src/physics/collision/cuda/sdf_octree.cu`（新） | 对应 host `sdf/octree.rs`：`box_project`（eps=1e-4）、`find_oct`（100 次上限/eps=1e-6/叶判定/子索引 `4z+2y+x`/grad 权重）、`sample_volume_sdf`（盒距+`dot8`）、`sample_volume_grad`（盒外差分 h=1e-4、盒内解析）、`halton`、`transform_aabb` |
| `src/physics/collision/cuda/sdf_math.cu`（新） | 对应 host `sdf/primitives.rs` + `sdf/descent.rs`：原语 SDF/梯度、`sdf`/`sdf_grad` 分发（含 volume 分支）、`clearance`、`compute_grad`、Armijo `gradient_step`、`gradient_descent`（`sdf_iterations`+1 次 intersection） |
| `src/physics/collision/cuda/sdf_narrowphase.cu`（新） | 对应 host `sdf/narrowphase.rs`：AABB 交集 + `sdf_initpoints` 个 Halton(2,3,5) 初始点 + 每初始点一接触 + `make_frame` |
| `src/physics/collision/cuda/mod.rs` | 追加 `SDF_OCTREE_CUDA`/`SDF_MATH_CUDA`/`SDF_NARROWPHASE_CUDA` 常量 |

- `rel_mat` 一律使用 `common.cu` 已按上游修正的 `relative_matrix`（`transpose(rot1)*rot2`），不依赖 host 旧约定。
- 验证：三文件均通过 `nvcc -std=c++17 -Xcompiler /utf-8 -arch=sm_89 -c`（纯编译）；`cargo check/clippy --features cuda-probe` 通过。
- 结论口径：仅「与 host 逐式对应 + 编译通过」；**SDF 正式判据（直接对同配方上游 f32 GPU）待窗口**。
- 截断/`nacon` 写入语义在第五批（接触写入）处理；本批只产生候选结构体。

## 9. 第四批（增量提交 4/5）：hfield prism CCD 设备版

| 文件 | 内容 |
| --- | --- |
| `src/physics/collision/cuda/hfield.cu`（新） | 对应 host `hfield/filter.rs` + `hfield/driver.rs`（上游 `_hfield_filter` + `ccd_hfield_kernel`）：`d_hfield_data`（G1 的 `hfield_size/nrow/ncol/adr/data` 视图）、`d_hfield_bounds`；`g5_hfield_filter`（水平/垂直 box-sphere、`hf_matᵀ` 变换、支撑紧 AABB、box-box）；`g5_hfield_pair_contacts`（子网格 `cmin/cmax/rmin/rmax`、prism 六顶点链式前移与底三角 `z=-size[3]`、顶三角高度早退、逐棱柱 `g5_ccd` 组合、`dist >= margin+gap` 跳过、0..3 四接触 + `MIN_DIST_TO_NEXT_CONTACT=1e-3`、`MJ_MAXCONPAIR=50` 溢出 → `OverflowType.HFIELD=1<<5`）；`d_hfield_result` 输出 `ncontact`/`prism_count`/`overflow`/4 接触 |
| `src/physics/collision/cuda/common.cu` | `relative_matrix` 改为显式 `__fmaf_rn` 链（上游 `mul(mat,mat)` 的 `muladd` 语义，见 §9.1） |
| `src/physics/collision/relmat.rs` | host 参考同 fma 链（f64 模拟、不用 `f32::mul_add`）+ warp 位型单测（见 §9.1） |
| `src/physics/collision/cuda/mod.rs` | 追加 `HFIELD_CUDA` 源码常量 |

- 世界→hfield 帧（设备侧**按上游语义**）：`pos = transpose_mul_mat3_vec(hf_mat, pos2 - hf_pos)`、
  `rot = relative_matrix(hf_mat, rot2)`（`transpose(rot1)*rot2`）；不复制 host 测试 helper 的旧转置约定。
- 字段口径与上游 `geom_collision_pair` 一致：`geom1`（hfield prism）单位位姿/`margin=0`/`index=-1`/`size=0`；
  `geom2.margin` 在子网格阶段设为两侧 `geom_margin` 之和，`geom2.pos` 为 hfield 帧位置且作为 `ccd(x_2)`。
- `overflow` 汇总：每棱柱 `g5_ccd` 的 `result.overflow`（`EPA_HORIZON`；`FLOAT_MAX`→`CCD`，G2 宿主口径）
  逐位或入 `d_hfield_result.overflow`；`MJ_MAXCONPAIR` 溢出再加 `OverflowType.HFIELD`。
  `nacon`/`naccdmax` 截断与 world 级计数/原子或归第五批（本批不写接触缓冲）。
- 验证（CPU 侧，无 GPU 计算）：`nvcc -std=c++17 -Xcompiler /utf-8 -arch=sm_89 -c hfield.cu` 通过
  （纯编译，无 GPU 上下文，未启动 kernel）；`cargo check --locked --features cuda-probe --lib` 通过。
- 结论口径：仅「与 host 逐式对应 + 编译通过」；**hfield 正式判据（对同配方上游 f32 GPU）待窗口**。

### 9.1 `relative_matrix` 的上游位型（fma 累加；2026-10-25 QA 实证）

- 上游 `warp/native/mat.h::mul(mat,mat)` 用 `muladd`（`fmaf`）逐项累加：
  `sum = 0; for k: sum = fma(a[i][k], b[k][j], sum)`（k 升序）；`mul(mat,vec)` 用普通乘加。
- 设备侧 `cuda/common.cu::relative_matrix` 改为**显式 `__fmaf_rn` 链**（不依赖 `--fmad` 自动收缩，
  避免判据位型随编译开关漂移）：`sum = a0*b0; sum = __fmaf_rn(a1,b1,sum); out = __fmaf_rn(a2,b2,sum)`。
- host 参考 `relmat.rs::upstream_relative_matrix` 同样 fma 链，但**不用 `f32::mul_add`**
  （本机 dev 档 opt-level 0 实测未融合：`0x3f17e6af` vs 正确 `0x3f17e6b0`），改用
  「f64 精确承载 f32 乘积 + 一次和 → 单次舍入回 f32」。
- **位型证据**（QA fixture `docs/development-evidence/g10-p5-hfield-host-independent-check/qa-sdf-rotation-cases.json`，
  mujoco_warp 3.15 f32 CPU、两次捕获位等）：
  - `sdf-rot-both`（box `euler="25 10 30"`、SDF `Rz(0.7)`）：`rel_mat` **9/9 cell 位级 == warp**；
  - `sdf-rot-rz-rx`（box `Rx(0.4)`、SDF `Rz(0.7)`）：**9/9 cell 位级 == warp**，含 QA 指出的
    `f32::mul_add` 敏感位 `rel[1][0] = 0x3F17E6B0`；
  - 对照：普通乘加在 `sdf-rot-both` 的 cell 6 与 warp 位型差 **1 ULP**（`0xBE0A8E4E` vs `0xBE0A8E4F`），
    与 QA `rel_mat_cells_diff_computed_vs_warp = 1` 一致 → 证明必须 fma 链（只改转置不够）。
- 单测：`cargo test --locked --lib relmat` 4 项全过（dev 档）——
  `upstream_relative_matrix_matches_warp_bits_sdf_rot_both`、`..._sdf_rot_rz_rx`、
  `plain_accumulation_differs_from_warp_bits`、`upstream_relative_matrix_matches_transpose_rot1_rot2`。
- 应用点：SDF 设备内核的 `rel_mat`、hfield 设备侧 `rot = relative_matrix(hf_mat, rot2)` 均已走该实现。
- 边界：这是「设备/host 参考函数与 warp 位型一致」的证据；**G3/G4 产品 host 路径的修正仍归
  `2026-10-25-g10-p5-sdf-relmat-fix`（排队等 G5 收口）**，本片不改 `sdf/descent.rs`、`hfield/driver.rs`。