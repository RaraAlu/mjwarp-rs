# R10-S2 复核观察收口 + 计划同步（纯文档）

任务：`2026-10-28-render-bvh-r10-s2-obs-closure`（P8 渲染与 BVH；纯文档治理片，无 GPU 窗口、无捕获）
来源：QA 独立复核 `2026-10-28-render-bvh-r10-s2-check`（verdict pass / 阻断 0 / 观察 4，均建议级），提交 `a3ce62e1` + `5cc6a3d5`
判据树：本任务 start 时 HEAD `1ad2f268`（R09-S2 收口后）
口径：**不改**任何窗口数字、判据、容差、参考、夹具；不重开窗口、不重跑 GPU；不宣称验收通过。

## 1. 四条观察的处置

| # | 观察内容 | 处置 | 落地 |
| --- | --- | --- | --- |
| O1 | `48ea01a2` 用 `git add -A '.normify'` 把 4 张登记卡（r09-s2 / r09-s2-check / r10-s2-check / per-camera-rgb-gap）一并入库（内容未改） | **accept**（不改历史） | 规范条目：此后提交一律显式列路径，禁用 `git add -A` / `git add .` |
| O2 | 证据 `manifest.txt` 自登记一行 `3a5a3c4b… 8325 manifest.txt`，而文件实为 8286 B —— 自哈希无法自洽且数值过时 | **fix** | 删除自引用行（改为注释说明「本清单不自登记」）；其余 36 行按实数重算并逐行校验 |
| O3 | README §1 写「日志 `mismatched=19074`」，而 attempt1 日志原文是断言 `left: 19074` / `right: 0`（无 `mismatched` 字面） | **fix 措辞** | README §1 改为日志原文 `left: 19074` / `right: 0`，并注明 19074 = ULP≠0 像素数、非超容差像素数；根因叙述不变 |
| O4 | 卡面要求「上游 nworld>1 数值**未覆盖**」字面，证据只有语义等价的「不对上游 nworld>1 做任何数值结论」 | **fix 补字面** | README §4 标题改为「上游 nworld>1 数值未覆盖」，保留原语义句 |

O2/O3/O4 均不改任何数值、判据、容差、参考与夹具；attempt1 / attempt2-exit9 / attempt2-run 三段记账与 `target/` 原始产物一字未动。

## 2. 证据自洽性复算（O2 修复后）

- `manifest.txt` 登记规律：`sha256 bytes path`，只登记文件、**不自登记**；
- 本任务提交时对 manifest 全部 36 行逐行复算：sha256 与 bytes 全部 match（本地校验脚本输出 `MANIFEST OK: all rows match`）；
- `attempt2-exit9/` 两项仍为 `facts.txt` 61 B / `069d142d…`、`guard.log` 51 B / `6c5acd89…`（与 target 原件一致，未删未改未补造）。

## 3. plan.json 同步（R09/R10 切片）

新增切片条目（状态对板上实事）：

| 切片 | 状态 | 依据 |
| --- | --- | --- |
| R09-S1 lighting-params | delivered | `ef557f6e` + `0a72c653`；QA `df804751`/`354c7808`；治理 `2f1bfd5a`/`4887402d` |
| R09-S2 lighting-shadow-kernel | in_progress | 实现 `6b5d6dc6` + 收口 `1ad2f268`；QA 复核卡 `…-r09-s2-check` 进行中（本任务登记时） |
| R10-S1 multi-world-state | delivered | `4cf1159c`/`14ff2536` → `0398c23c`/`879cd012`；复核 `ac1787ab`/`34ac8d1b` |
| R10-S2 multi-world-e2e | delivered | `504ded3c`/`01e0e73c`/`40e9238f` → 证据 `f0e400aa`/`48ea01a2`；QA `a3ce62e1`/`5cc6a3d5` |

`r01_p8_mapping`：`shadows_skybox_splats_flex_hfield` 由 `deferred` 改为 `partial`（R09-S1 delivered、R09-S2 in_progress；天空盒/泼溅/柔体/高度场仍 deferred），新增 `multi_world` = delivered（含 attempt2-run 8/8、floor 9708、守卫 46/46、零编译、上游 nworld>1 数值未覆盖、单份几何/灯位姿 backlog 引用）。**未改任何切片范围、判据或容差**（plan.json 自带的 rolling-table policy）。

## 4. 不变量与边界

- 零代码改动：`src/**`、`tests/**`、`.cu`、`fixtures/**`、`Cargo*` 未动；
- 不宣称验收通过；不写「窗口通过 / 已覆盖 / 精度达标 / 上游等价」；
- 不重开 R10-S2 窗口、不重跑 GPU、不改 attempt 记账。

## 5. 为什么不用 `normify fix`

`normify fix 2026-10-28-render-bvh-r10-s2-multi-world --reason …` 会追加 addenda，并作废该任务 4 条门禁结果（需按新定义重跑）。本批改动发生时工作树正被 R09-S2 实现占用（`src/render/kernel/` 多文件在改），此时重跑门禁会绑到半成品树，因此改用本独立治理任务承载同一批文档改动：门禁在本任务自己的最终树上重跑，改动范围与验收在卡面写明。

## 6. 门禁与提交

- `normify gate 2026-10-28-render-bvh-r10-s2-obs-closure`（绑最终树）：见 receipt 与 `.normify/runs/2026-10-28-render-bvh-r10-s2-obs-closure/`；
- 提交显式列路径（不 `git add -A`）。
