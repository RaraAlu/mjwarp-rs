# 渲染/BVH S6a：render context 参数/默认值/输出偏移/拒绝面（纯宿主）

- 任务：`2026-10-24-render-bvh-s6a`（depends_on `2026-10-24-render-bvh-s1` 已 verified）
- 日期：2026-10-25
- 结论：**pass**（覆盖范围内，纯宿主、无 GPU）。以 S0b 冻结 MJWarp 3.15 的
  `create_render_context` 调用参数与解析结果为参考：默认调用解析出的全部 **34** 个
  `resolved` 字段（nrender/cam_id_map/cam_res/render 掩码/rgb·depth·seg 偏移/total_rays/
  开关/background_color_unpacked/bvh_ngeom/enabled_geom_ids/znear/zfar/headlight/splat_count/
  geom_ray_types）逐项相等；`camera_paths`（projection/resolution/sensorsize/intrinsic/
  fovy/branch）与 `outputs` 的 rgb/depth/seg 形状一致；28 个公开参数的默认值与上游签名
  一致，`None` 推导、bool/name/index/空列表选择、元组/单元素/异构分辨率、输出偏移与
  缓冲大小、samples/AA 约束、splat/背景色/天空盒派生与 18 类拒绝面均有测试覆盖。
  资源所有权/生命周期留 S6b，渲染内核/AA 图像留 S7b。

## 1. 范围

覆盖（全部为 `src/render/context*` 宿主代码，无设备调用）：

- `src/render/context/params.rs`：28 个公开参数结构 `ContextParams`（字段顺序与上游签名
  一致、默认值同签名），`CamRes`/`RenderFlagSpec`/`SplatInputs` 输入形态枚举。
- `src/render/context.rs`：`resolve_context(fields, params)`：复用 S5 已验收的
  `CameraModel`/`resolve_cameras` 做相机选择/分辨率/输出开关/samples 约束，再派生
  enabled geom、灯光标志、头灯、天空盒、znear/zfar、背景色打包与相机路径；
  `ResolvedContext`（宿主侧解析结果，对应上游 RenderContext 的标量/元数据部分）。
- `src/render/context/output.rs`：`OutputLayout`：rgb/depth/seg 逐相机偏移（未启用 -1）、
  缓冲元素数/字节数、`total_rays/nsamples/ray_count/aa_accum/seg_slots`。
- `src/render/context/model.rs`：模型派生值（geom_group/type、light_type/attenuation、
  vis.headlight、tex_type/tex_width 天空盒、`vis.map.* * stat.extent` 的 znear/zfar）与
  字段缺失/类型/长度错误映射。
- `src/render/context/tests.rs`（`cfg(test)`）：调用矩阵与拒绝面单元测试。
- `tests/render_context_params.rs`：读取 S0b `context.json` 与 S1 `model-312.json` 的
  默认调用逐项对照、几何组/spp/splat/天空盒/背景色与相机/输出拒绝面。

不覆盖（显式，按 Arch 排活）：

- 设备缓冲分配、RenderContext 所有权/生命周期、BVH/mesh/texture 资源释放 → S6b。
- 渲染内核、AA 图像、渲染发射 ABI、阴影/天空盒着色 → S7a/S7b。
- 不新增夹具、不改 S0b/S1 证据、不改 S4 证据与夹具；产品测试不启动 Python。

## 2. 语义与接线（冻结上游 `71da24d`）

- **28 参数**：`ContextParams` 与上游签名一一对应；`cam_res=None`→模型分辨率、
  `render_rgb/depth/seg=None`→模型 `cam_output` 位推导、`cam_active=None`→全部相机；
  `enabled_geom_groups` 默认 `[0,1,2]`、`background_color` 默认 `(0,0,0,1)`、
  `shadow_light_fraction` 默认 `0.3`、`samples_per_pixel` 默认 1。
- **输入形态归一**：上游 `cam_res` 的 `tuple`（全相机同值，空选择时展开为空列表）、
  `list` 的「长度 1 且多相机复制」；`render_*` 的 `bool`（全相机）与 `list` 的
  「长度=模型相机数且存在筛选时按 active 重排、长度 1 复制、否则长度必须=激活相机数」。
  这些归一后交给 S5 `resolve_cameras`，由 S5 已验收实现执行。
