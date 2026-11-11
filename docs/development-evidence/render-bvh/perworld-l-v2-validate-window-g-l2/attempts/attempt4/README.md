# G-L2 验证窗 attempt 4（canonical；判据 A/B/C 全通过）

- 任务口径：Chief 2026-11-11 07:43/#8 后 GO 的 attempt-4 验证窗（阶段 1 monitor+freeze+baseline，阶段 2 12 exe 单次执行）。零 GPU 之外不碰冻结件；不改判据/参考/容差；单次执行、失败即停、不放宽。
- HEAD 漂移说明（窗口有效性）：**body 于 `b84d5996` 执行**；归档前 QA 复核诊断轮 #8 的两个提交 `bb179c9e`、`11530f82` 落在窗口之后（仅 governance/证据，`src/tests/fixtures/scripts` diff = EMPTY），窗口内 guards 输入重核 **92 match / 1 drift(继承) / 0 missing**（见 `window/actual-input-hashes.txt`），无新增漂移 ⇒ 不影响本窗有效性。
- 现场：HEAD `b84d5996`（相对 `b0e372be` 的 `src/tests/fixtures/scripts` diff = EMPTY）；Windows x64 MSVC / rustc 1.99.0 / RTX 4070 Ti SUPER / CUDA 12.8.1（NVRTC pinned）。
- **本轮结论：执行完成、判据 A PASS（6 帧 rgb/depth/seg 全 0）、B 0/0、world 隔离与 C/§2.4 通过；11/11 回归 exe + 实体 exit 0。** 上游等价性/渲染数值等价等对外声明不在本窗范围（见 §9 非声明）。

## 1. monitor（`window/monitor.csv`、`window/monitor-stdout.txt`）

- 命令：`monitor-until-clean.ps1 -Required 60 -MaxMinutes 30 -MaxUtil 40 -OutCsv …/attempt4/monitor.csv`
- 结果：**exit 0、samples=78、longest_clean=60、required=60、ready=true**；时间 `2026-11-11T07:45:21.970+08:00 → 07:51:54.302+08:00`；util 2–6%；free_min 12,136 MiB；`compute_filtered=0`、`build=0`。
- 注：首次（阶段 1）冻结曾用脚本默认 `-Attempt 1`；按 GO 指示已用 `-Attempt 4` **重新冻结**，旧 guards 不留痕（仅本次 attempt=4 的 guards JSON 入库）。

## 2. guards 冻结（`window/freeze-stdout.log`、`window/guards-perworld-l2-light.json`）

- 命令：`freeze-guards-perworld-l2-light.ps1 -Execute -Attempt 4 -AllowOldStashDrift`，exit 0 / wall 5.2 s / **SELFCHECK PASS 18/18**。
- guards JSON：sha256 **`94478b3de924ee2940315de1f04617d4c81e48e0b4b9adb77f8a8301ae48734f`**、**43729 B**、`attempt=4`、`criterion_version=render-sdf-criterion-v2.2`、`bound_ulp=128`。
- aggregate_sha256 **`081801076e26128d6bd16c8e8c12f14dc0157fadeccec20b42eb0553c4579ef8`**；`counts = 12 exe + 93 guard_inputs + 2 outline_lists = **107**`；stash `target/render-bvh/perworld-l2-window/frozen-exes`。
- **12/12 `live==guard==stash`**（body 前后各核一次；stash bytes+sha 全一致）。
- 旧 stash 已知漂移 1 条（只报告不修复）：`render_shadow_reference` recorded 101,102,921 B/`ce513922…` vs actual 100,859,152 B/`4500bbe3…`；旧 suite `ca3ecd5c…` 只读未变。
- 12 exe old（perworld-impl-s1 attempt3 suite）→ new 身份见 `window/exe-identities.txt`；新实体 `render_per_world_light` = 92590602 B / `73245bbf175bf10c3332b6f3180ae6bc600a6db5fadb55e873ebfdd69a0d2542`（与 diag8 实测一致）。

## 3. 三点基线 + floor（`window/baseline-three-points.txt`、`window/floor.txt`）

| 点 | 采样（util, used, free, temp） |
| --- | --- |
| 1 | `3 %, 3919 MiB, 12143 MiB, 26` |
| 2 | `2 %, 3919 MiB, 12143 MiB, 26` |
| 3 | `2 %, 3919 MiB, 12143 MiB, 26` |

