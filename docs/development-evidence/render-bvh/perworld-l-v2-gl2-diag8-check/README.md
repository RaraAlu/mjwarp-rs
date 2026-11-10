# QA 复核：诊断轮 #8（P10 验证，A 归零）

- 任务：`2026-11-11-render-perworld-l-v2-gl2-diag8-check`（owner QA；零 GPU、只读复核）。
- 对象：诊断轮 #8 现场 `target/render-bvh/perworld-l-v2-gl2/diag8/`。token 基准 `b0e372be`，复核 HEAD `b84d5996`。
- **结论：PASS（0 阻断，1 条信息观察）。** 单次/无重试、边界 clean、锁按记录（无审计日志）；实体 exit **0**（2 passed/0 failed，wall 3.30 s）、pilot 通过；dump8 **5,930 B / `7fe32fc6…`**、`schema==2`、`rgb/depth/seg_mismatches` **全 0**、6 帧全零、near-miss 每帧 76,800 像素全等；与 diag7 的 4 px **全部清空且无新增**；exe 92,590,602 B / `73245bbf…`（07:36 重编译，含 P10 内核）；P10 双内核身份 source 我独立复算与 P10 基线一致。**不判定整窗/修复结论**——G-L2 判定与 attempt 4 由 Chief 另行裁定；本轮只核「运行与 dump 合规」。

## 1. 运行合规 ✅

- HEAD `b84d5996`，`git diff b0e372be..HEAD -- src/ tests/ fixtures/ scripts/` **空**（仅 QA 治理/证据提交）；树 clean；diag8 未入库（`git ls-files` 空）。
- **单次/无重试**：仅一对 `test8.stdout/stderr.log`，无 retry/attempt/rerun 标记；时间线 pre-state 07:43:44 → prebuild 07:43:47 → test 07:43:51/54（dump 07:43:54）→ run-summary 07:44:39；prebuild `--no-run` 0.15 s cache hit（exe 已于 07:36 用新内核重建）。
- 运行：`cargo test --locked --features cuda-probe --test render_per_world_light -- --ignored --test-threads=1` → **exit 0、wall 3.30 s、`test result: ok. 2 passed; 0 failed; 0 ignored`**（entity `gpu_per_world_light_matches_reference` + `pilot_c1_crosscheck_matches_readout`）。
- 锁：pre-state/run-summary 记 Backend acquire→release；仓内无锁审计日志（既有 backlog），按记录+披露采信；当前 FREE。
- 现场：GPU pre 7%/3,919 MiB/12,143 free/26℃、post 2%/3,919 MiB/12,143 free/26℃（run-summary 已填，未再出现 `$gpu` 占位）；`body-env.txt` 路径本次**已正常**（无 TAB/退格污染，diag7 的 R2 已修正）。

## 2. dump8 与 diag7 对比 ✅

- dump8：**5,930 B / sha `7fe32fc6c0bc1345298103f3f62b1e8903aafbece118d5fc213f2f211978d267`**；`schema=2`、`kind=g-l2-entity-pixel-diagnostic`、`env=MJWARP_DIAG_PIXEL_DUMP`。
- `rgb_mismatches=0 / depth_mismatches=0 / seg_mismatches=0`；6 帧统计（world0/1/2 × cam0/1）**全 0/0/0/0**；near_miss 每帧 `equal=76,800 / mismatched=0`（diag7 为 w0c1=1、w1c1=3）。
- 与 diag7（55,485 B / `8cadf764…`）对比：diag7 的 4 个失配像素 `[(0,1,13268),(1,1,10744),(1,1,22903),(1,1,39492)]` 在 diag8 **全部消失且无新增**（dump8 mismatch 集为空）。`diagnostics` 字段只在失配条目出现，为空集时自然不存在 —— 与契约一致。

## 3. 身份 / exe ✅

- exe 实测 **92,590,602 B / `73245bbf175bf10c3332b6f3180ae6bc600a6db5fadb55e873ebfdd69a0d2542`**、mtime 2026-11-11T07:36:07（P10 修复后构建）。
- 我按 `mod.rs` 顺序独立重算拼接 source：megakernel **110,509 B / `6b1ade37…`**、depth_seg **53,552 B / `1e3f273c…`** —— 与 P10 修复卡基线逐位一致（PTX `48f8e7f8…`/348,399 与 `31912377…`/138,857 我在 P10 复核中用同方法验证过；本轮 src 相对 `b0e372be` 零 diff）。

## 4. 断言通过性（exit 0 的支撑）与信息限制

- 我重读 `tests/render_per_world_light.rs`：entity 用例内含 **C**（`verdict == PassMargin`、`check_capture_c1 == Compared`）、**A**（6 帧逐项 assert 全 0）、**B**（`(bstats.xpos_mismatched, bstats.xdir_mismatched)==(0,0)`）、**两跑 sha**（`assert_eq!(first_sha, second_sha)`）、**world 隔离**（world0/world2 changed==false、world1 any(true)）等断言 ⇒ **exit 0 即这些断言全部成立**。
- pilot 用例断言 capture §2.4 计数与当轮重算一致（c1 magnitude `[38285,40644]`、bitwise `[43604,41815]`、c2 depth `[35828,33883]`、seg `[16041,16754]`、PassMargin）——pilot 通过 ✅。
- **信息观察（唯一）**：cargo 捕获了通过用例的 stdout，**B / 两跑 sha / world 隔离 / C 的具体打印值未落盘**（A 可从 dump8 frames 直接看到全 0）；本轮 run-summary 以断言通过陈述。attempt 4 将用 `--nocapture` 补落打印值——建议届时把 B/sha/isolation/C 原文写入窗口证据。

## 5. 非声明

本复核只核运行与 dump 合规；**不判定修复效果/整窗结论**（A 归零的最终判定与 G-L2 状态更新由 Chief 裁定），不改判据/阈值/参考/容差，零 GPU，未重跑/未重冻。
