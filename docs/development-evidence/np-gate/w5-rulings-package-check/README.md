# W5 用户裁决点材料包完整性核对（QA）：7 项裁决材料 + 索引/建议

- 核对对象：`target/render-bvh/gl2-ptx-diff/w5-rulings-package/`（target 工作材料，不入仓）；
  manifest 以磁盘现状 **`d81efb84…`/1,134**（含 NP5-ERRATA-E 对 05 的追加）为准。
- 性质：只读核对（零 GPU、零窗）；本目录只写本卡证据。
- 结论：**核对通过（0 阻断，0 一般 + 2 低 + 3 观察）**；材料结构完整、不预设结论、05 口径已更新且与
  仓内 NP5-02/04 errata 一致；两处事实数字需修正（06 缺口条数、03 UTF-16 计数）。

## 1. 10 个文件 sha/bytes 与 manifest 逐条一致

- manifest 共 9 行（不自列）＋ manifest 自身 = **10 个文件**；我按行重算：**9/9 sha/bytes 全对**、
  文件集合相等（无缺失/多余）；manifest 自身 sha256 = `d81efb84228261849573280abd2a8cb98a8b6c3d9e1c4789183fd1618d2f0f15`、
  1,134 B，与派工口径一致（Backend 22:31 追加 05 后重算）。
- 文件清单：`01..07` 七个裁决项（3,657 / 3,722 / 3,344 / 4,618 / 6,389 / 4,272 / 4,608 B）、
  `README.md` 2,065 B、`w4-w8-slice-suggestions.md` 5,147 B、`manifest.txt` 1,134 B。

## 2. 每项结构齐全

- 7 个裁决项逐个检查必备段：**背景（事实与证据指针）/ 裁决对象 / 选项 A–C / 影响面 /
  供参考的建议（不预设结论）/ 若裁决后的后续动作 / 证据指针**——**7/7 全齐**（无缺失段）；
  选项均给出 A/B/C 三个方向；每项都有「不预设结论」明示 + 条件化建议（若选 X → 后续动作 Y）。
- 事实诚实性：明确无仓内定义的项写了「待 Chief 校准」（01 的 U8/U7 条目编号）、
  未核实项显式标注「未核实」（07 的 `scene_bvh_ray` 是否已被 2026-11-04 补采覆盖）；未见「已裁决/已批」类结论句。

## 3. 事实抽查（独立对仓内证据）

- **01**：`g10-p5-mesh-cap20` 卡 verified、提交 `307219fe` 存在；「`met 8 / partial 4 / unmet 0 /
  uncovered 6`」与 `w0-w9-plan-final.md:42` 原文一致；「A2 三片未登记」与板面实况一致。
- **02**：`r07-splat-precheck` 卡 verified；`r07-p8` 仍 `proposed`；ABI 范围（81 槽 splat 常量槽拆分 +
  9 槽重排、不含 S3b 判据）与计划 §4 一致。
- **03**：`.gitattributes` 候选规则与 `docs/development-evidence/** -text -diff` 已存在口径一致；
  「不做批量字节转换」与证据保全区既有纪律一致（**但 UTF-16 计数有偏差，见 L2**）。
- **05**：A2 签名记录 `b9cb2993` 存在；「实测 worst ≤ 已签 atol、不用 E 比值、CONVEX/SDF ×
  dist/pos/frame、97 标量、不宣称 native」与仓内 `np01-e-criterion-a2` / 契约 L267-L272 一致；
  追加的「确认与执行记录」（2026-11-11）明写「A2 即 E 判据定稿 + 全产品 E/atol 核对未完成」、
  16 处 append-only 更正（9+7）、两目录 manifest 5 条——**与 NP5-02/04 errata 逐项一致**，且注明
  「§2 原文保留、不选边」。
- **06**：4 张阻塞卡 = NP5-03 的 4 条 `blocked-with-card`（flex collision/constraint/solver/island），
  不计覆盖、不写成通过，与卡 H 口径一致（**但「其余 25 条」计数不符，见 L1**）。
- **07**：`scene_bvh_ray` 历史源 `8dad4d61…`（inventory 行在位）、当前 `scene.cu` 实测
  `caac8c86…`（LF 规范后同值、无 CRLF）；`d35cab68` 确实改动 `src/render/bvh/cuda/scene.cu`；
  计数快照 `source_identity_drift=9 / evidence_verified=91 / receipts_total=715 /
  receipt_ptx_entities_verified=711` 与 inventory 实况一致。

## 4. 7 项 id 与 NP5-04 七裁决点一一对应

| 材料包 | NP5-04 裁决点 id |
| --- | --- |
| 01 `u8-u7-w2-criteria` | `U8/U7` |
| 02 `abi-r07-splat` | `ABI` |
| 03 `eol-batch` | `eol` |
| 04 `equivalence-release` | `equivalence-release` |
| 05 `e-metric` | `e-metric` |
| 06 `blocker-cards`（4 卡排期） | `unimplemented-4-schedule` |
| 07 `scene-bvh-ray-drift` | `scene_bvh_ray-drift` |

**7/7 一一对应**，无多余/缺项；`w4-w8-slice-suggestions.md` 是派工建议、不属于裁决项（不参与 7 项映射）。

## 5. 发现

### 低
- **L1（`06-blocker-cards.md`）缺口条数不符**：原文「其余 **25 条** archive/closed/N-A/recorded/deferred」
  ——NP5-03 账本为 32 缺口、4 条 blocked，**其余 28 条**（9+2+3+6+2+2+4）；若按「排除 3 条 closed」
  算是 25，但同一句又把 closed 列在「其余」里，口径自相矛盾。建议改为「其余 28 条（含 3 条
  closed-implemented）」或「25 + 3 条 closed 分开列」。
- **L2（`03-eol-batch.md`）UTF-16 计数口径不准**：原文「169 个 UTF-16LE+BOM 文件（全仓 217）」。
  我实测（按文件前两字节 `FF FE`）：`docs/development-evidence` **168 个**、全仓（排除
  `target/.git/.normify`）**216 个**；169/217 仅在把 `docs/development-evidence/g10-p5-gpu-kernels/
  gpu-window/cases/cylinder-box/out1/frame.bin`（**二进制文件**、首两字节恰为 `FE FF`）当 UTF-16 计入时成立。
  建议改为「168 个 UTF-16LE+BOM（全仓 216）」或注明「含 1 个二进制误命中（BE 首字节）」。
  该数字是 eol 裁决的范围依据，故记为低。

### 观察
- **O1 少量选项标题带「推荐路径」标注**：`02` 的「A. 拆分签（推荐路径）」、`07` 的「A. CPU-only 补采
  （推荐路径之一）」在「选项」段内就给出倾向，与 README「建议仅供参考、不预设裁决」的表述略有张力；
  建议把推荐性标注只放「供参考的建议」段（或改「较稳路线」等中性词）。
- **O2 `04` 的「S-1 已签」缺独立签名指针**：其他签名项都有卡/提交/记录路径，S-1 只引「W0 行/W3 签名口径」；
  建议补一条卡号或提交号，便于用户核签。
- **O3 材料包为 target 工作材料**：manifest 历史版本不可从仓内复现（当前只能核 9/9 与 05 的追加），
  与 NP5-ERRATA-E 的 O2 同口径；不影响本次完整性结论。

## 6. 非声明

本核对只对材料包的**完整性/一致性/事实指针**负责；**不代表**任何裁决点已有结论，不宣称通过/等价/双平台，
不改任何卡 acceptance/判据/参考/容差。
