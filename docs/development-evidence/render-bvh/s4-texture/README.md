# 渲染/BVH S4：纹理转换、CUDA 纹理对象采样与注册表

- 任务：`2026-10-24-render-bvh-s4`（depends_on `2026-10-24-render-bvh-s1` 已 verified）
- 日期：2026-10-25
- 结论：**pass**（覆盖范围内）。冻结上游 MJWarp 3.15 参考对照：
  u8 `tex_data`→RGBA f32/255 转换对 3 个模型纹理逐 texel **位等**（3.12 夹具输入与 3.15
  MjModel `tex_data` 逐字节相等后再转换，SHA-256 全量摘要相等）；固定 **1040** 个 uv 查询
  （32×32 网格覆盖 [-0.125,1.125] 越界钳制 + 16 个边界/纹素中心点）对 3 个纹理的硬件
  LINEAR 采样与冻结参考 **逐位相等**（`max_abs_diff=0`，预算 ≤1e-6/通道）；`use_textures`
  开关行为与上游一致（关→注册表 0 个、开→3 个）；6 类转换/注册表拒绝面在分配前给出；
  纹理对象在采样器生命周期内有效、drop 释放后可重建（2 轮 create/sample/drop 无错误）。
  图像级 patch 延后 S7b/S8（本片无渲染内核）；天空盒/3D/mipmap 不做。

## 1. 范围

覆盖：

- `src/render/texture.rs`：MuJoCo `tex_data` u8 → RGBA f32 四通道转换（缺 g/b 补 0、缺 a 补
  255、逐通道乘 `f32(1.0/255.0)`）、纹理注册表（按模型纹理 id 顺序、`use_textures` 开关）、
  尺寸/通道/寻址/长度校验与 `TextureError`。
- `src/render/texture/cuda.rs`：CUDA 纹理对象（驱动 API：`cuArray3DCreate_v2` →
  `cuMemcpy2D_v2` → `cuTexObjectCreate`，格式 float4、LINEAR、CLAMP、归一化坐标、单 mip）、
  主上下文 retain/push/pop/release、drop 销毁对象与数组、采样内核封装（复用四缓冲
  `SynchronousKernel` ABI）。
- `src/render/texture/sample.cu`：PTX `tex.2d.v4.f32.f32` 取指内核；纹理句柄 u64 拆成两个
  meta i32 槽传递（正式 u64/texture-handle 渲染 ABI 属 S6）。
- `tests/render_texture.rs`：夹具解析、渲染字段构造与主机/GPU 验收（转换位等、通道补齐、
  注册表开关、拒绝面、采样逐位、生命周期）；`src/render/texture.rs` 内联单元测试覆盖转换
  公式与 6 类拒绝面。默认 ignore 的 GPU 用例显式执行。
- `scripts/authoring/render-bvh-s4/`：离线参考捕获（冻结上游转换/采样/注册表开关/零尺寸拒绝）。
- 夹具：`fixtures/render-bvh/scene-a/s4/textures.json`（转换摘要、uv 网格采样位型、合成
  通道补齐用例、注册表/拒绝参考）。

不覆盖（显式）：

- 渲染图像级 patch（纹理开/关的 RGB 差异）→ S7b/S8；本片不实现渲染内核。
- 天空盒着色、3D 纹理、mipmap/多级采样；纹理 UV 映射（平面/球/盒/网格 `_texture_*`）属
  渲染内核 S7b。
- 正式渲染发射 ABI（u64/scalar/texture-array 参数）→ S6；本片用四缓冲 ABI 打包句柄。

## 2. 语义与接线

- **转换**：上游 `render_util._convert_texture_data`（MJWarp 3.15，`71da24d`）：
  `tex_data[adr + (y*width + x)*nc + c]`，`r/g/b/a = float(u8) * wp.static(1.0/255.0)`，
  缺通道 g/b=0、a=255。`wp.static(1.0/255.0)` 是 double 1/255 舍入后的 f32 常量
  `0x3B80_8081`；产品用同一常量与同一乘法顺序（`f32::from_bits`），因此转换位等。
