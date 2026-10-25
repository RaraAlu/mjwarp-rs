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

## 10. 第五批（增量提交 5/5）：接触写入/容量截断 + `nworld` 上传布局

| 文件 | 内容 |
| --- | --- |
| `src/physics/collision/cuda/contact.cu`（新） | 上游 `collision_core.py::write_contact` 设备版：`active`/`detected`/`condim` 覆盖规则、`ContactType.CONSTRAINT/SENSOR` 位、`pairid` 跳过规则、`cid=atomicAdd(nacon)` 后**先判 `cid < naconmax` 再写**（不越界写）、`geomcollisionid=id_`（对内序号）、`efc_address` 全写 `-1`（宽度=`nmaxpyramid` 由调用方给）；`g5_claim_ccd_slot`：`nccd[geomgeomid]` 原子计数 + `ccdid >= naccdmax` → `OverflowType.CCD`（不越界用 EPA 缓冲）；`g5_ccd_pair_write`：上游 `eval_ccd_write_contact` 子集（margin 覆盖、`cutoff=sensor?1e32:gap`、原点平移、GJK→EPA、gap 早退、`dist += margin`、frame 符号规则、`0.5*(w1+w2)+origin`、单接触写入）；`g5_hfield_write_contacts`：hfield 0..3 接触逐个写 + 溢出位汇总 |
| `src/physics/collision/cuda/mod.rs` | 追加 `CONTACT_CUDA` 常量 |
| `src/runtime/collision_upload.rs`（新） | `CollisionUploadPlan`：G1 体积/SDF 模型字段**一份共享 payload**（16 字节对齐、字段顺序固定、f64→f32 只走 G1 唯一转换点）+ `pack()`；`ContactCapacity`：上游实测容量口径（`naconmax/naccdmax` 全局总量 = `nconmax/nccdmax × nworld`、接触行**全局编号**、`efc_address` 宽度 `max(1, 2*(nmaxcondim-1))`） |
| `src/runtime/mod.rs` | 注册 `collision_upload`（`pub(crate)`） |

### 10.1 `nworld>1` 上传口径的实测修正（重要）

- G1 §6 写的是：体积/SDF 模型字段共享一份缓冲；接触容量「每 world 的 `naconmax` 分段布局、
  kernel 按 world 计算行偏移」。
- **前半段与上游一致**：`io.py`/`types.py` 里 `hfield_size: array("nhfield", wp.vec4)`、
  `oct_aabb: array("noct", 2, wp.vec3)` 等模型字段形状都不带 world 维 → 一份共享缓冲。
- **后半段与上游不符**：上游 `nacon: array(1)` 是**全局**单计数器（`types.py` 描述
  "number of detected contacts (across all worlds)"、形状 `(1,)`），接触数组形状
  `(naconmax, ...)`、行号由 `wp.atomic_add(nacon, 0, 1)` **全局**分配，world 归属靠
  `contact_worldid` 字段；`naconmax` 是全部 world 的总行数（`nconmax * nworld` 只是默认换算），
  **没有**逐 world 行分段/行偏移。`naccdmax` 同理全局总量，`nccd` 按 `geomgeomid` 分组计数、
  截断判据 `ccdid >= naccdmax`。
  证据：`io.py:2390`（`d.nacon = wp.array([mjd.ncon * nworld])`）、`types.py:2408`、
  `io.py:1977`（`contact.efc_address = full((naconmax, nmaxpyramid), -1)`）、
  `io.py:1915`（`naccdmax = _resolve_batch_size(naccdmax, nccdmax, nworld, naconmax)`）。
- 本片代码与证据按**上游实测**实现（`ContactCapacity::contact_row_for_world` 返回全局 `cid`，
  注释写明无 world 偏移），建议 Arch 在 G1 证据 §6 追加注记（不改 G1 数值结论）。

