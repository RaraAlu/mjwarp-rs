# NP5-REVIEW-LOW：NP5 复核低项 append-only addenda（零 GPU、零窗）

- 任务：`2026-11-11-np5-review-low-items-addenda`（owner Backend）；基线 HEAD `dfed2c5d`。
- 来源：QA 对五张卡的复核全部 PASS（D-2 `06811c89`、NP5-01a 收尾 `5a5f0d9c`、NP5-02 `ac840ff7`、
  NP5-03 `d46f435f`、NP5-04 `82c3186a`；0 阻断 0 一般），本卡处置其中的 2 低（NP5-02）、1 低（NP5-03）、
  收尾 2 低、D-2 2 观察，以及 W4 §3 标题 24→25 笔误核实。
- 原则：**append-only、历史字节不改**；只改表述，不改分类/结论/数字；manifest 重算；policy_scan 0 命中。

## 1. 处置清单（9 条，见 `items.json`）

| # | id | 处置 | 落点 |
| --- | --- | --- | --- |
| 1 | `NP5-02-L1` | **已修**（append-only） | `np-gate/np5-02/addenda.md` §6：分层列 id（A-1a source 级 3 站点 + A-1b acc:5 同站点两层） |
| 2 | `NP5-02-L2` | **已修** | `np5-02/addenda.md` §6：§4 → §3 |
| 3 | `NP5-02-O1` | 记录不修 | `np5-02/addenda.md` §6：11 类原样口径为准，README 历史字节不改 |
| 4 | `NP5-03-L3` | **已修** | `np-gate/np5-03/addenda.md`：inventory `unimplemented=7`（历史快照）vs 现行剩余 4 |
| 5 | `NP5-01a-CLOSING-L1` | **已修** | `np-gate/np5-01a-closing/addenda.md`：双口径限定（算术 541→531 + rounding-note 67 = 598；反例 554/593） |
| 6 | `NP5-01a-CLOSING-L2` | **已修** | `np5-01a-closing/addenda.md`：11 站点片（+4 设备证据片 +5 治理卡） |
| 7 | `D-2-O1` | 记录不修 | 正向观察（三种字段落法口径一致） |
| 8 | `D-2-O2` | 记录不修 | 正向观察（附录单元登记合规、不计入 608） |
| 9 | `W4-GOV-§3-TYPO` | 已由前卡更正 | `docs/development-evidence/plan/W4-dependency-governance.md` §9（提交 `316b28dd`，13,791 B）核实无需再改 |

计数：**fixed_append_only 5 / already_corrected 1 / recorded_no_action 3 / total 9**。

## 2. append-only addenda（sha256 / bytes）

| 文件 | sha256 | bytes | 前置（历史字节不改） |
| --- | --- | --- | --- |
| `np-gate/np5-02/addenda.md` | `17e17efd8dfe3b4223fed10efb2ca8d6ef753c3ace1d43be38c1cecaa4264766` | 7642 | 5 文件（README/evidence/receipt/selfcheck/addenda）中历史 4 文件 sha 不变 |
| `np-gate/np5-03/addenda.md` | `587741d434440eebc31f2aa1d9c289152eaffa4f70c58b4c05e758fb434f16ba` | 1445 | 4 文件字节不变 |
| `np-gate/np5-01a-closing/addenda.md` | `5af42465b6c23ac395d0fff012958552b45be7a8d2218c80b3527d186921c916` | 2680 | 3 文件字节不变 |

manifest 重算：np5-02 5 条（不变）、np5-03 **4→5**、closing **3→4**；逐文件 sha256+bytes 自核见
`logs/selfcheck.txt`。

## 3. W4 §3 标题笔误核实（无需新写）

- `docs/development-evidence/plan/W4-dependency-governance.md`（13791 B，sha `ef78ba82`）已在
  §9「§3 片数更正（2026-11-11 卡 J）」append-only 更正：「24 片 = B0 4 + 实体 20」为算术笔误，正确为
  **25 片 = B0 4 + 实体 21**（GPU 窗数不变 3–4）；本卡只核实、不再改历史字节。

## 4. 验证

- selfcheck（`logs/selfcheck.txt`）：历史字节对照、append-only 前缀、manifest 自核、items 计数、
  W4 §9 在位、policy_scan 0 命中 + `--self-test` 34/34、编码/边界 PASS。
- 四门禁：fmt/clippy/clippy-cuda/unit（unit 1,406 过 / 0 败 / 402 ignored）。
- 本目录 `manifest.txt` 4 文件（README/items.json/receipt.json/logs/selfcheck.txt；不列自身）。

## 5. 边界与非声明

- 写域仅 `docs/development-evidence/np-gate/np5-02/`、`np5-03/`、`np5-01a-closing/`、
  `np5-review-low-items-addenda/` + `.normify/`；`src/`/`tests/`/`fixtures/`/`scripts/` 零改动。
- 零 GPU、零窗；不改判据/参考/容差、不改任何历史字节与数字；不宣称 NP01/NP02 通过、不宣称等价/双平台/位级保证。
