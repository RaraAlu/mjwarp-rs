# NP5-01a RE-1 复核（QA）：mass_solve 设备级对照（A1-final 6 族/12 变体；3 tile 站点定级）

- 复核对象：卡 `2026-11-11-np5-01a-re1-mass-solve-device-level`；landing 提交 **`a6f5a472`**；`acc:1` 改判修正 **`d6e0dba5`**（`Normify-Fixes`）。
- 性质：只读复核（零 GPU、零窗、不改判据/参考/容差）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，0 一般 + 1 低 + 2 观察）**。

## 1. A1-final 6 族 / 12 变体（我逐文件复算）

- 12 个参考 PTX 在 `target/render-bvh/gl2-ptx-diff/refgen/a1/final/` 全部在盘，**sha256/bytes 与 README §1、`evidence.json.a1_final.variants` 逐项一致（12/12）**：
  `tile_chol_factorize_sz11` `c9dc182a…`/52,955、`tile_chol_solve_sz11` `746f9d28…`/57,461、`tile_chol_factor_solve_sz11` `65e790b9…`/73,840、`small_chol_factorize_bs1` `ce8b25bc…`/35,644、`small_chol_solve_bs1` `3638aba3…`/37,419、`small_chol_factor_solve_bs1` `96f2a2a6…`/35,866、`tile_chol_factorize_sz12` `43b35861…`/53,852、`tile_chol_solve_sz12` `9095dec6…`/57,473、`tile_chol_factor_solve_sz12` `35585c97…`/74,758、`small_chol_factorize_bs2` `50194511…`/34,700、`small_chol_solve_bs2` `49fe2cec…`/35,845、`small_chol_factor_solve_bs2` `1bb65d64…`/39,045。
- 每变体带 factory/static_args/entry/`ptx_sha256`/`ptx_bytes`/`landed:false`/`regeneration`（脚本+pinned NVRTC 12.9 命令）✅；产品 PTX `46f0af89d5e2c53f…`/60,278（`compute_70`/NVRTC 12.8.93/-lineinfo）与磁盘一致，`landed:false`。
- **PTX 不入仓口径成立**：目录内无 `.ptx`；`manifest` 头注「PTX artifacts are NOT landed」；两份生成物只以 sha/bytes/命令记录。

## 2. 三个 tile 站点设备级定级（含 acc:1 改判）

- **acc:0 → `identical`**：产品 `mul.f32`/`sub.f32` 可收缩（`tile_schur_update` 内联 asm）vs 参考 `_tile_cholesky_factorize_block` 同形态；**B 类前提**（参考 12.9/sm_89/block_dim=32 vs 产品 12.8.93/compute_70）明写、不写位级 ✅
- **acc:1 → `engine-semantic`（`d6e0dba5` 改判）**：`addenda.md` 写明 `why_engine_semantic`（产品 opt-in MathDx tile 路径 `rsqrt.approx` vs 参考默认 scalar `sqrt.rn`/`rcp.rn`；实现路线不同、数值非严格等价 approx vs exact）、`bit_level_guarantee:false`、note（MathDx opt-in、默认不走、若变默认需在 E 定稿后重评）、**NP5-04 记 watch 项**；并明写**不得**用 `recipe-difference`（`value_equivalent` 不成立）、**不得**用 `pin-needed`（无 pin 规格）✅
- **acc:5 → `recipe-difference`**：带 `value_equivalent:true` + `value_equivalence_basis:"analytic, not bit-tested"` + product/reference form + why + `bit_tested:false`；MathDx opt-in 注记在位 ✅
- **指令形态我独立复算**（自写计数器逐 PTX 重算）：`tile_chol_factorize_sz11` = `ld.shared.f32 37 / mul.f32 22 / mov.f32 14 / sub.f32 14 / st.shared.f32 13 / st.global.f32 7 / ld.global.f32 4 / sqrt.rn.f32 1 / rcp.rn.f32 1`；`tile_chol_solve_sz11` = `div.rn.f32 2 / mul.f32 14 / sub.f32 14 / ld.shared.f32 42 / st.shared.f32 23 / …`；产品 PTX 的 tile 段 `rsqrt.approx.f32 1`、`rcp.rn.f32 2 + mul.rn.f32 2 + fma.rn.f32 14` —— **与 `evidence.json.device_sites[*].forms` 逐项一致**。
- **`no-lineinfo` 佐证**：参考 12 份 PTX 均无 `.loc`，产品 PTX 有 `.loc`（我直接检查）⇒ acc:1 的缺证理由与 B 类前提成立。

