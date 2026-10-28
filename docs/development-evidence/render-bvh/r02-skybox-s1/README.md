# R02-S1：天空盒离线实现（sample_skybox + 竖排 2D 纹理注册表子集 + 槽填充）

任务：`2026-10-29-render-bvh-r02-skybox-s1`（P8-渲染与BVH，owner Backend）
本片性质：**离线实现片**。不开 GPU 窗口、不做任何设备数值执行、不出渲染数值结论；
capture/参考/判据在 S2（authoring capture）与 S3（验证窗），两者待用户在场双签后才跑。

## 1. 做了什么

1. 内核新增 `sample_skybox`（`src/render/kernel/cuda/render_shade.cu`）：把上游
   `render.py` L494-556 的方向旋转 / face 选择 / sc,tc / `safe_div` / `t` clamp /
   `v=(face+t)/6` 逐段翻译；采样复用既有 `wp::texture_sample<wp::vec4>`（同文件 L293 已在用）。
2. 内核背景分支（`render_megakernel.cu` 的 `hit.geom_id == -1` 早退块）：新增
   `MK_FLAG_SKYBOX`（bit 9）门控，采样世界方向 `ray_dir_world` 并按上游 `store_pixel`
   （L1017-1031）语义写出——AA 走 `aa_store(color)`（`+= color*255`），非 AA 走
   `pack_rgba_to_uint32(r*255,g*255,b*255,255)`；未置位时**保持 S7b2 行为逐字节一致**。
3. host 侧新增 `src/render/context/texture_inputs.rs::texture_registry`：按上游
   `render_util.py` L527-535 语义，`use_textures=true` 全量实体化；`use_textures=false`
   且有天空盒时**只实体化天空盒那一个 id，其余 id 为 1×1 全零占位**（数组下标仍等于模型
   纹理 id）；都无 → 空注册表。
4. 门与槽接线：`context/lifecycle.rs` 纹理资源门改为 `(use_textures || skybox) && !empty`；
   `megakernel_uploads.rs` 纹理数组门同义放开、`skybox_tex_id`/`skybox_face_width` 两槽
   按 `Skybox` 填值（无天空盒仍 `[0]`/`[0]`），并新增「`render_skybox` 而纹理数组为空 ⇒
   发射前拒绝」的防御；`megakernel.rs` 删除天空盒前置拒绝；`megakernel_arrays.rs` 新增
   bit 9 常量与置位。
5. `MEGAKERNEL_SLOTS` 81 槽种类与顺序**零改动**；既有 `MK_FLAG_*` bit0-8 定义**零改动**。

## 2. 上游 → 仓内行号对照

| 上游（pinned `mujoco_warp 3.15.0`） | 语义 | 仓内位置 |
| --- | --- | --- |
| `render.py` L500-502 | 竖排说明：face 顺序 +X,-X,+Y,-Y,+Z,-Z；tex_data 竖排 | `render_shade.cu` `sample_skybox` 头注释 |
| `render.py` L503-505 | `rx=x, ry=z, rz=-y`（GL3: S=x,T=z,R=-y） | 同函数首 3 行 |
| `render.py` L507-509 | `arx/ary/arz` | 同函数 |
| `render.py` L511-545 | 三段最大轴判定 + face 0..5 + sc/tc 六组符号 + `ma` | 同函数 `if/else if/else` 全块 |
| `render.py` L547-548 | `s=(safe_div(sc,ma)+1)*0.5`、`t` 同式 | 同函数两行 |
| `render.py` L550-552 | `t_min=0.5*face_width_inv`，`t` 夹 `[t_min,1-t_min]` | 同函数 clamp 两行 |
| `render.py` L554 | `v=(float(face)+t)*(1/6)`；`1/6` 为 warp 编译期常量 | 同函数末两行 |
| `render.py` L555 | `wp.texture_sample(skybox_tex, vec2(s,v), vec4)` | 同函数 `wp::texture_sample<wp::vec4>` |
| `render.py` L1259-1268 | `geom_id==-1` 且 `render_rgb` 且 `render_skybox` → 采样并 `store_pixel` | `render_megakernel.cu` 早退块新增分支 |
| `render.py` L1017-1031 | `store_pixel`：AA `+= color*255`，否则打包写 | 同分支 `aa_store` / `pack_rgba_to_uint32` |
| `render_util.py` L514-520 | skybox 纹理 id 与 `face_width = tex_width` | `context/model.rs` `skybox()`（既有）+ `uploads` 槽填充 |
| `render_util.py` L527-535 | `if use_textures or has_skybox`；其余 id 用 1×1 空纹理 | `context/texture_inputs.rs` + `lifecycle.rs` + `uploads.rs` |
| `math.py` L318-319 | `safe_div(x,y)=x/(y!=0?y:MJ_MINVAL)`，`MJ_MINVAL=mjMINVAL=1e-15` | `render_shade.cu` `safe_div_render`（既有，同义） |

