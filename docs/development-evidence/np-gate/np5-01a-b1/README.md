# NP5-01a B-1：B 批肌腱 78 站点 source 级逐站点对照（零 GPU）

- 任务：`2026-11-11-np5-01a-b1-tendon-source-level`（owner Backend；p95；P8-渲染与BVH）。
- 依据：`np5-01a-SLICE-PLAN.md` 的 B 批口径；站点 = `np5-01a/sites.json` 的 7 个 B 批行（78 站点）；
  词表 `np5-01a-vocab-addendum.md`（`46282084…`/4867 B）。
- 本卡第一节同时处置 RE-2 复核的 2 一般项 + 1 低 + 2 观察（§4）。
- 边界：source-level、零 GPU、零窗；不改 `src/`/`tests/`/`fixtures/`/判据/参考/容差。

## 1. 站点与分类计数（78 = 7 组；NP5-01a 累计 139 + 78 = 217）

| 组 | 站点 | 构成 | 说明 |
| --- | --- | --- | --- |
| `fixed_tendon` | 7 | accumulation 1 / intermediate 2 / output 3 / rounding-note 1 | 固定肌腱长度/力臂（单 kernel） |
| `spatial_tendon` | 20 | accumulation 3 / intermediate 8 / output 7 / rounding-note 2 | site/geom/滑轮空间肌腱（site/moment/wrap 三 kernel） |
| `tendon` | 10 | intermediate 4 / output 6 | 原生全局顺序合并（values/indices 两 kernel，纯拷贝） |
| `tendon_wrap` | 9 | accumulation 2 / intermediate 4 / output 1 / rounding-note 2 | 球柱内侧绕行 device 函数族（wrap_circle/inside/intersect） |
| `tendon_velocity` | 7 | accumulation 1 / intermediate 3 / output 2 / rounding-note 1 | 肌腱速度（validate/compute） |
| `tendon_passive` | 12 | accumulation 2 / intermediate 5 / output 4 / rounding-note 1 | 肌腱被动力（validate/clear/compute） |
| `joint_passive` | 13 | accumulation 3 / intermediate 5 / output 4 / rounding-note 1 | 关节被动力（validate/compute/sum） |

分类计数：**{'n/a': 50, 'identical': 22, 'engine-semantic': 6}**
（逐站点明细见 `evidence.json`；不写覆盖类/通过类统计口径）。

## 2. 方法与判据

- 产品锚点：7 个产品源文件（`fixed_tendon.rs`/`spatial_tendon.rs`/`tendon.rs`/`tendon_wrap.rs`/
  `tendon_velocity.rs`/`tendon_passive/cuda.rs`/`smooth_force/joint_passive.rs`）的 CUDA raw string；
  `product_anchor` 为 repo 行号 + 该行表达式原文。
- 参考锚点（全部带 file:line）：`smooth.py` `_joint_tendon`（3987）/`_accumulate_jac_chain`（4026）/
  `_spatial_site_tendon`（4072）/`_spatial_geom_tendon`（4155）/`_spatial_tendon_wrap`（4367），
  `util_misc.py` `wrap_circle`（105）/`wrap_inside`（203）/`wrap`（328）/`_poly_force`（865），
  `forward.py` `_tendon_velocity`（1394），`passive.py` `_spring_damper_tendon_passive`（196）/
  `_spring_damper_dof_passive`（82）/`_qfrc_passive_kernel`（687），`math.py` `quat_sub`（178）/
  `quat_to_vel`（162）/`safe_div`（321），以及 warp `vec.h` `dot`/`normalize` 内建实现。
- 分类六类词表：`n/a` **50**（声明/缓冲/视图/纯拷贝/状态码/整型索引/零初始化/prose）、
  `identical` **22**（source 级表达式形态与次序对应；不声明位级）、
  `engine-semantic` **6**、`recipe-difference` 0、`not-decidable` 0、
  `pin-needed` 0。
- `engine-semantic` 6 条（同一数学解、不同实现/汇聚路线，不做数值等价结论）：
  `fixed_tendon:fixed_tendon:accumulation:0`（单线程顺序累加 vs 上游按 wrap 元素 atomic 累加）、
  `spatial_tendon:spatial_site:accumulation:0`（同上，site/geom 分段长度汇聚）、
  `spatial_tendon:spatial_site:accumulation:1`（wrap Newton 迭代的 acos/asin 入参夹紧 vs 上游直接调用）、
  `spatial_tendon:spatial_moment:accumulation:0`（顺序写 jac vs 上游 atomic 累加）、
  `tendon_wrap:wrap_circle_inside_intersect_device_函数族:rounding-note:0`（同上夹紧，展开记录）、
  `joint_passive:joint_passive_compute:intermediate:3`（四元数归一化：先取倒数再乘 vs warp 组件除法；
  近零保护分支语义不同）。
