# 重落球面 a-order fix（离线，零 GPU）

任务 `2026-10-31-render-sdf-sphere-dot-a-order-refix`（第 3 片，criterion v2 签发后）

- `render_math.cu`（**309 → 308 行**；空行 30 / 纯注释 49 / 代码 229；字节 12,325；LF-only）：`a = dot3_ba(...)`（(1,0,2)）、删 `dot3_ch0`、注释按 RCA 更正（与 `8126868c` 同一 diff）。
  - **errata（2026-10-31，Chief 指出）**：本片此前报的「290 行」是误报（PowerShell 计数口径错误），**权威值为 308 行**（309→308）。提交消息里的「290」按「不改写历史」处理，不改写；以本 README 与 receipt 的 308 为准。
- `render_shade.cu` L10/L14 的 `dot3_ch0` 注释残债同笔清掉（只改注释）。
- 内核身份：source `778216b2…`/PTX `34f91147…`/268,589 → **source `10070226…`/PTX `7782aeb1…`/268,589**；aa_resolve 不变。PTX 与 8126868c 修复形态一致；source 差异仅来自注释（注释计入源码哈希）。
- 门禁：fmt/clippy/clippy-cuda/unit 全绿（unit 1111/0/397）；`--no-run` EXIT=0；validate 0 错。
- **口径**：本形态只保证 scene-sdf 数据集内用于 attempt4 的位级比较；是否位级待 attempt4 验证。scene-a 球轮廓由 criterion v2 受控豁免覆盖。**不声明 SDF 通过。**
- 零 GPU/零窗口/零冻结；判据/参考/历史证据零改动。
