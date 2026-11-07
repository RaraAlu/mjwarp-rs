# NP01 A2 契约旧句治理注记（契约 L260 指向注记，不改历史原句）

任务：`2026-11-07-precision-np01-a2-contract-governance-note`（owner QA）。日期：2026-11-07。
边界：只给契约加一条带日期的治理注记 + 本证据目录；**不改历史原句、不改 A2/A2 复核证据、不改其它章节**；零 GPU、不采集、不开窗。
前置：A2 卡 `2026-11-05-precision-np01-e-criterion-a2`（verified，提交 `b9cb2993`）、A2 复核卡 `2026-11-05-precision-np01-e-criterion-a2-check`（verified）。

## 1. 定位（只读）

- 旧口径位置：`docs/numerical-precision-contract.md` L260「边界：**E尚未定稿**。上表E用「上游f32作产品代理」算出；G6首步会用产品输出直接对native f64重算确认，」，以及同段 L261–L262「并补采…后另测。若重算后E超过1.05倍红线，须回炉报告，不自动放宽本表。」。
- 所属节：「### 接触字段容差分档（用户2026-10-27拍板）」的末尾边界段（L260–L264）。
- 同文件既有事实：L267 起已有「### E指标定稿（NP01，用户2026-11-05拍板选项①）」；L283–L284 已写「自本口径起…写『已签』」与「本口径之前已verified的历史summary…是当时快照，不回改」。
- 因此同一文件内「L260 说 E 未定稿」与「L267 起说 E 已定稿」按字面并存，属需要治理的可读性冲突。

## 2. 判断：属历史形态 → 不回改原句，只加带日期注记

- 依据 1（A2 卡约定）：A2 卡 README §4 写死「只新增一个小节；被删改既有行 = 0」「不触碰…接触字段 native 侧选项 B 节、容差分档表」。L260 属 A2 落库前既有行。
- 依据 2（A2 复核卡结论）：`np01-e-criterion-a2-check/README.md` 观察 O1 明确「契约 L260 既有『E尚未定稿』与同文件新增『E指标定稿』并存，按字面冲突…属历史快照；建议后续单独治理卡在 L260 加指向注记（不动 A2 证据）」。
- 判定：该句是 **2026-10-27 拍板当时的边界/状态描述**，属历史形态，不是需要就地改正的当前生效规格；按既定规则「自 A2 起写已签、历史不回改」，**原句与前后文逐字不动**，只在同段末尾加带日期的治理注记，把读者指向 A2 卡/提交与下节定稿口径。
- 不采用「就地订正原句」：那会改写被 A2 冻结的历史文本，且与 A2 复核卡建议（加注记）不一致。

## 3. 处置（契约只增不删）

- 在 L264「除本表外，本文件其它章节的数字与口径不动。」之后、L265「记录见[接触字段容差分档]…」之前新增 5 行（1 空行 + 4 句注记，插入后位于 L265–L269）：
  - `注记（2026-11-07）`：声明 L260 旧句及其后两句为 2026-10-27 拍板当时的历史形态、保留不回改；
  - 声明 E 指标已于 **2026-11-05 由用户签署**（签名级，同码跨后端核对，A2），以 A2 卡 `2026-11-05-precision-np01-e-criterion-a2`、提交 `b9cb2993` 与下节「E指标定稿（NP01，用户2026-11-05拍板选项①）」为准；
  - 声明已签范围（CONVEX/SDF 两路径 × dist/pos/frame 三字段块，97 标量）判据为「实测 worst ≤ 已签 atol」的核对，**不再以 E 比值作判据**；
  - 声明本节 E 列数值仅保留为当时「上游 f32 作产品代理」的估算与历史记录，不构成当前设计依据。
