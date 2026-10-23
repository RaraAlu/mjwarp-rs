# G27@P2 常量计算阶段切片：set_const_fixed 固定常量

任务：`2026-10-11-roadmap-g27-p2`。日期：2026-10-23。执行：Backend（批次 1）。
矩阵：G27 常量计算（`set_const与长度范围` / `输入转换与动态更新` / P2/P7）。

## 1. 阶段语义与细化口径

原图只有 G27@P2 粗切片，未给具体函数。按当前代码布局，本片取
`mjwarp_rs::physics::constants::set_const_fixed`（架构计划 API U053；G27；P2/P7）
的**宿主侧固定常量**实现：由 f32 `body_mass` 与 `body_parentid` 计算
`body_subtreemass`（身体及其全部后代质量累加）。

P2 阶段只覆盖「输入转换」侧：不发射内核、不改模型布局、不引入 Model/Data 类型。
验收口径未放宽：矩阵核心验收「输入转换与动态更新」中，本片仅交付
固定常量这一部分；其余常量和动态更新（@P7）全部列为未验收边界（§6）。

## 2. 实现

- `src/physics/constants.rs`：`set_const_fixed(FixedConstantsInput) -> Result<FixedConstants, InputError>`。
- 累加顺序对齐上游 `mujoco_warp.set_const_fixed`：按 `body_tree` 深度分组、
  由深到浅，组内按身体编号升序把已累加质量加到父身体；f32 加法可交换，
  同模型下与上游并发原子累加的数值一致。
- 输入校验：长度一致、nbody≥1、质量有限且非负、世界身体父索引为 0、
  父索引在范围内且非自指、父链无环且最终连到身体 0。
- 全链 f32：无内部 f64 提升、无 f64 中间量（NP01）。

## 3. 参考来源与生成方式

| 项 | 值 |
| --- | --- |
| 模型源 | `fixtures/set-const/model.xml`（sha256 `63b33521…fa52d1`，nbody=9、nq=11、nv=10） |
| 原生参考 | MuJoCo 3.15.0（pinned venv）`mj_setConst`，输入先量化到 f32 |
| 上游参考 | MJWarp 冻结提交 `71da24d956378a87a703b6e1442b13aec0c4ac29`，Warp 1.15.0，`mujoco_warp.set_const_fixed` |
| 生成脚本 | `scripts/authoring/author-g27-set-const.ps1` + `author-g27-set-const.py`（离线，产品测试不启动 Python） |
| 产物 | `fixtures/set-const/g27-fixed.json`、本目录 `authoring.json` |
| 环境 | RTX 4070 Ti SUPER / 驱动 596.36；CUDA Toolkit 12.9（Warp 运行时） |

模型为专用四层身体树：9 个身体（含世界）、分支与多级链、非 2 幂质量
（1.3/2.7/0.9/3.1/0.7/1.9/2.3/0.5），用于覆盖多子节点累加顺序。

## 4. 对照结果（本次实测）

| 判定 | 结果 |
| --- | --- |
| 产品 vs 冻结上游（5 次运行） | **逐元素位型完全一致**（0 位差；上游 5 次仅 1 种位型） |
| 产品 vs 原生 f64 参考 | max abs **2.98e-07** ≤ 冻结容差 **1e-06** |
| 原生独立 f64 复算 vs `mj_setConst` | 0（参考自洽） |
| 确定性 | 同输入两次调用结果相等 |

冻结容差写在 fixture `tolerance.native_abs=1e-6`、`tolerance.upstream_abs=1e-6`，
测试前固定，未因结果放宽。上游并发原子累加在本树未观测到顺序位差
（`distinct_bit_patterns=1`、`max_abs_run_spread=0`）。

## 5. 验收指标映射（NP01–NP04）

- **NP01**：新增路径全部 f32；`body_mass` 为 f32 输入，累加为 f32，结果 f32；
  无 f64 中间量、无内部提升。
- **NP02**：记录冻结上游提交、函数名（`set_const_fixed`）、模型源 sha256、
  运行环境（MuJoCo/Warp/mujoco-warp 版本、GPU/驱动）与运行次数（5）。
- **NP03**：两级判定——先与原生 MuJoCo 参考比较（通过，2.98e-07 ≤ 1e-6），
  再与 MJWarp 参考比较（逐位一致）。参考清单与输入量化规则已冻结。
- **NP04**：**不适用/未测**——本片不发射内核、无批量对照与性能记录；
  不宣称性能或 GPU 证据。

## 6. 未验收边界

- 其余 `set_const` 常量：`tendon_length0`、`dof/body/tendon_invweight0`、
  `cam/light pos0`、`meaninertia`、`eq_data0`（依赖 G01/G02/G03 链与 G12 约束）。
- 执行器常量与长度范围（`actuator_acc0`、`actuator_biasprm` 阻尼比解析、
  `actuator_lengthrange`）依赖尚未实现的 G05 执行器模型字段。
- 动态更新（`set_const`/`set_const_0`/`set_const_spring` 与 GPU 内核）属 @P7 切片。
- 本片为宿主侧实现，未提供设备内核与上传路径；不宣称 G27 或精度验收通过。

## 7. 复现

```powershell
# 离线重新生成参考（不进入产品链，需要 pinned venv 与 GPU）
scripts/authoring/author-g27-set-const.ps1

# 产品侧定向测试（只读冻结 fixture，不启动 Python）
cargo test --locked --lib constants
```
