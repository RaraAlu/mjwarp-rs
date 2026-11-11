# QA 复核：G-L2 errata 批 r2（`84ed67d2`）

- 任务：`2026-11-11-render-perworld-l-v2-gl2-errata-batch-r2-check`（owner QA）。复核对象：提交 **`84ed67d2`**（34 files；HEAD 当时、树 clean、锁 FREE）+ 卡 `2026-11-11-render-perworld-l-v2-gl2-errata-batch-r2`（verified）。
- 方法：独立只读复核，零 GPU；只写本证据目录与复核卡。卡声明**不采纳为结论**，下列数字均为自算/自测。
- 基线：`5f1d436d`（attempt 4 归档）与 `84ed67d2^`（`b5e04c84`）；复核期间 Backend 的 z2vec 片 1 已改写 `src/render/kernel/cuda/render_flex.cu`（未提交），本复核引用的是 `84ed67d2` 的树内容，`tests/support/render_bvh/light/{diag,diag_ext}.rs` 在 `84ed67d2` 之后无改动（`git diff 84ed67d2 -- …` 空）。

## 结论

**复核通过（0 阻断，1 一般 + 4 低 + 1 观察）。** A/B/C/E 四项内容成立、边界干净、manifest 全对、编码卫生、B 的实现与 3 个静态单测正确（我独立跑 5 passed/0 failed）、E 的两份可读副本与我解码原始字节逐行一致、过程事件时间线可核。一般项是 C1 补记的跑后 GPU 值来源不实（不影响任何数值判定）。

## 一、A 段：append-only 与历史字节（✅ 成立，附 1 条口径低项）

字节级核（对 `84ed67d2^` 的旧 blob 求 longest common prefix/suffix）：

| 文件 | 旧 B | 新 B | 共同前缀 | 结论 |
| --- | --- | --- | --- | --- |
| single-normalize/README.md | 6,774 | 7,986 | 6,774 | **纯追加** |
| single-normalize/logs/call-chain-audit.txt | 3,578 | 4,065 | 3,578 | **纯追加** |
| gl2-criterion-a-rca/README.md | 9,084 | 10,265 | 9,084 | **纯追加** |
| mesh-normal-chain-pin/README.md | 8,570 | 9,102 | 8,570 | **纯追加** |
| single-normalize-check/README.md | 4,266 | 4,622 | 4,266 | **纯追加** |
| chain-pin-check/README.md | 6,078 | 6,336 | 6,078 | **纯追加** |
| gl2-diag7-check/README.md | 7,970 | 8,519 | 7,970 | **纯追加** |
| attempt4/README.md | 9,393 | 11,374 | 9,393 | **纯追加** |

- 追加内容核对：A1 §6、A4 §9、A5 §8 的「上游两次归一化（`warp/native/mesh.h:1880` 局部归一化 + `render.py:731-732` mat@n+normalize）、`f728b2d7` 删第一次、P10 `b0e372be` 恢复」表述一致，且均声明「原记录文字保留不改」；A3 Addendum 追加对应 `mesh.h`/`render.py` 引用；A6 两处指针指向 A1/A4/A5 正确。
- **低 L2（口径）**：A2 的 `single-normalize/receipt.json` **不是** append-only——从字节 5,245 起整段重排（紧凑 inline 对象改为多行）+ 末尾追加 `addenda[]`（6 键）。我按 JSON 展平逐键比对：旧 83 键与新非 addenda 键**完全相等**（语义未变），但**旧字节未保全**。建议措辞区分「README/logs 纯追加」与「receipt 结构化改写 + addenda」。

## 二、B 段：dump 字段 errata 方案 A（✅ 实现正确，附 1 条低项）