## 3. 常量与取整说明

* `safe_div`：`sc/ma`、`tc/ma` 走 `x / (y != 0 ? y : 1e-15f)`（`safe_div_render`，既有实现与
  `math.py` L318-319 同义；`MJ_MINVAL = mjMINVAL = 1e-15`）。
* `1/6`：上游 `wp.static(1.0/6.0)`，double `1.0/6.0` 舍入到 f32 = `0x3E2AAAAB`；内核写
  `(1.0f / 6.0f)`（f32 除法，同样正确舍入到 `0x3E2AAAAB`），单测 `sixth_constant_matches_warp_static_value` 钉住。
* `face_width_inv`：上游 L1263 `1.0 / float(skybox_face_width[...])`（f32 除法）；内核同式。
* `t` clamp：半 texel = `0.5/face_width`，两端夹取，防止竖排相邻面在 LINEAR 过滤下串色。
* 纹理布局：MuJoCo 天空盒 `tex_height = 6 * tex_width`（实测 scene-a：`tex_width=[64,32,64]`、
  `tex_height=[64,32,384]`、`tex_type=[0,0,2]`），因此**不需要 cube 纹理路径**——既有
  `texture/cuda.rs` 的 2D `CUtexObject`（FLOAT/4ch/1mip/CLAMP/LINEAR/归一化）+ `tex.2d.v4.f32`
  就是上游 `wp.Texture2D(..., LINEAR)` 的设备等价物。

## 4. flag 位表（`shading_words` 低 32 位）

| bit | 内核常量 | 来源（`ResolvedContext`） |
| --- | --- | --- |
| 0 | `MK_FLAG_TEXTURES` | `use_textures` |
| 1 | `MK_FLAG_VERTEX_NORMALS` | `enable_vertex_normals` |
| 2 | `MK_FLAG_SPECULAR` | `enable_specular` |
| 3 | `MK_FLAG_EMISSION` | `enable_emission` |
| 4 | `MK_FLAG_PER_LIGHT_AMBIENT` | `enable_per_light_ambient` |
| 5 | `MK_FLAG_AMBIENT_LIGHTING` | `use_ambient_lighting` |
| 6 | `MK_FLAG_HEADLIGHT` | `headlight.active` |
| 7 | `MK_FLAG_CULL_BACKFACES` | `enable_backface_culling` |
| 8 | `MK_FLAG_AA` | `nsamples > 1` |
| 9 | `MK_FLAG_SKYBOX`（本片新增） | `render_skybox` |
| 10-31 | 空闲 | — |

高位 32 bit 为 `background_color_packed`。新位只追加到未占用位，且与内核 `#define` 同值
（`tests/render_skybox.rs::kernel_flag_bits_and_slot_signature_unchanged` 逐条钉住既有 9 个
define 与新增 1 个 define，并断言 `MEGAKERNEL_SLOTS` 仍 81 槽 = 1 bounds + 7 标量）。

