# 渲染/BVH 子系统可行性预检与切片计划

任务：`2026-10-24-r01-p8-render-bvh-precheck`（p5，P8-渲染，owner=Arch）。
日期：2026-10-24。基线：仓库 `294c944`（HEAD，工作区干净），Warp `ffa038b1adf3927a8e893d4536f1a7562f17d749`（`v1.15.0`），MJWarp `71da24d956378a87a703b6e1442b13aec0c4ac29`。
性质：**只读预检 + 切片计划**。未改 `src/`、`tests/`、`fixtures/`、`Cargo.toml`、`Cargo.lock`，未写产品代码，未新增产品依赖，未启动产品测试。本预检不改变任何验收状态：渲染/BVH 仍未实现，相关验收项均未通过。

依据文件：本目录 `plan.json`、`upstream-references.tsv`、`smoke-render.py`、`smoke-render.json`；仓库文档 `docs/module-contracts.md` §8、`docs/compatibility-matrix.md` 渲染矩阵、`docs/development-evidence/w15-verbatim-headers/headers.json`。

## 结论

1. **原假设需要修正**：Warp `v1.15.0` **没有** `warp/_src/render.py`，也没有 CUDA 离屏渲染内核或相机/采样参数。Warp 的 `warp/_src/render/` 是 OpenGL/USD 视口渲染（`render_opengl.py`、`render_usd.py`），依赖窗口/GL 上下文，与 AGENTS「不要加入窗口或查看器」冲突。真正的渲染参考在 **MJWarp**：`mujoco_warp/_src/render.py`、`render_util.py`、`bvh.py`（Warp `@wp.func/@wp.kernel`），以及 `ray.py` 的 BVH 加速射线查询。Warp 侧可复用的是 `warp/native/bvh.h`、`mesh.h`、`texture.h` 等低层设备代码与 W15 已复制头文件。
2. **冻结参考可生成**：`target/mjwarp-parallel/venv` 内有 Python 3.12.13、Warp 1.15.0、MJWarp 3.15.0、MuJoCo 3.15.0；2026-10-24 在 RTX 4070 Ti SUPER（SM89，驱动 596.36）实测 `create_render_context` + `render` + RGB/深度/分割输出成功，连续两次输出摘要一致（见 §5.1）。该环境只用于离线 authoring，不进入产品构建/测试。
3. **产品缺口明确**：`src/render/mod.rs` 只有 `ImageBatchLayout` 宿主布局（文件头注释即写明「不生成像素或创建BVH」，L1–2）；产品模型没有 `mat_texid`、`mesh_*`、`tex_*`、`cam_intrinsic/sensorsize/projection/resolution/output`、`geom_matid/rgba` 等渲染输入；没有 BVH 构建/重拟合、资产上传、渲染内核、render context 生命周期。现有 launcher 的 L2 发射只接受数组实参（`src/runtime/launcher/warp.rs` L140–143、L219–237），渲染内核需要标量、`vec3`、`u64`、纹理数组等实参，必须新增专用发射 ABI 或扩展现有 ABI。
4. **MuJoCo 版本锁是硬约束**：产品 native probe 要求 `mj_version()==3012000`（`native/model_probe.cpp` L171–178），参考环境是 MuJoCo 3.15.0。实测 3.15 保存的 MJB 被 3.12 拒绝，3.12 保存的 MJB 也被 3.15 拒绝（均报 `different number of sizes`）。因此冻结参考必须采用**同一 XML、双版本 MJB、逐字段模型数组 parity** 的方式；parity 不成立时不得用参考图做验收。
5. **建议切片**：先落参考夹具与 parity（S0a/S0b），再做模型输入（S1）、场景 BVH（S2）、网格 BVH（S3）、纹理（S4）、相机/射线（S5）、context 参数与生命周期（S6a/S6b）、渲染发射 ABI 与内核（S7a/S7b），最后端到端（S8）。r01-p8 的「全部默认参数与生命周期」映射见 §7；阴影/天空盒/泼溅/柔体/多相机不在本次 MVP，归 R05–R10。总量约 60 人日（§6），每片新文件 ≤500 行。

## 1. 核对基线与方法

| 项 | 值 | 核对方式 |
| --- | --- | --- |
| 仓库 | `294c944`（登记提交） | `git -C D:\Rust\mjwarp-rs rev-parse HEAD`；`git status --short` 为空 |
| Warp | `ffa038b1adf3927a8e893d4536f1a7562f17d749`，tag `v1.15.0` | `git -C target/research/warp rev-parse HEAD`、`describe --tags` |
| MJWarp | `71da24d956378a87a703b6e1442b13aec0c4ac29` | `git -C target/mjwarp-parallel/upstream rev-parse HEAD` |
| W15 头文件 | 36 个头文件，`1,520,709` 字节 | `docs/development-evidence/w15-verbatim-headers/headers.json` L16–198 |
| 参考环境 | Python 3.12.13、Warp 1.15.0、MJWarp 3.15.0、MuJoCo 3.15.0、numpy 2.5.4、GPU RTX 4070 Ti SUPER / SM89 / 驱动 596.36 | `target/mjwarp-parallel/provenance.json`；`smoke-render.json` |
| 产品 MuJoCo | 3.12.0，头文件 `mjVERSION_HEADER==3012000` | `native/model_probe.cpp` L15、L171–178；`target/toolchains/mujoco-3.12.0/package` |

本预检不凭记忆写 API：所有上游引用给出文件与行号；MJWarp 的逐行引用清单由脚本生成到 `upstream-references.tsv`（匹配 `render|bvh|RenderContext|ray_mesh_with_bvh`，大小写不敏感；`scope=source|test`；共 964 行，source 464、test 500）。正文只列生产引用与关键定义。

## 2. 冻结上游 API 面

### 2.1 Warp v1.15.0：没有离线 render，只有低层 BVH/mesh/texture

**render 模块实际内容**（`target/research/warp/warp/_src/render/`）：

| 文件 | 字节 | 性质 |
| --- | --- | --- |
| `render_opengl.py` | 148,479 | OpenGL 视口渲染；L1–45 即 GLSL 顶点着色器与 GL 状态，不是离屏 CUDA 内核 |
| `render_usd.py` | 39,108 | USD 视口 |
| `imgui_manager.py` / `utils.py` / `__init__.py` | 10,889 / 4,189 / 139 | 视口辅助 |

在 `target/research/warp` 全树检索 `*render*`，没有 `warp/_src/render.py`；这与任务原文的假设不同。因此「Warp v1.15.0 的 render 模块（`warp/_src/render.py`、渲染内核、相机/采样参数）」在本基线上**不可得**；可替代的冻结参考是 MJWarp（§2.2）。OpenGL 视口不作为替代：它需要窗口/GL 上下文，不能作为无头批量渲染参考。

**Warp 低层 BVH**（`target/research/warp/warp/native/`）：

| 资产 | 位置 | 可复用面 |
| --- | --- | --- |
| `bvh.h` | L175–206 `struct BVH`；L313–356 节点打包/LCA；L442–563 `bvh_query(_aabb/_ray)/bvh_query_next`；L570–581 主机 create/destroy/refit/rebuild 声明；L585–607 设备 create/destroy/refit 声明；L613 引入 `tile_bvh.h` | 设备端遍历与查询、BVH 结构/ABI 可直接用于 W15 编译 |
| `bvh.cpp` | L62–733 `TopDownBVHBuilder`（initialize/build_with_groups/build/partition_median/partition_midpoint/build_recursive/refit）；L573 `reorder_top_down_bvh`；L790 `bvh_create_host`；L819 `bvh_destroy_host`；L840 `wp_bvh_create_host`；L854/860/866 refit/rebuild/destroy | **主机端 SAH 构建/重拟合实现；W15 未复制** |
| `bvh.cu` | L42 `bvh_refit_kernel`；L184/218/228/261/402/449/482 设备构建 kernel；L515 `LinearBVHBuilderGPU::build`；L625 `copy_host_tree_to_device`；L655 `bvh_create_device`；L772 `bvh_refit_device` | **设备端构建/重拟合实现；W15 未复制** |
| `bvh_cubql.cpp` / `.cu` | 主机/设备 cuBQL 集成；`bvh_cubql.cpp` L1–5 声明 Apache-2.0；`licenses/cubql-LICENSE.txt` 为 Apache-2.0 | 上游网格 BVH 使用 `constructor="cubql"`；W15 未复制 cuBQL 子目录 |
| `mesh.h` | L1764 `mesh_query_ray`；L1972 `mesh_query_ray_count_intersections`；L2436 `mesh_query_aabb`；L2762–2765 descriptor 声明 | 设备端网格射线/最近点查询 |
| `mesh.cpp` | L118 `wp_mesh_create_host`；L194 destroy；L210 `wp_mesh_refit_host`；L241 `wp_mesh_set_points_host`；L257 set velocities | 主机端网格 BVH 创建/重拟合；W15 未复制 |
| `mesh.cu` | 设备端网格查询/重拟合实现 | W15 未复制 |
| `texture.h` | `texture2d_t` L182；设备采样 L819（`tex2D<float4>`）等 | 设备端纹理采样结构/函数 |
| `texture.cpp` | L21 `wp_texture_create_host`；L149 `wp_texture_create_device`；L363 `wp_texture_object_create_device`；L431 destroy | 主机端纹理创建/上传/CUDA texture object；W15 未复制 |

