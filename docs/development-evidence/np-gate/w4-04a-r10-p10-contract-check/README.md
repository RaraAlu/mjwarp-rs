# QA 复核 W4-04A r10-p10 批量渲染调度（P10）契约核对（`2026-11-11-w4-04a-r10-p10-contract`，提交 `be36d0ea`）

- 复核对象：卡 `2026-11-11-w4-04a-r10-p10-contract`（`docs/development-evidence/plan/w4-04a-r10-p10-contract/`）；内容提交 **`be36d0ea`**。
- 性质：只读复核（零 GPU、零窗）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，0 一般 + 0 低 + 1 观察）**。

## 1. 契约事实（独立核验）

- **矩阵覆盖**：**5 行**（`R10`、`A04`、`A07`、`A09`、`M11`）逐条对 `docs/compatibility-matrix.md` 行号核实，
  **锚点 0 漂移**（行内容与 id/族/阶段一致）；与 receipt `matrix_rows=5` 一致。
- **指针漂移记录**：`drift_notes` **1 条**——G-SRC 指针/锚点已在文档逐行核对（来源指针漂移）；
  指针行确为说明段/他阶段行、解析锚点确为矩阵实际行（我逐行核对，记录准确）。
- **伞卡/依赖**：伞卡 `2026-10-11-roadmap-r10-p10` 板面 `status=proposed` / `checks=0`
  与契约一致；依赖状态逐卡对板面**全部一致**；`coverage_mapping` 5 条与矩阵行一一对应。
- **缺口 4 项**：`G-NP`、`G-PRED`、`G-SRC`、`G-CONTRACT`；`np_gate_status=「等待 NP 门禁」`，
  detail 明写 NP01/NP02 未验收（NP5-00…04 为证据卡、NP 伞卡未 verified）⇒ **片 B 不得开窗、不得宣称准入**。
- **结论旗标**：`ct_complete=True`、`admission_claim=False`
  （contract 与 receipt `admission_claim=False` 一致）。
- **证据/门禁/编码**：`plan/w4-04a-r10-p10-contract/` manifest **4/4 全对**、selfcheck **RESULT: PASS**、
  policy_scan 3 文件 **0 命中**；提交仅本目录 + `.normify`；UTF-8/LF 无 BOM/裸 CR/NUL。

## 2. 发现

### 观察
- **O1 漂移记录与「等待 NP 门禁」口径记录正确**：`drift_notes` 的指针/锚点行我逐行核对准确；
  contract / receipt / README 三处一致地不宣称准入、片 B 等待 NP 门禁——本片只做 CT，未生成平台证据。

## 3. 非声明

本复核只对契约数字/锚点/口径负责；**不宣称**任何阶段或切片准入通过、NP 通过、上游等价、位级等价、
双平台或发布；零 GPU、零窗、不改判据/参考/容差/历史字节。
