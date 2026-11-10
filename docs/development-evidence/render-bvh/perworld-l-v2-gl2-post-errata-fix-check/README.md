# QA 复核：G-L2 后置 errata 批复核修正卡（`3836da0d` + `a3dc5e7b`）

- 任务：`2026-11-11-render-perworld-l-v2-gl2-post-errata-fix-check`（owner QA；零 GPU、只读复核）。
- 对象：主提交 **`3836da0d`**（19 files，纯文档/证据；卡 `2026-11-11-render-perworld-l-v2-gl2-post-errata-fix` verified）+ 过程事件 addenda **`a3dc5e7b`**（1 file，卡 JSON）。复核基线 HEAD `a3dc5e7b`。
- **结论：PASS（0 阻断，3 条低残留 + 1 条口径注记）。** S1 四处记录已全部改为实测事实、历史字节未动、证据保全口径成立；低 3 条与 dump-ext 残留 2 条均已落地并被我的独立测量复现；四目录 manifest 全对、14 个改动文档编码干净。3 条低残留均为记录面文字（不阻断）：README 表格行 4 未标「见 §5」、receipt 称提交信息「已记入卡 addenda」但 addenda 只有锁事件、process_events 未注明「仓内无锁审计日志」。

## 1. ① S1 更正属实 ✅

- **历史字节未动**：`perworld-l-v2-ptx-order-pins/logs/src.diff` 仍 **6,772 B / sha `12be6af5…` / 魔法 `FF FE`（UTF-16LE+BOM）**，worktree 与 HEAD blob 逐字节一致；UTF-8 解码失败、UTF-16LE 解码 60 个 CRLF —— 与我的 S1 复核一致。
- **四处记录已更正**：
  1. 批 README §3（L41）改为实测事实并标「S1 更正，见 §5」；
  2. §5（L52-54）完整更正 + 逐条列出「四处不实记录」与「不改历史字节」裁定；
  3. receipt `fix_card_1_errata.src_diff_lf_verification` 重写为实测（UTF-16LE+BOM / UTF-8 decode fails / 61 lines / 60 CRLF / 原始字节计数误导）；
  4. `artifacts/ptx-order-pins-src-diff-eol.txt` 重写（2,031 B：BOM/首尾字节/`-text`/解码/`raw_byte_scan` 标注 MISLEADING/复现命令）。
  提交信息（不可改）在 README §5 项 ④ 与 receipt `review_corrections.s1_src_diff_encoding` 记录（见残留 R2 的 addenda 措辞问题）。
- **解释成立**：原「CRLF 0 / LF 62」是原始字节计数（UTF-16LE 的 CRLF 编码为 `0D 00 0A 00`、LF 为 `0A 00`）导致的误判；我复现 `b"\r\n"=0 / b"\n"=62`，解码 CRLF=60 ✅。

## 2. ② 证据保全口径 ✅

- `.gitattributes` 既有规则 `docs/development-evidence/** -text -diff` 覆盖该路径（我实测规则存在）；两提交**未改 `.gitattributes`**；`git ls-files --eol` = `i/-text w/-text attr/-text`。
- **未批量转换**：两提交只改 14 个文档 + 4 个 run + 1 卡；`src.diff` 字节不变；其余历史 UTF-16 文件未被触碰。
- **计数一致**：我按 tracked 文件独立扫描 UTF-16 BOM = **169**（`docs/development-evidence` 内）/ **217**（全仓）——与声明逐数一致。
- **新日志口径落地**：receipt `evidence_log_policy` + README §5 观察 ② 写明「证据 log 一律 UTF-8/LF；PowerShell 用 `[IO.File]::WriteAllText(... UTF8Encoding($false))` 或 `Out-File -Encoding utf8`，禁止默认 `>` 重定向」✅

## 3. ③ 低 3 条 ✅（我独立复现全部数字）

