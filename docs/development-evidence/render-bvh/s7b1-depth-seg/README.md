# 渲染/BVH S7b1：主射线 / 平面深度 / 分割 · 证据

任务：**`2026-10-25-render-bvh-s7b1-kernel`**（plan S7b 拆分第一片；依赖 S2b/S3/S5/S6b/S7a，均 verified；
原 `…-depth-seg` 与 `…-render-kernel` 因运行态 files 范围不足被 abandon，本任务为目录级范围的最终登记）。
参考：冻结上游 MJWarp 3.15 `71da24d956378a87a703b6e1442b13aec0c4ac29` 的
`render.py::_render_megakernel` 主射线/深度/分割子集；S0b 冻结参考（fast_math=true）与
**同配方参考**（`use_fast_math=false`，Arch 裁决 B）。

本片只做**主射线阶段**：S5 预计算/动态射线 → 相机系→世界系（含 `max_cam_dist`）→ 场景 BVH + 网格 BVH
求交 → 平面深度（米，0=无命中）与 `(object_id, object_type)` 分割 → S6b `OutputResources`。
不含着色/RGB（S7b2）、不含 AA/flag 变体（S7b3）。

## 1. 结论

| 验收（任务 JSON） | 结果 |
| --- | --- |
| seg `(object_id, object_type)` 与冻结参考逐像素全等、背景 `(-1,-1)` | **通过**：cam0/cam1 各 76,800/43,200 像素 `seg_mismatch=0`（含 mesh geom 3 的 754 像素）；两份参考 seg 也逐像素相同 |
| 平面深度（米、0=无命中、不 clamp）与参考最大差 ≤4 ULP | **通过**：对同配方参考（`use_fast_math=False`）**两相机 120,000/120,000 像素逐位相同、max ULP 0**（cam0 `[76800,0,0,0,0,0]`、cam1 `[43200,0,0,0,0,0]`）；球面求交的运算顺序对齐见 §6 |
| `use_precomputed_rays=true` 与 dynamic 输出逐位一致 | **通过**：120,000 射线 depth 逐位 0 差异、seg 全等，hash `03efa95416bbd008` |
| primitive + 网格同池；双 world 各自 BVH 不跨；`set_points`+显式 refit 变化像素非空且两次逐位一致 | **通过**：primitive 与 mesh 同池；world 0 与单 world 渲染**逐位一致**、world 1 全 miss/全背景；refit `changed_pixels=1698`、两次运行逐位一致（hash `ce475440457773ff`） |
| 相机位姿（cam_xpos/cam_xmat）与 S0b rays 原点逐位断言 | **通过**：两相机 data `cam_xpos` 与 S0b `rays.npz` camera_pixels 原点位等（夹具 `camera_data_check`，转换脚本内自检）；camera 1 是 TARGETBODY 跟踪相机（见 §3） |
| 精简入口 `render_depth_seg`，证据给「子集槽→上游槽」映射；PTX `precisionGate: f32-render`，只走 `compile_render_artifact` | **通过**：32 槽（1 bounds + 24 数组 + 7 标量），映射表见 §5；PTX 137,824 B、`source_sha256=82ea0d3b…`、`ptx_sha256=01018ed9…`、恰好 2 条 W15 f64 死数据常量；PTX 同时含 `scene_bvh_ray`（S2b 只读复用）与 `render_depth_seg` 两个 entry |
| 禁用 depth/seg 保持 S6b `DisabledOutput`；RGB 本片不写 | **通过**：占位缓冲发射成功、`byte_len`/`read` 拒 `DisabledOutput`、RGB 全 0 |
| S6b 回归（GPU 5/5 + 全量测试）不回归 | **通过**：`render_context_lifecycle` 5/5，100 次 create/destroy growth **0 B**、drop retained **0 B**；`cargo test --locked` 97 个测试二进制全绿（532 passed / 0 failed，lib 392） |
| 门禁 fmt/clippy/clippy-cuda/unit | **通过** |
| 不改 `launch.rs`（恰 500 行）、不改 `src/render/bvh/**`、不改 S2–S6 证据夹具 | **通过**：`launch.rs` 仍 500 行；`scene.cu` 只读复用（sha 未变）；S2–S6 证据/夹具 `git status` 无改动 |
| GPU 90 分钟独占窗口、开始核对设备、结束回桌面基线 | **通过**：RTX 4070 Ti SUPER / 596.36 / SM89；窗口起点 4,193 MiB（桌面波动 4,0xx–4,2xx），结束 4,205 MiB、无 CUDA 计算进程；球面对齐修复后复跑 3,412 MiB → 3,412 MiB、仍无计算进程 |