- 验证（CPU 侧，无 GPU 计算）：`nvcc -std=c++17 -Xcompiler /utf-8 -arch=sm_89 -c contact.cu` 通过；
  `cargo test --locked --lib collision_upload` **6/6 过**（共享布局与 world 数无关、全局行号、
  容量换算/非法配置拒绝、payload 位级等于 G1 转换点、字段名映射 G1 字段表）；
  `cargo check --locked --features cuda-probe --lib` 通过。
- 口径：仅「与上游逐式对应 + 编译通过 + 布局单测」；**正式判据（接触数组/截断/多世界结构对
  同配方上游 f32 GPU）待窗口**。
- 本批未实装：multicontact 设备版（box/mesh/cylinder 组合的多接触仍是单接触，见 unmet）；
  宿主 launcher 与真实设备上传/回读归 GPU 窗口阶段。

## 11. GPU 窗口（2026-10-25 17:53–17:57）：设备内核 vs 上游 f32 GPU 对照

**窗口纪律**：Arch 批准 45 min / 满载 <10 min / 峰值增量 <2 GiB / 复跑 ≤5。/ 实测总 GPU 占用约 **30 s**
（阶段 1：基线 → CUDA 准备 → 构建 → 10 case 探针 → 比对，28.4 s；阶段 2：`--fmad=false` 复跑，1.5 s）。
基线 4460–4534 MiB used，逐 2 s 采样峰值 **4492 MiB**（未超基线），跑完 4460/4344 MiB 回桌面基线；
`nvidia-smi --query-compute-apps` 无本任务进程（只剩桌面应用）。

### 11.1 参考口径（已获 Arch 批准，不重复采 10 case）

- G0b 交付参考 `fixtures/g10-p5/references/*.json` 本身就是 `capture_g0b_references.py` 以
  `wp.set_device("cuda:0")` 捕获（provenance 含 `gpu: RTX 4070 Ti SUPER`），即**已是上游 f32 GPU 参考**；
  QA `g0b-cuda-recapture`（`g10-p5-gjk-epa-independent-check/`）另做过 cuda:0 两次捕获、10 case、
  与交付参考排序后位级全等。
- 本窗口用 `prepare_cases.py --device cuda` 以 CUDA 后端重跑 10 case 并**在窗口内绑定**：
  10/10 `matches_reference_sorted_bitwise = true`；spot-check 三例双捕获：
  `hfield-sphere` raw 位等、`cylinder-box` raw 位等、`sdf-box` raw **不等**（atomic nacon 顺序，
  与 QA 记录一致）但排序后位等。
- 桥接链：QA CUDA 重采证据 → 交付 CUDA fixtures → 本窗口 CUDA 重跑 + spot-check 互证。
  **未声称** 10 case 在本窗口全部独立重采（只重跑验证，不重采）。

### 11.2 设备对照（nvcc 独立探针，B 路线）

- 探针 `probe.cu` 用**同一批 device 源**（`#include` `contact.cu`/`hfield.cu`/`sdf_narrowphase.cu`），
  nvcc 12.8.93、`--fmad=true`、`-O3`、`sm_89`；构建命令/全部源 sha256/两个变体产物 sha256 见
  `gpu-window/build-probe-report.txt`；探针每 case 跑两次。
- 方法学修正（重要）：上游 `collision_driver.py` 的 dispatch 里 `(SPHERE,BOX)`、`(CAPSULE,BOX)`
  走 **primitive_narrowphase**，交付参考不是 ccd 产物 → `sphere-box-shallow/deep`、`capsule-box`
  **不列入**设备 ccd 对照（设备 primitive 路径本片未实装）。
- 可对照 CONVEX/SDF/HFIELD case 结果（`gpu-window/result.json`、`compare-probe.json`）。
  判据按 Arch 2026-10-25 裁决：`frame`/`normal` 是单位向量 → ULP 统一按 **scale=1.0** 计，
  分量绝对差 ≤ **4 ULP@1.0 ≈ 4.77e-7**（零/非零一视同仁，**不设近零带**）；
  `dist`/`pos` 按各自量级 ULP（零参考分量用 ≤4 ULP@1.0 绝对差兜底）；参数/整数位级相等。

