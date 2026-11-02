# B 类判据前提标注（criterion audit follow-up，2026-11-02）

依据：Chief 2026-10-31 对 QA 判据审计（31 条 = A 9 / B 11 / C 11）的裁定 —— **B 类（参考-前提型）
全部降级：不得单独作发布依据，必须写明前提（同 recipe / 同编译形态 / 同 GPU+驱动 / 同 warp 版本），
前提不成立即不适用。** 本文件把前提落成文字；**不改判据数值、不改代码逻辑、不改 `.cu`**。

## B 类清单与前提

| CR | 判据 | 主要位置 | 前提 |
| --- | --- | --- | --- |
| CR-10 | depth `<= MAX_DEPTH_ULP(4)` | `src/render/kernel/output.rs`（常量 + `diff_depth`） | 同 recipe / 同编译形态 / 同 GPU+驱动 / 同 warp 版本 |
| CR-11 | depth 位级 | `output.rs::diff_depth`、`tests/support/render_bvh/s7b1.rs::compare_reference` | 同上 |
| CR-12 | RGB `<= 1 LSB` | `tests/support/render_bvh/s7b2.rs::diff_rgb` | 同上 |
| CR-16 | DepthSeg 跨路径位级 | `tests/render_sdf_reference.rs` | 同上 + 同内核身份 |
| CR-18 | U4 历史用法（曾用 CR-10/11 + CPU 采集参考判 SDF 正确性） | `tests/render_sdf_reference.rs` + 历史证据 | 原前提不成立（SDF 与 sphere 非同一 kernel 形态）→ 自 criterion v2 起改用位级 SDF 参考 |
| CR-23 | 物理上游常量镜像 | `src/` 相关常量（具体位置见 `materials/src-tolerance-constants.txt`） | 同上游版本 / 同 warp 版本 |
| CR-24 | g10 `bit_equal+4 ULP@scale` | g10 证据 | 同 recipe / 同形态 / 同 GPU |
| CR-25 | witness 1/24 超界 | 记录 | **记录未决**，不得写成通过 |
| CR-26 | faithful-fma 4 ULP | FMA 证据 | 同 FMA 形态 |
| CR-27 | 接触 `2e-6` | g10-p5 接触字段 | 同 recipe / 同形态 / 同 GPU |
| CR-28 | 接触 `1e-6`（B 组关联） | g10-p5 | 同上 |

## 规则（与 Chief 裁定一致）

- B 类结论**不得单独作为发布依据**，必须与 A 类或位级证据同现。
- 前提不成立即不适用，**不得做解释性放宽**。
- C 类（guards / manifest / unchanged 哈希）只作完整性证据，**不得单独支撑「判据零改 / 已通过」**。
- **CR-25 witness 1/24 超界维持「记录未决」**。

## 本次注释落点（仅注释/文档）

- `src/render/kernel/output.rs`：`MAX_DEPTH_ULP` 常量与 `diff_depth` 补 B 类前提注释。
- `tests/support/render_bvh/s7b2.rs`：`diff_rgb` 补 CR-12 前提注释。
- `tests/support/render_bvh/s7b1.rs`：`compare_reference` 补 CR-11 前提注释。
- `tests/render_sdf_reference.rs`：头部 B 类前提段（CR-10/11/12/16）**未纳入上一张卡（卡 files 未覆盖），该文件本次未改**；待后续小卡 `2026-11-02-render-sdf-reference-premise-header` 落地。
- 本文件：B 类清单与规则。

## 不改动项

指针值/常量数值、`.cu`（内核身份保持）、判据数值、清单、参考、`fixtures`、产品逻辑均未改；
本片只加注释与文档。
