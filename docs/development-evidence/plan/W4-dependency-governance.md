# W4-B0 依赖治理与 W4 派工片拆分（草案；2026-11-11）

> **草案；裁决项未定，不构成结论。** 本文件是 W4 的**计划 addendum**（不改 W0–W9 快照，不新增判据），
> 由卡 `2026-11-11-w4-b0-dependency-governance` 落仓。依据 Arch `target/render-bvh/gl2-ptx-diff/w4-b0-governance-cards.md`
> （`2de12570…`/8,871 B，**工作材料，非证据**）+ 板面实测 + 依赖图修复 `496c01e6`（死依赖 65→0）+ W5 计划（NP 门禁先行）。
> 口径：**先决条件只认 verified**；`checks=0` 占位卡不能开工；阶段准入 ≠ 发布声明。

## 0. 事实底稿（不变）

- W4 的 7 张占位卡（`r10-p8` / `r10-p10` / `m03-p8` / `m10-p10` / `p8-entry` / `p8-base` / `p8`）**全部 `checks=0`**，
  一张都不能直接 start。
- r01–r06/r08/r09-p8 已 verified；`r07-p8` 仍 `proposed(0)`，且同时卡 `r10-p8` 与 `p8-base`。
- 多张 W4 卡面明写「NP01 与 NP02 须先行」；NP5-00 证据卡已落仓（`875ae41d`，账本 190 条，只记状态）。

## 1. W4-B0 依赖治理四片（其中 B0-1/2/4 **待 Chief 裁决/执行**，本卡只列待裁点）

| 片 | 动作 | 状态 | 验收 |
| --- | --- | --- | --- |
| B0-1 | **r07-p8 处置**（二选一）：(a) R07 家族收口卡 verified 后把 r10-p8/p8-base 的 blocked_by 改指该收口卡；(b) 按板面 `conservative-stage-aggregation` 口径由 Chief 裁决 `waive` 并登记 waived 依据（日期/裁决人/理由）。两者都不成立时只登记 `blocked-by-r07-p8` 事实，**不伪造解除** | **待 Chief 裁决**（本卡未执行） | 板面 0 错误；blocked_by 只剩真实待办实体卡或「明确 waived + 依据」 |
| B0-2 | **g10-p5 伞卡改指**：`2026-10-26-g10-p5`（proposed,0）改指已 verified 的 `2026-11-04-g10-p5-family-closeout`（只改依赖/note，不动 acceptance） | **待 Chief 裁决**（本卡未执行） | W4 卡不再引用空壳伞卡 |
| B0-3 | **依赖图全量复核**：7 张 W4 卡 + p8 相关卡 blocked_by 逐条解析到 verified 卡/waived；跑依赖检查 | 依赖边未改（B0-1/2 未决），复核排在其后 | 0 错误 0 警告；复核结果追加为本文件 addendum |
| B0-4 | **NP 门禁联动 note**：在 p8-entry/p8-base/r10-p8 的 note 写明前置是「NP5-01/02/03 verified」（不是整个 NP 伞卡） | **待 Chief 裁决**（本卡未执行；7 张新卡 acceptance 已按此口径登记） | note 可复核；不提前标 NP 通过 |

## 2. 7 张实体卡登记表（已登记：status=proposed、checks=[]，启动前须补机器检查项）

| # | 实体卡 id | 对应占位卡 | acceptance 摘要（只引用占位卡，不新增判据） | 前置（登记口径） | 写域 | 窗 |
| --- | --- | --- | --- | --- | --- | --- |
| E1 | `2026-11-11-render-p8-entry-admission` | `p8-entry` | 适用切片与矩阵核对（`replication-plan.md:107-108,273`）+ 范围内 NP 门禁 + 双模式平台证据 | NP5-01/02/03 verified（否则只记「等待 NP 门禁」） | `docs/development-evidence/render-bvh/**` | 1 |
| E2 | `2026-11-11-render-m03-p8-asset-conversion` | `m03-p8` | 单位/路径/内容摘要三项（`compat matrix` 行 295 附近；R01 资产路径） | r01-p8 verified | `docs/.../render-bvh/**`、`tests/**` | 0（触发设备 +1） |
| E3 | `2026-11-11-render-r10-p8-batch-scheduling` | `r10-p8` | 独立缓冲与完成令牌契约（`replication-plan.md:265-276`；`compat matrix` 行 324）+ Windows 实体验收 | NP 门禁 + B0-1 处置；r01–r05/r08/r09-p8 verified | `src/render/**`、`tests/**`、证据 | 1 |
| E4 | `2026-11-11-render-r10-p10-batch-scheduling` | `r10-p10` | 同 E3 + P10 差异；family closeout 可作前置引用 | E3 verified + NP 门禁 | 同 E3 | 1 |
| E5 | `2026-11-11-render-m10-p10-rust-callback` | `m10-p10` | 时序/状态/同步三条契约（`compat matrix` 行 302） | m10-p7 verified | `src/render/**`、证据 | 0（涉设备时序 +1） |
| E6 | `2026-11-11-render-p8-base-admission` | `p8-base` | 基础批量渲染准入（8 条 r-p8 verified + batched-device-transfers verified + r07-p8 处置） | B0-1 + E1/E2/E3/E4/E5 对应范围 verified + NP 门禁 | 证据目录 + 准入核对 | 1（可与 E7 合并） |
| E7 | `2026-11-11-render-p8-admission` | `p8` | 全部渲染与查询准入（p8-base/m03-p8/g28-p8 前置；r06-flex-closeout verified） | E6 verified + E2 verified + g28-p8 verified + r06-flex-closeout verified | 证据目录 + 准入核对 | 1（可与 E6 合并） |

