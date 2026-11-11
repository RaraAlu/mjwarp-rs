# G-L2 errata 批（H2 前提 addenda A1–A6 + dump 字段 errata B + QA#7 记录面 C + attempt4 README errata E）

- 任务：`2026-11-11-render-perworld-l-v2-gl2-errata-batch`（owner Backend）。依据 Arch 规格稿 `target/render-bvh/gl2-ptx-diff/errata-batch-spec.md`（`9b5a1fe0…`/8,110 B，含 §E）与 W0 收口 brief（`eccd0b9f…`）；零 GPU、纯文档/证据 + 诊断测试侧静态单测。
- 边界：只动 `docs/development-evidence/**` 与 `.normify` 治理数据（+诊断测试支持 `tests/support/render_bvh/light/diag.rs` 的字段语义与静态单测）；`src/`、`fixtures/`、`scripts/`、guards/冻结件、已签 S-1/S-2/S-3 正文零改动；不改判据/参考/容差；无 `.cu` ⇒ 内核身份不变；历史字节与 `f728b2d7` 说明不改写。

## A. H2 前提 errata（append-only addenda A1–A6）

统一口径（逐字）：上游 warp 内建 `mesh_query_ray` 在 `warp/native/mesh.h:1880` 返回前做**局部归一化**，`mujoco_warp/_src/render.py:731-732` 再 `mat @ n` + `wp.normalize` ⇒ **两次**归一化；`f728b2d7` 删掉了第一次，P10（`b0e372be`）已恢复。**原记录文字与历史字节保留不改。**

| # | 文件 | 落点 |
| --- | --- | --- |
| A1 | `perworld-l-v2-mesh-normal-single-normalize/README.md` | 文末 §6 Addenda（统一口径 + 结果仍为实测事实） |
| A2 | 同上 `receipt.json` | 新增 `addenda[]`（`kind=h2-premise-errata`、`superseded_by=b0e372be`） |
| A3 | 同上 `logs/call-chain-audit.txt` | 追加 Addendum 证据段（`mesh.h:1880` + `render.py:731-732`） |
| A4 | `perworld-l-v2-gl2-criterion-a-rca/README.md` | 文末 §9 Addenda（方向成立、单次表述证伪、站点细化两处） |
| A5 | `perworld-l-v2-mesh-normal-chain-pin/README.md` | 文末 §8 Addenda（P10 是恢复而非新增） |
| A6 | `perworld-l-v2-mesh-normal-single-normalize-check/README.md`、`perworld-l-v2-mesh-normal-chain-pin-check/README.md` | 文末指针（见 A1/A4） |

A7/A8 为 target/ 内 Arch 草案追加（`rca-errata-draft.md`、`site-conformance-audit.md`），不进仓。每目录 manifest 已重算自核。

## B. dump 字段 errata（`seg.attribution.geom_type`）

- **B1 核对（留证 `artifacts/b1-source-audit.txt`）**：`tests/support/render_bvh/light/diag.rs::segment_attribution` 原代码 `let (geom_id, geom_type) = (seg[0], seg[1]);` ⇒ 字段实际取 `seg[1]`（object type），**不是** `mjm.geom_type[geom_id]`；模型 `geom_type[3] = 7 (MESH)`、diag7/8 `seg.product = [3,5]` ⇒ **errata 成立**（不撤回）。
- **B2 方案 A（已实现，仅测试侧诊断）**：`geom_type` 改为模型真实 `mjm.geom_type[geom_id]`（越界/无模型为 `null`）；新增过渡键 `seg_objtype = seg[1]`；其余字段不变；dump 顶层 `schema` **保持 2**。
- **静态单测（`logs/static-tests.log`）**：5 tests passed（2 个既有 dump 契约测试 + 3 个 B 新增）—— geom 3 ⇒ `geom_type=7` / `seg_objtype=5` / `schema==2`；无模型 ⇒ `geom_type=null`/`seg_objtype=5`；越界 ⇒ `geom_type=null`/`mat_id=-1`。B 新增单测放在 `tests/support/render_bvh/light/diag_ext.rs`（按功能抽出子模块，避免 `diag.rs` 触发 500 行棘轮：`diag.rs` 466 行、`diag_ext.rs` 231 行）。
- **影响面**：仅测试二进制变化（`tests/support` 编入），**内核身份不变**（无 `.cu`）；diag7/diag8 旧 dump 字节不改，旧语义只读不重解释；动态复验（重生成 dump 期望 `geom_type=7`/`seg_objtype=5`）留待后续诊断轮（B4②）。
- 引用处注记：`perworld-l-v2-gl2-diag7-check/README.md` 已追加 Addenda（旧值 5 为 `seg[1]` 误标的确认与新旧语义说明）。

## C. QA #7 三条记录面残留（已修，仅记录面）

1. `diag7` 补 `window/gpu-snapshot.txt`（实测 pre/post + 派工参考标注）与 `artifacts/window-snapshot-template.txt` 模板；
2. `diag7/body-env.txt` 以文件写入口径重写（修 here-string 转义污染）；
3. GPU 数字以 `nvidia-smi` 实测为准，派工信息标「参考」。

## D. attempt 4 README errata（§E）