- `git diff --numstat docs/numerical-precision-contract.md` = **5 增 / 0 删**；L260–L264 原句逐字未动（`git diff -U0` 仅新增 hunk，无 `-` 行）。
- 契约仓库 blob（LF）：before `82f6bb24895d981d2b5b95aad571f1e1d6ebae4b` / 27,605 B / 605 行 / sha256 `12c7b7248bc2e1c6e54dadd096ffa18a5907c99d88f2c35b4793271a2e24f17b`；after（提交后 blob）见 receipt 的 `contract_change.after`（LF 28,445 B / 610 行）；工作区因 `core.autocrlf=true` 为 CRLF（29,055 B / 610 行），提交时按 LF 归一。

## 4. 全仓同类旧口径清单（只列，未顺手改）

| 位置 | 性质 | 处置 |
| --- | --- | --- |
| `docs/numerical-precision-contract.md` L260–L262 | 2026-10-27 节内边界/状态句（历史形态） | **本卡加带日期注记**（原句不动） |
| `docs/development-evidence/native-contact-frame-normal-tiers/README.md:31` | 同一句在已落库历史证据中的副本（冻结） | 不改（历史不回改；证据目录零 diff） |
| `docs/numerical-precision-contract.md` L284（插入前）/ L289（插入后） | 治理规则本身（「历史 summary 不回改」） | 不改 |
| `docs/development-evidence/np01-e-criterion-a2/README.md:22`、`.../np01-e-criterion-a2-check/README.md:36` | A2 措辞规则与复核记录本身 | 不改 |
| `docs/development-evidence/np01-e-criterion-a2-check/facts.json` O1 | 该观察的记录（建议加注记） | 不改（本卡即其落地） |
| `.normify/changes/2026-11-04-precision-np01-e-recompute.json:10` | A2 之前任务卡 intent：引用契约 L260「E 尚未定稿」，并把「③维持现状（E 不定稿、只留 upstream-f32 判据）」列为候选 | 不改（历史任务记录，不回改） |
| `.normify/changes/2026-11-04-g10-p5-family-closeout.json:18`、`...-g10-p5-family-closeout-check.json:18` | A2 之前收口/复核卡的量词规则「NP01 =『已核对、E 指标 1 项待用户签』」 | 不改（A2 措辞规则明确历史快照不回改） |
| `.normify/changes/2026-11-04-render-r01-p8-closeout-check.json:10,16` | A2 之前复核卡「NP01 E 指标仍待用户签」「…待用户签（A2）」 | 不改（历史任务记录） |

扫描口径：`git grep -n -E 'E尚未定稿|E 尚未定稿|E指标1项待用户签|E指标待用户签'`（全仓 tracked 文件）。命中分四类：① 契约 L260 旧句（本卡加注记、原句不动）；② 契约 L266 与本卡证据内的**引用性提及**（新注记/规则本身，非旧口径）；③ 冻结证据副本（`native-contact-frame-normal-tiers/README.md:31`）；④ `.normify/changes` 历史任务卡（上表三行，均不改）。

## 5. 未波及面与门禁

- 零 diff：`docs/development-evidence/np01-e-criterion-a2/`、`...-a2-check/`、`.../native-contact-frame-normal-tiers/`、`docs/development-evidence/render-bvh/`、`src/`、`tests/`、`fixtures/`。
- 四门禁 fmt / clippy / clippy-cuda / unit：结果见 `.normify/runs/2026-11-07-precision-np01-a2-contract-governance-note/`（本卡要求全过；unit 1359 过 / 0 败）。
- 本目录 `manifest.txt` 自核（README + receipt，manifest 不自列）；三文件 UTF-8/LF/无 BOM/裸 CR·TAB 0。
- 零 GPU、不采集、不开窗、不重跑窗口。

## 6. 非宣称

- 不宣称 NP01 / NP03 / NP04 整族通过；不宣称 native 口径通过；不宣称 E 比值作判据。
- 不改任何 atol/rtol、不新增数值、不改任何已 verified 证据的语义与数字。
- 本卡只做契约可读性治理（指向注记），不产生新的判据效力、不改变任何验收状态。