- **manifest 行序**：更正为「行内容不变、仅按 ASCII 路径重排」并给出共同路径行位移实测；我独立复算（`bb29e351`→`dde275c1` 共同路径 index 变化数）：G-L2 **12**、修复卡-1 **11**、诊断轮 **6** —— 与声明逐数一致；四个 manifest 现均 ASCII 有序。
- **行号**：`outline.rs` `enabled()` 实测 **:51**、`diff_depth_with_outline()` 实测 **:363**；`run-window.ps1` post-window 校验 **:357-369**（入口 `Verify-OutlineLists 'post-window'` **:358**）、pre-body **:283** —— 声明全部正确，且 README/`body-env-contract-errata.md`/artifact 均已更正。
- **轮 #4 证据路径**：`target/render-bvh/perworld-l-v2-gl2/diag4/pixel-dump.json`（**44,977 B / `48597d0e…`**）与 `diag4/run-summary.txt`（**3,931 B / `df653b5b…`**）实测存在且哈希/字节逐位匹配 ✅（dump 留 `target/` 不入库，符合口径）。

## 4. ④ dump-ext 残留 2 条 ✅

- README §2「JSON 位置」已改为「**键始终存在**：未启用 env（renderer=None）为 null、启用但未命中/越界也为 null、不会缺失（`attach_diagnostics` 契约）」；receipt `dump_schema.json_location` 同步更正（英文版同义），原「absent when env unset」已不存在。
- 配套卡计数更正为 **+84/−8**（numstat 2/2 + 3/3 + 1/1 + 2/2 + 76/0）；历史提交信息不改写，记录于 errata 批 receipt/README 与卡 addenda。

## 5. ⑤ 过程事件 ✅（附 1 条记录建议）

- README §6 与 receipt `process_events` 各记一份：时间（2026-11-11 06:47-06:57）、提交号（`dde275c1`/`26d3cada`）、无锁范围（`normify fix`/`gate`/`done` + 卡状态 JSON/`.normify/runs`/docs/提交）、当时状态（无他人持锁/申请、无冲突、树 clean、无数据损坏）、违规性质、补救=规则固化为硬约束；两者与卡 addenda 互引 ✅。
- **仓内无锁审计日志**：`scripts/validate/repo-write-lock.ps1` 只保存当前状态、无历史日志 ⇒ 该事件只能依据披露与时间线；建议在 process_events/§6 加一句「无审计日志可回溯」（残留 R3）。

## 6. ⑥ 门禁 / 证据

- 本卡 `normify gate` 复跑：fmt / clippy / clippy-cuda / unit 全过，unit **1,391 passed / 0 failed / 402 ignored**。
- 四目录 manifest 独立重算：本批 **8**、G-L2 **66**、修复卡-1 **12**、诊断轮 **12**，bad=[] / missing=[] / extra=[]，均 ASCII 有序。
- 14 个改动文档全部 UTF-8、无 BOM、无 CRLF ✅；`src.diff` sha 未变 ✅；`src/`、`tests/`、`fixtures/`、`scripts/` 两提交零 diff ✅。

## 7. ⑦ 口径 ✅

receipt `non_claims` 与 README：不改 G-L2 判定（仍为未通过 / 判据 A FAIL）、不宣称任何修复生效、不改判据/阈值/参考/容差/相机/recipe；dump 代码字段改名与 guards `body_env` 仍是 backlog。

## 8. 低残留（3 条，记录面，不阻断）

1. **R1**：README 开头「本批四项」表格行 4（L12）仍写「`logs/src.diff` LF 复核（已 LF、字节未变）」，未标「见 §5 更正」——与同文件 §3/§5 自相矛盾。建议改为「原记录见 §5 更正」或直接改行。
2. **R2**：receipt `review_corrections.s1_src_diff_encoding` 称不可改的提交信息「recorded here and in card addenda」，但该卡只有 **1 条 addenda（锁事件）**，不含 S1；README §5 项 ④ 的措辞（「已在本节与 receipt 记录」）才是准确的。建议改 receipt 措辞或补一条 addenda。
3. **R3**：process_events/§6 未注明「仓内无锁审计日志、事件依据披露」；建议补一句（不影响事件事实成立）。
4. 口径注记：文档写「按 UTF-16LE 解码为 61 行」用的是 `split('\n')` 段数（含尾部空段），`splitlines()` 为 60 行；无实质影响，可统一口径。

## 9. 非声明

本复核只核记录/证据/口径，不改变 G-L2 判定（仍为未通过 / 判据 A FAIL），不宣称任何修复生效，未重跑/重冻/采集，零 GPU。