GPU 用例 6 条：**6 passed / 0 failed**（4.43 s）；S6b 回归 5/5（7.56 s）。

## 2. 冻结参考与同配方参考（Arch 裁决 B）

S0b 冻结参考用 `PARAMS_28.use_fast_math=True` 的 render megakernel 捕获；产品 A′ 冻结
`WARP_1_15_RENDER` 为 `fast_math=false`。S0b 的 Warp 内核缓存 PTX（
`target/render-bvh/s0b-cache/1.15.0/wp__build_megakernel__locals___render_megakernel_f77904cd_be6b38f/…sm89.ptx`，
619,077 B，sha256 `9662200b…`）的**逐指令精确统计**（Arch 已独立复核）：`div.approx.ftz` **267**、
`sqrt.approx.ftz` **77**、`rsqrt.approx.ftz` **3**、`rcp.approx.ftz` **37**、`div.full.ftz` 3，
`.rn` 变体（div.rn / sqrt.rn / rcp.rn）**全部为 0** —— 参考 megakernel 确实是 fast-math 编译。

按 Arch 裁决 B：**depth ≤4 ULP 以「同配方参考」（`use_fast_math=False`）为硬指标依据**；S0b fast-math
参考继续作为 seg 的源；跨配方差异**只记录为已知偏差，不作通过依据、不放宽容差**；若同配方参考下仍
>4 ULP，停下上报。修复前确曾触发该停止条件（149 像素 >4 ULP，已上报），根因定位后按上游运算顺序修复
（§6），现「同配方参考 120,000/120,000 位等、max 0 ULP」，容差与配方均未改动。

同配方参考由本片新增 authoring 脚本捕获（同一 pinned 3.15 环境、同一 `scene-315.mjb`、仅
`use_fast_math` 置 False，不换场景/不重采射线）：

| 项 | 值 |
| --- | --- |
| 夹具 | `fixtures/render-bvh/scene-a/s7b/depth_seg_fast_math_false.json` |
| 字节 / sha256 | 1,492,762 B / `d6ab6dde58034df5c1bc69ab397026a8f7c3aaf58d29d25edb032d3818000dc7` |
| 脚本 | `scripts/authoring/render-bvh-s7b/capture_fast_math_false.py`（15,791 B，sha256 `29ae04ac…`） |
| 生成时间 | 2026-10-25T03:29:25Z |
| 确定性 | 同进程两次 `mjwarp.render` 逐位一致（`determinism.bitwise_identical=true`） |
| 编译口径证据（kernel_cache，逐指令精确统计） | 同配方 megakernel PTX 637,336 B sha256 `ad21b93f…`：`div.rn` **206**、`div.approx.ftz`/`div.full.ftz` **0**、`sqrt.rn` **77**、`sqrt.approx.ftz` **0**、`rsqrt.approx.ftz` **3**、`rcp.rn` 89、`rcp.approx.ftz` 7（对比 S0b fast-math：div.approx.ftz 267 / sqrt.approx.ftz 77 / rsqrt 3 / rcp.approx 37 / .rn 全 0） |
| 与 S0b 的跨配方差异 | 参考↔参考：cam0 max 2 ULP（超 4 ULP=0）、cam1 max 4 ULP（超 4 ULP=0）；seg 两相机逐位相同 |

