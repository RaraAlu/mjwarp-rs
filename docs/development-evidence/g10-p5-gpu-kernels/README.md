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
  **判据（逐字同 `result.json.criterion`；Arch 2026-10-25 裁决含裁决 2 的位置向量补全，
  实现与判别力自证见 `gpu-window/criterion_scale.rs`、说明见 §11.6.2）**：

  > CONVEX/SDF/HFIELD case：count 相等 + worldid 分组/稳定排序后比较。dist：按自身量级 ≤4 ULP（ref==0 用 4 ULP@1.0 绝对差）；pos（位置向量）：scale=max_i|ref_i|（整向量全零回落 1.0），逐分量 |dev-ref| ≤ 4·ULP(scale)，位级优先；frame（单位向量）：scale=1.0，绝对差 ≤4 ULP@1.0（4.77e-7，不设近零带）；接触参数与整数字段位级相等。判据补全的判别力自证见 gpu-window/criterion_scale.rs（修前 pos 8.3e-5 / dist 4.4e-8 在现判据下仍被拒绝）

| case | count | dist 位等 | pos 位等 | frame 位等 | 实测（窗口 1 旧判据） | 结论 |
| --- | --- | --- | --- | --- | --- | --- |
| hfield-sphere | 8/8 | 8/8 | 24/24 | 72/72 | 全 0 ULP | ✅ 全字段位级 |
| hfield-capsule | 7/7 | 7/7 | 21/21 | 63/63 | 全 0 ULP | ✅ 全字段位级 |
| sdf-box | 80/80 | 80/80 | 202/240 | 675/720 | dist 0 ULP；pos max 4 ULP；frame max_abs **2.38e-7**（≤4.77e-7） | ✅ dist 位级、pos ≤4 ULP、frame ≤4 ULP@1.0 |
| ellipsoid-box | 2/2 | 0/2 | 0/6 | — | dist **47 ULP**（绝对 4.4e-8）、pos 绝对 8.3e-5 | ❌ 窗口 1 未过；根因与修复见 §11.6，复测见 §11.7.1 |

- 每 case：`efc_address` 全 -1（宽度 `nmaxpyramid`）、`OverflowType` 汇总 0、接触数相等、
  两次运行排序后位等；`nccd` 记录为 0 或每 world 1 次申请（hfield/SDF 探针路径不申请，容量门归 launcher）。
- 测试拆分（提交棘轮 ≤500 行）：入口 `tests/g10_p5_gpu_kernels.rs`（81 行）+
  `tests/common/gpu_compare.rs`（438 行，`#[path]` 引入的比对辅助，非独立测试目标）。
- `--fmad=false` 复跑（`compare-probe-fmad-false.json`）：hfield-capsule 仍全位级，但 `sdf-box`
  大幅偏移（pos max_abs 0.18、frame max_abs 1.9，initpoint 梯度下降分岔）→ **官方判据用 fmad=true**，
  `--fmad=false` 仅诊断，不作为判据。

### 11.3 unmet（窗口 1 历史状态；后续窗口的更新见 §11.7）

- **Arch 裁决落地**：对照集修正（primitive 三例排除、multicontact 三例不比对）已接受；
  frame 判据改为「scale=1.0 的 4 ULP 绝对差」并已由 `result.json` 重算；`ellipsoid-box` 记为未过。
- **ellipsoid-box（CONVEX）窗口 1 未过 ≤4 ULP**：dist 47 ULP（4.4e-8）、pos 绝对 8.3e-5。
  → 根因与修复见 §11.6（设备 `v3_sign` 零值语义），复测见 §11.7.1（dist 2 ULP/位级、pos 0.81 ULP@向量尺度）。
- ~~**witness 对照待补**~~：**已完成**（§11.7.3），`cylinder-box`/`box-box-margin`/`mesh-mesh` 三例
  对直调上游 `collision_gjk.ccd` 全过 ≤4 ULP（两条全位级、一条 2 ULP）。
- **primitive narrowphase 未实装**：`(SPHERE,BOX)`、`(CAPSULE,BOX)` 等上游走 primitive 的 case 无法进设备对照。
- **multicontact 设备版未实装**：`cylinder-box`/`box-box-margin`/`mesh-mesh` 接触数组不做全等
  （witness 对照只覆盖 `ccd()` 层）。