- **采样**：上游 `render_util.create_warp_texture` 建 `wp.Texture2D(data, filter_mode=LINEAR)`：
  浮点 RGBA CUDA 数组 + 纹理对象（`normalized_coords=True`、address=CLAMP、`num_mip_levels=1`），
  设备端 `wp.texture_sample` 走 `texture.h` 的 `tex2D<float4>` 硬件取指。产品用驱动 API 建
  **相同描述符**的纹理对象，内核用 PTX `tex.2d.v4.f32.f32`（PTX ISA 要求纹理对象取指带坐标
  类型后缀 `.f32`；离线 `ptxas -arch=sm_89` 校验）取指，故与参考逐位相等，不用软件双线性
  近似硬件过滤。
- **上下文**：传输会话使用设备主上下文；创建/销毁纹理对象时 `cuDevicePrimaryCtxRetain` +
  `cuCtxPushCurrent_v2`，结束后 pop + release。采样发射仍走 `SynchronousKernel`（同一会话、
  同一流；四缓冲 ABI + 4 个 u32 维度，grid 256 线程）。
- **注册表**：`use_textures=true` 时按 `ntex` 逐纹理创建（顺序=模型纹理 id）；`false` 时
  注册表为空（上游无天空盒请求时 `textures_registry` 长度 0）。`render_skybox` 占位纹理
  属天空盒范围（S7b），本片不实现。
- **范围外但同口径**：纹理对象在采样器 drop 时销毁（先 object 后 array）；采样器持有会话
  克隆，避免悬空流。

## 3. 参考捕获（authoring，GPU）

- 语义来源：冻结上游 `71da24d` 的 `render_util.py`（`_convert_texture_data`、
  `create_warp_texture`、`create_render_context`）与 `render.py` 的 `wp.texture_sample` 调用。
- 设备：RTX 4070 Ti SUPER / 驱动 596.36；warp 1.15.0、mujoco 3.15.0、mujoco_warp 3.15.0、
  numpy 2.5.4；捕获显存基线 2885 MiB、进程内 3098 MiB，跑完回 2870 MiB。
- 输入交叉校验（全部 true）：3.12 夹具 `model-312.json` 的 `tex_width/height/nchannel/adr/
  type` 与 3.15 MjModel 逐项相等；`tex_data` 89088 字节逐字节相等（sha256 `fa1b4cdf…`）。
- 转换参考：逐纹理直接 launch 上游 `_convert_texture_data`（与 `create_warp_texture` 同一
  内核/参数）捕获 RGBA f32 位型；夹具按小端 u32 串联算 SHA-256 全量摘要（`converted_sha256`），
  另存输入 u8 摘要（`data_sha256`）保证输入可追溯。
- 采样参考：`create_warp_texture(mjm, tex_id)` 建纹理对象后，用固定内核在
  `uv_grid()` 上 `wp.texture_sample(..., dtype=vec4)` 捕获 3×1040×4 个 f32 位型。
  uv 网格：32×32 轴从 -0.125 到 1.125（覆盖 CLAMP 越界）+ 16 个边界/纹素中心/越界点。
- 通道补齐：合成 u8 纹理用例 nc=1 `8x6`、nc=2 `6x5`、nc=4 `5x4`（数据 `(i*37+11)%256`），
  同样调用上游转换内核捕获位型，验证 g/b=0、a=255 补齐。
- 注册表/拒绝：`create_render_context(use_textures=False/True)` 捕获 `textures_registry`
  长度（0 / 3）与 `use_textures` 标志；零尺寸纹理用 `wp.Texture2D(np.zeros((0,8,4)))`
  捕获上游拒绝（`ValueError: Texture height must be a positive integer`）。