S0b 参考夹具 `depth_seg.json`：1,491,460 B / sha256 `97fc6838…`（含 depth/seg 全图位型 + 相机 data 位姿；
上一版 1,490,571 B / `f193bc5a…` 因补入相机 data 位姿被取代，depth/seg 数组未变）。

产品实测（同一帧）：
- **对同配方参考**：cam0 `[76800,0,0,0,0,0]`、cam1 `[43200,0,0,0,0,0]` —— **两相机全部 120,000 像素位等、max 0 ULP**；
- **对 S0b fast-math 参考**（已知偏差，不作通过依据）：cam0 `[57726,17681,1393,0,0,0]` max 2 ULP；
  cam1 `[39136,3677,382,3,2,0]` max 4 ULP，超 4 ULP=0。

由于产品已与同配方参考逐位相同，后一组数字与夹具里的 `cross_recipe_vs_s0b`（参考↔参考）**完全一致**：
跨配方差异只是 fast-math 编译噪声（≤4 ULP，无 >4 ULP 像素），与产品实现无关；测试把它固定成回归守卫
（断言直方图逐项相同）。球面切线带原先那 149 个 >4 ULP 像素的根因与修复见 §6。

## 3. 相机 data 位姿（关键发现）

scene-a 的 camera 1 是 **TARGETBODY 跟踪相机**（`cam_mode=[0,3]`、`cam_targetbodyid=[-1,2]`，XML `cam-track`），
data `cam_xmat` 是 `mjwarp.forward` 的跟踪 look-at 产物，**不是** `quat_to_mat(cam_quat)`（其 cam_quat 是占位
identity）。按 quat 推导会让 camera 1 变成「朝下看地板」：全画面只命中 floor plane、depth 恒 0.7，与参考
（cam1 命中 geom 0..5、含 mesh）完全不符——本片 GPU 首次对照时即由此暴露。

渲染器因此接受**显式** `&[CameraPose]`（按模型相机 id 索引），由调用方按上游 data 语义提供；固定相机可用
`CameraPose::from_fields`（`f32(cam_pos)` + `quat_to_mat(f32(cam_quat))`）派生。跟踪相机的 data 位姿计算
复用产品现有 `physics::camlight` 组件（其 RenderContext 接线登记在 S8 端到端任务
`2026-10-25-render-bvh-s8-e2e`），本片不从 `cam_quat` 重实现第二份跟踪相机数学（Arch 已确认）。

**证据边界（Arch 明确要求）**：
- S7b1 的相机 data 位姿由**调用方**按上游 data 语义传入；跟踪相机 data 位姿取自 pinned 3.15
  `mj_forward(scene-315.mjb)` 的冻结值（CPU authoring，不重采、不占 GPU）。
- 本片的「两相机全图 seg/depth 与参考一致」**不等于「产品跟踪相机已接通」**；它只证明渲染器在
  data 位姿正确时与上游等价。
- 跟踪相机数学的产品侧接线归 **S8 端到端**（`2026-10-25-render-bvh-s8-e2e`，files 含
  `src/physics/camlight.rs`）；S8 验收要求相机 data 位姿必须由产品 camlight 产出、夹具位姿只能用于
  比对不能当渲染输入，并断言 camera1 与参考 data xmat 一致。本片不宣称该接线已完成。

夹具 `camera_data_check`（转换脚本用 pinned 3.15 `mjwarp.forward` 读 f32 位型并自检）：

| cam | mode | target | origin_bits | xmat_bits（行主序 f32 位型） | xmat vs f64 复算误差 |
| --- | --- | --- | --- | --- | --- |
| 0 | 0 fixed | -1 | `[1067030938,3213675725,1061997773]` | `[1060761557,3206302579,3198267176,1057406113,1061821170,3198267176,1055021982,1031887085,1063587192]` | 7.529e-08 |
| 1 | 3 targetbody | 2 | `[3209481421,1063675494,1060320051]` | `[3207690738,1030141075,3208113682,3208150666,3177049595,1060171513,0,1065301899,1033899914]` | 5.615e-08 |

