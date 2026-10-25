# 渲染/BVH S7b2：81 槽全签名 megakernel 着色/RGB

任务：`2026-10-25-render-bvh-s7b2-shading`（渲染/BVH S7b2）
冻结上游：`71da24d956378a87a703b6e1442b13aec0c4ac29`（MJWarp 3.15）
精度门禁：`f32-render`（A′ `check_render_ptx`，编译只走 `compile_render_artifact`）
GPU：NVIDIA GeForce RTX 4070 Ti SUPER / 驱动 596.36 / SM89 / 最终采集窗口起止显存均 3335 MiB（桌面基线，无计算进程残留）

## 1. 结论

- 81 槽 `_render_megakernel` 全签名迁移完成（1 bounds + 73 数组 + 7 标量），逐槽与 S7a 冻结
  `MEGAKERNEL_SLOTS` 一致；内部自有 BVH 打包经 `bvh_id`（u64 设备指针）传入，**不宣称与 Warp BVH
  等价/互操作**。
- **RGB 硬指标达成且超出**：对**同配方**（`use_fast_math=False`）参考，两相机
  **120,000 / 120,000 像素逐位相同（0 像素差异、max byte diff 0）**（判据 ≤1 LSB）。
- **depth/seg 不回归**：与 S7b1 同配方参考两相机 **120,000 / 120,000 位等、max 0 ULP、seg 0 不一致**。
- dynamic（`use_precomputed_rays=false`）本片写前拒绝；两次运行 RGB/depth/seg 逐位相同。
- 灯位姿：3 盏灯 `xpos`/`xdir` 6 组 f32 位型与 pinned 3.15 `mjwarp.forward` **逐位相等**。
- 门禁：`cargo fmt --check` 通过；`cargo clippy --all-targets --locked --features cuda-probe -- -D
  warnings` 通过；`cargo test --locked` **532 passed / 0 failed（98 个测试二进制，lib 392）**；
  S6b 回归 `tests/render_context_lifecycle.rs` **5/5**；S7b1 GPU 回归 **6/6**；S7b2 GPU **3/3**。
- PTX（最终源码）：**195,594 B**，`source_sha256=0fcdb02ac4687780e307005b69f8eb08ec2932e5b1bdc57a08aefc599cb3e42b`，
  `ptx_sha256=75a6ad35ed7ba139d788b74bd3bf9420bdfd2eef8c8f6e4026b6dc7c2db62eb9`，A′ 门禁通过。

## 2. 范围与提交边界

- 新增：`src/render/kernel/cuda/{render_common.cu,render_shade.cu,render_megakernel.cu}`、
  `src/render/kernel/{megakernel.rs,megakernel_arrays.rs}`、`tests/render_kernel_rgb.rs`、
  `tests/support/render_bvh/s7b2.rs`、`scripts/authoring/render-bvh-s7b/capture_rgb_fast_math_false.py`、
  `fixtures/render-bvh/scene-a/s7b/{rgb.json,rgb_fast_math_false.json}`、本证据目录。
- 修改：`src/render/kernel/mod.rs`（`MEGAKERNEL_SOURCE` 拼接 + 导出）、
  `src/render/context/lifecycle.rs`（`*_render_arg_2d`，`pub(crate)`，S6b 语义不变）。
- 依赖（独立配套任务，先于本片提交）：`src/render/context/owner.rs` 的
  `TextureResources::handles()` 只读访问器（6 行、`pub(crate)`）由
  **`2026-10-25-render-bvh-texture-handles`** 任务单独提交（提交 `a96a158`）：`TextureResources.objects`
  为 owner 模块私有字段，S7b2 范围内的文件无法访问，故按 Arch 裁决把该访问器拆为独立任务，
  避免同一提交跨两个任务的范围。
- 只读未动：`src/render/kernel/launch.rs`（恰 500 行）、`src/render/bvh/**`（含 S2b `scene.cu`，
  sha `8dad4d61…` 未变）、`fixtures/render-bvh/scene-a/reference/**`、S2–S6 证据与夹具、`Cargo.*`。

## 3. 内核结构

