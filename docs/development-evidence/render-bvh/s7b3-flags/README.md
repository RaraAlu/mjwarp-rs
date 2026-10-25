# 渲染/BVH S7b3：flag 变体矩阵与 AA 图像级参考

任务：`2026-10-25-render-bvh-s7b3-flags`（渲染/BVH S7b3）
冻结上游：`71da24d956378a87a703b6e1442b13aec0c4ac29`（MJWarp 3.15）
精度门禁：`f32-render`（A′ `check_render_ptx`，编译只走 `compile_render_artifact`）
GPU：NVIDIA GeForce RTX 4070 Ti SUPER / 驱动 596.36 / SM89

## 1. 结论

- **flag 变体矩阵**：5 组变体（textures_off、vertex_normals_off、specular_off、emission_off、
  culling_off）**各自一份**同配方（`use_fast_math=False`）参考；产品对各自参考：
  RGB **0 像素差异**（判据 ≤1 LSB）、culling_off 的 depth **0 ULP** / seg **0 不一致**。
- **AA（spp=2）**：nsamples=4；产品对同配方 AA 参考 RGB **0 像素差异**、depth **0 ULP**、
  seg **0 不一致**；两次运行逐位相同。子样本顺序 = S5 布局 `[sample][camera][row][column]`，
  与上游 `for sample in range(nsamples)` + `ray_base = sample * total_rays` 一致。
- **不回归基线**：S7b1 depth/seg **120,000/120,000 位等、max 0 ULP**；S7b2 RGB 对同配方参考
  **120,000/120,000 像素位等、0 差异**。
- **S5 的 AA 图像级 deferral 在本片结项**（映射见 §6），仍不宣称 R01 整体通过。
- 门禁：fmt/clippy/clippy-cuda 通过、unit 532/0；S7b1 GPU 回归 6/6；S6b 回归 5/5；
  S7b2 GPU 回归（RGB/depth/seg 三项）全过，**唯一一项预期失效**见 §8。
- PTX（最终源码）：megakernel **197,583 B**（`source_sha256=e0fd7ed1…`、`ptx_sha256=582b2429…`）；
  `aa_resolve` **27,536 B**（`source_sha256=0bf8af5a…`、`ptx_sha256=b28e155c…`）；两者都过
  A′ `f32-render` 门禁（`check_render_ptx`）。

## 2. 实现

### 2.1 AA（`samples_per_pixel=2` → nsamples=4）

上游语义（pinned 3.15 `render.py`）：
```text
nsamples = samples_per_pixel²
if nsamples > 1: aa_accum.zero_()
for sample in range(nsamples):
    launch _render_megakernel(..., ray_base = sample * total_rays)
if nsamples > 1:
    launch _aa_resolve(accum, 1/nsamples) -> rgb_data
store_pixel: aa 时 aa_accum[worldid, adr] += color * 255.0；否则 pack 并写 rgb_out
_aa_resolve: c = accum * inv_n; rgb_out = pack(clamp(c, 0, 255), 255)
```

产品实现（`src/render/kernel/`）：
- `megakernel.rs`（495 行）：`nsamples = layout.nsamples`；`for sample in 0..nsamples` 发射
  `render_megakernel`，只改 `ray_base` 槽（47）= `sample * total_rays`；之后发射
  `aa_resolve`（`AA_RESOLVE_ENTRY`，4 槽：bounds + accum vec3 + rgb u32 + inv_n f32[1]）。
  `inv_n = 1.0 / nsamples`（f32），与上游 `1.0 / float(nsamples)` 一致。
- `megakernel_aa.rs`（93 行，本片新增）：AA 子样本发射（`launch_samples`，只改 ray_base 槽）
  与 `AAResolve::resolve`（resolve 发射）；从 `megakernel.rs` 拆出以满足单文件 ≤500 行棘轮
  （`megakernel.rs` 483 → 本次 +AA 后 549，拆分后 495；`megakernel_arrays.rs` 233 行）。
