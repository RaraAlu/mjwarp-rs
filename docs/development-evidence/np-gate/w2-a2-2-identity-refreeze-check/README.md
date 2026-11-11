# QA 复核 W2/A2-2 内核身份重采 + 10 exe 重冻 + fixture 13→15 + NP02 6 行（`f9d76798`）

- 复核对象：卡 `2026-11-12-w2-a2-2-identity-refreeze`；提交链 **`670d322b`（登记 in_progress）→ `b23d5dae`（第一节 + fixtures）→ `0e5343b8`（身份/重冻/NP02）→ `f9d76798`（卡收口 verified，HEAD）**。
- 证据目录：`docs/development-evidence/g10-p5-mesh-cap32-identity/`（manifest 11/11）、
  `docs/development-evidence/np02-evidence/a2-collision-2026-11/`（manifest 20/20）、
  `docs/development-evidence/g10-p5-mesh-cap32/`（manifest 11/11）、
  `docs/development-evidence/g10-p5-mesh-cap20/`（manifest 11/11）、`docs/development-evidence/np02-evidence/`。
- 性质：只读复核（零 GPU、零窗）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，0 严重、0 一般 + 2 低 + 3 观察）**。

## 1. 提交链与改动边界

- 四个提交的路径清单我用 `git show --name-status` 逐提交核过：`670d322b` 仅卡 JSON；`b23d5dae` 8 路径
  （cap20 addenda + manifest、cap32 README + addenda + manifest、2 个 33-gon fixture、`tests/support/branch_closure_case.rs`）；
  `0e5343b8` 37 路径（identity 目录 12 + NP02 新波次 20 + `inventory.json` + `receipt.json` + 1 交集）；
  `f9d76798` 卡 JSON + 4 个 gate run。
- **边界 ✅**：`670d322b^..f9d76798` 共 48 路径，按目录分组仅 `.normify/`（5）、`docs/development-evidence`（40）、
  `fixtures`（2）、`tests/support`（1）；`git diff --name-status ... -- src Cargo.toml Cargo.lock abi scripts build.rs`
  **输出为空**（`src/`、Cargo、ABI、scripts 零改动）；`tests/` 仅 `FIXTURE_SHA256` 13→15 与新增两条。
- **无外部 agent 文件 ✅**：四个提交均无 `code-size-quick-screen` 相关路径；工作区仅该外部项 untracked，我未碰。

## 2. 第一节：A2-1 证据低 L1 修正（append-only）

- **README 改动最小 ✅**：我按字节 diff 核 `2ee0540d` → `b23d5dae`，仅 §6 一行变化：
  「与上游一致（over-32）、（旧措辞）」→「与上游一致（over-32）、不作位级等价声明、不提供位级等价保证。」，
  非声明语义更严；**数字/结论/测试条目零改动**。新 sha `502c4f35…`/6,928 B。
- **addenda ✅**：`addenda.md` `1b92cac7…`/3,098 B；L1 节把改前原文放进
  `<!-- policy-scan:quote:start -->` / `end` 闭标记块并写明「已作废」，另记 O1（7 组 9 条口径）、O2（fan-cone-24 用例改写）。
- **manifest 10→11 ✅**：`74f346ee…`/1,378 B，**11 条逐条重算 0 错**，不列 manifest 自身。
- **历史字节不变 ✅**：`receipt.json`（`7b656e30…`）、`logs/gate.txt`、2 个定向日志、`line-counts.txt`、
  4 个 materials 与 `2ee0540d` 的 blob **逐个相同**。
- **我独立复扫 ✅**：`policy_scan.py` 对 cap32 目录 **24 个文件 0 命中 PASS**；`--self-test` **34/34 PASS**。

## 3. A1 身份 supersession

- **A1 正文逐字节未改 ✅**：`g10-p5-mesh-cap20/README.md` 在 A2-2 前后 blob 均为 `52542006…`；
  其 manifest 从 10 条追加为 **11 条（只多 1 行）**，新增行指向 `addenda-a2-identity.md`（`f5727d99…`/1,775 B）。
- **接管链清楚 ✅**：addenda 表格列 A1 旧值（`12d50d68…`/254,263 B、3,883,769 B）→ A2 现值
  （`fb03c5b1…`/254,434 B、`78da9dea…`/3,883,643 B、6 入口），明确旧 frozen exe/guards 不复用；
  A1 manifest **11 条逐条重算 0 错**。