`MEGAKERNEL_SOURCE = concat(scene.cu, render_math.cu, render_common.cu, render_shade.cu,
render_megakernel.cu)`；入口 `render_megakernel`（81 槽）。

- `scene.cu`（S2b，只读复用）：`V3` 辅助、`ray_*` primitive 求交、`intersect_primitive`。
- `render_math.cu`（S7b1，本片不动）：mesh woop 求交、`render_sphere_hit`（球面 fma 顺序对齐）、
  `normalize3_mesh`、`dot3_ba`/`dot3_ch0`。
- `render_common.cu`：`compute_ray`、`MegakernelBlob`/`SceneView`、mesh leaf 遍历、
  `cast_ray`（透明跳过、背面剔除、近局 tol、primitive+mesh 同池）、`render_geom_rgba`。
- `render_shade.cu`：5 类纹理 UV（plane/sphere-ellipsoid/cylinder/capsule/box + mesh fallback）、
  `sample_texture`（wrap + `wp::texture_sample`）、`vertex_normal`、`compute_lighting`、
  `pack_rgba_to_uint32`。
- `render_megakernel.cu`：81 槽入口、相机选择（`cam_res` 累积）、主射线/深度/分割、着色组合
  （emission → headlight/per-light ambient → 逐灯 diffuse/specular → headlight diffuse/specular
  → clamp → pack）。

## 4. 81 槽逐槽表（槽序 = S7a 冻结上游顺序）

`real` = 填真实数据；`placeholder` = 1 元素占位（内核不读，仅 ABI 对齐）；`own` = 本实现自有打包
（占用上游本片未使用的槽，语义见 §4.1）。

