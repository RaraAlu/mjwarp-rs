# 渲染/BVH S7b3 独立复核（flag 变体矩阵 / AA 图像级 / 合成顶点法线 / S7b2 断言升级）

任务：`2026-10-25-render-bvh-s7b3-independent-check`（QA 独立复核，渲染/BVH S7b3）
复核对象：`52c22b7`（S7b3 主体，父 `624948b`）；配套修正 `3ab52cf`（同步 S7b2 拒绝面断言）
证据基线 HEAD：`3ab52cf`（本复核开始时；复核对象未变）
结论：**验收面全部成立，0 阻断**；2 条证据/注释口径问题（建议修）、若干观察项。

> 口径：本复核**未跑完整 G01**；不宣称完整渲染/R01 通过；不宣称 `vertex_normals` 合成法线
> 用例与上游参考一致（该用例只证明分支生效，无 pinned 参考）。本目录只含 QA 证据，未修改
> 产品代码 / 交付夹具 / 原交付证据 / 容差。

## 1. 方法（不读交付断言）

- **独立 harness**（`qa-harness-main.rs.txt`，`target/qa-s7b3-harness/`，产品 crate path 依赖 +
  `cuda-probe`）：自带模型/场景/相机位姿解码，不 import `tests/support/**`；按变体构造
  `ContextParams`、自建 81 槽内核与 AA 内核，落盘 RGB/depth/seg **原始位**、AA 射线缓冲与 PTX。
- **独立参考重采**（`capture_qa.py`，pinned venv Python 3.12.13 / warp 1.15.0 / MuJoCo 3.15.0 /
  mujoco_warp 3.15.0 / numpy 2.5.4；`PARAMS_28` 仅变体 override、`use_fast_math=False`）：QA 自己
  跑 `forward + create_render_context + render`，读 `rc.rgb_data/depth_data/seg_data` 与
  `d.cam_xpos/cam_xmat` 位；每变体两次渲染逐位一致。
- **独立判定**：`judge_variants.py`（逐变体 RGB/depth/seg）、`judge_rays.py`（AA 射线布局反推）、
  `judge_aa.py`（AA 语义：首样本 depth/seg + 平均 RGB）、`judge_synthetic.py`（合成法线判别）、
  `judge_flags.py`（夹具/hash/index 记录）、`judge_evidence.py`（交付证据/范围/配套修正）。

## 2. 核心数字（QA 独立重算/重采）

### 2.1 逐变体独立参考（每组自己重采、自己解码比较）

| 变体 | override | 产品 vs **QA 自采**参考 | 与 base 差异（QA 复算） |
| --- | --- | --- | --- |
| `textures_off` | `use_textures=false` | 两相机 RGB **120,000/120,000 位等** | cam1 **4,130** 像素；按 base seg 几何 `{1:2202, 2:747, 3:754, 5:427}`（均有 mat_texid），无纹理几何 0 |
| `vertex_normals_off` | `enable_vertex_normals=false` | 两相机 RGB **位等** | **0**（本夹具参考自身无判别力，见 §3） |
| `specular_off` | `enable_specular=false` | 两相机 RGB **位等** | cam0 **55,504**、cam1 **2,457** |
| `emission_off` | `enable_emission=false` | 两相机 RGB **位等** | cam1 **1,947**（geom1 1,424 + geom3 523） |
| `culling_off` | `enable_backface_culling=false` | RGB 位等；depth **0 ULP**；seg **0** | cam1 RGB **38**、depth **38**、seg **29**（背面命中） |
| `aa2` | `samples_per_pixel=2`（nsamples=4） | RGB 位等；depth **0 ULP**；seg **0** | cam0 RGB 534 / cam1 1,320；depth 59,896 / 15,739；seg 91 / 210 |

- 判据为「解包后每通道 ≤1 LSB / depth ≤4 ULP / seg 全等」；产品实际为 **0 差异、0 ULP、0 不一致**。
- QA 自采的 7 组参考（base + 6 变体）与交付夹具**逐位相同**（RGB、depth、seg；culling_off/aa2 的
  depth/seg 亦同），两次渲染逐位一致；`vertex_normals` 开/关两版参考在 scene-a 上**逐位相同**。
- 与 base 的差异统计与 `index.json::stats_vs_base` 的**总量**逐项一致；逐 geom 归因方法不同
  （交付按变体/AA 的 seg 归因、QA 按 base seg 归因），属方法差异不是矛盾。

### 2.2 AA 语义（QA 用「每样本单独渲染 + AA 渲染」独立判定）

AA 布局：`nsamples=4`、`total_rays=120,000`、`ray_count=480,000`、`aa_accum_width=120,000`。

