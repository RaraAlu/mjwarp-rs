# NP5-01a RE-1：mass_solve 设备级对照（A1-final 6 族/12 变体参考 PTX；PTX 不入仓；零 GPU）

- 任务：`2026-11-11-np5-01a-re1-mass-solve-device-level`（owner Backend；p95；P8-渲染与BVH）。
- 材料：Arch `A1-final`（`np5-01a-a1-final.md` `55b15ada38b548ed…`/5803 B；
  `refgen/a1/final/A1-final.json` `fb4c8ad098cdc905…`/11079 B）+ `A0` 值级核对
  （`799b0ef56808d845…`）。词表以 `np5-01a-vocab-addendum.md`（`46282084cbe884e5…`）为准。
- 边界：**PTX 实体不入仓**（A1-final 12 变体 + 产品 PTX 只记 sha/bytes/命令，可由脚本+pinned 环境确定性重生成）；
  零 GPU、零窗；不改 `src/`/`tests/`/`fixtures/`/`scripts/`/判据/参考/容差。

## 1. A1-final（factor/solve 6 族 / 12 static 变体）

| 族（factory） | 变体（static） | PTX sha256（前 16） | bytes |
| --- | --- | --- | --- |
| `_tile_cholesky_factorize_block` | `tile_chol_factorize_sz11` [11] | `c9dc182a1de2b74a…` | 52955 |
| `_tile_cholesky_solve_block` | `tile_chol_solve_sz11` [11] | `746f9d28fd72fa77…` | 57461 |
| `_tile_cholesky_factorize_solve_block` | `tile_chol_factor_solve_sz11` [11] | `65e790b9d90d4316…` | 73840 |
| `_small_cholesky_factorize_block` | `small_chol_factorize_bs1` [1] | `ce8b25bc8aa6fadd…` | 35644 |
| `_small_cholesky_solve_block` | `small_chol_solve_bs1` [1] | `3638aba3601506b4…` | 37419 |
| `_small_cholesky_factorize_solve_block` | `small_chol_factor_solve_bs1` [1] | `96f2a2a620ab6e72…` | 35866 |
| `_tile_cholesky_factorize_block` | `tile_chol_factorize_sz12` [12] | `43b3586145343a74…` | 53852 |
| `_tile_cholesky_solve_block` | `tile_chol_solve_sz12` [12] | `9095dec6f8408934…` | 57473 |
| `_tile_cholesky_factorize_solve_block` | `tile_chol_factor_solve_sz12` [12] | `35585c97e306f443…` | 74758 |
| `_small_cholesky_factorize_block` | `small_chol_factorize_bs2` [2] | `50194511656e3170…` | 34700 |
| `_small_cholesky_solve_block` | `small_chol_solve_bs2` [2] | `49fe2ceca925b2a3…` | 35845 |
| `_small_cholesky_factorize_solve_block` | `small_chol_factor_solve_bs2` [2] | `1bb65d6443211b68…` | 39045 |

- 6 族：`_small_cholesky_factorize_block` / `_small_cholesky_solve_block` /
  `_small_cholesky_factorize_solve_block` / `_tile_cholesky_factorize_block` /
  `_tile_cholesky_solve_block` / `_tile_cholesky_factorize_solve_block`；static 参数由 A0 参考模型
  （mixed-joints / inertial-tree / rotated-tree / zero-dof / massless-tree / armature-chain）复刻 `m_block_layout` 推导。
- 4 个 host 驱动（`_factor_i_sparse`/`_factor_blocks`/`_factor_solve_blocks`/`_solve_LD_sparse`）本身无 PTX；
  `_solve_LD_sparse_fused`/`_factor_solve_lu_sparse_fused` 因 A0 模型无 >64 dof tree 未触发（capture-scenario-miss）。
- 产品 PTX：NVRTC 12.8.93 / compute_70 / -lineinfo 离线编译 `MASS_SOLVE_CUDA`（entry `rigid_mass_solve`），
  sha256 `46f0af89d5e2c53f…`/60278 B（不入仓）。

## 2. Tile 站点设备级结论（3 条）

