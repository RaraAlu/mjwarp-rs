# QA 复核 W4-07A P8 全部渲染与查询准入核对（`2026-11-11-w4-07a-p8-contract`，提交 `16280b49`）

- 复核对象：卡 `2026-11-11-w4-07a-p8-contract`（`docs/development-evidence/plan/w4-07a-p8-contract/`）；内容提交 **`16280b49`**。
- 性质：只读复核（零 GPU、零窗）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，0 一般 + 0 低 + 1 观察）**。

## 1. 契约事实（独立核验）

- **矩阵覆盖**：**15 行**（`R01`、`R02`、`R03`、`R04`、`R05`、`R06`、`R07`、`R08`、`R09`、`R10`、`G28`、`M03`、`M11`、`A11`、`A07`）逐条对 `docs/compatibility-matrix.md` 行号核实，
  **锚点 0 漂移**；与 receipt `matrix_rows=15` 一致。
- **漂移记录**：本片 `drift_notes` **0 条**（无新漂移；矩阵指针已指向实际锚点行）。
- **伞卡/依赖**：伞卡 `2026-10-11-roadmap-p8` 板面 `backlog/0` 与契约一致；
  依赖 **4 项**状态逐卡对板面（不一致 0 项）。
- **准入前提四项**：`prerequisites_verified=False`、`np_gate_passed=False`、`ad_evidence_done=False`、`window_available=False`（全 false）。
- **前置依赖**：`2026-10-11-roadmap-p8-base`=backlog；`2026-10-11-roadmap-m03-p8`=proposed；`2026-10-11-roadmap-g28-p8`=verified；`2026-11-04-render-r06-flex-closeout`=verified——p8-base/m03-p8 未验收 + r07-p8 等 ABI + NP 门禁，均与我逐卡对板面一致。
- **缺口 7 项**：`G-BASE`、`G-M03`、`G-R07`、`G-NP`、`G-AD`、`G-G28`、`G-CLAIM`；`np_gate_status=「None」`。
- **结论旗标**：`ct_complete=True`、`admission_claim=False`
  （contract 与 receipt 一致）。**本片只做 CT，不构成任何准入**。
- **证据/门禁/编码**：`plan/w4-07a-p8-contract/` manifest **4/4 全对**、selfcheck **RESULT: PASS**、
  policy_scan 3 文件 **0 命中**；提交仅本目录 + `.normify`；UTF-8/LF 无 BOM/裸 CR/NUL。

## 2. 发现

### 观察
- **O1 前置/缺口口径记录正确**：p8 前置（p8-base/m03-p8 未验收）+ NP + ABI——契约把未满足项如实列入缺口与结论，未写成已满足；
  contract / receipt / README 三处一致不宣称准入。

## 3. 非声明

本复核只对契约数字/锚点/前置状态负责；**不宣称**任何准入通过；不宣称 NP 通过；
不宣称上游等价；不宣称位级等价；不宣称双平台或发布；零 GPU、零窗、不改判据/参考/容差/历史字节。