| slot | 上游形参 | 填值 | 类别 |
| --- | --- | --- | --- |
| 0 | `launch_bounds_t<2> dim` | `[nworld=1, total_rays=120000]` | real |
| 1 | `geom_type` | `RenderFields.geom_type` | real |
| 2 | `geom_dataid` | `RenderFields.geom_dataid` | real |
| 3 | `geom_matid` | model `geom_matid` | real |
| 4 | `geom_size` | `RenderFields.geom_size`（`D` f64 → f32） | real |
| 5 | `geom_rgba` | model `geom_rgba` | real |
| 6 | `cam_projection` | `RenderFields.cam_projection` | real |
| 7 | `cam_fovy` | `RenderFields.cam_fovy`（f64 → f32） | real |
| 8 | `cam_sensorsize` | `RenderFields.cam_sensorsize`（Vec2） | real |
| 9 | `cam_intrinsic` | `RenderFields.cam_intrinsic`（Vec4） | real |
| 10 | `light_type` | `RenderFields.light_type` | real |
| 11 | `light_castshadow` | `light_castshadow`（Flag → bool words） | real |
| 12 | `light_active` | `light_active`（Flag → bool words） | real |
| 13 | `light_attenuation` | `light_attenuation`（Vec3） | real |
| 14 | `light_cutoff` | `light_cutoff`（F） | real |
| 15 | `light_exponent` | `light_exponent`（F） | real |
| 16 | `light_ambient` | `light_ambient`（Vec3） | real |
| 17 | `light_diffuse` | `light_diffuse`（Vec3） | real |
| 18 | `light_specular` | `light_specular`（Vec3） | real |
| 19 | `flex_vertadr` | `[0]` | placeholder（flex 未实现） |
| 20 | `flex_edge` | `[0,0]`（Vec2I） | placeholder |
| 21 | `flex_radius` | `[0.0]` | placeholder |
| 22 | `mesh_faceadr` | `RenderFields.mesh_faceadr` | real |
| 23 | `mesh_normaladr` | `RenderFields.mesh_normaladr` | real |
| 24 | `mesh_normal` | `RenderFields.mesh_normal` | real |
| 25 | `mat_texid` | `mat_texid [1,nmat,10]` | real |
| 26 | `mat_texuniform` | `mat_texuniform`（Flag → bool words） | real |
| 27 | `mat_texrepeat` | `mat_texrepeat`（Vec2） | real |
| 28 | `mat_emission` | `mat_emission`（F） | real |
| 29 | `mat_specular` | `mat_specular`（F） | real |
| 30 | `mat_shininess` | `mat_shininess`（F） | real |
| 31 | `mat_rgba` | model `mat_rgba` | real |
| 32 | `geom_xpos_in` | context 场景状态（`SceneDevice` 每 geom 位姿逐 geom id 反解） | real |
| 33 | `geom_xmat_in` | 同上 | real |
| 34 | `cam_xpos_in` | 调用方显式 `CameraPose`（data 语义） | real |
| 35 | `cam_xmat_in` | 同上 | real |
| 36 | `light_xpos_in` | `light_data_poses`（`RenderFields` 固定灯公式推导） | real |
| 37 | `light_xdir_in` | 同上 | real |
| 38 | `flexvert_xpos_in` | `[0,0,0]` | placeholder |
| 39 | `nrender`（scalar i32） | `2` | real |
| 40 | `use_shadows`（scalar bool） | `false`（true 由 host 写前拒绝） | real（固定 false） |
| 41 | `bvh_ngeom`（scalar i32） | `7` | real |
| 42 | `bvh_nflexgeom`（scalar i32） | `0` | real |
| 43 | `cam_res` | 扁平 `[320,240,240,180]`（Vec2I） | real |
| 44 | `cam_id_map` | `resolved.cam_id_map()` | real |
| 45 | `ray` | S5 预计算射线方向（120,000） | real |
| 46 | `ray_offset` | S5 预计算射线偏移 | real |
| 47 | `ray_base`（scalar i32） | `0` | real |
| 48 | `rgb_adr` | `OutputLayout.rgb_adr` | real |
| 49 | `depth_adr` | `OutputLayout.depth_adr` | real |
| 50 | `seg_adr` | `OutputLayout.seg_adr` | real |
| 51 | `render_rgb` | 每相机 bool | real |
| 52 | `render_depth` | 每相机 bool | real |
| 53 | `render_seg` | 每相机 bool | real |
| 54 | `bvh_id`（scalar u64） | **自有 blob 设备地址**（§4.1） | own（real 指针） |
| 55 | `group_root` | `[0]`（内核不读） | placeholder |
| 56 | `flex_bvh_id` | `[0,0]`（u64 as 2×u32） | placeholder |
| 57 | `flex_group_root` | `[0]` | placeholder |
| 58 | `enabled_geom_ids` | `resolved.enabled_geom_ids`（空则 `0..ngeom`） | real |
| 59 | `mesh_bvh_id` | `[0,0]`（mesh 遍历走自有 blob） | placeholder |
| 60 | `mesh_facetexcoord` | `RenderFields.mesh_facetexcoord`（Vec3I） | real |
| 61 | `mesh_facenormal` | `RenderFields.mesh_facenormal`（Vec3I） | real |
| 62 | `mesh_texcoord` | 2 个 `0.0` 占位（场景无 texcoord；ABI 拒绝零长数组） | placeholder |
| 63 | `mesh_texcoord_offsets` | `RenderFields.mesh_texcoordadr` | real |
| 64 | `hfield_bvh_id` | `[0,0]` | placeholder |
| 65 | `flex_rgba` | `[0;4]` | placeholder |
| 66 | `flex_geom_flexid` | `[0]` | placeholder |
| 67 | `flex_geom_edgeid` | `[0]` | placeholder |
| 68 | `skybox_tex_id` | `[0]`（skybox 写前拒绝） | placeholder |
| 69 | `skybox_face_width` | `[0]` | placeholder |
| 70 | `textures` | S4 纹理句柄（`Texture2dArray`）；`use_textures=false` 时 1 元素 dummy | real（禁用时 placeholder） |
| 71 | `splat_position` | **own**：`[headlight_ambient, headlight_diffuse, headlight_specular, background]`（4×vec3） | own |
| 72 | `splat_rotation` | `[0;4]`（Quat） | placeholder |
| 73 | `splat_scale` | **own**：`[znear, zfar, 0]`（vec3，仅 `.y`=zfar 被读） | own |
| 74 | `splat_rgba` | `[0;4]` | placeholder |
| 75 | `splat_bvh_id`（scalar u64） | **own**：`(packed_background_u32 << 32) | flags`（§4.1） | own |
| 76 | `splat_group_root` | `[0]` | placeholder |
| 77 | `rgb_out` | `OutputResources` RGB array2d（`nworld × rgb_pixels`） | real |
| 78 | `aa_accum_out` | `[0;3]`（spp=1 不写） | placeholder |
| 79 | `depth_out` | `OutputResources` depth array2d | real |
| 80 | `seg_out` | `OutputResources` seg array2d | real |

