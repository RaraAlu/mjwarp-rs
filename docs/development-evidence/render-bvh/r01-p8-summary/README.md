# R01@P8 渲染上下文聚合收口（10 项映射刷新 + NP 核对 + 已知限制）

任务：`2026-11-04-render-r01-p8-closeout`（owner Arch）。日期：2026-11-04（空闲段）。
**边界：证据与记账收口，零 GPU、零 `src/`/`tests/`/`fixtures/` 改动、不改判据/容差/参考、不回改已 verified 证据。**
前置：A（R06 flex 家族收口）已 verified，证据 `1de0321a`；本卡引用 A 而不重述其数字。

## 1. 一句话结论

R01@P8（渲染上下文）的切片 **S0a–S8 + 渲染 perf 全部 verified**，QA 独立复核 0 阻断；此后 **R02 / R05 / R06 / R08 / R09 / R10 家族也已交付并 verified**。
本卡把 `plan.json` 的十项映射从 `planned` 刷新为 `delivered` 并补证据指针，登记 NP 核对与已知限制。

**仍未完成/不适用（写死）**：
- **NP01 的 E 指标口径 1 项待用户签（A2）** → 不作「100% 类型匹配 / 零精度偏离」声明；
- **NP03 双平台不适用**（用户 2026-10-24 裁决 Linux 终止）→ 不宣称双平台；
- **release 全量构建未跑** → 已知限制，发版/对外声明时由用户点名；
- **R07 高斯泼溅未做，不在 R01 范围**；
- Linux/T4、更高 spp、与上游完全一致均未覆盖。

## 2. 十项映射（`plan.json.r01_p8_mapping` 已同步刷新）

| # | 项 | 结论 | 关键证据 |
| --- | --- | --- | --- |
| 1 | worlds | 单 world 默认行为 verified；nworld>1 由 R10 独立交付 | S6a；S8；R10-S1 `2026-10-28-render-bvh-r10-s1-world-state`、R10-S2 `2026-10-28-render-bvh-r10-s2-multi-world` |
| 2 | cameras | 字段/筛选/分辨率/射线 verified；选择矩阵与异构分辨率由 R08 交付 | S1、S5、S6a；R08-S1/S2 |
| 3 | assets | mesh/texture/material 输入与上传 verified | S1（116 字段/89,941 标量 0 diff）、S3（+ coverage fix）、S4（3×1040×4 bit-equal） |
| 4 | bvh | 场景/网格构建、refit、查询 verified | S2a（AABB 56/56 bit、0 ULP）、S2b（383/382、0 ULP）、S2-close（六映射 + R01–R19 拒绝清单）、S3（436×5 射线 0 ULP） |
| 5 | all_default_parameters | 28 参数逐项 resolution（34 项 + 15 拒绝） | S6a，QA `1c0929e` 34/34、15/15 |
| 6 | lifecycle | create/destroy、显式 refit、无隐式刷新 verified | S6b（100 循环 growth 0 B；QA `e3ea82c`）、S8 |
| 7 | rgb_output | 打包/通道序/容差 verified | S7b2（120,000/120,000 零差；QA `38298e5`）、S7b3（5 变体 + AA 零差；QA `7e5affd`）、S8 |
| 8 | depth_output | 原始深度、无命中 0、`get_depth` 缩放 verified | S7b1（最大 0 ULP；QA `51ee2d0`）、S8；R08-S1 的 clamp 顺序 |
| 9 | segmentation_output | `(id,type)`、背景 `(-1,-1)`、精确相等 verified | S7b1（seg 0 mismatch）、S8 |
| 10 | resource_and_layout_checks / rejection_surface | 布局寿命检查 + 15 类拒绝 verified | S6b（回归 5/5）、S6a（15/15） |

补充映射项（同一 `r01_p8_mapping` 内）：
- **shadows_skybox_splats_flex_hfield** → `delivered-except-r07`：R09（S1/S2/S2b/S3a/S3b）、R02（S1/S1b/S2/S2b/S3/S3a）、R05（S1/S2/S3a/S3b）、R06（S0–S3c + 家族收口 `1de0321a`）全部 verified；**R07 未做，不在 R01 范围**。
- **multicamera_heterogeneous** → delivered（R08-S1/S2/S3；QA `337abc14` pass/0 blockers）。
- **multi_world** → delivered（R10-S1/S2；QA `a3ce62e1`/`5cc6a3d5`）。

## 3. NP 核对表（写死措辞）