## 5. host 语义

* 注册表子集（`texture_inputs.rs`）：`use_textures=true` → 全量；`false` + 天空盒 → 仅
  天空盒 id 真实纹理 + 其余 id 1×1 全零占位；都无 → 空。天空盒 `face_width` 必须等于
  `tex_width[skybox_id]`，否则报 `LengthMismatch{field:"skybox_face_width"}`；id 越界报
  `LengthMismatch{field:"skybox_texture_id"}`。
* 纹理资源门（`lifecycle.rs`）：`wants_textures = use_textures || skybox.is_some()`，
  仍要求注册表非空。
* 槽与发射（`uploads.rs`/`megakernel.rs`）：纹理数组门同义放开；两槽有天空盒时填
  `[texture_id]`/`[face_width]`，无天空盒保持 `[0]`/`[0]`（槽只在 bit9 置位时被读）；
  `render_skybox=true` 而纹理数组为 `None` 时发射前拒绝（防设备侧用占位句柄取样）。
* 无天空盒场景：注册表空、纹理数组 `None`、两槽 `[0]`/`[0]`、bit9=0 —— 与 S7b2 完全一致。

## 6. 零泄漏静态论证（本片无 GPU 执行；位级回归证据留给 S3 验证窗）

1. **控制流不可达**：新增代码只在 `hit.geom_id == -1` 的早退块内、且第一层条件
   `(flag_bits & MK_FLAG_SKYBOX) != 0u` 为真时执行；bit9 来源 `resolved.render_skybox`
   （`shading_words`），无天空盒场景为 `false` ⇒ 落入原 `else if (aa) / else` 两支，语句
   与 S7b2 逐字节相同；`sample_skybox` 只在 `if` 内被调用，`ray_dir_world` 已是既有量。
2. **输入面不变**：无天空盒时 `skybox_tex_id`/`skybox_face_width` 仍 `[0]`/`[0]`（`uploads.rs`
   的 `match resolved.skybox { None => ([0i32],[0i32]) }`）、`texture_array` 仍 `None`
   （门含 `|| resolved.render_skybox`）、注册表仍空（`texture_registry` 在 `use_textures=false`
   且 `skybox=None` 时返回空）、`lifecycle` 门结果为 `None`。
3. **PTX 差异**（详见 `materials/ptx-diff-summary.txt`）：kernel source `72bc364c…`→
   `e68e3412…`、ptx `97467d83…`→`0b947492…`、`ptx_bytes 264015→268275`（+4260）；
   语义层增量恰好是：`tex.level.2d` 取样 +1（天空盒）、`and.b32 %r,%r,512`（bit9 判定）+1、
   `and.b32 %r,%r,256`（bit8 = AA 位判定）2→3（+1，新分支内 `if (aa)` 的位复检）、
   `max.f32` +1 / `min.f32` +1（`t` 两端夹取）、`sqrt` 不变；**`and.b32` 总量 43→45（+2）**。
   行级 diff 面较大（805 行内 +135 行、Compare-Object 4810/4675）来自 NVRTC 插入代码后的
   寄存器/标签重编号，已在材料中说明。（本节于 2026-10-29 按 QA O1 补正 256 位一行。）
4. **无设备执行**：本片未跑任何 `--ignored` 设备用例（只跑 NVRTC 纯编译门）、未开窗口、
   未产生任何渲染数值结论；**零泄漏的位级回归证据待 S3 验证窗**（8 个既有回归实体一并覆盖）。

## 7. 单测（纯 host，无 CUDA）

`tests/render_skybox.rs`（10 个）+ `src/render/context/texture_inputs.rs`（1 个）：

1. `skybox_only_registry_materializes_skybox_and_dummy_textures`：len=ntex、天空盒 64×384、
   其余 id 1×1 全零、整体 texels 计数；