| 判定 | 结果 |
| --- | --- |
| depth 单样本 = **第一个** pass | `depth_aa == depth_sample0` **逐位成立**；`depth_aa != depth_sample3` |
| seg 单样本 = **第一个** pass | `seg_aa == seg_sample0` **逐位成立**；`seg_aa != seg_sample3` |
| 判别力（首样本 vs 次样本确有差异） | depth **71,024** 像素、seg **283** 像素不同（`depth_aa` 与 sample1 差 71,024，与 sample0 差 0） |
| RGB = 四样本平均（`color*255` 累积 → ×1/4 → clamp → int 截断） | 按「每通道 \|aa_byte − floor(mean(sample bytes))\| ≤ 1」：**0 violation**（max 偏差 1），alpha 全 255 |
| 两次渲染逐位相同 | 是（QA harness 进程内两帧 RGB/depth/seg 全等） |

上游语义核对（pinned 3.15 `render.py`）：`store_pixel` 的 AA 分支 `aa_accum_out[...] += color * 255.0`；
`_aa_resolve` 的 `c = accum * inv_n` + clamp + pack；depth/seg 写入条件为
`ray_base == 0 and render_depth/seg[camid]`（源码原文「Depth and seg are single-sample outputs:
only the first pass writes them」）。产品 `render_megakernel.cu` 的 AA 分支与 `ray_base == 0`
条件与该语义一致（实现本身已读代码核对）。

### 2.3 AA 射线布局（独立反推，不依赖 warp 超越函数）

从产品射线方向反解 `x = -znear·dir.x/dir.z` 并逆映射相机坐标，恢复每条射线的 `(px, py)`，
再与 QA 自己按 `build_rays.cu`/S5 契约推导的映射比较：

- base（spp=1）：`[camera][row][column]`，`px=x, py=y`，img=(w,h) → **0 bad**（2 相机共 120,000 条）；
- aa2（spp=2）：`[sample][camera][row][column]`，`px=x·2+sample%2`、`py=y·2+sample//2`，
  img=(2w,2h) → **0 bad**（2 相机 × 4 样本共 480,000 条）；偏移全 0（透视）。

### 2.4 合成顶点法线（证据强度：仅分支可达性）

- 把 4 个顶点法线全部替换为 6 个极端方向，开/关 `enable_vertex_normals` 两版比较：
  `[1,0,0]` **653** 像素不同、`[0,0,1]` **9** 像素不同，其余 4 个方向 **0**；
- 与交付记录（653/9/0）**逐项一致**（QA 独立复现）；
- **限制**：该用例**没有 pinned 参考**，只能证明 `vertex_normal` 分支被走到并影响着色，不能
  写成「与上游一致」；scene-a 参考本身开/关逐位相同（QA 重采确认），该变体在本夹具无判别力。

### 2.5 S7b2 旧断言：语义升级而非回归（独立核实）

- `samples_per_pixel=2` 在 `52c22b7` 已是**合法配置**：QA 用 spp=2 context + 480,000 条 AA 射线
  渲染**成功**（`reject.json::aa2_supported`）；
- S7b2 旧用例喂的是 **spp=1 的 120,000 条射线**，AA context 下返回
  `InvalidArray("ray_count")`（QA 独立复现 `aa2_short_rays`，与交付 `s7b2-regression.log` 的失败
  原因一致）——是「旧断言与新合法配置冲突」，不是产品渲染回归；
- S7b2 基线在 `52c22b7` 与当前 HEAD 均保持：RGB 两相机 0 差异、depth 0 ULP、seg 0（QA 变体
  base 对比 + 交付用例重跑 3/3）；
- 配套修正 `3ab52cf`：非 `.normify` 变更**只有** `tests/render_kernel_rgb.rs`，从拒绝列表移除
  spp=2 并注明 AA 归 S7b3，其余 3 类断言不变；带 `Normify-Fixes: 2026-10-25-render-bvh-s7b2-shading`。

## 3. 其余复核面

- **A′ 门禁**：megakernel PTX **197,583 B**（`source_sha256=e0fd7ed1…` = 五段内核源 76,256 B
  拼接的 sha256，QA 独立重算一致；`ptx_sha256=582b2429…`，entry 含 `scene_bvh_ray` +
  `render_megakernel`）；`aa_resolve` PTX **27,536 B**（`source_sha256=0bf8af5a…` = `aa_resolve.cu`
  2,364 B 的 sha256；`ptx_sha256=b28e155c…`；entry `aa_resolve`）。两者 f64 仅 W15 白名单 2 条
  （aa_resolve 同源 include builtin.h）+ megakernel 的 1 组 sin/cos 归约三元组。
- **拒绝面**：AA 支持；`use_shadows` / `use_precomputed_rays=false` / `render_skybox` 仍由
  renderer 返回 `Unsupported`（建 context 成功后、无 panic）；AA 射线数不符 → `InvalidArray("ray_count")`。
- **门禁重跑**：`cargo test --locked` **99 个测试二进制 532/0**；fmt、clippy（cuda-probe）通过；
  交付 GPU 用例 S7b3 **6/6**、S7b1 **6/6**、S7b2 **3/3**（配套修正后）、S6b **5/5**。