自检（失败即不落盘）：origins 与 S0b `rays.npz` camera_pixels 位等、与 data `cam_xpos` 位等；
fixed 相机 xmat vs f64 `quat_to_mat` ≤1e-6；TARGETBODY xmat vs f64 look-at（`z=normalize(pos-target)`、
up=world z）≤1e-6。MJWarp f32 data 与 `f32(native f64)` 有若干元素差 1 ULP，故用 **MJWarp forward f32 位型**。

## 4. 冻结语义（已用 S0b 参考数值钉死）

- **平面深度**：`depth = dist * -ray_dir_local_cam[2]`；`dist` 沿**未重归一化** world 方向
  `cam_xmat @ dir_local` 的参数，`-dir_z = cos(光轴夹角)`；**0 = 无命中**（未归一化、未 clamp）。
- **zfar 裁剪**：`max_cam_dist = zfar / max(-dir_local_z, 1e-6)`（`zfar>0`，scene-a 125.0）；
  命中后 `dist*-dir_local_z > zfar` 按 miss；无命中写 0。
- **分割**：命中 `(geom_id, 5)`（`mjOBJ_GEOM=5`，pinned 3.15 实测；flex 才是 9）；背景 `(-1,-1)`；
  上游 `render()` 先 `seg_data.fill_(-1,-1)` 再只写命中，本片内核每次渲染显式写背景，等价。
- **cast 组合**：查询上界 `dist*(1+1e-6)+1e-9`；透明 `rgba[3]==0` 跳过；背面剔除 `dot(dir,n)>0` 置 miss；
  近平局 `tol=1e-6*max(dist,1)` 取更大 geom id；primitive 与 mesh 同池。
- **AA**：depth/seg 只在 `ray_base==0` 写；本片 `samples_per_pixel=1`（`nsamples>1` 直接拒绝，S7b3）。
- **数值核验**（`scripts/authoring/render-bvh-s7b/check_depth_semantics.py`，CPU）：S0b 74 个命中像素，
  与欧氏预测最大相对差 **4.268e-01**、与平面预测 **6.085e-06**（max abs 8.464e-06，19/74 位等）→ 平面深度。
- 参考 seg 分布：cam0 `(0,5)×59,805 + (-1,-1)×16,995`；cam1 `(-1,-1)×27,498、(0,5)×10,720、(1,5)×2,202、
  (4,5)×852、(3,5)×754、(2,5)×747`。

## 5. 实现与 32 槽 → 上游 81 槽映射

| 文件 | 行数 | 内容 |
| --- | ---: | --- |
| `src/render/kernel/cast.rs` | 494 | `CameraPose`（data 位姿）、`ModelRenderArrays`、`SceneDevice`/`MeshDevice`/`CameraDevice` 打包 |
| `src/render/kernel/renderer.rs` | 265 | `DepthSegRenderer`：context 拥有 BVH → device 打包 → 32 槽实参 → 发射；禁用通道占位 |
| `src/render/kernel/mod.rs` | 429 | `DEPTH_SEG_SLOTS`、`DepthSegKernel`（唯一编译入口）、源码拼接、单测 |
| `src/render/kernel/cuda/render.cu` | 499 | 世界变换、dynamic `compute_ray`、组合 cast、深度/分割写入、ABI 文档（球面分支转 `render_intersect_primitive`） |
| `src/render/kernel/cuda/render_math.cu` | 309 | S3 mesh woop 数学逐式复制（仅 `normalize3` 改名）+ 球面求交的上游运算顺序对齐（§6） |
| `src/render/kernel/output.rs` | 450 | 两份参考解析（含相机 data 位姿/同配方 recipe 段）、`diff_depth`/`diff_seg` |
| `src/render/context/lifecycle.rs` | 416 | `pub(crate)` 渲染实参 + 场景/网格 BVH 访问器（公共 API 未变） |
| `src/render/context/owner.rs` | 408 | `MeshResources::bvh()`（`pub(crate)`，1 行，不扩公共 API） |
| `tests/render_kernel_depth_seg.rs` | 422 | 2 主机 + 1 NVRTC 门禁 + 5 GPU 用例 |
| `tests/support/render_bvh/s7b1.rs` | 469 | 场景/相机/射线夹具与 ULP 直方图统计（纯主机） |
| `scripts/authoring/render-bvh-s7b/convert_depth_seg.py` | 332 | S0b 夹具 + 相机 data 位姿转换 |
| `scripts/authoring/render-bvh-s7b/capture_fast_math_false.py` | 362 | 同配方参考捕获（authoring；一次 GPU forward+render） |
| `scripts/authoring/render-bvh-s7b/check_depth_semantics.py` | 131 | 深度语义/分割类型核验（CPU） |

