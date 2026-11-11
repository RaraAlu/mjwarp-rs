# NP5-01a RE-3 addenda（2026-11-11，A-3 卡第一节）

本文件是 RE-3 证据的 append-only 补充；`evidence.json`/`README.md`/`receipt.json` 历史字节**不改**。

## 1. L1：rows[] 逐行机器可读字段补齐（24 行全覆盖，不抽样）

- RE-3 复核 L1：`evidence.json.rows[]` 只有 `classification`/`basis`/`device_observation`，缺
  RE-2 先例的逐行字段。本次在 **`rows-fields-v1.json`** 中按行补齐（`row_count: 24`）：
  - `engine-semantic`（15 行）：补 `why_engine_semantic`（= evidence 的 basis，逐行保留）+
    `bit_level_guarantee: false`；
  - `not-decidable`（7 行）：补 `not_decidable_reasons`（六类原因数组）+ `why_not_decidable`
    （= evidence 的 basis）+ `missing_evidence`；
  - `n/a`（2 行）：补 `why_not_applicable`（= evidence 的 basis）。
- 补字段与 `evidence.json` 一一对应（`source_evidence.sha256/bytes` 指向原文件），
  原文件字节未动；`rows-fields-v1.json` 为同一证据的版本化字段补页（`supersedes: null`，
  因原文件未被改写）。manifest 已重算以纳入新增两文件。

## 2. O1：not-decidable 缺因拆分的实测口径更正

- 派工摘要此前写作「no-reference-entry 40 站点 / no-lineinfo+frozen 33 站点」，与 evidence
  实测不符；按 `evidence.json.rows[].sites` + `not_decidable_reasons[]` 复算：
  - `no-reference-entry`：**4 行 47 站点**（tendon_velocity 7 + applied_force 12 +
    kinetic_energy 13 + sleep_wake 15）；
  - `no-lineinfo+frozen-artifact-only`：**3 行 26 站点**（tendon_wrap 9 + flex_edge 9 +
    flex_face 8）；
  - 合计 **7 行 73 站点**（与证据总数一致）。
- RE-3 证据数据本身不改；后续引用按 47/26 口径。机器可读值见
  `rows-fields-v1.json.not_decidable_reasons_split_measured`。

## 3. 非声明

补齐仅提供机器可读字段与口径更正；不改变 RE-3 分类、不重判 source 级、不宣称 NP01 通过或位级保证。