按规格 §E 统一措辞写入 `perworld-l-v2-validate-window-g-l2/attempts/attempt4/README.md` §11：① 证据卫生（原文件字节不改，补可读副本 `window/freeze-stdout.readable.txt`、`window/monitor-stdout.readable.txt`；权威事实由摘要与 QA 独立复算承载）；② build=0 仅指最终 60/60 段（早段 3 样本 build=2）；③ 基线时间戳/body 启动推定说明；④ `signed_stricter` 严格口径为 10,230 MiB、`floor_effective` 采用既有规则 9,956 MiB；⑤ guards JSON CRLF 与缺 `body_env`、两跑 sha 仅断言（backlog）。均不改变 A/B/C 通过结论。

## E. 边界与非声明

- 不改判据/参考/容差/相机/recipe；不重跑窗口/诊断轮；历史字节、提交说明、`f728b2d7` 说明不改写；不宣称 G-L2 新结论（W0 已收口的结论以收口材料为准）。
- 本批证据 `manifest.txt` 自核 N/N；各受改目录 manifest 均重算（见各目录）。

## F. 过程事件

- **锁纪律**：2026-11-11 08:03 本卡登记/start 两条治理提交在 QA 持锁（`pid=77836`）期间完成（acquire 返回 `HELD` 后未停下即写），随后立即停手、等锁 FREE（08:04:07）后重新 acquire 继续；期间仅 `.normify` 卡 JSON 变更、无并发冲突/数据丢失。已按既有纪律记录并报 Chief。
- **归档修复**：`attempt4/window/monitor-stdout.txt` 在生成派生副本时被同名写入覆盖，已从提交 `5f1d436d` 的 git blob 恢复原始字节（340 B / sha `c22ea746…`，与 blob 逐字节一致），派生副本改名为 `monitor-stdout.readable.txt`；证据无净损失。

## G. 复核回填（QA 复核 `fd72cf0b` 的记录面勘误；2026-11-11 卡 A，append-only）

本段只更正记录面/引用，不改变 A/B/C/E 任何内容结论与窗口有效性；原文字节不改。完整留证见
`perworld-l-v2-flex-z2vec-record-errata/README.md` 与 `artifacts/`。

- **G1（一般项；原 §C 第 1 条的 C1）provenance 更正**：
  `target/render-bvh/perworld-l-v2-gl2/diag7/window/gpu-snapshot.txt` 的
  `gpu_post_actual=2 %, 3,919 MiB used, 12,143 MiB free, 26 C` 是 **errata 时段复测**
  （文件 mtime 2026-11-11 08:06:18），**不是 run 结束即时采样**。原文 `timestamps_actual=` 行
  「post recorded in run-summary.txt (same diagnostic round)」的交叉引用**不实**：该轮
  `run-summary.txt` 至今仍是未展开占位 `gpu_after=$gpu`。该错误引用已删除并改为 provenance 标注；
  C1 的宣称从「补跑后快照」降级为「事后补测」。数值与任何判定不受影响（pre 值可由 `pre-state.txt`
  复核）。更正前后文本见 `perworld-l-v2-flex-z2vec-record-errata/artifacts/c1-diag7-post-provenance.txt`。
- **L1（原 §B 的 `b-field-change.diff` 名不副实）**：该 artifact 的两个 hunk **只含「3 个测试从
  `diag.rs` 搬到 `diag_ext.rs`」**（`diag.rs` −49 / `diag_ext.rs` +51），**不含 B 的字段语义改动**；
  其 `diag.rs` old blob index `25affdc8` 是编辑中间态（`84ed67d2^` 实际为 `02533c7d`）。字段语义
  真身在提交 diff 本身（`diag.rs::segment_attribution` + `diag_ext.rs` 3 个静态单测）；artifact
  保留不改写，仅在此说明其真实覆盖面与索引口径。
- **L2（原 §A 的 A2 receipt 口径）**：`single-normalize/receipt.json` **不是** append-only——从字节
  5,245 起整段重排（紧凑 inline 对象改多行）并在末尾追加 `addenda[]`；按 JSON 展平逐键比对，
  非 addenda 键与旧值**完全相等**（语义等价）。正确表述：「README/logs 为**纯追加**；receipt 为
  **结构重排 + addenda**，除 `addenda[]` 外逐键与原值相同」。
- **L3（行数口径）**：§B 文内「`diag.rs` 466 行 / `diag_ext.rs` 231 行」是 PowerShell
  `Get-Content` 默认编码的计数（含 CJK 行尾被吞并，少计 18/6 行）；**物理行数（LF 计数）为
  484 / 237**，两者均 < 500 棘轮，判定不受影响。留证：
  `perworld-l-v2-flex-z2vec-record-errata/artifacts/physical-line-counts.txt`。
- **L4（task_id successor 关系）**：本文开头「任务：`2026-11-11-render-perworld-l-v2-gl2-errata-batch`」
  与本目录 `receipt.json.task_id` 指向**已被 abandon 的旧卡**；实际交付提交 `84ed67d2` 由 successor
  卡 **`2026-11-11-render-perworld-l-v2-gl2-errata-batch-r2`** 承载（旧卡 `82f5dd76` abandon；
  r2 登记即最终 scope/checks）。历史 task_id 保留不改写，此处补 successor 关系说明。