**源码拼接**（`mod.rs::DEPTH_SEG_SOURCE`）：`concat!(S2b scene.cu, cuda/render_math.cu, cuda/render.cu)`：
组装后 **51,482 B / sha256 `82ea0d3bea34da898e3e1ee15564ddef9462fdc7229ed27e619c7c19dbf02bdc` / 1,270 行**
（462 + 309 + 499）；`scene.cu` 只读、sha256 `8dad4d61…` 未变（17315 B）；`render_math.cu` 12127 B /
sha256 `73dd8e1e13f2b611ee79f9741f289e6c7497631dbe46d9ca7056c427b530cc09`；`render.cu` 22040 B /
sha256 `37494a09f218cc890c9f8a3fa4b2d53a0b39918d67bd6caa19e140add61c5237`。PTX 同时含 `scene_bvh_ray` 与
`render_depth_seg` 两个 entry（互不发射）。

**「子集槽 → 上游 `_render_megakernel` 槽」映射**（上游槽号 1-81，S7a 冻结签名；本片槽序即上游子集顺序）：

| 本片槽 | 类型 | 上游槽（S7a `MEGAKERNEL_SLOTS` 1-based） |
| --- | --- | --- |
| 1 | `launch_bounds_t<2>` | 1 |
| 2-6 | scene_meta/scene_params/mesh_meta/mesh_params/mesh_offsets | **本片自有**（本片展开的 BVH/网格设备布局；上游是 `bvh_id`/`mesh_bvh_id` 句柄，槽 55/60） |
| 7 | geom_type(i32) | 2 |
| 8 | geom_rgba(vec4) | 6 |
| 9 | geom_matid(i32) | 4 |
| 10 | mat_rgba(vec4) | 32 |
| 11 | cam_projection(i32) | 7 |
| 12 | cam_fovy(f32) | 8 |
| 13 | cam_sensorsize(vec2) | 9 |
| 14 | cam_intrinsic(vec4) | 10 |
| 15 | cam_xpos(vec3) | 35 |
| 16 | cam_xmat(mat33) | 36 |
| 17 | cam_res(vec2i) | 44 |
| 18 | depth_adr(i32) | 50 |
| 19 | seg_adr(i32) | 51 |
| 20 | render_depth(bool) | 53 |
| 21 | render_seg(bool) | 54 |
| 22 | ray(vec3) | 46 |
| 23 | ray_offset(vec3) | 47 |
| 24 | depth_out(f32) | 80 |
| 25 | seg_out(vec2i) | 81 |
| 26 | nrender(i32) | 40 |
| 27-30 | znear_bits/zfar_bits/depth_stride/seg_stride(i32) | **上游无**（前两者是 `wp.static` 编译期常量，以 f32 位型传入；后两者是上游 `array2d` pitch 的显式化） |
| 31 | use_precomputed_rays(bool) | **上游无**（`wp.static`） |
| 32 | cull_backfaces(bool) | **上游无**（`wp.static rc_static`） |

**本片不宣称与上游 megakernel 等价/ABI parity**：这是 S7b1 的精简入口（只含真实数据），81 槽全签名
迁移归 S7b2（S7a 的 81 槽 ABI 验收不受影响）。

## 6. 球面求交运算顺序对齐（根因与修复）

