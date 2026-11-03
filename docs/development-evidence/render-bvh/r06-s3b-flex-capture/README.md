# R06-S3b① r2：flex 渲染参考离线采集（`capture_render_flex.py` + `scene-flex`）

任务：`2026-11-04-render-r06-s3b-flex-capture-r2`（p74，owner Backend；depends `2026-11-04-render-r06-scene-parity-flex-fields`）。
基线 HEAD：`7b18c3e8`。脚本/场景/模型 parity 为**零 GPU**；参考采集为**秒级短 CUDA**（Chief go 制，用前群内报一句）。
不做验收窗口、不排队占窗、不产参考比对；2D/3D 只留机制，首版场景 = **1D flex + 刚性几何共存**。

## 1. 交付面

| 文件 | 行数/大小 | 内容 |
| --- | --- | --- |
| `scripts/authoring/capture_render_flex.py` | 498 行 | 离线 authoring：pinned editable 校验（禁 PyPI 轮子）、`--repeat 2` 字节一致、fixtures sha 前后对照、flex provenance、`state.npz`（含 `flexvert_xpos`） |
| `fixtures/render-bvh/scene-flex/scene.xml` | 4613 B | 1D flex×2（`flex-line` r=0.02 Z 字线；`flex-line-fat` r=0.04 单边，alpha≠0）+ 刚性几何（floor/box/mesh/capsule）+ 2 相机 + 2 灯；flex 与刚性面/地板 ≥0.3（tie-break flex 永不赢，不依赖平局） |
| `fixtures/render-bvh/scene-flex/model-312.json` | 67109 B | MuJoCo 3.12 C API 编译 dump，**含 17 个 flex 数组**（R06 scene parity flex fields 卡的扩展工具） |
| `fixtures/render-bvh/scene-flex/model-315.json` | 65371 B | MuJoCo 3.15 Python dump，同名同宽 flex 数组 |
| `fixtures/render-bvh/scene-flex/scene-{312,315}.mjb` | 27291/27233 B | 两版编译产物 |
| `fixtures/render-bvh/scene-flex/provenance.json` | 3285 B | 场景/模型 hash、parity 结果、flex 字段摘要、参考语义、命令、环境 |
| `fixtures/render-bvh/scene-flex/reference/<group>/` | 采集后生成 | `rgb/depth/seg/state.npz` + `capture.json` |

## 2. §pre-check：实现口径

- **pinned editable**：`target/mjwarp-parallel/upstream`（HEAD `71da24d…`，clean），脚本断言
  `mujoco_warp.__file__` 落在该 checkout 内；PyPI 3.15.0 轮子与 pin 有 5 个 `.py` 逐字节差异（S0 已留陷阱）。
- **场景编译**：pinned MuJoCo 3.15 → `nflex=2, flex_dim=[1,1], flex_radius=[0.02,0.04], nflexvert=6,
  nflexedge=4, nflexelem=4, nflexelemdata=8`；1D flex 图元映射 = 每边一个胶囊（`edgeid = flex_edgeadr[f]+e`）。
- **模型 parity**：`parity/parity.json`（扩展工具）→ `fields_compared=150`、`judged_failures=[]`；
  flex int 数组严格相等、`flex_radius/flex_vert/flex_vert0` f64 尾差但 f32 位相等、`flex_rgba` f32 相等
  （`parity/flex-fields.json`）。唯一硬闸 `counts_match=false` 来自 `nbuffer`（312=37552 vs 315=37296；
  同场景去 flex 对照 32368==32368）——属 3.15 flex 缓冲区尺寸版本差异、渲染路径不消费；
  **未放宽判据、未宣称 parity pass**，已报 Chief 裁决（`provenance.json.model_parity.known_difference`）。
- **参考语义（S3a 契约）**：flex 命中 `geom_id=-2`、`mesh_id=flexid`、seg `(flexid, 9)`（首样本写）、
  depth `dist * -ray_dir_local_cam[2]`、颜色 `flex_rgba[mesh_id]`。
- **采集 recipe**：产品同 recipe（`use_fast_math=false`、`use_shadows=true`、`samples_per_pixel=1`、
  `flex_render_smooth=true`）；`--repeat 2` 两跑逐数组 + npz 字节双比；采集前后除 `scene-flex/` 外
  全部 `fixtures/render-bvh` 文件 sha 对照。
- **输出**：`rgb.npz`/`depth.npz`/`seg.npz`/`state.npz`/`capture.json`；`state.npz` 含
  `qpos, geom_xpos, geom_xmat, light_xpos, light_xdir, flexvert_xpos`（flex 参考 replay 必需）。

## 3. 零 GPU 验证（已完成，含真模型 CPU preflight）

- `scripts/authoring/capture_render_flex.py --dry-run` → exit 0，wall 15.4 s（`dry-run.json`）：
  pinned 版本 warp 1.15.0 / mujoco 3.15.0 / mujoco_warp 3.15.0 / numpy 2.5.4、upstream clean
  （`mujoco_warp.__file__` 落在 pin checkout 内）、legacy 文件 84 个、计划输出 5 个文件、
  scene sha `fe233c49…`。
