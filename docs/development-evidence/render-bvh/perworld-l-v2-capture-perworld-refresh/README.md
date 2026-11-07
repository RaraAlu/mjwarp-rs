# perworld L v2 采集脚本 P0-1：每 world 场景刷新（`mjw.refit_bvh` after forward, before render；零 GPU）

- 任务：`2026-11-07-render-perworld-l-v2-capture-perworld-refresh`（owner Backend、p62、P8-渲染与BVH）
- 定位：**最小候选修复**（零判据改动、零 GPU、不采集、不开窗）；修复是否解决判别力由后续 **GPU pilot**（Chief token）判定，本卡**不宣称**判别力恢复。
- 来源：G-L1 材料期 P0 发现（`2026-11-07-render-perworld-l-v2-capture-g-l1-material` README §0.2 / §7 P0-1）：既有 L v1 参考 `qpos/geom_xpos` 逐 world 不同、`rgb/depth/seg` 三 world **逐位完全相同**（判别力脚本判 `non-discriminating`）。

## 1. 结论与改动

- **最小改法已落地**：`scripts/capture_render_perworld.py::render_images` 在 `mjw.render(m, d, rc)` **紧前一句**调用
  `refit_scene_before_render(m, d, rc)`；helper 位于**已有**模块 `scripts/perworld_capture_meta.py`：
  ```python
  def refit_scene_before_render(m, d, rc) -> None:
      import mujoco_warp as mjw
      mjw.refit_bvh(m, d, rc)
  ```
- **覆盖每一次渲染**：`capture_once` 只调用 `render_images` 一次；`main` 调用 `capture_once` **两次**（首遍 + `--repeat 2` 第二遍），
  因此每遍采集的每次 render 前都恰好一次 refit；**不重建 context / 不重编 megakernel**。
- **根因（pinned 上游源码，CPU 侧证明）**：
  - `create_render_context` 内部 `mjd = mujoco.MjData(mjm); mujoco.mj_forward(mjm, mjd)`（`_src/render_util.py:403-404`）
    → 场景 BVH 用**创建时刻的 CPU 快照**建树；
  - `refit_scene_bvh` 以 `dim=(d.nworld, rc.bvh_ngeom)` 发射 bounds kernel、读 `d.geom_xpos`/`d.geom_xmat`
    （`_src/bvh.py:382-402`）→ **一次 refit 覆盖全部 world**（无需逐 world 循环，也无需重建 context）。
  - 与 R06 多帧 H1 同型：render 自身不 refit，需在 render 前显式刷新。

## 2. 调用序静态断言（机器可复核）

- 脚本：`scripts/verify_refresh_call_order.py`（AST/text，**零 GPU**）；结果：`logs/call-order-assertion.log` → **SUMMARY PASS 10/10**。
- 正向检查：① `render_images` 内恰 1 次 `mjw.render`，其**直接前一条语句**是 `refit_scene_before_render(...)`（无 if/for 包裹）；
  ② `capture_once` 恰调用 `render_images` 1 次、`main` 恰调用 `capture_once` 2 次且都在 `create_render_context` 之后；
  ③ helper 体内无控制流、恰 1 次 `mjw.refit_bvh(m, d, rc)`（实参顺序 m,d,rc）且从 `perworld_capture_meta` import；
  ④ pinned 上游 `__init__.py` 导出 `refit_bvh`、`_src/bvh.py` 定义 `refit_bvh(m, d, rc)`、`refit_scene_bvh` 用
  `(d.nworld, rc.bvh_ngeom)` 并读 `d.geom_xpos/xmat`。
- 负例（同一断言器必须拒绝）：缺 refit、refit 在 render 之后、refit 被 `if` 包裹、重复 refit、helper 带控制流、
  helper 实参顺序错、上游签名不符 —— 7 条全部按预期拒绝（见日志）。

## 3. 默认 / 单 world 行为对照（零 GPU 可核部分）

- **默认 dry-run**（`--dry-run --out-dir fixtures/render-bvh/scene-a/perworld/prep-default-dryrun`）：
  `artifacts/default-dryrun-diff.json` → `unexpected = 0`；允许差异仅 `/recipe/script/sha256|bytes`（脚本被编辑）。
  dry-run 不执行渲染路径，refit 不进入 dry-run plan。
