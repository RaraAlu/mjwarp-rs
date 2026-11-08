# `camera_recipe` / perworld `capture.json.recipe.cameras` consumer inventory (zero GPU)

- 扫描时间：2026-11-08（本卡实现期）；基线修订 `19d5fb51`（start 过渡），工作树含本卡脚本改动。
- 目的：确认「8 字段 → 10 字段」没有消费者断言封闭的 8 键集合；若有 `tests/`/`src/` 依赖则停手报 Chief。
- 结论：**无封闭 8 键消费者**；`tests/`、`src/` 对本卡 `camera_recipe` **零引用**，本卡 `unchanged tests/, src/` 成立。

## 1. 本卡 `camera_recipe` 的生产/消费者（`git grep -n 'camera_recipe' -- scripts tests src`）

| 位置 | 角色 | 处置 |
| --- | --- | --- |
| `scripts/capture_render_perworld.py:53` | import（来自 `perworld_capture_meta`） | 本卡更新（+`camera_recipe`，仍 1 行） |
| `scripts/capture_render_perworld.py:451` | 唯一调用 `recipe["cameras"] = camera_recipe(mjm, rc)` | 未改 |
| `scripts/perworld_capture_meta.py:127` | 唯一定义（本卡搬入并补 2 字段） | 本卡 |
| `scripts/capture_render_hfield.py:174,390` | **独立脚本副本**（hfield 自己的 `camera_recipe`） | 不在本卡范围，未改 |
| `scripts/capture_render_sdf.py:176,406` | **独立脚本副本**（sdf 自己的 `camera_recipe`） | 不在本卡范围，未改 |
| `tests/`、`src/` | **0 处** | 无需改动 |

## 2. perworld 参考（`fixtures/render-bvh/scene-a/perworld/nw3-hinge02040/`）的消费者

| 位置 | 用法 | 是否读 `recipe.cameras` |
| --- | --- | --- |
| `tests/render_per_world.rs` | 内嵌 4 个 `*.npz` + 硬编码 npz sha256；读 `state.npz` 的 `geom_xpos/geom_xmat` | **否**（不解析 capture.json） |
| `scripts/validate/render-bvh/guards-sdf-validate.json` | 把 `capture.json` 当作冻结输入记录 `bytes`+`sha256`（哈希绑定） | **否**（仅哈希，非 schema） |
| 全仓 `git grep 'scene-a/perworld'` | 其余命中均为本脚本自身的路径/文档与 `perworld_light_prep.py` 默认 out-root | **否** |

## 3. 其他 `recipe.cameras` 命中（与本题无关）

- `tests/support/render_bvh/*`、`tests/render_kernel_*.rs`、`tests/render_r08_cameras.rs` 等出现的 `recipe.cameras`
  属于 **Rust 侧其他参考格式**（hfield / sdf / r08 / s7b2 的 `Reference` 结构），不是 perworld Python
  `capture.json.recipe.cameras`，本卡不触碰。
- `scripts/capture_render_hfield.py` / `capture_render_sdf.py` 的 `recipe["cameras"]` 分别写各自的
  capture.json（独立副本），本卡不触碰。

## 4. 判定

- 无任何消费者断言「恰好 8 键/封闭集合」；从 8 到 10 的扩展对现有消费者是**纯增量**。
- 本卡未触发「tests/ 需改 → 停手报 Chief」条件；`tests/`、`src/` 零 diff 可独立复核。