统计：real 数据槽 54、自有打包槽 3（54/71/73 位、75）、placeholder 24（其中 62 因 ABI 禁零长数组）。
`bvh_ngeom=7`、`bvh_nflexgeom=0`、`use_shadows=false` 与冻结口径一致。

### 4.1 自有打包（仅签名对齐，内部语义自有）

- `bvh_id`（slot 54）：单个设备指针指向自有 blob；header 10×u32（五段字节偏移/长度：scene_meta、
  scene_params、mesh_meta、mesh_params、mesh_offsets，段间 16 字节对齐），随后五段数据。布局与
  S7b1 `SceneDevice`/`MeshDevice` 的 meta/params 完全一致（host 单测按偏移切分校验）。
  **不消费 Warp BVH 句柄，不宣称与上游 BVH 等价。**
- `splat_bvh_id`（slot 75）：低 32 位 flags（bit0 textures、1 vertex_normals、2 specular、
  3 emission、4 per_light_ambient、5 ambient_lighting、6 headlight、7 cull_backfaces），
  高 32 位 = `packed_background`（ABGR u32）。
- `splat_position`/`splat_scale`：见上表；上游本片不使用这些槽（splat 未实现）。

## 5. 验收结果

用例：`cargo test --locked --features cuda-probe --test render_kernel_rgb -- --ignored --test-threads=1 --nocapture`

### 5.1 RGB（硬指标：同配方参考，解包后每通道 ≤1 LSB）

- cam0（320×240）：`histogram(0,1,2,3,>3)=[76800,0,0,0,0]`，**0 像素差异**，max byte diff 0。
- cam1（240×180）：`histogram(0,1,2,3,>3)=[43200,0,0,0,0]`，**0 像素差异**，max byte diff 0。
- 合计 **120,000 / 120,000 像素逐位相同**。

### 5.2 跨配方偏差记录（**不作通过依据**）

对 S0b `PARAMS_28.use_fast_math=True` 参考：cam0 `[76798,2,0,0,0]`（2 像素差 1 byte）、cam1
`[43200,0,0,0,0]`（全位等）—— 与采集脚本记录的 `cross_recipe.per_camera` **逐项一致**
（同配方参考 vs S0b 参考：cam0 也是 2 像素差 1 byte、cam1 全位等）。说明参考侧 fast-math 与否对
RGB 8 位量化几乎无影响；本片 RGB 差异全部来自实现（见 §6），与配方无关。

### 5.3 depth/seg 回归（S7b1 基线，硬门）

- 两相机：`depth_mismatch=0`、`max_ulp=0`、`seg_mismatch=0`（120,000 / 120,000 位等，背景 `(-1,-1)`）。

### 5.4 其余

- 两次运行：RGB / depth / seg 全部逐位相同（`depth_seg_hash=03efa95416bbd008`、
  `rgb_hash=ed50f750936c275`）。
- 写前拒绝面 4/4：`use_shadows`、`use_precomputed_rays=false`、`samples_per_pixel=2`、
  `render_skybox` 均在发射前返回 `LauncherError::Unsupported`。
- 灯位姿：`light_data_check` 断言 3 灯 `xpos`/`xdir` f32 位型与 pinned 3.15 `mjwarp.forward` 逐位相等。
- A′ 门禁：`check_render_ptx(ptx, "render_megakernel")` 通过（f32-render 配方，PTX 恰好 2 条 f64 常量，
  与 S7b1 同口径；编译只走 `compile_render_artifact`）。

## 6. 根因与修复（本片关键）

**首轮结果**：RGB 对 S0b fast-math 参考 cam1 有 **332 / 43,200 像素 >1 LSB（max 3 byte）**，
全部集中在 geom2（球面 174）+ geom5（圆柱 158）；cam0 0 超限。