| 项 | 结论 | 依据 | 不可写成的表述 |
| --- | --- | --- | --- |
| NP01 | **已核对；E 指标口径 1 项待用户签（A2）** | 类型门 `2026-11-03-precision-np01-cuda-type-gate`、host source gate、字段表 B1–B3/b1b/b2a–b2e/b3、host-device expr/fields、E 重算 `2026-11-04-precision-np01-e-recompute` 全 verified | 「100% 类型匹配」「产品精度偏离为零」 |
| NP02 | **满足** | `2026-10-23-np02-frozen-inventory`、`2026-11-03-precision-np02-evidence-gate`、`-receipt-rehash`、`2026-11-04-precision-np02-new-kernel-evidence`、`-ptx-entity-restore` 全 verified | 「证据覆盖 100%」以外的扩展结论 |
| NP03 | **不适用、不宣称** | 用户 2026-10-24 裁决 Linux 终止；Windows 证据见 `2026-10-23-np03-windows-evidence`（verified） | 任何双平台结论 |
| NP04 | **适用项满足（相对基准）** | `2026-10-23-np04-windows-performance`（verified）+ 渲染 perf `2026-10-25-render-bvh-perf`（verified，QA `a2123fb`） | 绝对性能承诺 |

**E 指标说明**：`np01-e-recompute` 结论是「native f64 口径在现有归档上无法定稿」（缺设备原始接触数组，`ellipsoid-box/pos` 界跨红线不可判定），三候选待用户签字。本卡只登记「已核对、1 项待签」，不代签、不降级、不宣称。

## 4. 已知限制（逐条登记）

1. **release 全量构建未跑**：只在发版/对外声明等里程碑由用户点名时跑。
2. **R07 高斯泼溅不在 R01@P8 范围**（无切片、无采集、无设备路径）。
3. **更高 spp**：AA 仅覆盖既有样本级参考。
4. **Linux/T4**：用户已裁决终止，不宣称；全部 GPU 证据为 RTX 4070 Ti SUPER / driver 596.36 / SM89。
5. **与上游完全一致未声明**：全部判据为同 GPU / 同 recipe / 同编译形态下的参考型（B 类前提）。
6. **R05/R09 的窗口例外**：hfield 的 authoring-only util<60 例外、R05-S3b 的 validation-only 例外均按各自证据记录，不当作常规阈值通过。
7. **R06 单帧边界**：flex 只到单帧位级，多帧演化未覆盖（跟踪卡 `2026-11-04-render-r06-flex-multiframe`）。

## 5. 非声明

- 不声明完整 R01 通过；
- 不声明双平台（Linux/T4）；
- 不声明与上游完全一致；
- 不声明 release 全量已验收；
- 不声明 R07 或 R01 之外的任何渲染能力；
- 不作对外发布依据。

## 6. 证据索引

| 项 | 证据 |
| --- | --- |
| 预检与计划 | `docs/development-evidence/render-bvh-precheck/`（本卡在 §7 追加 2026-11-04 addendum，原 2026-10-25 结论未改） |
| S0a/S0b | `……/s0a-reference-fixture/`、`……/s0b-reference-capture/`、`……/independent-check/` |
| S1–S6b | `……/s1-model-input/`、`……/s2a-scene-bvh-host/`、`……/s2b-scene-bvh-gpu/`、`……/s2-scene-bvh/`、`……/s3-mesh-bvh/`、`……/s4-texture/`、`……/s5-camera-rays/`、`……/s6a-context-params/`、`……/s6b-context-lifecycle/`（各含 QA 复核目录） |
| S7a–S8 | `……/s7a-launch-abi/`、`……/s7b1-depth-seg/`、`……/s7b2-shading/`、`……/s7b3-flags/`、`……/s7b-followups/`、`……/s8-e2e/` |
| 渲染 perf | `……/perf/`、`……/perf-independent-check/` |
| 家族收口 | R02 `……/r02-skybox-summary/`、R08 `……/r08-summary/`、R06 `……/r06-flex-summary/`（`1de0321a`） |
| NP | `docs/development-evidence/np01-*`、`docs/development-evidence/np02-*`、`docs/development-evidence/np03-windows-*`、`docs/development-evidence/np04-windows-*` |

## 7. plan.json 记账

- `r01_p8_mapping` 十项 + 三个补充项逐项刷新（`status` + 证据指针，`delivered-except-r07` 见上）。
- `delivery_tracking.rows` 追加 `R01-P8` 收口行；`scheduling_notes` 追加 `N15`；`updated_at=2026-11-04`。
- **roadmap-r01-p8 本卡不 start/done**：roadmap 节点不承载实现，待家族汇总（C 系列）全部 verified 后统一关闭。
