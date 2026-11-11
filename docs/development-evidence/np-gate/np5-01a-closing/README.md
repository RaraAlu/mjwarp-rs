# NP5-01a 收尾 addenda 与完成报告（2026-11-11；零 GPU）

- 任务：`2026-11-11-np5-01a-closing-addenda`（owner Backend）。
- 性质：NP5-01a 全片收口汇总（append-only 报告，不改任何历史片字节）；含双口径、errata 清单、
  A-1 派生计数、L manifest 复核、累积观察与分片清单核对。
- 完成声明：NP5-01a 场表 **608 站点（唯一表达式 598）** 已全部分片收口——逐站点 source 级
  类型映射与表达式分类全覆盖；device 级按可映射子集完成（RE-1/RE-2/RE-3 + A-1 部分），
  缺项逐条 `not-decidable` 并写原因。**不宣称 NP01 通过、E 指标完成、上游等价、双平台或位级保证。**

## 1. 分片清单与站点核对（608 = A179 + B78 + C226 + D125）

| 批 | 片 | 站点 | 主提交 | 证据目录 |
| --- | --- | --- | --- | --- |
| A | A-1a（mass_solve factor） | 40 | `de8f44dc` | `np-gate/np5-01a-a1a/` |
| A | A-1b（mass_solve solve） | 36 | `3f68a621` | `np-gate/np5-01a-a1b/` |
| A | A-2（kinematics+crb） | 63 | `2cede9b6` | `np-gate/np5-01a-a2/` |
| A | A-3（com/mass_multiply） | 40 | `12156259` | `np-gate/np5-01a-a3/` |
| B | B-1（肌腱 7 路径） | 78 | `bbd7c7cc` | `np-gate/np5-01a-b1/` |
| C | C-1（flex） | 52 | `2d3a17a4` | `np-gate/np5-01a-c1/` |
| C | C-2（force/energy） | 68 | `215693d3` | `np-gate/np5-01a-c2/` |
| C | C-3（rne/gravity） | 43 | `37a21e59` | `np-gate/np5-01a-c3/` |
| C | C-4（pose/batch/sleep） | 63 | `f503c9df` | `np-gate/np5-01a-c4/` |
| D | D-1（gjk/epa host） | 88 | `32fd81b8` | `np-gate/np5-01a-d1/` |
| D | D-2（ccd host + 2 附录单元） | 37 | `06811c89` | `np-gate/np5-01a-d2/` |

- 合计：A 40+36+63+40 = **179**；B **78**；C 52+68+43+63 = **226**；D 88+37 = **125**；
  **总 608 站点** ✓（`sites.json` rows 30 张场表行 + 3 张 host 表行 + 2 个附录单元）。
- 证据/设备级片（不占站点）：RE-0（`d8f5402d`，参考 PTX 证据入库）、RE-1（`a6f5a472`，mass_solve
  设备级）、RE-2（`949bf18b`，A-2 设备级 + policy 修复）、RE-3（`01d4c979`，B/C 设备级 24 行 304 站点）。
- 治理/复核片：K（`4353796d`）、L（`a20efe2a`）、A2-POL（`df5fc8aa`）、POL-SCAN（`09b4094a`）、
  QA-ADD（`e077765a`）；各片复核卡与 append-only addenda 均在对应证据目录。

## 2. 双口径（608 / 598 / 10）——同一来源说明

- **608 = 场表行×站点口径**：A179 + B78 + C226 + D125（与 §1 逐片相加一致）。
- **598 = 去重唯一表达式**：`rigid_com_position` 内核同时登记于 **kinematics 行**（A-2 覆盖，11 站点）
  与 **com_position 行**（A-3 覆盖，14 站点）；按 `(kernel, kind, index)` 去掉行前缀后比对，
  有 **10 个 tail 重复**：`acc:0/1/2`、`int:0–3`、`out:0–2` ⇒ **608 − 10 = 598**。
- 来源：A-2 README §1 与 A-3 复核观察 O1；机器可读值见 `np-gate/np5-01a-d1/receipt.json`
  `section1.o1_dual_count` 与 `np-gate/np5-01a-a3/rows-fields-v1.json`。
- **报告口径纪律**：608 是场表行×站点数，**不是** 608 个唯一表达式；任何汇总必须并列两口径，
  不得把 608 读成唯一表达式数。

## 3. A-1 派生计数（landing 直方图 + 最终口径）

| 片 | landing 直方图（evidence.json） | 最终口径（A-1a/A-1b addenda + QA-ADD 处置后） |
| --- | --- | --- |
| A-1a 40 | n/a 22 / identical 12 / recipe-difference 2 / not-decidable 2 / pin-needed 1 / engine-semantic 1 | n/a 22 / identical 11 / recipe-difference 3 / not-decidable 3 / engine-semantic 1 |
| A-1b 36 | n/a 21 / identical 11 / engine-semantic 3 / not-decidable 1 | n/a 21 / identical 10 / engine-semantic 3 / not-decidable 2 |
| **A-1 合计 76** | n/a 43 / identical 23 / recipe-difference 2 / not-decidable 3 / pin-needed 1 / engine-semantic 4 | **n/a 43 / identical 21 / recipe-difference 3 / not-decidable 5 / engine-semantic 4** |

