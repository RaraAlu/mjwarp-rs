# NP5-01a A-2：kinematics + crb source 级逐站点对照（63 站点；零 GPU）

- 任务：`2026-11-11-np5-01a-a2-kinematics-crb-source-level`（owner Backend；p95；P8-渲染与BVH）。
- 依据：`np5-01a-SLICE-PLAN.md` A-2 口径；站点 = `np5-01a/sites.json` 的 kinematics 33 + crb 30；词表
  `np5-01a-vocab-addendum.md`（`46282084cbe884e5…`）。
- 边界：source-level、零 GPU、零窗；不改 `src/`/`tests/`/`fixtures/`/`scripts/`/判据/参考/容差。

## 1. 站点与分类计数（63 = 33 + 30；NP5-01a 累计 139）

| 组 | 站点 | 说明 |
| --- | --- | --- |
| `kinematics.rigid_kinematics` | 19 | 5 intermediate / 5 accumulation / 9 output |
| `kinematics.rigid_com_position` | 14 | 4 intermediate / 4 accumulation / 6 output |
| `kinematics.rounding-note` | 5 | 说明性注记（n/a） |
| `crb.rigid_mass_matrix` | 12 | 6 intermediate / 4 accumulation / 2 output |
| `crb.resident_crb_init` | 5 | 1 intermediate / 2 accumulation / 2 output |
| `crb.resident_crb_accumulate` | 3 | 1 intermediate / 1 accumulation / 1 output |
| `crb.resident_crb_matrix` | 7 | 3 intermediate / 3 accumulation / 1 output |
| `crb.rounding-note` | 3 | 说明性注记（n/a） |

分类计数：**{'n/a': 32, 'identical': 27, 'engine-semantic': 4}**（逐站点明细见 `evidence.json`；无覆盖率/通过率表述）。

## 2. 方法与判据

- 产品锚点：`src/physics/mod.rs` 的 `KINEMATICS_CUDA` raw string（含 `rigid_kinematics`/`rigid_com_position`）与
  `src/physics/mass_matrix.rs` 的 `MASS_MATRIX_CUDA`（`rigid_mass_matrix` 及 resident 变体）；行号为 repo 行号。
- 参考锚点：上游 `smooth.py` `_kinematics_branch`（48–146）、`_compute_body_matrices`（169–176）、
  `_site_local_to_global`（210–227）、`_crb_accumulate`（1000–1015）、`crb`（1050–1068）、`_M`、
  `_subtree_com_init/acc/div`、`_cinert`、`_compute_body_inertial_frames`，以及 `math.py` `inert_vec`（121–130）、
  `rot_vec_quat`/`mul_quat`/`quat_to_mat`——**每条均带 file:line**（`reference_anchor`）。
- 分类六类词表：`n/a` 32（声明/缓冲/视图/纯拷贝/零初始化/prose）、`identical` 27（source 级；不声明位级）、
  `engine-semantic` 4（树序 atomic 累加 vs 确定性顺序、惯量旋转实现路线、armature 加法位置、常驻原子汇聚）——
  这些差异均为**实现路线/调度语义差异**，不作数值等价结论。
- 产品侧显式对齐注释（`matrix_from` 复刻冻结 `quat_to_mat` 独立 mul/add/sub；`matrix_natural` 与冻结内核一致；
  `mmul/madd/msub/mfma` 复刻冻结 `math.inert_vec`/`wp.dot` 序列）作为 source 级分类依据记录，**不声明位级**。

## 3. 输入与哈希

- 场表 kinematics `dccba32da8c1678125d203a99ff72815e677b95ec42cd35e79f8f221767a156c`；crb `184ae56752bd8e3657ece61acc038d6c84d19f79076b01a2450dfe7133f80cda`。
- spec `5321f31258ca9ce02f344dd38c1d1d32ad2005fc1cd66b27f21fe1c8b86c5389`；词表 `46282084cbe884e53446155f0a2ed44336880367651661376b4a1e5d081f9591`（工作材料，非证据）。
- upstream revision `71da24d956378a87a703b6e1442b13aec0c4ac29`；tree.head `cb13e7b3379b333a0587f7b20ebb5d1399f0e077`（本卡主提交父提交）。

## 4. 非声明

不宣称 NP01 通过/E 指标完成/上游等价/双平台/位级等价；不改判据/参考/容差；零 GPU、零窗。