- **输出布局**：逐相机按顺序累加 `rgb_adr/depth_adr/seg_adr`（未启用 -1），
  `total_rays=Σ像素`，`nsamples=spp²`，`ray_count=total_rays*nsamples`，
  `seg_slots=max(seg_pixels,1)`，`aa_accum_width=nsamples>1?rgb_pixels:1`；
  字节数按 `nworld` 乘 world 维（rgb u32、depth f32、seg vec2i、ray/ray_offset vec3、
  aa_accum vec3）。全部乘法用 `checked_*`，溢出映射到 `CountOverflow`。
- **znear/zfar**：上游 `float(vis.map.znear * stat.extent)`，MjModel 中 `vis.map.*` 是 f32、
  `stat.extent` 是 f64，乘积按 f64；`ResolvedContext.znear/zfar` 存 f64（与 context.json
  逐位相等）。S5 的 `CameraPlan.znear/zfar`（f32）保留给射线内核，两者语义已在注释说明。
- **几何/灯光/头灯/天空盒**：`enabled_geom_ids=np.isin(geom_group, groups)`、
  `geom_ray_types=enabled 的 raw geom_type 去重升序`、`bvh_ngeom=ids.len()`；
  `light_attenuation_is_default` 按 `np.allclose(atten,(1,0,0),rtol=1e-5,atol=1e-8)`；
  `has_spot_lights=any(light_type==mjLIGHT_SPOT)`；头灯三组 f32 直接取自
  `vis.headlight.*`；`render_skybox` 仅在请求且模型存在 `mjTEXTURE_SKYBOX` 时为 true，
  并记录 `skybox.texture_id/face_width`。
- **splat/背景色**：splat 六参数须同时提供（`position=None` 时其它必须为 None，
  否则 `SplatAttributeMismatch`；提供 position 时 rotation/scale/rgba 必须存在且长度相等，
  否则 `SplatMissing`/`SplatShape`；`group_id` 长度须=`nworld`）；`splat_count=position.len()`。
  背景色按上游 `pack_rgba_to_uint32(r*255,g*255,b*255,a*255)`（整数截断）打包，并给出
  `background_color_unpacked=[r,g,b,a]`（0..255 f64）；分量越界/非有限显式拒绝（防御性
  硬化，上游不校验）。
- **拒绝面映射**（`ContextError`）：相机选择/分辨率/输出开关/samples 错误复用
  `CameraError`（`Camera(..)` 变体，含字段名）；模型字段缺失/类型/长度 →
  `MissingField/FieldKind/LengthMismatch{field}`；splat → `SplatAttributeMismatch/
  SplatMissing/SplatShape/SplatGroupCount`；背景色 → `BackgroundColor{channel}`；
  计数 → `CountOverflow{field}`。全部在分配前返回。

## 3. 参考对照与测试

参考（只读，未新增/未修改夹具）：

- S0b `fixtures/render-bvh/scene-a/reference/context.json`（sha256 `e08cce0a…`）：
  `create_render_context_parameters`（28 参数）+ `resolved`（34 字段）+ `camera_paths` +
  `outputs.*_shape`。
- S1 `fixtures/render-bvh/scene-a/model-312.json`（sha256 `cdd490f0…`）：116 个渲染字段，
  测试逐字段还原成 `RenderFields` 后调用产品解析。

命令与结果：

```text
cargo test --locked --test render_context_params
3 passed / 0 failed
```

- 默认调用：用 context.json 的参数（`render_rgb/depth/seg=true`、
  `enabled_geom_groups=[0,1,2,3]`，其余用上游签名默认值）解析，34 个 `resolved` 字段、
  2 条 `camera_paths`、rgb/depth/seg 形状与参考逐项相等（整数/布尔精确、f32 按位宽转 f64
  相等；znear `0.024999999441206455`、zfar `125.0`、头灯 0.3/0.6/0.1 精确）。
- 调用矩阵：bool 掩码 / name / index（含反序） / 空列表 / 空列表+tuple / 异构分辨率 /
  元组 / 单元素复制；输出开关 bool 与 list（含「长度=模型相机数时按 active 重排」）；
  偏移 `rgb_adr/depth_adr/seg_adr`、缓冲字节数、`aa_accum`、`seg_slots`。
