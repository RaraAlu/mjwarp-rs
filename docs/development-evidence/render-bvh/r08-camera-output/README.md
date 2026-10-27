# 渲染/BVH R08-S1：逐相机输出视图（get_rgb / get_depth / get_segmentation 语义，宿主侧）

- 任务：`2026-10-28-render-bvh-r08-camera-output`（R08 异构多相机首片 S1；depends_on S8 已 verified）
- 日期：2026-10-28
- 结论（本片范围）：在既有整缓冲 + `OutputLayout` 上新增**宿主侧逐相机输出视图**，逐位复刻冻结上游
  `render_util.py`（`71da24d`）的 `unpack_rgb_kernel`/`extract_depth_kernel`/`_extract_seg_kernel`；
  与 S7b 冻结同配方参考反解逐位对账通过。**本片不重渲染、不开 GPU、无窗口、无新捕获**；
  不声明渲染级/产品级结论。

## 1. 范围

新增/改动（本提交）：

- 新增 `src/render/context/camera_output.rs`（495 行）：`CameraOutputViews`、`CameraImage`、
  `WholeOutputData`、`CameraViewError`。
- `src/render/mod.rs`：以 `#[path = "context/camera_output.rs"] pub mod camera_output;` 声明新模块
  并顶层再导出同 API。**本任务 files 只覆盖 `src/render/context/`（目录）与 `src/render/mod.rs`，
  不含父模块文件 `src/render/context.rs`**，故不触碰 `context.rs`；新模块已登记为
  `mjwarp.render` 模块源（26 项），模块路径 `mjwarp_rs::render::camera_output`（顶层再导出）。
- 新增 `tests/render_camera_output.rs`（493 行，宿主用例，默认执行）：参考对账 + 隔离不变量 + 错误面。
- 过程记录：曾两次尝试把 `context.rs` 接线拆成独立配套任务先提交（`…-module-wiring` / `…-r2`），
  分别因 depends_on 提交顺序死锁、以及提交钩子要求「模块源文件与接线同一提交」的指纹校验而作废
  （`7e4e690f`/`a1b759b0`、`090a4070`/`cb75fa56`/`db9abb09`）；最终采用上述 `#[path]` 最小方案。
  若后续任务范围补齐 `src/render/context.rs`，可在不影响公共 API 的前提下把声明移回该文件。

不改（红线核对）：

- `src/model/render.rs`（正好 500 行）未动；`src/render/context.rs` 未动；`src/render/kernel/*`、
  `src/render/kernel/cuda/*`、设备 `.cu` 未动；`DEVICE_SOURCES`(28)、产品入口(6)、`CollisionArgs`
  504 B、`fixtures/precision`、T4 身份钉子、18 个 pinned 源、判据/参考/容差/夹具均未动。
- 无 `Cargo.toml`/`Cargo.lock` 改动；无新 feature；无 GPU 作业。

## 2. 冻结上游语义（`render_util.py`，rev `71da24d…`，796 行）

引用副本（与 pinned 上游检出逐字节相同）：

- `docs/evidence/d04-complex-physics/authoring-runs/run-5/frozen-source/_src/render_util.py`
  **31524 B / 796 行 / sha256 `5f43efc8…9a04`**；
- 同一文件在 pinned 上游检出 `target/mjwarp-parallel/upstream/mujoco_warp/_src/render_util.py`
  （`71da24d`）sha256 相同（本机核对）。

| 上游 | 行 | 语义 | 产品对应 |
| --- | --- | --- | --- |
| `unpack_rgb_kernel` | L170 起 | `b=f32(val&0xFF)*(1/255)`、`g=f32((val>>8)&0xFF)*(1/255)`、`r=f32((val>>16)&0xFF)*(1/255)`；输出 `vec3(r,g,b)` | `CameraOutputViews::rgb(i)`（解包）/ `rgb_packed(i)`（原样 packed 区段） |
| `get_rgb` | L212 | 以 `camera_index` 在 `rc.rgb_data` 上取 `rgb_adr[camera_index]` 起 `pixels` 个 | 同上，像素顺序 `[y][x]`、x 最快 |
| `extract_depth_kernel` | L193 起 | `clamp(val / depth_scale, 0.0, 1.0)`（**除法在前**、clamp 在后） | `depth(i, depth_scale)` / `depth_raw(i)` |
| `get_depth` | L228 | 同上，参数 `depth_scale` | 同上 |
| `_extract_seg_kernel` | L247 起 | `seg_data[world, seg_adr[camera_index]+pixel]` 原样 `(id,type)` | `segmentation(i)` / `segmentation_raw(i)` |
| `get_segmentation` | L264 | 同上 | 同上 |