**Warp Python 生命周期**（参考，不是产品链）：

- `warp/_src/types.py` L5545 `class Bvh`；L5696/5705 主机/设备创建；L5725/5729 销毁；L5734 `refit`；L5746 `rebuild`。
- `warp/_src/types.py` L5822 `class Mesh`；L5934/5946 创建；L5968/5972 销毁；L5977 `refit`；L6017 `set_points`。
- `warp/_src/texture.py` L1192 `class Texture2D`；L1228 构造参数（filter/address/normalized coords/mip）；L1266 `__ctype__` 把句柄/尺寸/通道传给内核。

### 2.2 MJWarp 71da24d：渲染/BVH 的真实参考

**生产引用**（逐行清单见 `upstream-references.tsv`；下表为关键入口）：

| 文件 | 行 | 用途 |
| --- | --- | --- |
| `mujoco_warp/__init__.py` | 32–33 | 重导出 `refit_bvh`、`refit_splat_bvh` |
| `mujoco_warp/__init__.py` | 69–73 | 重导出 `render`、`create_render_context`、`get_rgb/depth/segmentation` |
| `mujoco_warp/__init__.py` | 123 | 重导出 `RenderContext` |
| `_src/types.py` | 123–124 | `Model.block_dim.render = 64` |
| `_src/types.py` | 1200–1345 | 渲染相关 Model 字段（`geom_matid`、`site_*`、`mat_texid/texrepeat` 等） |
| `_src/types.py` | 2629–2821 | `RenderContext` 定义：BVH/网格/纹理/相机/输出偏移/AA/泼溅字段 |
| `_src/render.py` | 16–43 | 依赖 `bvh`、`ray`、`render_util`、`types` |
| `_src/render.py` | 55–568 | 泼溅、纹理采样、天空盒、顶点法线、材质颜色等 `@wp.func` |
| `_src/render.py` | 585–973 | `_make_cast_ray` / `_make_compute_lighting`（按 geom 类型静态特化） |
| `_src/render.py` | 976–988 | `_aa_resolve` |
| `_src/render.py` | 991–1487 | `_build_megakernel`（按 context/模型静态特化 megakernel，含全部输入/输出参数） |
| `_src/render.py` | 1490–1604 | `render(m,d,rc)` 发射与 AA 解析 |
| `_src/render_util.py` | 29–69 | `_convert_texture_data`、`create_warp_texture`（u8→float4，LINEAR） |
| `_src/render_util.py` | 72–160 | `compute_ray`（透视/正交、内参/传感器、znear） |
| `_src/render_util.py` | 163–281 | `pack_rgba_to_uint32`、`unpack_rgb_kernel`、`extract_depth_kernel`、`get_rgb/depth/segmentation` |
| `_src/render_util.py` | 285–309 | `_build_rays`（AA 子采样） |
| `_src/render_util.py` | 312–342 | `create_render_context` 签名（`mjm` 后 28 个参数） |
| `_src/render_util.py` | 403–796 | context 构造：splat/mesh/hfield/flex BVH、纹理注册、相机筛选、输出偏移、znear/zfar、ray/AA 分配、`build_scene_bvh` |
| `_src/bvh.py` | 38–43 | `refit_bvh`：场景 + 柔体 |
| `_src/bvh.py` | 177–311 | 六类几何的 AABB 计算 kernel |
| `_src/bvh.py` | 314–379 | `build_scene_bvh`：GPU 计算 AABB → `wp.Bvh(..., constructor="sah")` → group roots |
| `_src/bvh.py` | 382–424 | `refit_scene_bvh`：重算 AABB → `rc.bvh.refit()` |
| `_src/bvh.py` | 427–450 | `build_mesh_bvh`：`wp.Mesh(points, indices, bvh_constructor, bvh_leaf_size)`；`constructor="cubql"` 由 context 传入 |
| `_src/bvh.py` | 994–1180 / 1221–1273 | flex/splat BVH 构建/重拟合（R06/R07） |
| `_src/ray.py` | 695–757 | `ray_mesh_with_bvh(_anyhit)`：`wp.mesh_query_ray` |
| `_src/ray.py` | 1022–1092 | `_ray_geom_mesh_bvh`：网格/高度场 BVH 分支 |
| `_src/ray.py` | 1094–1177 | `_ray_bvh`：`wp.bvh_query_ray` + `bvh_query_next` |
| `_src/ray.py` | 1179–1225 / 1226–1310 | 公开 `ray` / `rays`；`rc` 可选时走 BVH 路径 |
| `_src/cli.py` | 55–75 | 渲染 CLI 参数（宽高/RGB/深度/分割/相机/纹理/阴影/背面剔除/天空盒） |
| `_src/cli.py` | 150–234 | `init_structs` 创建 `RenderContext` 的 CLI 接线 |
| `record.py` | 44–45、74–111 | `render_mode` / 通道；`refit_bvh` + `render` 循环 |
| `testspeed.py` | 186–190、282–297 | context 统计与 `refit_bvh`+`render` 基准 |
| `_src/set_const.py` | 867 | 注释：静态 body 位姿变化会使 BVH 失效 |

**测试引用**：`_src/render_test.py` 229 行、`_src/ray_test.py` 160 行、`_src/render_util_test.py` 51 行、`_src/bvh_test.py` 49 行；这些是行为/数值口径的重要参考（例如 `render_test.py` L67–77 最小 RGB 用例，L93–105 泼溅遮挡，L111–140 多 world；`render_util_test.py` L42–120 相机/纹理/分割）。测试不计入产品测试，但为切片验收提供上游行为样本。

**相机/采样参数**：`create_render_context` 的 28 个参数（`mjm` 之后）与默认值见 `render_util.py` L312–342；`module-contracts.md` L653–672 已逐项列出默认值。相机选择支持 `bool` / 名称 / 索引 / 空列表（`render_util.py` L537–559）；名称走 `mujoco.mj_name2id`（L548），这是产品必须补的名称表能力之一。

### 2.3 W15 已复制头文件：可直接复用 vs 缺失

`include/warp-headers/` 36 项、`1,520,709` 字节，来源唯一性已由 W15 证据锁定（`headers.json` L1–16；`w15-verbatim-headers/README.md` L9–17）。逐项比较（`headers.json` L17–198）：

| 类别 | 头文件 | 结论 |
| --- | --- | --- |
| 设备端基础 | `builtin.h`、`deterministic.h`、`vec.h`、`mat.h`、`quat.h`、`array.h`、`range.h`、`tuple.h`、`initializer_array.h`、`matnn.h` | 可直接复用；`builtin.h` 是闭包根，包含 `tile.h` 等 |
| BVH 设备端 | `bvh.h`、`tile_bvh.h`、`intersect.h`、`intersect_tri.h` | 结构体/遍历/查询可直接复用；**主机/设备构建实现不在头文件** |
| 网格设备端 | `mesh.h`、`tile_mesh.h` | 查询/求交可复用；`wp_mesh_create_host/refit` 实现缺失 |
| 纹理设备端 | `texture.h` | `texture2d_t`/采样可复用；主机创建/上传/CUDA texture object 缺失 |
| 杂项 | `solid_angle.h`、`hashgrid.h`、`volume.h`、`svd.h`、`noise.h`、`rand.h`、`spatial.h`、`fabric.h`、`crt.h`、`cuda_crt.h`、`tile_*.h`、`nanovdb/PNanoVDB.h` | 按需可用；本 MVP 只需其中一部分 |
| **未复制** | `warp.h`、`aot.h`、`exports.h`、`*.cpp`、`cuBQL/` | `headers.json` L214–219 明确排除；`bvh_cubql.cpp/cu`、`bvh.cpp/cu`、`mesh.cpp/cu`、`texture.cpp` 均不在 W15 |
| **运行时缺失** | — | `wp.Bvh`/`wp.Mesh`/`wp.Texture2D` 的宿主生命周期、CUDA 资源、描述符表、注册表、上下文全部没有 Rust 对应物 |

