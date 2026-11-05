# R06 多帧专题收口治理（只读、CPU-only）

- 任务：`2026-11-05-render-r06-multiframe-closeout`（owner Backend、p60、P8-渲染与BVH）
- 来源：Chief 2026-11-05 派工（QA G-3 复核 PASS `d8fc8eeb`、27/27 自检、0 阻断之后）
- 边界：**只读治理、CPU-only、零 GPU**；不改任何既有证据/代码/判据/spec/fixtures/guards；两张目标卡的 `status/title/acceptance/depends_on` 一律不动。

## 1. 结论

- 多帧专题链完整收口：G-1 负结果（`dbe8d2b7`）→ 诊断定论（采集缺每帧 `refit_bvh`，`77aeb6da`）→ 修复 r2（`ff8135e5`/`43295e49`）→ **G-2 采集有效**（`5f2bd0bb`/`38a7767c`，verdict pass-margin）→ 实体 A + 小修（`2a27c2fd`/`a2d7d4c5`）→ **G-3 验证窗 PASS**（`b3a05f2b`）→ **QA G-3 复核 PASS**（`d8fc8eeb`，27/27、0 阻断）。
- 已更新两张卡的 note（仅 `note` + `updated_at`）：
  - tracking 卡 `2026-11-04-render-r06-flex-multiframe`：**保持 backlog、不 start**；
  - roadmap 卡 `2026-10-11-roadmap-r06-p8`：**保持 proposed**，不宣称整族通过。
- QA G-3 复核 O1 登记为 backlog 卡 `2026-11-05-render-r06-multiframe-guards-eol-backlog`（今晚不改 `.gitattributes`）；O2/O3 记已知（见 §4）。

## 2. 覆盖边界（如实）

| 维度 | 已覆盖 | 未覆盖 |
| --- | --- | --- |
| 场景 | 单场景 `scene-flex`（312/315 同源模型） | 多场景 / 多模型 |
| 帧集 | 3 帧：`0 / +0.10 / +0.20 m` 刚性锚点位移（同一次采集，`Δz=0.100`） | 长序列、时间积分、物理步进 |
| 帧内判据 | rgb ≤1 LSB 且 beyond=0；depth 严格位等（frame-000 单像素签名 allowlist `ULP≤4`）；seg 逐像素全等；两跑整块 sha 相同 | 跨设备 / 跨编译形态 |
| 跨帧 | 可见性：cam2-only 变化像素 ≥384（实测 1167/1074、1262/1126） | 动画通用性、几何一致性全广度 |
| 规格 | spec `1d9fa5ce…` / 10,506 B 未变、不重签 | — |

## 3. 证据索引

| 材料 | 路径 | 关键提交 |
| --- | --- | --- |
| 冻结 spec | `docs/development-evidence/render-bvh/r06-flex-multiframe-criterion/spec.md` | — |
| G-2 采集落库 | `docs/development-evidence/render-bvh/r06-flex-multiframe-capture-g2/` | `5f2bd0bb` / `38a7767c` |
| G-2 QA 复核 | `docs/development-evidence/render-bvh/r06-flex-multiframe-capture-g2-check/` | — |
| 实体 A | `docs/development-evidence/render-bvh/r06-flex-multiframe-entity/` | `2a27c2fd` / `241d702f` |
| 实体小修 | `docs/development-evidence/render-bvh/r06-flex-multiframe-entity-fix/` | `a2d7d4c5` |
| G-3 验证窗 | `docs/development-evidence/render-bvh/r06-flex-multiframe-verify-g3/` | `b3a05f2b` |
| G-3 QA 复核 | `docs/development-evidence/render-bvh/r06-flex-multiframe-verify-g3-check/` | `d8fc8eeb` |
| guards | `scripts/validate/render-bvh/guards-r06-multiframe.json`（28 条冻结，aggregate `6a6cca42…`） | — |

判据参考：`spec §4` 帧内三相机口径、`spec §5` 跨帧 cam2 判据（384 验收 / 576 采集余量）、`spec §6` 签名 allowlist。

## 4. 已知项（QA G-3 复核 observation，均不阻断）

- **O1（→ backlog）**：`guards-r06-multiframe.json` 由冻结器 `ConvertTo-Json` 生成、行内分隔为 CRLF，仓库未对该路径声明 `-text`/`eol`，导致 raw 工作树 sha256/bytes（`64bfecdf…`/11,899 B）与 HEAD blob（`bf4b9302…`/11,696 B）不一致；G-3 证据已用 **LF 规范化快照**兜底，判据与数值不受影响。**今晚不改 `.gitattributes`**，登记 `2026-11-05-render-r06-multiframe-guards-eol-backlog`（backlog，不排期）。
- **O2（记已知）**：禁词扫描若按字面匹配，可能在非声明的否定句中误报（「动画一致性」类词）；后续证据措辞避开该类精确短语，本卡已避。
- **O3（记已知）**：G-3 窗口证据未留 `repo-write-lock` 的 status/acquire/release 原始输出；后续窗口证据补该留痕。

## 5. 非声明

本卡只登记覆盖边界与证据指针：**不写** R06 整族通过 / 动画与几何一致性全广度覆盖 / 与上游实现的等价结论 / 发布依据；**不重开**任何已 verified 判定；两张目标卡状态保持 backlog / proposed 不变。

## 6. 变更清单（本卡范围内）

- `.normify/changes/2026-11-04-render-r06-flex-multiframe.json`：仅 `note` 追加收口段 + `updated_at`；
- `.normify/changes/2026-10-11-roadmap-r06-p8.json`：仅 `note` 追加收口段 + `updated_at`；
- 新增 backlog 卡 `.normify/changes/2026-11-05-render-r06-multiframe-guards-eol-backlog.json`；
- 本证据目录（README / receipt.json / manifest.txt）。

## 7. 待 QA 一致性复核的检查点

1. 两张卡 `status` 仍为 `backlog` / `proposed`，`title/acceptance/depends_on` 与 HEAD 前逐字一致（仅 note/updated_at 变）；
2. 收口 note 含覆盖边界字段（单场景、3 帧、0/+0.10/+0.20、帧内位级 + 跨帧可见性、动画通用性/多场景/长序列未覆盖、spec sha、证据指针）；
3. O1 backlog 卡存在且 status=backlog、今晚未改 `.gitattributes`；
4. 边界：`unchanged src/, tests/, fixtures/, scripts/` 与既有证据目录；
5. 本目录 manifest 自核（UTF-8/LF/无 BOM、首行 `#`、裸 CR/TAB 自查）。
