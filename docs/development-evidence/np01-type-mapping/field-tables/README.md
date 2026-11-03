# NP01 逐字段类型表（B1 批）

任务：`2026-11-03-precision-np01-field-tables`。口径：`docs/numerical-precision-contract.md` 的 NP01
（类型映射率 100%、实际精度偏离 0）。本目录只做**类型审计与登记**：不改产品代码/判据/容差，零 GPU。
**NP01 仍未验收**——B1 只完成 5/30 条产品路径 + 0/3 条 host 站点。

## 一、表的粒度与判读

每条路径一张 `<row_id>.json`，逐项写清：

| 段 | 内容 |
| --- | --- |
| `source` | 产品源文件、`const` 名（内联 CUDA）或 `.cu` 文件名、内核名 |
| `kernels[].params` | 每个形参的名字、C 类型、角色（in / out / scalar / in-out） |
| `kernels[].intermediates` | 局部中间量与类型（f32 / 位型搬运 / 索引） |
| `kernels[].accumulations` | 每一处累加的类型与顺序（顺序 f32 加 / `atomicAdd` / `__fadd_rn`） |
| `kernels[].outputs` | 每个输出字段的类型、在输出块中的位置、转换/舍入点 |
| `quantization` | 原生 f64 → f32 的转换位置、契约 sentinel 的精确比较要求 |
| `rounding_notes` | 冻结的舍入/融合写法（如 `fmaf` + `__fmul_rn`）及其上游依据 |
| `upstream` | 上游文件与函数、置信级别 |
| `evidence` | 可复核的现有探针/证据路径 |
| `audit_level` / `status` | 本表审到哪一层、还剩什么 |

粒度边界：本批审到**参数/缓冲/累加/输出层**；逐表达式的位级对照仍由实现片的探针证据承担，
表里不重复也不冒充。`status` 里如实写该行仍缺什么（如多体原子归约的逐位重放记录、Linux 复验）。

## 二、B1 已完成（5 条）

| row_id | 路径 | 内核 | 累加方式 | 输出 |
| --- | --- | --- | --- | --- |
| `com_position` | `com_pos` | `rigid_com_position` | f32 树序顺序加（非原子） | sm/com/ci/cd（f32；com 处一次 f32 除法） |
| `com_velocity` | `com_vel`、`cdof_dot` | `com_velocity` | f32 顺序加（dof 按 adr 序） | cvel/cdof_dot（f32；motion_cross 分组括号固定加法序） |
| `gravity_compensation` | `gravity_compensation` | 4 内核（clear/validate/compute/sum） | `atomicAdd` f32 + `__fadd_rn` 显式单次舍入 | parts/dof 输出（f32）；sentinel `-131072` |
| `rne_bias` | `rne_bias` | `rne_bias` | f32 顺序加 + 树逆序加 | acc/frc/bias（f32）；重力种子位型搬运 |
| `rne_acc` | `rne_cacc`/`rne_cfrc`/`rne_backward`/`rne_qfrc` | 4 内核 | f32 顺序加 + `atomicAdd`（backward） | acc/frc/bias（f32）；`_cfrc` 冻结 `fmaf(i1,a1,__fmul_rn(i3,a0))` |

## 三、待补（25 条产品路径，按批排）

- **B1b（4 条，需逐输出中间量）**：`kinematics`（12 输出，142 行内核）、`crb`、`mass_multiply`、`mass_solve`（resident/sparse 多路径）。
- **B2（18 条，G10/G-limbs 面）**：`joint_passive`、`smooth_force`、`passive_force`、`fixed_tendon`、`spatial_tendon`、`tendon_wrap`、`tendon`、`tendon_velocity`、`tendon_passive`、`flex_position`、`flex_edge`、`flex_face`、`flex_hessian`、`flex_edge_passive`、`fluid_force`、`applied_force`、`kinetic_energy`、`attached`、`camlight`、`sleep_wake`、`field_batch`（按依赖再分两小批）。
- **B3（3 条 host 站点）**：`gjk_host`、`ccd_host`、`epa_host`（口径已在 `inventory.json` 的 `hostSiteRegistration` 登记，缺逐字段表）。

## 四、门禁联动

`tests/precision_type_gate.rs` 增加一条断言：`inventory.json` 的每一行必须在 `field-tables/` 下有表、
或在该测试内显式登记的「待批清单」里；表内 `kernels/params/intermediates/accumulations/outputs/upstream/evidence`
缺段即失败。待批清单随每批交付收紧——**不允许静默缺表**。

## 五、边界

- 不改 `src/`、`include/warp-headers/`、fixtures、判据/容差；零 GPU；不宣称 NP01 通过。
- 表中的「上游置信级别」沿用 `inventory.json` 原值，不升级。