W15 的 NVRTC 编译路径已支持内存头文件（`src/runtime/launcher/artifact.rs` L77–99；`artifact/nvrtc.rs` L40–110），并对头文件树追加 `--device-as-default-execution-space`（`w15-verbatim-headers/README.md` L27–32）。这意味着**冻结的 Warp 生成 CUDA 源（只 include `builtin.h`/`deterministic.h`）可以用现有 NVRTC 路径 + W15 头树编译**；但生成源是特化代码，不是通用产品内核，见 §8 决策 2。

## 3. 产品现状与对接点

### 3.1 已有

- `src/render/mod.rs`：仅 `OutputChannels`、`ImageOutput`、`ImageBatchLayout`（L28–126）；文件头注释「不生成像素或创建BVH」（L1–2）。
- `src/lib.rs` L10 导出 `render`。
- `src/model`：已有 `geom_type/geom_size`、刚体/相机/光源位姿等物理字段；没有渲染材质/纹理/网格/相机内参/输出位姿。
- `native/model_probe.cpp`：通过 `mj_loadModelBuffer` 读取 MJB；版本严格等于 3.12.0（L171–178）；当前 G01 字段 45 项（`src/io/g01.rs` L51–97），无渲染字段。
- `build.rs`：`native-model-probe` feature 需要 `MJWARP_MUJOCO_ROOT` 指向 MuJoCo 3.12 包（L21–24），编译 `native/model_probe*.cpp`（L45–49）。
- launcher：`NvrtcSource` 内存头文件 → 临时目录 → NVRTC（`artifact/nvrtc.rs` L40–110）；`KernelSourceRoute` 四类路线（`route.rs` L20–28）；`RouteKey` 纳入源/头/选项/arch/版本（`route.rs` L106+）；LTOIR 接口（`artifact/ltoir.rs` L27–146）；MathDx LTO（`launcher/mathdx/*`）；W15 头树可用。
- 自定义发射：`transfer/kernel/launchers_physics.rs` L64–71、L135–137 等展示「数组 + 标量」逐内核打包模式；`module.rs` L24 `KernelHandle`。

### 3.2 缺口

| 缺口 | 现状 | 影响 |
| --- | --- | --- |
| 渲染模型输入 | 无 `mat_texid/rgba/specular/shininess/emission`、`mesh_*`、`tex_*`、`cam_projection/sensorsize/intrinsic/resolution/output`、`light_*` 颜色/衰减、`vis.headlight`、`stat.extent`、名称表 | S1 必须先补 |
| 资产上传 | 无 mesh BVH、texture object、texture registry、mesh registry | S3/S4 |
| BVH | 无场景/网格 BVH 构建、重拟合、查询、group root | S2/S3 |
| 渲染内核 | 无 ray cast/纹理/光照/像素打包/AA/深度/分割内核 | S7b |
| context | 无 `RenderContext`、无输出偏移/相机映射/资源寿命 | S6a/S6b |
| 发射 ABI | L2 只接受数组实参（`warp.rs` L140–143、L219–237）；无标量/`vec3`/`u64`/纹理数组的通用打包 | S7a |
| MJWarp 名称语义 | `cam_active` 支持名称；产品无 `mj_id2name` 名称表 | S1/S6a |
| Python 边界 | 参考生成需要 Python；产品禁止 Python | 仅 S0b authoring |

**对接点**：

- W13 nvJitLink：本 MVP 不需要 LTO 链接；渲染内核走 NVRTC → PTX/CUBIN（`artifact/nvrtc.rs`）即可。若后续把生成源作为 LTOIR，可复用 `artifact/ltoir.rs`。
- W15 头文件：`NvrtcSource::with_headers_from_dir("include/warp-headers")` + `--device-as-default-execution-space`（W15 已实现）；生成的 render `.cu` 只 include `builtin.h`/`deterministic.h`，与 W15 闭包一致。
- 内核来源路线：`HandWrittenCuda` 适合手写移植；`UpstreamSource` 名义上是上游源码树，但 MJWarp 的渲染内核是 Python `@wp.kernel`，不是可直接编译的上游 `.cu`；冻结生成源需要新增路线/证据类别，不能冒充 `UpstreamSource`。
- MathDx LTO：渲染不需要；不接入。
- 现有 `WarpArray<T>` 支持 1–4 维、任意 `element_bytes`（`abi/array.rs` L53–126），可承载 `vec3`（12 B）/`vec4`（16 B）数组；但标量与纹理句柄仍需 S7a。

## 4. 证据策略（可机器判定）

### 4.1 原则

1. **语义证据**：只比较上游公开行为的结果（射线命中、图像像素、深度、分割、context 解析值、资源寿命），不比较结构体偏移、字段存在性或内部树布局。
2. **冻结参考**：离线 authoring 生成，脚本进仓，带 provenance 与哈希；产品构建/测试只读冻结文件，不启动 Python。
3. **双版本 parity**：参考（MuJoCo 3.15）与产品（3.12）使用同一 XML、各自 MJB；先证明渲染相关模型数组逐字段一致，再比较渲染输出。parity 失败即停止，不调容差掩盖。
4. **失败可定位**：每个断言输出 `field/index/expected/actual/diff/ulp` 与首个差异；汇总 JSON + 退出码。
5. **不造假能力**：没有参考/无法判定时写 `unverified` 与原因，不用布局探针或空跑探针充数。

### 4.2 证据类别

| 类别 | 用途 | 判定 |
| --- | --- | --- |
| `exact` | 整数 ID、标志、计数、分割 `(id,type)`、context 整数/枚举、模型数组 | 逐元素相等 |
| `f32_bits` | 模型 f32 数组、context 派生 f32、射线方向/偏移 | 逐元素 bit 相等；若上游/产品编译器差异导致不可能，降级为 ULP 并记录原因 |
| `f32_ulp` | 射线距离/法线、深度、纹理采样值 | 每项 ≤ 预算 ULP（下表） |
| `image_lsb` | RGB | 每通道 `|a-b| ≤ 1`（u8），逐像素；记录最差与超限像素数 |
| `hash_repeat` | 同输入两次运行 | 输出 SHA-256 相同 |
| `lifecycle` | create/destroy/refit/资源释放 | 显存前后差、无 UAF、无隐式刷新（行为断言） |
| `reject` | 非法组合 | 错误类别/字段与上游对应；不比拼错误文案 |
| `receipt` | 每次验收 | 版本、GPU/驱动、命令、输入/输出哈希、容差、未验证项 |

### 4.3 建议容差（proposed，需裁决）

| 对象 | 建议 | 理由 |
| --- | --- | --- |
| 模型数组/context 整数/`seg` | `exact` | 确定性数据 |
| 模型 f32/context 派生 f32 | `f32_bits` 优先；不可达时 ≤1 ULP | 同源同精度；版本差异用 parity 先排除 |
| 射线距离 / 深度 | ≤4 ULP 或 `1e-5` 相对（取更严） | 三角求交/投影的 FMA 顺序可能不同 |
| 射线法线 | 角度 ≤`1e-4` rad（或 ≤8 ULP 分量） | 归一化/开方差异 |
| RGB | 每通道 ≤1 LSB | 8-bit 量化；超过即失败 |
| 纹理采样（float4） | 每通道 ≤`1e-6` 绝对 | 线性过滤；同 GPU 期望更小 |
| 同输入重复 | SHA-256 相等 | 排除非确定性 |

这些阈值是**建议值**；未获裁决前，切片验收报告必须同时给出实际最大误差与超限计数，不得只写「通过」。

### 4.4 参考生成方式与边界

- 脚本：`scripts/authoring/render-bvh/`（S0a/S0b 落仓；本轮预检只把 smoke 脚本放在证据目录，见 §5.1）。
- 参考环境：`target/mjwarp-parallel/venv`（Python 3.12.13、Warp 1.15.0、MJWarp 3.15.0、MuJoCo 3.15.0、numpy 2.5.4）；GPU RTX 4070 Ti SUPER / SM89 / 驱动 596.36。
- provenance：脚本 SHA-256、XML SHA-256、双 MJB SHA-256、Warp/MJWarp/MuJoCo/Python/GPU/驱动/CUDA arch、运行命令、重复次数；写入 `provenance.json`。
- 可复现：同一环境连续两次输出哈希一致；换 GPU/驱动/数学库必须重新生成并记录，不得混用。
- 产品边界：`cargo build`/`cargo test` 不调用 Python；测试只读 `fixtures/render-bvh/` 下的 XML/MJB/参考 NPZ/JSON。
- 双 MJB：3.12 MJB 供产品 native probe；3.15 MJB 供参考渲染；同一 XML 编译，`parity.json` 证明渲染相关数组一致。若 parity 失败，参考不能用于验收；可选出路见 §8 决策 1。