- `cuda/render_megakernel.cu`：`MK_FLAG_AA`（bit8）；`aa_store` 按上游 `store_pixel` 累积
  `color * 255.0`；**`ray_base == 0` 时覆盖写**（免每帧 host 清零，逐样本求和顺序与上游
  `zero_()` + `+=` 等价，每个像素每个 sample 都写，含背景）。
- `cuda/aa_resolve.cu`：`_aa_resolve` 的逐式翻译（自带 pack 等价实现，仅依赖 `builtin.h`）。
- **depth/seg 单样本语义**：上游「Depth and seg are single-sample outputs: only the first
  pass writes them」（`if ray_base == 0`）；产品在 depth/seg 写入处加 `ray_base == 0`
  （spp=1 时恒真，S7b1/S7b2 基线不变）。这是本片首轮 AA depth 78% 不匹配的根因，修复后 0 ULP。
- `context/lifecycle.rs`：`OutputResources` 新增 `aa_accum`（`3 × aa_accum_width` 个 f32 的扁平
  缓冲，`nsamples == 1` 时宽度 1 占位）与 `aa_accum_render_arg_2d`（crate 可见）。
- `context/` 现有 `OutputLayout.aa_accum_width` = `nsamples > 1 ? rgb_pixels : 1`（S6a 已有）。

### 2.2 flag 变体

S7b2 的 81 槽内核已按 `splat_bvh_id` flags 位传递全部开关（textures/vertex_normals/specular/
emission/per_light_ambient/ambient_lighting/headlight/cull_backfaces）＋ S7b3 新增 AA 位；
本片**不改内核着色语义**，只补 AA 分支与单样本 depth/seg 语义。各变体通过 `ContextParams`
切换（`variant_params`），与 authoring override 一一对应。

## 3. 变体矩阵与判据

夹具目录：`fixtures/render-bvh/scene-a/s7b/flags/`；采集脚本
`scripts/authoring/render-bvh-s7b/capture_flags_aa.py`（pinned 3.15、`PARAMS_28` 仅变体 override、
`use_fast_math=False`、两次捕获逐位一致）。

| 变体 | override | RGB 参考 | depth/seg 参考 | 产品结果 | vs base 差异（判别力） |
| --- | --- | --- | --- | --- | --- |
| `textures_off` | `use_textures=False` | `textures_off_rgb.json` | —（只影响着色） | RGB 0 差异 | cam1 4,130 像素（geom1/2/3/5，即有纹理几何） |
| `vertex_normals_off` | `enable_vertex_normals=False` | `vertex_normals_off_rgb.json` | — | RGB 0 差异 | **0（本夹具无判别力，见 §5）** |
| `specular_off` | `enable_specular=False` | `specular_off_rgb.json` | — | RGB 0 差异 | cam0 55,504 + cam1 2,457 |
| `emission_off` | `enable_emission=False` | `emission_off_rgb.json` | — | RGB 0 差异 | cam1 1,947（geom1/3） |
| `culling_off` | `enable_backface_culling=False` | `culling_off_rgb.json` | `culling_off_depth_seg.json` | RGB 0 差异、depth 0 ULP、seg 0 | cam1 RGB 38 / seg 29 / depth 38（背面命中） |
| `aa2` | `samples_per_pixel=2`（nsamples=4） | `aa2_rgb.json` | `aa2_depth_seg.json` | RGB 0 差异、depth 0 ULP、seg 0 | cam0 534 / cam1 1,320 像素 |

判据（逐变体独立）：RGB 解包后每通道 ≤1 LSB；depth ≤4 ULP（同配方参考）；seg 逐像素全等。
产品对**每一组**都达到 0 差异/0 ULP/0 不一致。

### 3.0 逐 geom 归因口径

交付的 `rgb_changed_by_geom` 按**变体/AA 的 seg**（产品侧 seg 参考，即该变体自己的 seg）
归因；QA 复核按 **base seg** 归因。同一像素在「变体 seg」与「base seg」下的 geom 归属可能不同
（例如 culling_off 让原本 miss 的像素命中背面 geom）——两种口径语义等价、不是数值分歧；交付的
逐 geom 计数与测试断言以本变体 seg 为准。

### 3.1 变体语义断言（可判定，不靠「不崩」）

