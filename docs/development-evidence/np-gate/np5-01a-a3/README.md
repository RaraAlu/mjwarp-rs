# NP5-01a A-3：com_position + com_velocity + mass_multiply source 级对照（40 站点；零 GPU）

- 任务：`2026-11-11-np5-01a-a3-com-mass-multiply-source-level`（owner Backend；p95；P8-渲染与BVH）。
- 依据：`np5-01a-SLICE-PLAN.md` A 批口径；站点 = `np5-01a/sites.json` 的 com_position 14 +
  com_velocity 8 + mass_multiply 18；词表 `np5-01a-vocab-addendum.md`（`453c5c91…`/6,771，六类词表 +
  not-decidable 六原因 + reference_anchor.status）。
- 边界：source-level、零 GPU、零窗；不改 `src/`/`tests/`/`fixtures/`/`scripts/`/判据/参考/容差。
- 本片为 A 批最后一片：NP5-01a 累计 **443 + 40 = 483**（A 批 179 = A-1 76 + A-2 63 + A-3 40 全收口）。

## 1. RE-3 复核 L1/O1 处置（第一节）

- **L1（24 行补机器可读字段，不抽样）**：`np-gate/np5-01a-re3/rows-fields-v1.json`
  含全部 24 行（`row_count: 24`）：engine-semantic 15 行补 `why_engine_semantic` +
  `bit_level_guarantee:false`；not-decidable 7 行补 `not_decidable_reasons` + `why_not_decidable` +
  `missing_evidence`；n/a 2 行补 `why_not_applicable`。`addenda.md` 说明补齐方式与原
  `evidence.json` 对应关系；**原 evidence 字节不改**（`f5988b7c…`/94,815 保持），RE-3 manifest 重算为
  6 文件（新增 addenda.md、rows-fields-v1.json）。
- **O1（缺因口径更正）**：按 evidence 实测，not-decidable 缺因 = `no-reference-entry` **4 行 47 站点**
  + `no-lineinfo+frozen-artifact-only` **3 行 26 站点**（合计 7 行 73 站点）；派工摘要此前的 40/33 有误，
  后续引用按 47/26；RE-3 证据数据不改（机器可读值见 `rows-fields-v1.json.not_decidable_reasons_split_measured`）。

## 2. 站点与分类计数（40 = 14 + 8 + 18；累计 483）

| 组 | 站点 | 切分（intermediate / accumulation / output / rounding-note） |
| --- | --- | --- |
| `com_position.rigid_com_position` | 14 | 6 / 3 / 4 / 1 |
| `com_velocity.com_velocity` | 8 | 3 / 2 / 2 / 1 |
| `mass_multiply.mass_multiply_validate` | 2 | 1 / 0 / 1 / 0 |
| `mass_multiply.mass_multiply` | 5 | 2 / 2 / 1 / 0 |
| `mass_multiply.borrowed_mass_validate` | 3 | 2 / 0 / 1 / 0 |
| `mass_multiply.borrowed_mass_multiply` | 5 | 2 / 2 / 1 / 0 |
| `mass_multiply.rounding-note` | 3 | 0 / 0 / 0 / 3 |

分类计数：**`identical` 18 / `engine-semantic` 3 / `n/a` 19**（逐站点明细见 `evidence.json`；
无覆盖率/通过率表述、不声明位级）。

- `engine-semantic` 3 条（均为 com_position，沿用 A-2 同型先例口径）：
  - `intermediate:4` / `accumulation:2`：产品显式 3 层循环三重乘加 vs 上游 warp 矩阵乘组合
    （`mat @ wp.diag(inert) @ wp.transpose(mat)`）——数学等价但实现/收缩形态不同；
  - `accumulation:0`：产品串行树序累加（非原子） vs 上游 `_subtree_com_acc` 分层 `wp.atomic_add`
    并行累加——f32 加法顺序/实现路线不同。
- `identical` 18：com_position 8（中间量 3 + accumulation:1 + output 4）、com_velocity 4
  （accumulation 2 + output 2）、mass_multiply 6（4 个双向累加 + 2 个 output）——source 级表达式/类型/
  累加次序对应，**不是**位级结论。
- `n/a` 19：视图/缓冲/输入绑定、校验内核（产品自加 sentinel/isfinite/CSR 结构校验）、rounding-note。
- **mass_multiply 双口径**：RE-2 device 级记「冻结件无 `_mul_m`/`_mul_m_dense` entry」→
  `not-decidable`（`no-reference-entry`）；本片 source 级仍逐站点分类，两条口径并行不互相替代
  （与 A-1a/A-2 的 source/device 两层做法一致）。

## 3. 方法与判据

- 产品锚点（repo 行号）：`src/physics/mod.rs` `COM_POSITION_CUDA`（`rigid_com_position`，186–258）、
  `src/physics/com_velocity/cuda.rs`（`com_velocity` + `motion_cross`，15–52）、
  `src/physics/mass_matrix/multiply/cuda.rs`（`mass_multiply_validate`/`mass_multiply`，4–26）、
  `src/physics/mass_matrix/multiply/borrowed/cuda.rs`（`borrowed_mass_*`，5–37）。
- 参考锚点（冻结上游 `71da24d9`，`target/mjwarp-parallel/upstream/mujoco_warp/_src/`）：
  `smooth.py` `_subtree_com_init`（656–666）、`_subtree_com_acc`（669–684）、`_subtree_div`（687–700）、
  `_cinert`（703–746）、`_cdof`（749–791）、`com_pos`（794–809+）、`_comvel_root`（2508–2511）、
  `_comvel_branch`（2514–2587）、`com_vel`（2590–2614）、`_subtree_vel_forward`（3830–3867）；
  `math.py` `motion_cross`（133–144）；`support.py` `mul_m_kernel::_mul_m`（153–187）、
  `mul_m_dense::_mul_m_dense`（190–214）、`mul_m`（217–256）——每条均带 file:line（`reference_anchor`）。
- `identical` 的 source 级含义：产品与上游在同一表达式上写法/类型/累加次序可对应（f32 全链、无升精度）；
  `engine-semantic`：实现路线/调度语义不同；`n/a`：非算术点（视图/缓冲/校验/prose）。
- `com_position` 行与 A-2 `kinematics` 行内的 `rigid_com_position` 站点按 sites.json 分组口径分别统计
  （id/kind 切分不同，分类口径一致，不重复计为同一站点）。

## 4. 输入与哈希

- sites.json `e0d4a95bf5c0acf13510b20f51e92ece1ec63ab7c7ac9e37be38cdd0841bac3c`/261,483（工作材料）。
- 词表 addendum `453c5c9157341fa28ecb009c50f2ff560b6f43e15f593ed9b7a75f27dd5b1b26`/6,771（工作材料）。
- upstream revision `71da24d956378a87a703b6e1442b13aec0c4ac29`；tree.head `dbffa8dd7460bdf31f647b3c1811073f19751e29`（本卡主提交父提交）。
- RE-3 原 evidence `f5988b7ce0ebebf1bc072b9149c14e8a8a4ae37d2d08bda905765e18eda7873e`/94,815（未改）。

## 5. 非声明

不宣称 NP01 通过/E 指标完成/上游等价/双平台/位级保证；不改判据/参考/容差/相机/recipe；
零 GPU、零窗；`identical` 仅为 source 级表达式/类型/次序对应。
