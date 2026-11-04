# R06 flex 家族收口（A）复核 + attempt6 stats-print 抽查 ——**PASS-with-observations（0 阻断）**

- 复核卡：`2026-11-04-render-r06-flex-closeout-check`（owner QA，p84）
- 对象 A：`2026-11-04-render-r06-flex-closeout`（提交 **`1de0321a`**，verified）——`docs/development-evidence/render-bvh/r06-flex-summary/`（README 7,196 B / `9f9a2cd0…`、receipt 4,893 B、manifest 2/2）+ `docs/development-evidence/render-bvh-precheck/plan.json`
- 对象 B（顺手抽查）：attempt6 stats-print `2026-11-04-render-r06-s3b-flex-attempt6-stats-print`（提交 **`a0118271`**，verified）
- 纪律：只读、零 GPU、未重跑窗口；写锁仅用于本证据落库。

## 1. A 家族收口：六项复核

| # | 复核项 | 独立核对结果 |
| --- | --- | --- |
| 1 | 「单帧位级成立；多帧演化未覆盖」写死 | README §1 原文写死；receipt `multi_frame_evolution_covered=false`；缺口指向跟踪卡 `2026-11-04-render-r06-flex-multiframe`；README 明确「§2 表格是单帧判据、不承载动画一致性声明」 |
| 2 | 非声明齐全 | README §6 + receipt `non_claims`（5 条）：不声明动画与几何一致性整体成立 / 上游完全等价 / parity 通过 / flex 渲染整体可用 / 对外发布依据 / R06 家族之外能力 —— 逐条在场 |
| 3 | 签名 allowlist `3406f1ca` 逐字 | `cam2 / (x=134,y=128) / flexid0`、`ULP ≤ 4` 且 `beyond4 = 0`、`beyond4 ≠ 0` 即失败、不扩大不写成一般容差 —— 与签名卡 `3406f1ca` 措辞一致（我逐项匹配通过） |
| 4 | D8 分开登记 | `a1cc22ae` 的 321 px（beyond4 297 / maxULP 259）与 `d35cab68` 后 1 px 分开；明写「不混算、不叠加、不重解释」 |
| 5 | 证据索引与实物一致 | §2 表列 9 片（S0 / S1a / S2 / S2b / S2c / S3pre / S3a / S3b / S3c），逐片核：**实现卡全部 verified、`delivered_commits` 全部 `git cat-file` 存在、证据目录存在**（S1a 无独立证据目录，表内如实注明）；S3b 行覆盖 closure 证据 |
| 6 | `plan.json` 记账与实物一致 | `slices` 新增 **9 条 R06（全部 `status=delivered` + `delivered_commits`）**；`delivery_tracking.rows` 新增 **10 行**（九片 + S3b QA 复核行），逐行 `task_id` 卡文件 `verified` 且行内 `normify_status=verified`；`scheduling_notes` 新增 **N14**（含 attempt6 六项数字、allowlist、D8 分开、多帧未覆盖、已知限制、非声明）；`updated_at` → 2026-11-04；`roadmap-r06-p8` 保持 `proposed`（受 `roadmap-r01-p8` 门控）与 N14 声明一致 |

**收口提交面**：`1de0321a` 仅 5 个文件（card、plan.json、r06-flex-summary 的 README/manifest/receipt），`src/`/`tests/`/`fixtures/` 零 diff；plan.json diff 为 **+268 / −1**（唯一删除行是 `updated_at` 2026-11-03 的替换，属预期）；汇总 manifest **2/2** 自核通过；README/receipt 中 attempt6 六项数字（exe `e48c7268…`/107,854,859 B、bin `c5014d83…`/85,041,741 B、guards `attempt=2 frozen 16/16`、aggregate `c5263d33…`/`ec6fa222…`、monitor 60/60 free_min 14,414 ≥ 11,688、`running 1 test → 1 passed` 1.41 s、两跑 sha）与 attempt6 原始证据**逐项一致**（我已在收口复核中核过，本次再对读）。

## 2. B 顺手抽查：attempt6 stats-print `a0118271`

| 项 | 结果 |
| --- | --- |
| 改动面 | 仅 `tests/render_flex_validate.rs`、`tests/support/render_bvh/allowlist.rs` + 证据 + runs；`src/`、`fixtures/` **零 diff** |
| diff 性质 | tests diff **新增 43 行、删除 0 行**；新增内容=3 处 `println!`（reference flex px / 逐相机 depth 统计 / 逐相机 seg）+ 纯函数 `format_depth_stats`/`print_depth_stats` + 1 条格式自证 CPU 测试 |
| 是否动断言/阈值/判据/容差 | **没有**：`assert_depth_allowlisted` 函数体未改（无删除行）；allowlist 常量（cam2/134/128）未改；`MAX_DEPTH_ULP` 仅在被新增的统计函数里读取；未新增/修改任何 `assert_eq!` 业务判据（新增的 3 条 assert 全在新 CPU 格式测试内，且只断言打印文本） |
| 记账 | 卡 `verified`、证据 manifest 1/1、unit gate **1321 passed / 0 failed**（+1 新测试）；README 声称「19 tests / 18 passed / 1 ignored」——我按当前 HEAD 独立复跑确认（日志 `logs/cpu-selftest-qa.log`） |
| 结论 | **只加「通过时打印统计」**，未动判据/断言/阈值/容差；与 attempt6 收口结论无冲突 |

## 3. 观察（低，不阻塞）

1. **O-A1**：`plan.json` diff 有 1 行删除（`updated_at` 2026-11-03 → 2026-11-04）——属记账更新，非缺陷；已在证据中说明。
2. **O-A2**：README §2 证据路径用 `……/` 缩写（非全路径）；可读性好但 grep 略差，后续汇总若需机读可写全。
3. **O-A3**：README §4「17 filtered out 属正常过滤」是 attempt6 时点数字（当时 18 条测试）；stats-print 后测试为 19 条（18 passed + 1 ignored），该行仅描述当时运行、无需改。
4. **O-A4**：`roadmap-r06-p8` 的状态不在 plan.json 本体而是在其卡文件（`2026-10-11-roadmap-r06-p8`，`proposed`）；N14 的声明正确，仅「plan.json 内看不到该状态」这点可加注。

## 4. 结论

**PASS-with-observations（0 阻断）**：A 家族收口的六项必含内容全部成立、与已 verified 的实物一致，未发现扩大声明或记账错误；stats-print `a0118271` 抽查确认 test-only 只读增强、未动任何断言/阈值/判据/容差。可交 Chief 汇总；B（r01-p8）另行复核。

不宣称上游等价 / parity / 对外发布依据；不重开 attempt6 判定、不改任何证据。

## 5. 复现

```powershell
$ev = "docs\development-evidence\render-bvh\r06-flex-summary-check"
target\mjwarp-parallel\venv\Scripts\python.exe -B "$ev\scripts\qa_flex_closeout_check.py" --out "$ev\logs\closeout-check.log"
# CPU 自证（不碰 CUDA）
cargo test --locked --features cuda-probe --test render_flex_validate
```

## 6. 产物清单

- `README.md`（本文）、`facts.json`、`manifest.txt`
- `scripts/qa_flex_closeout_check.py`
- `logs/closeout-check.log`、`logs/cpu-selftest-qa.log`
