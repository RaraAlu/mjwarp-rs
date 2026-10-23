# G18@P6 离散积分切片：DISCRETE 步进映射与 discrete_acc 恒等分支

任务：`2026-10-11-roadmap-g18-p6`。日期：2026-10-23。执行：Backend（批次 3）。
矩阵：G18 离散积分（`DISCRETE步进映射` / `有效度量与约束一致性` / P6）。

## 1. 阶段语义与细化口径

旧计划 G18@P6 只有粗切片（DISCRETE 步进映射；验收「有效度量与约束一致性」）。
按当前代码布局，本片交付：

- `discrete`：对齐上游 `forward.discrete` = `_advance(m, d, d.qacc)`——先用 qacc
  推进速度，再用**新速度**推进位置（quat 积分复用 G16 实现），`time += timestep`。
- `discrete_acc`：对齐上游 `inverse.discrete_acc` 的两个恒等分支——DISCRETE
  积分器、以及禁用 `EULERDAMP` 的 EULER（上游 `wp.copy(qacc, d.qacc)`）；
  EULER 带阻尼时需要的 `(M + dt·diag(damping))` 有效度量求解分支显式拒绝。

## 2. 实现

| 文件 | 内容 |
| --- | --- |
| `src/physics/integrator/discrete.rs` | `DiscreteAccBranch`、`discrete`、`discrete_acc` 与边界拒绝 |
| `src/physics/integrator/discrete/tests.rs` | 冻结上游单步对照、原生 f64、恒等分支、拒绝面 |
| `src/physics/integrator.rs` / `src/physics/mod.rs` | `UnsupportedDiscreteAcc` 变体、模块挂接与导出 |

## 3. 参考来源与生成方式

| 项 | 值 |
| --- | --- |
| 模型源 | `fixtures/discrete/model.xml`（sha256 `e4945358…`；ball+hinge+slide 三关节、`integrator="discrete"`、无自由关节，nq=6/nv=5/timestep=0.01） |
| 用例 | 2 组状态（default / tilted）；上游 `forward` 产生 qacc 后 `discrete` 单步 |
| 上游参考 | MJWarp `71da24d956…`，Warp 1.15.0：`forward` + `discrete` |
| 原生参考 | MuJoCo 3.15.0（pinned venv）f64 `mj_forward` + `_next_velocity` 公式 + `mj_integratePos` |
| 生成脚本 | `scripts/authoring/author-g18-discrete.ps1` + `.py`（离线；产品测试不启动 Python） |
| 产物 | `fixtures/discrete/cases.json`、本目录 `authoring.json` |

## 4. 对照结果（本次实测）

| 判定 | 结果 |
| --- | --- |
| 产品 vs 冻结上游 | 单步 qpos/qvel 位型一致 **18/22**，max abs **5.96e-08** ≤ 1e-6（FMA 末位差）；time 逐位一致 |
| 产品 vs 原生 f64 一级判定 | max abs **5.96e-08** ≤ 1e-5 |
| 与 G16 Euler 直接分支等价 | 同一 qacc 输入下 `discrete` 与 `euler` 的 qpos/qvel 逐位相等（`_advance` 语义） |
| `discrete_acc` 恒等分支 | 输出与输入 `qacc` 位型一致；`DampedEuler` 分支返回 `UnsupportedDiscreteAcc` |

复跑：`cargo test --locked --lib discrete` → **2 过 0 败**；全量 lib 通过；
`cargo fmt --check` 与 `cargo clippy --all-targets -- -D warnings` 通过。

## 5. 验收指标映射（NP01–NP04）

- **NP01**：新增路径全 f32（qacc、状态推进、恒等映射）。
- **NP02**：记录冻结上游提交、`forward`/`discrete` 函数名、模型 sha256、关节布局与环境。
- **NP03**：两级判定——原生 f64 先行（5.96e-08），再与 MJWarp f32 参考对照（1e-6）。
- **NP04**：**不适用/未测**——不发射内核、无批量耗时与显存记录。

## 6. 未验收边界

- 未实现 `(M + dt·diag(damping))` 有效度量求解分支（需要质量矩阵求解链）。
- 未实现 IMPLICITFAST 的 `deriv_smooth_vel` + factor-solve、`_launch_discrete_free_gyro`
  自由体陀螺修正（本片模型无自由关节）、约束求解与约束一致性、`check_discrete`
  配置校验。
- 未实现 GPU 内核与 CUDA 图条件；不宣称完整 G18 或 P6 验收通过；NP04 未测。

## 7. 复现

```powershell
scripts/authoring/author-g18-discrete.ps1
cargo test --locked --lib discrete -- --nocapture
```
