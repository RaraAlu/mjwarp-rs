# NP5-ERRATA-E addendum（W8-00 卡第一节，2026-11-11）：QA 观察 O1/O2 记录

本文件为 append-only：本目录 `README.md`/`corrections.json`/`receipt.json`/`logs/selfcheck.txt` 历史字节不改；
`manifest.txt` 4→5 条（新增本 addenda）。任务：`2026-11-11-w8-00-coverage-audit`。

## QA 观察 O1：`historical_snapshot` 是归一化摘要，非逐字引用

- QA 复核（`c0c6d3ca`）观察：`corrections.json` 的 `repo_evidence[].historical_snapshot` 是**归一化摘要**
  （以「 / 」连接、使用省略号与英文改写），**不是逐字引用**；原文与位置以历史文件字节为准。
- 本注记明确该口径：`historical_snapshot` 仅用于指路与对照（file/line/field 为定位），不构成逐字引文；
  逐字原文以 `np5-02/README.md`、`np5-04/README.md` 与两目录 `evidence.json`/`receipt.json` 对应行
  （历史字节保留、未改）为准。
- 不修改 `corrections.json` 历史字节；本条为 append-only 说明。

## QA 观察 O2：W5 working docs（target，不入仓）——记录，不修

- 4 条 W5 working docs 更正位于 `target/render-bvh/gl2-ptx-diff/`（`w5-np01-02-plan.md` §8、
  `np5-rulings-memo.md` §7、`w5-rulings-package/05-e-metric.md`、材料包 `manifest.txt`）；`target/` 不入仓，
  只能本机按 sha 复现，SHA 见 `corrections.json.working_docs[]` 与 receipt `working_docs`。记录，不修。

## 边界与非声明

append-only；零 GPU、零窗；不改判据/参考/容差、不改历史字节；
不宣称 NP01/NP02 通过、不宣称 E 完成、不宣称上游等价/双平台。