**现象**（修复前）：对同配方参考，cam0 全 76,800 像素位等；cam1 有 149/43,200 像素超 4 ULP（max 73 ULP），
全部落在 sphere（geom 2）切线带（cam1 x≈169-176、y≈120-122 附近，深度 1.29-1.40 m）；seg 全等。两份上游
参考（fast-math / 同配方）在这些像素上彼此只差 ≤1 ULP，说明参考侧稳定、差异在产品内核。

**根因**：产品渲染配方是冻结的 CUDA 12.8.1 NVRTC / `compute_70`；参考 megakernel 由 warp 1.15 自带 NVRTC
（CUDA toolkit 12.9、sm_89）编译。同一 C++ 表达式在两套编译器下收缩顺序不同。逐指令读参考 PTX
（`…render_megakernel_92d94e46….sm89.ptx` 第 529-531、940-949 行）得到它实际发射的球面序列：

| 量 | 参考 megakernel PTX 的运算顺序 |
| --- | --- |
| `a = dot(vec, vec)` | `mul(d0*d0)` → `fma(d1*d1)` → `fma(d2*d2)`（0 号分量先乘） |
| `b = dot(vec, dif)` | `mul(d1*dif1)` → `fma(d0*dif0)` → `fma(d2*dif2)`（1 号分量先乘） |
| `dd = dot(dif, dif)` | `mul(dif1*dif1)` → `fma(dif0*dif0)` → `fma(dif2*dif2)` |
| `det = b*b - a*c` | `fma(b, b, -(a*c))`（融合） |

产品编译器对同一表达式给出的是：`a` 用「1 号分量先乘」、`det` 不融合。球面切线处 `det ≈ 4e-5`、
近根敏感度 `d(dist)/d(det) ≈ -25…-78`，这些 1 ULP 级差异被放大到几十 ULP。用离线 GPU 变体探针
（`target/gpu_sphere_probe.py`，本地诊断、不入库）穷举 matvec/dot/det/分母的显式运算顺序：上表序列在
747 个 cam1 球面像素上 **747/747 位等、max 0 ULP**；产品默认序列为 564/747 位等、149 个 >4 ULP —— 根因确认。

**修复**（本片范围内；不改 `scene.cu`、不改配方、不放宽容差）：`cuda/render_math.cu` 新增
`render_sphere_hit`，用显式 `__fmul_rn`/`__fmaf_rn`/`__fsub_rn` 固定上表的运算顺序（相机世界变换仍复用
产品现有序列，与参考一致）；`cuda/render.cu` 的 `cast_ray` 对 `geom_type == 2` 改调
`render_intersect_primitive`，其余 primitive 与 mesh 路径完全不变；`scene.cu` 只读、哈希未变。

**修复后实测**：

```
cargo test --locked --features cuda-probe --test render_kernel_depth_seg -- --ignored --test-threads=1 --nocapture
```

| 用例 | 结果 |
| --- | --- |
| `depth_seg_source_compiles_under_render_gate` | ok（PTX 137,824 B；两个 entry；2 条 W15 f64 死数据常量） |
| `depth_and_seg_match_frozen_reference_for_both_cameras` | **ok**：对同配方参考 cam0 `[76800,0,0,0,0,0]`、cam1 `[43200,0,0,0,0,0]` 全位等、max 0 ULP、seg 全等；对 S0b fast-math 参考只记录跨配方偏差（≤4 ULP、>4 ULP=0，与夹具 `cross_recipe_vs_s0b` 直方图逐项相同） |
| `dynamic_rays_render_bitwise_identical_to_precomputed` | ok（hash `03efa95416bbd008`） |
| `two_worlds_do_not_cross_hit` | ok |
| `mesh_refit_changes_pixels_and_repeat_runs_are_bitwise_identical` | ok（`changed_pixels=1698`，hash `ce475440457773ff`） |
| `disabled_depth_and_seg_keep_s6b_rejections` | ok |

GPU 用例 **6 passed / 0 failed**（4.43 s；复跑起点 3,412 MiB → 结束 3,412 MiB，无 CUDA 计算进程）。

