# NP5-01a C-2：C 批 force/energy 68 站点 source 级逐站点对照（零 GPU）

- 任务：`2026-11-11-np5-01a-c2-force-energy-source-level`（owner Backend；p95；P8-渲染与BVH）。
- 依据：`np5-01a-SLICE-PLAN.md` C-2 口径；站点 = `np5-01a/sites.json` 的 5 个 force/energy 行（68 站点）；
  词表 `np5-01a-vocab-addendum.md`（`453c5c91…`/6771 B）。
- 本卡第一节同时处置 C-1 复核的 1 条低项 + 2 条观察（§4）。
- 边界：source-level、零 GPU、零窗；不改 `src/`/`tests/`/`fixtures/`/判据/参考/容差。

## 1. 站点与分类计数（68 = 5 组；NP5-01a 累计 269 + 68 = 337）

| 组 | 站点 | 构成 | 说明 |
| --- | --- | --- | --- |
| `fluid_force` | 27 | accumulation 4 / intermediate 13 / output 6 / rounding-note 4 | 流体 body force（椭球/盒两分支）+ qfrc 投影 + validate/clear |
| `kinetic_energy` | 13 | accumulation 3 / intermediate 5 / output 3 / rounding-note 2 | 动能 0.5*qvelᵀMqvel（32-lane warp 归约）+ validate |
| `applied_force` | 12 | accumulation 2 / intermediate 5 / output 3 / rounding-note 2 | 外加体力→dof 的 delta/publish（显式 __fadd_rn）+ validate |
| `smooth_force` | 9 | accumulation 3 / intermediate 3 / output 2 / rounding-note 1 | qfrc_smooth 四段合成（显式 __fadd_rn/__fsub_rn）+ validate |
| `passive_force` | 7 | accumulation 2 / output 4 / rounding-note 1 | 被动力合成/重力清零/流体累加（显式 __fadd_rn） |

分类计数：**{'identical': 37, 'n/a': 31}**
（逐站点明细见 `evidence.json`；不写覆盖类/通过类统计口径）。

## 2. 方法与判据

- 产品锚点：5 个产品源文件（`fluid_force/cuda.rs`/`kinetic_energy/cuda.rs`/`applied_force.rs`/
  `smooth_force.rs`/`passive_force.rs`）的 CUDA raw string；`product_anchor` 为 repo 行号 + 表达式原文。
- 参考锚点（全部带 file:line）：`passive.py` `_fluid_force`（359）/`geom_semiaxes`（54）/
  `_pow4`（48）`_pow2`（43）/`_qfrc_passive_kernel`（687），`support.py` `_apply_ft`（259），
  `sensor.py` `_energy_vel_kinetic`（3105），`forward.py` `_qfrc_smooth`（1975）。
- 分类六类词表：`identical` **37**（source 级表达式形态与次序对应；不声明位级）、
  `n/a` **31**（声明/指针/状态码/零初始化/局部量/纯拷贝/warp intrinsic/prose）、
  `engine-semantic` 0、`recipe-difference` 0、`not-decidable` 0、`pin-needed` 0。
- 该片核心对照：流体 body force 的椭球/盒分支逐式对应（added-mass、magnus、kutta、drag、viscous、
  box pow 链）；qfrc 投影与 `_apply_ft` 同式（`dot(jac,f)+dot(cross,top(f))`，b 升序、无原子）；
  动能归约与 Warp1.15 `tile_reduce` 的 32-lane 树（16/8/4/2/1）对应；applied/smooth/passive 的合成
  加法与上游同顺序（显式 `__fadd_rn/__fsub_rn` vs 普通 `+/−`，均一次 RN）。
- `not-decidable` 未使用：68 站点均有 source 锚点；设备级映射留给 RE-3。
- `reference_anchor.status` 用法沿用 C-1 词表口径（本片无 `no-reference-expression` 之外的未定义状态；
  全部 n/a 站点的 `status` 均为 `no-reference-expression`）。
