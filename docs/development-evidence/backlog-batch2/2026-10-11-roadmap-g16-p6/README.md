# G16@P6 积分器切片：Euler 半隐式与 RK4 状态推进

任务：`2026-10-11-roadmap-g16-p6`。日期：2026-10-23。执行：Backend（批次 2）。
矩阵：G16 Euler 与 RK4（`上游积分语义` / `单步误差与步长收敛` / P6）。

## 1. 阶段语义与细化口径

旧计划 G16@P6 只有粗切片（Euler 与 RK4；验收「单步误差与步长收敛」）。按当前
代码布局，本片交付计划 API `euler`（U014）与 `rungekutta4`（U015 家族）的
**宿主侧状态推进算术**：

- `next_position`：对齐上游 `_next_position`——free 关节平移 + `math.quat_integrate`
  四元数积分、ball 四元数积分、hinge/slide 标量积分；`qvel_scale` 逐关节生效。
- `next_velocity`：`qvel + qacc_scale * qacc * timestep`。
- `euler`：半隐式——先推进速度，再用**新速度**推进位置，`time += timestep`
  （对齐禁用 `EULERDAMP` 时的 `_advance(m, d, d.qacc)` 分支）。
- `rungekutta4`：`_rk_accumulate` 的 `1/6,1/3,1/3,1/6` 顺序累加与 `_advance`；
  **4 级导数（各级 qvel/qacc）由冻结输入提供**——本片不实现 `forward()` 动力学链。

## 2. 实现

| 文件 | 内容 |
| --- | --- |
| `src/physics/integrator.rs` | `JointLayout`/`IntegratorState`/`Rk4Stage`、`quat_integrate`、`next_position`、`next_velocity`、`advance`、`euler`、`rungekutta4` 与校验 |
| `src/physics/integrator/tests.rs` | 冻结上游单步对照、原生 f64 一级判定、合成 ODE 收敛阶、校验面 |

## 3. 参考来源与生成方式

| 项 | 值 |
| --- | --- |
| 模型源 | `fixtures/integrator/model.xml`（sha256 `0bc4a417…`；free/ball/hinge/slide 四关节，nq=13、nv=11、njnt=4、timestep=0.01） |
| 用例 | 3 组状态（default / rotated / spinning），每组记录 euler 单步与 RK4 单步 |
| 上游参考 | MJWarp `71da24d956…`，Warp 1.15.0：`euler`（禁用 `EULERDAMP` 走直接 `_advance`）与 `rungekutta4`（用上游 `_next_position`/`_next_velocity` 内核逐级采集 4 级导数后冻结） |
| 原生参考 | MuJoCo 3.15.0（pinned venv）f64：`mj_forward` + `mj_integratePos` + `_next_velocity` 公式复刻 euler；原生 f64 复刻 RK4 4 级组合并对照原生 `mj_step`(RK4) |
| 生成脚本 | `scripts/authoring/author-g16-integrator.ps1` + `.py`（离线；产品测试不启动 Python） |
| 产物 | `fixtures/integrator/cases.json`、本目录 `authoring.json` |

## 4. 对照结果（本次实测）

| 判定 | 结果 |
| --- | --- |
| 单步 vs 冻结上游 | euler 位型一致 **64/72**、max abs **5.96e-08**；RK4 位型一致 **67/72**、max abs **5.96e-08**（差异为上游 CUDA 内核 FMA 收缩的末位差，按批次 1 口径 1e-6 判定） |
| 冻结级 replay | 用冻结的 4 级导数在上游内核上重放组合，与上游 `rungekutta4` 最终状态 **max abs 0**（组合与推进语义一致） |
| 原生 f64 一级判定 | euler max abs **1.19e-07** ≤ 1e-5；RK4 max abs **6.26e-06** ≤ 1e-5 |
| 原生 RK4 组合自洽 | 原生 f64 复刻 4 级组合 vs 原生 `mj_step`(RK4) max abs **6.26e-06** |
| 步长收敛（合成调和 ODE，T=1，步数 2/4/8/16） | Euler 误差 `[2.455e-1, 1.206e-1, 6.097e-2, 3.081e-2]`，比值≈2.0（一阶）；RK4 误差 `[5.195e-4, 3.250e-5, 1.926e-6, 1.522e-7]`，比值 16.0/16.9/12.7（四阶） |

复跑：`cargo test --locked --lib integrator` → **3 过 0 败**；全量
`cargo test --locked --lib` 通过；`cargo fmt --check` 通过；
`cargo clippy --all-targets -- -D warnings` 通过。

## 5. 验收指标映射（NP01–NP04）

- **NP01**：新增路径全 f32（qpos/qvel/qacc、四元数积分、RK4 权重与累加）。
- **NP02**：记录冻结上游提交、`euler`/`rungekutta4`/`_next_position`/`_next_velocity` 函数名、模型 sha256、表与联合布局、环境。
- **NP03**：两级判定——原生 f64 先行（euler 1.19e-07、RK4 6.26e-06），再与 MJWarp f32 参考对照（1e-6 冻结容差）。
- **NP04**：**不适用/未测**——不发射内核、无批量耗时与显存记录。

## 6. 未验收边界

- 未实现 activation/mocap 推进、`_next_time` 的 overflow/efc 统计与 history 写入。
- 未实现阻尼隐式分支（`EULERDAMP`/`DAMPER` 的 M 对角修正与 factor-solve）。
- 未实现 `forward()` 动力学链，因此未验收「物理链」的单步误差/收敛；本片收敛阶
  只在合成调和 ODE 上验证积分器数学语义。
- 未实现 DISCRETE/IMPLICIT/IMPLICITFAST 积分器、GPU 内核、CUDA 图条件。
- 不宣称完整 G16 或 P6 验收通过；NP04 未测。

## 7. 复现

```powershell
scripts/authoring/author-g16-integrator.ps1
cargo test --locked --lib integrator -- --nocapture
```
