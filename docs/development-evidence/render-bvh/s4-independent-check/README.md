# 渲染/BVH S4 独立复核（u8→f32 转换 / 3 纹理×1040 uv 硬件采样 / 注册表 / 拒绝面）

- 任务：`2026-10-24-render-bvh-s4-independent-check`（QA；Arch 30 分钟 GPU 独占窗口，实测 GPU 约 1 分钟）
- 复核对象：`3e3426d`（S4 功能提交，任务 `2026-10-24-render-bvh-s4`），复核基线 HEAD `9736f90`
- 日期：2026-10-25
- 结论：**验收面全部成立（0 阻断）**。转换位等（QA 独立重算 u8×f32(1/255) 公式与产品/夹具三方一致）、3 纹理×1040 uv×4 通道硬件采样**逐位相等**（max_abs_diff=0）、`use_textures` 开关/注册表/生命周期成立、8 个拒绝用例（6 类）全部在创建前拒绝、524/0；夹具 325,755 B 真实数据。
- **1 个非阻断证据问题（建议 `normify fix`）**：交付回执/README 里 `src/render/texture.rs` 记录的 sha256 与提交 blob 不一致（见 §6）。

## 1. 复核方式（独立重算，不读交付测试断言）

| 检查 | QA 独立做法 |
| --- | --- |
| 转换 | QA harness（`target/qa-s4-harness`，产品 crate path 依赖、不导入交付测试/支持模块）从夹具元数据 + `model-312.json` 的 89,088 B `tex_data` 重建纹理，导出 RGBA f32 位型；**QA Python 用独立公式**（f32 常量 `0x3B808081`、缺 g/b=0、缺 a=1.0）重算 SHA-256 与合成用例逐纹素位型，三方（产品/QA 重算/夹具）比对 |
| 采样 | 产品 `TextureObjects` 建 3 个硬件纹理对象，对夹具 1040 个 uv 采样，导出原始 RGBA f32 位型；QA Python 逐通道位型比对与 max_abs_diff 计算；uv 网格越界点计数（CLAMP 覆盖） |
| 注册表/生命周期 | `use_textures` off/on 的注册表长度/纹理元数据/texels 总数、off 时越界 id 采样被拒、2 轮 create/sample/drop |
| 拒绝面 | 8 个字段构造用例覆盖 6 类（零宽/零高、nchannel 0/5、负 adr、data 越界、属性长度不符、字段 kind 不符），全部要求创建前 Err |
| 证据 | 8 个产物从 git blob 重算 sha256/行数；夹具字节/行数/跨查；`src/runtime` 自 S2b 起未改 |
| PTX/DESC | 产品编译路径捕获 PTX：`tex.2d.v4.f32.f32` 计数、f64 token、source sha 与 `sample.cu` blob；DESC 构造逐字段复核（显式字段赋值，不用 zeroed 枚举） |

## 2. 逐项结果

### ① u8→f32/255 RGBA 转换（成立）

- 3 个模型纹理（nc=3，覆盖 a=1.0 补齐）：产品导出位型 SHA-256 == **QA 独立重算** == 夹具 `converted_sha256`（3/3）；`width/height/channels/texel_count` 与夹具一致（4096/1024/24576 texels，合计 29,696）；
- 合成 nc=1 `8x6`、nc=2 `6x5`、nc=4 `5x4`：产品逐纹素 RGBA 位型 == 夹具 `converted_bits` == QA 重算（3/3），g/b=0、a=1.0 补齐成立；
- 输入 `tex_data` 89,088 B，sha256 `fa1b4cdf…` 与夹具 `cross_check_312.tex_data_sha256`、`tex_data_bytes=89088` 一致。

### ② 3 纹理 × 1040 uv × 4 通道硬件采样逐位（成立）

| 纹理 | 查询 | 不等通道 | max_abs_diff |
| --- | --- | --- | --- |
| tex 0 | 1040 | 0 | 0.0 |
| tex 1 | 1040 | 0 | 0.0 |
| tex 2 | 1040 | 0 | 0.0 |

- 1040 个 uv 中 **350 个越界**（[-0.125,1.125] 网格 + 边界/纹素中心点），覆盖 CLAMP 行为；
- 夹具记录的硬件 vs Warp CPU 软件路径差 `5.960464477539063e-08`（≤1e-6），仅作记录、不作验收参考；产品走硬件路径与冻结参考逐位一致，**不是软件双线性近似**。

### ③ use_textures / 注册表 / 生命周期（成立）

- `use_textures=false`：注册表长度 0、越界纹理 id 采样被拒（`TextureObjects::len=0` 时采样 Err）；
- `use_textures=true`：注册表 3 个纹理、顺序=模型纹理 id，texels 总数 **118,784 f32 分量**（= 29,696×4）；
- 2 轮 `create/sample/drop` 无错误，释放后可重建并继续采样；drop 先 `cuTexObjectDestroy` 后 `cuArrayDestroy`，上下文 `cuDevicePrimaryCtxRetain` + `cuCtxPushCurrent_v2`/`cuCtxPopCurrent_v2`/`cuDevicePrimaryCtxRelease_v2` 成对出现。

### ④ 6 类拒绝面（成立）

QA 用产品公开 `RenderFields` + `TextureRegistry::from_fields` 构造 8 个用例，全部在创建纹理对象/分配前拒绝：

- 零宽、零高（`ZeroSize`）、nchannel 0/5（`InvalidChannels`）、负 `tex_adr`（`NegativeAddress`）、`tex_data` 越界（`DataRange`）、属性长度不符（`LengthMismatch`）、字段 kind 不符（`FieldKind`）；
- 上游零尺寸 `wp.Texture2D` 的 `ValueError`（关键词 `positive`）在夹具 `rejections.zero_size` 中记录，产品为写前显式拒绝（防御性硬化，README 已声明）。

