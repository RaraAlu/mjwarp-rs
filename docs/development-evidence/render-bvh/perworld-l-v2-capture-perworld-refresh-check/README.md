# P0-1 每 world 场景刷新 QA 只读复核（refit_bvh 紧前于 render）

**结论：PASS（0 阻断，0 观察）。**

- 任务：`2026-11-07-render-perworld-l-v2-capture-perworld-refresh-check`（owner QA、P8-渲染与BVH、p62；QA 自登记，依赖 P0-1 修卡）
- 复核目标：HEAD **`e34df3b7`**（修卡 `verified`；登记 `27bbd8fb` → start `ff5cba9e` → 实现 `e34df3b7`），基线 `edc6019b`
- 复核方式：只读 + 独立复核（独立 AST 检查、执行方断言器重跑、上游 pinned 源码核对、同树旧/新脚本默认 dry-run 对比、`--worlds 1` 两版本对照、证据日志复跑、行数/manifest 独立重算）；QA 脚本自检 **16/16 PASS**；CPU-only、零 GPU、不采集
- 本卡登记提交：见 `git log`（`...refresh-check` 登记提交）

## 1. 结论

P0-1 的改动与声明全部成立：`refit_scene_before_render` 位于 `mjw.render` **紧前一条语句**、每次渲染前恰一次、不重建 context/不重编 megakernel；断言器可复跑（10/10，含 7 条负例）；根因引用与 pinned 上游源码逐条对上；默认行为与 `--worlds 1` 行为不变；行数 500/244/160、门禁 unit 1359/0、边界零改动、证据 manifest 18/18 自核。**无阻断、无观察。**

交叉说明（信息项，不影响本卡结论）：本修复解决的是「采集脚本未在 render 前刷新场景 BVH/快照」；QA 另卡的只读复核（`2026-11-07-render-perworld-l-v1-reference-discriminability-check`，`a654c2a7`）实测 cam-persp 视野内只有 world 不变的地板（参考 seg 仅 floor + 背景）——因此 **P0-1 是必要修复，但单靠它不能让 cam-persp-only 参考获得 depth/seg 判别力**；P0-2 pilot 若仍 `non-discriminating`，属该卡 P0-4 预期分支（另起诊断卡，不在本卡重试/放宽）。

## 2. 逐项结果

### 2.1 改法（PASS）

- `scripts/capture_render_perworld.py::render_images`：`mjw.render(m, d, rc)` 的**直接前一条语句**是 `refit_scene_before_render(m, d, rc)`（AST 复核：无 `if`/`for` 包裹；`render_images` 内 `mjw.render` 恰 1 次、refit 调用恰 1 次）。
- helper `scripts/perworld_capture_meta.py::refit_scene_before_render`：函数体内**无任何控制流**、恰 1 次 `mjw.refit_bvh(m, d, rc)`（实参顺序 m,d,rc）；由主脚本 `from perworld_capture_meta import ... refit_scene_before_render ...` 引入。
- **覆盖面**：AST 计数 `capture_once` 内调用 `render_images` **1 次**；`main` 内调用 `capture_once` **2 次**（行 443/444），且都在 `create_render_context`（行 442）之后 ⇒ 首遍 + `--repeat 2` 第二遍的每次 render 前均恰一次 refit；不重建 context、不重编 megakernel（diff 仅 +1 行调用 +14 行 helper）。
- 执行方断言器重跑：exit 0、**SUMMARY PASS 10/10**（3 正向 + 7 负例：缺 refit / refit 在 render 后 / `if` 包裹 / 重复 / helper 控制流 / helper 实参顺序 / 上游签名不符，全部按预期拒绝）；证据日志含 `exit_code: 0`。

### 2.2 根因引用（PASS）

pinned 上游 `target/mjwarp-parallel/upstream/mujoco_warp`（71da24d9）实测：

- `_src/render_util.py` 的 `create_render_context` 内确有 `MjData(` 与 `mj_forward(`（实测行 **403/404**）→ 场景 BVH 建自创建时刻 CPU 快照；
- `_src/bvh.py`：`refit_bvh(m, d, rc)`（定义行 39，实参顺序 m,d,rc）+ `refit_scene_bvh`（行 382）体内使用 `(d.nworld, rc.bvh_ngeom)` 并读 `d.geom_xpos`/`d.geom_xmat` → 一次 refit 覆盖全部 world；
- `__init__.py` 导出 `refit_bvh`。

### 2.3 默认 / 单 world 行为（PASS）