> 7 张卡 acceptance 均引用占位卡 acceptance、**未新增判据**、**未改占位卡**；`checks=[]` 表示登记态，**禁止直接 start**。

## 3. 派工片拆分（24 片 = B0 4 片 + 实体 20 片）

| 片 | 卡 | 范围 | 窗 | 前置 | 验收 |
| --- | --- | --- | --- | --- | --- |
| W4-B0-1 | B0 | r07-p8 处置（改指/waive/登记 blocked） | 0 | — | **待 Chief 裁决** |
| W4-B0-2 | B0 | g10-p5 伞卡改指 family closeout | 0 | — | **待 Chief 裁决** |
| W4-B0-3 | B0 | 依赖图全量复核 + addendum | 0 | B0-1/2 | 0 错误 0 警告 |
| W4-B0-4 | B0 | NP 门禁联动 note（NP5-01/02/03） | 0 | NP5-00 已落仓（`875ae41d`） | note 可复核 |
| W4-01A | E1 | 适用切片与矩阵核对（只读） | 0 | — | 切片清单 + 矩阵覆盖 + 缺口 |
| W4-01B | E1 | 双模式平台证据（debug/release） | 1 | W4-01A + NP5-01/02/03 verified | 两模式证据齐 + monitor/floor 记录 |
| W4-01C | E1 | 证据卡 + QA 复核 | 0 | W4-01B | README/receipt/manifest + QA PASS |
| W4-02A | E2 | 资产转换契约与矩阵核对 | 0 | — | 单位/路径/摘要口径无歧义 |
| W4-02B | E2 | 单位换算/路径解析/内容摘要实现或核对 | 0（触发设备 1） | W4-02A、r01-p8 | 三项逐条证据 |
| W4-02C | E2 | 证据 + QA | 0 | W4-02B | 同上 |
| W4-03A | E3 | 独立缓冲与完成令牌契约核对 | 0 | — | 契约 2 条 + 来源行号 |
| W4-03B | E3 | 批量调度实体验收（Windows） | 1 | W4-03A + B0-1 + NP 门禁 | 独立缓冲/令牌实测 + 无共享污染 |
| W4-03C | E3 | 证据 + QA | 0 | W4-03B | README/receipt/manifest + QA PASS |
| W4-04A | E4 | P10 变体契约核对 | 0 | — | 同 W4-03A + P10 差异 |
| W4-04B | E4 | P10 批量调度实体验收 | 1 | W4-04A + W4-03B verified | 同 W4-03B |
| W4-04C | E4 | 证据 + QA | 0 | W4-04B | 同上 |
| W4-05A | E5 | 时序/状态/同步契约核对 | 0 | — | 三条契约逐条 |
| W4-05B | E5 | 回调实现或契约验收 | 0（设备时序 1） | W4-05A、m10-p7 verified | 三条契约证据；不宣称性能 |
| W4-05C | E5 | 证据 + QA | 0 | W4-05B | 同上 |
| W4-06A | E6 | 准入核对（8 条 r-p8 + 传输层 + r07 处置） | 0 | B0-1 | 逐条状态表 |
| W4-06B | E6 | 基础批量渲染准入证据（双模式） | 1 | W4-06A + E1/E2/E3/E4/E5 对应 verified + NP 门禁 | 准入证据齐（可与 W4-07B 合并一窗） |
| W4-06C | E6 | 汇总 + QA | 0 | W4-06B | 同上 |
| W4-07A | E7 | 全量前置核对 | 0 | W4-06A | p8-base/m03/g28/r06 状态表 |
| W4-07B | E7 | 全部渲染与查询准入证据 | 1（可与 W4-06B 合并） | W4-06B + W4-02 verified + g28-p8 + r06-flex-closeout | 准入证据齐；不得超范围声明 |
| W4-07C | E7 | 汇总 + QA | 0 | W4-07B | 同上 |

## 4. 顺序、窗数与风险

- **顺序**：B0（0 窗，待裁决项先行）→ 01A/02A/05A（0 窗、单写者串行）→ NP5-01/02/03（0 窗，先行件）→
  03B/04B/06B/07B（GPU 窗串行）。
- **窗数**：3–4 窗（p8-entry 1、r10-p8 1、r10-p10 1、p8-base+p8 合并 1–2；m03/m10 预计 0）。GPU 纪律沿用 W0：
  开窗前 5 分钟基线、monitor 60/60、floor 取严、单写者、fail-stop 不重试。
- **风险**：① r07-p8 若需用户 ABI 签名，W4-03B/06B 必须等 R07 裁决；② p8-entry/p8-base 的 NP 门禁与 NP5 强耦合，
  NP 未 verified 时片 B 不在窗内执行；③ `src/render/**` 写域重叠，派工时按文件/模块拆分避免并行写冲突；
  ④ E6/E7 是阶段准入，**不是发布声明**。

## 5. 边界与非声明

- 本草案只登记「7 张实体卡 + 派工拆分」；**未执行** B0-1（r07-p8 处置）、B0-2（g10-p5 改指）、B0-4（NP note）——
  这些属 Chief 裁决项，本文件只列待裁点，**不伪造解除、不写 waived 依据**。
- 不改任何占位卡 acceptance/依赖边；不改判据/参考/容差；零 GPU、零窗、不写 `src/`/`tests/`/`fixtures/`/`scripts/`。
- 不宣称 P8 准入通过、不宣称全部渲染已通过/等价/发布；E6/E7 阶段准入 ≠ 发布声明。