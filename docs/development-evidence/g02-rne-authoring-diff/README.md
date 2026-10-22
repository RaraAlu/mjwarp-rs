# G02 RNE 加速度 521 块缺口：离线 authoring 根因定位

任务：`2026-10-23-g02-rne-acc-authoring-diff`（登记 `109ebcd`）。
输入来源：`2026-10-22-g01-all-probes-env-wiring`（`6cb5a71`）接线后 all-probes 首次执行 `resident_rne_acceleration`，报 521 个 native-tolerance 失败块（scalars=900288）。

## 结论（根因）

**521 块不是本库与上游 MJWarp 的实现分歧，而是「冻结原生参考口径」问题：**

1. 521 块全部是同一组合：`mixed-joints`、gravity 索引 2、state 7、`flag=false`、worst_dof=3；本库 f32 实际值 `1.5473079681396484`，MuJoCo 3.12 原生 f64 参考 `1.5471062036731382`，差 2.02e-4，normalized 3.96（容差 ABS=REL=2e-5）。
2. 用 pinned 环境离线生成上游 MJWarp（冻结 `71da24d…`）同场景 RNE 加速度参考后：**该失败块的产品 qfrc_bits 与上游参考逐位相同（0/11 位差）**。
3. 上游 MJWarp 自身对同一冻结原生参考也失败同一块（natural validation：240 条记录中仅 1 条超出容差，即 mixed-joints/g2/state7/flag=false，normalized 3.961；其余 239 条全在容差内）。
4. 原因：该 state 的 dof3 是量级很小的 Coriolis/偏置项（1.55 量级），MJWarp 按 f32 累加，而原生参考是 f64；f32 舍入在该小量级上超出 2e-5 绝对+相对容差，而 `flag=true` 因 qacc 项量级大（30.9）相对容差随之放大，故同 state 通过。这与 G01 W-D4 重力位证书是同类口径问题。

**修复计划（不需要改产品代码，需用户裁决参考口径）：**
- **推荐 A（与 W-D4/G03 同法）**：把 `fixtures/rne-acceleration` 的期望值重基线为离线 authoring 的 MJWarp f32 参考（保持 2e-5 容差不变，只换参考来源为「冻结上游 MJWarp 同口径 f32」）。实测产品 vs 上游参考在该容差下 **0 条失败、worst normalized 0.743**；重基线后 521 块应全部转绿，产品零改动。
- 备选 B：保留原生 f64 参考并放宽该 state 容差——不推荐，会削弱门禁，且上游同样过不了，属于用放宽容差掩盖口径问题。
- 需另立「参考重基线/裁决」任务并等用户确认；本任务只出证据，不修改 `fixtures/`、测试断言或容差。

## 次要发现（非 521 块成因，单独可选对齐项）

产品与上游参考在 240 条单世界记录中 84 条存在**小 ULP 级差异**（首差出现在 `cacc` 阶段；`cfrc_int`/`qfrc_bias` 继承），最大 normalized 0.743 < 1，故在现容差下全部通过。该差异来自 M3 融合 RNE 加速路径的 FMA/累加顺序与上游不逐位一致，量级远低于门槛；若后续要求与上游逐位一致，应另立「RNE 加速路径逐位对齐」任务（像矩阵链那样按首差归零），不要混入本次口径裁决。

## 离线 authoring（可复现，非产品链）

```
pwsh -NoProfile -File scripts/authoring/author-g02-rne-acceleration.ps1
```

- 环境：pinned venv（Python 3.12.13 / mujoco 3.15.0 / warp-lang 1.15.0 / numpy 2.5.4 / mujoco-warp 3.15.0），上游 checkout HEAD=`71da24d956378a87a703b6e1442b13aec0c4ac29` 且 clean，已安装 75 个 .py 与上游逐文件 SHA256 一致，`warp.dll` 与 pinned wheel RECORD 一致；启动前任一不符即拒绝运行。
- 场景：5 models × 3 gravities × 8 states × flags{false,true} = 240 条；模型用 XML host（3.15 可加载）+ `fixtures/rne-bias` 的产品输入数组重建（与产品同一模型口径），`.mjb` 为 MuJoCo 3.12 冻结产物、pinned 3.15 无法直接加载，故采用与 G03 authoring 相同做法。
- 生成链：`kinematics → com_pos → crb → com_vel → rne(flg_acc)`，每条记录运行两次逐位一致（deterministic_runs=2）。
- 天然校验：上游参考 vs 冻结原生 MuJoCo 3.12 f64 参考（ABS=REL=2e-5），240 条中 1 条超限（见上），其余通过；不调参凑数。
- 产物（不进产品 fixtures，产品构建/测试不读）：
  - `upstream-rne.json`（上游参考：qfrc_bias/cacc/cfrc_int 的 f32 位串）
  - `natural-validation.json`（上游 vs 原生参考逐条归一化误差）
  - `provenance.json`（版本/上游提交/输入 SHA256/命令/脚本 SHA256/GPU/驱动/确定性）
  - `product-rne.json`（产品侧离线 dump，`target/g03-precheck-dump` dev 工具生成，非产品代码）
  - `product-vs-upstream.json`（逐 stage 位差，84 条记录）
  - `analysis.json`、`product-vs-upstream-tolerance.json`（归一化误差：0 失败，worst 0.743）

## 证据判读与边界

- `analysis.json.failing_block.product_equals_upstream = true`：521 块失败块的 qfrc 位串产品=上游。
- `analysis.json.natural_validation.failed_records`：上游在原生参考下也只失败 mixed-joints/g2/state7/flag=false 这 1 条（flag=true 因相对容差通过）。
- `product-vs-upstream-tolerance.json`：产品对上游 f32 参考 240 条全部在 2e-5 容差内，worst 0.743 → 重基线参考即可转绿。
- 未修改产品 `src`、`tests`、`fixtures`、断言与容差；未引入外部运行时组件；Python 仅离线 authoring，产品测试只跑 cargo。
- 未验收边界：参考重基线未做（等用户裁决）；RNE 逐位对齐未做（可选后续任务）；Linux/性能未测。
