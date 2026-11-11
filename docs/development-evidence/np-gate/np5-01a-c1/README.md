# NP5-01a C-1：C 批 flex 族 52 站点 source 级逐站点对照（零 GPU）

- 任务：`2026-11-11-np5-01a-c1-flex-source-level`（owner Backend；p95；P8-渲染与BVH）。
- 依据：`np5-01a-SLICE-PLAN.md` C-1 口径；站点 = `np5-01a/sites.json` 的 5 个 flex 族行（52 站点）；
  词表 `np5-01a-vocab-addendum.md`（`453c5c91…`/6771 B）。
- 本卡第一节同时处置 B-1 复核的 2 条低项 + 1 条观察（§4）。
- 边界：source-level、零 GPU、零窗；不改 `src/`/`tests/`/`fixtures/`/判据/参考/容差。

## 1. 站点与分类计数（52 = 5 组；NP5-01a 累计 217 + 52 = 269）

| 组 | 站点 | 构成 | 说明 |
| --- | --- | --- | --- |
| `flex_hessian` | 15 | accumulation 5 / intermediate 6 / output 3 / rounding-note 1 | 21 系数拉伸 Hessian 块装配 + validate（单 kernel 一 world 顺序累加） |
| `flex_edge_passive` | 11 | accumulation 2 / intermediate 4 / output 4 / rounding-note 1 | 柔体边弹簧/阻尼力（validate/clear/compute 原子加） |
| `flex_position` | 9 | accumulation 1 / intermediate 4 / output 3 / rounding-note 1 | 节点/顶点位置（flex_invalidate + flex_positions 插值） |
| `flex_edge` | 9 | accumulation 2 / intermediate 3 / output 3 / rounding-note 1 | 边长度/速度/稀疏雅可比（flex_edges） |
| `flex_face` | 8 | accumulation 2 / intermediate 3 / output 2 / rounding-note 1 | 线性壳体面位姿（面内切向 + 极分解四元数 flex_faces） |

分类计数：**{'identical': 32, 'n/a': 18, 'engine-semantic': 2}**
（逐站点明细见 `evidence.json`；不写覆盖类/通过类统计口径）。

## 2. 方法与判据

- 产品锚点：5 个产品源文件（`flex_hessian.rs`/`flex_edge_passive/cuda.rs`/`flex.rs`/`flex_edge.rs`/
  `flex_face.rs`）的 CUDA raw string；`product_anchor` 为 repo 行号 + 该行表达式原文。
- 参考锚点（全部带 file:line）：`smooth.py` `_flex_vertices`（231）/`_flex_nodes`（311）/
  `_flex_edges`（347）/`_flex_face_kinematics`（479），`support.py` `mat33_to_quat_polar`（68）/
  `eval_basis_trilinear`（1004），`passive.py` `_spring_damper_flexedge_passive`（261）/
  `_flex_hessian_elem`（1668）/`_flex_hessian_diag`（1773），以及 Hessian 辅助
  `stretch_edge_vectors`/`stretch_elongation`/`stretch_metric`/`stretch_tension`/
  `stretch_stiffness`/`stretch_stiffness_block`。
- 分类六类词表：`identical` **32**（source 级表达式形态与次序对应；不声明位级）、
  `n/a` **18**（声明/缓冲/纯拷贝/状态码/整型标志/零初始化/局部量/prose）、
  `engine-semantic` **2**、`recipe-difference` 0、
  `not-decidable` 0、`pin-needed` 0。
- `engine-semantic` 2 条（同一数学解、不同汇聚路径）：`flex_hessian:flex_hessian:accumulation:3`
  （同边块叠加：产品单线程顺序累加 vs 上游按元素 `wp.atomic_add`）、
  `flex_hessian:flex_hessian:accumulation:4`（对角 = -Σ 边块：产品顺序累减 vs 上游原子加负块）。
- `not-decidable` 未使用：52 站点均有 source 锚点；设备级映射留给 RE-3。
- `reference_anchor.status` 词表（L2 补）：`no-reference-expression` 表示该站点本身不是可比较算术表达式
  或上游无对应表达式（用于 `n/a`，本片 6 条）；`no-reference-entry` 表示有语义点但参考侧无 entry/PTX
  （用于 `not-decidable` reasons，本片未使用）。二者字段层级不同，不得互相替代。
- 类型映射：52/52 记录 product/upstream/type_class，`double_in_product=false`。
- **观察（非缺陷）**：field-table 对 `flex_hessian_validate` 的两条站点仍写早期描述（`ok` bool/`status`
  f32），当前产品 validate 只写 i32 私有 `flags[nf]=1`、无 ok/status；已在对应站点 note 与
  `product_anchor.expression` 记明，分类仍为 `n/a`（上游无 validate 内核），不影响其余 50 站点。