- 改动面：`84ed67d2^..84ed67d2` 的 `tests/` 只有 `diag.rs`（+20/−2）与 `diag_ext.rs`（+51）；**无 `.cu`、无 `src/` 改动** ⇒ 内核身份不变（P10 基线 source `6b1ade37…`/PTX `48f8e7f8…` 未被本次提交触碰，`git diff 84ed67d2 -- '*.cu'` 空）。
- 语义（逐行核）：`segment_attribution` 中原 `let (geom_id, geom_type) = (seg[0], seg[1]);` 改为 `seg_objtype = seg[1]`；`geom_type` 改为模型 `geom_type[geom_id]`（`geom_id<0` 或越界 ⇒ `null`）；无模型 ⇒ `geom_type=null`、`seg_objtype` 保留、`mat_id=null`；越界 ⇒ `mat_id=-1`；其余字段与键序不变；`SCHEMA: u32 = 2` 未动。
- B1 源审计可复现：`tests/support/render_bvh/light/model-315.json` 实读 `geom_type=[0,6,2,7,3,5,4]`（geom 3 ⇒ **7 MESH**）、`geom_matid=[2,0,1,0,2,1,2]`（geom 3 ⇒ mat 0），与 `artifacts/b1-source-audit.txt` 逐项一致；diag7/diag8 `seg.product=[3,5]` ⇒ 旧值 5 是 `seg[1]`（`mjOBJ_GEOM`）误标，errata 成立（不撤回）。
- 静态单测（我独立跑，非采信日志）：`cargo test --locked --features cuda-probe --test render_per_world_light -- light_diag` ⇒ **5 passed / 0 failed**（`schema_constant_and_document_top_level_agree`、`rgb_mismatch_entries_always_carry_diagnostics_key`、3 个 `segment_attribution_*`），与 `logs/static-tests.log` 一致；`diag_ext.rs` 的 3 个用例确实断言 `geom 3⇒geom_type=7/seg_objtype=5/schema==2`、无模型 null、越界 null+mat −1。
- **低 L1**：`artifacts/b-field-change.diff` **名不副实**——其两个 hunk 只是「3 个测试从 `diag.rs` 搬到 `diag_ext.rs`」（`diag.rs` −49 / `diag_ext.rs` +51），**不含 B 的字段语义改动**；且其 `diag.rs`「old」blob index `25affdc8` 是编辑中间态（最终 `84ed67d2^` 为 `02533c7d`）。字段语义真身在提交 diff 本身。
- **低 L3（行数口径）**：卡/README 记 `diag.rs 466 / diag_ext.rs 231` 行，来自 PowerShell 默认编码 `Get-Content` 计数（把含 CJK 行尾的 LF 吞并，少计 18/6 行）；物理行数（LF 计数/`ReadAllLines`）为 **484 / 237**，两者均 < 500 棘轮 ⇒ 判定不受影响，但数字应写物理行数。

## 三、C 段：QA #7 三条记录面（✅ 落点成立，一般项 1 条）

- C1：`target/render-bvh/perworld-l-v2-gl2/diag7/window/gpu-snapshot.txt`（672 B）存在，含 pre/post/派工参考/时点/notes 五项 + 模板 `artifacts/window-snapshot-template.txt`（617 B）。
- C2：`diag7/body-env.txt` 已重写为可读 UTF-8/LF（无 TAB/退格污染），`where nvrtc64_120_0.dll` 行可读、PATH/sha/bytes 齐全、明确「outline env pair: NOT set（实体只需 PATH）」；与 diag7 日志事实一致。
- C3：GPU 数字以 `nvidia-smi` 实测定为权威、派工数字标「reference only」；pre 值与 `pre-state.txt` 逐字一致（`gpu_pre=2 %, 3924 MiB, 12138 MiB, 26`）。
- **一般 G1（C1 未真正补齐，且引用不实）**：`gpu-snapshot.txt` 写 `gpu_post_actual=2 %, 3,919 MiB used, 12,143 MiB free, 26 C`，但该轮**唯一跑后记录** `run-summary.txt` 仍是未展开占位 `gpu_after=$gpu`，窗口内不存在跑后采样文件；`gpu-snapshot.txt` 的 `timestamps_actual=` 行称「post recorded in run-summary.txt (same diagnostic round)」——**与事实矛盾**（该文件无此值）；文件 mtime = 08:06:18（errata 时段），说明 post 值为**事后查询/回填**。应标注「errata 时段复测，非 run 结束即时采样」，并改掉错误交叉引用。不影响任何数值/判定，但 C1 的宣称（补跑后快照）应降级为「事后补测」。

## 四、E 段：attempt 4 errata §11 + 可读副本（✅）

