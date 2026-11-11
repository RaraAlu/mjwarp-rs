# NP5-01a A-3 addenda（2026-11-11，D-1 卡第一节）

本文件是 A-3 证据的 append-only 补充；`evidence.json`/`README.md`/`receipt.json` 历史字节**不改**。

## 1. L1：40 条补 `kind`/`label` 字段（不抽样）

- A-3 复核 L1：`evidence.json.entries[]` 缺此前各片都有的 `kind`/`label` 字段（只能由 id 段推导）。
- 本次在 **`rows-fields-v1.json`** 中按 40 行补齐（`row_count: 40`）：每行含
  `id`/`kind`/`label`/`classification`，`kind`/`label` 与 `sites.json` 三行并集逐条一致
  （构建时已校验），行顺序与 evidence 一致。
- 原 `evidence.json` 字节未动（`c9a9676f…`/53,532）；`rows-fields-v1.json` 为版本化字段补页
  （`supersedes: null`，因原文件未被改写）；A-3 manifest 已重算以纳入新增两文件。
- **1 条 policy redaction**：`com_position:rigid_com_position:rounding-note:0` 的 `sites.json` label
  含历史正面措辞（政策禁词），本证据不复制该措辞，`label` 记 `policy-redacted` 并指向
  `sites.json` 原文位置；其余 39 条 `kind`/`label` 与 `sites.json` 逐字一致。

## 2. O2：engine-semantic 措辞沿用 A-2 先例（记录）

- A-3 的 3 条 com_position `engine-semantic` 沿用 A-2 同型站点先例口径（实现路线/数据流差异，
  `bit_level_guarantee:false`、不作数值等价结论）；此处仅记录口径来源，不改变分类。

## 3. 非声明

补齐仅提供机器可读字段与口径记录；不改变 A-3 分类、不重判 source 级、不宣称 NP01 通过或位级保证。