## 3. 自检

- 52/52 站点与 `sites.json` 的 id/kind/label 逐条一致；5 组计数 = 15/11/9/9/8。
- `pending_manual=0`；分类计数与逐条目重算一致；`not_decidable_reasons` 词表未被使用（0 条）。
- `policy_scan`（`scripts/np-gate/policy_scan.py`，`1d3a88b9…`/11712 B）
  对本目录 `evidence.json`/`README.md`/`receipt.json` 与 B-1 `addenda.md` 命中 0；工具
  `--self-test` 24/24 PASS（含「未撤回/不作废/没有撤回 不豁免」新增用例）。
- 编码 UTF-8 / LF / 无 BOM；manifest 逐文件 sha256+bytes 自核；详见 `logs/selfcheck.txt`。

## 4. 第一节：B-1 复核低项/观察处置

- **L1（扫描器声明否定漏洞）**：引用块声明识别改**肯定式**——声明词前（去分隔符后）紧邻
  未/不/没/无/非/否/没有/无法/不能/不会/尚未/并未/不予/未予 时不构成声明、不豁免；固定句式
  `撤回(5)`/`supersedes:` 仍豁免。自测 20→**24/24 PASS**；工具新版
  `1d3a88b9…`/11712 B（B-1 落库时 `e28ddb89…`/8,013 B、20/20 为历史值）。
- **L2（状态词表文档化）**：`reference_anchor.status` 两值定义与区别写入
  `np5-01a-vocab-addendum.md` §6（`453c5c91…`/6771 B）
  与本 README §2；B-1 实测 21 条 `no-reference-expression`。
- **观察（人工复核痕迹）**：quote 块须留 `reviewed_by: <name>-<date>`（块内首选）或 receipt
  `quote_block_inventory` 清单；扫描器新增 `--list-quote-blocks`。现存豁免块
  `np-gate/np5-01a-a2/addenda-v2.md` §2（行 29–38）复核人 `backend-2026-11-11`，
  清单见 `receipt.json` 的 `quote_block_inventory`（A-2 块字节不改）。
- **B-1 闭环**：`np-gate/np5-01a-b1/addenda.md`（`933cec55…`/2676 B）
  append-only 记录；B-1 manifest 重算 `f89d2016…`/724 B；
  B-1 的 README/evidence/receipt/logs 字节不改。

## 5. 输入与哈希

- 站点清单 `target/render-bvh/gl2-ptx-diff/np5-01a/sites.json` = `e0d4a95bf5c0acf13510b20f51e92ece1ec63ab7c7ac9e37be38cdd0841bac3c` / 261483 B。
- 场表（5 行，`docs/development-evidence/np01-type-mapping/field-tables/`）：
  - `flex_hessian.json` `7a58e449d4b979e1db69d79effda2653f8b174f02e38dc76231a4b1d2082de7b`
  - `flex_edge_passive.json` `8579e03b9208a45574d02f89f1d1cf2a08f170bee18d791e9d43da42c0180b43`
  - `flex_position.json` `88693e0fbd30d6362ac43e46194a32e665a040324e8535e8f027d31701cc1d12`
  - `flex_edge.json` `87f711490cf2d68f80d19121e8732d329bba3b790552018f7c6cab997a93f64b`
  - `flex_face.json` `c45ac8df1e9fbafa4bccfc031c922d7d5ea26bb2a36dbbf6b513050e6712b3c0`
- spec `target/render-bvh/gl2-ptx-diff/np5-01a-exec-spec.md` = `5321f31258ca9ce02f344dd38c1d1d32ad2005fc1cd66b27f21fe1c8b86c5389` / 12402 B（工作材料，非证据）。
- 词表 `target/render-bvh/gl2-ptx-diff/np5-01a-vocab-addendum.md` = `453c5c9157341fa28ecb009c50f2ff560b6f43e15f593ed9b7a75f27dd5b1b26` / 6771 B（工作材料，非证据）。
- upstream revision `71da24d956378a87a703b6e1442b13aec0c4ac29`；`tree.head` `e199c14cd0c9e5be9e00af134413047efbc107b4`（本卡主提交父提交）。

## 6. 非声明

不宣称 NP01 通过/E 指标完成/上游等价/双平台/位级保证；不改判据/参考/容差；零 GPU、零窗；
flex 族设备级映射不在本片范围（RE-3 后续）。