实现口径：

- RGB 常量取 S4 冻结的 `INV_255 = f32::from_bits(0x3B80_8081)`（= `float32(1.0/255.0)`），
  乘法按 f32；`(1.0/255.0)` double 舍入到 f32 与本位型相同（测试断言）。
- depth 的 clamp 按上游 `wp.clamp` 的 `fminf/fmaxf` 语义实现：`NaN → 0.0`、`-0.0 → +0.0`、
  `>1 → 1.0`、`<0 → 0.0`；测试侧独立用 `clamp` 公式对账（夹具值域为有限非负，二者逐位相同）。
- `camera_index` 语义：**激活相机顺序位置**（`cam_active` 选中顺序 = `OutputLayout` adr 顺序），
  不是模型相机 id；`CameraOutputViews::camera_id(i)` 给出→模型 id 的映射。
- 逐相机分辨率取 `CameraPlan.resolutions`（允许各相机不同宽高）。
- 本片只做单 world：`nworld > 1` 明确拒绝（`NworldUnsupported`），多 world 视图归 R10。

## 3. 错误面（全部读取前返回，不 panic）

| 条件 | 错误 |
| --- | --- |
| `camera_index` 越界 | `CameraIndex { index, cameras }` |
| 该相机该通道未启用（`*_adr[i] == -1`） | `ChannelDisabled { channel, camera_index }` |
| 通道已有相机启用但未提供整缓冲 | `BufferMissing { channel }` |
| 整缓冲长度与布局不符 | `BufferLength { channel, expected, actual }` |
| 该相机区段末端超出缓冲（`adr + pixels > len`） | `BufferRange { channel, camera_index, end, actual }` |
| `depth_scale <= 0` 或非有限 | `DepthScale { depth_scale }` |
| `nworld > 1` | `NworldUnsupported { nworld }` |
| `plan`/`layout` 非同一解析（相机数/开关表/像素总数不符） | `CameraCount` / `PlanFlags` / `LayoutPixels` |

## 4. 参考对账（不重渲染：从 S7b 冻结参考反向构造整缓冲再逐相机反解）

夹具（authoring 期 pinned 上游 GPU 采集；json 内 `resolved.cam_res=[[320,240],[240,180]]`、
`rgb_adr/depth_adr/seg_adr=[0,76800]`、共 120,000 像素）：

- `fixtures/render-bvh/scene-a/s7b/rgb_fast_math_false.json` = packed ABGR 参考（同配方 `use_fast_math=false`）；
- `fixtures/render-bvh/scene-a/s7b/depth_seg_fast_math_false.json` = depth f32 位型 + seg 对参考。

把两相机的逐相机参考值按 adr 顺序拼成整缓冲后，经 `CameraOutputViews` 逐相机反解：

| 项 | 计数 | 结果 |
| --- | --- | --- |
| RGB packed 区段（`rgb_packed`） | 76,800 + 43,200 = **120,000** | **逐位相等 120,000/120,000** |
| RGB 解包通道值（`rgb`，r/g/b × 3） | 120,000 × 3 = **360,000** | **逐位相等 360,000/360,000**（对上游同款公式） |
| depth 原始位型（`depth_raw`） | **120,000** | **逐位相等 120,000/120,000** |
| depth 缩放后（`depth`，scale ∈ {1.0, 0.1, 100.0}） | 120,000 × 3 = **360,000** | **逐位相等 360,000/360,000**（对独立 clamp 公式） |
| seg 对（`segmentation`/`segmentation_raw`） | **120,000** | **全等 120,000/120,000** |