- **texture off 不采样纹理**：与 base 的差异像素按 seg 几何全部落在 geom1/2/3/5
  （`mat_texid` 有效的几何），无纹理几何（geom0/4/6）逐位相同；`index.json` 的
  `rgb_changed_by_geom` 记录逐 geom 计数，测试断言其非空。
- **culling off 背面命中**：与 base 相比 seg 变化 29 像素、depth 变化 38 像素（背面被命中），
  且产品对 culling_off 的 depth/seg 参考 0 ULP/全等。
- **specular/emission off 贡献消失**：与各自关项参考一致（0 差异），且与 base 差异非空
  （cam0 55,504 / cam1 2,457；cam1 1,947）。
- **vertex_normals**：见 §5（本夹具无判别力，合成法线用例承担判别）。

## 4. AA 图像级验收与 S5 deferral 映射

S5 记录：「AA 图像级验收按 Arch 裁决延后 S7b/S8，本片只验射线层」「AA 合成图像：产品尚无渲染
内核，图像级验收延后 S7b/S8」。

本片结项映射：
| S5 遗留项 | S7b3 承接方式 |
| --- | --- |
| AA 射线层（子样本顺序/布局） | S5 已有：`[sample][camera][row][column]`、`planned_ray_count = samples × total_rays` |
| AA 合成图像（产品内核） | 本片：megakernel 每 sample 一次发射 + `aa_accum` 累积 + `aa_resolve`，对同配方 AA 参考 **0 像素差异** |
| AA depth/seg 单样本语义 | 本片：`if ray_base == 0` 只写第一个 sample（上游语义），对参考 0 ULP / 全等 |
| AA 验收层 | 本片图像级验收；仍不宣称 R01 整体通过 |

## 5. `vertex_normals_off` 在本夹具下无判别力（如实记录）

- scene-a 唯一 mesh 是 geom3（4 顶点 / 4 面），仅 cam1 可见 **754 像素**；
- 顶点法线与面法线点积：face1 三点全部 `do < 0.8`（回退面法线），face0/2/3 部分点
  dot = 0.9635/1.0000/0.4664/0.5726 等 → 插值法线与面法线的差异在该视角下被 8 位量化完全吸收；
- pinned 3.15 参考自身 `enable_vertex_normals=True/False` 两版**逐位相同**（0 像素差异），
  属数据/量化原因，**不是产品缺陷**；
- 可判别替代（本片测试承担）：`synthetic_mesh_normal_switches_vertex_normal_path` 把
  `mesh_normal` 换成 6 个极端方向，渲染 `enable_vertex_normals=True/False` 两版：
  `[1,0,0]` → **653 像素不同**、`[0,0,1]` → 9 像素不同、其余 0 → 断言「至少一个方向必须不同」，
  证明 `vertex_normal` 分支生效；reference 夹具仍保留并对产品断言 RGB 0 差异（判据不降级）。
- 结论：该变体在本夹具下**无判别力**（已在 `index.json` 以 `has_discriminating_effect=false`
  记录，测试断言该记录），分支判别由合成法线用例承担，不用「不崩」充数。

## 6. 夹具与 provenance

目录 `fixtures/render-bvh/scene-a/s7b/flags/`（9 文件）：

| 文件 | bytes | sha256 |
| --- | --- | --- |
| `index.json` | 5495 | `f83f73e2f836ad26a009678c43e9b4c03747598d616a84115957e552b62bf62c` |
| `textures_off_rgb.json` | 1320472 | `428f9c8986a078ca2a94e98fbfa13a686a7efa7a5c2398eab25686e437f0a1cb` |
| `vertex_normals_off_rgb.json` | 1320487 | `6119ec42a36cd922580ae8c6f11642953893394b7ef58faa56e87dc6e8fbb20d` |
| `specular_off_rgb.json` | 1320475 | `df03e72f4e15da6dc802d0a15df0e5dcb7b7c78085b9b6bc7941bf0e1127ad5f` |
| `emission_off_rgb.json` | 1320475 | `2f07cb8a0aebc2a96edd85bb97989bdad567cb030b10ed9c4ed05a9ce4fe1019` |
| `culling_off_rgb.json` | 1320482 | `f1762d55999bc45a16b975e9b982f7d80b9b922ef5d397ada5e0f066ca0aaa67` |
| `culling_off_depth_seg.json` | 1490382 | `db802a5cb0722ec7cb3f821f0246729c069e2eaedf5b355ce58a875dbd700358` |
| `aa2_rgb.json` | 1320464 | `e773cdacca1c8d5c1c2b0973b2c97b69c5f72585c9d6f752f2813da9859c793d` |
| `aa2_depth_seg.json` | 1490542 | `79a275eff5d45f6bef764957e5f4987d0e6abfbb9b7a2cb578c8e1f717d4b7e1` |

