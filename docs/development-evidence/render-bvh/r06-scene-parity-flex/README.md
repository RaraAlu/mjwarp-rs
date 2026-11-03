# R06 scene parity：flex 字段 additive 扩展（`model_parity` 三工具）+ scene-a 零回归

任务：`2026-11-04-render-r06-scene-parity-flex-fields`（p74，owner Backend；depends `2026-11-04-render-r06-s3a-flex-device`）。
基线 HEAD：`a8ea9274`。**离线、零 GPU、零 fixtures 改动**（本片只改 `scripts/authoring/render-bvh/` 三个工具 + 本证据目录）。

## 1. 目的

S3b①（`capture_render_flex.py` + `scene-flex` fixture）需要 `model-{312,315}.json` 覆盖 **flex 模型数组**，
否则 flex 渲染参考的「产品 MuJoCo 3.12 与参考 MuJoCo 3.15 编译同一场景」前提没有机器证据
（QA 报的 S2c/S3b 前置）。本片把 `flex_*` 字段 **additive** 加进 S0a 的 parity 工具链：

| 文件 | 改动 |
| --- | --- |
| `scripts/authoring/render-bvh/compile_scene_312.cpp` | `dump_counts` 增 6 个 flex 计数；`dump_arrays` 增 17 个 flex 数组（int32/float64/float32，与 3.12 ABI 宽度一致） |
| `scripts/authoring/render-bvh/model_parity.py` | `FLOAT32_FIELDS`/`DOUBLE_FIELDS`/`INT32_FIELDS`/`COUNT_FIELDS` 增同名 flex 字段（3.15 Python 侧） |
| `scripts/authoring/render-bvh/model_parity_judge.py` | **未改**：获批判据（int/byte 精确、float32 f32 位、float64 先看 f32 位）对新 f64 字段自动适用，未新增豁免/容差 |

新增 flex 字段（两版同名同宽）：counts `nflex/nflexvert/nflexedge/nflexelem/nflexelemdata/nflexshelldata`；
int32 `flex_dim/flex_vertadr/flex_vertnum/flex_edgeadr/flex_edgenum/flex_elemadr/flex_elemnum/flex_elemdataadr/flex_shellnum/flex_shelldataadr/flex_vertbodyid/flex_elem/flex_shell`；
float64 `flex_radius/flex_vert/flex_vert0`；float32 `flex_rgba`。

## 2. scene-a 零回归自证（旧 render 字段逐字节不变）

命令（与 S0a 相同 flow，仅多一个 `--out` 指向 target）：

```
cl /nologo /O2 /EHsc /std:c++17 /I<3.12 include> compile_scene_312.cpp /link /LIBPATH:<3.12 lib> mujoco.lib
python -B scripts/authoring/render-bvh/model_parity.py \
  --exe target/render-bvh/s0a/compile_scene_312.exe \
  --cpp scripts/authoring/render-bvh/compile_scene_312.cpp \
  --scene fixtures/render-bvh/scene-a/scene.xml \
  --out target/parity-scene-a-regression \
  --evidence docs/development-evidence/render-bvh/r06-scene-parity-flex/scene-a-regression
```

结果（`scene-a-regression/parity.json` + `scene-a-regression/zero-regression.json`）：

- `parity = pass`、`judged_failures = []`、`counts_match = true`、`deterministic = true`；
- `fields_compared = 150`（旧 133 + flex 17），`fields_equal = 143`（7 个历史项仍按 S0a 判据分类/豁免，与 S0a 一致）；
- **旧字段零回归**：312/315 两侧各 **134 个旧数组条目（含 `_end`）逐条相等**、`old_fields_changed = []`、
  `removed_fields = []`、旧 `counts` 键零变化；只新增 17 个 `flex_*` 数组。
- `fixtures/render-bvh/scene-a/` 零改动（`--out` 指向 `target/`，`unchanged fixtures/` 随时可验）。

## 3. scene-flex 首跑（含 flex 数组；硬闸差一步，等 Chief 裁决）

同一扩展工具对 `scene-flex`（1D flex ×2 + 刚性几何）跑 parity（`scene-flex/parity.json`）：

- `fields_compared = 150`、`fields_equal = 143`、**`judged_failures = []`**；
- flex 字段全部落在「strict 相等或 f64 tail/f32 位相等」内：`flex_dim/…vertadr/…` 等 int 数组严格相等，
  `flex_radius/flex_vert/flex_vert0` 为 f64 尾差但 **f32 位相同**（S0a 已获批判据），`flex_rgba` f32 严格相等；
- 唯一硬闸不通过：`counts_match = false` → **`nbuffer` 312=37552 vs 315=37296**；
- 对照实验：**同场景删除 `<deformable>`（去 flex）后 `nbuffer` 两侧均 32368**（相等）→ 差异来自 3.15 对
  flex 缓冲区的尺寸算法变更，不是场景/产品缺陷；`nbuffer` 不被渲染路径消费（upstream render 读模型数组，不读 nbuffer）。
- 处置：**本片不自行放宽判据**（S0a 判据属用户 2026-10-24 签名级口径，`counts_match` 是硬闸）。
  已在卡讨论中报 Chief 裁决「场景级记录 nbuffer 版本差异」或替代口径；裁决前 **不宣称 flex 场景 model parity 通过**，
  S3b① 的 `model-{312,315}.json` 也暂不落 fixture（避免把 fail 证据写进参考）。

## 4. 验证

- 零 GPU、零窗口、零参考采集；不改 `fixtures/`、`src/`、判据/容差。
- 工具行数：`compile_scene_312.cpp` 375、`model_parity.py` 490、`model_parity_judge.py` 122（均 ≤500）。
- 门禁：fmt / clippy / clippy-cuda / unit 全过（Rust 面零改动，走既有 gate 记录）。
- 证据文件 sha256 见 `manifest.txt`；源工具 sha256 见 `scene-a-regression/zero-regression.json`。

## 5. 遗留

- `nbuffer` 裁决（Chief）：批 → S3b① 直接复用本扩展工具生成 flex 场景 `model-{312,315}.json` 并把
  「nbuffer 已知版本差异」写进 scene-flex `provenance.json`；不批 → S3b① 改为「render 字段 parity 通过 +
  flex 数组严格相等（本证据 §3）」分项记录，flagship 判据不动。
- 本片不跑 scene-flex 采集（S3b① 归 r2 卡）。
