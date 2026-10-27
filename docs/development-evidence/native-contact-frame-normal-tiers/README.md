# native 接触字段 frame/normal 容差分档（用户 2026-10-27 拍板）

**任务**：`2026-10-27-native-contact-frame-normal-tiers`（owner Arch）。
**决定来源**：用户在 g10-p5 native 容差决策卡上选择「**按路径分档：CONVEX 2e-6 / SDF 5e-5**（推荐）」；
其余项（dist/pos/结构字段）按建议默认、用户未反对。**本片只登记契约量级与来源，不重跑 E、不改代码。**

## 1. 落点

`docs/numerical-precision-contract.md` §接触字段（native 侧）新增「### 接触字段容差分档（用户2026-10-27拍板）」：
逐字段/路径 atol/rtol 表 + 实测 worst + E（上游 f32 代理）+ 边界。原 §接触字段（native 侧）中
「本节不写入任何生效atol或rtol」一段保留为 2026-10-25 选项 B 的历史记录，并注明自 2026-10-27 起对接触字段不再适用。

| 字段 | 路径 | atol | rtol | 实测 worst | E（上游 f32 代理） |
| --- | --- | --- | --- | --- | --- |
| dist | CONVEX | 1e-6 | 1e-6·\|native\| | 9.31e-9 | 0.009 |
| dist | SDF | 2e-6 | 1e-6·\|native\| | 6.88e-7 | 0.34 |
| pos（各分量） | CONVEX | 1e-6 | 1e-6·\|native\| | 1.86e-8 | 0.017 |
| pos（各分量） | SDF | 1e-6 | 1e-6·\|native\| | 4.62e-7 | 0.42 |
| frame/normal | CONVEX | 2e-6 | 0 | 8.19e-7 | 0.41 |
| frame/normal | SDF | 5e-5 | 0 | 2.78e-5 | 0.56 |
| 结构字段 | 全部 | 0 | 0 | 0 失配 | — |

## 2. 边界（必须与表一起引用）

- **E 尚未定稿**：上表 E 用「上游 f32 作产品代理」计算；G6 首步会用**产品输出直接对 native f64 重算确认**，
  并补采 `hfield-box`、设备路径等 NVRTC launcher 接线后另测。
- 若重算后 `E_candidate > 1.05 × E_mjwarp`，须**回炉报告**，不自动放宽本表。
- SDF frame/normal 的 2.78e-5 含有限差分 `h=1e-4` 的方法学放大，非缺陷；分档后 E=0.56 相对红线有余量。
- 接触字段准入仍以**对上游 f32 判据**为主；本表用于 native 侧（native_error_v2）判定，不替代主判据。
- 结构字段零容差不变；其它章节数字与口径不动。

## 3. 数值来源

- 决策背景与历史选项 B：`docs/development-evidence/native-contact-tolerance-decision-b/README.md`（含
  `qa-native-f64.json` 读数与样本量说明：真 native 当时仅 `ellipsoid-box` 1 case / 2 接触点）。
- 逐字段 max_abs 与 E：见决策卡所列 10 case「上游 f32 vs 原生 f64」逐字段统计（QA 独立重采；
  决策卡原文由 Chief 转达，数值摘录见本目录 `receipt.json.decision_card`）。
- 本轮**未重算**任何 E，也未新增采集；只做契约登记。

## 4. 不宣称

不宣称 E 已定稿、不宣称 G6 放行、不宣称 native 接触字段已通过；本片只记录用户批准的量级与边界。