- RGB 编码：packed u32 ABGR（a=255、`int()` 截断），row-major；解包 `float32(byte) *
  float32(1/255)`（上游 `unpack_rgb_kernel`）；判据「解包后每通道 ≤1 LSB」⇔ byte 差 ≤1。
- depth 编码：IEEE-754 binary32 位型（u32 十进制）；seg：`[object_id, object_type]` int32 对。
- depth/seg 夹具带 `recipe.compile_fast_math=false` 与 `camera_data_check`（S7b1 同 schema，
  `parse_reference` 校验索引连续与复算误差）。
- provenance（`index.json`）：upstream commit、`scene-315.mjb` / `model-312.json` sha256、
  采集脚本 sha256、生成时间、环境版本；每组 `determinism: {repeat: 2, bitwise_identical: true}`。

## 7. 环境

| 项 | 值 |
| --- | --- |
| GPU / 驱动 / SM | RTX 4070 Ti SUPER / 596.36 / SM89 |
| 产品编译 | NVRTC（CUDA 12.8.1 工具链，compute_70）、`WARP_1_15_RENDER`（fast_math=false） |
| authoring | pinned venv：Python 3.12.13、warp 1.15.0（自带 NVRTC CUDA 12.9）、mujoco/mujoco_warp 3.15.0、numpy 2.5.4 |
| GPU 窗口 | 起止见 `gpu-run.log` 与 receipt，无计算进程残留 |

## 8. 已知项与未满足项

- **S7b2 的「spp=2 必须写前拒绝」断言已被本片替代**：S7b3 实现 AA 后，`samples_per_pixel=2`
  不再是拒绝项（AA 合法配置），S7b2 的 `tests/render_kernel_rgb.rs::unsupported_configurations_
  are_rejected_before_launch` 因此失败（实际返回 `InvalidArray("ray_count")`，因为该用例喂的是
  spp=1 射线）。这是**预期语义升级**，不是回归：S7b2 的 RGB/depth/seg 基线在本次回归中全部
  0 差异；该旧断言需在 `tests/render_kernel_rgb.rs` 删除/更新（文件不在本片 files，已报 Arch）。
  本片在 `aa_supported_and_remaining_rejections_pre_launch` 覆盖新拒绝面：
  AA 已支持；`use_shadows`、`use_precomputed_rays=false`、`render_skybox` 仍写前拒绝；
  AA 射线数不符 → `InvalidArray("ray_count")` 写前拒绝。
- 阴影/天空盒/splat/flex/hfield/SDF/多相机/动态射线仍不实现（拒绝面见上）。
- AA 只验 `samples_per_pixel=2`（nsamples=4）；更高 spp 未采样参考（同一实现路径，未单独验收）。
- `vertex_normals_off` 参考在本夹具无判别力（§5），分支判别由合成法线用例承担。
- 未跑完整 G01（团队规则：开发迭代只跑定向 + fmt/clippy/clippy-cuda/unit）。

## 9. 证据文件

- `gpu-run.log`：S7b3 GPU 用例（变体/AA/基线/合成法线/拒绝面，PTX 摘要）
- `s7b1-regression.log` / `s7b2-regression.log` / `s6b-regression.log`：回归运行日志
- `capture.log`：authoring 采集输出（逐变体 sha、`determinism=two_captures_bitwise_identical` 两次捕获位等结论、vs base 统计）
- `unit-summary.log`：`cargo test --locked`
- `receipt.json`：机器可判定回执
