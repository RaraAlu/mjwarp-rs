# perworld L v2 灯备选准备片：脚本参数化 + `scene-a-light` 场景变体（零 GPU）

- 任务：`2026-11-06-render-perworld-l-v2-light-prep`（owner Backend、p55、P8-渲染与BVH）
- 边界/定位：**备选准备片**，不构成 perworld L v2 启动；零 GPU、不采集参考、不写 `fixtures/` 参考数据（仅新增场景 XML）、
  不改判据/容差/已签 criterion v2.2、不动既有 perworld 参考与冻结件；用户选定方向后才排采集窗与判据签名卡。

## 1. 结论

- 采集脚本参数化完成：新增 `--scene`（默认 `fixtures/render-bvh/scene-a/scene.xml`）与 `--out-root`
  （默认 `fixtures/render-bvh/scene-a/perworld`）；**默认参数下 dry-run 除新增参数回显外逐字段一致**
  （`artifacts/default-dryrun-diff.json`：unexpected=**0**；允许差异=新增 `/paths` 回显、脚本 sha/bytes、
  legacy 计数 106→107，理由见 §5）。
- 新场景变体 `fixtures/render-bvh/scene-a-light/scene.xml`：三灯 `sun`/`lamp`/`spot` 挂在 `tri_body` 上；
  场景其余内容与 scene-a 同源、灯属性逐字不变；唯一结构差异=三灯从 worldbody 移入 `tri_body` 元素
  （`artifacts/scene.diff`）。
- dry-run 三项自检通过（`logs/explicit-light-dryrun.json`）：① 场景加载与静态自检；② 三灯×三 world
  灯位姿数值表 + 差异键 + 非零判定；③ 默认/显式路径回显、显式 out-dir 位于 out-root 之下。
- 负例自检：`--out-dir` 目标非空即拒（exit 1）；out-dir 在 out-root 之外即拒（exit 1）
  （`logs/negative-*.log`）。

## 2. 参数化实现与行数

| 项 | 内容 |
| --- | --- |
| `--scene` | 默认 `SCENE_XML`（scene-a）；只换加载 XML，不改 recipe/判据/容差 |
| `--out-root` | 默认 `OUT_ROOT`（scene-a/perworld）；`--out-dir` 必须位于其下、目标非空即拒 |
| `legacy_manifest(out_root)` | 排除集合随 `--out-root` 走（新根不误伤既有参考） |
| 拆分模块 | `scripts/perworld_light_prep.py`：路径回显、dry-run plan 组装、三 world 灯位姿 CPU 自算；主脚本只 import+调用 |
| 不变 | recipe 字段、判据/容差、`--repeat 2` 两次采集逐位一致语义、默认路径行为 |
| 行数（git blob 换行口径） | `scripts/capture_render_perworld.py` **499 → 497**；`scripts/perworld_light_prep.py` **160（新）**；均 ≤500 |
| sha256 | 主脚本 `3e1d207c…`、模块 `9fd5f8c2…`、场景 `4db66e2c…`（详见 `artifacts/prep-meta.json`） |

## 3. 场景变体与 MJCF 挂载方式

- MJCF **没有** `<light body="...">` 属性（实测 `XML Error: Schema violation: unrecognized attribute: 'body'`），
  因此按 MuJoCo 标准做法把三灯**移入** `<body name="tri_body">` 元素实现挂载。
- 灯属性（`type`/`pos`/`dir`/`diffuse`/`specular`/`ambient`/`attenuation`/`cutoff`/`exponent`/
  `softness`/`castshadow`/`active`/`bulbradius`/`intensity`/`range`）**逐字未改**；
  `cam-persp` mode 0 固定、`cam-track` mode 3 不变；scene-a 既有文件零改动。
- `scene-a-light/` 目录**只含 `scene.xml`**（无 npz/日志/临时文件）。

## 4. dry-run 三项自检（显式 scene-a-light 路径）

- **① 场景加载/静态自检**：`njnt`/`nq`/`jnt_type`/`jnt_qposadr`/`qpos0`、相机 mode（cam-persp 0、cam-track 3）。
- **② 三灯 × 三 world 灯位姿（CPU `mujoco.MjData`+`mj_forward`，不建 CUDA device、不渲染）**：