- signed：S-2 **10,230 MiB**、S-3 **9,956 MiB**（取严 signed floor = 9,956）；`onspot_floor = free_min(12,143) − budget(3,072) = 9,071`。
- **`floor_effective = max(9,956, 9,071) = 9,956 MiB`**；三点 free 均 12,143 MiB ≥ floor，util ≤3% < 40%，build=0。

## 4. body 环境（`window/body-env.txt`、`window/canonical-env-checklist.txt`）

- `PATH = target/toolchains/cuda-12.8.1/bin;<inherited>`；`MJWARP_RENDER_OUTLINE_LIST_DIR` 与 `MJWARP_REQUIRE_OUTLINE_CRITERION=1` **成对导出**（与 `run-window.ps1:291-292/307-308` 同口径）；`ENV_SELFCHECK_PASS failures=0`。
- pre/post outline 列表校验均 **OK**：recipe `cfd1038a…`/10,099 B/183 行/28,969/73；aa2 `14893fc6…`/9,509 B/172 行/29,448/91；sidecar 校验通过；`exact_max_ulp ≤ bound 128`。
- canary（`render_e2e.stdout.log`）：`outline[recipe] cam1: list_rows=183 outside_mismatch=0 in_list_differing=183 in_list_max_ulp=73 violations=0 exact_max_ulp=73` ⇒ outline 判据生效（非 raw 回落）。

## 5. 12 个冻结 exe（单次执行、直接冻结路径、`--ignored --test-threads=1 --nocapture`、零编译）

| # | entity | exit | wall (s) | test result |
| --- | --- | --- | --- | --- |
| 1 | render_per_world | 0 | 1.588 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 1.53s |
| 2 | render_e2e | 0 | 9.754 | test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 9.70s |
| 3 | render_kernel_rgb | 0 | 4.273 | test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 4.22s |
| 4 | render_kernel_depth_seg | 0 | 4.919 | test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 4.86s |
| 5 | render_kernel_flags | 0 | 11.016 | test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 10.96s |
| 6 | render_r10_multi_world | 0 | 2.744 | test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 2.69s |
| 7 | render_skybox_reference | 0 | 1.557 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 1.49s |
| 8 | render_context_lifecycle | 0 | 7.801 | test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 7.75s |
| 9 | render_mesh_bvh | 0 | 0.802 | test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.75s |
| 10 | render_hfield_reference | 0 | 2.739 | test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 2.68s |
| 11 | render_shadow_reference | 0 | 5.189 | test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 5.13s |
| 12 | render_per_world_light | 0 | 2.419 | test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 18 filtered out; finished in 1.98s |

- **12/12 exit 0**，与各 test result 行一致（退出码用 Python `subprocess.returncode` 捕获）；wall 合计约 61 s。
- body 期间**零 Rust 编译**：未调用 cargo/rustc，exe 直接取自冻结 stash；`build_processes=0`。

## 6. 实体 `render_per_world_light` 明细（`window/run/exe-logs/render_per_world_light.std*`）

- **判据 C（参考侧，产品数据前）PASS**：`c1_rgb_changed=[38285, 40644]`、`c2_depth_changed=[35828, 33883]`、`c2_seg_changed=[16041, 16754]`、`verdict=PassMargin`；诊断 `campersp depth=[0,0] seg=[0,0]`、`camhinge rgb=[50363, 52329]`。
- **§2.4 对账 = Compared**（非 Skipped/Skipped-旧结构）。
- **判据 A PASS**：world0/1/2 × cam0/1 六帧 `rgb_mismatched_pixels=0`、`rgb_max_byte_diff=0`、`depth=0/0 ULP`、`seg=0`（与 diag8 dump8 的 0 失配一致）。
- **判据 B PASS**：`LightStats { worlds: 3, lights: 3, xpos_mismatched: 0, xdir_mismatched: 0 }`。
- **world 隔离 PASS**：`isolation changed=[[false, false], [true, true], [false, false]]`（world0/world2 不变、world1 变化）。
- **两跑 sha**：`assert_eq!(first_sha, second_sha)` 通过（整块 rgb/depth/seg 确定性相等）。**说明**：当前测试源码只断言相等、未 `println!` 具体 sha 值，故本证据只记录「相等（断言通过）」；如需逐值留档，建议后续任务给测试加一行 sha 打印（不在本窗改代码）。
- 另一用例 `pilot_c1_crosscheck_matches_readout` PASS：pilot C1/C2 与 readout 一致，旧参考缺 §2.4 字段 → 显式 `Skipped`（符合预期）。