### 4.5 无法逐位验证的项与原因

1. **BVH 内部布局/遍历顺序**：不是上游公开语义；SAH/LBVH/CUBQL 产生不同树。用射线命中/距离/法线验证，不比较节点数组。
2. **浮点超越函数**：`tan/pow/sqrt/normalize` 在 NVRTC 与 Warp 编译链、`fast_math` 下可能差 ULP；用 ULP 预算并记录。
3. **硬件纹理过滤**：CUDA `tex2D` 的线性插值/边界处理可能因 GPU/驱动不同；同 GPU 参考可做到很小误差，跨 GPU 不能承诺逐位。
4. **AA 累加与多 world 调度**：像素内采样顺序固定时可验证；并行调度不影响单像素值，但若实现改变累加树，需要重新定容差。
5. **泼溅/柔体/高度场**：混合顺序与网格化依赖实现；本 MVP 不验证，归 R06/R07/R05。
6. **跨编译器 FMA 收缩**：同一 f32 表达式可能因 `--fmad`/优化不同而差 ULP；配方必须冻结，验收记录编译选项。
7. **MuJoCo 3.12 vs 3.15 模型差异**：只能靠 S0a parity 证明；证明不了就不能宣称参考等价。

## 5. 实测记录（预检）

### 5.1 参考渲染 smoke（可复现）

命令（作者环境，非产品测试）：

```text
target/mjwarp-parallel/venv/Scripts/python.exe \
  docs/development-evidence/render-bvh-precheck/smoke-render.py \
  docs/development-evidence/render-bvh-precheck/smoke-render.json
```

夹具：1 相机（32×32，rgb+depth）、1 光源、3 个几何（plane/box/sphere）、1 张 32×32 checker 纹理、无网格。默认 `use_textures=true`、`use_fast_math=true`、`samples_per_pixel=1`。实测（`smoke-render.json`）：

| 输出 | 形状/类型 | 摘要 |
| --- | --- | --- |
| RGB | `(1,1024)` `uint32` | min `4278190080`，max `4294581790`，sha256 `d3a98fa8…` |
| 深度 | `(1,1024)` `f32` | min `0.0`，max `5.85213565826416`，sha256 `8bd92717…` |
| 分割 | `(1,1024,2)` `i32` | min `-1`，max `5`，sha256 `68923a3c…` |

三次运行（两次重编译、一次缓存命中）三个摘要一致。该结果只证明「冻结参考可生成」，不证明产品渲染可用。

### 5.2 MuJoCo MJB 版本锁（实测）

- 3.15 Python 保存 3,914 B MJB → 3.12 DLL `mj_loadModel` 返回 `null`，警告 `Model and executable have different number of sizes in mjModel`。
- 3.12 `sample/compile.cc` 风格工具保存 3,924 B MJB → 3.15 Python `MjModel.from_binary_path` 抛 `ValueError: mj_loadModel: failed to load from mjb`，同一警告。
- 结论：不能用一个 MJB 同时喂 3.12 产品与 3.15 参考；必须双 MJB + parity（§4.4）。本轮工具在 `target/` 临时目录，未进仓；S0a 会把 3.12 编译/dump 工具正式落仓。

### 5.3 Warp 生成 CUDA 的可用性（观察）

smoke 运行在 `target/render-bvh-precheck/warp-cache`（临时）生成：

- `wp_mujoco_warp._src.render_util_*.cu`（36,059 B）
- `wp_mujoco_warp._src.bvh_*.cu`（187,376 B）
- `wp__build_megakernel__locals___render_megakernel_*.cu`（272,838 B）

生成源只 `#include "builtin.h"` 与 `"deterministic.h"`（文件头 L1–5），与 W15 闭包一致；这说明「冻结生成源 + W15 + NVRTC」在头文件层面可行。**尚未实测 NVRTC 编译/链接/GPU 执行**，列为 S7a 的前置验证；不得把「头文件层面可行」写成已通过。

## 6. 切片计划

完整字段见 `plan.json`。摘要（工作量为人日估算，含测试与证据；新文件按 ≤500 行拆分）：

| ID | 切片 | 类别 | 依赖 | GPU | 人日 | 新文件上限 |
| --- | --- | --- | --- | --- | --- | --- |
| S0a | 参考夹具 + 双 MJB + 模型 parity | 参考 | — | 否 | 2 | 500 |
| S0b | 冻结参考捕获（图像/射线/默认值） | 参考 | S0a | 是 | 2 | 500 |
| S1 | 渲染模型输入（相机/光源/材质/纹理/网格/名称） | 资产与拓扑 | S0a | 否 | 5 | 500 |
| S2 | 场景 BVH 构建/重拟合/查询（已拆 S2a/S2b/S2-close） | BVH | S1 | 是 | 8 | 500 |
| S3 | 网格 BVH 与射线 uv/face | BVH | S2 | 是 | 8 | 500 |
| S4 | 纹理转换/上传/采样 | 渲染内核 | S1 | 是 | 5 | 500 |
| S5 | 相机/射线/AA/预计算射线 | 渲染内核 | S1 | 是 | 4 | 500 |
| S6a | context 参数/默认值/偏移/拒绝面 | context | S1 | 可选 | 4 | 500 |
| S6b | context 生命周期/所有权/显式 refit | context | S6a,S2,S3,S4 | 是 | 3 | 500 |
| S7a | 渲染发射 ABI（标量/vec/纹理/handle） | 渲染内核 | S2,S5 | 是 | 3 | 500 |
| S7b | 渲染内核（primitive+mesh+texture，RGB/depth/seg） | 渲染内核 | S2,S3,S4,S5,S6a,S7a | 是 | 12 | 500 |
| S8 | 端到端夹具与验收 receipt | e2e | S0b,S6b,S7b | 是 | 4 | 500 |

合计约 60 人日。S0a/S0b 是 authoring/夹具，不是产品能力；S1–S8 才进入产品代码。每片交付独立证据目录 `docs/development-evidence/render-bvh/<slice-id>/README.md` + JSON/LOG；切片完成只代表该片验收，不代表 R01 整体通过。

> 排程拆分：S2 已拆为 S2a/S2b/S2-close；S7b 已按输出阶段拆为 S7b1/S7b2/S7b3（见 6.1 推进表与 plan.json `structural_decisions`）。拆分只影响交付顺序与验收面，不改切片范围语义。

### 6.1 推进表（滚动，截至 2026-10-24）

本表只更新「切片 → 任务 id → owner → 状态 → 证据路径」，不改切片、验收与容差语义。