- **NVRTC 产品 launcher 未接线**：本窗口用 nvcc 独立探针跑判据数字；`src/runtime/collision_gpu.rs`
  与 ignored 产品入口单列后续 launcher 小片。
- 性能 benchmark 未跑（归 G6）；Linux/T4 未验证；GPU 结果不外推跨硬件结论。
### 11.4 ellipsoid-box 三方对照（2026-10-25 追加，CPU-only）

同一探针输入（`gpu-window/cases/ellipsoid-box/`）下：

| 对比 | dist | pos | frame |
| --- | --- | --- | --- |
| ②上游 CPU vs ③上游 CUDA（`ellipsoid-box-cpu-capture.json`） | ≤2 ULP（1.86e-9） | 7.45e-9 | 8.19e-7 |
| ①宿主 `ccd_pair` vs ②上游 CPU | **0（位级）** | 7.45e-9 | 3.84e-7 |
| ①宿主 `ccd_pair` vs ③上游 CUDA | 2 ULP | 7.45e-9 | 8.19e-7 |
| **设备探针 vs ③上游 CUDA** | **47 ULP（4.4e-8）** | **8.3e-5** | 1.17e-6 |

结论（Arch 已确认定性）：**设备侧结构/分支差异**，不是 fma 收缩差（设备 pos 差 ≈11,000× 上游 CPU/GPU
自身差）、不是输入口径、不是 nvcc vs CUDA 12.9 编译差。目标参考锁定**上游 CUDA（G0b fixtures）**。
下一步：设备探针 dump 中间量（`dist`/`x1`/`x2`/EPA `face_idx`/`normal`），与宿主/上游 CPU 同一中间量
逐项比，找第一个先偏的量后按 PTX/上游表达式定序修；不放宽容差、不跳过 case。宿主诊断测试
`ellipsoid_box_host_ccd_pair_vs_cuda_reference`（ignored）保留于 `tests/g10_p5_gpu_kernels.rs`。
### 11.6 ellipsoid-box 根因与修复（2026-10-25，CPU-only 定位）

**根因**：设备 `cuda/common.cu::v3_sign` 把零/-0.0 映射为 0；上游
`include/warp-headers/builtin.h:765 sign(x) = x < 0 ? -1 : 1`（零/-0.0 → **+1**），
宿主 `gjk/math.rs::sign3` 已正确。`ellipsoid-box` 两 world 的 box `local_dir` y 分量精确为 0 →
设备 BOX 支撑取面中心（`vertex_index=5`）而非角点（上游 =7）→ GJK 首轮支撑点即分叉 →
`dim` 4→3 → EPA 走 `polytope3`（face 44/45）而非 `polytope4`（21/22）→ witness 沿切向偏 8.3e-5，
而 `dist` 仅差 4.4e-8。GJK/EPA 主循环本身无问题（逐句一致）。

**修复（产品源，G5 范围）**
- `cuda/common.cu::v3_sign` → `x < 0 ? -1 : 1`（3 分量，零/-0.0→+1）；唯一使用点
  `cuda/support.cu` BOX 支撑（对应上游 `collision_gjk.py` box support 的 `wp.sign(local_dir)`）。
- **同类清扫**：`cuda/sdf_math.cu` 3 处自写「零→0」sign 三元式改为上游 `wp.sign` 语义
  （零/-0.0→+1）：`radial_field`（上游 `collision_sdf.py:152-154`）、cylinder `w`
  （上游 `:275`）、梯度步 `sign`（上游 `:629`）。

**修复后中间量对照（同一 case、CPU 执行 device 源 vs 宿主，`device-logic-intermediates-after-fix.log` vs `host-intermediates.log`）**

| world | dim | face_idx | lower | normal | 三顶点 | 权重 w | pos |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | **4**（修复前 3） | **21**（修复前 44） | 0.00999989174 | 与宿主位级一致 | (9,8,7) 同宿主 | 与宿主位级一致 | 与宿主差 ~1e-9（CPU 执行环境） |
| 1 | **4** | **22** | 0.00999982003 | 位级一致 | (9,8,7) | 位级一致 | ~1e-9 |

