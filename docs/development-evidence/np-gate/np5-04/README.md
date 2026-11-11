# NP5-04：NP01/NP02 状态汇总与用户裁决点（不宣称通过；零 GPU）

- 任务：`2026-11-11-np5-04-summary-and-rulings`（owner Backend）。
- 依据：W5 计划 §3/§5；汇总 NP5-02（NP01 精度偏离审计）与 NP5-03（NP02 缺口处置）证据，
  只读聚合、零 GPU、零窗。
- **结论：NP01 与 NP02 均未验收**；NP5-02/03 只交付审计与处置证据；**不宣称任何门禁通过**。

## 1. NP01 状态（类型映射 / 精度偏离）

| 项 | 值 |
| --- | --- |
| 类型表 | 33（30 产品路径 field-tables + 3 host-tables） |
| NP5-01a 站点/唯一表达式 | 608 / 598 |
| 类型偏离 | **0 项**（依据类型表 + CUDA/host token 门禁 + 608 站点 type_mapping，非数值反推） |
| 精度偏离 0 项 | **未完成**（E 指标未定稿；跨 recipe 位级前提不成立；device 可映射子集） |

`NP01 精度偏离 0 项未完成`（强制句，见 `np-gate/np5-02/evidence.json` 与 README）。

## 2. NP02 状态（内核与精度证据覆盖）

| 项 | 值 |
| --- | --- |
| inventory 行 | 102（2026-11-11 补 2 个真实渲染内核行） |
| 覆盖口径 | evidence_present 102 / 100%、evidence_missing 0、verified 91、源身份 89.2% |
| 缺口 | 32 条，**32/32 已处置**（archive 9 / archive-derived 2 / closed-implemented 3 / blocked 4 / N-A 6 / recorded 4 / deferred 4） |
| 未实现子系统 | **4 条 blocked**（flex collision、constraint、solver、island；卡 H 已登记，不计覆盖） |

**NP02 未验收**：4 个未实现子系统仍 blocked；Linux 4 条 deferred；不得写成通过。

## 3. 残余阻塞（7 项）

1. **未实现子系统 4 条**：flex collision / constraint / solver / island —— 卡 H 已登记，用户裁决是否排 W2/W3/R07。
2. **Linux 4 条 deferred**：用户 2026-10-24 暂停 Linux，W7 集中回测解冻后处理。
3. **errata-0001**：`flex_hessian_validate` field-table 描述滞后，已入收尾 errata 清单，field-table 历史不改。
4. **deferred 偏离清单**：空（A-1a R2 明确不进；MathDx opt-in 只记 watch）。
5. **NP5-04 watch 项 2 条**：A-1a `acc:3`（MathDx reciprocal vs 除法）、RE-1 `acc:1`
   （`rsqrt.approx` vs `sqrt.rn`）——若 MathDx 变默认路线需在 E 定稿后重评。
6. **第 10 条漂移 `scene_bvh_ray`**：待用户/Chief 裁决（CPU-only 补采回执或按裁决 B 归档），本波未自行处理。
7. **NP03/NP04 与 P11**：NP01/NP02 未过前不得 start、不得提发布声明（用户级）。

## 4. 用户裁决点清单（7 项）

| id | 主题 | 需要 | 阻塞 |
| --- | --- | --- | --- |
| **U8/U7** | W2（G10@P5 mesh 方案 A 增量）判据/口径提升：U8/U7 条目是否提升 pass/fail + A2 容量 20/10→32/32 | 用户裁决（编号待 Chief 校准） | W2 GPU 窗 |
| **ABI** | R07/W4 ABI 签名口径（含 constraint/solver 接口） | 用户/Chief 裁决 | R07 实现片 |
| **eol** | 仓库/证据 eol 政策（LF 规范化与历史字节保留） | 用户裁决 | 无当前阻塞 |
| **等价/发布** | 上游等价口径、双平台等价范围、P11 发布声明 | 用户裁决 | NP03/NP04 与 P11 |
| **E 指标定稿** | 2026-10-27 口径（G6 首步产品输出对 native f64 重算） | 用户裁决 | NP01 精度偏离 0 项验收、NP03 等价 |
| **4 阻塞卡排期** | flex collision/constraint/solver/island 是否/何时排 W2/W3/R07 | 用户裁决 | NP02 覆盖无法补齐 |
| **scene_bvh_ray 漂移** | 第 10 条漂移选路（补采 vs 归档） | 用户/Chief 裁决 | NP02 源身份口径 |

## 5. 非声明与边界

- 不宣称 NP01/NP02 通过、上游等价、双平台、位级保证；不把未实现子系统写成通过。
- 只读聚合；不改判据/参考/容差；零 GPU、零窗；QA 复核本片。