| 切片 | 任务 id | owner | 状态 | 证据路径 | 排程/窗口 |
| --- | --- | --- | --- | --- | --- |
| 预检 | `2026-10-24-r01-p8-render-bvh-precheck` | Arch | verified | `docs/development-evidence/render-bvh-precheck/` | 已完成（`d0ab630`） |
| S0a | `2026-10-24-render-bvh-s0a` | Backend | verified | `docs/development-evidence/render-bvh/s0a-reference-fixture/` | 已收口 |
| S0b | `2026-10-24-render-bvh-s0b` | Backend | verified | `docs/development-evidence/render-bvh/s0b-reference-capture/` | 已收口 |
| S0a+S0b 复核 | `2026-10-24-render-bvh-s0ab-independent-check` | QA | verified | `docs/development-evidence/render-bvh/independent-check/` | 已收口 |
| S1 | `2026-10-24-render-bvh-s1` | Backend | verified | `docs/development-evidence/render-bvh/s1-model-input/` | 已收口 |
| S1 复核 | `2026-10-24-render-bvh-s1-independent-check` | QA | verified | `docs/development-evidence/render-bvh/s1-independent-check/` | 已收口（`0b63829`，6/6 成立）；verified 检查不可改，实际门禁按 S1 自有路径执行，addenda/receipt 留痕 |
| S2a | `2026-10-24-render-bvh-s2a` | Backend | verified | `docs/development-evidence/render-bvh/s2a-scene-bvh-host/` | `f9adf14`；AABB 56/56 位等、max ULP 0、506 passed/0 failed |
| S2a 复核 | `2026-10-24-render-bvh-s2a-independent-check` | QA | verified | `docs/development-evidence/render-bvh/s2a-independent-check/` | `eb80566`；5/5 成立（56/56 位等、64/64、14/14 拒绝） |
| S2b | `2026-10-24-render-bvh-s2b` | Backend | verified | `docs/development-evidence/render-bvh/s2b-scene-bvh-gpu/` | `c7b5d7f`；383/382 比较、0 ULP、双 world 0 跨 |
| S2b 复核 | `2026-10-24-render-bvh-s2b-independent-check` | QA | verified | `docs/development-evidence/render-bvh/s2b-independent-check/` | `5ef8cb6`；9/9 成立、无阻断 |
| S2 收口 | `2026-10-24-render-bvh-s2-close` | Backend | verified | `docs/development-evidence/render-bvh/s2-scene-bvh/` | `651e253`；六条映射 + R01–R19 拒绝清单；S2 里程碑完成 |
| S3 | `2026-10-24-render-bvh-s3` | Backend | verified | `docs/development-evidence/render-bvh/s3-mesh-bvh/` | `8efd4ad`；436×5、distance/u/v 0 ULP、refit 32、退化网格 |
| S3 复核 | `2026-10-24-render-bvh-s3-independent-check` | QA | verified | `docs/development-evidence/render-bvh/s3-independent-check/` | `96e50d3`；436×5/0 ULP/边界/refit/退化独立复核 |
| S3 覆盖修正 | `2026-10-25-render-bvh-s3-coverage-fix` | Backend | verified | `docs/development-evidence/render-bvh/s3-mesh-bvh/` | `7b60e74` 修复 / `e02e01a` S3 追加；face 1/2/3 全覆盖、removed_rays |
| S4 | `2026-10-24-render-bvh-s4` | Backend | verified | `docs/development-evidence/render-bvh/s4-texture/` | `3e3426d`；硬件纹理对象 3×1040×4 逐位、524/0 |
| S4 复核 | `2026-10-24-render-bvh-s4-independent-check` | QA | verified | `docs/development-evidence/render-bvh/s4-independent-check/` | `cc5d223`；12,480 通道逐位、524/0；证据哈希修正 `9c8eb81` |
| S5 | `2026-10-24-render-bvh-s5` | Backend | verified | `docs/development-evidence/render-bvh/s5-camera-rays/` | `1df634d`；17 context 全网格 SHA-256、516/0 |
| S5 复核 | `2026-10-24-render-bvh-s5-independent-check` | QA | verified | `docs/development-evidence/render-bvh/s5-independent-check/` | `742e6d1`；17/17 全网格、123 抽样、7 拒绝、PTX f64 仅为已知 sin/cos 归约 |
| S6a | `2026-10-24-render-bvh-s6a` | Backend | verified | `docs/development-evidence/render-bvh/s6a-context-params/` | `dad622f`；34 resolved/15 拒绝、531/0 |
| S6a 复核 | `2026-10-24-render-bvh-s6a-independent-check` | QA | verified | `docs/development-evidence/render-bvh/s6a-independent-check/` | `1c0929e`；34/34、15/15 拒绝、531/0、无 GPU |
| S6b | `2026-10-24-render-bvh-s6b` | Backend | verified | `docs/development-evidence/render-bvh/s6b-context-lifecycle/` | `a171044`；100 次增长 0、显式 refit、531/0 |
| S6b 复核 | `2026-10-24-render-bvh-s6b-independent-check` | QA | verified | `docs/development-evidence/render-bvh/s6b-independent-check/` | `e3ea82c`；100 循环 growth 0 B、drop retained 0 B、5/5、531/0；1 条非阻断（5 个产物行数陈旧，sha256/bytes 一致） |
| S7a | `2026-10-24-render-bvh-s7a` | Backend | verified | `docs/development-evidence/render-bvh/s7a-launch-abi/` | `a0d9af0`；81 槽 ABI（13 类元素/56B RenderArray）、`WARP_1_15_RENDER`+`F32Render`、PTX 25996 B、GPU 2/2、532/0 |
| S7a 复核 | `2026-10-25-render-bvh-s7a-independent-check` | QA | verified | `docs/development-evidence/render-bvh/s7a-independent-check/` | `2bb8155`；A′ 成立（全 PTX 恰 2 条 f64、11 例变异判定 1:1）、81 槽 13/13、拒绝 10/10、2/2、532/0 |
| S7b1 | `2026-10-25-render-bvh-s7b1-kernel` | Backend | verified | `docs/development-evidence/render-bvh/s7b1-depth-seg/` | `3e43278`；depth 对同配方参考 **120,000/120,000 位等（max 0 ULP）**、seg 全等、532/0；originally 登记为 s7b1-depth-seg / s7b1-render-kernel，均因运行态范围冻结 abandon，最终以目录级范围 s7b1-kernel 交付 |
| S7b1 复核 | `2026-10-25-render-bvh-s7b1-independent-check` | QA | verified | `docs/development-evidence/render-bvh/s7b1-independent-check/` | `51ee2d0`+`7df5ad6`；120,000/120,000 位等、seg 0 mismatch；独立 PTX 反推球面顺序 747/747 位等、**换新位姿 813/813 位等**（错误顺序 187/192 像素 >4 ULP）→ 证明是算术顺序对齐非夹具调参；532/0 |
| S7b2 | `2026-10-25-render-bvh-s7b2-shading` | Backend | verified | `docs/development-evidence/render-bvh/s7b2-shading/` | `f228099`（配套 `a96a158` 句柄访问器）；81 槽签名迁移、RGB 对同配方参考 **120,000/120,000 像素 0 差异**、depth/seg 120,000/120,000 位等、灯位姿 6/6、PTX 195,594 B、532/0 |
| S7b2 复核 | `2026-10-25-render-bvh-s7b2-independent-check` | QA | verified | `docs/development-evidence/render-bvh/s7b2-independent-check/` | `38298e5`；独立重采同配方参考：RGB **0 差异/beyond_1_lsb=0**、depth max_ulp 0、新位姿泛化、灯位姿四路一致、81 槽逐槽核对；`mat_shin_exp` 语义被参考 headlight 调用点独立证实 |
| S7b3 | `2026-10-25-render-bvh-s7b3-flags` | Backend | verified | `docs/development-evidence/render-bvh/s7b3-flags/` | `52c22b7`；flag 变体 5/5 各自独立同配方参考 RGB **0 差异**、AA(spp=2) RGB 0 差异/depth 0 ULP/seg 0、不回归 S7b1/S7b2 基线、532/0；结掉 S5 AA deferral |
| S7b3 复核 | `2026-10-25-render-bvh-s7b3-independent-check` | QA | verified | `docs/development-evidence/render-bvh/s7b3-independent-check/` | `7e5affd`；独立重采 7 份参考：产品各组 RGB **位等 0 差异**/depth 0 ULP/seg 0；AA 480,000 射线布局反推 0 bad、depth/seg 只取首 pass 逐位成立；`vertex_normals_off` 无判别力如实登记；532/0 |
| S7b 配套修正 | `2026-10-25-render-bvh-s7b-followups` | Backend | verified | `docs/development-evidence/render-bvh/s7b-followups/` | `62c58a5`（+`33303a7` S7b3 证据修正、`1b2f79c` S7b2 addenda）；合成法线 pinned 参考产品 **0 差异**、on/off 653/9 像素可判别；行数 495/93/233 与注释纠正 |
| S8 | `2026-10-25-render-bvh-s8-e2e` | Backend | verified | `docs/development-evidence/render-bvh/s8-e2e/` | `d8bc9d8`；产品 camlight 位姿 **0 位差**、端到端 RGB **120,000/120,000 像素 0 差异**/depth 0 ULP/seg 0、反证成立（篡改夹具位姿不变；篡改当输入变 115,972 像素）、growth 0 B、532/0 |
| S8 复核 | `2026-10-25-render-bvh-s8-independent-check` | QA | verified | `docs/development-evidence/render-bvh/s8-independent-check/` | `20c728c`；自写驱动复核产品链：两相机/三灯位姿 **0 位差**、模型输入灵敏度独立验证、篡改判别 6 origin+17 xmat 位不同；端到端同配方 0 差异/0 ULP/seg 0；1 ULP 判为辅助口径非缺陷；0 阻断 |
| 渲染 perf | `2026-10-25-render-bvh-perf` | Backend | verified | `docs/development-evidence/render-bvh/perf/` | `eadae87`（+`18ae715` S8 非阻断修正）；单帧 median 14.030/14.389 ms、AA 36.174/36.423 ms、AA/单帧 2.578/2.531、显存峰值 54 MiB；**相对基准**；r01-p8 第 3 条补成满足 |
| perf 复核 | `2026-10-25-render-bvh-perf-independent-check` | QA | in_progress | `docs/development-evidence/render-bvh/perf-independent-check/` | 30 分钟 GPU；核计时与原始序列同源/独占窗口/显存口径/相对措辞/未测项 + `18ae715` 三点修正 |

