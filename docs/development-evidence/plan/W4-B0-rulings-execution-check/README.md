# QA 复核：W4-B0 裁定执行（`9830bb76`，卡 I）

- 任务：`2026-11-11-w4-b0-rulings-execution-check`（owner QA）。复核对象：提交 **`9830bb76`** + 卡 `2026-11-11-w4-b0-rulings-execution`。零 GPU、只读复核。
- 基线：`7f26192f`（卡 I 登记/start）；被执行的裁定来自 `7f26192f` 记的 B0-2/B0-4/B0-1(c)/观察1/B0-3。

## 结论

**复核通过（0 阻断，3 低 + 2 观察）。** 五项执行事实全部成立：g10-p5 已改指 **verified** 的 `2026-11-04-g10-p5-family-closeout`（acceptance 逐字未动）；p8-entry/p8-base/r10-p8 三卡 note 已写「NP5-01/02/03 verified（不是整个 NP 伞卡）…不提前标 NP 通过」且**无「已通过/已签/已裁决/已批」措辞**；r07-p8 依赖边**保留未删**并只登记 blocked-by 事实（无 waive、无伪造解除）；7 张实体卡依赖边与计划表 §2 前置列一致且全部指向真实存在卡片；板面 `normify status` **0 错 0 警**、`validate` **0 错误**（另 5 条架构告警为既有存量，与卡 I 无因果）。

## 一、B0-2（g10-p5 改指）✅

- `2026-10-11-roadmap-g10-p5.schedule.depends_on`：`[g07-p5, g08-p5]` → **`[2026-11-04-g10-p5-family-closeout]`**（我核该卡 `status=verified`）；`status` 保持 `proposed`；`acceptance` 我按 JSON 语义 diff 逐键比对**未变**；note 追加「B0-2（Chief 2026-11-11 批准执行，卡 I）……只改 depends_on/note，不动 acceptance」。
- 语义 diff（old vs new JSON 展平）：g10-p5 仅 `/note` 与 `/schedule/depends_on` 两处变化，其余全等。

## 二、B0-4（NP 门禁联动 note）✅

- `p8-entry` / `p8-base` / `r10-p8` 三卡 note 均追加：「前置 = **NP5-01/02/03 verified**（不是整个 NP 伞卡）；NP5-00 只是清单冻结、**不解除该前置**；**不提前标 NP 通过**」。
- 三卡 note 逐字检查：命中「NP5-01/02/03 verified」；未命中「已通过」「已签」「已裁决」「已批」。

## 三、B0-1（r07-p8 事实登记，不 waive）✅

- `r10-p8` 与 `p8-base` 的 note 追加「B0-1（Chief 裁定 (c)，卡 I 只登记事实）：blocked-by-r07-p8（当前 `status=proposed`）；**不 waive、不伪造解除**；等 R07 收口 verified 或用户明确 waive 后再改指。depends_on 里的 r07-p8 边不删除」。
- 我核两卡 `depends_on`：**`2026-10-11-roadmap-r07-p8` 仍在**（r10-p8 9 条依赖含 r07-p8；p8-base 7 条依赖含 r07-p8）；r07-p8 卡 `status=proposed`（真阻塞）；全仓无新增 waive 记录。

## 四、观察 1（7 张实体卡补依赖）✅

- 我逐卡核（目标卡存在性/状态）：
  - `p8-entry-admission` → `np5-00-gate-ledger-freeze-r2`（verified，NP 门禁**可解析代理**）
  - `m03-p8-asset-conversion` → `r01-p8`（verified）
  - `r10-p8-batch-scheduling` → `r07-p8`(proposed，真阻塞) + `np5-00-r2`(verified) + r01–r05/r08/r09-p8（全 verified）＝9 条
  - `r10-p10-batch-scheduling` → `r10-p8-batch-scheduling`（proposed，E3）
  - `m10-p10-rust-callback` → `m10-p7`（verified）
  - `p8-base-admission` → `r07-p8` + E1–E5 + `np5-00-r2` ＝7 条
  - `p8-admission` → `p8-base-admission` + `m03-p8`(均 proposed) + `g28-p8`(verified) + `r06-flex-closeout`(verified) ＝4 条