附加交叉不变量（夹具数值，非产品推断）：

- 两相机背景 `(-1,-1)` 像素数 = depth 零像素数 = **16,995 / 27,498**（test 断言）；
- scale=1.0 时 clamp 到 1.0 的像素 **12,890 / 15,702**；参考原始深度最大 **1.68258 / 3.70316 m**；
- `INV_255` 位型 = `float32(1.0/255.0)` = `0x3B80_8081`（double 1/255 舍入一致）。

## 5. 相机隔离不变量（合成哨兵）

2 相机、不同分辨率 **4×3（12 像素）+ 2×2（4 像素）**、相机顺序与模型 id 相反（`cameras=[1,0]`）：

- `rgb_adr/depth_adr/seg_adr = [0, 12]`；A 视图恰为 `[0,12)`、B 视图恰为 `[12,16)`，切片互不重叠、不越界读邻居；
- 每相机像素按**相机局部**行主序 `y*width+x` 映射（`CameraImage::get(x,y)` 断言）；
- 不同分辨率边界正确（A 4×3 / B 2×2；`row_major_index` 越界返回 `None`）；
- depth/seg 同样逐相机区段化（depth 缩放后可按全局下标反解，无跨相机污染）。

## 6. 定向测试与门禁

命令（默认特性 = `cuda`；本文件为宿主用例，不建 context、不发射内核、不读设备）：

```text
cargo test --locked --test render_camera_output
```

结果：**4 passed / 0 failed / 0 ignored**（`test result: ok. 4 passed`；wall ≈0.10 s，见
`logs/directed-render-camera-output.log`）。四条用例：参考对账（RGB）、参考对账（depth+seg）、
相机隔离/行主序、错误面（越界/未启用/缺失/长度/`depth_scale`/`nworld`/plan-layout 不一致）。

四门禁（`normify gate` 绑定最终树；同树命令输出存档在 `logs/`）：

| 门禁 | 命令 | 结果 |
| --- | --- | --- |
| fmt | `cargo fmt --check` | 通过（exit 0） |
| clippy | `cargo clippy --all-targets --locked -- -D warnings` | 通过（exit 0） |
| clippy-cuda | `cargo clippy --all-targets --locked --features cuda -- -D warnings` | 通过（exit 0） |
| unit | `cargo test --locked` | 通过：**1010 过 / 0 败 / 376 ignored**（124 个 `test result` 行，含 doctests 13；日志 2132 行） |

## 7. 文件与证据

- 改动源码：`src/render/context/camera_output.rs`（495 行，新文件）、`src/render/mod.rs`（374 行，
  `#[path]` 声明 + 顶层再导出）；测试：`tests/render_camera_output.rs`（493 行）——行数均 **≤500**；
  `src/render/context.rs`（426 行）**未触碰**。
- 证据清单与 sha/bytes/lines：`manifest.json`（改动文件、夹具、运行日志）。
- 运行日志：`logs/fmt-check.log`、`logs/clippy.log`、`logs/clippy-cuda.log`、`logs/unit.log`
  （>500 行的机器可读例外，见 receipt `evidence_format_notes`）、
  `logs/directed-render-camera-output.log`、`logs/gates-summary.txt`。
- 依据：R08 计划 `docs/development-evidence/render-bvh-precheck/plan.json`（`34bc2bfe`），
  上游 pin `71da24d956378a87a703b6e1442b13aec0c4ac29`，夹具 provenance
  `fixtures/render-bvh/scene-a/provenance.json`（`upstream_revision` 同上）。

## 8. 边界与不宣称

- 只做宿主侧视图与对账：**不重渲染、不开 GPU、无窗口、无新捕获**，不构建/发射内核。
- 不宣告任何渲染级/产品级验收、覆盖、精度或等价结论（任务 `forbid_claim` 3 条词表在证据文本与
  提交信息中 0 命中；本文亦不复述该词表）。
- 未接产品入口调用链（S2 端到端另片）；多 world（`nworld>1`）归 R10；AA/着色/光照不在本片。
