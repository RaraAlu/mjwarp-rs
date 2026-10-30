# 逐 world 渲染参考采集 · 离线 authoring（不跑 GPU、不写参考）

任务：`2026-10-30-render-perworld-capture-authoring`（Arch；P8；offline authoring）
上游缺口卡：`2026-10-28-render-per-world-geom-light-gap`（per-world geom 位姿；本片只做采集 authoring）

本片交付**只有脚本 + 可行性证据 + dry-run 自检**：不开 GPU 采集、不写 `fixtures/` 参考、不跑产品代码。
实际采集必须另走已签窗口（脚本已按「窗口内逐字执行」的形态写好）。

## 1. 采集脚本

| 项 | 值 |
| --- | --- |
| 路径 | `scripts/capture_render_perworld.py` |
| sha256 | `67a760fa400f2cf89c19a700e49d08c619a78008664f03cd69fab21bd2077815`（窗口期修复后；旧 `971cb8b1…` 见 §7 errata） |
| 字节 | 24,008 |
| 行数 | **499**（LF 计数；新文件 500 行上限内） |
| ASCII 自检 | 非 ASCII 字节 **0** |
| 语法自检 | pinned python `ast.parse` → `PY_SYNTAX_OK` |
| dry-run | `--dry-run` exit **0**（输出见 `materials/dryrun-plan.json`；含静态模型自检，见 §7） |

派生自 `scripts/capture_render_hfield.py`（R05-S3a）的工程骨架（pinned env 校验、deterministic ZIP_STORED npz、两跑逐数组/逐字节相等、legacy reference manifest 前后对拍、`--dry-run` 只做环境与计划检查）；R05/R09 脚本与既有参考**零改动**。

## 2. 可行性结论（pinned 环境原生支持多 world）

不新增上游能力，全部用 pinned `target/mjwarp-parallel/upstream` @ `71da24d956378a87a703b6e1442b13aec0c4ac29` 现成 API：

| 需求 | 上游 API / 事实 | 出处 |
| --- | --- | --- |
| 多 world 容器 | `mujoco_warp.put_data(mjm, mjd, nworld=N)`；`Data.qpos` = `(nworld, nq)` float | `io.py::put_data`；`types.py`（`qpos: array("nworld","nq",float)`） |
| 逐 world 位姿 | `mjw.forward(m, d)` → `geom_xpos (nworld,ngeom,3)`、`geom_xmat (nworld,ngeom,3,3)` | `types.py` Data 字段 |
| 逐 world 渲染上下文 | `create_render_context(mjm, nworld=N, ...)` | `render_util.py` 签名 |
| 逐 world 取图 | `mjw.render(m,d,rc)` + `get_rgb/get_depth/get_segmentation`，输出 `(nworld, H, W)` | `render_util.py::get_rgb` 文档与 kernel `dim=(rgb_out.shape[0], H*W)` |
| 逐 world 采样依据 | 上游着色 kernel 按 `geom_xpos_in[worldid, gi]` 2D 索引 | `render.py`（`_geom_rgba`/`_ray_geom` 等） |

dry-run 实测（`materials/dryrun-plan.json`，只读、未建 CUDA device）：
- `put_data_params` 含 `nworld`；`create_render_context_params` 含 `nworld`
- 版本：python 3.12.13 / warp 1.15.0 / mujoco 3.15.0 / mujoco_warp 3.15.0 / numpy 2.5.4
- python `sha256=feeec36120097a03201a96ae17a0e2c1fba70dd6241cb4bfdd9e7c2a00bada77`
- upstream `71da24d9…`、`clean=True`
- scene `fixtures/render-bvh/scene-a/scene.xml` `sha256=a9f1f0a35464f4df17f191e1d68431bda0f17648273578e58fa08f84b0449289`（4,639 B）
- `camera_modes=[0,3]` → 只启用 `mode==0` 固定相机（`cam_active_fixed_only=[true,false]`），避免 targetbody 相机模式带来未验证的位姿语义
- **静态模型自检**（dry-run 新增，窗口期修复）：`njnt=2`、`jnt_type=[0(FREE),3(HINGE)]`、`jnt_qposadr=[0,7]`、`nq=8`、`qpos0=[0.1,-0.2,0.3,0.961636,0.10017,-0.050085,0.250426,0.0]`、相机 `[id=0 cam-persp mode=0, id=1 cam-track mode=3]`、每 world `qpos` 形状 **(3,8)**（hinge 地址 7 依次 0 / 0.349066 / 0.698132 rad）
- legacy reference manifest = **79** 条（窗口期实测；69 + SDF 采集参考新增 10 个文件；采集前后对拍，防误改既有参考）

## 3. 参考设计（窗口内执行）