- **CPU preflight（新增，真模型、不碰 CUDA）**：`cpu_preflight()` 在 `--dry-run` 和非 dry-run 的 GPU 段
  **之前**都执行——`wp.set_device("cpu")` → pinned `mjwarp.put_model/put_data/forward` 跑同一模型 →
  `state_arrays()` 校验 `flexvert_xpos` 形状/有限性（`[1, 6, 3]`，sum `1.4000000953674316`）→
  `flex_model_report()`，其中 `Stub` 复算 upstream `render_util.create_render_context` 的
  `bvh_nflexgeom`/`flex_geom_flexid`/`flex_geom_edgeid`（本场景 `[0,1,2,0]`）。preflight 未通过则 fail，
  **不允许进 GPU 段**；结果同时写进 `dry-run.json` 与正式 `capture.json.environment.cpu_preflight`。
  attempt B/C 的两次失败都落在该函数路径上，故先以 CPU preflight 离线自证。
- 场景编译 + model-312/315 dump + parity：见 §2。
- 脚本行数 497 ≤500（line-limit 棘轮内）；`fixtures/` 变更全部登记在本卡（`scene-flex/`），
  `src/render/kernel/cuda/` 零改动。

## 4. 采集尝试记录（失败也如实入库）

| attempt | 时间 | wall | 中止阶段 | 失败原因 | 参考产物 | GPU |
| --- | --- | --- | --- | --- | --- | --- |
| A | 06:56:55 | 1.3 s | `put_model`（渲染前） | `NotImplementedError: Rigid flexes are not implemented.`（场景 flex 全绑 world → rigid） | 无 | 已释放（free≥14701 MiB / util 2% / 27 °C） |
| B | 06:59:04 | 44.8 s | `capture.json` provenance（两跑渲染+npz 落盘之后） | `TypeError`：`flex_edge` 是 2-D 数组，逐元素 `int()` 失败 | 仅 partial `reference/base/*.npz`；**已删**，`capture.json` 未生成 | 已释放（同上） |
| C | 06:59:47 | 1.9 s | `capture.json` provenance（warp 缓存已热） | `AttributeError`：`rc.flex_dim_np` 是 numpy 数组（无 `.numpy()`） | 仅 partial；**已删**，`capture.json` 未生成 | 已释放（同上） |
| D | 07:03:12 | 2.4 s | 完成（有界重试 #1） | 成功但 **seg 0 个 flex 像素**：原两相机没框住 flex 簇 | 输出被 E 覆盖 | 已释放 |
| E | 07:04:02 | 2.3 s | 完成（有界重试 #2） | **成功**：两跑一致、legacy 84 未动、`cam-flex` 有 **654** 个 `(flexid,9)` 像素（flexid 0/1 都覆盖） | `reference/base/` 最终版 | 已释放 |

- 原始输出逐字节保留：`attempts/attempt-{A,B,C}.log`（sha256 见 `attempts/attempts.json`）。
- A 的根因修复：flex 顶点改绑 3 个静态 anchor body（`flex_rigid=false`），同步重算 model-312/315/parity。
- B/C 的根因修复：`flex_model_report` 展平 2-D flex 数组 + 容忍 numpy/warp 两种数组；并新增 §3 的
  CPU preflight，第三次起必须先离线通过该路径才允许进 GPU。
- 每次 abort 后均确认 GPU 空（见上表）；无验收数据、无参考比对、不进验收链。

## 5. 正式采集（attempt E，success）

- 命令：`target/mjwarp-parallel/venv/Scripts/python.exe -B scripts/authoring/capture_render_flex.py --group base`
- 前置：CPU preflight 通过（exit 0，`report_ok=true / state_ok=true`）+ 三点基线 07:03:53
  `free=14771 MiB / util 1–2% / temp 28 °C`（无其他 GPU 计算任务）。
- wall **2.3 s**（≤2 分钟）；`--repeat 2` 两跑**逐字节一致**；`legacy_reference_untouched=true`（84 个旧 fixture 文件 sha 前后相同）。
- `files_sha256`：`rgb.npz 5ed502872b869bb1f37659ec22816f9f1c04405c1666f3d366b28fd298678207`、
  `depth.npz c3ee4adca6cf9874f40ab87b7f72a16b17f26647e45758342895919b725d8deb`、
  `seg.npz f3b00fa82df1e54d4c89309d820212d9808038e00ae25f25841f2b40a6277bef`、
  `state.npz 7f2fff501bfa98f41fb8d4b689337e9e62e1ec54d98c576dc0a6e8b880b61769`；
  `capture.json` sha256 **`4713e2c7e403d47e1a23aed064e8f565336287839bfd0f96fb9242a1b1f10289`**。
- 环境：python 3.12.13 / warp 1.15.0 / mujoco 3.15.0 / mujoco_warp 3.15.0 / numpy 2.5.4；GPU RTX 4070 Ti SUPER、driver 596.36；
  pinned upstream `71da24d…` clean。
- flex 覆盖：`cam-flex` seg = **654** 个 `(object_id, 9)` 像素（flexid 0/1 均出现）；`cam-persp`/`cam-track` 各 0（预期，flex 不在其视野）。
- `state.npz`：`qpos/geom_xpos/geom_xmat/light_xpos/light_xdir/flexvert_xpos`（`flexvert_xpos` `[1,6,3]`，sum `1.4000000953674316`，与 CPU preflight 一致）。
- 注：`capture.json` 的 `task_id` 仍是原卡 id（脚本常量早于 r2 重登记），产物按 r2 交付。

## 6. 边界

- 不宣称 flex 渲染数值可用：参考采集只产 raw 数据；判据/窗口归 S3b②（Arch）。
- 不做验收窗口、不排队占窗、不重冻守卫、不写判据/容差。
- 2D/3D flex 不进首窗（机制已由 S3a/S3c 单测与 NVRTC 覆盖）。
