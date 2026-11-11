# NP5-04 复核（QA）：NP01/NP02 状态汇总与用户裁决点清单（`any_gate_passed=false`）

- 复核对象：卡 `2026-11-11-np5-04-summary-and-rulings`；内容提交 **`82c3186a`**。
- 性质：只读复核（零 GPU、零窗）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，0 一般 + 0 低 + 2 观察）**。

## 1. 汇总数字（与 NP5-02/03 复核一致）

- **NP01**：criterion、33 类型表、608/598、`type_deviation_count=0`、`precision_deviation_zero_items_verified=false`、
  `not_verified_statement`、`source_evidence` 指向 `np-gate/np5-02/`——我逐项对上 NP5-02 复核结论。
- **NP02**：`inventory_rows=102`、counts `102/98/102/91/100/89.2/715/711/32`、8 类处置合计 32、
  `gap_disposition_complete=true`、`unimplemented_remaining=4`——与 NP5-03 我复算的数字一致。

## 2. 未通过声明（无任何通过声明）

- `conclusion.any_gate_passed=false`、receipt `any_gate_passed=false`、selfcheck 断言该字段为 false 均 PASS；
  `NP01/NP02` 两处 status 均为「未验收」；结论句明写「NP03/04 与 P11 不得启动」。
- policy_scan 对 README/evidence/receipt 3 文件 **0 命中**；无通过/等价/发布声明。

## 3. 7 项用户裁决点 / 7 项残余阻塞

- 裁决点 **7/7**：`U8/U7`、`ABI`、`eol`、`equivalence-release`、`e-metric`、`unimplemented-4-schedule`、
  `scene_bvh_ray-drift`，每项带 `topic/action_needed/blocks`，与 receipt `user_decision_count=7` 一致。
- 残余阻塞 **7/7**：`unimplemented-4`、`linux-deferred-4`、`errata-0001`、`deferred-deviation-list`（空）、
  `watch-np04`（A-1a acc:3 / RE-1 acc:1 两条 MathDx 路线 watch）、`scene_bvh_ray-drift-10`、
  `np03-np04-p11`——与既有裁定逐条对上（MathDx 只记 watch、不进 deferred 偏离清单）。

## 4. 发现

### 观察
- **O1 `any_gate_passed` 只在嵌套字段出现**：evidence 顶层无该字段（在 `conclusion.any_gate_passed`），
  README 用中文「均未通过」表述、无该字面。selfcheck 已断言嵌套值与 receipt，结论无风险；建议 README
  补一句 `any_gate_passed=false`，便于 grep/自动核对（跨卡引用时更稳）。
- **O2 watch 与 deferred 偏离清单的区分（正向）**：`deferred-deviation-list` 为空是 A-1a R2 裁定的结果
  （MathDx opt-in 只记 watch），与 NP5-02 §3「recipe 差单列」口径交叉一致，两卡不冲突。

## 5. manifest / 门禁 / 边界 / 非声明

- manifest 4/4（540 B，sha/bytes 全对）；selfcheck 全 PASS；门禁 fmt/clippy/clippy-cuda exit 0、
  unit **1,406/0/402**；policy_scan 0 命中；编码 UTF-8/LF 无 BOM、CR、NUL；边界仅 `np5-04` 与 `.normify`。
- 哈希：README `20d5e976…`/3,788、evidence `4a9f42cb…`/6,177、receipt `fd4ef266…`/2,206、
  manifest `490a7a99…`/540、selfcheck `bb527433…`/1,044。
- **不宣称** NP01/NP02 通过、精度完成、上游等价、双平台；清单只交付现状与待裁决项。
