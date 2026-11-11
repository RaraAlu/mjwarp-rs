# NP5-ERRATA-E 复核（QA）：E 判据状态 append-only 更正（NP5-02/NP5-04 + W5 working docs）

- 复核对象：卡 `2026-11-11-np5-e-metric-errata`（登记 `40b7f16a`）；内容提交 **`a98f718b`**。
- 性质：只读复核（零 GPU、零窗）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，0 一般 + 0 低 + 2 观察）**。

## 1. 20 条 append-only 更正逐条在位

- **计数**：`corrections.json` `repo_evidence[]` 16 条 + `working_docs[]` 4 条 = **20**；拆分
  **NP5-02 9 条**（evidence 5 + README 2 + receipt 2）/ **NP5-04 7 条**（evidence 4 + README 2 + receipt 1），
  与 receipt `corrections.{np5_02:9, np5_04:7, total:20}` 一致。
- **表格一一对应**：两 addenda 的逐处更正表行号 1..9 / 1..7 连续，我把每行 `文件:行` 归一化后与
  corrections 的 (basename, line) 集合比对：**locations_match=True（missing=[] extra=[]）**。
- **位置真实**：16 条引用位置我逐条打开对应历史文件核过——`evidence.json` 的 scope / criterion.e_metric /
  coverage.not_covered_or_blocked[0] / conclusion.reasons[0] / non_claims[0]、README §5 表 E 行与 §6 未完成
  理由、receipt `coverage_blocked[0]` / `non_claims[0]`、np5-04 的 np01.status / watch-np04 items[1] /
  user_decision_points[e-metric].topic / conclusion.np01_status / README §1/§4 / receipt np01_status，
  **全部落在范围内且确为 E 状态的历史表述位**（行号含 `948-952` 区间）。
- **W5 working docs 4 条**：`w5-np01-02-plan.md`（8,983 B `99dce362…`）、`np5-rulings-memo.md`
  （10,307 B `7a1f21a7…`）、`w5-rulings-package/05-e-metric.md`（6,389 B `389a5f08…`）、
  `w5-rulings-package/manifest.txt`（1,134 B `d81efb84…`）——我逐个核 sha/bytes 与 corrections 记录一致，
  正文含 A2/2026-11-05/更正段；确认**不在仓（`git ls-files` 无匹配）**，属 target 工作材料，与卡声明一致。

## 2. 历史字节未改 + manifest 4→5

- np5-02 四个历史文件与 `ac840ff7` 基线**逐字节相同**：README `15a247c7…`/5,483、evidence
  `6526ca13…`/32,689、receipt `98b05ef7…`/10,211、selfcheck `0a71e377…`/1,356。
- np5-04 四个历史文件与 `82c3186a` 基线**逐字节相同**：README `20d5e976…`/3,788、evidence
  `4a9f42cb…`/6,177、receipt `fd4ef266…`/2,206、selfcheck `bb527433…`/1,044。
- 两目录 manifest 均由 4 条重算为 **5 条**（新增 addenda），我按行重算 sha/bytes 与文件集合：
  **5/5 全对**；表头注明「addenda added；historical 4 files unchanged」（np5-04 另注 summary only、
  不声称任何门禁通过）。
- addenda 自身 sha/bytes 与 errata receipt 记录一致：np5-02 `5c5280db…`/5,057、np5-04 `94b41c8f…`/4,589。

## 3. 分类/结论/站点数据零改动

- 因四个历史文件字节未改，以下**全部保持原值**（我不只看自述，直接读 evidence/receipt）：
  `precision_deviation_zero_items_verified=false`、`type_deviation_count=0`、`any_gate_passed=false`、
  608 站点 / 唯一 598、33 张类型表、engine-semantic source 20 + RE-3 204 站点、NP02 inventory 102/98、
  32 缺口处置 9/2/3/4/6/2/2/4、4 未实现子系统 blocked、7 项残余阻塞、7 个裁决点 id 集合
  （`U8/U7`、`ABI`、`eol`、`equivalence-release`、`e-metric`、`unimplemented-4-schedule`、
  `scene_bvh_ray-drift`）——与 NP5-02/NP5-04 我原复核结论一致。
- **E 口径只认**：已签 A2（2026-11-05，选项①「同码跨后端核对」，签名级，范围 CONVEX/SDF ×
  dist/pos/frame、97 标量、「实测 worst ≤ 已签 atol」、不用 E 比值、不宣称 native）+ **全产品 30 路径
  E/atol 核对未完成**；「E 指标未定稿（2026-10-27）」只作为已过时/已撤回历史表述保留。
- 我核了两 addenda 中「未定稿」全部出现位置（np5-02 3 处 / np5-04 4 处）：均在「历史表述」列或
  「已过时口径」说明句中，**无一处作为现行口径**；两 addenda 均明写「结论不变」「不宣言 NP01 通过」，
  np5-04 明写 `any_gate_passed` 仍 false、裁决点总数仍 7。**未出现「E 完成」「NP01 通过」类表述。**

## 4. 扫描 / 门禁 / 边界 / 编码

- `policy_scan`：对本目录 README/receipt/corrections + 两 addenda **5 文件 0 命中**；
  `--self-test` **34/34 PASS**（我自行重跑）。
- 该卡四门禁运行记录：fmt/clippy/clippy-cuda `passed=true`、unit `1,406 passed / 0 failed / 402 ignored`。
- 提交 `a98f718b` 只含 `.normify/**` 与三个证据目录（两 addenda + 两 manifest + errata 目录 5 文件），
  **无 src/tests/fixtures/scripts 改动**；UTF-8/LF 无 BOM、无 NUL、无裸 CR（含 addenda 与 errata 全文件）。
- 本目录 manifest 4 文件（README/receipt/corrections/logs/selfcheck），manifest 不自列。

## 5. 发现

### 观察
- **O1 `historical_snapshot` 为归一化摘要而非逐字引用**：`corrections.json` 的 `historical_snapshot`
  把多字段块用「 / 」连接、长句用「…」省略（如 evidence 948-952 的 item/status/impact 三段合并），
  addenda 表格同口径。位置与语义我在位核验通过；但若后续有人按字面 grep 原文会不匹配，
  建议在 corrections.json 或 README 注一句「历史快照为归一化摘要、非逐字引用」。
- **O2 W5 working docs 只在 target、仓内不可复现**：4 条更正（3 个文档 + 材料包 manifest）是工作材料，
  `git ls-files` 确认未入仓，只能在本机按 sha/bytes 复核；README §2.3/§5 已明确该口径，记观察即可。

## 6. 非声明

本复核对 ERRATA-E 的 20 条更正、历史字节、不变量、口径与扫描负责；**不宣称** E 完成、NP01/NP02 通过、
上游等价；不宣称位级等价；不宣称双平台等价；不改判据/参考/容差/历史字节。
