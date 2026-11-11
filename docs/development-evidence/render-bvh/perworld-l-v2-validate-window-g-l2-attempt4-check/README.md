# QA 复核：G-L2 验证窗 attempt 4（canonical）归档证据

- 任务：`2026-11-11-render-perworld-l-v2-validate-window-g-l2-attempt4-check`（owner QA；零 GPU、只读复核）。
- 对象：归档提交 **`5f1d436d`**（58 files：attempt4 证据 52 + guards JSON + 4 run + 卡）、治理 `fd084dd1`/`6ed4065d`；卡 `2026-11-11-render-perworld-l-v2-validate-window-g-l2-attempt4`。复核 HEAD `5f1d436d`。
- **结论：复核通过（0 阻断，1 条一般 + 3 条低 + 2 条观察）。** 窗口有效性与结论（12/12 exit 0、A/B/C 全过、冻结面 107、12/12 live==guard==stash、aggregate 独立复算一致、monitor 最终 60/60、floor 与 env 成对、canary、manifest 51/51、diff 面干净、无 INVALID_ROUND）全部独立核实成立。**一般项是证据卫生缺陷**（2 份辅助 stdout 日志二进制损坏：含 NUL、非 UTF-8；摘要文件与独立复算承载全部结论）——**不构成窗口有效性阻断**，建议归档补一份可读副本/errata 收口；低项为记录措辞/时间戳细节。**不宣称对外「上游等价/渲染数值等价」**（见非声明）。

## 1. 冻结面（guards / aggregate / 12 exe / 输入）✅

- guards JSON 实测 **`94478b3d…` / 43,729 B**（repo 与证据副本逐字节一致）、`attempt=4`、`criterion_version=render-sdf-criterion-v2.2`、`bound_ulp=128`、`counts=12 exe + 93 inputs + 2 lists = **107**`。
- **aggregate 独立复算**：按冻结器自身算法（`Sort-Object { $_.path }` → 12+93+2 条 sha256 hex 串无分隔拼接 → SHA256(ASCII)）复算得 **`08180107…`**，与 guards 记录一致；同一算法复算旧 suite（96 条）得 **`8a4ddc50…`**，与旧 suite 记录一致 ⇒ 算法与自洽性成立。（注意：**不能**用 Python 序数排序复现，PowerShell `Sort-Object` 是文化/大小写不敏感排序；这是复现要点，非遗漏。）
- **12/12 `live==guard==stash`**：我逐条对 live exe（`target/debug/deps/*.exe`）、stash（`target/render-bvh/perworld-l2-window/frozen-exes/*.exe`）与 guards 记录做 bytes+sha256 三方比对，**12/12 全等**；`render_per_world_light` = 92,590,602 B / `73245bbf…`（与 diag8 实测一致）。
- **93 guard inputs**：逐文件复算 **match=92 / drift=1 / missing=0**；唯一 drift 为继承项 `fixtures/render-bvh/scene-e2e/camlight-model.json`（guards 记 6,434 B/`470adbef…`，实际 6,020 B/`e44786d6…`），与 `guards-input-drift.txt` 一致、按 `-AllowOldStashDrift` 只报告不修复 ✅。
- 旧 suite 只读未变：文件仍 38,659 B / **`ca3ecd5c…`**，其 aggregate 字段 `8a4ddc50…` ✅；旧 stash 已知 drift 1 条（`render_shadow_reference`）在 guards `old_stash_drift` 与 freeze log 中如实记录 ✅。
- 冻结 SELFCHECK：**18/18 PASS**（`freeze-exit-code.txt` 可读记录；freeze 主 stdout 日志损坏——见 §5 一般项）。

## 2. monitor / 基线 / floor ✅（2 条低项）

- monitor：**samples=78、longest_clean=60、required=60、ready=true、exit 0**；util 2–6%、free_min 12,136 MiB、`compute_filtered=0`。**最终 60/60 段**（07:46:53.752→07:51:54.302，300.6 s）逐样本 `clean=True / build=0 / compute_filtered=0 / util≤6`、streak=60 ✅。
- 低项 A：README §1 写「build=0」，但整个 monitor 会话有 **3 个样本 build=2**（07:46:38/43/48，发生在最终 clean 段之前、使 streak 重置）；最终 60/60 段确实 build=0。属措辞不精确，非纪律违规。
- 基线/floor：三点 free **12,143 MiB**（util 3/2/2%）；`onspot_floor=12,143−3,072=9,071`；`floor_effective=max(9,956, 9,071)=9,956`；现场 free 12,143 ≥ 9,956 ✅。
- 低项 B：baseline 三行**时间戳完全相同**（`07:54:27.722167`），util 3/2/2 说明至少两次独立查询，但未按样点记录时间；**body 启动时刻未记录**，5 分钟规则只能由 body wall 合计（≈55 s）与归档写入时刻（07:55:54）推定（基线后约 30 s）。
- 低项 C：`floor.txt` 把 9,956 标为 `signed_stricter`；S-3 签名 floor 即 9,956（S-3 不复用 S-2），若严格按 `max(S-2,S-3)` 取严应为 10,230。**现场 free 12,143 同时 ≥ 两者**，故口径措辞不影响判定。

