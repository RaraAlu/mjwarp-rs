# U4 SDF validation window — attempt1/attempt2 key artifacts (INVALID_ROUND, not a pass)

任务：`2026-10-31-render-sdf-validate`（U4 验证窗；本目录是其 attempt1/attempt2 的**关键产物快照**）
状态：**两次 attempt 均 `INVALID_ROUND exit10`，U4 未通过（红线）**；U1b 已回滚（`07252cd2`），U1 保留。
本目录只搬入不可从代码复算的窗口产物；完整归档（9 实体日志、facts、samples、sha-guards、monitor 原文件等）
仍在 `target/render-bvh/sdf-validate-window/attempts/{attempt1,attempt2}/`（工作树，不入库）。

## 1. 为什么入库

窗口结论依赖的产物（guards 冻结身份、两份 monitor 原始 CSV、INVALID_ROUND 判定、9 实体结果）此前只在
`target/`，不可通过代码复现也不随仓库分发。本目录把它们按字节复制进证据树，便于独立复核与长期留痕；
`target/` 原件保留、未改名、未覆盖。

## 2. 两次 attempt 的身份

| 项 | attempt1 | attempt2 |
| --- | --- | --- |
| guards 文件 | `attempts/attempt1/guards-sdf-validate.json` | `attempts/attempt2/guards-sdf-validate.json` |
| sha256 | `0639f0ff5064bd2a7a44c1b9f6a29298b623b4d4901c9f089e0648a97591910d` | `d921caae93df0d911c51a64501f2427dcb58cb86f64a4158fd1f45b06933d726` |
| bytes | 33,651 | 33,651 |
| aggregate_sha256 | `02b2021376e27140552000a7124e4e5cff527ee558670677b6d58dcbdccf826f` | `a50e99c951c1a9277d8821f8aca9fcad2506ec5878183ff0f7ead0c53837e67a` |
| counts | 9 exe + 83 inputs = 92 | 9 exe + 83 inputs = 92 |
| 冻结 | 首次冻结 | **全新冻结**（attempt1 冻结不复用） |
| body | 8/9 exit 0，`render_sdf_reference` exit 101 | 同（复现） |
| 判定 | `INVALID_ROUND exit10` | `INVALID_ROUND exit10` |

## 3. monitor 证据（两份都在，绝不写成常规通过）

- `attempts/attempt1/monitor-pre-window.csv` — **常规 `util<40` monitor 失败**（354 样本、longest 28/60、`util` 5–50、`free_min` 11,545 MiB、exit 20）；**原样保留**。
- `attempts/attempt1/monitor-pre-window-exception.csv` — 已签条件式 validation-only `util<60` 例外 monitor（60/60、83 样本、exit 0）。
- `attempts/attempt2/monitor-pre-window-attempt2.csv` — 常规 monitor 失败（354 样本、longest 41/60、`util` 1–53、`free_min` 11,467 MiB、exit 20）。
- `attempts/attempt2/monitor-pre-window-exception-attempt2.csv` — 例外 monitor（60/60、83 样本、exit 0）。

两次 approach 都是「先常规、失败才启用已签例外」；例外 load-bearing 独立重算：attempt1 `ge40_clean=9` ⇒ true，
attempt2 `ge40_clean=8` ⇒ true；`ge60` 样本（attempt1 2 个、attempt2 1 个）均在 60 连清段之外（预热期、streak 0），
证据里照实记录，不写 `ge60=0`，不写「常规通过」。

## 4. `-GuardsOnly` 92/92 的来源（如实标注）

- attempt1：窗口内 `window/guard.log` 已含 `GUARDS_OK=92/92`（body 之前）；另有一次 `guards-only-run/`
  （`window_start=2026-10-31T01:52:41+08:00`，**晚于 body 结束**）——这是**窗口后补跑的复核**，
  `-GuardsOnly` 只做文件 sha 校验（CPU-only，不碰 GPU、不改变窗口判定）。两处时间在本目录以外，
  此处只作说明；该补跑记录未入库（`target/` 保留）。
- attempt2：`GUARDS_OK=92/92` 只来自窗口内 `window/guard.log`（body 前），**没有**窗口后补跑。

## 5. 偏差方向（attempt2 只读诊断）

`render_sdf_reference`：`cam0` 215/76,800、`cam1` 4/43,200；全量 `max |ULP|` = 191（cam0）/ 25（cam1）；
首差像素 (230,69)，参考 `seg=(2,5)` = sphere 轮廓。诊断是**双向偏差**：cam0 前 40 例 `product>ref` 16 例、
`product<ref` 24 例、ULP 7–71。RGB `mismatched=0`/`max_byte_diff=0`/`beyond_1_lsb=0`、seg `mismatched=0`。

## 6. 未做的事

不改判据/容差/参考、不缩相机、不跳球、不抬 `MAX_DEPTH_ULP`、不把「诊断已定位」当通过；本目录不含任何
「渲染数值验收通过 / 上游等价 / 精度达标」的声明。后续：sphere 轮廓 depth 只读 RCA → 独立修复片 →
U4 attempt=3（全新冻结）。