- 派生值：group `[3]`（只留 mesh geom，`geom_ray_types=[7]`）/空 group；spp=2
  (`nsamples=4`、`ray_count=4×total`、`aa_accum_width=rgb_pixels`)；splat 2 条合法输入；
  `render_skybox=true` 定位 skybox 纹理（tex_id=2、face_width=64）；自定义背景色
  `(1,0.5,0.25,0)` → packed `0x00FF7F3F`、unpacked `[255,127,63,0]`。
- 拒绝面（主机测试 + lib 单元测试）：mask 长度、未知相机名、相机 id 越界、分辨率数量、
  render flag 长度、spp=0、spp>1 需预计算射线、spp>1 需 RGB、splat 部分/缺失/形状/组长度、
  背景色越界、模型字段类型与长度错误。
- 离线无 GPU：本片所有命令为 `cargo test/check/clippy/fmt`，不启动 Python、不加载 NVRTC。

## 4. 门禁与行数与哈希（提交 blob / LF 口径）

```text
normify gate 2026-10-24-render-bvh-s6a
fmt 通过
clippy 通过
clippy-cuda 通过
unit 通过 测试531过0败
```

| 文件 | 行数 | sha256 |
| --- | --- | --- |
| `src/render/context.rs` | 415 | `d374434cf382d5ce46cedcf65a55c2ff7e66ac881e0108e60a6beb02560aeff1` |
| `src/render/context/params.rs` | 183 | `777dd6c726152f469a2995f3ba914458622d04546f47f9e3e69f1433e3faf1ba` |
| `src/render/context/output.rs` | 151 | `34f645268dfb9b255bda0320f3bc9201f5305a484fa97b047ea81bfc8fdd83d8` |
| `src/render/context/model.rs` | 213 | `789670379f1665fe163c55d52c730784b1dc36d2658a0886d666969864e2cff5` |
| `src/render/context/tests.rs` | 255 | `86b97a1ff943d5ded124796561a86a8fadcce6b59967a5f185b4245d992e8b55` |
| `src/render/mod.rs` | 363 | `ea432c3a1cccbb5a7355ab8dd10551040aff78498c54d4741fe76635baee26ea` |
| `tests/render_context_params.rs` | 409 | `2feedff8e43622cd82997a381542bbdf84014a5007f9a841757d728262c91eeb` |

新文件行数全部 ≤500（含 `cfg(test)` 的 `context/tests.rs`）；`src/render/mod.rs` 只加
`pub mod context` 与导出（`ContextError/ContextParams/ResolvedContext/resolve_context`）。

## 5. 验收映射（plan S6a 五条）

| plan S6a 验收 | 本切片 | 状态 |
| --- | --- | --- |
| ① 默认调用精确解析 28 参数（对照 context.json） | 34 个 resolved 字段 + camera_paths + 输出形状逐项相等；28 参数默认值单测 | ✅ |
| ② bool/list/name/index/空列表/异构分辨率矩阵精确解析 | 选择/分辨率/复制规则/空选择/反序/重排均有断言 | ✅ |
| ③ rgb/depth/seg 偏移与 buffer 大小与参考一致 | 默认对照 + 关闭输出/掩码/hetero 分辨率/字节数断言 | ✅ |
| ④ 每类非法组合映射到匹配错误类别与字段 | 相机 8 类 + splat 4 类 + 背景色 + 模型字段 2 类 | ✅ |
| ⑤ 不需要 GPU 渲染 | 全部主机测试；无 NVRTC/GPU/Python | ✅ |

## 6. 未满足项 / 限制

- 资源分配/所有权/生命周期（S6b 范围），本片 `ResolvedContext` 只是宿主解析结果，
  不含设备缓冲；`lower/upper/group/bvh` 等设备数组字段未在本片构造。
- 渲染内核、AA 图像、阴影/天空盒着色、渲染发射 ABI 不在本片（S7a/S7b）。
- 模型输入受 S1 `RenderFields` 字段集限制：没有 flex/hfield/splat 的模型侧数组，所以
  `bvh_nflexgeom`/hfield BVH 等派生量未覆盖；splat 只做参数校验与计数（设备侧在 S6b/S7b）。
- 调用矩阵的期望值来自上游代码语义（S0b 只捕获默认调用），未新增矩阵参考夹具；
  默认调用与 S0b context.json 逐项对照，矩阵用例是上游规则的行为断言。
- 背景色越界/非有限拒绝是产品防御性硬化，上游不校验该输入。
- 未跑全量 G01；本片按 fast-loop 口径（fmt/clippy/clippy-cuda/unit + 定向主机测试）。
