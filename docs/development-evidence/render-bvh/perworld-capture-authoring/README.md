# 逐 world 渲染参考采集 · 离线 authoring（不跑 GPU、不写参考）

任务：`2026-10-30-render-perworld-capture-authoring`（Arch；P8；offline authoring）
上游缺口卡：`2026-10-28-render-per-world-geom-light-gap`（per-world geom 位姿；本片只做采集 authoring）

本片交付**只有脚本 + 可行性证据 + dry-run 自检**：不开 GPU 采集、不写 `fixtures/` 参考、不跑产品代码。
实际采集必须另走已签窗口（脚本已按「窗口内逐字执行」的形态写好）。

## 1. 采集脚本

| 项 | 值 |
| --- | --- |
| 路径 | `scripts/capture_render_perworld.py` |
| sha256 | `971cb8b13db82a9076c81b15936b2c97ffe2df8d3c0e47d1d9d0b10c78552775` |
| 字节 | 23,531 |
| 行数 | **497**（LF 计数；新文件 500 行上限内） |
| ASCII 自检 | 非 ASCII 字节 **0** |
| 语法自检 | pinned python `ast.parse` → `PY_SYNTAX_OK` |
| dry-run | `--dry-run` exit **0**（输出见 `materials/dryrun-plan.json`） |

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
- legacy reference manifest = **69** 条（采集前后对拍，防误改既有参考）

## 3. 参考设计（窗口内执行）

- **场景**：`fixtures/render-bvh/scene-a/scene.xml`（单 hinge `tri-hinge`、3 灯、2 相机；已有参考不覆盖 per-world）
- **world 位姿**：`--worlds 3`、`--hinge-deg "0,20,40"`（度），逐 world 写入唯一标量 hinge 的 `qpos` 地址 → `geom_xpos/geom_xmat` 逐 world 不同
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
| `materials/dryrun-plan.json` | pinned python `--dry-run` 原样输出（环境 + 计划 + 能力断言） |