2. `use_textures_materializes_all_ids_unchanged`：全量实体化与是否请求天空盒无关；
3. `no_skybox_and_no_textures_yields_empty_registry`；
4. `skybox_face_width_must_match_tex_width`；
5. `skybox_texture_id_must_be_in_range`；
6. `kernel_flag_bits_and_slot_signature_unchanged`：10 个 `MK_FLAG_*` define 逐条存在且
   `MEGAKERNEL_SLOTS` 仍 81 槽（1 bounds + 7 标量）；
7. `axis_directions_map_to_the_six_faces`：六轴→六面 + 中心 `s=0.5`、`v=(face+0.5)/6`；
8. `sixth_constant_matches_warp_static_value`：`(1.0f/6.0f).to_bits()==0x3E2AAAAB`；
9. `face_boundary_and_t_clamp`：`arx==ary` 取 +X、`t=0/1` 两端夹到半 texel、
   `face_width=1` 夹到 0.5、`1024` 夹到 `1-1/2048`（边界像素不串面）；
10. `zero_direction_is_not_nan`：零方向 `ma=0` 走 `1e-15` 除数不产生 NaN；
11. `neither_textures_nor_skybox_yields_empty_registry`（impl 文件内）。

## 8. 门禁与行数

见 `materials/gates.txt`（exit code 与数字）与 `materials/line-counts.txt`（逐文件 LF 计数与
卡面上界对照；`lifecycle.rs` 净 +2 ≤  +2、`megakernel_arrays.rs` 332 ≤ 340、新实现文件 157、
新测试文件 260；红线文件 `texture.rs`/`cast.rs`/`render.cu`/`launch.rs`/`model/render.rs` 零改动）。

## 9. 模块源码登记与影响面口径

* `normify module source mjwarp.render --add src/render/context/texture_inputs.rs`：源码 39 → **40**（API 11 不变）。
* 文件级 impact 对 `src/render/kernel/**` 不可用（多个 `.rs`/`.cu` 未登记为模块源，既有治理
  缺口 `2026-10-28-arch-volume-module-sources`）；本片**只补了新文件的模块源登记**，kernel
  那 6 个 `.rs` 仍未登记，该 backlog 现状不因本片改变。替代口径 = 模块级
  `normify impact mjwarp.render`（命中模块、依赖与约束一致）+ 卡 scope + `validate --fingerprints`。

## 10. 限制与非声明

* 本片**无 GPU 执行**：设备侧 `sample_skybox` 与本片新增分支的像素级行为**未经数值验证**；
  S2/S3 窗口材料（capture、参考、判据比较、两次运行 sha）待用户在场双签后才产生。
* 设备数值比对口径沿用既有位级口径（RGB `mismatched=0/max_byte_diff=0/beyond_1_lsb=0`、
  depth `mismatch=0/max_ulp=0/beyond_ulp=0`、seg `mismatch=0`、两次运行 sha 相同）；
  本片未改判据/容差/参考/fixture。
* 天空盒方向→面的语义正确性以 S3 位级比对为最终依据；本片单测只覆盖 host 参考实现与
  host 注册表/槽/flag 接线，不等价于设备正确性。
* `tests/render_skybox.rs` 现盘 **260 LF = 卡面上界（≤260），零余量**（QA O4）：后续任何追加
  都会破行数棘轮，扩展前必须先拆文件或按流程调整上界并报批。
* 设备侧 `sample_skybox` 的六轴方向、face 边界与零方向退化**无执行级验证**（QA O5）；建议
  S2/S3 首窗把这三类纳入 capture 检查点（本证据只声明 host 侧单测覆盖）。
* 新增的 `pub fn texture_registry` 是新的公开 Rust 项（QA O2）；「API 11 不变」指 module
  `[[apis]]` 的 planned-rpc 合同口径，不含 Rust `pub` 项。
