# QA 复核 W8-02 P4 准入契约核对（`2026-11-11-w8-02-p4-ct`，提交 `495a5159`）

- 复核对象：卡 `2026-11-11-w8-02-p4-ct`（`docs/development-evidence/plan/w8-02-p4-admission-contract/`）；内容提交 **`495a5159`**。
- 性质：只读复核（零 GPU、零窗）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，0 一般 + 0 低 + 1 观察）**。

## 1. 契约事实（独立核验）

- **矩阵覆盖**：**8 行**（`G01`、`G02`、`G03`、`G04`、`G05`、`G06`、`G19`、`G26`）逐条对 `docs/compatibility-matrix.md` 行号核实，
  **锚点 0 漂移**；与 receipt `matrix_rows=8` 一致。
- **漂移记录**：本片 `drift_notes` **0 条**（无新漂移；矩阵指针已指向实际锚点行）。
- **伞卡/依赖**：伞卡 `None` 板面 `None/None` 与契约一致；
  依赖 **0 项**状态逐卡对板面（不一致 0 项）。
- **准入前提四项**：`ct_no_gap=False`、`np_gate_passed=False`、`prereq_stages_passed=False`、`window_available=False`（全 false）。
- **T4 重基线**：`t4-historical-capture-rebaseline-decision` 未定 ⇒ P4 AD 口径待裁，契约记为「等待 T4 重基线裁决 + NP 门禁」。
- **阶段/伞卡**：阶段 `P4`；伞卡 `2026-10-11-roadmap-p4`=backlog/0；`P4` 阶段：slices_total=1/verified=0，未验收依赖 1 项——逐项对板面（不一致 **0** 项）。
- **缺口 5 项**：`G-T4`、`G-NP`、`G-DEPS`、`G-GATE`、`G-AD`；`np_gate_status=「None」`。
- **结论旗标**：`ct_complete=True`、`admission_claim=False`
  （contract 与 receipt 一致）。**本片只做 CT，不构成任何准入**。
- **证据/门禁/编码**：`plan/w8-02-p4-admission-contract/` manifest **4/4 全对**、selfcheck **RESULT: PASS**、
  policy_scan 3 文件 **0 命中**；提交仅本目录 + `.normify`；UTF-8/LF 无 BOM/裸 CR/NUL。

## 2. 发现

### 观察
- **O1 前置/缺口口径记录正确**：T4 重基线裁决 + NP 门禁——契约把未满足项如实列入缺口与结论，未写成已满足；
  contract / receipt / README 三处一致不宣称准入。

## 3. 非声明

本复核只对契约数字/锚点/前置状态负责；**不宣称**任何准入通过；不宣称 NP 通过；
不宣称上游等价；不宣称位级等价；不宣称双平台或发布；零 GPU、零窗、不改判据/参考/容差/历史字节。