**回归影响判断（窗口内确认）**：修复只涉及 BOX support 与 SDF 原语 sign →
`hfield-sphere`/`hfield-capsule`（HFIELD+SPHERE/CAPSULE，不经 BOX support/SDF 原语）预期不动；
`sdf-box` 已跑的 80 接触未踩到 `radial_field` 零分量，预期不动；三者待窗口回归确认。

**同类判定清扫表（cuda 目录 ↔ 上游 `builtin.h`/host `gjk/math.rs`）**

| 判定点 | 设备 | 上游/宿主 | 结论 |
| --- | --- | --- | --- |
| `v3_sign` 零值 | 旧：0 | `builtin.h:765`/`sign3`：+1 | **已修**（本次根因） |
| `sdf_math` 自写 sign（3 处） | 旧：0 | `wp.sign`：+1 | **已修**（同类清扫） |
| `v3_normalize` 零保护 | 返回零向量 | 宿主同（上游 native 无保护，调用点均有 `MINVAL` 前置） | 一致，保留 |
| cylinder `d > G5_MINVAL` / `top = z >= 0` | 同宿主 | 上游同 | 一致 |
| capsule `local_dir.z < 0` | 同宿主 | 上游同 | 一致 |
| hfield 支撑 `vertex_index = dir.z<0 ? -2 : -3` | 同宿主 | 上游同 | 一致 |
| GJK `lmbda == 0` / `cutoff == 0` / `norm 阈值` | 同宿主 | 上游同 | 一致 |
| EPA `lower2 <= 0` / `dist2 == 0` / 选面 `<` | 同宿主 | 上游同 | 一致 |
| **host** `sdf/primitives.rs` 用 `f32::signum()` | — | 上游 `wp.sign`：-0.0→+1，signum(-0.0)→-1 | **发现差异，未改**（G3 已验收；交 Arch 裁定是否开 fix 小片） |

### 11.6.1 同类判定清扫：全 14 文件逐文件（2026-10-25 补，Chief 点名）

方法：对 `src/physics/collision/cuda/` 全部 14 个 `.cu` 文件 grep
`sign|normalize|== 0|<= 0|>= 0|< 0|> 0|fmaxf|fminf|MINVAL|eps|1e-`，逐条对照 pinned 上游
（`collision_gjk.py` / `collision_sdf.py` / `math.py` / `warp/native/{builtin.h,vec.h}`）与
host `gjk/{math,simplex,core,epa,polytope}.rs`。结论口径：**等价** = 与上游同表达式/同顺序；
**已修** = 本片改过；**防御性差异** = 上游该输入为 UB/不可达，设备给确定值；**扩展** = 上游无对应分支。
6 组重点关注（Arch 2026-10-25 点名）逐条见下表「已扫描判定点」列。

