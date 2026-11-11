# R06@P8 伞卡关闭评估：`2026-10-11-roadmap-r06-p8`（结论：**不关闭**）

- 任务：治理卡 `2026-11-11-w1-r06-p8-umbrella-closure-eval`（owner Backend）。
- 被评估卡：`2026-10-11-roadmap-r06-p8`（R06@P8 柔性体渲染，**proposed / checks=0**，阶段占位）。
- 日期：2026-11-11（空闲段；**零 GPU、零窗**）。
- 性质：**只读评估报告**——不改任何卡状态、不动依赖边、不改判据/参考/容差。
- 判定口径（Chief 2026-11-11）：伞卡 6 条 acceptance **全部满足才关闭**；任何一条需要
  「超出 R06 家族收口」的声明（动画与几何一致性 / 等价 / 准入 / NP 门禁）→ **不关闭**，只出报告。

## 1. 一句话结论

**不关闭**。6 条 acceptance 仅 **1 条满足**（记录真实测试与未测项）、1 条部分（模块与写入范围由切片承接但伞卡自身未补齐）、
**4 条未满足**（动画与几何一致性、性能测试、NP01/NP02 先核对、NP03/NP04 验收）；且伞卡
`checks=0`（阶段占位「无机器检查项，不参与就绪判定」）→ 无关闭判定依据。`r06-p8` 保持 `proposed`。

## 2. 逐条 acceptance 映射（6 条）

| # | acceptance（原文） | 判定 | 依据 / 证据指针 |
| --- | --- | --- | --- |
| 1 | 动画与几何一致性 | **unmet（超范围）** | 家族收口 receipt `non_claims` 明写「no animation-and-geometry consistency claim」、`multi_frame_evolution_covered=false`、`known_limits`「multi-frame evolution not covered (no criteria, no reference)」；多帧收口卡 `2026-11-05-render-r06-multiframe-closeout`（verified）note 已写死「因『动画与几何一致性』广度未覆盖，本节点保持 proposed」（G-2 `5f2bd0bb`/`38a7767c`、G-3 `b3a05f2b`、QA `d8fc8eeb`） |
| 2 | 先补齐模块与写入范围。 | **partial（非伞卡自身补齐）** | 伞卡 `schedule.files=[]`、note「写入范围采用保守声明/启动前须补细分模块契约」；家族收口把工作拆成 9 个切片卡（R06-S0…S3c，均 verified 且有各自文件范围）——按「伞卡不启动、由切片承接」口径不阻塞，但伞卡自身写入范围仍未补齐 |
| 3 | 性能测试须独占GPU。 | **unmet（未执行）** | 家族收口 `known_limits` 明写「no performance」；无任何 R06 性能测试证据（R06 系族全部证据为单帧/多帧位级与功能证据，`gpu_used=false` in closeout）；条件句本身未触发→未满足 |
| 4 | 记录真实测试与未测项。 | **met** | 家族收口 receipt `known_limits`（多帧演化未覆盖、z2vec 潜伏缺陷、flex-vs-flex tie、attempt5 失败性质、无上游等价/parity/性能/Linux/T4/higher spp/multi-world）+ README §3 判据/§5 覆盖边界；各切片证据目录与 QA 复核卡在仓 |
| 5 | 先核对NP01与NP02。 | **unmet（NP 未验收）** | `2026-10-11-roadmap-np01`/`np02` 均 **backlog/checks=0**；NP5-04 汇总卡 `82c3186a` receipt：`any_gate_passed=false`、`np01_status`/`np02_status` 均「未验收」（含 4 个未实现子系统、32 缺口处置、7 残余阻塞）；NP5-02 `ac840ff7`、NP5-03 `d46f435f` 仅审计/处置，不构成通过 |
| 6 | 再验收适用NP03与NP04。 | **unmet（未验收）** | `2026-10-11-roadmap-np03`/`np04` 均 **backlog/checks=0** 且依赖 np01/np02；属用户级裁决（等价/发布口径 + Linux 恢复），当前无验收证据 |

**计数**：met **1** / partial **1** / unmet **4** ⇒ `all_acceptance_satisfied=false`。

## 3. checks 与可启动性

- 伞卡 `checks=[]`（阶段占位；note：「无机器检查项，不参与就绪判定；真实验收由已验证收口卡/后续可执行切片承担」）。
- 因此即便 acceptance 全满足，也无 `normify gate/check/done` 的机器判定依据；**不得为关闭而编造 checks**。
- 依赖：`depends_on=[2026-10-11-roadmap-r01-p8]`，该卡 **verified**（依赖本身满足；不构成关闭障碍）。

## 4. 影响面（若关闭会怎样）——本次不关闭，不动依赖边

- **全板扫描（`git ls-files .normify/changes`，共 **1,164** 张已提交卡）**：`depends_on` 引用 `r06-p8` 的卡 **0 张**；
  `checks` 引用 `r06-p8` 的卡 **0 张**（扫描另命中 1 张为**本评估卡自身**的 json 守卫引用该卡路径，属自引用，不计）。
- 后继阶段卡的 R06 前置已由**家族收口卡** `2026-11-04-render-r06-flex-closeout`（verified）承接
  （例：`2026-10-11-roadmap-r10-p8` 现 `depends_on` 含 r01–r09 p8 + r06-flex-closeout，语义修正见 `9830bb76`/`496c01e6`）。
- 故：**关闭与否对当前后继卡就绪无直接影响**（0 引用）；本片结论「不关闭」独立由 acceptance/checks 未满足成立。
  若未来有卡把伞卡写入 `depends_on`，关闭会解除其阻塞——但本片**不改任何依赖边**。

## 5. 边界与非声明

- 只读评估：不改 `r06-p8` 的 status/title/acceptance/note/依赖边；不改任何其它卡。
- 不改判据/参考/容差/夹具；零 GPU、零窗。
- **不宣称**：R06 整族通过、准入、上游等价、发布、NP01–NP04 通过、性能达标。

## 6. 证据文件与 manifest

- `README.md`（本文件）
- `evaluation.json`（6 条逐条映射 + 证据卡状态 + 影响面，机器可读）
- `receipt.json`（结论 `umbrella_closed=false` / `all_acceptance_satisfied=false` / decision）
- `manifest.txt`（3 文件 sha256+bytes 逐文件自核，不列自身）
