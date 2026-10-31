# 重落球面 a-order fix（离线，零 GPU）

任务 `2026-10-31-render-sdf-sphere-dot-a-order-refix`（第 3 片，criterion v2 签发后）

- `render_math.cu`（290 行）：`a = dot3_ba(...)`（(1,0,2)）、删 `dot3_ch0`、注释按 RCA 更正（与 `8126868c` 同一 diff）。
- `render_shade.cu` L10/L14 的 `dot3_ch0` 注释残债同笔清掉（只改注释）。
- 内核身份：source `778216b2…`/PTX `34f91147…`/268,589 → **source `10070226…`/PTX `7782aeb1…`/268,589**；aa_resolve 不变。PTX 与 8126868c 修复形态一致；source 差异仅来自注释（注释计入源码哈希）。
- 门禁：fmt/clippy/clippy-cuda/unit 全绿（unit 1111/0/397）；`--no-run` EXIT=0；validate 0 错。
- **口径**：本形态只保证 scene-sdf 数据集内用于 attempt4 的位级比较；是否位级待 attempt4 验证。scene-a 球轮廓由 criterion v2 受控豁免覆盖。**不声明 SDF 通过。**
- 零 GPU/零窗口/零冻结；判据/参考/历史证据零改动。