## 7. 运行后完整性（`window/post-integrity.txt`、`window/final-integrity.txt`）

- 12/12 `live==guard==stash` 复核通过；guards sha **body 前后未变**（`94478b3de924ee2940315de1f04617d4c81e48e0b4b9adb77f8a8301ae48734f`）；outline post 校验 OK；旧 suite 与旧 stash 只读未变。
- 树状态（body 结束）：仅 `M scripts/validate/render-bvh/guards-perworld-l2-light.json`（冻结产物，随本证据提交）；无其它改动。
- GPU 运行后 1–2% / free ≥12,140 MiB / 26–27 ℃；无 residual `render_*` 进程。

## 8. 证据结构（本目录 manifest 自核 N/N，见 `manifest.txt`）

- `window/monitor.csv|monitor-stdout.txt|monitor-exit-code.txt`
- `window/freeze-stdout.log|freeze-exit-code.txt|guards-perworld-l2-light.json`（guards 副本，只读对照用）
- `window/pre-integrity.txt|post-integrity.txt|final-integrity.txt|exe-identities.txt|guards-input-drift.txt`
- `window/baseline-three-points.txt|floor.txt`
- `window/body-env.txt|canonical-env-checklist.txt|outline-pre.txt|outline-post.txt|canary.txt`
- `window/run/entity-results.csv|run-summary.txt|exe-logs/*.stdout.log|*.stderr.log`（24 个日志）

## 9. 边界与非声明

- 边界：本窗只新增本证据目录 + 冻结产物 `scripts/validate/render-bvh/guards-perworld-l2-light.json`；`src/`、`tests/`、`fixtures/` 零改动；旧 suite/旧 stash/guards 只读不改；不重跑、不挑选结果、不放宽判据。
- 非声明：不宣称 G-L2 对外「上游等价 / 灯字段端到端验收 / 渲染数值等价」等结论（这些声明需 S-1 判据与签名流程另行裁定）；不改变任何判据/阈值/参考/容差/相机/recipe；历史失败轮次（attempt-3 及此前）不改写。

## 11. QA 复核后 errata（2026-11-11，W0 收口）

> **Addendum（2026-11-11，W0 收口；QA 复核 `5b586e77` / 收口卡 `6446c509`）**：
> ①（一般项·证据卫生）`window/freeze-stdout.log`（3,740 B，NUL=1,867）与 `window/monitor-stdout.txt`（340 B，NUL=167）为二进制损坏（含 NUL、非 UTF-8）；窗口有效性由摘要文件（`freeze-exit-code.txt`、`monitor-exit-code.txt`）与 guards/aggregate/stash/输入的独立复算承载，**不构成阻断**，可读副本与差异说明见 `…-attempt4-check/README.md` §5 与 `artifacts/evidence-diff-encoding.txt`；本目录另补可读 UTF-8/LF 副本 `window/freeze-stdout.readable.txt`、`window/monitor-stdout.readable.txt`（非原始字节，头三行注明派生）；
> ②（低 A）本卡 §1「build=0」仅指**最终 60/60 段**：monitor 早段 07:46:38/43/48 有 3 个样本 `build=2`（使 streak 重置），最终 60/60 段 `build=0`；
> ③（低 B）§1 基线三点时间戳相同、未按样点记录时间；body 启动时刻未记录（由 body wall ≈55 s 与归档写入 07:55:54 推定基线后约 30 s）；
> ④（低 C）`floor.txt` 的 `signed_stricter` 标注在严格 `max(S-2,S-3)` 口径下应为 **10,230 MiB**（S-3 签名 floor 为 9,956 MiB）；现场 free 12,143 MiB 同时 ≥ 两者，不影响判定；
> ⑤（观察）guards JSON 为 CRLF（684 个 CRLF，PowerShell `ConvertTo-Json` 行为，JSON 合法、哈希与证据副本一致）；guards 仍无 `body_env` 键（G-L2 后置 errata backlog；body env 以 `body-env.txt`/checklist 记录）。
> 以上均不改变 A/B/C 通过与窗口有效性结论。

- 证据 log 口径（本 addenda 引用一次）：**一律 UTF-8/LF 写出**（PowerShell `[IO.File]::WriteAllText(..., [Text.UTF8Encoding]::new($false))` 或 `Out-File -Encoding utf8`），**禁止默认 `>` 重定向**。
- 两跑 sha 具体值属 backlog（R1），不在本批改测试代码（避免动实体身份）。
