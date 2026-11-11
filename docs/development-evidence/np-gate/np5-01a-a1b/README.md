# NP5-01a A-1b：mass_solve solve 半段 source 级逐站点对照（36 站点；零 GPU）

- 任务：`2026-11-11-np5-01a-a1b-mass-solve-solve-source-level`（owner Backend；p95；P8-渲染与BVH）。
- 依据：`np5-01a-SLICE-PLAN.md` A-1b 口径 + `A-1-split.json` A-1b 列表（逐 id 一致）；词表以
  `np5-01a-vocab-addendum.md`（`fcbec8f9…`/4,560 B）为准。
- 边界：source-level、**零 GPU、零窗**；不改 `src/`/`tests/`/`fixtures/`/`scripts/`/场表/判据/参考/容差。

## 1. 站点与分类计数（A-1b = solve 半段 36；A-1a 40 + A-1b 36 = mass_solve 76）

| 组 | 站点数 | 说明 |
| --- | --- | --- |
| `rigid_mass_solve` solve 类 | 5 | intermediate:5（tmp 局部量）/accumulation:5（tile 求解）/6（scalar 求解）/7（kind2 求解）/output:3 |
| `resident_mass_substitute` | 4 | split_sources 派生，solve 半段文本与 rigid 逐字相同 |
| `mixed_solve_validate/mixed_solve_fused` | 9 | 小阶 Schur + level 原子更新 + D 分支 |
| `sparse_solve_validate/sparse_solve_fused` | 8 | 稀疏 LD 前代/对角/回代 |
| `resident_sparse_validate/gather` | 4 | resident 两阶段 gather（纯搬运/校验） |
| `resident_mixed_validate/gather` | 4 | 同上 |
| rounding-note 1/4 | 2 | tile 求解语义 / resident 派生机制说明 |

分类计数：**{'n/a': 21, 'not-decidable': 1, 'identical': 11, 'engine-semantic': 3}**（逐站点明细见 `evidence.json`；无覆盖率/通过率表述）。

## 2. 方法（7 步法落地）

1. 组→上游函数映射：`_solve_LD_sparse_fused`（smooth.py:3314）、`_small_cholesky_solve`（3404）、
   `_small_cholesky_solve_block`（3430）、`_tile_cholesky_solve_block`（3457）、`_solve_blocks`（3486）。
2. 产品锚点：solve 半段落 `cuda.rs:89-130`、`resident.rs:94-97`（派生）、`mixed/solve/cuda.rs`、
   `sparse/resident/solve/cuda.rs`、resident gather 两个 `.cu`。
3. 参考侧行号见各 record `reference_anchor`；unique 内核（tile solve）A1-final 已有离线 PTX，设备级对照留 RE-1。
4. 类型映射：全链 f32、产品 double=0；bool/int 站点单独标注。
5. 分类：`identical` 均为 **source 级**（不声明位级）；`engine-semantic` 3 条（kind2 块内循环 vs level 扫描、
   D 乘分支时机 ×2）；`not-decidable` 1 条（tile solve 设备级留 RE-1，带 `no-lineinfo`/`frozen-artifact-only`）。
6. 写回 evidence.json（36 条，五项必填字段齐全）。
7. 自检 `logs/selfcheck.txt`：**PASS**（36/36、pending=0、A-1a+A-1b=76、词表/理由码/必填齐全、无未加否定通过类表述）。

## 3. A-1a 裁定回写落点（Chief 2026-11-11 修订版）

A-1a 的 4 条待复核项由 Chief 修订裁定，**append-only 记录在 `np-gate/np5-01a-a1a/addenda.md`**（不改 A-1a
`evidence.json` 字节；A-1a manifest 已重算列入该文件）：

- **R1** `rigid_mass_solve:intermediate:3` → 认可 `engine-semantic`（同一数学解、不同分解路径、位级不保证；不得当 `identical`）。
- **R2** `rigid_mass_solve:accumulation:3` → **改判 `recipe-difference`**（带 `value_equivalent: true` +
  `value_equivalence_basis: "analytic, not bit-tested"`）；注明 MathDx 为产品 **opt-in 路线、默认路径不走**，
  未来若变默认需重评；**不记 pin-needed、不进 deferred 偏离清单**，NP5-04 只记 watch 项。
- **R3** `rigid_mass_solve:accumulation:0/1` → **保持 `not-decidable`**（不引入新词）；note 记「A1 codegen 已具备，
  由 RE-1 升级为 device 级」。
- **R4** `rigid_mass_solve:accumulation:2` → `identical` 前提补「收缩许可、未证位级相等」不变。

## 4. RE-1 交接

- A-1a `acc:0/1` 与 A-1b `acc:5`（tile solve）由 **RE-1 用 A1-final PTX 升级 device 级**（tile factor/solve sz11/sz12 已生成）。
- A-1a `acc:3` 按 R2 回写口径 `recipe-difference`，RE-1 引用；solution 站点附 A0 值级旁证说明。
- 4 个 host 驱动（`_factor_i_sparse`/`_factor_blocks`/`_factor_solve_blocks`/`_solve_LD_sparse`）记 `host-path-no-ptx`；
  `_solve_LD_sparse_fused` 等记 `capture-scenario-miss`。

## 5. 输入与哈希

- 场表 `docs/development-evidence/np01-type-mapping/field-tables/mass_solve.json` = `964dab77d4c0f3fbec8be7437eb330f7fcc03ad93854b28be9f2d7d5dc1efa02`
- 规格 `target/.../np5-01a-exec-spec.md` = `5321f31258ca9ce02f344dd38c1d1d32ad2005fc1cd66b27f21fe1c8b86c5389`（工作材料）
- 词表 `target/.../np5-01a-vocab-addendum.md` = `46282084cbe884e53446155f0a2ed44336880367651661376b4a1e5d081f9591`（工作材料）
- 拆分 `target/.../A-1-split.json` = `ec5fc25713dbe64782d0db87b8de032977711fb3fd4716db6d7b0f6c612024b1`（工作材料）
- upstream revision `71da24d956378a87a703b6e1442b13aec0c4ac29`；tree.head = `f5ded6c942558e9d231c487f790f332dffdbf18f`（本卡主提交父提交）

## 6. 非声明

不宣称 NP01 通过/E 指标完成/上游等价/双平台/位级等价；E 未定稿（用户 2026-10-27）；不改判据/参考/容差；零 GPU、零窗。