## 3. body env / outline pre-post / canary ✅

- `body-env.txt`：`PATH=target/toolchains/cuda-12.8.1/bin;<inherited>`；`MJWARP_RENDER_OUTLINE_LIST_DIR` 与 `MJWARP_REQUIRE_OUTLINE_CRITERION=1` **成对导出**（与 `run-window.ps1:291-292/307-308` 同口径）；`ENV_SELFCHECK_PASS failures=0`。
- outline pre/post 均 OK：recipe `cfd1038a…`/10,099 B/183 行/first_pixel 28,969/exact_max_ulp 73；aa2 `14893fc6…`/9,509 B/172 行/29,448/91；sidecar `8a80fb0a…`/5,027 与 `f23ca697…`/4,734 实测存在且哈希一致；bound 128。
- canary（`render_e2e.stdout.log`）：`outline[recipe] cam1: list_rows=183 outside_mismatch=0 in_list_differing=183 in_list_max_ulp=73 violations=0` ⇒ outline 判据生效、非 raw 回落 ✅。

## 4. 12 exe / 实体 A/B/C / 两跑 sha / world 隔离 ✅

- 12 个冻结 exe：entity-results.csv **12/12 exit=0**；逐 exe stdout 含 `test result: ok.` 且与 csv 的 test_result 一致、stderr 全空；body 为零编译（直接启动冻结 stash exe，`build_processes=0`；日志无 cargo/rustc）✅。
- 实体 `render_per_world_light`（`--nocapture` 全量落盘）：
  - **A PASS**：world0/1/2 × cam0/1 六帧 `rgb_mismatched=0 / max_byte_diff=0 / depth=0（0 ULP）/ seg=0`；
  - **B PASS**：`LightStats { worlds: 3, lights: 3, xpos_mismatched: 0, xdir_mismatched: 0 }`；
  - **world 隔离 PASS**：`isolation changed=[[false,false],[true,true],[false,false]]`；
  - **C PASS**：`c1=[38285,40644]`、`c2_depth=[35828,33883]`、`c2_seg=[16041,16754]`、`PassMargin`；`capture.json C1 交叉核对 = Compared`；
  - **两跑 sha**：`assert_eq!(first_sha, second_sha)` 通过（**只断言相等、未打印具体值**——已知 backlog，README §6 已如实记录，本窗未改码；观察项）。
  - pilot 用例 PASS；其旧 pilot capture 无 §2.4 字段 → 显式 `Skipped(...)`（符合预期）。
- `launcher-exitcode-selftest.txt` 记录退出码捕获机制自检（0/7/101 均正确捕获）✅。

## 5. 证据完整性 / diff 面 / 编码卫生（1 条一般项 + 2 条观察）

- manifest **51/51** 独立重算全对（bad=[]/extra=[]）；README `7ec3761f…`/9,393、receipt `23c424b6…`/9,577、manifest `d3b7a882…`/5,772 与声明逐字节一致 ✅。
- 归档提交 58 files = attempt4 证据 52 + guards JSON 1 + `.normify` 5；`src/`、`tests/`、`fixtures/` **零 diff**，`scripts/` 唯一变更是 guards 冻结产物（预期）；参考 5 文件目录、判据/spec 文件**零改动**；历史 attempt1–3 证据未改写 ✅。
- HEAD 漂移：body 于 `b84d5996`；窗口后我的两个提交 `bb179c9e`/`11530f82`（仅治理/证据、`src/tests/fixtures/scripts` 零 diff）被如实写入证据；`git diff b84d5996..5f1d436d` 的 `src/tests/fixtures` 为空 ✅。
- **一般项（证据卫生）**：`window/freeze-stdout.log`（3,740 B，NUL=1,867）与 `window/monitor-stdout.txt`（340 B，NUL=167）内容损坏——开头为两个 U+FFFD（BOM 被替换）+ UTF-16LE 残留字节（含 NUL），既非有效 UTF-8 也不可读。两份摘要（`freeze-exit-code.txt`、`monitor-exit-code.txt`）与 guards/aggregate/stash/inputs 的独立复算承载了全部结论，故**不构成窗口有效性阻断**；但违反「证据 log 一律 UTF-8/LF」口径，建议归档补一份可读副本或记 errata（不需重跑窗口）。
- 观察：guards JSON 为 CRLF（684 个 CRLF；PS `ConvertTo-Json` 行为，JSON 合法、哈希与证据副本一致）；guards 仍无 `body_env` 键（G-L2 后置 errata 已登记 backlog，body env 以 `body-env.txt`/checklist 记录）。
- **无 INVALID_ROUND**：`window/invalid-rounds-none.txt` 明确记录，窗口目录无无效轮附件；12 exe 单次执行、无重试 ✅。

## 6. 非声明

本复核只核归档证据的事实/完整性/纪律与判定依据；**不宣称对外「上游语义等价/灯字段端到端验收/渲染数值等价」**（需 S-1 判据与签名流程另行裁定），不改任何判据/阈值/参考/容差/相机/recipe，零 GPU、未重跑、未重冻；窗口结论（A/B/C 全过）与整窗状态收口由 Chief 宣布。