结构说明：
- 原 `2026-10-24-render-bvh-s2` 已 abandon（`8d4357f`，理由：运行中父任务写路径锁与派生任务冲突、冻结字段不可改）；拆分前参考捕获 `75166cf` 保留在 git 历史与 `fixtures/render-bvh/scene-a/s2/`，S2b 继续引用。
- S2a/S2b/S2-close 登记于 `2a62e2b`；三任务均已 verified（`f9adf14` / `c7b5d7f` / `651e253`），S2 里程碑完成。
- S2a 复核 `eb80566`（5/5）与 S2b 复核 `5ef8cb6`（9/9）均 verified；QA 的 `use_fast_math=True` 参考 vs 产品 `false` 已作为非阻断观察项入 S2-close 证据。
- S2 收口后进入 S3：mesh 三角 BVH 已 verified（`8efd4ad`），S3 复核 `96e50d3` 已 verified；S3 里程碑完成。
- S3 覆盖修正已完成（`7b60e74` 修复 / `e02e01a` S3 追加）：退化网格 17 条/10 命中，face 1/2/3 全覆盖；S3 原验收结论不变。
- S5 已 verified（`1df634d`）：17 context 全网格 SHA-256 位等，GPU 2/0、主机 4/0、516/0；AA 图像延后 S7b/S8，世界系归 S7b，正交 authoring 副本，`sha2` dev-dep。
- S5 复核已 verified（`742e6d1`）：17/17 全网格、123 抽样、7 拒绝面、516/0；PTX 4 处 f64 token 均为已知 sin/cos 整数角归约，无物理 f32 提升。
- S4 已 verified（`3e3426d`）：硬件纹理对象 3×1040×4 逐位、524/0；S4 复核 `cc5d223` 已 verified；证据哈希修正 `9c8eb81` 已挂。
- S6a 已 verified（`dad622f`）：34 resolved/15 拒绝、531/0；S6a 复核 `1c0929e` 已 verified。
- S6b 已 verified（`a171044`）：100 次 create/destroy 增长 0、显式 refit、531/0；S6b 复核 `e3ea82c` 已 verified（QA 独立重算 growth 0 B、drop retained 0 B、5/5、531/0），1 条非阻断证据问题（5 个产物行数陈旧，sha256/bytes 全一致）按 `normify fix` 修正，不改验收结论。
- S7a 已 verified（`a0d9af0`，22 文件）：81 槽渲染发射 ABI（1 bounds + 73 数组 + 7 标量 + 4 输出、13 类元素宽度/对齐、56 字节 `RenderArray`）、`RecipePurpose::Render` + `WARP_1_15_RENDER`（fmad=true/fast-math=false/C++17/compute_70/`PrecisionGate::F32Render`）、W15+NVRTC PTX 25,996 B、GPU round-trip 2/2、门禁 532/0；S7a 复核已 start（30 分钟 GPU）。
- **S7a 口径新增（A′，物理门禁不变）**：W15 `svd.h` 经 `builtin.h` 固定带两条 `_svd_config<double>` 死数据常量（`SVD_EPSILONE`/`QR_GIVENS_EPSILONE` = `0d3D719799812DEA11`）；渲染门禁精确放行这两条（符号名+字面量+行尾逐字节、各一次），其余 f64/f16/bf16/tf32 与无 sin/cos 归约三元组的 `.reg .f64` 一律拒绝，`check_physics_ptx` 未改。这是精确白名单，不是容差放宽；QA 复核列为必查项。
- S7a 偏差（已被接受）：裁决④ `mjwarp.render.kernel` 叶子模块被 normify 结构规则阻塞（父模块 `mjwarp.render` 声明了 API）；表现为 kernel 源码登记进 `mjwarp.render`、launcher 源码登记进 `mjwarp.runtime`，validate 0 错误。另登记 backlog `2026-10-25-render-bvh-module-split`（S7b2 后 promote：先拆 `mjwarp.render` API 再建叶子）。
- **S7b 拆片（`74928dd`）**：plan 的 S7b（渲染内核，估 12 人日）按输出阶段拆为 S7b1（主射线/世界系变换/深度/分割）→ S7b2（着色/RGB）→ S7b3（flag 变体矩阵 + AA 图像级参考，需新 authoring 捕获，结掉 S5 的 AA deferral）。理由：单任务单窗口单验收面不可交付，三片各有可判定冻结参考；`src/render/kernel/launch.rs` 已恰 500 行，后续片不得改长该文件。
- S7a 复核已 verified（`2bb8155`）：A′ 精确放行面成立（全 PTX 恰 2 条 f64、非 `.global` f64 行 0 条、11 例注入/变异判定与产品 1:1、`ptx.rs` diff 为空）、81 槽 ABI 13/13、`RenderArray` 56 B、写前拒绝 10/10、GPU round-trip 2/2、532/0；`launch.rs` 恰 500 行。偏差④ 仅注明、未当缺陷。
- 窗口语义修正（QA 提出，采纳）：复核类任务的「GPU 窗口」只约束 GPU 占用，CPU 侧独立 harness 编译/变异/比对可以放在窗口外；S7a 复核实际 GPU 占用约 2 秒、CPU 迭代超窗口约 20 分钟，期间显存保持桌面基线、无争用。后续窗口按此口径计时并记录，不把 CPU 迭代时间算进 GPU 独占。
- **S7b1 已 verified（`3e43278`）**：主射线/世界系变换（`cam_xmat @ dir`、正交、`max_cam_dist`）/平面深度（米，0=无命中）/(object_id, object_type) 分割；对**同配方（`use_fast_math=False`）**参考两相机 **120,000/120,000 像素 depth 逐位相同（max 0 ULP）**、seg 逐像素全等（含 mesh geom3 754 像素）；dynamic≡precomputed、双 world 零跨、`set_points`+显式 refit 变化 1698 像素且两次位等、禁用 depth/seg 保持 `DisabledOutput`、A′ 门禁过（PTX 137,824 B、两个 entry）、S6b 回归 5/5、门禁 532/0；GPU 起 4193 → 终 4205 MiB（桌面波动、无计算进程残留）。
- **S7b1 技术裁决（NVRTC 收缩顺序对齐，`3e43278`）**：产品 CUDA 12.8.1 NVRTC 与参考 warp 自带 NVRTC（12.9/sm_89）对球面求交的 **fma 收缩顺序不同**（a/b/dd 分量先乘顺序、det 是否融合）；cam1 掠射处 det≈4e-5、敏感度 -25…-78，1 ULP 被放大成数十 ULP（原 149/43,200 像素 >4 ULP、max 73）。修法是在 `src/render/kernel/cuda/render_math.cu` 新增 `render_sphere_hit`，用显式 `__fmul_rn`/`__fmaf_rn`/`__fsub_rn` 按参考 PTX 实际发射顺序固定，`render.cu` 只对 `geom_type==2` 改调；`scene.cu` 只读、A′ 配方与容差未动、参考未换。**不是放宽容差，是对齐上游编译产物的运算顺序**；S7b2/S7b3/S8 着色路径沿用同一方法。
- **参考配方实验留痕**：按裁决 B 补采过 `s7b/depth_seg_fast_math_false.json`（1,492,762 B，`div.rn` 206/`sqrt.rn` 77/`rcp.rn` 89、`div.approx` 0，两次捕获位等），证明参考确实关掉了快算；该参考现作为 depth 0 ULP 硬指标依据，S0b fast-math 参考的跨配方差异只作已知偏差记录。A/B/C 三个「放宽容差/位级对齐/改 arch」方案因修复而**全部作废**（B 实质已由修复完成）。
- **S7b1 任务登记过程留痕**：原登记 `2026-10-25-render-bvh-s7b1-depth-seg`、`...-s7b1-render-kernel` 均因 normify「运行中任务声明/范围冻结」（start 后无法补 `owner.rs`/`render_math.cu`/`renderer.rs`/`tests/support/` 路径）而 abandon，最终以**目录级范围** `2026-10-25-render-bvh-s7b1-kernel` 交付；教训：新切片任务的范围按目录粒度登记，避免运行期冻结后无法扩。
- S7b1 复核已 verified（`51ee2d0`+`7df5ad6`）：独立重解码 120,000/120,000 位等、seg 0 mismatch；独立从参考 PTX 反推球面 fma 顺序，参考顺序 **747/747 位等**、错误顺序仅 87/747（187 像素 >4 ULP）；**换新位姿后 813/813 位等**、错误顺序 192 像素 >4 ULP → 证明是对齐上游算术顺序而非对夹具调参；A′ 门禁/`scene.cu`/`launch.rs`/532/0 复核成立。
- **S7b2 已 verified（`f228099`，配套 `a96a158`）**：81 槽 `_render_megakernel` 签名迁移（1 bounds + 80 形参、逐槽 real/placeholder 表、`bvh_id` 自有打包 blob 仅签名对齐）；着色 headlight/背景/base color/纹理/顶点法线/自发光/高光/背面剔除，RGB 写 packed ABGR；**RGB 对同配方（`use_fast_math=False`）参考两相机 120,000/120,000 像素 0 差异**；depth/seg 120,000/120,000 位等（S7b1 回归门保住）；灯位姿由 `RenderFields` 按上游 f32 序列推导并与 pinned 3.15 位等（6/6）；A′ 门禁 PTX 195,594 B；fmt/clippy/clippy-cuda/unit 532/0、S7b1 回归 6/6、S6b 5/5；GPU 3335→3335 MiB。
- **S7b2 根因（参数级，不是 fma 顺序）**：修复前 RGB 对 S0b fast-math 参考 cam1 332/43,200 像素 >1 LSB（max 3 byte，geom2 球面 174 + geom5 圆柱 158），排查链为「同配方参考排除跨配方 → no-textures 消融排除纹理 → 上游源码逐行对照」，根因是 headlight 高光指数传了固定 64 而非材质 `mat_shin_exp`（上游 `var_340=128`、`var_353=mat_shininess*128`）。修复后同配方参考 0 差异；**对 S0b fast-math 参考的实际记录是 cam0 `[76798,2,0,0,0]`（2 像素差 1 byte）、cam1 全位等**（此前表中 332 为修复前症状）。分层口径 `rgb_stats_vs_same_recipe_reference`（硬指标）/`rgb_stats_vs_s0b_fast_math_known_deviation`（跨配方已知偏差，不作通过依据）已写入 receipt，S8 沿用。
- **S7b3 已 verified（`52c22b7`）**：flag 变体矩阵 5/5（textures/culling/specular/emission/vertex_normals off）各自独立同配方参考、产品 RGB **0 像素差异**；`culling_off` depth 0 ULP/seg 0；AA(spp=2→nsamples=4) 对同配方参考 RGB **0 差异**、depth 0 ULP、seg 0、两次运行位等、子样本顺序 = 上游 `ray_base=sample*total_rays`；**修复 81 槽迁移遗漏的「depth/seg 单样本输出只写 `ray_base==0`」**（AA depth 首轮 78% 不匹配的根因）；A′ 门禁 megakernel 197,583 B + aa_resolve 27,536 B；`megakernel.rs` 因 500 行棘轮拆出 `megakernel_aa.rs`（474/86/216 行）；不回归 S7b1 depth/seg 与 S7b2 RGB 基线；532/0、S7b1 6/6、S6b 5/5、GPU 回 3449 MiB。**S5 的 AA 图像级 deferral 在此结项**。
- **S7b3 两处证据强度缺口（已登记 `2026-10-25-render-bvh-s7b-followups`）**：① `vertex_normals_off` 在 scene-a 参考自身开/关逐位相同（mesh 仅 cam1 754 像素、8 位量化吸收）→ 无判别力，现有合成法线用例只证明产品分支生效、缺 pinned 参考对照，需补 after-override 参考；② S7b2 的 `unsupported_configurations_are_rejected_before_launch` 中 spp=2 拒绝断言因 S7b3 实现 AA 而失效，按「语义升级」改为 3 类拒绝 + AA 支持路径，并用 `normify fix` 给 S7b2 证据留痕（不改原验收数值）。
- **S7b2 范围冻结留痕（第三次）**：S7b2 files 目录级但漏 `src/render/context/owner.rs`，而**已关闭任务不能改 files**（钩子拒「关闭时不能改写已冻结字段」）；处置为把 6 行只读访问器拆成独立任务 `2026-10-25-render-bvh-texture-handles` 先提交（`a96a158`），S7b2 按原范围随后提交（`f228099`），验收不变。教训升级：任务范围要覆盖整个 `src/render/context/` 目录，不只列单文件。
- `gpu:windows-t4` 是项目排程互斥名，不等于 T4 硬件；本机实际为 RTX 4070 Ti SUPER / SM89 / 驱动 596.36。开窗前核对设备，GPU 满载任务串行。
- S1 复核的 `unchanged` 结构裁决已由 QA 在 receipt/addenda 留痕；normify 0.7.0 不允许改已 verified 检查，记录保持登记宽口径，实际门禁按 S1 自有路径执行。
- S3 前排程注记：先补两个新叶子模块 `mjwarp.model.render-model`、`mjwarp.io.render` 的 layout order/apis；`src/model/render.rs` 当前正好 500 行，后续任何加功能必须先拆文件，S3+ 切片不得撞 500 行棘轮。

