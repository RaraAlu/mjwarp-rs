# G21@P7 平滑速度导数切片：deriv_smooth_vel 与解析/有限差分对照

任务：`2026-10-11-roadmap-g21-p7`。日期：2026-10-23。执行：Backend（批次 2）。
矩阵：G21 平滑速度导数（`deriv_smooth_vel` / `解析导数与有限差分` / P7）。

## 1. 阶段语义与细化口径

旧计划 G21@P7 只有粗切片（`deriv_smooth_vel`；验收「解析导数与有限差分」）。
按当前代码布局，本片交付上游 `deriv_smooth_vel` 的**被动阻尼导数子集**：

```text
qDeriv = -(线性阻尼 + 多项式阻尼对速度的导数)   // 仅对角 dof
out    = M - dt * qDeriv                          // 写回 d.M 结构地址
```

- `M`（`d.M` 值 + `M_fullm_i`/`M_fullm_j`/`M_elemid` 结构）、`dof_damping`/
  `dof_dampingpoly`、`qvel` 全部冻结输入。
- 上游其余导数项（执行器 actuation/moment、肌腱阻尼、椭球/盒流体力）不在本片。

## 2. 实现

| 文件 | 内容 |
| --- | --- |
| `src/physics/derivative.rs` | `DerivativeLayout`、`poly_force_deriv`、`deriv_smooth_vel` 与校验 |
| `src/physics/derivative/tests.rs` | 冻结上游逐位对照、解析 vs 中央差分（上游 f32 / 原生 f64）、校验面 |

## 3. 参考来源与生成方式

| 项 | 值 |
| --- | --- |
| 模型源 | `fixtures/deriv/model.xml`（sha256 `7e3b9bbe…`；双摆两 hinge + 末端 slide，nv=3、nC=6、fullm=6，关节阻尼 0.5/0.3/0.2，无执行器/肌腱/流体） |
| 配置 | (a) 合成多项式阻尼 `dof_dampingpoly=[[0.5,0.2],[0,0],[0.3,-0.1]]`（覆盖多项式分支），3 组状态；(b) 线性阻尼（原生可对照），2 组状态 |
| 上游参考 | MJWarp `71da24d956…`，Warp 1.15.0：`deriv_smooth_vel` 输出（M - dt*qDeriv） |
| 原生参考 | MuJoCo 3.15.0（pinned venv）f64 `mj_forward` 的 `qfrc_passive` 中央差分（eps=1e-3） |
| 生成脚本 | `scripts/authoring/author-g21-deriv.ps1` + `.py`（离线；产品测试不启动 Python） |
| 产物 | `fixtures/deriv/cases.json`、本目录 `authoring.json` |

## 4. 对照结果（本次实测）

| 判定 | 结果 |
| --- | --- |
| 产品 vs 冻结上游 `deriv_smooth_vel` | 多项式配置 3 用例 **18/18 元素逐位一致（max abs 0）**；线性配置 2 用例同样 max abs 0 |
| 解析 vs 上游 f32 中央差分（含多项式阻尼，eps=1e-3） | 对角 `M - dt*qDeriv` 与差分 max abs **1.55e-06** ≤ 1e-4 |
| 解析 vs 原生 f64 中央差分（线性阻尼，eps=1e-3） | max abs **4.67e-08** ≤ 1e-4 |
| 多项式分支覆盖 | `dof_dampingpoly` 直接覆盖后，产品/上游/差分三方一致（含 `2p0|v| + 3p1v²` 项） |

复跑：`cargo test --locked --lib derivative` → **2 过 0 败**；`cargo fmt --check` 与
`cargo clippy --all-targets -- -D warnings` 通过。

## 5. 验收指标映射（NP01–NP04）

- **NP01**：新增路径全 f32（M、阻尼、速度、导数、组装）。
- **NP02**：记录冻结上游提交、`deriv_smooth_vel` 函数名、模型 sha256、M 结构布局与阻尼参数。
- **NP03**：两级判定——原生 f64 中央差分先行（4.67e-08），再与 MJWarp f32 参考逐位对照（max abs 0）与上游 f32 差分（1.55e-06）。
- **NP04**：**不适用/未测**——不发射内核、无批量耗时与显存记录。

## 6. 未验收边界

- 未实现执行器 actuation/moment 导数、肌腱阻尼导数、椭球/盒流体力导数与 `has_fluid` 分支。
- 未实现 `efm`/隐式积分组合、dense 紧凑块、世界批量广播语义。
- 未实现 `forward`/`fwd_velocity` 链（输入冻结）；无 GPU 内核与图执行。
- 多项式阻尼为 MJWarp 专属字段（由执行器 dampingpoly 折叠），原生无对应字段，
  因此原生差分只用线性阻尼配置对照；不宣称完整 G21 或精度验收通过；NP04 未测。

## 7. 复现

```powershell
scripts/authoring/author-g21-deriv.ps1
cargo test --locked --lib derivative -- --nocapture
```
