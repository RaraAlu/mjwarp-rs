# G27@P7 常量计算阶段切片：qpos0 依赖常量（set_const_0 子集）

任务：`2026-10-11-roadmap-g27-p7`。日期：2026-10-23。执行：Backend（批次 1 后继）。
矩阵：G27 常量计算（`set_const与长度范围` / `输入转换与动态更新` / P2/P7）。

## 1. 阶段语义与细化口径

P2 切片（`70b36b7`）已交付固定常量 `body_subtreemass`。本 P7 切片交付
计划 API `mjwarp_rs::physics::constants::set_const_0`（U052；G27；P2/P7）中
**不依赖质量矩阵求解链**的 qpos0 依赖常量：`tendon_length0`、
`cam_pos0/cam_poscom0/cam_mat0`、`light_pos0/light_poscom0/light_dir0` 与
`meaninertia`（M CSR 对角线均值，nv=0 取上游默认 1.0）。

输入为**冻结的 qpos0 中间量**（ten_length、cam/light 世界位姿、xpos、
subtree_com、M CSR）；本片不重跑 kinematics/com/camlight/tendon/crb/factor_m 链，
不发射内核。动态更新语义以两套 qpos0（模型默认与运行时修改后）各自重算验证。

## 2. 实现

- `src/physics/constants.rs` 增加 `mod qpos` 与导出；`src/physics/constants/qpos.rs`
  实现 `set_const_0(QposDependentInput) -> Result<QposDependentConstants, InputError>`。
- 公式逐分支对齐上游：`cam_pos0 = cam_xpos - xpos[body]`；
  `cam_poscom0 = cam_xpos - subtree_com[target≥0 ? target : body]`；
  `cam_mat0 = cam_xmat`；light 同构（`light_dir0 = light_xdir`）；
  `tendon_length0 = ten_length`；`meaninertia = Σ M[row_start+rownnz-1] / nv`。
- 校验：相机/光源/中间量长度、body/target 索引范围、M CSR 行列地址越界；
  不做缓存，可对任意 qpos0 中间量重复重算。

## 3. 参考来源与生成方式

| 项 | 值 |
| --- | --- |
| 模型源 | `fixtures/set-const/qpos-model.xml`（sha256 `707a599f…`；3 身体、2 相机、2 光源、1 固定肌腱、球关节 + 滑动关节） |
| 原生参考 | MuJoCo 3.15.0（pinned venv）`mj_setConst`（NP03 一级） |
| 上游参考 | MJWarp `71da24d956…`，Warp 1.15.0，`set_const_0(restore=False)`（NP03 二级） |
| 生成脚本 | `scripts/authoring/author-g27-qpos-constants.ps1` + `.py`（离线，产品测试不启动 Python） |
| 产物 | `fixtures/set-const/qpos-dependent.json`、本目录 `authoring.json` |
| 案例 | `default`（模型默认 qpos0）与 `modified`（球关节 qpos0 旋转 30° + 滑动 0.1） |

## 4. 对照结果（本次实测）

| 判定 | 结果 |
| --- | --- |
| 产品 vs 冻结上游（两案例 × 8 组常量） | **逐元素位型一致**（容差 0） |
| 产品 vs 原生 f64 一级判定 | max abs **1.19e-07** ≤ 冻结 1e-05 |
| 动态重算 | 两案例的 `tendon_length0`/`cam_pos0`/`cam_poscom0`/`light_pos0` 均不同且各自与上游一致；同输入重复调用结果相等（无缓存级联） |
| meaninertia | 两案例均为 0.42（与上游一致）；nv=0 时取上游默认 1.0（单元测试） |

## 5. 验收指标映射（NP01–NP04）

- **NP01**：新增路径全 f32（中间量、常量），无 f64 中间量。
- **NP02**：记录冻结上游提交、`set_const_0` 函数名、模型源 sha256、环境与两案例。
- **NP03**：两级判定——原生 `mj_setConst` 先行（max abs 1.19e-07），再与 MJWarp f32 逐位一致。
- **NP04**：**不适用/未测**——不发射内核、无批量对照与性能记录。

## 6. 未验收边界

- `dof/body/tendon_invweight0` 未做：依赖 M 求解链（G02/G03 集成）。
- `eq_data0`、执行器常量（依赖 G05 字段）、`set_const_spring`、全量 `set_const`
  与 `cam_pos0` 之外的渲染/图像用途未做。
- 无链式重算（kinematics/com/camlight/tendon/crb/factor_m）与设备内核；
  固定常量（P2 切片）不重复声明。
- 不宣称完整 G27 或精度验收通过；NP04 未测。

## 7. 复现

```powershell
scripts/authoring/author-g27-qpos-constants.ps1
cargo test --locked --lib qpos
```