| world | `sun` xdir 夹角 | `lamp` pos 位移 | `spot` pos 位移 |
| --- | --- | --- | --- |
| 1（hinge 20°） | 10.745878447° | 0.222378165 m | 0.0 |
| 2（hinge 40°） | 21.255926664° | 0.437999482 m | 0.0 |

判定：`verdict.nonzero = true`（`sun`/`lamp` 差异非零；**`spot` 的 pos=0 0 2、dir=0 0 -1 落在 tri_body
z 旋转轴上，旋转下位移/朝向恒 0**——属场景几何事实，只记录、**不做渲染判别力断言**）。
- **③ 路径参数化**：`paths` 块同时打印默认与显式 scene/out-root、`scene_explicit`/`out_root_explicit`、
  生效 out_dir；显式路径均在 `--out-root` 之下。
- 本片**不做渲染判别力断言、不落参考**（采集窗与判据签名时另办）。

## 5. legacy 计数变化说明（Arch 要求：写清前后值与理由）

| run | `legacy_manifest_entries` | 理由 |
| --- | --- | --- |
| 改动前默认 | 106 | — |
| 改动后默认 | 107 | 新增静态夹具 `fixtures/render-bvh/scene-a-light/scene.xml` 被 legacy 全量扫描计入（默认 out-root 仍是 scene-a/perworld） |
| 显式 light 路径 | 112 | 其 out-root=`scene-a-light/perworld`，排除集合随之改变 → `fixtures/render-bvh/scene-a/perworld/` 的 5 个既有文件被计入（107+5）；这是「排除随 out-root 走」的**预期**行为 |

## 6. 边界与非声明

- 零 GPU、不开窗、不采集、不写参考；仅新增场景 XML；不改判据/容差/criterion v2.2、不动既有 perworld 参考与冻结件；
  `unchanged src/`、`unchanged fixtures/render-bvh/scene-a/`、`unchanged docs/development-evidence/render-bvh/sdf-criterion-v2.2/`。
- **非声明**：本片只是备选准备，**不构成 perworld L v2 启动**；不产参考、不宣称 per-world 灯位姿覆盖已成立、
  不宣称与上游实现等价、不宣称任何渲染数值结论。
- 可回滚：撤回 2 个脚本改动 + 1 个场景文件 + 本卡证据即可，不影响任何既有验证面/冻结件。

## 7. 证据清单

| 路径 | 说明 |
| --- | --- |
| `README.md` | 本文件 |
| `receipt.json` | 机器可读回执（`/task_id` 供 normify `json` 检查） |
| `logs/before-default-dryrun.json` | 改动前（HEAD 脚本）默认 dry-run 原始输出 |
| `logs/after-default-dryrun.json` | 改动后同命令默认 dry-run 原始输出 |
| `logs/explicit-light-dryrun.json` | 显式 `--scene/--out-root` dry-run 原始输出（含三灯自检） |
| `logs/negative-outdir-nonempty.log` | 目标非空即拒负例（exit 1） |
| `logs/negative-outdir-outside-root.log` | out-dir 越界负例（exit 1） |
| `logs/line-counts.txt` | 两个 .py 行数（git blob 口径） |
| `artifacts/default-dryrun-diff.json` | 默认行为前后逐字段 diff + 分类（unexpected=0） |
| `artifacts/scene.diff` | scene-a → scene-a-light 场景 diff（仅三灯挂载） |
| `artifacts/script.diff` | 主脚本改动 diff |
| `artifacts/prep-meta.json` | 脚本/模块/场景 行数、bytes、sha256 + legacy 计数说明 + 灯自检 |
| `manifest.txt` | 本目录 sha256 + bytes（首行 `#`，不自列） |

## 8. 复现

```powershell
# 改动后默认（与改动前逐字段对照用）
target/mjwarp-parallel/venv/Scripts/python.exe -B scripts/capture_render_perworld.py --dry-run `
    --out-dir fixtures/render-bvh/scene-a/perworld/prep-default-dryrun
# 显式 scene-a-light + 三项自检
$env:PYTHONIOENCODING='utf-8'
target/mjwarp-parallel/venv/Scripts/python.exe -B scripts/capture_render_perworld.py --dry-run `
    --scene fixtures/render-bvh/scene-a-light/scene.xml `
    --out-root fixtures/render-bvh/scene-a-light/perworld `
    --out-dir fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040 --worlds 3 --hinge-deg 0,20,40
```