### S0a 参考夹具与模型 parity

- 范围：同一 `scene.xml` 编译 3.12/3.15 两份 MJB；3.12 侧 C++ dump（authoring）、3.15 侧 Python dump；逐字段 parity。
- 交付：`scripts/authoring/render-bvh/compile_scene_312.cpp`（≤300 行）、`model_parity.py`（≤250）、`fixtures/render-bvh/<scene>/scene.xml`、`scene-312.mjb`、`scene-315.mjb`、`model-312.json`、`model-315.json`、`parity.json`。
- 验收：两份 MJB 各自由对应版本加载；渲染相关数组（geom/camera/light/material/texture/mesh/vis/name 表）逐元素相等；任何差异列 `field/index/expected/actual` 并使切片失败；脚本重复运行摘要一致。
- GPU：无。证据：`parity.json`、MJB/XML 哈希、工具版本。

### S0b 冻结参考捕获

- 范围：参考渲染图像/深度/分割、固定射线集命中、context 默认值 receipt；两次重复。
- 交付：`scripts/authoring/render-bvh/capture_reference.py`（≤450）、`reference/*.npz`、`reference/*.json`、`provenance.json`。
- 验收：连续两次输出 SHA-256 相同；射线集覆盖相机像素、随机种子、擦边/无命中；context receipt 含 28 参数解析值与输出偏移；provenance 完整。
- GPU：是（RTX 4070 Ti SUPER / SM89；换 GPU 重采）。

### S1 渲染模型输入

- 范围：扩展 native ABI 与 Rust 模型，读取并校验相机/光源/材质/纹理/网格/vis/名称；只做宿主输入，不做 GPU 内核。
- 交付：`native/model_probe.cpp`（扩展）、`include/mjwarp_render_probe.h`、`src/model/render.rs`、`src/io/render.rs`、测试。
- 验收：对 3.12 MJB 的每个渲染字段与 `model-312.json` 精确相等；计数/长度/溢出/负值拒绝；名称查找与上游 `mj_id2name` 语义一致；无 GPU。
- 证据：host 测试报告 + JSON diff（0 差异）。

### S2 场景 BVH

- 范围：primitive geom 的 AABB 计算、SAH 构建、group root、refit、射线查询（`geomid/dist/normal`）。
- 验收：固定射线集在初始位姿与移动位姿（refit 后）均与参考命中一致；`geomid` 精确、`dist` ≤4 ULP、`normal` ≤8 ULP/角度阈值；AABB 逐位相等；world group 不串；空场景拒绝。禁止树布局断言。
- 证据：`bvh-ray-parity.json`（逐射线 diff、最差 ULP、超限数）+ GPU 日志。

### S3 网格 BVH

- 范围：`mesh_vert/face` 三角 BVH 构建/重拟合、`mesh_query_ray` 语义（u/v/face/normal）、动态点更新。
- 验收：网格射线命中三角形/uv/法线一致；退化/擦边/双面案例记录；更新点后 refit 命中变化与参考一致；与 S2 共用查询 ABI。
- 证据：`mesh-ray-parity.json`；纹理/图像在 S7b/S8 复验。

### S4 纹理

- 范围：u8→float4 转换、纹理对象/采样、纹理注册表；天空盒只留接口，不验收。
- 验收：转换后字节/通道与参考一致；固定 uv 网格采样 float4 每通道 ≤`1e-6`；纹理开/关渲染 patch 与参考一致。
- 证据：`texture-parity.json`、采样表、图像 patch。

### S5 相机/射线/AA

- 范围：`compute_ray`、透视/正交、内参/传感器、znear/aspect、相机筛选、`samples_per_pixel`、预计算/动态射线。
- 验收：每相机/投影像素射线方向/偏移与参考一致（优先逐位，否则 ≤1 ULP）；AA 2×2 图像一致；`samples_per_pixel>1` 组合限制与上游一致；空相机列表合法。
- 证据：`camera-rays.json`、AA 图像摘要。

