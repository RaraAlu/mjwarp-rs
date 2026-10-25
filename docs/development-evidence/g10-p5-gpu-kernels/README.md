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

## 4. 后续批次（尚未提交/未完成）

2. 设备 GJK phase/EPA（`simplex.cu` 已完成并通过 nvcc 编译，`polytope.cu`/`ccd.cu` + 宿主 launcher 待写）；
3. SDF narrowphase 设备版（initpoints/Halton/Armijo）；
4. hfield prism CCD 设备版（filter/prism/四接触/`MJ_MAXCONPAIR`）；
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