| 文件 | 已扫描判定点（行号） | 设备 vs 宿主 vs 上游 |
| --- | --- | --- |
| `common.cu` | `v3_sign` 零值(40)、`v3_normalize` 零(59-63)、`make_frame` selector(101)/零保护(103)、`relative_matrix` fma 链(86-95) | **已修**(sign)；`v3_normalize` = 上游 `warp/native/vec.h:1093-1099`（`l>kEps? a/l : 0`）且 `kEps=0.0f`（`builtin.h:573`）→ **等价**；`make_frame` 零保护 = 上游 `math.py:209` 原句、selector = `math.py:206` → **等价**；relmat = 上游 `mat.h mul(mat,mat)` fma 链（§9.1） |
| `geom.cu` | 仅 `d_mesh`/`d_geom`/`d_support_point` 结构定义 | **无判定点** |
| `support.cu` | BOX `v3_sign`(101-107)、CAPSULE `local_dir.z<0`(111)、ELLIPSOID normalize(114-115)、CYLINDER `d>G5_MINVAL`(120)/`top=z>=0`(125)、MESH `graphadr==-1||vertnum<10`(26)/`index>-1`(27)/`subidx>=0`(65)、HFIELD `dir.z<0 ? -2:-3`(132)/`dist>max_dist`(138)、TRIANGLE/FLEX 严格比较(150/160)、margin>0(165) | 全部 = 上游 `support()` 逐句（`collision_gjk.py:118-254`，含 tri/flex/hfield 分支：`:202-240`）；唯一差异 `vertadr/vertnum > 0 ? : 0` 下界 = 对「无 mesh data 的 MESH」防御（上游该路径读 -1 为 UB，有效模型不可达）→ **防御性差异** |
| `simplex.cu` | `same_sign`(13-16)、`nn==0`(41/53)、`nv!=0 && nn>MINVAL`(45/57)、第三平面无 guard(60-70)、`l<0` 剔除(374)、`length<MINVAL`(378)、`dot1/dot2` 同号(385) | = 上游 `_project_origin_plane`(`:337-365`)、`_S1D/_S2D`、三角/线判据逐句 → **等价**（G2 host 对照已验收） |
| `gjk_core.cu` | `x_norm<1e-4`(51)、`edge_norm2>MINVAL2`(55)、`dir_norm>MINVAL`(59)、`normal_norm>MINVAL`(68)、`sign=dot<0?-1:1`(69)、`xnorm<min_norm`/`|Δxnorm|<MINVAL`(108)、Frank-Wolfe(123)、`cutoff==0`(127)、`lower>0` 分离(128/136)、`lmbda==0` 剔除(151)、`n==4`(165)、`xnorm>0` 复检(176)、`separated` 复判(180)、`dist=(n==4&&!separated)?0:xnorm`(182)、`is_discrete`(199) | 全部 = 上游 `gjk()`/`_gjk_support()` 逐句；**:69 = 上游 `collision_gjk.py:649 wp.sign(dot)`（零→+1）** → **等价**（非本轮需改点） |
| `polytope.cu` | face 位域/get/set、`_attach_face` 外向校正 `dot(r,p1-center)<0`(59)、`_ray_triangle` 零 tie(109-110)、`_epa_support` normalize(179)、`_epa_witness` 权重坐标(184-191) | `_ray_triangle` 与上游 `:930-938` **同顺序**（先全 `>=0`→1，再全 `<=0`→-1；零处先命中前者→1）→ **等价**；其余逐句（G2 host 已验收） |
| `polytope_build.cu` | `_polytope2/3/4` 状态码、`norm<G5_MINVAL`(118)、`_tri_point_intersect`(147/151)、`dist>1e-5`(155)、`_test_tetra`、attach `shadow<G5_MIN_DIST3`(159 起) | = 上游 `_polytope2/3/4` 逐句（含 `MINVAL`、`1e-5`、`MIN_DIST3` 常量）→ **等价** |
| `epa.cu` | `n<0`(25)、`long long` 节点、`lower2>upper2||idx<0`(67)、`lower2<=0`(71)、`upper-lower<epsilon`(91)、`dot(face_pr,w)-norm2>1e-10`(122)、`dist2==0`(145)、`dist2∈[lower2,upper2]` 有效面(150)、`nvalid==0||idx==-1`(157)、`epa_iterations>1000`(52)、`epsilon=is_discrete?MIN_EPATOL:tolerance`(50) | 全部 = 上游 `_epa`（`collision_gjk.py:1354-1470`）逐句 → **等价**；已入表项归位 |
| `ccd.cu` | `_inflate` 坐标 `>0` 剔除(71-78)、`margin>0` 膨胀(90-91)、`size1+size2>0`(127)、`multiccd_idx`：margin!=0→-1(268)、BOXMESH/CYLINDER 限定(269-273)、`G5_FLOAT_MAX→CCD overflow`(303) | = 上游 `_inflate`/`epa_phase`（`:2664-2677`）逐句；`multiccd_idx` 仅信息（设备 multicontact 未实装）→ **等价** |
| `sdf_octree.cu` | `box_project`：全内早退 `q<=0`(40)、逐轴 `q>=0`(47/51/55)、`r>0? -(q+eps): +(q+eps)`(49/53/57)、`eps=1e-4`(45)；`find_oct`：`eps=1e-6`(75)、半开 `p±eps<vmin / >vmax`(85-86)、`coord<0.5`(126-128)、`4z+2y+x`(129)、叶判据、grad 权重(108-120)；`sample_volume_grad` `dist0>0` 差分(151)；Halton `n0>0`(177) | 全部 = 上游 `collision_sdf.py:399-434`（box_project）与 `:330-393`（find_oct）逐句 → **等价** |
| `sdf_math.cu` | `radial_field` sign(50-52)、`box` `a>=0` 早退(61)、`grad_box` `a<=0→g=0`(108-110)、cylinder `w=sign(p.z)`(148)、`radial_dist>MJ_MINVAL`(144/153)、`len_v>MINVAL`、`c>G5_MINVAL`(136)、`max_val` sign(245)、Armijo(amin=1e-4/rho=0.5/c=0.1) | sign 三处**已修**；其余 = 上游 `collision_sdf.py:148-160/164-170/215-228/263-283/629` 逐句 → **等价** |
| `sdf_narrowphase.cu` | AABB 交集 `min>max` 早退(52)、Halton 三序列(58-60)、`limit=min(initpoints, capacity)`(55) | AABB/初始点 = 上游 `_sdf_narrowphase` 逐句；`capacity` 截断是探针容量保护（产品 `naconmax` 语义在 launcher）→ **等价 + 容量保护** |
| `hfield.cu` | `cmin/rmin` 下界 clamp(168/172)、`count>=MAXCONPAIR→overflow`(226)、`prism[11..17]<zmin` 高度早退(247)、`dist>=margin+gap` 分离早退(266)、`0<d<MIN_DIST_TO_NEXT_CONTACT=1e-3` 去重(311/328/348)、`noverflow>0` 早退(202/222) | = 上游 `collision_convex.py:576/592/649/707` 与 `ccd_hfield_kernel` 逐句（常量 `MIN_DIST_TO_NEXT_CONTACT=1e-3` 一致）→ **等价** |
| `contact.cu` | `adhesion!=0` 两处(67-70)、`detected=dist<margin+gap`(69)、`pairid[0]==-2`(73)、`pairid[1]>=0` sensor(79)、`atomicAdd(nacon)+cid<naconmax`(82-83)、`dist<=margin` frame 方向(195)、`dist>=gap` 早退(188) | = 上游 `write_contact`/`eval_ccd_write_contact` 逐句（含「先计数后判容量、超容量不写」）→ **等价** |