- **场景**：`fixtures/render-bvh/scene-a/scene.xml`（freejoint 基座 + 单 hinge `tri-hinge`、3 灯、2 相机；已有参考不覆盖 per-world）
- **world 位姿**：`--worlds 3`、`--hinge-deg "0,20,40"`（度），以**模型默认 `qpos0` 为基座**（不是全零），逐 world 只覆盖唯一 hinge 的 `qpos` 地址 7 → `geom_xpos/geom_xmat` 逐 world 不同；输出 `qpos` 形状 **(3,8)** float32
- **配方**：`use_fast_math=false`（产品同配方）、`use_shadows=true`、`samples_per_pixel=1`、固定相机
- **输出**（`fixtures/render-bvh/scene-a/perworld/<tag>/`）：
  `rgb.npz`/`depth.npz`/`seg.npz`（`*_w{w}_cam{c}`）、`state.npz`（`qpos`/`geom_xpos`/`geom_xmat`/`light_xpos`/`light_xdir`）、`capture.json`（recipe + 两跑相等判定 + legacy manifest 对拍）
- **两跑确定性**：`--repeat 2`，数组逐个 `np.array_equal` 且 npz 字节（deterministic zip，固定时间戳）相等，否则非零退出
- **参考只增不改**：`--out-dir` 必须在 `scene-a/perworld/` 子树内且不得已存在非空；legacy manifest 覆盖除该子树外的全部 render-bvh 参考

## 4. 范围与不宣称

- 本片 **不跑 GPU 采集**、**不写 `fixtures/` 参考**、不改产品代码、不改判据/容差。
- 该参考只覆盖 **per-world geom 位姿**（`geom_xpos/geom_xmat`）。`light_xpos/light_xdir` 一并落盘**仅供完整性**：产品设备路径当前按单 world 索引灯位姿，因此**不宣称 per-world 灯位姿**已覆盖（写入 `capture.json.per_world_scope`）。
- 采集窗口属**新窗口**（authoring-only 也算 GPU 满载任务），须用户双签窗口计划+基线后才能执行。
- 采集产出参考后，判定/容差与是否纳入验收仍需按精度契约流程另行批准；本片不宣称任何渲染数值结论。

## 5. 用法

```text
<pinned-python> -B scripts/capture_render_perworld.py --help
<pinned-python> -B scripts/capture_render_perworld.py --dry-run \
    --out-dir fixtures/render-bvh/scene-a/perworld/nw3-hinge02040
<pinned-python> -B scripts/capture_render_perworld.py \
    --out-dir fixtures/render-bvh/scene-a/perworld/nw3-hinge02040
```

pinned python：`target/mjwarp-parallel/venv/Scripts/python.exe`；`MJWARP_CAPTURE_AT`（ISO8601）用于固化 `capture.json` 的采集时刻。

## 6. 证据清单

| 文件 | 内容 |
| --- | --- |
| `README.md` | 本文（可行性、设计、范围） |
| `receipt.json` | 任务 id、脚本 sha/字节/行数、dry-run 结论、环境版本 |
| `materials/dryrun-plan.json` | pinned python `--dry-run` 原样输出（环境 + 计划 + **静态模型自检**） |

## 7. 窗口期修复 errata（attempt1 采集失败后，2026-10-31）

per-world 采集窗 attempt1（例外 monitor 60/60 通过后）在**脚本场景自检阶段**非零退出、未到渲染/写盘：
`expected exactly one scalar hinge joint, found scalar=[0,1] hinge=[1]`。失败记录原样保留在
`docs/development-evidence/render-bvh/perworld-capture-window/attempt1/` 与
`target/render-bvh/perworld-capture-window/attempt1/`；本条只更正错误假设，不改写任何历史结论。

| # | 缺陷（错误假设） | 修复 |
| --- | --- | --- |
| 1 | `scalar` 把 FREE(0) 也计入并要求「恰好 1 个 scalar」 | `scalar` 只取 SLIDE(2)/HINGE(3)，要求恰好 1 个 HINGE；FREE/BALL 允许且不计入；错误串写清 |
| 2 | 每 world `qpos` 以全零为基座 → free 的四元数变 (0,0,0,0)、基座位姿丢失 | 基座改用模型默认 `qpos0`：`base = np.array(mjm.qpos0)`，再按 world 覆盖 hinge 地址 7；输出仍 float32 |
| 3 | 材料假设 `nq=1`、`qpos (3,1)`、legacy 69 | 实测 `nq=8`、`qpos (3,8)`、legacy **79**（69 + SDF 参考新增 10）；材料与期望产物同步更正 |
| 4 | dry-run 只查环境与计划，挡不住「材料假设与真实场景不符」 | dry-run 增加静态模型自检：`njnt/jnt_type/jnt_qposadr/nq/qpos0`、每 world hinge 角度与 `qpos`、相机列表（id/mode/name）、legacy 计数，全部写入 `materials/dryrun-plan.json` |

窗口期修复先例：R02-S2 `b3a47aca`。脚本身份 `971cb8b1…`/23,531 B/497 行 → `67a760fa…`/24,008 B/499 行；
dry-run 离线复跑 exit 0（不建 CUDA、不写盘），输出即 `materials/dryrun-plan.json`。