## 4. 内核身份（源 / PTX）

- **逐字节对盘 ✅**：新波次 6 个 `.cu` 实体均为 `fb03c5b1…`/**254,434 B**，6 个 `.ptx` 实体均为
  `78da9dea…`/**3,883,643 B**；`capture-report.json` 6/6 `gateOk=true`、`f32-physics`。
- **我独立重跑探针 ✅**：`cargo test --locked --features cuda-probe --lib nvrtc_smoke -- --ignored --nocapture`
  → `nvrtc_smoke_ok ptx_bytes=3883643 entries=6 source_bytes=254434 source_sha256=fb03c5b1…`，
  与证据逐字一致（CPU-only NVRTC，无设备上下文）。

## 5. 10 exe 重冻

- **10/10 对盘 ✅**：`target/a2-freeze/` 10 个 exe 的 **bytes 与 sha256 全部等于** `refreeze-10-exe.json` 的
  `rows`，且 `sha256 == stash_sha256`、`copy_match=true`（我逐项复算）。
- **源码树 ✅**：清单登记的 **29 个 `.cu` 逐文件 sha256 与磁盘全等**（总数 29 与「29 个 `.cu`」声明一致）；
  旧 `target/s4-freeze/`、`target/m3-freeze/` 磁盘已不存在，「作废、不复用」与事实一致。
- 观察：`source_tree_digest.combined`（`266bbf94…`）的拼接口径未写进证据，见 §10 O1。

## 6. fixture 13→15

- **新增实体 ✅**：`fan-cone-33.obj` `58908058…`/**1,425 B**、`prism-33.obj` `bb1b05c2…`/**3,382 B**，
  与 `receipt.json`/`FIXTURE_SHA256` 三方一致；内容为 33-gon 扇/棱柱（over-32 边界）。
- **守卫测试我重跑 ✅**：`cargo test --locked --features cuda-probe --test g10_p5_branch_closure_cuda`
  → **3 passed / 0 failed / 2 ignored**（与 `logs/fixture-guard-test.log` 一致）。
- **不进 pass/fail ✅**：`LIMIT_CASES` 仍为 **2**（U7 cone-box 24、U8 prism-box 20），33-gon 两 fixture 仅出现在
  `FIXTURE_SHA256` 守卫表；over-32 只由宿主容量门 CPU 覆盖（A2-3 计划同口径）。

## 7. NP02 6 行重采

- **新波次 ✅**：6 入口 ×（cu/ptx/json）= 18 实体 + README + capture-report + manifest，manifest **20/20 逐条重算 0 错**。
- **补采索引 ✅**：`receipt.json` receipts 列表 **27 → 33（+6）**；旧 27 条前缀**逐字段不变**、无重复
  `(entry,ptxSha)`；`cmd_index` 以 `(entry,ptxSha256)` 去重（代码读过），重跑不重复追加。
  **注意**：证据两处写「25 → 33」，与实际及 mapping 自记的 **27 → 33** 不符，见 §10 L2。
- **inventory 身份换指 ✅**：6 行只改 `artifact.ptx_sha256[0]` 与 `artifact.evidence_paths[0]`（我按扁平化逐字段比对），
  **其余 96 行零改动**、行数 102 不变、文件 **+6 B**；`counts.receipts_total=715`、`receipt_ptx_entities_verified=711`
  等计数全部不变；旧波次目录 `new-kernels-2026-11/` 一个提交内零改动。
- **全量重算未执行且如实记录 ✅**：README §4、receipt `full_inventory_regeneration`、mapping `note` 三处一致说明
  「本片不做 inventory 全量重算，归 NP5-03 面」；**全文无「inventory 全量一致」类正面声明**。

## 8. manifest / 门禁 / 卡

- manifest 全对 ✅：identity **11/11**、NP02 新波次 **20/20**、cap32 **11/11**、cap20 **11/11**，均逐条重算 0 错、不自列。
- 门禁 ✅：`.normify/runs/2026-11-12-w2-a2-2-identity-refreeze/` 四个 run（fmt/clippy/clippy-cuda/unit）；
  `unit.json` = **1409 passed / 0 failed / 402 ignored**、exit 0，与基线一致。
- 卡 ✅：`status=verified`、`closed_at` 在位；13 项检查（4 gate + `unchanged src/` + 8 条 json 指针）——
  我把 8 条 json 指针逐条对照实际文件值（task_id/verdict/两个 sha/entries=6/exe_count=10/guard_count=15/policy_scan_hits=0），
  **全部满足**。

## 9. A2-3 窗口计划核查（只计划）

- 判据矩阵覆盖 M2/M3/S4 branch 9/U7/U8/A1 回归/A2 边界/over-32/NP02/stage2；10 个 exe 逐个列名与覆盖；
  **单窗互斥**、fresh freeze（固定 toolchain + MuJoCo root + `--all-features --no-run` + 复制后重哈希）、
  **零 Rust 编译**（窗内只跑冻结 exe，发现 cargo/rustc/cl/nvcc 即 abort exit 9）、**monitor 60/60**（100 ms 采样）、
  **三点基线**+pre/post 复核、**floor 取严**（`free ≥ 10240 MiB` 与 `baseline_free − 3072` 取更严）、
  **fail-stop**（不重试不放宽）、sha 守卫（10 exe + 15 fixture + 2 参考）、参考只绑定不重采、attempt ≤ 2。
- 开窗前四前提清单 + 群里留痕 + 待裁决点（over-32 设备 case 是否入窗、U7 判红规则、NP02 索引/清单归属）在位；
  全文为「计划/期望/记录」措辞，**无通过声明**。

## 10. 发现

### 低
- **L1（已由 Backend 自报、Chief 定处置）证据日志快照与 receipt 不一致**：
  `g10-p5-mesh-cap32-identity/logs/policy-scan-a2-1-evidence.log` 提交内容是**中间失败快照**
  （`hits=2 RESULT: FAIL`，命中项为修正过程中的 addenda/manifest 旧文本），而 A2-2 receipt
  `a2_1_evidence_errata.policy_scan_hits=0` 与最终状态一致。**我独立复扫确认最终态**：cap32 目录 24 文件 **0 命中**、
  self-test **34/34 PASS**、README §6 已为否定式，故 **L1 修正本身成立、A2-2 结论不受影响**；矛盾仅在日志这一项。
  处置（Chief 已定）：并入 A2-3 登记卡第一节 append-only（补最终 PASS 日志 + addenda 说明「中间失败快照被最终扫描取代、
  历史字节未改」+ manifest 11→12）。
- **L2 索引基数写错（新发现）**：`g10-p5-mesh-cap32-identity/README.md` §4 与
  `np02-evidence/a2-collision-2026-11/README.md` §2 写「补采索引追加 6 条（25 → 33）」，但实际为
  **27 → 33**（`25 + 6 = 31` 算术亦不自洽）；同一片的 `np02-identity-mapping.json` 反而记录
  `entries_before: 27 / added: 6 / total_after: 33`（正确）。历史链：13 → 25（`25bd3479`）→ 27（`d46f435f`，NP5-03 +2）
  → 33（`0e5343b8`）。建议与 L1 同批 append-only 勘误（只加勘误说明，不改历史字节）。

### 观察
- **O1 组合摘要口径未记载**：`refreeze-10-exe.json` 的 `source_tree_digest.combined`（`266bbf94…`）我按
  「sha 列表」「路径+sha」「内容拼接」等常见口径均无法复现；但 **29/29 逐文件 sha 全部对盘**，
  身份判定可用，建议后续写明拼接算法。
- **O2 计划稿头部与落仓事实不符**：`a2-3-window-plan.md` 开头写「写入 target 工作材料（不入仓）」，
  实际已随 `0e5343b8` 入仓；属陈旧措辞，不影响内容（计划本身无通过声明）。
- **O3 注释未同步**：`tests/support/branch_closure_case.rs` L128 doc 注释仍写「13 个 fixture」，
  而数组已是 `[(&str,&str); 15]`；建议后续提交顺手改为 15。

## 11. 非声明

本复核只对 A2-2 的身份重采/重冻/fixture/NP02 换指/证据卫生负责；**不宣称** U7/U8 通过（设备数值判定属 A2-3 窗）、
mesh 容量缺口已清、G10@P5 整族通过、NP01/NP02 门禁通过、与上游等价（over-32）；**不作位级等价声明、不提供位级等价保证**；
零 GPU、零窗；不改判据/参考/容差/夹具/源码。