**清扫结论**：14 文件全部过表；除本片已修的 `v3_sign`+`sdf_math` 3 处 sign 外，未发现新的
零值/符号语义偏离。仅 2 条记录性差异：①`support.cu` 的 `vertadr/vertnum` 下界（上游 UB → 防御）；
②host `sdf/primitives.rs::signum()`（G3 已验收，另片修正 `2026-10-25-g10-p5-sdf-primitives-sign`）。

### 11.6.2 判据补全：位置向量改用向量尺度 ULP（Arch 2026-10-25 裁决，非放宽）

**问题**：`ellipsoid-box` world0 `pos.y` 参考值 |3.26e-5|，其自身量级 ULP=3.7e-12 → 残差
3.02e-9 被旧「逐分量按各自量级 ULP」判成 830 ULP；但 `pos` 由 `0.5*(w1+w2)+origin`
（origin≈0.04–0.055）合成，舍入尺度是坐标量级，不是近零分量自身。

**裁定（全文，不得再放宽）**
- `pos`/`x1`/`x2`（位置向量）：`scale = max_i |ref_i|`，逐分量 `|dev-ref| ≤ 4·ULP(scale)`，
  位级优先；**整向量全零**时才回落 `scale = 1.0`。
- `frame`/`normal`：不变（`scale=1.0`，≤4 ULP 绝对 ≈4.77e-7，不设近零带）。
- `dist`（标量）：按自身量级 ≤4 ULP；`ref==0` 用 4 ULP@1.0 绝对差。
- 参数/整数字段：位级相等。

