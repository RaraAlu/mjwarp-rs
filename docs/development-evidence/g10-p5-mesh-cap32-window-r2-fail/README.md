# r2 最小窗失败记录（FAILED；fail-stop：U7/U8 comparator failures≠0）

- 任务：`2026-11-12-w2-u7u8-full-condition-assert`（卡保持 in_progress）；本目录为**失败窗证据**，不是通过记录。
- 窗口：r2 最小窗 **attempt 1**（单实体 `g10_p5_branch_closure_cuda`，r2 冻结 exe）；**仅执行 1 次，未重试**。
- 结论：**verdict=FAILED**。测试 `test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 39.74s`；panic：`U7/U8 pass_rule 全条件与 S3 上游 GPU 参考不一致（21 条）`。
- fail-stop：命中第 6 条「U7/U8 pass_rule 任一条件不满足即停」→ 停在 in_progress、不重试、不放宽、不删证据。

## 1. 执行事实（纯记录）

| 项 | 值 |
| --- | --- |
| exe | `target/a2-freeze-r2/g10_p5_branch_closure_cuda-1b98cdc445ce5abc.exe`（sha `6d2b39fc…373677` / 41,771,008 B） |
| 实体结果 | `RESULT target=g10_p5_branch_closure_cuda exit= elapsed_s=40.05`（40.05 s 轮；日志 `test result: FAILED. 1 passed; 1 failed`） |
| sha 守卫 | 18/18 SHA_OK（1 exe + 15 fixture + 2 参考） |
| floor | GPU pre :: 4 %, 2622 MiB, 13440 MiB, 25, 596.36；GPU post :: 2 %, 2622 MiB, 13440 MiB, 25, 596.36；硬门 10,240 MiB → 通过 |
| 窗内 0.25 s 采样 | WINDOW-SAMPLING target=g10_p5_branch_closure_cuda samples=134 interval_target_ms=250 interval_min_ms=288.7 interval_max_ms=361.2 util_max=42 free_min=13439 csv=window-samples.csv（CSV `window-samples.csv`） |
| 零 Rust 编译 | facts 内 pre/POST 双检查 PASS；无残留 compute 进程 |
| 汇总行 | `SUMMARY entities=1 failed=4`（其中 failed=4 为 runner 计数缺陷，见 §3） |

## 2. 失败明细（21 条 comparator FAIL，见 `u7-u8-failures.json`）

- U7 `mesh-limits-cone-box`：**4 条**
  - `frame[0]` 符号相反；`geom[0][0]/geom[1][0]` 设备 `[0,1]` vs 参考 `[1,0]`；contact.geom 序断言失败。
- U8 `mesh-limits-prism-box`：**17 条**
  - `contact.geom=[0,1]` vs 参考 `[1,0]`；`frame[2]` 符号相反（逐行）；`pos` 逐行配对 ±0.05 交换。
- 可观测子集（record-only 打印）仍与 A2-3 一致：U7 count=1、dist 4 ULP、pos 0/0/1 ULP；
  U8 count=4、dist 4/4 均 0 ULP——说明该差异只有全条件 comparator 才暴露。

## 3. 窗口工具缺陷（如实记录，不改判据）

- `results.json` 的 `exit_code` 为 **null**（`exit_code_capture_bug`）：本 runner 为做窗内 0.25 s 采样未用
  `Start-Process -Wait`；本机 PowerShell 下 `-PassThru` 无 `-Wait` 时 `ExitCode` 读回为空（用 `cmd /c exit 3` 复现）。
  失败事实由实体日志 `test result: FAILED` 与 `.err` panic 行机证，不依赖该空值。
- `SUMMARY entities=1 failed=4`：`$results` 元素为 `[ordered]@{}`（OrderedDictionary），
  `Where-Object | .Count` 取到字典键数 4；实际失败实体数 = 1。
- 处置：**未重跑窗口**（不重试）；runner 修复归入修正片（`.NET ProcessStartInfo` 读 ExitCode + `@(...)` 计数）。

## 4. monitor attempt-1（开窗前未通过的那次监视，供留档）

- CSV：`logs/monitor-pre-window-r2-attempt1.csv`；统计：{"rows": 142, "util_max": 94, "free_min": 11802, "clean_all": false, "streak_last": 37}
  （未达 60/60 clean、util 超阈 → 当时未开窗；随后 Chief 重跑 monitor 达 60/60 才给 GO，本次窗口在第二次 monitor 后执行）。

## 5. 根因（已由 Chief 核源码批准；详见 `target/render-bvh/gl2-ptx-diff/w2-a2-precheck/r2-fail-rootcause.md`）

- 上游 `_add_geom_pair`（`collision_driver.py:351-380`）在 pair-builder 把 geom 对归一化为类型序
  （`type1 > type2` 时交换；相等保持调用顺序）；U7/U8 参考 `contact.geom=[1,0]` 是规则必然。
- 设备 `contact_multi` 入口不做窄相归一化（与上游分层一致）；`geomcollisionid = 0..n-1` 每行唯一。
- S4 `LIMIT_CASES` 的 `geom_type=[7,6]` + `build_meta` 硬编码 `g1=0,g2=1` → 传了 (mesh,box) 非归一化 pair；
  修正为类型序后（`g1=1,g2=0,type1=6,type2=7`）才与上游一致。M2 同结构（count=4/geom=[1,0]/ids=0..3）
  全条件 PASS，反证产品内核在相同输入下未偏离。
- **本失败不是判据歧义**：结构键唯一、配对确定；不需放宽判据。

## 6. 非声明

- 本目录是**失败记录**：不宣称 U7/U8 pass_rule 全条件通过、不宣称 U8 已验收、不宣称产品与上游等价或不等价、
  不宣称 mesh 容量缺口已清、G10@P5 整族通过、NP01/NP02 门禁通过；**不作位级等价声明、不提供位级等价保证**。
- 未改判据/容差/参考/comparator；未重跑窗口；本记录 append-only。

## 7. 证据文件

`README.md`、`receipt.json`、`u7-u8-failures.json`、`failure-record.md`、`facts.txt`、`results.json`、
`sha-guards.txt`、`window-samples.csv`、`logs/`（实体日志 + `.err` + 规范化 runner console + monitor attempt-1 CSV）、
`manifest.txt`。