- **实现依赖项**：`fluid_force` 盒分支 `powf(diam,3.0f)`/`powf(box*,4.0f)` 的位级随 NVRTC 版本，
  已记在 rounding-note:3 与对应 output 站点 note，**不宣称其位级**。

## 3. 自检

- 68/68 站点与 `sites.json` 的 id/kind/label 逐条一致；5 组计数 = 27/13/12/9/7。
- `pending_manual=0`；分类计数与逐条目重算一致；`not_decidable_reasons` 词表未被使用（0 条）。
- `policy_scan`（`scripts/np-gate/policy_scan.py`，`9a63e720…`/13482 B）
  对本目录 `evidence.json`/`README.md`/`receipt.json` 命中 0；工具 `--self-test` **28/28 PASS**
  （含英文否定声明用例）。
- 编码 UTF-8 / LF / 无 BOM；manifest 逐文件 sha256+bytes 自核；详见 `logs/selfcheck.txt`。

## 4. 第一节：C-1 复核低项/观察处置

- **L1（英文否定声明漏洞）**：`DECLARATION_NEGATORS` 补英文否定词（not/no/never/without/none/cannot
  及常见缩写）并加词边界检查——`never supersede`/`no withdrawal`/`must not supersede` 不再构成肯定声明、
  不豁免；`supersedes:` 等固定肯定式仍豁免。自测 24→**28/28 PASS**；
  工具新版 `9a63e720…`/13482 B。
- **O1（`reviewed` 字段语义）**：`--list-quote-blocks` 的 `reviewed` 只表示「块内存在 `reviewed_by`
  标记」；为 false 不代表未复核（痕迹可在证据 receipt 的 `quote_block_inventory`）。已写进工具
  docstring 与本 README 本节，避免误读；历史 A-2 豁免块字节不改、清单位于 C-1 receipt。
- **O2（field-table errata 记录）**：`flex_hessian_validate` 的 field-table 仍写 `ok` bool/`status` f32，
  当前产品 validate 只写 i32 私有 `flags[nf]=1`。本片仅记录（`errata_pending` `errata-0001`），
  **不改 field-table 历史**，收尾 addenda 卡统一入 errata 清单。

## 5. 输入与哈希

- 站点清单 `target/render-bvh/gl2-ptx-diff/np5-01a/sites.json` = `e0d4a95bf5c0acf13510b20f51e92ece1ec63ab7c7ac9e37be38cdd0841bac3c` / 261483 B。
- 场表（5 行，`docs/development-evidence/np01-type-mapping/field-tables/`）：
  - `fluid_force.json` `49828cd07d4b2bc89e8ec3ad445ed094a2685c51a82a5748f7ca6d9693074b98`
  - `kinetic_energy.json` `d4376959beb5bc9eb4dfbb99465a271b445b32ae6a833d0bedd89b006f4ec0f6`
  - `applied_force.json` `6b1a20ebe102125073d5ed3a50782a1888e59e43dadaec7d19b827cad706d9f6`
  - `smooth_force.json` `8dfd8e252c7be5c942972150b489923a9abf2ea10e6fb207359616c5657e72a7`
  - `passive_force.json` `65a310180e2e2e9ff5d44dfae98ae479927a1133b612badb74758538001f5644`
- spec `target/render-bvh/gl2-ptx-diff/np5-01a-exec-spec.md` = `5321f31258ca9ce02f344dd38c1d1d32ad2005fc1cd66b27f21fe1c8b86c5389` / 12402 B（工作材料，非证据）。
- 词表 `target/render-bvh/gl2-ptx-diff/np5-01a-vocab-addendum.md` = `453c5c9157341fa28ecb009c50f2ff560b6f43e15f593ed9b7a75f27dd5b1b26` / 6771 B（工作材料，非证据）。
- upstream revision `71da24d956378a87a703b6e1442b13aec0c4ac29`；`tree.head` `d112b7a4ba8028f252cce15408b3444203540fae`（本卡主提交父提交）。

## 6. 非声明

不宣称 NP01 通过/E 指标完成/上游等价/双平台/位级保证；不改判据/参考/容差；零 GPU、零窗；
force/energy 设备级映射不在本片范围（RE-3 后续）。