**为什么不用 4 ULP@1.0 近零带（Arch 裁决采纳向量尺度的理由）**：`pos` 由
`0.5*(w1+w2)+origin` 合成、舍入尺度是坐标量级，向量尺度判据只放行
`4·ULP(0.055)=1.49e-8`；而 4 ULP@1.0 带会放行到 **4.77e-7**，松约 **32×**
（4.768e-7 ÷ 1.490e-8 = 32.0；Arch 裁决文字写 31×，按精确值应为 32×，此处以数值为准）。
`ellipsoid-box` 残差 3.02e-9 在向量尺度下 = **0.81 ULP**，在 1.0 带下同样通过——但后者会同时
放行更大的近零分量误差，判别力更差。

**实现（单一来源，不是两份副本）**：`gpu-window/criterion_scale.rs` 提供
`ulp_at`/`vector_scale`/`vector_bound`/`vector_component_ok`/`dist_ok`，仅由
**`gpu_compare.rs`** 以 `#[path = "criterion_scale.rs"] pub(crate) mod criterion_scale;`
引入一次；**`witness_compare.rs` 通过 `use crate::gpu_compare::criterion_scale::{...}`
复用同一模块**（兄弟模块同属一个测试目标，路径可用；无第二份副本、无 lint 豁免）。
判别力 `#[test]` 放在 `gpu-window/criterion_test.rs`，只由 **`gpu_compare.rs`** 以
`#[path = "criterion_test.rs"] mod criterion_test;` 引入一次。
`result.json` / `witness-result.json` 的 `criterion` 字段同步；README §11.2 的判据文字
**逐字同 `result.json.criterion`**（脚本核对 True）。

**判别力（可执行断言，不是断言式声明）**：`gpu_compare::criterion_test::criterion_rejects_pre_fix_defects`
把修前两组实测数字喂进**新**判据并断言 FAIL：
- 修前 `pos` 绝对差 8.3e-5 vs 允许 `4·ULP(0.055)=1.49e-8` → **22,300 ULP@0.055**
  （旧口径 830 ULP@分量尺度）→ 拒绝；
- 修前 `dist` 绝对差 4.4e-8 vs `4·ULP(0.01)`（ULP=9.3e-10）→ **47 ULP** → 拒绝；
- 同时断言修后 `pos` 3.02e-9、`dist` 位级 → 通过。
  运行位置：`cargo test --locked --features cuda-probe --test g10_p5_gpu_kernels` →
  **1 passed / 3 ignored**；该测试在**集成测试目标**
  （`tests/g10_p5_gpu_kernels.rs`，`#![cfg(feature = "cuda-probe")]`）里，**不进默认
  `cargo test --locked` 的 unit 计数**（gate `unit` 仍 **582 过 0 败**，本片无变化）；
  `-- --ignored` 的对照测试同目标另一入口（3 passed）。

**before/after 全表重算（存盘 bins，纯 CPU；唯一翻转 = ellipsoid-box）**

| case | 旧判据（逐分量 ULP） | 旧结果 | 新判据（向量尺度） | 新结果 | 翻转？ |
| --- | --- | --- | --- | --- | --- |
| hfield-sphere | dist 8/8 位级、pos 24/24 位级、frame 72/72 | ✅ | 0.0 ULP@scale；位级计数不变 | ✅ | 否 |
| hfield-capsule | dist 7/7、pos 21/21、frame 63/63 位级 | ✅ | 0.0 ULP@scale；位级计数不变 | ✅ | 否 |
| sdf-box | pos 4 ULP@分量（203/240 位等）、dist 80/80、frame max_abs 2.38e-7 | ✅ | pos **1.0 ULP@scale(0.0983)**（203/240 位等）、dist 80/80、frame 2.38e-7 | ✅ | 否 |
| ellipsoid-box | pos 830 ULP@分量（0/6 位等）、dist 47 ULP | ❌ FAIL | pos **0.81 ULP@scale(0.055)**（5/6 位等）、dist 2 ULP（1/2 位等）、frame 3.80e-7 | ✅ PASS | **是（唯一）** |
| witness cylinder-box | x1 5/6 位等、max 2 ULP@分量 | ✅ | x1 5/6 位等、**0.5 ULP@scale(0.1)**；x2 6/6 位级 | ✅ | 否 |
| witness box-box-margin | dist/x1/x2 12/12 位级 | ✅ | 全位级（0.0 ULP@scale） | ✅ | 否 |
| witness mesh-mesh | dist/x1/x2 12/12 位级 | ✅ | 全位级（0.0 ULP@scale） | ✅ | 否 |

