# R08-S2 独立复核（QA）：多相机选择矩阵 + 异构分辨率端到端窗口

- 任务：`2026-10-28-render-bvh-r08-camera-e2e-check`（复核对象 `2026-10-28-render-bvh-r08-camera-e2e`）
- 提交链：`d62a007b`（CPU 测试片）→ `5a6b6c33`（空选择 no-op）→ `f8a15ef1`（A2，仅测试）→ `59ab0dbf`（证据 35 文件）→ `5686563b`（verified 记录）；复核时 HEAD `5686563b`
- 结论：**pass / 阻断 0 / 观察 4（4 建议）**；只读复核，未重开门、未重跑、未开 GPU、未改代码/判据/参考/容差

## 1. 总体判定

两轮窗口（attempt1 有效但新 target 未过 → A2 测试侧修正 → attempt2 全绿）的**记账、产物保留与数字**都由我
从原始产物独立重算：attempt2 145 样本 / used 4322–4593 / free_min 11469 / util_max 36 / streak 0 /
delta 270·248·271 / floor 8667；attempt1 146 样本 / 4290–4560 / 11502 / 43 / floor 8705 / delta 275·260·270；
45/45 守卫自行重哈希（attempt2 现网全 match；attempt1 44/45，唯一差异正是 A2 重建的新 target exe，历史性）；
7 实体 exit 全 0、新 target 6 passed/0 failed、四形态判据全 0（各 120000）；零编译 1816→1816；
runner `c8786951…`/14976 B 零编辑；证据 manifest 45 项对 committed blob 0 不符。**两条窗口口径
（空选择 no-op 已批行为变更；异构 RGB 属产品缺口、backlog 未 start）核实成立**。4 条观察均为记录措辞层面。

## 2. 窗口采样（我从 samples.csv 独立重算，raw 与入库副本逐项相同）

| 项 | attempt1（有效轮，未过） | attempt2（最终轮） |
| --- | --- | --- |
| 样本数 | **146** | **145** |
| used_min / used_peak（seq） | **4290** / **4560**（seq 21） | **4322** / **4593**（seq 22） |
| free_min | **11502** | **11469** |
| util_max / util≥80 streak | **43** / **0** | **36** / **0** |
| delta vs 基线 / 首样本 / 样本最小 | **275 / 260 / 270** | **270 / 248 / 271** |
| sustained_delta（本轮 guard 口径 = peak−baseline） | **275** | **270** |
| baseline_free → floor（min(10240, free−3072)） | 11777 → **8705** | 11739 → **8667** |
| 采样间隔 median（min–max） | 0.2815（0.2781–0.3540） | 0.2816（0.2788–0.3052） |
| 窗口起止 / body | 02:29:26.5586→02:30:10.5365 / 41.2 s | 02:39:59.7093→02:40:42.8477 / 40.9 s |

`increment_limit=3072` 未触、floor 未破、guard 未 abort、residual 0；两次 attempt 均在 02:29 / 02:39（空闲块
00:00–09:00 内，早于 08:50 边界），无跨块"同窗"表述。

## 3. 守卫（自己重哈希）

- attempt2：**45/45** 逐条重算 sha256+字节全 match（7 frozen exe + 38 只读输入）；窗后 7 exe **7/7**。
- attempt1：45 条里 **44/45** 今天仍 match；唯一不匹配 = `render_r08_cameras-…exe`（A2 重建：94027937 B
  `6a73c51a…` → 94029705 B `918b5851…`），属预期历史差异（见观察 O2）。
- `guards-check/` 与 `guards-check-a2/` 的 sha-guards 均已刷新为新 sha **918b5851…**，旧 `6a73c51a` 在
  guards-check 内 **0 命中**；`guards-check/sha-guards.txt` mtime **02:33:11**（非 02:22 旧值）。
- 旧 sha 在证据中仅出现于 `logs/attempt1/sha-guards.txt`（历史留档），位置正确。

## 4. 实体与判据（读 attempt2 原始实体日志）

- 7 实体 exit 全 **0**（entity-results.csv 与 facts 双重一致）；a1 为 `render_r08_cameras` **101** + 其余 6 个 0。
- 新 target `render_r08_cameras`：attempt2 `6 passed / 0 failed`；attempt1 `5 passed / 1 failed`（显存快照顺序）。
- 四形态（同冻结参考）：All / Names 输出 sha `f9fe1b9e…be688` 相同（两次运行一致），Mask `cfc1a4b6…`，
  Indices(reversed) `73891435…`；每形态 `rgb_pixels/depth_pixels/seg_pairs` 聚合（120000 或 Mask 76800）与
  `rgb_beyond_1_lsb: 0`、`depth_mismatch: 0`、`max_depth_ulp: 0`、`seg_mismatch: 0` 全部为 0；
  逐相机 7 条视图行 `beyond_1_lsb=0 / depth_mismatch=0 / max_ulp=0 / seg_mismatch=0`。
- 动态射线 vs 预计算：depth/seg 逐位相同，sha `b09e2eec…0c31`。
- 隔离：subset_cam0（76800/0）、subset_cam1（0/43200）、reorder（76800/43200）均与组合渲染逐位相同。
- 异构通道：`depth_adr=[0, -1]`、`seg_adr=[-1, 0]` 逐相机对账 + 未启用通道读取被拒成立；异构 RGB 明确记为
  产品缺口（backlog `2026-10-28-render-per-camera-rgb-gap`，status=proposed，未 start）。

