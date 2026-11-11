# NP5-01a D-1 addenda（2026-11-11，D-2 卡第一节）

本文件是 D-1 证据的 append-only 补充；`evidence.json`/`README.md`/`receipt.json` 历史字节**不改**。

## 1. L1：88 条补 `kind`/`label`（不抽样）

- D-1 复核 L1：`evidence.json.entries[]` 缺此前各片都有的 `kind`/`label` 字段（A-3 已有补页先例）。
- 本次在 **`rows-fields-v1.json`** 中按 88 行补齐（`row_count: 88`）：每行含
  `id`/`kind`/`label`/`classification`，`kind`/`label` 与 `sites.json` 的 gjk_host/epa_host 两行并集
  **逐字一致**（构建时校验；无命中政策禁词的条目，无需 redaction）。
- 原 `evidence.json` 字节未动（`2d509468…`/115,577）；`rows-fields-v1.json` 为版本化字段补页
  （`supersedes: null`，因原文件未被改写）；D-1 manifest 已重算以纳入新增两文件。

## 2. 非声明

补齐仅提供机器可读字段；不改变 D-1 分类、不重判 source 级、不宣称 NP01 通过或位级保证。