- **`--worlds 1`**：两版本均**按设计拒绝**（`--worlds must be >= 2 (this is a per-world reference)`，exit 1、不建 out_dir），
  前后 stderr/exit/落盘完全一致（`artifacts/worlds1-behavior.json`）。
- **单 world 时 refit 语义**：若处于 nworld=1，`refit_bvh` 用 `forward` 后的 `d.geom_xpos` 重算 bounds，与建树同源、语义上幂等；
  真实像素级对照留 pilot，**本卡不宣称**。

## 4. 行数 / 棘轮

| 文件 | 前 | 后（git blob 换行） | 说明 |
| --- | --- | --- | --- |
| `scripts/capture_render_perworld.py` | 499 | **500** | 仅 +1 行调用；≤500 |
| `scripts/perworld_capture_meta.py` | 230 | **244** | 新增 helper（+14）；≤500 |
| `scripts/perworld_light_prep.py` | 160 | **160** | 未改 |

## 5. 边界与非声明

- **零判据/零产品面**：不改判据/阈值/容差/参考、不改 `src/`、不改 `fixtures/`、不改 spec/receipt/manifest 既有内容；
  不采集、不开窗、零 GPU、不落参考。
- **G-L1 依赖**：G-L1 正式采集**依赖本卡 verified + QA pass + P0-2 pilot 判别力 `pass-margin`**；本卡只实现「每 world 场景刷新」，
  pilot 与正式采集不在本卡。若 pilot 仍 `non-discriminating`，按材料 P0-4 另起诊断卡，不在本卡重试或改判据。
- **非声明**：本卡不宣称判别力恢复、不宣称既有 L v1 参考可修复、不宣称采集是否有效、不宣称 per-world 灯验收、
  不宣称与上游实现等价、不宣称任何渲染数值结论。

## 6. 证据清单

| 路径 | 说明 |
| --- | --- |
| `README.md` | 本文件 |
| `receipt.json` | 机器可读回执（`/task_id` 供 normify `json` 检查） |
| `scripts/verify_refresh_call_order.py` | 调用序/上游导出静态断言器（含 7 条负例） |
| `logs/call-order-assertion.log` | 断言原始输出（SUMMARY PASS 10/10，含 exit_code） |
| `logs/before-default-dryrun.json` / `logs/after-default-dryrun.json` | 默认 dry-run 前后原始输出 |
| `logs/before-worlds1-dryrun.*` / `logs/after-worlds1-dryrun.*` | `--worlds 1` 拒绝行为（前后一致） |
| `artifacts/default-dryrun-diff.json` | 默认行为逐字段 diff（unexpected=0） |
| `artifacts/worlds1-behavior.json` | `--worlds 1` 前后 exit/stderr 一致性 + 幂等语义说明 |
| `artifacts/scripts.diff` | 主脚本 + meta 模块改动 diff |
| `artifacts/perworld-refresh.json` | 根因引用、行数、上游导出核验、断言摘要 |
| `logs/line-counts.txt` | 三个 .py 行数（git blob 口径） |
| `manifest.txt` | 本目录 sha256 + bytes（首行 `#`，不自列） |

## 7. 复现

```powershell
# 调用序静态断言（零 GPU）
target/mjwarp-parallel/venv/Scripts/python.exe -B docs/development-evidence/render-bvh/perworld-l-v2-capture-perworld-refresh/scripts/verify_refresh_call_order.py
# 默认 dry-run 对照
target/mjwarp-parallel/venv/Scripts/python.exe -B scripts/capture_render_perworld.py --dry-run --out-dir fixtures/render-bvh/scene-a/perworld/prep-default-dryrun
# --worlds 1 拒绝行为
target/mjwarp-parallel/venv/Scripts/python.exe -B scripts/capture_render_perworld.py --dry-run --worlds 1 --hinge-deg 0 --out-dir fixtures/render-bvh/scene-a/perworld/prep-worlds1-dryrun
```