最终口径的改判来源：A-1a R2 `acc:3` pin-needed→recipe-difference；A-1a `resident_mass_factor_派生:acc:0`
与 A-1b `resident_mass_substitute:acc:0`（含 tile 分支）identical→not-decidable（QA-ADD G1）；
A-1a R3 `acc:0/1` 保持 not-decidable（由 RE-1 升级）。历史 evidence 字节均未改。

## 4. errata 清单（统一入册；不改 field-table 历史）

| id | 位置 | 内容 | 状态 |
| --- | --- | --- | --- |
| `errata-0001` | `docs/development-evidence/np01-type-mapping/field-tables/flex_hessian.json` | field-table 仍描述 `flex_hessian_validate` 的 `ok` bool / `status[4+w]` f32，而产品 validate 内核只写 i32 flags（`ok/status` 描述滞后） | **记录并转发（本报告统一入册）**；C-2 首次记录，C-3/C-4 保持转发态；field-table 历史字节不改，修正留待 field-table 维护卡 |

- 除 errata-0001 外，本批未发现其他需要入 errata 清单的字段表滞后项（C-2/C-3/C-4 复核确认）。

## 5. 累积观察清单（全部处置或记录在案）

| # | 来源 | 内容 | 状态 |
| --- | --- | --- | --- |
| 1 | A-2 | evidence 中 2 处正面「位级」表述（crb acc:2 why、rounding-note label） | **已闭环**：A-2 append-only addenda 撤回改写（v1 恢复 + v2 supersedes），后续片 selfcheck 禁词表补齐 |
| 2 | A-2 | policy_scan 分隔符绕过/归因错误/清单对齐 | **已闭环**：RE-2 修复工具并入仓（`scripts/np-gate/policy_scan.py`），A-2 归因更正 |
| 3 | A-1a/A-1b/RE-0 | QA-ADD 8 项（tile identical 改判、2603→303、待复核计数、词表版本、residual 类合并、绝对路径说明、脚本用途、措辞统一） | **已闭环**：QA-ADD 卡 append-only 处置 |
| 4 | RE-2 | mass_multiply 18 站点属 A-3；device 行级与 source 逐站点粒度混用 | **已闭环**：A-3 完成 source 级逐站点分类；RE-2 addenda-v2 注记 |
| 5 | RE-2/B-1 | addenda 非 append、quote 块豁免、政策工具时点 | **已闭环**：版本化 addenda-v2、quote 块豁免（B-1 加固）、工具 landed_at 更正 |
| 6 | C-2 | `flex_hessian_validate` field-table 滞后（errata-0001） | **入 errata 清单**（§4） |
| 7 | C-2 | `--list-quote-blocks` 的 `reviewed` 字段语义 | **已闭环**：C-2/C-3 docstring+README 写明；receipt 留 `quote_block_inventory` 痕迹 |
| 8 | C-4 | `field_batch` wrap-pulley `1.0/wrap_prm` 唯一 f64 接触点 | **记录**：`double_in_product:true` + `double_precision_parity`（上游 numpy float64→f32 同路径，非产品单方面升精度） |
| 9 | A-3 | `rigid_com_position` 双行重复登记（10 tail） | **记录**：双口径 608/598（§2） |
| 10 | A-3 | engine-semantic 沿用 A-2 先例 | **记录**：A-3 addenda §2 |
| 11 | D-1/D-2 | host 路径无 PTX；host 非融合 recipe 与 device fma.rn 两套 recipe | **记录**：D-1/D-2 evidence `device_level_note`/`host_recipe_note`（整体记 `host-path-no-ptx`） |
| 12 | RE-0 | G1-partial PASS（仅「归一化指令流等价」）；fast_math=False 前提修正；interim 计数更正 | **记录**：RE-0 README（不写字节/位级等价） |
| 13 | RE-1 | MathDx opt-in tile 路径 `rsqrt.approx` vs 参考 scalar `sqrt.rn`（approx vs exact 非值等价） | **记录**：engine-semantic + `bit_level_guarantee:false`；NP5-04 watch 项（非 deferred 偏离清单） |
| 14 | RE-2/RE-3 | 设备行级结论与 source 逐站点粒度不同；不重判 source 级 | **记录**：RE-2 addenda/RE-3 method_rules |
| 15 | K/L | K 的 L1（multiframe 测试侧措辞）+ L 收口；K backlog=1 指 F-L1 | **已闭环**：卡 L 修复 4 处措辞（断言零改动），L receipt 记录 exe 身份 old→new |

## 6. L manifest 6/6 复核

- 目录：`docs/development-evidence/render-bvh/r06-flex-multiframe-recipe-params-comment-wording/`；
  manifest 列 6 项（README + `logs/diff-stat.txt` + `logs/identity-old-new.txt` + `logs/line-counts.txt`
  + `logs/test-log.txt` + receipt），本报告逐文件重算 sha256/bytes：**6/6 全部一致**（详见
  `logs/selfcheck.txt`）；L 的 exe 身份 old→new 与「R06 后续窗需重冻」声明在 L receipt 内。

## 7. 非声明与边界

- 不宣称 NP01 通过/E 指标完成/上游等价/双平台；不写位级保证；不改判据/参考/容差/相机/recipe。
- device 级结论只覆盖可映射子集，缺项逐条 `not-decidable` 并写原因；**不写覆盖率/通过率**。
- 零 GPU、零窗；本报告为聚合报告，不改任何历史片字节（各片 manifest 仍指向其 landing 字节）。