| 站点 | 产品形态 | 参考形态（A1-final） | 分类 |
| --- | --- | --- | --- |
| A-1a `rigid_mass_solve:accumulation:0`（tile Schur factor） | `mul.f32`/`sub.f32` 可收缩（`tile_schur_update` 内联 asm） | `_tile_cholesky_factorize_block`：`mul.f32`/`sub.f32` 可收缩（+ sqrt.rn/rcp.rn 属对角步） | **identical**（归一化指令形态；B 类前提、不写位级） |
| A-1a `rigid_mass_solve:accumulation:1`（tile 对角步） | `rsqrt.approx.f32` + `mul.rn.f32`（MathDx 冻结 PTX 形态） | 参考 scalar 回退 `sqrt.rn.f32` + `rcp.rn.f32` | **not-decidable**（approx vs exact、非值等价；词表适配需裁，见 §4） |
| A-1b `rigid_mass_solve:accumulation:5`（tile 求解列扫描） | `rcp.rn`×2 + `mul.rn` + `fma.rn` 更新 | `_tile_cholesky_solve_block`：`div.rn`×2 + `mul.f32`/`sub.f32` 可收缩 | **recipe-difference**（`value_equivalent: true` + `value_equivalence_basis: "analytic, not bit-tested"`） |

- 三条均为 **device-level 归一化指令形态对照**；B 类前提：参考 PTX 为 NVRTC 12.9 / sm_89 / block_dim=32 离线生成，
  产品为 12.8.93 / compute_70；**不写字节相等、不写位级等价**。
- `recipe-difference` 的 MathDx 注记：**MathDx 为产品 opt-in 路线、默认路径不走**；若未来变默认路线需重评。

## 3. A-1a 裁定回写（R1–R4，append-only 落点 `np5-01a-a1a/addenda.md`）

- **R1** `intermediate:3` → `engine-semantic` 认可（同一数学解、不同分解路径、位级不保证；不得当 identical）。
- **R2** `accumulation:3` → **改判 `recipe-difference`**（`value_equivalent: true` + `analytic, not bit-tested`；
  MathDx opt-in/默认不走/未来变默认需重评；不记 pin-needed、不进 deferred 偏离清单；NP5-04 仅 watch 项）。
- **R3** `accumulation:0/1` → 保持 `not-decidable`；本卡把 **accumulation:0 升级为 `identical`**（§2），
  **accumulation:1 保持 `not-decidable` 并列入待裁**（§4）。
- **R4** `accumulation:2` → `identical` 前提补「收缩许可、未证位级相等」。

## 4. 待裁点（requires_decision）

- `mass_solve:rigid_mass_solve:accumulation:1`：产品 MathDx tile 对角步（`rsqrt.approx`）vs 参考 scalar 回退
  （`sqrt.rn`+`rcp.rn`）为 **approx 与 exact 的库实现差异**（值不预期等价）。原 spec §3 的
  `recipe-difference` 覆盖「approx 库实现差异（量化、单列）」；但 vocab addendum 要求 `recipe-difference`
  必须 `value_equivalent: true`，此处不成立 → 本卡暂记 `not-decidable`，请 Chief/Arch 裁口径（是否允许
  `recipe-difference` + 非零期望量级、或维持 not-decidable）。

## 5. A0 值级旁证（B 类，只作旁证）

- `matrix`/`factor` 与 upstream（rev `71da24d9`）**differentBits=0**；`solution` 19 bits / 6.1e-5；
  acceptance NP03 maxResidual 1.13e-7 / maxReconstruction 4.07e-7（scope=G03 helper remaining-error repair）。
- 用途：**只作值级旁证**（尤其 factor），不得当位级证据；solution 相关站点在后续 C 批引用时附残差说明。

## 6. 输入与哈希

- A1-final JSON `fb4c8ad098cdc905c5adc16623073fb5f59763b8d1fd01822192ad560bcf48e9`；A1-final md `55b15ada38b548ed71383b21fb94ec866d3cefcbac89df8cbe82145da7ab3a8c`；A0 md `799b0ef56808d845dedfbadf5e190b6da22d821843d2462c915e0cb0647e9709`；spec `5321f31258ca9ce02f344dd38c1d1d32ad2005fc1cd66b27f21fe1c8b86c5389`；词表 `46282084cbe884e53446155f0a2ed44336880367651661376b4a1e5d081f9591`。
- tree.head（生成时 HEAD）`371119d6bca61851895020374e6db9549abc58f9`（本卡主提交父提交）。

## 7. 非声明

不宣称 NP01 通过/E 指标完成/上游等价/双平台/位级等价；不改判据/参考/容差；零 GPU、零窗；PTX 实体不入仓。