**排查链（按证据推进，不猜）**：

1. **排除「跨配方噪声」**：采同配方（`use_fast_math=False`）RGB 参考
   （`rgb_fast_math_false.json`，`kernel_cache` 证明 `div.rn=206`/`div.approx=0`，即快算确实关闭）。
   结果：同配方参考与 S0b 参考几乎位等（cam0 2 像素 1 byte、cam1 全位等）→ 参考侧配方不是变量；
   产品 332 像素 >1 LSB 是实现差异。
2. **排除纹理路径**：临时消融 `use_textures=False`（同配方参考 + 产品，均不入库）：cam1 仍有
   **334 像素 >1 LSB**（geom2 172 + geom5 162）→ 与纹理 UV/采样无关。
3. **对照上游生成的 megakernel CUDA 源**
   （`target/render-bvh/s7b2-fastmath-cache/1.15.0/wp__build_megakernel__locals___render_megakernel_92d94e46_dde26ad/*.cu`）：
   headlight 的 `compute_lighting` 调用传的是**材质** `mat_spec`/`mat_shin_exp`（`var_352`/`var_353`，
   由 `mat_specular[mat_id]`/`mat_shininess[mat_id] * MAX_SHININESS` 得到），
   而本实现误传固定常量 `MK_HEADLIGHT_SHININESS = 64.0`。
4. **数值分布吻合**：高光指数 `mat_shininess*128` 为 mat0=76.8、mat1=12.8、mat2=25.6；错误常量 64 对
   mat1（球面/圆柱；shin=12.8）偏差最大（2-3 byte），mat2（平面；25.6）偏差小（1 byte，cam0 的
   26,649 像素），mat0（box/mesh；76.8）偏差最小（无 >1 LSB）—— 与实测分布一致。

**修复**：`render_megakernel.cu` 的 headlight 调用改传 `mat_shin_exp`（删 `MK_HEADLIGHT_SHININESS`）。

**修复前后**：

| 指标 | 修复前 | 修复后 |
| --- | --- | --- |
| cam0 RGB（vs 同配方） | 26,649 像素差 1 byte | **0 像素差异** |
| cam1 RGB（vs 同配方） | 954 差异 / 332 超 1 LSB / max 3 | **0 像素差异** |
| depth/seg 回归 | 0 ULP | 0 ULP（不变） |

**非调参证据（避免「对着夹具调参」）**：排查过程中另发现两处与上游的疑似语义偏离，一并做了实测：
① `wp.texture_sample` 的 `lod` 实参（上游 `-1.0f` 走 `tex2D`，本实现原 `0.0f` 走 `tex2DLod`）；
② 球面 normal 的 `wp.normalize` 除法顺序（`scene.cu` 的 `normalize3` 用 `a*(1/l)`，上游 `a/l`）。
两处分别修改后对超限像素**无任何影响**；把这两处**同时回退**后 RGB 仍为 **0 像素差异** ——
证明唯一根因是 §6 的高光指数参数，且修复不依赖任何夹具特化。两处最终按「最小 diff」原则保留原状
（记录为已排查项；当前场景/硬件上不产生位级差异）。

## 7. 夹具与 provenance

### 7.1 `fixtures/render-bvh/scene-a/s7b/rgb.json`（S0b 派生，跨配方记录用）

- 1,322,081 B，sha256 `57f85a049c25be3c98dccc37c304fb6db6232215266fdfe1b616a2202a7126d2`
- 来源：S0b `reference/rgb.npz`（sha256 `9379f5d3…`）经 `get_rgb` 解包 f32 → 字节 →
  `pack_rgba_to_uint32` 重新打包；含 3 灯 `light_data_check` 位型。
- 派生脚本 `scripts/authoring/render-bvh-s7b/convert_rgb.py`（11,299 B，sha256 `d9321038…`），
  pinned venv（Python 3.12.13 / warp 1.15.0 / mujoco & mujoco_warp 3.15.0 / numpy 2.5.4），
  两次 forward 逐位一致，packed↔npz 位型 round-trip 自检。

### 7.2 `fixtures/render-bvh/scene-a/s7b/rgb_fast_math_false.json`（同配方参考，硬指标依据）