- 硬件 vs 软件过滤（仅记录，不作验收参考）：同转换数据在 Warp `cpu` 设备用软件路径采样，
  与 GPU 硬件路径最大绝对差 **5.96e-8**（texture 0 完全相等；texture 1/2 各 416/340 个
  通道差 1 ulp），说明本机 Ada 硬件的浮点线性过滤接近全精度；产品选择硬件路径是为了与
  冻结参考逐位一致。
- 确定性：两次完整捕获（转换+GPU 采样+CPU 采样）逐位一致，第二遍 1.034 s。
- 夹具：`fixtures/render-bvh/scene-a/s4/textures.json`（325,755 B、15,332 行，sha256
  `cbeb58ca4e854639ee500e31ccb0d9b4549d059d657d63a19d98999857b8fb16`）。文件含
  `second_pass_seconds`/显存字段，重采时哈希会因计时不同而变，但位型与摘要必须一致
  （`determinism.bitwise_identical=true` 已断言）。

## 4. GPU 验收结果（命令与原始输出）

```text
set CUDA_PATH=target\toolchains\cuda-12.8.1; PATH=%CUDA_PATH%\bin;%PATH%
cargo test --locked --features cuda-probe --test render_texture -- --ignored --test-threads=1 --nocapture
```

结果：`2 passed / 0 failed`（测试体内 0.32 s；NVRTC 首次编译在首个用例内）。

```text
texture lifecycle round 0: create/sample/drop ok
texture lifecycle round 1: create/sample/drop ok
texture 0: queries=1040 max_abs_diff=0.000e0 unequal_channels=0
texture 1: queries=1040 max_abs_diff=0.000e0 unequal_channels=0
texture 2: queries=1040 max_abs_diff=0.000e0 unequal_channels=0
```

- ① 转换：3 个模型纹理（3 通道，覆盖 a=255 补齐）+ 合成 nc=1/2/4 用例，产品转换的
  RGBA f32 位型 SHA-256 与参考相等；`width/height/nchannel` 与参考一致。
- ② 采样：3 个纹理 × 1040 个 uv × 4 通道与冻结硬件参考**逐位相等**（max abs diff 0，
  验收预算 ≤1e-6/通道）；网格含越界坐标（CLAMP）与边界点。
- ③ 注册表：`use_textures=False` 注册表为空、采样越界纹理 id 被拒绝；`True` 时 3 个纹理
  对象，按 id 顺序可采样。
- ④ 生命周期：2 轮 create/sample/drop 无错误，释放后重建仍可采样；纹理对象只在本进程
  采样器生命周期内使用。
- ⑤ 拒绝面（主机）：零宽/零高、`tex_nchannel` 0/5、负 `tex_adr`、`tex_data` 越界、
  纹理属性长度不一致、字段类型不符，全部在创建纹理对象/分配前拒绝；上游零尺寸
  `ValueError` 的 `positive` 关键词已记录在夹具。

## 5. 门禁与测试

```text
normify gate 2026-10-24-render-bvh-s4
fmt 通过
clippy 通过
clippy-cuda 通过
unit 通过 测试524过0败
```

- `cargo fmt --check`、`cargo clippy --all-targets --locked -- -D warnings`、
  `cargo clippy --all-targets --locked --features cuda-probe -- -D warnings` 均通过（0 警告）。
- 默认 `cargo test --locked`（无 cuda-probe）不含 GPU 用例；`unit` 门禁 524 过 0 败
  （S4 新增 5 个集成主机用例 + 3 个 `src/render/texture.rs` 内联单元测试）。
- 新文件行数（≤500）：`texture.rs` 486、`texture/cuda.rs` 298、`texture/sample.cu` 43、
  `tests/render_texture.rs` 410、`capture_s4_textures.py` 281、`s4_texture_support.py` 184。