**结论**：唯一「此前失败现在通过」是 `ellipsoid-box`，其修前缺陷（pos 8.3e-5 / dist 4.4e-8）
已被上面的可执行断言证明**仍会被现判据拒绝**；其余 case 位级计数不变 → **口径补全 ≠ 放宽容差**。

## 11.7 秒级窗口（2026-10-25 18:30–18:38）：ellipsoid-box 复测 + 三例回归 + witness 对照

**窗口纪律**：Arch/Chief 批准「时长 ≤2 min GPU 满载、峰值增量 ≤1 GiB、复跑 ≤3」。
实测 3 次短跑（全部 GPU 满载时间合计约 5–8 s，其余为 CPU 侧 nvcc 构建与 warp JIT）：

| 跑次 | 时间段（wall） | 内容 | 基线 used | 跑后 used | 说明 |
| --- | --- | --- | --- | --- | --- |
| 1 | 18:30:10–18:31:01（51.5 s） | 重建 probe（fmad=true/false）+ witness probe、4 例接触探针 ×2、3 例 witness 探针 ×2、上游 CUDA witness 首采、ignored 测试 | 3357 MiB | 3559 MiB | 首采 authoring 缺陷（见 11.7.3），接触对照结果有效；GPU 占用秒级 |
| 2 | 18:33:49–18:34:04（15.1 s） | 上游 CUDA witness 复采（修 launch 越界守卫） | 3706 MiB | 3609 MiB | 仍有 world 维读取问题，弃用 |
| 3 | 18:37:51–18:38:05（14.2 s） | 上游 CUDA witness 复采（改逐 world launch；warp JIT 13–14 s 为 CPU 时间） | 3701 MiB | 3701 MiB | **有效采集** `witness-cuda.json` |

`nvidia-smi --query-compute-apps` 三次跑前/跑后均无本任务进程（仅桌面应用）；峰值采样脚本
本轮未取到样本（`Start-Job` 相对路径缺陷，已修 `run_window_witness.ps1`），离散观测 + 探针
`cudaMemGetInfo` 前后 free 15109/14079 MiB 表明无异常增长；**不声称拿到逐秒峰值**。

### 11.7.1 ① `ellipsoid-box` 修复后复测（contact 数组，判据 = 同配方上游 f32 GPU）

| world | dist 设备 vs 参考 | pos | frame max_abs |
| --- | --- | --- | --- |
| 0 | −0.0099998917 vs −0.0099998899（**2 ULP**） | x/z 位级；y −3.2566240e-05 vs −3.2569260e-05（**绝对差 3.02e-9**） | 3.80e-7 |
| 1 | −0.0099998210 vs −0.0099998210（**0 ULP，位级**） | **全 3 分量位级** | 7.28e-10 |

- 修复前：dist 47 ULP、pos 绝对 8.3e-5；修复后 pos 残差 3.02e-9 **低于上游自身 CPU/GPU
  后端差 7.45e-9**（§11.4），x/z 与 world1 位级 → 结构差异已消除。
- world0 `pos.y`（参考 |3.26e-5|）按 **向量尺度 ULP** 判据（Arch 裁决，§11.6.2）=
  **0.81 ULP@scale(0.055)**、判过；旧口径下的 830 ULP 是「近零分量自身 ULP 无定义」的假象，
  不是精度问题。`result.json` 已按新判据重算：dist 1/2 位等（max 2 ULP）、pos 5/6 位等、
  frame max_abs 3.80e-7 → **✅ PASS**。

### 11.7.2 ② 三例回归（修复影响面实测，不许推断）

