# A2-2 证据勘误（A2-3 第一节；append-only，历史字节不改）

- 任务：`2026-11-12-w2-a2-3-m2-m3-s4-replay-window` 第一节（Chief 2026-11-12 指示；QA A2-2 复核 PASS 的 2 低 + 3 观察）。
- 被修正证据：`docs/development-evidence/g10-p5-mesh-cap32-identity/`（A2-2，提交 `f9d76798`）。
- 规则：**不改历史字节**；本 addenda 与日志末尾 append 为生效说明；identity manifest 11→**12**（本文件）。
- QA 结论：A2-2 **PASS**（0 阻断 / 0 严重 / 0 一般 + 2 低 + 3 观察）。

## L1（QA 低）：`logs/policy-scan-a2-1-evidence.log` 中间失败快照

- 已提交内容为**中间失败快照**：`policy_scan: files=12 hits=2 RESULT: FAIL`（当时 cap32/addenda 原文引用与 manifest 头注尚未修改）；
  两条命中均是修正过程态，不是 L1 未修。
- 处置：在该日志**末尾 append**「FINAL RESCAN」段（最终 12 文件 0 命中 PASS + `--self-test 34/34 PASS`），
  不删除/不改写原快照字节；本 addenda 记录取代关系。
- 最终态复核（QA 独立 + 本片复现）：cap32 证据 12 文件 **0 命中 PASS**、self-test **34/34 PASS**、README §6 为邻近否定式、
  addenda 原文在闭标记引用块内。A2-1 L1 修正成立，A2-2 结论不受影响。

## L2（QA 低）：索引计数「25 → 33」应为「27 → 33」

- 受影响表述（只加说明，不改原文）：
  - `docs/development-evidence/g10-p5-mesh-cap32-identity/README.md` §4「补采索引追加 6 条（25 → 33）」；
  - `docs/development-evidence/np02-evidence/a2-collision-2026-11/README.md` §2「索引总计 25 → **33**」。
- 事实：实际 **27 → 33**（27 + 6 = 33）。历史链：13 → 25（提交 `25bd3479`）→ 27（`d46f435f`，NP5-03 +2）→ 33（`0e5343b8`，A2 本片 +6）。
  `np02-identity-mapping.json` 的 `entries_before: 27 / added: 6 / total_after: 33` **正确**；`np02-evidence/receipt.json` 现 33 条（复核实测）。
- 结论：仅读数笔误，证据链与结论不变；以本 addenda 与 mapping 为准。

## O1（QA 观察）：`source_tree_digest.combined` 拼接口径未记载

- 口径（本片复现验证）：对 `src/physics/collision/cuda/*.cu` **按文件名排序**（29 个）逐文件做 LF 归一化（`\r\n`→`\n`），
  以 `combined = sha256( update(name_utf8 + content) ... )` 依次累加，得到：
  `266bbf94fd92cce3202159df196d84824dacbaa6d22f4b848eddd83ac57fcb44`
  （与 `refreeze-10-exe.json::source_tree_digest.combined` 一致）。
- 可复算脚本片段（不新增文件，记录于此）：
  ```python
  from pathlib import Path; import hashlib
  h = hashlib.sha256()
  for p in sorted(Path('src/physics/collision/cuda').glob('*.cu')):
      b = p.read_bytes().replace(b'\r\n', b'\n')
      h.update(p.name.encode() + b)
  print(h.hexdigest())  # 266bbf94…cb44
  ```
- 逐文件 sha 已在 `refreeze-10-exe.json::source_tree_digest.files`（29/29，QA 已独立核对）；本口径仅为合并值复现。

## O2（QA 观察）：`a2-3-window-plan.md` 头部「写入 target 工作材料（不入仓）」与实际不符

- 事实：该文件已随 A2-2 证据入仓（`docs/development-evidence/g10-p5-mesh-cap32-identity/a2-3-window-plan.md`）。
- 更正：头部「写入 target 工作材料（不入仓）」**作废**；其内容现为仓内 A2-3 计划证据，窗口材料另见就绪包
  `target/render-bvh/gl2-ptx-diff/w2-a2-precheck/a2-3-window-readiness.md`（target，不入仓）。历史字节不改。

## O3（QA 观察）：`tests/support/branch_closure_case.rs:130` 注释「13 个 fixture」与数组 15 不符

- 记录：注释为历史笔误（A2-2 fixture 13→15 后未同步）。
- 处置：**本片不修**。原因：现在改 `tests/` 会使冻结 exe（A2-2 构建）与 HEAD 源码不同源，破坏窗口一致性；
  待 **A2-3 窗口完成后**再改该注释（测试注释改动随窗后证据片重跑四门禁）。

## 三条裁决（Chief 2026-11-12；本卡与就绪包 §8–§10 为准）

1. **over-32 设备 case 不进 A2-3 窗口**：host 确定性容量错误已由 A2-1 边界单测覆盖；device over-32 为 record-only、不进 pass/fail；
   不补 XML/MeshData/case，保持窗口最小；若 QA 要求 record-only 证据 → 另开 0 窗片、不占窗。
2. **NP02 inventory 全量重算不做**：保持 identity-only swap；scanner 基线（100 路径 / 33 缺口）与仓内 patched 清单（102 / 32）
   的差异如实挂 **NP5-03 面**；**不得宣称 inventory 全量一致**。
3. **新 exe 身份已变**：后续任何重建 Rust exe 的窗口（含渲染）都要重冻；既有窗口证据历史保留、不混用新身份。

## 窗口执行说明（摘要；全文见就绪包）

- 10 个 A2 冻结 exe（逐项 sha/bytes 见就绪包 §1；`target/a2-freeze/`，10/10 copy_match）顺序执行；
  stage2 用 `window_product_equivalence_dump`（record-C）。
- 判据：M2/M3/S4 原判据；U7/U8 改判 pass/fail（U7 强制 dist/pos；U8 设备 count 1→4 + pos 四角）；
  A1 CPU 回归、A2 边界 CPU、NP02 6 行（新 PTX `78da9dea…`）同窗复核。
- 纪律：fresh freeze 复用（开窗前 `src/` 自冻结零 diff + 10 exe 未重建，否则整包作废）；monitor 60/60；三点基线；
  floor 取严；sha 守卫 10 exe + 15 fixture + 2 参考；**窗内零 Rust 编译**；fail-stop 8 条，不重试不放宽。
- 窗口由 Chief 开：四前提勾表 → monitor 60/60 → 三点基线 → 群留痕 → go；Backend 不自行开窗。

## 边界

- 本 addenda 只记录勘误/裁决/窗口说明，不改 A2-2 任何历史字节（日志仅末尾 append）；identity manifest 11→12。
- 不宣称 U7/U8 通过（窗口前）、mesh 容量缺口已清、与上游等价（over-32）、G10@P5 整族通过、NP01/NP02 门禁通过。
