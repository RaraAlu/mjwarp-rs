# SDF 发布结论（criterion `render-sdf-criterion-v2.2`）

## 发布声明

**SDF 渲染能力已按 `render-sdf-criterion-v2.2` 验证（scene-sdf 参考位级 + scene-a 清单豁免，
前提见 v2.2）。**

（本文件只记录该声明与证据索引/边界；越界的跨上游一致、精度合规或验收类结论不在本文，具体禁词以项目门禁定义为准。）

## 生效判据

- `render-sdf-criterion-v2.2`，签署时间 **2026-11-01T21:42:50+08:00**（`approved_by=user via Chief`）；
  spec：`../sdf-criterion-v2.2/spec.md`（sha256
  `053e1104c8f6e25fc1fc99e47f5017ea52eb7b8b6236a117662b345cc76996e0` / 15,677 B）。
- 判据要点：§1 SDF 实体（`render_sdf_reference`）位级不变；§2 scene-a 精确差集清单（recipe 183 /
  aa2 172，清单外位级、清单内 `|Δ| ≤ 128 ULP`）；§3 B = 128 ULP 由精确 replay 精确可算（exact max
  73/91，非解析界）；§2.5 条款 1（变体臂复用 recipe 清单 + 零 GPU 静态断言）；§4.5 条款 2（S0b 跨
  配方 record-only，保留「>4 ULP ⊆ 冻结清单」fail-closed）。
- 数值沿革：v2 → v2.1 → v2.2 的 B=128、清单 183/172、exact max 73/91 均未变。

## 验证记录（窗口执行）

- 窗口卡 `2026-10-31-render-sdf-validate` 已 verified；attempt5 = **VALID ROUND**
  （9/9 实体 exit 0；常规 monitor 60/60；`zero_rust_compile=True`；post exe 9/9；清单 pre/post 2/2）。
- 冻结身份：guards sha256
  `9e000bfc1f8438a4bd6fd5f8c8ccf214c2ab43aee7c1ed1fce78ed51e7646c32` / 37,000 B、aggregate
  `034c7551226e321fab83b0ec4ef92a2dd82e9077ee3f31e7ad88f9a5d3b305b9`、94 = 9 exe + 83 inputs +
  2 清单；已入库 `scripts/validate/render-bvh/guards-sdf-validate.json`（证据副本在
  `../sdf-validate-window/attempts/attempt5/`）。
- 窗口双签：2026-11-02T08:29:18+08:00（window plan v4 + baseline v4，floor 10,376 MiB）。
- QA verdict：**2026-11-02T16:13:11+08:00** 独立复算，VALID ROUND、0 阻断 / 0 需修 / 2 观察已处理。
- attempt1–4：INVALID 轮次，归档在 `../sdf-validate-window/attempts/`，作废且不复用。

## 证据索引

| 证据 | 路径 |
| --- | --- |
| 判据 v2.2 spec/receipt/manifest | `../sdf-criterion-v2.2/` |
| 窗口执行记录（收口） | `../sdf-validate/README.md`、`../sdf-validate/receipt.json`（含 qa_verdict） |
| attempt5 证据 | `../sdf-validate-window/attempts/attempt5/`（facts/guard/时间线/monitor/outline/9 实体日志/manifest 35 项） |
| 窗口材料 v4 | `../sdf-validate-window-material-v4/`（plan v4、baseline v4、guards 身份） |
| 作废轮次 | `../sdf-validate-window/attempts/attempt1..4/`（保留历史，不作依据） |
| 参考冲突 RCA | `../sdf-validate/materials/reference-conflict-rca.md` |

## 边界与不声明

- 参考集为 **scene-sdf 单场景**（`fixtures/render-bvh/scene-sdf/capture-sdf/{sdf-mesh,sdf-occluder}`），
  不外推到其它场景。
- scene-a 豁免只覆盖 camera 1 的冻结清单像素（183/172），**清单外位级**、清单内 ≤128 ULP；
  **camera 0 位级**；SDF 实体位级。
- **B=128 由精确 replay 决定**（exact max 73/91 取 2 的幂），解析界（414.57/494.52）只记录、不用于定 B。
- **C 类证据（guards/manifest/unchanged 哈希）只是完整性证据，不得单独作为“通过”依据。**
- 不声明：跨上游一致、精度合规或验收类结论（具体禁词以项目门禁定义为准，本文不重复列出）。
- 本次未改判据数值、清单、参考、fixtures、产品 `src` 与内核身份
  （source `10070226…` / PTX `7782aeb1…` / 268,589 B）。
