# 窗口 attempt4 事实、记录 B 分类与 UNC-2 定稿（2026-10-26）

## 1. 窗口（attempt4，唯一有效窗口）
- 开窗前基线（原样）：`4187 MiB used / 11875 MiB free / util 36% / 33 °C / driver 596.36`；
  `query-compute-apps` 34 条桌面/系统上下文，无本任务计算进程（free≥10240 通过）。
- 结束基线：`4187/11875，8%，34 °C`；post `query-compute-apps` 无本任务残留。
- **执行结果（事实更正，不得写「5 个二进制 exit 0」）**：
  - **B/C/D 四个设备二进制 exit 0**：`nvrtc_launcher` 20.2 s、`primitive_cuda` 19.8 s、
    `multicontact_cuda` 20.3 s、`ccd_overflow` 0.3 s。
  - **E = `stage2_equivalence_window` 在窗口内 exit=101**：该进程内跑了全部 5 个测试，其中 3 个
    CPU 记录测试因**当时的 harness 行序比较 bug** 失败（见 §4）；**9 case dump 已全部落齐、
    两轮运行按 G5 行序口径逐位自洽**。harness 修正后 **CPU-only 重跑全部通过**（不占窗口、不重采）。
  - GPU 段合计 ~80 s（<2 min 计划）。
- 原始事实：`target/stage2-impl/window/attempt4/{baseline.txt,post.txt,facts.txt,*.log}`。

## 2. 窗口偏差（三条，如实登记）
1. 采样器 `Start-Job` 未落盘（`samples=0`）→ **guard v2 未实际生效**；仅有 pre/post 快照与逐步骤 wall、
   结束基线且无残留；未造成额外 GPU 负载（GPU 段 ~80 s）。**下一片开窗前必须先修好采样器并 dry-run 验证。**
2. **E 窗口内 exit=101 + 后续 CPU-only 修复重跑**：原因为 E 二进制一次跑了 5 个测试（dump + 3 个 CPU 记录），
   其中 CPU 记录测试因 harness 行序比较 bug（§4）失败；dump 本身已全部落盘、两轮逐位一致。
   修正 harness 后仅 **CPU-only** 重跑 `host_vs_product_dump_compare` / `window_device_vs_frozen_reference` /
   `three_record_summary` 全部通过（未重开窗口、未重采、未改 src 数值路径）。
3. attempt1/2（基线解析脚本 bug）、attempt3（NVRTC 不在 PATH）均**零 GPU 工作量**、作废不拼接；
   attempt4 为唯一有效窗口。

## 3. 记录 B（device vs G5 冻结 device 参考）分类
口径：run1/run2 一致性按 **G5 既有「行序不作判据」** 执行 —— worldid 分组 + G5 `sort_contacts` 键
（dist→pos[0..2]→geomcollisionid）稳定排序后逐位比较；数值判据/期望/参考未动。

| case | 分类 | 依据 |
| --- | --- | --- |
| cylinder-box / mesh-mesh / sphere-box-shallow | 有效参考，全绿 | 判据内通过（cylinder/mesh-mesh 26/26 位等） |
| ellipsoid-box | 有效参考，26/26 位等 | 原红为 harness 行序比较 bug（§4），修正后 26/26 位级 |
| sdf-box | 有效参考，1040/1040 位等 | Sdf-vs-out1 |
| sphere-box-deep / capsule-box | `reference_not_comparable` | **G5 README §11.2**：上游 dispatch 走 primitive_narrowphase，不列入设备 ccd 对照；out1 仅是探针 ccd 分支产物、从未与上游参考绑定 |
| convex-mesh / degenerate-mesh | `no_frozen_reference` | G2-only case，从未有设备输入 |

**不是 B 全绿**；明细：`target/stage2-impl/window/{record-b.json,three-records.json,result.json}`。

## 4. harness 行序修正前后差异（事实记录）
- 修前：run1/run2 一致性按**原始数组顺序**比较 → `ellipsoid-box` 因设备行序（worldid `1,0` vs `0,1`，
  由 atomic `nacon` 计数决定）误红；数值字段本身 26/26 逐位相同。
- 修后：按 **G5 口径（worldid 分组 + `sort_contacts` 排序键 dist→pos[0..2]→geomcollisionid）**
  稳定排序后逐位比较行集合，行序不作判据；`ellipsoid-box` 26/26 位等通过。
- **数值判据、期望、参考一律未动**；该修正是把 harness 对齐 G5 既有口径，不是放宽判据。

## 5. UNC-2 定稿（Chief 批准，限定语一字不改）
「默认构建含 CUDA 入口（`cuda` 默认特性、`--no-default-features` 可关）；产品入口经 NVRTC launcher 在真实设备执行中被调用（4 个设备测试二进制 exit 0；9 case dump 两次运行逐位一致）；记录 B 可评估子集全绿（cylinder/ellipsoid/mesh-mesh 26/26 位等、sphere-box-shallow 判据内、sdf-box 1040/1040 位等）；2 例按 G5 §11.2 参考不可比（sphere-box-deep/capsule-box）、2 例无冻结参考（convex/degenerate）——**不等于 B 全绿**；**默认选择层仍 Host、Device 需显式选择**——本项关闭不代表默认数值路径走 Device，不代表 host↔device 等价、上游等价、精度达标。」

## 6. QA 独立复核要点（`...-stage2-independent-check`）
- 核 B 分类的 G5 §11.2 依据（sphere-box-deep/capsule-box 为何不可比）；
- 核 2 例不可比 + 2 例无冻结参考的清单与 `record-b.json` 字段；
- 核 **E 窗口内 exit=101 的披露**与「dump 已落齐、CPU-only 修复重跑」的事实链；
- 核默认 feature（`default=["cuda"]`、`--no-default-features` 可关）与**默认选择层仍 Host、Device 需显式选择**口径；
- 核无判据/容差/期望值/参考路径改动（`git diff` 边界）；
- 核本文件与 README §7/§7.1、`record-b.json`/`three-records.json` 一致。