- **同树旧/新脚本对照**（旧脚本 `edc6019b`/499 行临时置回真实路径运行后 `finally` 恢复并校验 sha）：默认 dry-run 逐叶 diff 仅 `/recipe/script/sha256`、`/recipe/script/bytes`；**added=[]、removed=[]**、`legacy_manifest_entries` 两侧 107 → 默认行为只多了脚本指纹（dry-run 不走渲染路径，refit 不进入 plan）。
- 被复核 before/after 原始日志按同一命令行复跑：**逐叶完全一致**（0 added/removed/changed）。
- **`--worlds 1`**：两版本均 exit 1（`--worlds must be >= 2`）、stderr 逐字相同、stdout 空、未创建 out_dir → 行为无变化。
- 单 world refit 幂等语义仅按证据说明记录；本卡不宣称像素级结论（留 pilot）。

### 2.4 行数 / 门禁 / 边界 / 证据（PASS）

- 行数（git blob）：`scripts/capture_render_perworld.py` **500**（基线 499）、`perworld_capture_meta.py` **244**（基线 230）、`perworld_light_prep.py` **160**（未改），均 ≤500。
- `git diff edc6019b..e34df3b7` = 恰 **27 路径**（P0-1 卡 + Arch 预登记的 L v1 复核卡 + 4 gate runs + 19 证据文件 + 2 脚本）；`fixtures/`、`src/`、`tests/`、spec 目录**零 diff**；未采集、无参考写入。
- 被复核卡 `status=verified`、`revision.before=27bbd8fb…`、unit gate **1359 passed / 0 failed / 400 ignored**。
- 证据 manifest 首行 `#`、**18/18** 条目复算通过、不自列（19 文件）；19 文件 UTF-8/LF/无 BOM/裸 CR·TAB 0；`artifacts/scripts.diff` 与实际 `git diff` 逐字一致；`artifacts/default-dryrun-diff.json` unexpected=0；`artifacts/worlds1-behavior.json` identical=true；`call-order-assertion.log` 带 exit_code。
- README/receipt 的 G-L1 依赖（本卡 verified + QA pass + **P0-2 pilot pass-margin**）与非声明（不宣称判别力恢复/采集有效/灯验收/上游等价/数值结论）在位。

## 3. 观察清单

无（0 阻断、0 观察）。信息项见 §1 交叉说明。

## 4. 证据清单

| 路径 | 说明 |
| --- | --- |
| `scripts/verify_refresh_review.py` | 复核脚本（16 项只读复算，可复跑） |
| `logs/verify-refresh-review.json` | 16 项检查机器可读结果（PASS 16/16） |
| `logs/verify-refresh-review.log` | 脚本原始输出 |
| `artifacts/scope.json` | 变更面 27 路径 + 零越界探针 |
| `artifacts/line-counts.json` | 500/244/160 与基线 499/230 |
| `artifacts/ast-checks.json` | 独立 AST：refit 紧前、单次、helper 契约、覆盖计数 |
| `artifacts/call-order-assertion.json` | 执行方断言器重跑 10/10 |
| `artifacts/upstream-citations.json` | pinned 上游根因引用核对（行号/签名/维度/字段） |
| `artifacts/default-behavior-diff.json` | 同树旧/新脚本默认 dry-run 逐叶 diff |
| `artifacts/evidence-logs-reproduced.json` | 被复核 before/after 日志复跑比对 |
| `artifacts/worlds1-behavior.json` | `--worlds 1` 两版本一致拒绝 |
| `artifacts/evidence-selfcheck.json` | manifest 18/18（19 文件）、编码、diff/artifacts 一致性 |
| `artifacts/reviewed-card.json` | 被复核卡状态 + unit 1359/0 |
| `README.md` / `receipt.json` | 本报告与机器可读回执 |
| `manifest.txt` | 本目录 sha256 + bytes（首行 `#`，不自列） |

## 5. 边界与非声明

- 只读复核 + 只写本卡证据目录；不采集、不开窗、不跑 GPU、不改被复核脚本/证据、既有证据、代码、fixtures。
- 复核期间为控制变量曾临时把旧脚本内容写回真实路径并在同进程 `finally` 恢复（sha 校验一致），最终工作树 `scripts/` 与 `fixtures/` 零改动。
- 本复核不产参考、不宣称判别力恢复、不宣称采集有效、不宣称 per-world 灯验收、不宣称与上游实现等价、不宣称任何渲染数值结论；G-L1 仍需 P0-2 pilot `pass-margin` 后才可正式采集。

## 6. 复现

```powershell
cd D:\Rust\mjwarp-rs
target/mjwarp-parallel/venv/Scripts/python.exe -B docs/development-evidence/render-bvh/perworld-l-v2-capture-perworld-refresh-check/scripts/verify_refresh_review.py
# 期望末行：SUMMARY PASS 16/16
```