### S6a context 参数/默认值/拒绝面

- 范围：`RenderContext` 参数解析、None 推导、相机筛选、输出偏移、`znear/zfar`、buffer 大小、错误面。
- 验收：调用矩阵（默认/布尔/列表/名称/索引/空列表/异构分辨率）解析值与参考 receipt 精确一致；偏移/大小精确；错误类别与上游对应；不要求 GPU 渲染。
- 证据：`context-defaults.json`、`context-rejections.json`。

### S6b context 生命周期

- 范围：资源所有权（BVH/mesh/texture/output buffer）、drop、重复 create/destroy、显式 `refit_bvh`、禁止隐式刷新、布局/寿命检查。
- 验收：100 次 create/destroy 显存稳定（阈值另定）、无 UAF；移动几何后 `render` 仍用旧 BVH，`refit_bvh` 后才更新（与上游语义一致）；禁用输出不可读。
- 证据：`lifecycle.json`、显存记录、行为断言日志。

### S7a 渲染发射 ABI

- 范围：标量/`vec3`/`vec4`/`u64`/纹理数组/句柄打包；kernel module 加载；输出缓冲；参数数量/类型拒绝。
- 验收：测试内核逐类实参 round-trip（语义值相等）；错误参数被拒；W15 + NVRTC 编译通过；生成源编译验证（§5.3）。
- 证据：`launch-abi.json`、编译日志、round-trip 表。

### S7b 渲染内核

- 范围：primitive + mesh + texture 的通用 megakernel（运行时分支，不做模型特化）；RGB/深度/分割；headlight/ambient/背景/背面剔除/顶点法线/specular/emission；阴影/天空盒/泼溅/柔体/高度场不做。
- 验收：夹具图像 RGB 每通道 ≤1 LSB、深度 ≤4 ULP、分割精确；两次运行哈希相同；开关组合（纹理/法线/specular/emission/背剔）与参考一致；失败输出逐像素 diff。
- 证据：`render-images/`、`image-diff.json`、`receipt.json`。

### S8 端到端

- 范围：夹具含 primitive + mesh + texture + 2 相机；两帧之间 refit；默认 context；输出 RGB/深度/分割；生命周期。
- 验收：全部输出在容差内；无 Python；重复哈希一致；负例（错误相机/陈旧 BVH/禁用输出）被测试捕获；receipt 完整。
- 证据：`e2e-receipt.json`、图像、diff、命令与版本。

## 7. r01-p8 原验收映射

| 原验收项 | 切片 | 说明 |
| --- | --- | --- |
| 世界 | S6a、S8 | `nworld=1` 默认与单 world 行为；`nworld>1` 归 R10，不在本 MVP |
| 相机 | S1、S5、S6a | 字段、筛选（bool/名称/索引/空）、射线、分辨率/内参 |
| 资产 | S1、S3、S4 | mesh/texture/material/纹理上传 |
| BVH | S2、S3 | 场景/网格构建、refit、查询 |
| 全部默认参数 | S6a | 28 个参数逐项 receipt；默认值表见 `module-contracts.md` L657–672 |
| 生命周期 | S6b、S8 | 所有权、create/destroy、显式 refit、无隐式刷新 |
| RGB 输出 | S7b、S8 | 打包/解包、通道序、图像容差 |
| 深度输出 | S7b、S8 | 原始深度、无命中 0、`get_depth` 缩放 |
| 分割输出 | S7b、S8 | `(id,type)`、背景 `(-1,-1)` |
| 资源/布局检查 | S6b | 强制布局与寿命检查；严格门面可选（不新增新鲜度拒绝） |
| 拒绝面 | S6a | 上游组合限制/非法输入 |
| 阴影/天空盒/泼溅/柔体/高度场/多相机 | 不在本 MVP | 分别归 R09/R02/R07/R06/R05/R08；默认关闭不阻塞默认参数验收 |

## 8. 风险、阻塞与裁决

### 风险

| 风险 | 影响 | 缓解 |
| --- | --- | --- |
| Warp 无离线渲染参考（原假设错误） | 不能按原计划直接对照 | 改用 MJWarp 参考；Warp 只提供低层头文件；OpenGL 视口不纳入 |
| MuJoCo 3.12/3.15 模型差异 | 参考图与产品输入不一致 | S0a 双 MJB + 逐字段 parity；失败即停 |
| W15 缺 BVH/mesh/texture 主机/设备实现 | 需要大段移植 | S2/S3/S4 分片；优先 SAH/LBVH；cuBQL 可选 |
| 渲染内核体积与特化 | 500 行限制难满足；Warp 生成源特化 | 手写通用 CUDA + 运行时分支；生成源仅作 oracle；按功能拆 S7b/S8 |
| 发射 ABI 只支持数组 | 渲染内核无法直接发射 | S7a 专用打包；沿用逐内核 launcher 模式 |
| NVRTC/Warp 数学差异 | 不能逐位 | 冻结配方；ULP 预算；记录最差误差 |
| 纹理硬件过滤 | 跨 GPU 不一致 | 同 GPU 参考；必要时软件采样并单独验收 |
| 许可/再分发 | cuBQL/CUDA/Warp 生成源 | Apache-2.0 记录；生成源 provenance；CUDA EULA 另议 |
| Python 边界 | 参考生成与产品链冲突 | 仅 authoring；产品测试只读冻结文件；CI 检查无 Python |
| 范围蔓延 | R01 被当成全部渲染 | MVP 明确；R05–R10 另立；不宣称完整 R01 |

### 阻塞项

1. **参考/产品模型版本口径**：3.12 vs 3.15。需要确认「双 MJB + parity」可作为正式口径；否则参考不可用于验收。
2. **内核路线**：手写通用 CUDA（产品可用、周期长）vs 冻结 Warp 生成 CUDA（快、但只覆盖特化夹具，不是通用产品）。需要裁决；建议前者，生成源只作 oracle。
3. **网格 BVH 构建器**：上游用 cuBQL；产品若用 SAH/LBVH，需接受「语义一致、树不同」并用射线/图像验证；若要求树级一致，需移植 cuBQL（工作量+许可）。
4. **MVP 范围**：是否同意把阴影/天空盒/泼溅/柔体/高度场/多相机排除在 r01-p8 MVP 之外；若不同意，切片需重划。
5. **图像容差**：RGB ≤1 LSB、深度 ≤4 ULP、分割精确是否可作为验收阈值；需书面确认。
6. **GPU 口径**：MVP 只认 Windows RTX 4070 Ti SUPER/SM89；T4/Linux 另立任务。

### 需要用户/Chief 裁决

- 决策 1（版本）：推荐保留产品 MuJoCo 3.12，参考用 3.15，双 MJB + parity；备选是升级产品 native ABI 到 3.15（影响所有既有 native 测试，不建议在本项做）。
- 决策 2（内核）：推荐手写通用 CUDA（NVRTC + W15），把 Warp 生成 CUDA 仅作为 oracle/对照；备选是冻结生成源做夹具 e2e（快，但不能证明通用能力）。
- 决策 3（网格 BVH）：推荐先 SAH/LBVH + 语义验收，出现 tie/uv 不一致再评估 cuBQL；备选是直接移植 cuBQL（更接近上游，但周期与许可成本高）。
- 决策 4（范围/容差）：确认 §6 的 MVP 范围与 §4.3 容差；确认后由 Chief 重划 r01-p8 并逐片派 Backend。

## 9. 复现与环境

- 参考 smoke：见 §5.1；脚本 `smoke-render.py` 只依赖已安装的 authoring venv，不写产品文件；缓存写系统临时目录。
- 版本锁：§5.2 的命令使用 `target/toolchains/mujoco-3.12.0/package` 与 venv 的 3.15；MJB 编译器是临时 C 工具，S0a 落仓后按脚本复现。
- 上游引用清单：

```text
python - <<'PY'
# 伪代码：遍历 target/mjwarp-parallel/upstream/mujoco_warp/**/*.py
# 匹配 render|bvh|RenderContext|ray_mesh_with_bvh，写 file/line/scope/text
PY
```

本目录 `upstream-references.tsv` 即该清单（964 行；source 464、test 500）。

## 10. 不宣称

- 不宣称渲染或 BVH 能力已经实现；不宣称任何验收项通过。
- 不宣称性能、显存、T4/Linux、多 GPU 覆盖。
- 不宣称阴影、天空盒、泼溅、柔体、高度场、多相机可用。
- 不宣称冻结参考可替代产品测试；参考只用于对照。
- 不宣称 W15 头文件已足够构建 BVH/资产/渲染上下文；主机/设备实现仍需移植。
