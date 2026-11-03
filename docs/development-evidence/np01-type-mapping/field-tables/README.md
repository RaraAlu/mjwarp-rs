# NP01 逐字段类型表（B1 / B1b 批）

任务：`2026-11-03-precision-np01-field-tables`（B1，5 条）、
`2026-11-04-precision-np01-field-tables-b1b`（B1b，4 条）。口径：`docs/numerical-precision-contract.md` 的
NP01（类型映射率 100%、实际精度偏离 0）。本目录只做**类型审计与登记**：不改产品代码/判据/容差，零 GPU。
**NP01 仍未验收**——B1 + B1b 只完成 9/30 条产品路径 + 0/3 条 host 站点。

## 一、表的粒度与判读

每条路径一张 `<row_id>.json`，逐项写清：

| 段 | 内容 |
| --- | --- |
| `source` / `extra_sources` | 产品源文件、`const` 名（内联 CUDA）或 `.cu` 文件名、内核名；多文件/多内核族用 `extra_sources` 列全 |
| `kernels[].params` | 每个形参的名字、C 类型、角色（in / out / scalar / in-out） |
| `kernels[].intermediates` | 局部中间量与类型（f32 / 位型搬运 / 索引） |
| `kernels[].accumulations` | 每一处累加的类型与顺序（顺序 f32 加 / `atomicAdd` / `__fadd_rn` / 原子 `atom.global.add.f32`） |
| `kernels[].outputs` | 每个输出字段的类型、在输出块中的位置、转换/舍入点 |
| `quantization` | 原生 f64 → f32 的转换位置、契约 sentinel 的精确比较要求 |
| `rounding_notes` | 冻结的舍入/融合写法（如 `fmaf` + `__fmul_rn`、可收缩 PTX）及其上游依据 |
| `upstream` | 上游文件与函数、置信级别（沿用 inventory 原值，不升格） |
| `evidence` | 可复核的现有探针/证据路径（须真实存在） |
| `audit_level` / `status` | 本表审到哪一层、还剩什么 |

粒度边界：B1 审到**参数/缓冲/累加/输出层**；B1b 对重路径额外登记**显式 PTX 舍入口径**（未加 `.rn` 的可收缩
PTX vs 显式 `.rn`）。逐表达式的位级对照仍由实现片的探针证据承担，表里不重复也不冒充。`status` 里如实写该行
仍缺什么（如多体原子归约的逐位重放记录、Linux 复验、逐表达式审计）。

机器扫描口径（`raw-kernels.json`）与表的对应关系：`fma` 计数是**源文本站点**（正则 `fmaf|__fmaf|fma(`），
不是运行时执行次数；`atomicAdd` 计数只认 `atomicAdd` 文本，**不含**内联 `atom.global.add.f32`。B1b 表在
`accumulation_metrics` 段把这两个数字落到具体站点并注明差异，避免把扫描口径读成执行次数。

## 二、B1 已完成（5 条）

| row_id | 路径 | 内核 | 累加方式 | 输出 |
| --- | --- | --- | --- | --- |
| `com_position` | `com_pos` | `rigid_com_position` | f32 树序顺序加（非原子） | sm/com/ci/cd（f32；com 处一次 f32 除法） |
| `com_velocity` | `com_vel`、`cdof_dot` | `com_velocity` | f32 顺序加（dof 按 adr 序） | cvel/cdof_dot（f32；motion_cross 分组括号固定加法序） |
| `gravity_compensation` | `gravity_compensation` | 4 内核（clear/validate/compute/sum） | `atomicAdd` f32 + `__fadd_rn` 显式单次舍入 | parts/dof 输出（f32）；sentinel `-131072` |
| `rne_bias` | `rne_bias` | `rne_bias` | f32 顺序加 + 树逆序加 | acc/frc/bias（f32）；重力种子位型搬运 |
| `rne_acc` | `rne_cacc`/`rne_cfrc`/`rne_backward`/`rne_qfrc` | 4 内核 | f32 顺序加 + `atomicAdd`（backward） | acc/frc/bias（f32）；`_cfrc` 冻结 `fmaf(i1,a1,__fmul_rn(i3,a0))` |

## 三、B1b 已完成（4 条）

