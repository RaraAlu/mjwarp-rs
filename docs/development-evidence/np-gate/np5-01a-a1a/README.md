# NP5-01a A-1a：mass_solve factor 半段 source 级逐站点对照（40 站点；零 GPU）

- 任务：`2026-11-11-np5-01a-a1a-mass-solve-factor-source-level`（owner Backend；p95；P8-渲染与BVH）。
- 依据：`np5-01a-SLICE-PLAN.md` A-1a 口径 + target 预热草稿（`A-1-split.json` A-1a 列表，逐 id 一致）+
  `np5-01a-vocab-addendum.md`（六类词表，`fcbec8f9…`/4,560 B 为唯一口径）。
- 边界：source-level 对照，**零 GPU、零窗**；不改 `src/`/`tests/`/`fixtures/`/`scripts/`/场表/判据/参考/容差。
- 词表：`identical` / `pin-needed` / `recipe-difference` / `engine-semantic` / `not-decidable` / `n/a`。
  `recipe-difference` 必带 `value_equivalent: true` + `value_equivalence_basis: "analytic, not bit-tested"`；
  `not-decidable` 必带 6 类 `not_decidable_reasons[]` + 缺证说明；**本文不写覆盖率/通过率**。

## 1. 站点与分类计数（A-1a = factor 半段 40）

| 组 | 站点数 | 说明 |
| --- | --- | --- |
| `rigid_mass_solve` factor 类 | 13 | intermediate 0–4 / accumulation 0–4 / output 0–2 |
| `resident_mass_factor（派生）` | 4 | split_sources 派生，算术文本与 rigid factor 半段逐字相同 |
| `sparse_mass_copy/update/diag` | 7 | 稀疏 LDL 因子三步 |
| `small_prefix_validate/small_prefix_factor_2..6` | 8 | 2..6 阶小 Cholesky |
| `mixed_factor_merge` | 2 | 小阶/稀疏拼接 |
| `sparse_dense_validate/gather` | 3 | 稠密化输入（纯搬运/校验） |
| rounding-note 0/2/3 | 3 | 说明性舍入注记 |

分类计数：**{'n/a': 22, 'engine-semantic': 1, 'not-decidable': 2, 'identical': 12, 'pin-needed': 1, 'recipe-difference': 2}**（明细逐站点在 `evidence.json`；`not_decidable` 原因直方图 {'no-reference-entry': 2, 'capture-scenario-miss': 2}）。

## 2. 方法（7 步校准法落地）

1. 组→上游函数映射（`_factor_i_sparse`/`_qLD_acc`/`_qLDiag_div`/`_small_cholesky_factorize_block`/
   `_tile_cholesky_factorize_block`/`_factor_blocks`，行号见各 record `reference_anchor`）。
2. 产品锚点校正：exact-snippet 直接采用；token 候选逐条在 kernel 内核对；resident 派生回到
   `src/physics/mass_solve/cuda.rs` 原内核 factor 半段（不取 split_sources 行）。
3. 参考侧定位表达式/设备函数行号（`smooth.py`、`include/warp-headers/tile_cholesky.h`）。
4. 类型映射：产品场表 type vs 上游 `wp.array2d[float]`/float；**全链 f32、产品 double=0**。
5. 分类：见 §1；`identical` 均为 **source 级**（`basis: source-level（无位级测试）`），**不声明位级等价**。
6. 写回 `evidence.json`（40 条，五项必填字段齐全）。
7. 自检 `logs/selfcheck.txt`：40/40 id 一致、pending=0、词表合法、理由码合法、必填字段齐全、无未加否定的通过类表述 → **PASS**。

## 3. 判断说明与待复核项（`evidence.json.requires_decision`）

- `rigid_mass_solve:intermediate:3`：tile/scalar 路径选择语义（产品按块大小>6、上游按 `M_tiles/elemid`）
  记为 `engine-semantic`，请 Arch/QA 复核是否认可。
- `rigid_mass_solve:accumulation:3`：产品按自身冻结 MathDx PTX 用 reciprocal 乘、上游 `_qLD_acc`
  用除法；记 `pin-needed`（≈1 ULP 级），请复核或改判。
- `rigid_mass_solve:accumulation:0/1`：tile 路径 `not-decidable`（参考 unique 内核未 capture）；
  A1 离线 codegen 成功后应升级，本片按现证据写。
- `rigid_mass_solve:accumulation:2` 记 `identical` 的前提：上游 `-=` 允许收缩（fmad=true），
  产品显式 `fmaf` 同形；**不声明位级**。

## 4. 输入与哈希

- 场表：`docs/development-evidence/np01-type-mapping/field-tables/mass_solve.json` = `964dab77d4c0f3fbec8be7437eb330f7fcc03ad93854b28be9f2d7d5dc1efa02`
- 规格：`target/.../np5-01a-exec-spec.md` = `5321f31258ca9ce02f344dd38c1d1d32ad2005fc1cd66b27f21fe1c8b86c5389`（工作材料，非证据）
- 词表：`target/.../np5-01a-vocab-addendum.md` = `46282084cbe884e53446155f0a2ed44336880367651661376b4a1e5d081f9591`（工作材料，非证据）
- 预热拆分：`target/.../A-1-split.json` = `ec5fc25713dbe64782d0db87b8de032977711fb3fd4716db6d7b0f6c612024b1`（工作材料，非证据）
- upstream revision：`71da24d956378a87a703b6e1442b13aec0c4ac29`（pinned，只读）
- tree.head（生成时 HEAD）：`427bfca51a23a10108ee6ead28bf4c6b32b985cf`（本卡主提交的父提交）

## 5. 非声明

不宣称 NP01 通过/E 指标完成/上游等价/双平台/位级等价；E 指标未定稿（用户 2026-10-27）；
不改判据/参考/容差；零 GPU、零窗；本片只登记 source 级对照事实与分类。
