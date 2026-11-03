# host 侧四元数→矩阵派生审计（P3，doc-only，无 fix 卡）

任务：`2026-10-29-render-bvh-host-quat-derivation-audit`（Arch 2026-10-29 登记；2026-10-29 只读审计完成，Chief 接受；2026-11-03 本目录落库后关单）
性质：只读审计 + 守则记录。**不改代码、不做 fix、不宣称任何覆盖结论**。

## 1. 审计目标

枚举产品 `src/` 中所有「四元数 → 矩阵/位姿」派生点，逐点判定是否被位级参考覆盖；给出风险清单与守则，防止未来新增 host 派生在位级验收里产生 1–2 ULP 漂移却无人察觉。

## 2. 逐点清单

| # | 派生点 | 文件:行 | 消费者 | 位级覆盖判定 |
| --- | --- | --- | --- | --- |
| 1 | `math::quat_to_mat`（未收缩代数展开） | `src/math/mod.rs:59` | 无产品调用（仅 math 内部/测试；`cast.rs:148` 注释明确禁止在产品路径使用） | **无任何比较**（非产品路径） |
| 2 | `CameraPose::from_fields` → `warp_quat_to_mat` | `src/render/kernel/cast.rs:66`（派生调用 :99，实现 :150，单测 :451） | 固定相机 world pose（渲染 `cam_xpos/cam_xmat`） | **已被位级参考覆盖**：R05-S3b 按 1003 条 GPU `cam_xmat` 反推上游收缩序；两相机 `cam_xmat` 9/9 位级 0 差、scene-a 9/9 |
| 3 | `light_data_poses` | `src/render/kernel/megakernel_arrays.rs:167`（`megakernel.rs:157` 调用） | 灯光朝向/位置 | **无矩阵派生**；与 pinned forward 6/6 位等 |
| 4 | geom/body 位姿 | `src/io/collision_inputs.rs`、`src/model/collision_inputs.rs` 等 | 场景/碰撞输入 | **mjData 原样拷贝**，无四元数→矩阵派生 |
| 5 | flex/physics `matrix_from` | `src/physics/**`（设备侧） | 设备内核 | 仅设备侧，不进 render host 路径 |

## 3. 判性汇总

- 已被位级参考覆盖：2 处（#2 固定相机派生；#4 原样拷贝场景位姿）。
- 只在容差断言下 / 无任何比较：0 处产品路径（#1 非产品路径）。
- 真实场景中会产生非平凡四元数、因此可能差 1–2 ULP 的点：**#2（移动体相机/非单位 xquat 世界位姿）**——当前 `CameraPose::from_fields` 只覆盖 fixed 相机（`cam_bodyid=[0,0]`，world 体 `xquat` 为单位四元数）；移动体相机需 `mul_quat(xquat_world, cam_quat)`，未覆盖。

## 4. 风险清单（按影响排序）

1. **新增 host quat→matrix 派生**（未覆盖）：任何新派生若直接调用 `math::quat_to_mat` 或用不同收缩序，会在位级判据下产生 1–2 ULP 差，且没有参考能定位。
2. **移动体相机位姿**（部分覆盖）：`from_fields` 只覆盖 fixed 相机；若未来相机跟随移动体，需先补 GPU 参考再启用。
3. **收缩序的栈依赖**：R05-S3b 反推的是本机 ptxas/driver（596.36）在该上游代码上的 SASS 收缩序；driver/工具链变更后可能需重新反推（参考：`plan.json` structural decision `6f246b46` 的 limit 段）。

## 5. 守则（doc-only，不做 fix）

今后新增任何 host quat→matrix 派生：
1. 必须先按目标 GPU 的收缩序做位级反推（用 GPU 参考覆盖）；
2. 严禁直接调用未收缩的 `math::quat_to_mat` 作为产品路径；
3. 必须在证据中写明覆盖的相机类型与前提（同 recipe / 同编译形态 / 同 GPU+驱动 / 同 warp 版本）。

## 6. 非声明

本审计**不改代码、不新增判据、不补参考**；结论只用于风险登记与守则。证据仅本 README 与 manifest。