## 5. A2 与空选择口径

- `f8a15ef1`：**1 文件 / +14−4 / 仅 `tests/support/render_bvh/r08_gpu.rs`**；无 `src/ fixtures/ native/ include/`
  路径。产品码在 A2 中零改动。
- 产品侧空选择改动（`5a6b6c33`）：2 个 `src/` 文件各 `1/1` 等行数替换（megakernel.rs + renderer.rs），
  其余为测试；`InvalidArray("render_cameras")` → `Ok(())` no-op 对齐上游 `wp.launch(dim=0)`。
- 全仓检索：`tests/` 与已签证据中**没有**把该 `InvalidArray` 当契约的断言/记录（命中 0 处 render_cameras 契约）。
- attempt1 产品侧实际执行到的断言：`Ok`（mega ×2 + depth_seg ×1）、构造后三通道 `is_enabled=false` 断言
  （测试 L270–279，在渲染前）——通过；`read().is_err()` 三通道断言在失败断言**之后**（L298–307），
  attempt1 未执行（日志无 `r08[empty]` 打印），只在 attempt2 验证（见观察 O1）。

## 6. 窗口事实与零编译

- `deps_before/after_count=1816/1816`、`deps_max_write` 前后同值（02:31:19）、`zero_rust_compile=True`；
  `feature_profile=cuda-probe`、`env_cuda_bin=…cuda-12.8.1\bin`、`compute_apps_post_raw=28 filtered=0`、
  窗后 `post_window_exe_recheck=7/7`、`residual_processes=0`。
- runner `target/r08-s2-window/run-window-attempt1.ps1` **14976 B / sha `c8786951…e71a`**，两次 attempt 同一字节。
- 无新 authoring 捕获（复用 S0b/S7b/S8 冻结参考；窗口内无 Python/采集）。

## 7. 证据与不变量

- `59ab0dbf` 35 文件全部在 `docs/development-evidence/` 与 `.normify/`，无 `src/tests/fixtures/Cargo*`；
  `5686563b` 5 文件。`manifest.json` **45 项**逐条 sha256/bytes/lines + committed blob **0 不符**。
- 行数：README **172**、receipt **396**、manifest **409**；新文件 **320/366/321**（≤500）；
  `megakernel.rs 498 / renderer.rs 265 / launch.rs 500 / model/render.rs 500 / context.rs 426 / mod.rs 374`。
- 红线上 `e2142204..HEAD` 对 `fixtures/precision`、`src/model/render.rs`、kernel cuda/launch/output、
  `native/ include/ Cargo*/build.rs`、`collision_gpu/source.rs` 的 diff 为空；`DEVICE_SOURCES`/入口/504 B 未动
  （随 unit 断言）。
- 证据文本 UTF-8 无 BOM/LF；`unit.log` 我重数 **1015 passed / 0 failed / 382 ignored / 125 result 行**；
  `directed-render-r08-cameras.log` 5 passed / 6 ignored；禁词 0。

## 8. 观察（4 条，均建议级，给修/接受建议）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | 建议 | README §3.1 把 attempt1 的空选择产品断言写成"全部成立：两个 renderer 均返回 Ok（各两次）、三通道缓冲未分配、无可读输出、不 panic"。实际：Ok 是 mega ×2 + depth_seg ×1（不是各两次）；`read().is_err()` 三通道断言在失败断言之后、**attempt1 未执行**（仅 attempt2 验证）；unchecked panic 时为测试断言而非产品 panic | **修**（措辞）：区分"attempt1 已验证（Ok 计数 + 构造后 is_enabled=false）"与"attempt2 才验证（无可读输出）"；不影响任何数字 |
| O2 | 建议 | attempt1 `sha-guards.txt` 仍写旧新 target exe sha（`6a73c51a…`/94027937 B），今天 45 条里 44 match；README 只说"其余 6 个 exe 逐字节未变"，未标注该 exe 已被 A2 重建 | **修**（旁注）：在 attempt1 守卫处注"该 exe 已被 A2 重建，旧 sha 仅历史留档，今日 44/45 属预期"；或**接受** |
| O3 | 建议 | `sustained_delta_mib` 本轮口径 = peak−baseline（a1 275 / a2 270），与早前 S3 窗口"排除尖峰后 max−min"口径不同；README/facts 只列数字未写定义 | **修**（写明定义）或**接受**（数字本身无误） |
| O4 | 建议 | `guards-check/` 与 `guards-check-a2/` 内容已完全相同（均刷新到新 sha），README §3.2 只说明 guards-check 刷新时间，未说明两者关系 | **接受**（历史产物）；建议后续目录注明"a2 为副本"或清理 |

## 9. 边界与不宣称

只读复核：未重开门、未重跑实体、未开 GPU、未改代码/判据/参考/容差、未改 runner/冻结 exe/守卫。本片只核
"两轮窗口事实与端到端判据是否被如实记录、数字是否可独立复算、口径是否成立"，不替代用户/Chief 的窗口签字，
也不对后续方向作验收宣称。
