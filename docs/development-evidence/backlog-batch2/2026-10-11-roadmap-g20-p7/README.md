# G20@P7 逆向动力学切片：qfrc_inverse 与 CSR 质量矩阵乘法

任务：`2026-10-11-roadmap-g20-p7`。日期：2026-10-23。执行：Backend（批次 2）。
矩阵：G20 逆向动力学（`inverse与逆向中间量` / `正逆向回代与力平衡` / P7）。

## 1. 阶段语义与细化口径

旧计划 G20@P7 只有粗切片（inverse 与逆向中间量；验收「正逆向回代与力平衡」）。
按当前代码布局，本片交付连续时间逆向动力学的宿主侧切片：

```text
qfrc_inverse = qfrc_bias + M @ qacc - qfrc_passive - qfrc_constraint
```

- `M` 按上游 `d.M` 逐世界非零值冻结，乘向量使用上游 `support.mul_m` 的
  收集布局（`M_mulm_rowadr`/`M_mulm_col`/`M_mulm_madr`）。
- `qacc`（由上游 `forward` 正向动力学产生）、`qfrc_bias`、`qfrc_passive`、
  `qfrc_constraint` 全部冻结输入；本片不做 `fwd_position`/`fwd_velocity`/`rne`/
  传感器链，也不做 `inv_constraint` 逆约束求解。

## 2. 实现

| 文件 | 内容 |
| --- | --- |
| `src/physics/inverse.rs` | `SparseMass`（mulm 收集布局）、`mul_m`、`inverse_dynamics` 与输入校验 |
| `src/physics/inverse/tests.rs` | 冻结上游 qfrc_inverse、原生 f64 一级判定、正逆向力平衡、校验面 |

## 3. 参考来源与生成方式

| 项 | 值 |
| --- | --- |
| 模型源 | `fixtures/inverse/model.xml`（sha256 `a820e6ad…`；双摆两 hinge + 末端 slide，nv=3、nC=6、mulm=9，关节刚度/阻尼，无接触/执行器） |
| 用例 | 3 组状态（default / opposite / large），每组冻结 qpos/qvel、forward qacc、`d.M` 值、`qfrc_bias/passive/constraint` 与上游 `inverse` 输出 |
| 上游参考 | MJWarp `71da24d956…`，Warp 1.15.0：`forward` + `inverse`（连续时间分支） |
| 原生参考 | MuJoCo 3.15.0（pinned venv）f64 `mj_forward` + `mj_inverse` |
| 生成脚本 | `scripts/authoring/author-g20-inverse.ps1` + `.py`（离线；产品测试不启动 Python） |
| 产物 | `fixtures/inverse/cases.json`、本目录 `authoring.json` |

## 4. 对照结果（本次实测）

| 判定 | 结果 |
| --- | --- |
| 产品 vs 冻结上游 `qfrc_inverse` | 3 用例 max abs **1.91e-06** ≤ 1e-5（`M@qacc` 的 FMA 收缩/求和顺序末位差） |
| 产品 vs 原生 f64 一级判定 | max abs **2.62e-06** ≤ 1e-5 |
| 正逆向回代力平衡 | 产品 `qfrc_inverse` 绝对值 max **2.62e-06** ≤ 1e-5；原生对应值 ~1e-15（上游 f32 对应 1.4e-06–2.4e-06） |
| 布局一致性 | `M_mulm_rowadr`（nv+1）、`M_mulm_col`/`M_mulm_madr`（9 项）与逐世界 `d.M`（nC=6）按上游语义使用 |

复跑：`cargo test --locked --lib inverse` → **2 过 0 败**；`cargo fmt --check` 与
`cargo clippy --all-targets -- -D warnings` 通过。

## 5. 验收指标映射（NP01–NP04）

- **NP01**：新增路径全 f32（质量矩阵、加速度、力项、组装结果）。
- **NP02**：记录冻结上游提交、`forward`/`inverse` 函数名、模型 sha256、CSR/mulm 布局与环境。
- **NP03**：两级判定——原生 f64 `mj_inverse` 先行（2.62e-06），再与 MJWarp f32 参考对照（1e-5）。
- **NP04**：**不适用/未测**——不发射内核、无批量耗时与显存记录。

## 6. 未验收边界

- 未实现 `discrete_acc`（EULER 阻尼修正、IMPLICITFAST 的 `deriv_smooth_vel` +
  factor-solve、DISCRETE 恒等）与 `_qfrc_inverse_discrete`（`efm_c`/`efm_ca`）。
- 未实现 `inv_constraint` 逆约束求解、dense 紧凑 DOF 块分支、GJ/睡眠/岛屿。
- 未实现 `forward`/`rne`/传感器链（输入全部冻结）；无 GPU 内核与图执行。
- 不宣称完整 G20 或 P7 验收通过；NP04 未测。

## 7. 复现

```powershell
scripts/authoring/author-g20-inverse.ps1
cargo test --locked --lib inverse -- --nocapture
```