## 3. A-1a 裁定回写、host 驱动、A0 旁证

- R1–R4 回写与 Chief 修订裁定逐条一致（R2 改判 recipe-difference + analytic 字段；R3 保持 not-decidable + A1/RE-1 升级；R4 identical 前提补「收缩许可、未证位级」）✅
- host 驱动 4 条（`_factor_i_sparse`/`_factor_blocks`/`_factor_solve_blocks`/`_solve_LD_sparse`）记 `host-path-no-ptx`；`_solve_LD_sparse_fused`/`_factor_solve_lu_sparse_fused` 记 `capture-scenario-miss`，各带 why ✅
- A0 值级旁证按 **B 类旁证**写：factor/matrix `differentBits=0`、solution 19 bits/6.1e-5、NP03 maxResidual 1.13e-7 / maxReconstruction 4.07e-7；**明写不得当位级证据**、solution 站点附残差说明 ✅

## 4. append-only（`d6e0dba5`）、manifest、门禁、边界

- **landing 字节未改**：`a6f5a472` → `d6e0dba5`，`evidence.json`(18,960)/`README.md`(6,786)/`receipt.json`(3,120)/`logs/selfcheck.txt`(334) **逐 blob 完全相同**。
- `addenda.md` = **1,775 B / `e17d0e97…`**（与派工值一致），append-only 新增；`manifest.txt` 我独立重算 **5 条全对**（README/addenda/evidence/receipt/logs，不自列；集合与目录 5 个非 manifest 文件相等），sha `2115dcdc…`/732。
- 门禁：fmt/clippy/clippy-cuda exit 0、unit **1,406/0/402**；卡 fix→gate→done 后 `status=verified`。
- 边界：`d6e0dba5` 仅 `.normify`（卡+4 gate 日志）+ `np5-01a-re1/addenda.md` + `manifest.txt`；无 `src/tests/fixtures/scripts` 改动；编码 UTF-8/LF 无 BOM/CR/NUL。
- 措辞：RE-1 目录内「覆盖率/通过率/位级等价/字节相等/上游等价/双平台/NP01 通过」均为否定句；`recipe-difference`/`pin-needed` 在 addenda 中只出现在「不得用」的禁止语境。

## 5. 过程事件（Backend 自报，按纪律记录，不额外开卡）

- Backend 自报：**18:15:09** 在 QA 持锁期间**未先核锁状态**即执行 `normify fix <RE-1 卡>`，致 `.normify/changes/…re1….json` 出现本地修改（**未提交、未再写仓**）；18:24:33 QA 释放后 acquire，在持锁态补做 addenda/manifest/gate/done/提交。该事件已由 Backend 记入 `np5-01a-re1/addenda.md` §过程事件；我核对该脏文件在我落卡全程未被误提交（我落卡一律 pathspec 暂存自己文件），门禁未受影响。

## 6. 发现

### 低
- **L1 manifest 计数口径**：manifest 实列 **5 条**（目录 6 个文件含 manifest 自身），提交说明/派工写「6 条含 addenda」是**含自身**的计数。与卡 L、A-1a 同类口径问题；建议统一写「N/N，不自列」。

### 观察
- **O1 `requires_decision` 历史残留**：`evidence.json` 仍保留 acc:1 的旧 `requires_decision`（append-only 设计），addenda 已明示取代；建议后续收口卡给「按现口径的派生分类计数」（acc:0 identical / acc:1 engine-semantic / acc:5 recipe-difference）与 requires_decision 去留说明。
- **O2 README 历史表述**：README §3/§4 仍写 acc:1 `not-decidable`/待裁（历史字节不改），现口径以 addenda 为准；读数时须交叉引用。

## 7. 非声明

本复核确认 RE-1 的 6 族/12 变体身份、3 站点定级（含 acc:1 改判）与 append-only 不变式；**不宣称** NP01 通过/精度完成/位级等价/上游等价/双平台，不改判据/参考/容差。
