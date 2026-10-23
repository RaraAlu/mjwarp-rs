# U038-3D 三维旋转/组合状态验证证据

## 结论（未宣称通过）

- 冻结上游同条件参考的 `body_isdofancestor` 语义已修正并由上游 `put_model` 天然校验。
- 产品链运动学八字段全部 0 超差（105+140+315+105+84+84+105+462=1400 项）。
- 产品重力算子与上游重力算子 77/77 全部 0 超差（`gravity_operator` 位差 0~1，来自原子累加顺序，max ≤4.77e-7）。
- 全链 `qfrc_gravcomp` 仍有 **8/77 项超 2e-6**（max 4.29e-6）。该差距来自产品运动学 f32 算序差
  （cdof max 1.79e-7、xipos 5.96e-8、subtree_com 8.94e-8）经重力高增益（Σm·gc·|g|≈95.6）放大，
  不是重力算子错误；容差与断言均未改动。
- 因此 `2026-10-16-next-u038-3d` **未完成**。用户 2026-10-23 裁决 B：该 8/77 超差登记为
  **已知债务**，U038-3D **不宣称通过**（详见 `known-deviation.md`）。旧债务任务
  `2026-10-23-u038-3d-gravity-propagation-debt` 已 abandon；跟踪任务
  `2026-10-23-u038-3d-gravity-propagation-debt-2` 保持 proposed、不 start。
- PTX 逐表达式对照证据（提交 `71d9d42`、`722e979`，`ptx-diff/`）：三表达式源形态收缩一致，
  但 FMA 收缩由内联上下文决定；com/cdof 上游分层原子累加与产品单内核顺序累加存在结构差异。

## 根因（authoring 参考错误，非产品错误）

初版 3D 参考把 `body_isdofancestor` 算成了「body 是 dof 所属体的祖先」（反向关系），
而上游 `mujoco_warp/_src/io.py`（71da24d）的定义是「dof 是 body 的祖先」：
从 body 自己的最后一个 dof 沿 `dof_parentid` 链回溯，经过的 dof 才影响该 body。

离线反证（CPU 复算，见 `triage.json`）：

| 假设矩阵 | 产品输出 vs 该矩阵复算 | 上游冻结输出 vs 该矩阵复算 |
| --- | --- | --- |
| 反向语义（初版 fixture） | max 80.93（不符） | max 4.77e-7（吻合） |
| 上游 io.py 语义（正确） | max 5.28e-6（f32 复算误差内） | max 80.93（不符） |

即：**产品实现与上游正确语义一致，冻结参考的矩阵语义反了**，导致上游重力算子把整块体的
重力算到错误自由度上（初版全链超差 70/77、max 80.93 或早期 run 的 161.87）。

## 修复

1. `scripts/authoring/author-u038-3d.py`
   - `dof_layout` 改为 MuJoCo 式 `dof_parentid` 链（首个 dof 指向最近有 dof 祖先体的末 dof，
     同关节内逐个串接）。
   - 新增 `body_isdofancestor_rows`：按 pinned 上游 `io.py` 语义计算矩阵。
   - 新增 `native_isdofancestor`：用同一 config 构造原生 MuJoCo 模型，调用
     `mujoco_warp.put_model(mjm).body_isdofancestor` 取得权威矩阵，逐项比对，不一致即中止生成。
   - 新增产物 `three-d-kinematics/natural-validation.json`（`exact_match: true`，
     matrix `sha256 aec677ba0ba62c318bb6f0c1e721d90570a38b0f0b6fe3e90701814f121d9fe0`，
     `dof_parentid [-1,0,1,2,3,4,5,6,7,8,-1]`，上游 `io.py` sha256 `7df64bab…`）。
   - `natural-validation.json` 纳入 receipt 文件清单与 authoring 记录。
2. 重新生成冻结参考（pinned 环境自检通过：Python 3.12.13 / mujoco 3.15.0 / warp-lang 1.15.0 /
   numpy 2.5.4 / mujoco-warp 3.15.0，上游 71da24d clean，warp.dll RECORD 一致，
   smooth 缓存 cu/meta/ptx 哈希一致；两次上游运行逐位一致）。
