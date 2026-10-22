# RNE 剩余超差归属审计（2026-10-16-next-rne-repair）

任务：`2026-10-23-rne-repair-audit`。对象：`2026-10-16-next-rne-repair`（p85，proposed）的验收仍引用 2026-10-14 的「false 模式 1042 块超差」。
输入：用户裁决 A 的参考重基线 `b13d1c5`（`fixtures/rne-acceleration` qfrc 期望改为冻结上游 MJWarp f32）；authoring 证据 `docs/development-evidence/g02-rne-authoring-diff/`；历史 f64 归档 `docs/development-evidence/g02-rne-acc-rebaseline/historical-f64/`。

## 结论

**已覆盖。** 1042 块不是单一缺口，而是两组各 521 块：

| 组 | 2026-10-14 记录 | 当前产品 vs 历史 f64 期望 | 覆盖路径 |
| --- | --- | --- | --- |
| `mixed-joints` gravity0 state3 flag=false | 521 块 | **通过**（max normalized 0.768） | 2026-10-14 之后产品侧变化使其消失（区间内夹具未变，RNE 代码有 10-19/10-21 迁移提交） |
| `mixed-joints` gravity2 state7 flag=false | 521 块 | **失败**（max normalized 3.961，worst dof=3，1 个 dof 超差） | 参考口径问题，由 `b13d1c5` 重基线覆盖；对重基线后期望 0 失败（worst 0.743） |

- 重基线后：当前产品对上游 f32 期望 **0 失败记录 / 0 块**（240 条记录，worst normalized 0.743）。
- 方向性说明：1042 = 2 × 521，与「每个 state 组覆盖 worlds [1,2,5,513] × 8 rounds，按 state 计 1+2+5+513 = 521 个 world-field 块」一致，不是 521 的统计扩大。
- 当前探针复测：`resident_rne_acceleration --ignored` **2 过 0 败**，`failed_world_field_blocks=0`、scalars=900288、容差仍为 2e-5/2e-5（见 `test-run.json`）。

## 方法

1. 取当前产品代码的离线 dump `product-rne.json`（240 条 = 5 models × 3 gravities × 8 states × 2 flags，`qfrc_bias_hex` 为大端 f32 位串）。
2. 分别对照：
   - 历史原生 f64 期望（`historical-f64/`，即 `b13d1c5` 前夹具）；
   - 重基线后的上游 f32 期望（当前 `fixtures/rne-acceleration/`）。
3. 对每条记录按 `|actual-expected| / (2e-5 + 2e-5|expected|)` 计算归一化误差，>1 记为超差。
4. 复现：`node docs/development-evidence/rne-repair-audit/audit-rne.mjs`（只读，输出 `attribution.json`）。

## 证据索引

- `attribution.json`：240 条逐记录对照、两组归属、覆盖判定与 worst 值。
- `audit-rne.mjs`：只读复现脚本（不写仓内数据，只写本目录 `attribution.json`）。
- `test-run.json`：当前 RNE 探针复测记录与 `full-numerical.json` 摘要。

## 关闭 2026-10-16-next-rne-repair 的依据

- 其两条验收：「定位 false 模式 1042 块超差并区分产品/原生/上游证据」与「保持容差与全部样本」——本审计逐组定位并区分了参考来源，容差、样本、断言与夹具均未改动。
- 剩余 521 块已由用户裁决 A 的参考重基线覆盖；另一组 521 块已不复现。故按规范 `abandon`（proposed 任务无工作提交，不能直接 verified）并在 note 写明「已被覆盖，无需产品修复」。

## 边界与未验收

- 本任务只做归属审计与一次探针复测，不宣称完整 G02 验收；Linux、性能、完整上游等价未验收。
- 产品与上游仍有 84/240 条 ULP 级小差异（authoring 证据，最大 normalized 0.743）；属可选的「RNE 逐位对齐」后续，不混入本次裁决。
- 未逐 commit 二分「state3/g0 组消失」的具体提交；区间内 `fixtures/rne-acceleration` 仅 `23dec05`（10-14 引入）与 `b13d1c5`（10-23 重基线）两次提交，RNE 产品侧有 10-19/10-21 迁移系列，归因区间明确。
- 未改产品代码、断言、容差、排除清单与夹具。
