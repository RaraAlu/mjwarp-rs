# QA 复核：W4-B0 依赖治理与派工片拆分（`279db5c8`，卡 G）

- 任务：`2026-11-11-w4-b0-dependency-governance-check`（owner QA）。复核对象：提交 **`279db5c8`**（内容）+ **`cc43c2cd`**（登记/start：卡 G + 7 张实体卡）+ 卡 `2026-11-11-w4-b0-dependency-governance`。零 GPU、只读复核。
- 基线：`875ae41d`（NP5 落仓后）。

## 结论

**复核通过（0 阻断，1 低 + 2 观察）。** 7 张 W4 实体卡登记正确（`status=proposed`、`checks=[]`、id 与计划表 E1–E7 一一对应、owner Backend、写域与计划表一致）；计划文档 `W4-dependency-governance.md` 8,561 B/`fd91bc8f…`、首行「草案；裁决项未定，不构成结论」、B0-1/2/4 明确「待 Chief 裁决/本卡未执行」、无任何结论性表述；两个提交**只写 `.normify` 与 `docs/development-evidence/plan/**`**，无代码/判据改动。低项：文档 §3 片数口径「24 片 = B0 4 + 实体 20」与表内实际 **25 行（4 + 7×3=21）** 不一致。

## 一、卡 G 本身（✅）

- 卡 G：`status=verified`、12 个检查 = 4 门禁 + `unchanged src/ tests/ fixtures/ scripts/` + **7 条 `json /status == proposed`**（对 7 张登记卡的守卫，防误启动/误改态，做法好）。
- 四门禁（`.normify/runs/2026-11-11-w4-b0-dependency-governance/`）：fmt/clippy/clippy-cuda exit 0、unit **1,406/0/402**。
- 写域：卡 G `schedule.files = [.normify/changes/, docs/development-evidence/plan/]`；两次提交实测只动这两处（见 §三）。

## 二、7 张实体卡登记（✅）

| 卡 | status | checks | deps | 写域 |
| --- | --- | --- | --- | --- |
| `…-render-p8-entry-admission`（E1） | proposed | [] | 空 | 证据目录 |
| `…-render-m03-p8-asset-conversion`（E2） | proposed | [] | 空 | 证据 + `tests/` |
| `…-render-r10-p8-batch-scheduling`（E3） | proposed | [] | 空 | `src/render/` + `tests/` + 证据 |
| `…-render-r10-p10-batch-scheduling`（E4） | proposed | [] | 空 | 同上 |
| `…-render-m10-p10-rust-callback`（E5） | proposed | [] | 空 | `src/render/` + 证据 |
| `…-render-p8-base-admission`（E6） | proposed | [] | 空 | 证据目录 |
| `…-render-p8-admission`（E7） | proposed | [] | 空 | 证据目录 |

- id/优先级/phase（P8-渲染与BVH）/owner（Backend）与计划表 §2 的 E1–E7 **逐项对应**；每张卡 acceptance 只**引用**占位卡口径（如 `replication-plan.md:265-276`、`compat matrix` 行号）并明写「**不新增判据**」「**当前 checks=[]，禁止直接 start**」「不得宣称全部渲染通过/等价」。
- `depends_on` 全为空是**刻意**的：B0-1（r07-p8 处置）/B0-2（g10-p5 改指）/B0-4（NP note）尚待 Chief 裁决，卡 note 写明「依赖边/blocked_by 的最终改指与 waive 需 Chief 裁决，本卡不伪造解除」；`checks=[]` 阻止直接开工（见观察 O1）。

## 三、计划文档与边界（✅）

- `docs/development-evidence/plan/W4-dependency-governance.md`：**8,561 B / `fd91bc8fba89d38e691a973e8605a4b8b2f16efb208594122a8e326356b064c9`**（与 Chief 派工值一致），83 行、UTF-8/LF（`git ls-files --eol` = `i/lf w/lf attr/-text`）、无 BOM/CR/NUL。
- 首行标题「W4-B0 依赖治理与 W4 派工片拆分（草案；2026-11-11）」+ 引用块**「草案；裁决项未定，不构成结论」**，明写「先决条件只认 verified」「`checks=0` 占位卡不能开工」「阶段准入 ≠ 发布声明」。
- B0 四项：B0-1/2/4 状态列 **「待 Chief 裁决（本卡未执行）」**，B0-3 排在 B0-1/2 之后；§5 明写「未执行 B0-1/2/4……只列待裁点，**不伪造解除、不写 waived 依据**」。
- **结论词 grep**：已裁决 0、已签 0、已批/已批准 0、已执行 0、已解除 0、已waive 0、发布通过 0；「准入通过」仅 1 处为**否定句**（§5「不宣称 P8 准入通过」）。
- 边界：`git diff 875ae41d..279db5c8` 只含 `.normify/{changes,runs}/…w4-b0-dependency-governance`、7 张实体卡 JSON、`docs/development-evidence/plan/W4-dependency-governance.md`；**无** `src/`/`tests/`/`fixtures/`/`scripts/` 改动（两侧提交均如此），guards/冻结件未动；不改判据/参考/容差；零 GPU、零窗。

## 四、低项 / 观察

- **低 L1（片数口径不一致）**：§3 标题与卡/提交信息都写「**24 片 = B0 4 片 + 实体 20 片**」，但表内实际为 **25 行**：B0 4 片 + 实体 7 张×3 片 = **21 片**（W4-01A…W4-07C 各 A/B/C 齐全）。我逐行数过 `| W4-` 行 = 25。建议更正标题/统计（若某片不计入需说明，例如 E6/E7 合并计窗但仍各是 1 片），或加一句「窗口可合并、片数不合并」。
- **观察 O1**：7 张卡 `depends_on` 为空 + `checks=[]`：这是「先登记、待 B0 裁决后补机器依赖/检查」的刻意设计，卡 note 已说明；但板面上它们会以 proposed 出现，建议在 B0-1/2/4 裁决后第一时间补 `depends_on`/`checks`，避免被误当可派工。
- **观察 O2（正向）**：卡 G 用 7 条 `json /status == proposed` 检查把「登记卡不得被顺手启动/改态」变成机器可验证约束，做法值得沿用（后续 H 卡可照此加同类守卫）。

## 五、口径

不宣称 P8 准入通过/全部渲染通过/等价/发布；E6/E7 阶段准入 ≠ 发布声明；本复核只核卡登记正确性、写域与文档口径；不改判据/参考/容差；零 GPU。