- `not-decidable` 未使用：B 批 78 站点均有 source 锚点（产品表达式 + 上游函数/表达式），设备级映射
  与缺口留给 RE-3；本片不写覆盖类统计。
- 类型映射：78/78 记录 product/upstream/type_class，`double_in_product=false`；B 批无数值型 f64 偏离。

## 3. 自检

- 78/78 站点与 `sites.json` 的 id/kind/label 逐条一致；7 组计数 = 7/20/10/9/7/12/13。
- `pending_manual=0`；分类计数与逐条目重算一致；`not_decidable_reasons` 词表未被使用（0 条）。
- `policy_scan`（`scripts/np-gate/policy_scan.py`，`e28ddb89…`/8013 B）
  对本目录 `evidence.json`/`README.md`/`receipt.json` 命中 0；工具 `--self-test` 20/20 PASS。
- 编码 UTF-8 / LF / 无 BOM；manifest 逐文件 sha256+bytes 自核；详见 `logs/selfcheck.txt`。

## 4. 第一节：RE-2 复核处置（G1/G2/L1/观察）

- **G1（A-2 addenda 版本化）**：A-2 `addenda.md` 恢复 v1 字节
  `0ba40a0b…`/2030 B（= `df5fc8aa` 时点内容）；
  修正内容版本化为 `addenda-v2.md`（`e6bde50c…`/6729 B，
  头部写 `supersedes: addenda.md@0ba40a0b…` + 修正原因）；A-2 manifest 重算
  `e5d4e35b…`/909 B。RE-2 evidence 指针同步更新。
- **G2（policy_scan 引用块豁免）**：新增显式标记引用块豁免（块内须含撤回/supersede/作废声明；未标记、
  无声明、未闭合、块外一律照常命中；引用须逐字原文，禁止拆写规避）；构造自测新增 8 条、合计 20/20 PASS；
  工具 `e28ddb89…`/8013 B 入仓，docstring 写用法与人工复核要求。
- **L1（时点）**：RE-2 `policy_tool.landed:true` 在 `949bf18b` 时点不成立；工具由 `e5315d95` 落仓，
  RE-2 evidence/receipt 已更正为 `landed_at=e5315d95`。
- **重扫**：A-2 evidence 5（历史已撤回）、addenda-v2 0（引用块豁免）、addenda v1 历史字节 7（不改历史）、
  README 1（否定式非声明句被 18 字符窗口漏豁免，人工复核确认非正面结论）、receipt 0。
- **观察项**：`mass_multiply` 18 站点属 A-3；设备行级与 source 逐站点粒度不同（A-2 `addenda-v2.md` §5）。

## 5. 输入与哈希

- 站点清单 `target/render-bvh/gl2-ptx-diff/np5-01a/sites.json` = `e0d4a95bf5c0acf13510b20f51e92ece1ec63ab7c7ac9e37be38cdd0841bac3c` / 261483 B。
- 场表（7 行，`docs/development-evidence/np01-type-mapping/field-tables/`）：
  - `fixed_tendon.json` `914fdf2cf76b27069ee98fed6f1efd0991957fab70257544f020f712ff8b87f3`
  - `spatial_tendon.json` `3d3524aa9047486aedb1248181264944446a3f5792cf46949858571594371019`
  - `tendon.json` `bbc9a73208c602aec4cc641928109cbcdf514f8ea5e949f2c18695fd590dd6ec`
  - `tendon_wrap.json` `adbc6881dfe65ca5a83cf61fe26770bbaefaa3a266a438eb0367cfeb23ca8f46`
  - `tendon_velocity.json` `2a9f9732bf4ab834be4f9790bb75e122f3c8998b52e5a74de240477493c3a7ab`
  - `tendon_passive.json` `8b4cedc27e3fd416ab974a75dc94c19364d245303a860a5e40c5d37cb9e8a98a`
  - `joint_passive.json` `64691223229fec65880c33cbbb59a49b3b4562fbcf5f00ae436ee878827644e1`
- spec `target/render-bvh/gl2-ptx-diff/np5-01a-exec-spec.md` = `5321f31258ca9ce02f344dd38c1d1d32ad2005fc1cd66b27f21fe1c8b86c5389` / 12402 B（工作材料，非证据）。
- 词表 `target/render-bvh/gl2-ptx-diff/np5-01a-vocab-addendum.md` = `46282084cbe884e53446155f0a2ed44336880367651661376b4a1e5d081f9591` / 4867 B（工作材料，非证据）。
- upstream revision `71da24d956378a87a703b6e1442b13aec0c4ac29`；`tree.head` `30341d184a233adfdcf731e9081ceecc78c2c8dd`（本卡主提交父提交）。

## 6. 非声明

不宣称 NP01 通过/E 指标完成/上游等价/双平台/位级保证；不改判据/参考/容差；零 GPU、零窗；
B 批设备级映射不在本片范围（RE-3 后续）。