S6b 回归（同窗口）：`cycle100 growth=0 B（阈值 16,777,216，elapsed 6,616 ms，created/destroyed 各 +100）`、
`drop retained=0 B（warmup_retention=33,554,432 B 一次性驱动缓存）`、陈旧 BVH→显式 refit 命中、
`DisabledOutput` 拒绝、纹理采样 `[0.49803925,0.14901961,0.49803925,1.0]` 与 S6b 记录一致（5/5，7.56 s）。
## 7. 门禁

| 门禁 | 结果 |
| --- | --- |
| fmt | 通过 |
| clippy | 通过（`--all-targets --locked -- -D warnings`） |
| clippy-cuda | 通过（`--features cuda-probe -- -D warnings`） |
| unit | 通过：`cargo test --locked` 97 个测试二进制 ok / 0 failed（532 passed，lib 392） |

## 8. 未满足项 / 边界

- depth 硬指标已满足（对同配方参考 max 0 ULP、120,000/120,000 位等）。球面求交的运算顺序对齐属本片
  渲染内核范围（§6）；S7b2 着色/法线若也遇到同源编译器差异，沿用「按参考 PTX 用显式 fma 序列固定顺序、
  不放宽容差」的做法，不在本片预判。
- 相机 data 位姿接线：渲染器接受显式 `CameraPose`；跟踪相机 data 位姿复用现有 `physics::camlight`，
  其 RenderContext 接线登记在 S8 端到端任务（Arch 已确认）。
- 双 world 用例相机位姿按激活相机单份打包（world 维共享）；scene-a 参考 `nworld=1`，未覆盖逐 world 相机数据。
- 未覆盖：正交相机（scene-a 两相机均为 perspective；正交 path 由 S5 覆盖）、AA（`nsamples>1` 拒绝，S7b3）、
  SDF（type 8，scene-a 无）、透明 geom（alpha 全 1，仅结构实现）、flex/hfield/skybox/splat/阴影/着色（非本片范围）。
- 未跑完整 G01（团队规则：开发迭代只跑定向 + fmt/clippy/clippy-cuda/unit；里程碑由用户点名）。

## 9. 复现命令

```
# S0b 夹具 + 相机 data 位姿（authoring；一次 GPU forward，不重采参考）
target/mjwarp-parallel/venv/Scripts/python.exe -B scripts/authoring/render-bvh-s7b/convert_depth_seg.py
# 同配方（use_fast_math=False）参考（authoring；一次 GPU forward+render）
target/mjwarp-parallel/venv/Scripts/python.exe -B scripts/authoring/render-bvh-s7b/capture_fast_math_false.py
# 深度语义核验（CPU）
target/mjwarp-parallel/venv/Scripts/python.exe -B scripts/authoring/render-bvh-s7b/check_depth_semantics.py
# 门禁
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo clippy --all-targets --locked --features cuda-probe -- -D warnings
cargo test --locked
# 定向（GPU + NVRTC）
cargo test --locked --features cuda-probe --test render_kernel_depth_seg -- --ignored --test-threads=1 --nocapture
cargo test --locked --features cuda-probe --test render_context_lifecycle -- --ignored --test-threads=1
```

## 10. 环境

| 项 | 值 |
| --- | --- |
| GPU / 驱动 / arch | NVIDIA GeForce RTX 4070 Ti SUPER / 596.36 / SM89 |
| CUDA toolkit（产品） | `target/toolchains/cuda-12.8.1`（compute_70 产物） |
| pinned authoring | `target/mjwarp-parallel/venv`：Python 3.12.13 / warp 1.15.0 / mujoco & mujoco_warp 3.15.0 / numpy 2.5.4 |
| 上游 | `71da24d956378a87a703b6e1442b13aec0c4ac29`（clean） |
| 显存 | 90 分钟窗口起点 4,193 MiB → 结束 4,205 MiB；球面对齐修复后复跑 3,412 MiB → 3,412 MiB；均无 CUDA 计算进程 |