- 全部依赖目标**存在**，无死依赖；`status` 仍 `proposed`、`checks=[]`（不标可开工）；note 追加「登记态待补 checks，不标可开工；NP5-00 只是清单冻结，NP5-01/02/03 verified 才是硬前置（卡待登记，登记后由 B0-3 复核改指）」。
- 代理边口径诚实：§6 明写「不伪装成 NP 门禁已满足」（见观察 O2）。

## 五、B0-3 复核 addendum / 板面 / 边界 ✅

- 计划文档 append-only：**8,561 → 13,457 B，共同前缀 8,561 = 旧长度**（纯追加）；新 sha **`7c54ea954b66423ea8174131ffe85c3a9d9c8cda266aa7adb19a07c502fc4f46`/13,457 B**，与 Chief 派工值一致；追加 §6 执行记录 + §7 B0-3 复核 + §8 非声明。
- 板面：`normify status` 错误 0/警告 0（与 §7 快照一致口径）；我另跑 `normify validate` = **0 错误 / 5 警告**（`api/leaf-empty`×2、`structure/leaf-too-coarse`、`layout/order-incomplete`×2）——均为模块结构既有告警、与本次纯治理改动无因果（见观察 O1）。
- 边界：提交 17 files = 12 个卡 JSON（4 个 roadmap 占位 + 7 个实体卡 + 卡 I）+ 4 门禁记录 + 计划文档；**无** `src/`/`tests/`/`fixtures/`/`scripts/` 改动、guards/冻结件未动、不改判据/参考/容差；零 GPU、零窗。
- 卡 I：`status=verified`；13 检查 = 4 门禁 + `unchanged src/tests/fixtures/scripts` + **8 条 `json …/depends_on/0`** 守卫（把改指结果机器可验证）。

## 六、低项 / 观察

- **低 L1（记录完整性）**：11 张被改内容的卡 `updated_at` **均未刷新**（如 g10-p5 仍为 2026-10-23T04:57、p8-entry 仍为 2026-10-19T09:48），与「卡已修改」事实不符；建议后续治理脚本在改字段时同步 `updated_at`（或说明 normify 不维护该字段、以提交时间/历史为准）。
- **低 L2（diff 噪声）**：4 张 roadmap 卡被**整文件重排**（PowerShell `ConvertTo-Json` 的 4 空格风格；语义仅 note/depends_on 变化，但 diff 达 87–106 行/卡，全提交 +294/−204）。字节仍 UTF-8/无 BOM/CR0/NUL（我已核 4 个 blob），但审计成本高；建议改用最小编辑（保留原 2 空格格式）。
- **低 L3（措辞不精确）**：计划文档 §6 B0-1 行写「**全仓无 waive 决策记录**（仅说明性文字）」——板面实际存在 **5 条既有豁免**（如 `g01-acceptance-fast-loop` 等，与本卡无关）；应改为「**本次无新增 waive**（r07-p8/g10-p5 均未 waive）」。
- **观察 O1**：`normify status` 的「警告」与 `normify validate` 的「警告」是两套口径——本卡 §7 用 status（0/0）正确，但读者若跑 validate 会看到 5 条既有架构告警；建议后续证据标明「status 口径」。
- **观察 O2（诚实性 + 风险）**：NP5-01/02/03 三卡**尚未登记**，实体卡用已 verified 的 NP5-00 r2 作「可解析代理边」；文档明写「不伪装成 NP 门禁已满足」，但板面上 E1/E3/E6 的 blocked_by 只有代理边，**真实硬前置（NP5-01/02/03 verified）尚未满足**——建议三卡登记后立即由 B0-3 改指，避免代理边被误读为门禁满足。

## 七、口径

不宣称 NP/P8/W4 准入通过、不宣称全部渲染通过/等价/发布；不改判据/参考/容差/acceptance；不改 r07-p8 依赖边（只登记事实）；零 GPU、零窗。