- 离线 NVRTC 预检（与产品同 recipe，不占 GPU）：`sample.cu` 编译通过、PTX 1690 B、
  无 f64/f16 token、entry `sample_texture2d`、`tex.2d.v4.f32.f32` 指令存在，且
  `ptxas -arch=sm_89` 汇编通过（早期漏写坐标类型后缀 `.f32` 被此检查发现并修正）。

## 6. 产物哈希（提交 blob / LF 口径）

| 文件 | 行数 | sha256 |
| --- | --- | --- |
| `src/render/texture.rs` | 486 | `54d39a6d295d1be898e5282af9be4a9bc2bdd23211910513509d2824a73bad32` |
| `src/render/texture/cuda.rs` | 298 | `d79512ae4d446c91afd4f52f361bca6c7a37d3ea9917f1c94e3243471ceb11c7` |
| `src/render/texture/sample.cu` | 43 | `145087f2de1c7127ed3cbc36a97bfef8e3ab227927bae053c812b513583e42da` |
| `src/render/mod.rs` | 361 | `e89f28f71a1458402c37458cf45e95044e8e662b44621d9ce3784bb66c938887` |
| `tests/render_texture.rs` | 410 | `7e54955f22b94610787dfcc7891df1cc66fe7af0b5d70d904927a490f013693e` |
| `scripts/authoring/render-bvh-s4/capture_s4_textures.py` | 281 | `72f170e5622b5ae77825970142f9f015ea413ad0549afdec3bbe3c5bdfb2fe0d` |
| `scripts/authoring/render-bvh-s4/s4_texture_support.py` | 184 | `051ff743f737d0c185df7753622eb78234a379476641ea5fdd58e4de27875af9` |
| `fixtures/render-bvh/scene-a/s4/textures.json` | 15332 | `cbeb58ca4e854639ee500e31ccb0d9b4549d059d657d63a19d98999857b8fb16` |

## 7. 验收映射（plan S4 五条）

| plan S4 验收 | 本切片 | 状态 |
| --- | --- | --- |
| ① converted texels == reference (u8/255) element-wise | 3 模型纹理 + 3 合成用例，f32 位型 SHA-256 全量摘要相等 | ✅ |
| ② dimensions and channel counts == reference | 逐纹理 width/height/nchannel 相等（含 nc=1/2/4 补齐） | ✅ |
| ③ fixed uv-grid samples ≤1e-6 per channel vs reference | 3×1040×4 查询与冻结硬件参考逐位相等（max 0） | ✅ |
| ④ texture on/off render patch within RGB tolerance | **延后 S7b/S8**（本片无渲染内核）；`use_textures` 注册表开关已验证 | ⏸ deferral |
| ⑤ missing/zero-size textures rejected | 零尺寸/通道/寻址/越界/长度/类型 6 类拒绝在分配前；上游 ValueError 关键词记录 | ✅ |

## 8. 未满足项 / 限制

- 图像级 patch（纹理开/关的 RGB 差异）延后 S7b/S8：本片只验转换与采样层，不实现渲染内核。
- 天空盒着色、3D 纹理、mipmap 与纹理 UV 映射（平面/球/盒/网格）不在本片。
- 正式渲染发射 ABI（u64/texture-array/scalar 参数）属 S6；本片在四缓冲 ABI 里用 meta 两槽
  打包 u64 纹理句柄，因此采样入口是 S4 专用封装，不是最终渲染 ABI。
- 单 fixture（scene-a 3 个纹理 + 合成 nc 用例 + 1040 个 uv），无 fuzz/随机纹理/大纹理/
  各向异性过滤；不同 GPU/驱动的硬件过滤精度未验证（本机 max 5.96e-8，仅记录）。
- 上游零尺寸/缺失纹理在 MuJoCo 模型层不可构造，产品的零尺寸/长度/类型拒绝属防御性校验
  （上游仅 `Texture2D` 构造处的 ValueError 可捕获）。
- 未跑全量 G01；本片按 fast-loop 口径（fmt/clippy/clippy-cuda/unit + 定向 GPU 测试）。
