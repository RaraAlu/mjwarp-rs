# QA 复核 W9-00 只读诊断（hfield/sdf/shadow refit 风险 + P0-01 计时文档 + R06 recipe params）（`2026-11-11-w9-00-readonly-diagnostics`，提交 `bd7c9e3a`）

- 复核对象：卡 `2026-11-11-w9-00-readonly-diagnostics`（`docs/development-evidence/plan/w9-00-readonly-diagnostics/`）；内容提交 **`bd7c9e3a`**。
- 性质：只读复核（零 GPU、零窗）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，0 一般 + 0 低 + 2 观察）**。

## 1. 独立核验

- **W9-1 采集源身份 ✅**：`mjwarp_collision_hfield`/`sdf` 回执 `sourceSha256=12d50d68…`（我核 receipt 原件一致）、`render_megakernel` `cf356b72…`、`render_depth_seg` `1e3f273c…`——4 个单元记录 sha 与当前装配一致；`scene_bvh_ray` 记录 `8dad4d61…` vs 当前 `scene.cu` 实测 `caac8c86…`（`d35cab68` 所致，已知第 10 条待裁）。
- **refit 词频 ✅**：我独立 grep `product.cu`/`scene.cu`/`render_shadow.cu`/`render_shade.cu` 的 `refit` 命中 **均为 0**；与「refit 是 host 侧 API、不在被采内核源码」的解释一致。
- **W9-2 P0-01 文档缺口 ✅**：`docs/replication-plan.md` L405–419 的 2026-10-22 P0-01 复核记录含验证命令/平台/通过/失败/未测/误差与容量/剩余缺口，**无 refit 计时边界说明**（全文 `refit` 出现 0 次）——记为文档缺口、不改历史报告，建议合理。
- **W9-3 R06 7 vs 28 ✅**：`multiframe_recipe.rs` 188 行、`RECIPE_PARAM_KEYS` 数组 26 项 + nworld + cam_res = 28 项口径；`tests/render_flex_validate.rs` 单帧侧覆盖 28 项；复核时点 backlog 卡为 `backlog/0`（我在 `bd7c9e3a`/`76bd3ccc` 两个 blob 上核实），维持不修理由与补齐触发条件在案。
- **边界/证据 ✅**：提交 `bd7c9e3a` 仅本目录 + `.normify`（**无 src/tests/fixtures/scripts 改动**）；manifest 4/4、selfcheck RESULT: PASS、policy_scan 0 命中、四门禁记录齐、UTF-8/LF 无 BOM/裸 CR/NUL。

## 2. 发现

### 观察
- **O1**：inventory 的 `source.sha256` 是单文件 `product.cu`（`cad08d3e…`），而 W9 比对用的 `recorded_source_sha256` 是回执里的装配 sha（`12d50d68…`）——两者语义不同、各自正确；W9 文本「receipt/inventory 记录的 sourceSha256」表述略混合，建议后续注明「回执 sourceSha256（装配）」与「inventory source.sha256（单文件）」区别。
- **O2**：R06 backlog 卡后续已被治理链 promote/关闭（`8a04f15a` promote、`be3af2cd` 登记、`f2ad1ce2` 收口）——属后续片，不在本片范围；本片复核时点卡确为 `backlog/0`，两不矛盾。

## 3. 非声明

本复核只对诊断/核查的事实与范围负责；**不宣称**任何门禁或阶段准入通过；不宣称 NP 通过；
不宣称上游等价；不宣称位级等价；不宣称双平台或发布；零 GPU、零窗、不改判据/参考/容差/历史字节。