| row_id | 路径 | 内核 | 累加/舍入口径 | 输出 |
| --- | --- | --- | --- | --- |
| `kinematics` | 12 输出跨 3 内核 | `rigid_kinematics`；`rigid_com_position`（见 com_position 表）；`rigid_mass_matrix`（见 crb 表） | f32；xmat 用未加 `.rn` 的可收缩 PTX（驱动 JIT 定收缩）、ximat 自然写法；fma 文本口径 7（全部在未调用的 `matrix_inertial` 内） | 7 条本表 + 3 条 com_position + 2 条 crb，12 条逐一落表 |
| `crb` | `crb`、`cinert 组合` | `rigid_mass_matrix` + `resident_crb_init/accumulate/matrix` | f32 树序顺序加（跳过 world 根）+ `mfma` 链 + armature 最后加；resident 2 处 `atomicAdd`；fma 文本口径 19 | crb[10b] 与 mat 块（f32） |
| `mass_multiply` | `mass_multiply`、`borrowed_mass_multiply`、`resident 乘` | `mass_multiply_validate/multiply`、`borrowed_mass_validate/multiply` | f32 列序/CSR 行内顺序累加，无 fma/atomicAdd；sentinel `-131072` 精确比较 | n 个结果 + 状态码（f32） |
| `mass_solve` | `rigid_mass_solve`、`sparse dense/small/mixed/resident`、`substitute` | 11 组内核族（含 resident 文本切分的 `resident_mass_factor`/`substitute`） | tile 分支 `rsqrt.approx` + 可收缩 Schur vs scalar 分支 `sqrt/div/fmaf`；求解列扫描 `__frcp_rn/__fmul_rn/__fmaf_rn`；sparse/mixed 用内联 `atom.global.add.f32`；fma 文本口径 4 | status/ld/inv/x、CSR 因子与解（f32） |

## 四、B2a 已完成（4 条，肌腱族）

| row_id | 路径 | 内核 | 累加/舍入口径 | 输出 |
| --- | --- | --- | --- | --- |
| `fixed_tendon` | `fixed_tendon` | `fixed_tendon` | f32 顺序累加 `sum += coeff*qpos`（raw qpos，非 qpos-qpos0）；行内禁重复 joint → jac 直写 | length[nt] + jacobian[nnz]（f32） |
| `spatial_tendon` | `spatial_site/moment/wrap` | 3 内核（wrap Newton 迭代、CSR 雅可比、整型索引） | f32 顺序累加（每 world 单线程，无原子）；`1e10f` 有效点 sentinel；wrap 输出 13/点 scratch | length/points/scratch13（f32）+ jac（f32）+ wrap 索引（i32） |
| `tendon` | `tendon_values/indices` | 2 内核 | 纯搬运（f32 原样搬、i32 直搬），无算术；固定行共享游标不加点 | values（f32）+ indices（i32） |
| `tendon_velocity` | `tendon_velocity_validate/compute` | 2 内核 | f32 稀疏雅可比点积顺序累加；`-131072` sentinel + 索引越界拒绝 | status（f32）+ 肌腱速度（f32） |

## 五、待补（17 条产品路径 + 3 条 host 站点）

- **B2b–B2e（17 条，G10/G-limbs 面）**：`joint_passive`、`smooth_force`、`passive_force`、`tendon_wrap`、`tendon_passive`、`flex_position`、`flex_edge`、`flex_face`、`flex_hessian`、`flex_edge_passive`、`fluid_force`、`applied_force`、`kinetic_energy`、`attached`、`camlight`、`sleep_wake`、`field_batch`（按依赖分小批交付）。
- **B3（3 条 host 站点）**：`gjk_host`、`ccd_host`、`epa_host`（口径已在 `inventory.json` 的 `hostSiteRegistration` 登记，缺逐字段表）。

## 六、门禁联动

`tests/precision_type_gate.rs` 增加一条断言：`inventory.json` 的每一行必须在 `field-tables/` 下有表、
或在该测试内显式登记的「待批清单」里；表内 `kernels/params/intermediates/accumulations/outputs/upstream/evidence`
缺段即失败。待批清单随每批交付收紧（B1 后 25 条 → B1b 后 21 条 → B2a 后 17 条）——**不允许静默缺表**。

## 七、边界

- 不改 `src/`、`include/warp-headers/`、fixtures、判据/容差；零 GPU；不宣称 NP01 通过。
- 表中的「上游置信级别」沿用 `inventory.json` 原值，不升级（如 `mass_multiply` 保持文件级）。
