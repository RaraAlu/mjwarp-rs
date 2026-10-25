# G6-3 native 容差落点：用户已拍板 B（诊断不参与判定；不落生效 atol/rtol）

- 任务：`2026-10-25-g10-p5-acceptance-perf-native`（G6-3）
- 状态：**用户 2026-10-25 拍板选项 B**（经 Chief 转达）。契约已由 Arch 单独落地并提交：`docs/numerical-precision-contract.md` → `## 接触字段（native 侧）`（契约提交 `09896792`，登记提交 `ce9baa5a`）。本片按 B 收口：`dist`/`pos`/`frame`/`normal` 的 native 侧**只作诊断、不参与通过或失败**，**不写任何生效 atol/rtol**。
- 拍板记录与依据：`docs/development-evidence/native-contact-tolerance-decision-b/README.md`（契约原文逐字引用、v3 材料摘要、用户拍板记录、样本量说明）。
- 边界：不改已批准判据、不改已 verified 证据数字、不放宽产品容差；不得写「接触字段精度已达标 / native 等价 / native 放行 / 原生放行」。

## 1. 拍板结果（2026-10-25，选项 B）

| 选项 | 内容 | 结果 |
| --- | --- | --- |
| A | 粗门禁：`pos` 1e-3 / `frame`·`normal` 2.5e-3 / `dist` 1e-6 + rtol 1e-4 / 结构零容差 | **不采用**：唯一可比样本仅 `ellipsoid-box` 1 case / 2 接触点，且设备与上游 f32 对 native 的差几乎相同（属引擎接触选择语义差）；A 比真实缺陷（8.3e-5）松 12×。**留待真 native 样本扩充后再议** |
| B | 只作诊断、不放行 | **用户 2026-10-25 拍板采用**；契约落点 `docs/numerical-precision-contract.md` `## 接触字段（native 侧）`（提交 `09896792`） |
| C | 两者（诊断 + 生效门禁分栏） | **不采用**：B 已含诊断，当前无生效 native 门禁可分栏 |

**方法约束（非用户选项）**：无论选 A/B/C，都要执行「用真实产品输出复算并标注样本量」这一动作（原任务记录中的 E）。它不是第四选项，而是任何落点都附带的复算义务；Chief 明确容差卡只放 A/B/C。当前真 native 输出只有 `ellipsoid-box` 1 case / 2 接触点（native 侧还只是读数），**样本量不足，不可据此定容差**。复算结果只进诊断列，不能作为生效 atol/rtol 的依据。

## 2. native 现有材料（诊断）

| 材料 | 路径 | 性质 |
| --- | --- | --- |
| native 探针 C ABI（G01 字段/运动学/碰撞几何输入） | `include/mjwarp_native_probe.h`、`include/mjwarp_collision_probe.h`、`native/model_probe*.cpp` | 只做**模型字段/状态拷贝**（如 `mjwarp_native_copy_primitive_geometry`），无接触/窄相输出 |
| native-error-seed（G2） | `docs/development-evidence/g10-p5-gjk-epa/native-error-seed.json` | native **f64 输入**读数：`ours_vs_native` 208 分量 max 8.1928766e-7；`mjwarp_vs_native` 同；**非接触参考、非判据** |
| G2 契约框架读数 | `docs/development-evidence/g10-p5-gjk-epa/receipt.json`（`final_criterion.native_side`） | native 侧接触字段待验收不放行（用户拍板 B） |
| 用户拍板 B 契约与证据 | `docs/numerical-precision-contract.md`（`## 接触字段（native 侧）`）；`docs/development-evidence/native-contact-tolerance-decision-b/` | 接触字段 native 侧只作诊断、不参与通过/失败；`qa-native-f64.json` 读数只作诊断与趋势 |
| 精度契约 | `docs/numerical-precision-contract.md`（f32 全链/不得 f16·TF32；逐字段冻结容差；NP01-NP04） | 原生参考只作诊断，不得反馈产品物理结果；接触精度仍由对上游 f32 判据承担 |

**E 复算（本片诊断口径，标样本量）**：唯一可比 case 为 `ellipsoid-box`（2 world × 各 1 接触 = **2 个接触点**）。设备（产品）vs native：`dist` 3.979e-7（rel 3.98e-5）、`pos` 4.546e-4、`frame` 1.170e-3、`normal` 1.170e-3；上游 f32（warp CUDA）vs native：3.961e-7 / 4.546e-4 / 1.171e-3 / 1.171e-3 —— 设备与上游 f32 几乎相同 → 属 MuJoCo C 与 warp 的接触选择语义差，不是实现/f32 精度差。**样本量不足，不可外推，只作诊断，不作容差依据。**

**结论（诊断）**：native 目前只有 f64 输入/模型字段与误差读数，**没有 native 接触输出**；`native-error-seed` 的 8.19e-7 主要来自 native f64 与上游 f32 的输入口径差（G2 README 已定位 27 ULP→1 ULP 的输入位姿口径），**不能作为容差依据**（G5 独立复核观察同此结论）。

## 3. `hfield-box` 配对检查（G6-3 要求）

- 目标：先测 native 每 world 接触数能否与设备输出配对；不能配对则如实记录「不可配对、样本不增」。
- 检查结果：**不可配对**。理由：
  1. native 桥暴露的碰撞相关 ABI 只有 `mjwarp_native_copy_primitive_geometry`（模型字段拷贝，`include/mjwarp_collision_probe.h`），没有 `contact`/`narrowphase`/`hfield` 输出入口（`native/model_probe_collision.cpp` 全文只有字段拷贝与校验）；
  2. G2/G3/G4/G5 的 native 材料均定位为「输入/读数」，接触由宿主或设备计算；`hfield-box` 没有 native 接触参考；
  3. 因此 `hfield-box` 的 native 每 world 接触数无法获得 → **不可配对、样本不增**，native 诊断样本仍为 `ellipsoid-box` 1 case / 2 接触点（且该 case 的 native 侧也只是读数）。
- 影响：在 native 接触输出实现前，native 诊断的「真实产品输出样本」无法扩到 hfield-box；本片不伪造配对。

## 4. 已拍板后的约束与后续

1. **B 是现行口径**：native 侧不提供放行信号，这是选项 B 的既定后果；若未来要 native 放行，须先扩充真 native 样本（另立 native 接触输出探针小片）并重新拍板。
2. 是否另立 native 接触输出探针小片（让 `hfield-box` 可配对、样本可扩）：**尚未决定**，不属于本片容差拍板。
3. A（粗门禁）留待真 native 样本扩充后再议；届时需给出「诊断列 vs 生效列」分栏方式与报告标注（NP03 `np03_admitted` 字段），本片不提前写。

**本片按 B 收口：native 侧待验收、诊断读数见证据；`map.json` G6-3.1 不提供放行信号，不进入生效契约。**