3. `tests/t4_gravity_3d_product_chain.rs` 只同步 `PACKAGE_SHA256` 常量；断言、容差、比较结构未变。
4. 生成器按新文件 ≤500 行规范拆为 entry + u038_3d_common.py/u038_3d_config.py/u038_3d_reference.py，四份均归档入 	hree-d-kinematics/ 并纳入 receipt 文件清单。

生成命令（离线 authoring，非产品链）：

```text
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/authoring/author-u038-3d.ps1
```

## 复跑结果（2026-10-23，run5；run3/run4 初跑结果相同）

环境：Windows msvc、RTX 4070 Ti SUPER 596.36、CUDA 12.8.1 工具链；命令
`cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --test t4_gravity_3d_product_chain -- --ignored --test-threads=1`
（`MJWARP_GRAVITY_3D_EVIDENCE_DIR=target/u038-3d/run5/final`）。

| 字段 | 值数 | 位差 | 超差 | max_abs |
| --- | --- | --- | --- | --- |
| xpos | 105 | 4 | 0 | 5.96e-8 |
| xquat_wxyz | 140 | 7 | 0 | 5.96e-8 |
| xmat | 315 | 17 | 0 | 1.79e-7 |
| xipos | 105 | 4 | 0 | 5.96e-8 |
| xanchor | 84 | 3 | 0 | 5.96e-8 |
| xaxis | 84 | 4 | 0 | 1.19e-7 |
| subtree_com | 105 | 9 | 0 | 8.94e-8 |
| cdof | 462 | 55 | 0 | 1.79e-7 |
| qfrc_gravcomp | 77 | 19 | **8** | 4.29e-6 |

- 合计 1477 项、122 位差、**8 超差**、max 4.29e-6（初版修复前 70 超差）。
- `gravity_source_propagation`：19 位差 / 8 超差 / max 4.29e-6。
- `gravity_operator`：**0 超差**；位差 0（run3/run5）/1（run4），均为原子累加顺序差异（max 4.77e-7 ≤ 2e-6）。

## 剩余差异分析

- 产品与上游在 3D 自由/球关节路径上有末位 f32 算序差（1400 个运动学字段中 99 个位差，
  最大 1.79e-7，均远低于 2e-6）。
- 重力算子灵敏度：每自由度 ∂qfrc/∂cdof ≈ Σ m·gc·|g| = (1.5·1+2.5·0.5+3.5·2)·9.81 = 95.6，
  1.79e-7 × 95.6 ≈ 1.7e-5 的理论上界；实测 8 项超差 max 4.29e-6，与传播量级一致。
- 结论：剩余 8 项不能通过改重力算子或放宽容差消除。PTX 对照已确认：源级表达式收缩一致，
  但内核内 FMA 收缩受内联上下文决定，源级转录不能复现上游收缩；com/cdof 还存在上游分层
  原子累加与产品顺序累加的结构差异。用户 2026-10-23 裁决 B：登记为已知债务，不宣称通过；
  债务跟踪任务 `2026-10-23-u038-3d-gravity-propagation-debt-2` 保持 proposed、不 start。
  详见 `known-deviation.md` 与 `ptx-diff/`。

## 未验收边界

- 完整三维状态空间未覆盖（单一设计场景 7 worlds）。
- NP03/NP04 未宣称完成；未跑满额回归（本任务只跑定向 GPU 测试）。
- release/Linux/性能未验证。
- 产品源码未改（`src` 保持 unchanged），因此本任务未触发 T4 源码身份重钉。

## 文件

- `known-deviation.md`：用户裁决 B 已知债务登记（偏差数据、根因、复现入口、未验收边界）。
- `ptx-diff/`：PTX 逐表达式对照证据（`ptx-diff.json`、产品 PTX 摘录、探针源码；工具 `scripts/u038-ptx-diff.ps1`）。
- `precheck.md`：设计预检与场景清单。
- `triage.json`：根因反证数据。
- `run.json` / `product-gravity-raw.json` / `upstream-gravity-raw.json`：
  run5 原始证据副本（`target/u038-3d/run5/final/`）。
- `natural-validation.json`：上游 `put_model` 矩阵天然校验副本。