| case | count | dist | pos | frame | 结论 |
| --- | --- | --- | --- | --- | --- |
| hfield-sphere | 8/8 | 8/8 位级 | 24/24 位级 | 72/72 位级 | ✅ 未移动 |
| hfield-capsule | 7/7 | 7/7 位级 | 21/21 位级 | 63/63 位级 | ✅ 未移动 |
| sdf-box | 80/80 | 80/80 位级 | 203/240 位级，**1.0 ULP@scale(0.0983)**（旧逐分量口径 4 ULP） | 675/720 位级，max_abs 2.38e-7 | ✅ 与窗口 1 相同（pos 位等 202→203） |

### 11.7.3 ③ witness 对照（multicontact 三例：设备 `g5_ccd` vs 上游 CUDA 直调 `ccd()`）

参考：`capture_witness_ccd.py --device cuda`（直调 pinned `collision_gjk.ccd`，调用口径与上游
`eval_ccd_write_contact` 对齐：margin 覆盖、原点平移、`cutoff=gap`、迭代数取 meta）；
设备：`witness_probe.exe`（同一批 device 源、`--fmad=true`、`-O3`、`sm_89`）。按 **worldid 配对**
（设备数组按 world 索引写出、上游按显式 worldid 写出；逐 world 明细见 `witness-result.json.per_world`）。

| case | dist | x1 | x2 | ncontact | multiccd_idx | 结论 |
| --- | --- | --- | --- | --- | --- | --- |
| cylinder-box | 2/2 位级 | 5/6 位级，world1 max **0.5 ULP@scale(0.1)** | 6/6 位级 | 相等 | 6/8 与上游相同（仅信息） | ✅ ≤4 ULP |
| box-box-margin | 2/2 位级 | 6/6 位级（0.0 ULP） | 6/6 位级（0.0 ULP） | 相等 | −1 相同 | ✅ 全位级 |
| mesh-mesh | 2/2 位级 | 6/6 位级（0.0 ULP） | 6/6 位级（0.0 ULP） | 相等 | 1/2 相同 | ✅ 全位级 |

- 逐 world 实测：`cylinder-box` world0 全位级、world1 dist/x2 位级、x1 有 1 分量残差
  = 0.5 ULP@向量尺度；`box-box-margin`/`mesh-mesh` 两 world 的 dist/x1/x2 **全部位级相等**。
- 上游 CPU witness 仅诊断（跨后端）：dist max ULP = 1/0/5（CPU/CUDA 自身差，不作判据）。
- **首采 authoring 缺陷（留档，上游参考可信度依赖这条链）**：首采 CUDA 内核两 world 都产出
  world0（因此看起来像「world 配对反了」），根因三条：（1）warp CUDA 实际启动线程数可多于
  `dim`，内核需显式世界守卫；（2）`wp.array2d` 世界维读取在本 authoring 内核中不可靠；
  （3）多线程共享同一组 EPA 缓冲（上游产品是每 pair 独立 `epa_vert[ccdid]`，竞态会污染结果）
  → 改为「按 world 抽一维 pose 数组 + 逐 world launch（EPA 缓冲逐 pair 独占）」。前两次采集
  `witness-cuda.json` 已覆盖为有效版本；`window-witness-rerun{,2,3}-log.txt` 保留完整诊断链。
- 该对照**仅覆盖 `ccd()` witness**；multicontact 设备版仍未实装 → 接触数组仍不做全等（unmet 保留）。

### 11.7.4 本窗口结论与残留

- ① `ellipsoid-box`：结构根因（`v3_sign` 零值）修复生效 → 结构差异消除；残差 3.02e-9
  按 Arch 裁定的向量尺度判据 = 0.81 ULP（§11.6.2），**判过**。
- ② 三例回归全过（hfield 两例全位级、sdf-box 位级计数不变）。
- ③ witness 对照三例全部 ≤4 ULP（两条全位级、一条 0.5 ULP@scale）→ 原 `witness unmet` 清掉。
- 判据补全后 `result.json` / `witness-result.json` 全表重算：`failed_cases` 均为 **[]**；
  唯一「此前失败现在通过」= ellipsoid-box（修前缺陷数字在 §11.6.2 可执行断言下仍被拒绝）。
- 仍属 unmet：multicontact 设备版、NVRTC 产品 launcher、primitive narrowphase、
  性能基准（G6）、Linux/T4；窗口逐秒峰值未取到（见 §11.7 开头）。