- 1,323,726 B，sha256 `d0de2b6069241518513a8b6ec1f65dfb76ce43d4c86b663374e8cf32c6e85b4a`
- 来源：pinned 3.15 + `PARAMS_28` 仅 `use_fast_math=False`，一次 `create_render_context` +
  两次 `mjwarp.render`（逐位一致），直接捕获 `rc.rgb_data` packed u32（ABGR，a=255）两相机全图；
  `self_check` 含 `unpack(rgb_data)` 与上游 `get_rgb` 的位等断言、以及与本片 depth/seg 同配方参考
  （`depth_seg_fast_math_false.json`）的位等自证（`depth_seg_recipe_equal=true`）。
- 采集脚本 `scripts/authoring/render-bvh-s7b/capture_rgb_fast_math_false.py`（19,641 B，
  sha256 `c12097fb…`）；`kernel_cache` 记录生成 PTX 195 KB 级、`div_rn=206`、`div_approx=0`、
  `sqrt_rn=77`，证明 `use_fast_math=False` 生效。
- 编码：packed u32 ABGR（`a=255`，`r/g/b = int(clamp(c,0,1)*255)` 截断）；解包
  `float32(byte) * float32(1/255)`（上游 `unpack_rgb_kernel`）；判据「解包后每通道 ≤1 LSB」⇔ packed
  byte 差 ≤1。测试只读位型，不引入 zip/inflate 依赖，不启动 Python。

## 8. 环境

| 项 | 值 |
| --- | --- |
| GPU / 驱动 / SM | RTX 4070 Ti SUPER / 596.36 / SM89 |
| 产品编译 | NVRTC（CUDA 12.8.1 工具链，compute_70）、`WARP_1_15_RENDER` 冻结配方（fast_math=false） |
| 参考环境 | pinned venv：Python 3.12.13、warp 1.15.0（自带 NVRTC CUDA 12.9）、mujoco 3.15.0、mujoco_warp 3.15.0、numpy 2.5.4 |
| GPU 窗口 | 最终采集：起点 3335 MiB → 结束 3335 MiB，无 CUDA 计算进程残留（桌面基线 3300±40 MiB 波动） |
| 参考 megakernel 源 | `target/render-bvh/s7b2-fastmath-cache/1.15.0/wp__build_megakernel__locals___render_megakernel_92d94e46_dde26ad/`（本地缓存，不入库） |

## 9. 未满足项与边界（不宣称）

- 阴影（`use_shadows=true`）、天空盒（`render_skybox=true`）、splat、AA（`samples_per_pixel>1`）未实现，
  全部在写前 `Unsupported` 拒绝（归 S7b3+）。
- flex / hfield / SDF / 动态射线（`use_precomputed_rays=false`）未实现（写前拒绝或占位）。
- 正交相机未覆盖（scene-a 两相机均为透视）。
- 大网格/带 texcoord 的网格未覆盖：scene-a 的网格无 texcoord，走 OBJECT_PLANE fallback；
  `texture_mesh` 的 texcoord 插值分支已实现但本片未验证。
- 相机 data 位姿由调用方显式传入（跟踪相机 TARGETBODY 数学的产品接线归 **S8 端到端**，
  `2026-10-25-render-bvh-s8-e2e`）；「夹具位姿 → 全图一致」不等于「产品跟踪相机已接通」。
- 只支持 `light_mode=0` 固定灯的 data 位姿推导（其他 mode 明确报错）。
- `bvh_id` 为自有打包，**不宣称**与 Warp BVH 的 ABI/内存布局等价。

## 10. 证据文件

- `gpu-run.log`：S7b2 GPU 用例（RGB / depth-seg 回归 / 拒绝面 / PTX 摘要）
- `s7b1-regression.log`：S7b1 GPU 用例 6/6
- `s6b-regression.log`：S6b `render_context_lifecycle` 5/5
- `unit-summary.log`：`cargo test --locked` 532 passed / 0 failed
- `capture.log`：同配方 RGB 采集脚本输出（含 sha、div_rn/div_approx 证据）
- `receipt.json`：机器可判定回执