| case | count | dist 位等 | pos 位等 | frame 位等 | 实测 | 结论 |
| --- | --- | --- | --- | --- | --- | --- |
| hfield-sphere | 8/8 | 8/8 | 24/24 | 72/72 | 全 0 ULP | ✅ 全字段位级 |
| hfield-capsule | 7/7 | 7/7 | 21/21 | 63/63 | 全 0 ULP | ✅ 全字段位级 |
| sdf-box | 80/80 | 80/80 | 202/240 | 675/720 | dist 0 ULP；pos max 4 ULP；frame max_abs **2.38e-7**（≤4.77e-7） | ✅ dist 位级、pos ≤4 ULP、frame ≤4 ULP@1.0 |
| ellipsoid-box | 2/2 | 0/2 | 0/6 | — | dist **47 ULP**（绝对 4.4e-8）、pos 绝对 8.3e-5 | ❌ **未过**（G5 不收口） |

- 每 case：`efc_address` 全 -1（宽度 `nmaxpyramid`）、`OverflowType` 汇总 0、接触数相等、
  两次运行排序后位等；`nccd` 记录为 0 或每 world 1 次申请（hfield/SDF 探针路径不申请，容量门归 launcher）。
- 测试拆分（提交棘轮 ≤500 行）：入口 `tests/g10_p5_gpu_kernels.rs`（81 行）+
  `tests/common/gpu_compare.rs`（438 行，`#[path]` 引入的比对辅助，非独立测试目标）。
- `--fmad=false` 复跑（`compare-probe-fmad-false.json`）：hfield-capsule 仍全位级，但 `sdf-box`
  大幅偏移（pos max_abs 0.18、frame max_abs 1.9，initpoint 梯度下降分岔）→ **官方判据用 fmad=true**，
  `--fmad=false` 仅诊断，不作为判据。

### 11.3 unmet（本窗口不构成 G5 收口）

- **Arch 裁决落地**：对照集修正（primitive 三例排除、multicontact 三例不比对）已接受；
  frame 判据改为「scale=1.0 的 4 ULP 绝对差」并已由 `result.json` 重算；`ellipsoid-box` 记为未过。
- **ellipsoid-box（CONVEX）未过 ≤4 ULP**：dist 47 ULP（4.4e-8）、pos 绝对 8.3e-5。
  pos 量级明显不是纯收缩差，需定位设备 contact 层 / EPA 与上游差异（候选：
  `g5_ccd_pair_write` 的 origin 平移后 `x_1/x_2`、EPA witness 选面/tie-break、或输入口径）；
  修复按 pinned PTX 顺序显式 fma（S7b 方法），**不放宽容差**；修好后报秒级窗口复跑。
- **witness 对照待补**（Arch 裁决 1）：`cylinder-box`/`box-box-margin`/`mesh-mesh` 需 authoring
  直接调用上游 `collision_gjk.ccd` 产出 dist/x1/x2，再与设备探针 witness 比 ≤4 ULP；
  计划与 ellipsoid-box 修复后的秒级窗口一起跑（省窗口）。
- **primitive narrowphase 未实装**：`(SPHERE,BOX)`、`(CAPSULE,BOX)` 等上游走 primitive 的 case 无法进设备对照。
- **multicontact 设备版未实装**：`cylinder-box`/`box-box-margin`/`mesh-mesh` 接触数组不做全等。
- **NVRTC 产品 launcher 未接线**：本窗口用 nvcc 独立探针跑判据数字；`src/runtime/collision_gpu.rs`
  与 ignored 产品入口单列后续 launcher 小片。
- 性能 benchmark 未跑（归 G6）；Linux/T4 未验证；GPU 结果不外推跨硬件结论。