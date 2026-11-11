# NP5-01a C-3：C 批 rne/gravity 43 站点 source 级逐站点对照（零 GPU）

- 任务：`2026-11-11-np5-01a-c3-rne-gravity-source-level`（owner Backend；p95；P8-渲染与BVH）。
- 依据：`np5-01a-SLICE-PLAN.md` C-3 口径；站点 = `np5-01a/sites.json` 的 3 个 rne/gravity 行（43 站点）；
  词表 `np5-01a-vocab-addendum.md`（`453c5c91…`/6771 B）。
- 本卡第一节处置 C-2 复核的 2 条观察（§4）。
- 边界：source-level、零 GPU、零窗；不改 `src/`/`tests/`/`fixtures/`/判据/参考/容差。

## 1. 站点与分类计数（43 = 3 组；NP5-01a 累计 337 + 43 = 380）

| 组 | 站点 | 构成 | 说明 |
| --- | --- | --- | --- |
| `rne_acc` | 18 | accumulation 6 / intermediate 7 / output 4 / rounding-note 1 | 空间加速度前向 + 内部力 + 反向原子汇聚 + qfrc 点积（rne.cu 四 kernel） |
| `gravity_compensation` | 13 | accumulation 3 / intermediate 6 / output 4 | 重力补偿 body force→dof（validate/compute/sum/clear） |
| `rne_bias` | 12 | accumulation 4 / intermediate 4 / output 3 / rounding-note 1 | 单 kernel 完整 RNE 偏置（acc 前向、cfrc、逆序汇聚、点积） |

分类计数：**{'identical': 29, 'n/a': 13, 'engine-semantic': 1}**
（逐站点明细见 `evidence.json`；不写覆盖类/通过类统计口径）。

## 2. 方法与判据

- 产品锚点：`src/physics/rne.cu`（真实 .cu 源，四个 kernel：`rne_cacc`/`rne_cfrc`/`rne_backward`/
  `rne_qfrc`）、`src/physics/gravity_compensation/cuda.rs`、`src/physics/rne_bias.rs` 的 CUDA 源；
  `product_anchor` 为 repo 行号 + 表达式原文。
- 参考锚点（全部带 file:line）：`smooth.py` `_cacc_world`（1324）/`_cacc_branch`（1342）/
  `_cfrc`（1399）/`_cfrc_backward`（1430）/`_qfrc_bias`（1470），`math.py` `inert_vec`（121）/
  `motion_cross_force`（148），`passive.py` `_gravity_force`（327）/`_qfrc_passive_kernel`（687），
  `support.py` `jac_dof`（505）。
- 分类六类词表：`identical` **29**（source 级表达式形态与次序对应；不声明位级）、
  `n/a` **13**（声明/指针/状态码/零初始化/局部量/prose）、
  `engine-semantic` **1**、`recipe-difference` 0、
  `not-decidable` 0、`pin-needed` 0。
- 核心对照：rne_acc 的 cacc 初值（`__int_as_float` 位型搬运 vs 直接 -gravity）、
  `a += dot*qvel` / `a += cdof*qacc`、`_cfrc` 的 `inert_vec`+`motion_cross_force` 与显式 fma 冻结序、
  `_cfrc_backward` 的树层反向原子加与产品 atomicAdd、`_qfrc_bias` 的 6 分量点积；gravity_compensation
  的 `(-g*mass)*gc`、`jac_dof` 的 `cdof_lin + cross(cdof_ang, offset)` 与原子汇聚、sum 的
  `__fadd_rn` 显式单次舍入；rne_bias 的同一 rne 链式写法。
- `engine-semantic` 1 条：`rne_bias:rne_bias:accumulation:1`（子树力向父汇聚：产品每 world 单线程按
  body 逆序确定性累加 vs 上游按 body_tree 逆层并行 `wp.atomic_add`；f32 求和次序可能不同，不声明等价）。
- `not-decidable` 未使用：43 站点均有 source 锚点；设备级映射留给 RE-3。

## 3. 自检

- 43/43 站点与 `sites.json` 的 id/kind/label 逐条一致；3 组计数 = 18/13/12。
- `pending_manual=0`；分类计数与逐条目重算一致；`not_decidable_reasons` 词表未被使用（0 条）。
- `policy_scan`（`scripts/np-gate/policy_scan.py`，`8753685b…`/14885 B）
  对本目录 `evidence.json`/`README.md`/`receipt.json` 命中 0；工具 `--self-test` **31/31 PASS**
  （含对称声明词与否定 veto 用例）。
- 编码 UTF-8 / LF / 无 BOM；manifest 逐文件 sha256+bytes 自核；详见 `logs/selfcheck.txt`。

## 4. 第一节：C-2 复核观察处置

- **O1（声明词表不对称）**：声明词补对称集合——中文 撤回/作废/撤销/收回；英文 supersede/withdraw/
  withdrawn/withdrawal/retract/retracted。规则同时收紧：块内任一**被否定的声明词** veto 整个豁免
  （`withdrawn: never supersede` 不豁免；保守方向）。docstring 已完整列出声明词与否定词表。
  自测 28→**31/31 PASS**；工具新版
  `8753685b…`/14885 B。
- **O2（errata 清单）**：`errata-0001`（flex_hessian_validate field-table 滞后）已从 C-2 evidence 转发到
  本片 `errata_pending`（status=forwarded to closing addenda errata list）；仅记录、不改 field-table 历史，
  收尾 addenda 卡统一入清单。

## 5. 输入与哈希

- 站点清单 `target/render-bvh/gl2-ptx-diff/np5-01a/sites.json` = `e0d4a95bf5c0acf13510b20f51e92ece1ec63ab7c7ac9e37be38cdd0841bac3c` / 261483 B。
- 场表（3 行，`docs/development-evidence/np01-type-mapping/field-tables/`）：
  - `rne_acc.json` `5a3640ea2dec484fc9d5256008c8fbe592525206dc8b0b7c537b040e0e10bddb`
  - `gravity_compensation.json` `ff5051b80d2670273b458b52c00c6f40bff58e242d08679da34184091b1533ea`
  - `rne_bias.json` `b082e4b218481cd8151a71e701889ab63fb36578cc43c4bcaf07d27668ce96f1`
- spec `target/render-bvh/gl2-ptx-diff/np5-01a-exec-spec.md` = `5321f31258ca9ce02f344dd38c1d1d32ad2005fc1cd66b27f21fe1c8b86c5389` / 12402 B（工作材料，非证据）。
- 词表 `target/render-bvh/gl2-ptx-diff/np5-01a-vocab-addendum.md` = `453c5c9157341fa28ecb009c50f2ff560b6f43e15f593ed9b7a75f27dd5b1b26` / 6771 B（工作材料，非证据）。
- upstream revision `71da24d956378a87a703b6e1442b13aec0c4ac29`；`tree.head` `f48eaf823126c0ea765b8b125e098cf306a941ac`（本卡主提交父提交）。

## 6. 非声明

不宣称 NP01 通过/E 指标完成/上游等价/双平台/位级保证；不改判据/参考/容差；零 GPU、零窗；
rne/gravity 设备级映射不在本片范围（RE-3 后续）。