- **交付证据**：8 个证据文件与 `52c22b7` git blob **逐字节相同**；PTX/宿主模块 bytes+sha256、
  `launch.rs` **500 行**、`scene.cu` `8dad4d61…` 未变、`git diff 624948b..52c22b7` **33 文件全在
  范围**（out_of_scope 空）、S2–S6 证据夹具与 `src/render/bvh/**` 未改、`tests/render_kernel_rgb.rs`
  **不在本片**（只在配套修正里改）。
- **夹具**：`flags/` 9 文件 bytes/sha256 与 receipt **全对**；RGB 全 `a=255`、长度 = w·h；depth/seg
  长度与 QA 重采逐位一致；collect script sha **7ecfa0aa…** 与 receipt 一致；`index.json` 6 变体
  齐全、determinism `{repeat:2, bitwise:true}`、`has_discriminating_effect` 与 QA 实测一致
  （`vertex_normals_off=false`，其余 true）。

## 4. GPU 窗口

- 设备 RTX 4070 Ti SUPER / 596.36 / CUDA 13.2 / 16 GiB；窗口开始时无 CUDA 计算进程。
- 显存：Arch 交接基线 `3449 MiB`；QA 首次测量 `3538 MiB`；结束 **`4987 MiB`** 且
  `nvidia-smi --query-compute-apps` **无 CUDA 计算进程**（QA 全部进程已退出；期间桌面图形应用
  （浏览器/Electron）占据的显存上移，与 QA 分配无关）。
- 实际 GPU 活动：7 次参考重采（首次变体 kernel 编译 ~2.9 s，其余缓存命中）+ 7 变体 × 2 帧
  + 每样本 4 帧 + AA 1 帧 + 合成法线 12 帧 + 拒绝面若干帧 + 4 套交付用例（10.48+4.15+3.07+7.90 s），
  合计约 2–3 分钟量级；30 分钟窗口内、无冲突、未超窗口占用。

## 5. 问题清单

### 一般（证据/注释口径，不影响验收结论）

1. **receipt `host_modules.lines` 与文件实际行数不符**：`megakernel.rs` 记 474（实际 **495**）、
   `megakernel_aa.rs` 记 86（实际 **93**）、`megakernel_arrays.rs` 记 216（实际 **233**）；
   三者的 `bytes` 与 `sha256` 与文件**完全一致**（说明只是行数字段陈旧/口径错）。实际行数均
   < 500，≤500 棘轮结论不受影响。建议 `normify fix` 修正 lines 字段（或统一行数口径）。
2. **`megakernel.rs` 模块头注释与实现相反**：文件头写「depth/seg 每个 sample 都写（与上游一致，
   最后一个 sample 生效）」，而实现与上游均为 `ray_base == 0` 只写**第一个** pass（这正是首轮
   AA depth 78% 不匹配的根因修复）。注释残留自首版设计，建议改为单样本/首 pass 语义。

### 建议

3. `capture.log` 未打印「两次捕获逐位一致」的结论（determinism 只写在 `index.json` 里）；
   QA 自己的重采两次逐位一致已独立确认，建议日志同时打印，便于证据链自洽。
4. AA 的 `rgb_changed_by_geom` 在交付 `index.json` 以变体/AA seg 归因（含背景 `-1`），QA 以 base
   seg 归因，两者总量一致、逐 geom 分布不同；建议证据注明归因口径，避免后续误读。

### 如实记录的证据强度限制

5. `vertex_normals` 合成法线用例（653/9 像素）**只证明分支生效**，无 pinned 参考对照，不得写成
   「与上游一致」；scene-a 参考本身开/关逐位相同，该变体在本夹具无判别力（已在交付与 QA 证据
   中如实记录）。
6. AA 只验 `samples_per_pixel=2`（nsamples=4）；更高 spp 未采样参考（同一实现路径未单独验收）。

## 6. 未覆盖 / 边界（不宣称）

- 未跑完整 G01；不宣称完整渲染/R01/双平台。
- 阴影、天空盒、splat、flex/hfield/SDF、多相机、动态射线仍未实现（写前拒绝）。
- 正交相机、更高 spp、带 texcoord 网格未覆盖。
- `bvh_id` 为自有打包，仅签名对齐，不宣称 Warp BVH 等价。
- 本片只做图像级 AA，不宣称 R01 整体通过。

## 7. 证据文件

- `receipt.json`：机器可判定回执；`qa-artifact-manifest.json`：QA 运行产物（target/）哈希绑定；
  `artifact-check.json`：提交证据与交付证据的哈希清单。
- `judge_variants.json` / `judge_rays.json` / `judge_aa.json` / `judge_synthetic.json` /
  `judge_flags_fixtures.json` / `judge_evidence.json`：各项独立判定结果。
- `harness_*.json`（7 变体）、`reject.json`、`sample_split.json`、`synthetic_normal.json`。
- `qa-gpu-runs.log`（4 套交付用例原样重跑）、`qa-unit-tests.log`（99 二进制 532/0）。
- 脚本：`capture_qa.py`、`judge_*.py`、`qa-harness-main.rs.txt`。