- `attempt4/README.md` §11 追加内容逐条覆盖 E1–E5（证据卫生/`build=0` 仅最终 60/60 段/基线时间戳与 body 启动推定/`signed_stricter` 严格口径 10,230 vs `floor_effective` 9,956/guards CRLF 与缺 `body_env`、两跑 sha backlog），并附证据 log 口径，均声明不改变 A/B/C 与窗口有效性结论。
- 原文件字节未改：`window/freeze-stdout.log` 3,740 B/`4d983c68…`、`window/monitor-stdout.txt` **340 B/`c22ea746…`**（与声称的恢复 sha 一致），`git diff 5f1d436d..84ed67d2 -- …/attempt4/window/` 仅新增两份 readable。
- 可读副本**忠实性我独立验证**：把原始损坏字节按 UTF-16LE 解码并剔除 NUL/U+FFFD 后，内容与 `freeze-stdout.readable.txt`（24 条 CHECK/WARN + `GUARDS_FROZEN … aggregate_sha256=08180107… guards_sha256=94478b3d… bytes=43729` + `SELFCHECK PASS 18/18`）与 `monitor-stdout.readable.txt`（`samples=78 longest_clean=60 required=60 ready=True start=07:45:21.970 end=07:51:54.302 util_min=2 util_max=6 free_min=12136`）**逐行一致** ⇒ 不是事后编造；副本头三行已注明「派生、原字节不改」。
- 与 attempt4 归档交叉一致：旧 suite `8a4ddc50…`、guards `94478b3d…`/43,729 B、aggregate `08180107…` 与我此前独立复算一致。

## 五、证据、边界、门禁

- **manifest 独立重算（8 个受影响目录，逐行 sha+bytes、并查 extra/missing）**：errata-batch **8/8**、single-normalize 13/13、single-normalize-check 2/2、criterion-a-rca 5/5、chain-pin 17/17、chain-pin-check 5/5、diag7-check 6/6、attempt4 **53/53**（=51+2 readable）；全部 bad=0 / missing=0 / extra=0，且 manifest 不自列。
- 声明哈希复核：批次 README `dd3836e8…`/5,719、receipt `3ee94ff3…`/4,650、manifest `1274effe…`/968；attempt4 README `4725b7de…`/11,374、manifest `b9932264…`/5,985 —— 逐位一致。
- **边界**：`git diff 5f1d436d..84ed67d2`：`src/`/`fixtures/`/`scripts/` **0**；`tests/` 仅上述 2 文件；`*.cu` **0**；guards JSON `94478b3d…`/43,729 B 未变；无 S-1/S-2/S-3 签名材料改动；`src/` 判据/参考/容差零改动；旧卡 `…-errata-batch` 由 `82f5dd76` abandon（status=abandoned），r2 卡 verified。
- **编码**：本提交新增/改动的 docs 与 tests 文件全部 UTF-8、无 BOM、无 CR、无 TAB、无 NUL（含两份 readable 与 `b-field-change.diff`）。
- **门禁**（读 `.normify/runs/…-r2/` 记录）：fmt/clippy/clippy-cuda exit 0；unit **1,394 passed / 0 failed / 402 ignored**、exit 0（1,391 + 3 新单测，数量自洽）。
- **低 L4**：证据 README 第 3 行「任务：`2026-11-11-render-perworld-l-v2-gl2-errata-batch`」与 `receipt.json task_id` 指向**被 abandon 的旧卡**，而实际提交卡为 `…-r2`；卡 note 有生命周期说明，但证据面未标注 successor 关系，建议补一句。

## 六、过程事件（✅ 记录齐全，观察 1 条）

- 事件①锁纪律：`d82d5905`(08:03:30) 与 `f9bc436e`(08:03:32) 两条治理提交仅改 `.normify` 卡 JSON（+75 / +3−4 行），时间落在我 `6446c509`(08:03:13) 与 `5b586e77`(08:03:48) 之间 ⇒ 与「QA 持锁期间完成」陈述一致；已立即停手、等锁 FREE 后重新 acquire。新口径「acquire 显式校验 `REPO_WRITE_LOCK_ACQUIRED` 且 owner==自己」已写入卡 note/receipt/README §F。
- 事件②归档修复：`monitor-stdout.txt` 现 340 B/`c22ea746…` 且 git 无 M ⇒ 从 `5f1d436d` blob 逐字节恢复成立，派生副本改名 `monitor-stdout.readable.txt`，净损失 0。
- **观察 O1**：仓内 `repo-write-lock.ps1` 仍无历史审计日志，事件只能依披露 + 提交时间线交叉验证（本次两者一致）；建议后续把锁 acquire/release 事件落日志。

## 七、口径

不宣称修复生效/上游等价/灯字段端到端验收；不改判据/阈值/参考/容差/相机/recipe；零 GPU、未重跑/未重冻；W0 与 G-L2 结论以 Chief 材料为准。本批复核为记录面/边界/实现正确性，**未判定任何渲染数值回归**。