### ⑤ PTX tex 坐标后缀与 DESC 构造坑（成立）

- 产品编译路径 PTX：entry `sample_texture2d`，**`tex.2d.v4.f32.f32` 1 条**（带坐标类型后缀 `.f32`），`f64` token **0**，PTX 1,689 B；
- `source_sha256 = 145087f2…` 与提交的 `src/render/texture/sample.cu` blob 逐字一致（编译的确实是提交源码）；
- DESC 构造逐字段核对（全部显式赋值，不用 `std::mem::zeroed()` 的枚举零值，避免 cudarc bindgen 枚举零值不合法导致的 debug panic）：
  - `CUDA_ARRAY3D_DESCRIPTOR{Width, Height, Depth=0, Format=CU_AD_FORMAT_FLOAT, NumChannels=4, Flags=0}`，pitch = `width*4*4` 字节；
  - `CUDA_RESOURCE_DESC{resType=CU_RESOURCE_TYPE_ARRAY, array.hArray, flags=0}`；
  - `CUDA_TEXTURE_DESC{addressMode=CLAMP×3, filterMode=LINEAR, flags=CU_TRSF_NORMALIZED_COORDINATES, maxAnisotropy=0, mipmapFilterMode=LINEAR, mipmapLevelBias=0, min/maxMipmapLevelClamp=0, borderColor=0, reserved=0}`；
  - 创建/拷贝/对象创建走 `cuArray3DCreate_v2` → `cuMemcpy2D_v2` → `cuTexObjectCreate`，与冻结上游 `wp.Texture2D(filter=LINEAR)` 描述符一致；交接 ABI 用 meta 两个 i32 槽拼 u64 句柄（正式渲染 ABI 属 S6，已在 README 声明）。

### ⑥ 证据 / 门禁（成立，除 §6 哈希问题）

- 夹具 `fixtures/render-bvh/scene-a/s4/textures.json`：**325,755 B、15,332 行、sha256 `cbeb58ca…`**，与回执/git blob 一致；3 纹理 + 3 合成 + 1040 uv + 注册表/拒绝项，真实位型非占位（跨查 3/3 + 采样 12,480 通道位等）；
- `src/runtime` 自 S2b（`c7b5d7f`）起未改（`git diff --name-only c7b5d7f..HEAD -- src/runtime` 为空）；
- `cargo test --locked`：**524 passed / 0 failed**；`cargo test --locked --test render_texture` 主机用例 **5 passed / 0 failed**；
- 交付 GPU 测试原样重跑：**2 passed / 0 failed**（3 个纹理各 `queries=1040 max_abs_diff=0.000e0 unequal_channels=0`，2 轮生命周期）；
- 新代码文件全部 <500 行（486/298/43/410/281/184；`src/render/mod.rs` 361 为既有文件）。

## 3. GPU 窗口

- 开始：`2899 MiB`（桌面），驱动 596.36 / CUDA 13.2 / RTX 4070 Ti SUPER；无其他满载任务。
- harness 约 `2880 MiB`；交付 GPU 测试 `2878 → 2871 MiB`。
- 结束：`2871 MiB`，无本任务 CUDA 计算进程，已回桌面基线；实际 GPU 用时约 1 分钟。

## 4. 未满足项 / 限制

- 图像级 patch（纹理开/关的 RGB 差异）延后 S7b/S8（本片无渲染内核）；不宣称图像/AA 验收；
- 天空盒着色、3D 纹理、mipmap、纹理 UV 映射不在本片；正式渲染发射 ABI（u64/texture-array/scalar）属 S6，本片在四缓冲 ABI 里用 meta 两槽打包句柄；
- 单 fixture（3 纹理 + 合成 nc + 1040 uv），无 fuzz/大纹理/各项异性过滤；不同 GPU/驱动的硬件过滤精度未验证（本机 5.96e-8 仅记录）；
- 零尺寸/长度/类型拒绝是产品防御性校验（上游 MuJoCo 模型层不可构造，只有 `wp.Texture2D` 构造处 ValueError 可捕获）；
- 未跑全量 G01（fast-loop 口径）。

## 5. 复核产物

- `harness-compare.json`：转换三方比对、合成用例、3×1040×4 采样位等、注册表/生命周期/8 拒绝、夹具信息
- `static-check.json`：8 产物 blob 哈希/行数、夹具跨查、runtime diff、PTX 摘要
- `product-gpu-test.log`、`ptx-record.json`、`ptx-fma-summary.txt`、`unit-tests-summary.txt`、`gpu-window.txt`

## 6. 非阻断证据问题（建议 `normify fix` 修正，不改验收结论）

**`src/render/texture.rs` 的哈希登记与提交 blob 不一致**：

- 交付回执与 README 记录：`sha256=54d39a6d295d1be898e5282af9be4a9bc2bdd23211910513509d2824a73bad32`，486 行；
- 实际 `3e3426d` 与当前 HEAD 的 git blob / 工作树：**17,275 B、486 行、sha256 `ec21970331de9f4cfb9d42ba8aea5563518379c61fbcd79656d4a6218e292b88`**（三种口径 raw/LF/blob 全一致，排除 CRLF 口径问题）；
- 其余 7 个产物哈希/行数全部与 blob 一致；行为证据（转换/采样/拒绝）不受影响；
- 建议：像 S1 `field-diff.json` 行数那样用 `normify fix` 修正 README/receipt 的该行哈希（S4 任务已 verified，属证据标注修正，不重跑、不影响结论）。
