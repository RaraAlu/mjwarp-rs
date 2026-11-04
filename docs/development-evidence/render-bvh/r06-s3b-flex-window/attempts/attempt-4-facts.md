# attempt 4 事实记录（R06-S3b④ flex 渲染验证，D5 修复后重验）— 设备侧失败，停手

任务：`2026-11-04-render-r06-s3b-flex-validate-attempt4`（owner Arch，唯一执行会话 arch-group）。
执行日：2026-11-04，12:14–12:33（空闲块内）。**结果：实体首步即设备侧 CUDA 错误，判据未评估，停手留档。**
本文件为事实记录，不修饰、不给「通过/不通过」结论。

## 1. 开窗基线

- HEAD `5a32ef31`（plan 已补真实命令与 floor 手核要求）；tracked 树干净；build 进程 0；无外部 CUDA 计算任务。
- 三点基线（`nvidia-smi --query-gpu=memory.free,utilization.gpu,temperature.gpu --format=csv,noheader,nounits`，间隔 2.5 s，12:14:25–31）：

| point | time | free MiB | util % | temp C |
| --- | --- | --- | --- | --- |
| 1 | 12:14:25 | 14288 | 4 | 27 |
| 2 | 12:14:28 | 14288 | 4 | 27 |
| 3 | 12:14:31 | 14288 | 5 | 27 |

- `min_free = 14288`；`floor = max(11,688, 14288 − 3,072) = **11,688**`（脚本算，非手填；11,688 为 attempt3 生效值/ Chief 留痕 11,618 之上取严）。逐点 free ≥ floor、util < 40。
- 原始 CSV：`attempts/attempt-4-monitor-pre-window.csv`。
- 群内另有一组 12:15:19 的复采三点 `14,227 ×3`（util 4/4/2）——属开窗留痕阶段的二次采读，非本窗正式基线；两组 floor 复算均为 11,688，不影响结论。

## 2. monitor（通过）

- 首次运行 12:14:37 起 `monitor-until-clean.ps1 -Required 60 -MaxMinutes 30`，循环跑完后**收尾写默认 CSV 时崩溃**：默认输出目录 `target/s3b-validation-window/` 不存在（工具缺陷 D6），未产出 CSV、未改变任何判据。
- 补采：显式 `-OutCsv docs/development-evidence/render-bvh/r06-s3b-flex-window/attempts/attempt-4-monitor-official.csv`，同参数 `-Required 60 -MaxMinutes 12`，结果：
  `monitor samples=60 longest_clean=60 required=60 ready=True start=2026-11-04T12:25:21.869+08:00 end=2026-11-04T12:30:22.092+08:00 util_min=2 util_max=7 free_min=14286`
- `free_min 14,286 ≥ floor 11,688`；全程 util ≤ 7 < 40；compute 过滤命中 0、build 0。
- 另有同会话逐样本快照 `attempts/attempt-4-monitor-live.csv`（5 s 一行，含 floor_ok 列），作为人工逐点核对旁证。

## 3. 现场真冻结（通过）

命令：`freeze-guards-r06-s3b-flex.ps1 -Attempt 1`（12:30:25）：

```
GUARDS_OK=16/16 suite=render-r06-s3b-flex exes=2 inputs=14 stash=target/render-bvh/r06-s3b-flex-window/frozen-exes aggregate_sha256=de4a427972cfed2727f4f31c98661c12411eef47807d7a7b70703e2fe3c0d13a guards_sha256=70d4cbc698922c2b8e350c6b7d5f4764bf3a7342798cfd6f4906e26483f93152 bytes=7508
```

- guards JSON：`attempt=1 status=frozen`、`frozen_exes=2`、`guard_inputs=14`、`total=16`；`aggregate_inputs_sha256 = ec6fa222…`（**与冻结前一致**，未被重算改动）；文件 sha256 由 `42981f8d…` 变为 `70d4cbc6…`（预期变化）。
- 独立等价核验（同会话复算）：
  - 2/2 exe：live sha == guards sha == stash sha，bytes 一致；
  - 14/14 输入：live sha/bytes 与 guards 条目全等；
  - `aggregate_inputs_sha256`（仅 14 输入，Ordinal 路径序）复算 = `ec6fa222…` ✓；
  - `aggregate_sha256`（16 条目）复算 = `de4a4279…` ✓（与冻结输出一致）。
  - 注：用 PowerShell `Sort-Object`（culture 排序）会得到错误聚合值，必须用 `[StringComparer]::Ordinal`，与脚本一致。

## 4. 实体（失败：设备侧 CUDA 错误，判据未评估）

命令（唯一合法入口，PATH 前置 `target/toolchains/cuda-12.8.1/bin`）：

```
cargo test --locked --features cuda-probe --test render_flex_validate -- --ignored --nocapture
```

- 日志确认 `running 1 test`（无 0 用例 / 假绿）；D5 位姿诊断为真实输出：cam0 `angle=0.025deg`、cam1 `85.517deg`、cam2 `75.562deg`（与诊断/errata 一致，说明用例已改读参考 data 位姿）。
- 结果：`test flex_renders_match_captured_reference ... FAILED`，`finished in 1.49s`，进程 EXIT=101，墙钟 1.75 s。
- panic 位置与报错：

```
thread 'flex_renders_match_captured_reference' panicked at tests\render_flex_validate.rs:327:79:
flex 渲染发射: Driver("render-launch: 传输后端错误：CUDA失败：transfer-wait；错误码700")
```

- 错误码 700 = `CUDA_ERROR_ILLEGAL_ADDRESS`（设备侧非法内存访问），发生在 **render 发射的 transfer-wait**，不是 rgb/depth/seg 的数值比较；因此 **rgb/depth/seg 判据未进入、两跑 sha 未产生**。
- 原始日志：`attempts/attempt-4-entity-flex.log`。

## 5. 停手处置与边界

- 按窗口失败政策：**失败即停**，未重试、未重冻到过、未改判据/容差/参考、未请求第五次。
- 不宣称 flex 渲染通过或不通过；本窗唯一可写结论：**attempt4 = 设备侧 CUDA 700 失败，判据未评估**。
- 后续：需新增诊断（建议 D7，owner Backend）定位 `transfer-wait` 处 CUDA 700 的根因——是 win 上 flex 路径设备越界/同步缺陷，还是 D5 改动后新增的 host 侧传参越界；在根因未明前不开 attempt5，不调整任何判据。
- 归档：`attempts/attempt-4-{monitor-pre-window.csv,monitor-live.csv,monitor-official.csv,entity-flex.log,facts.md}` + 冻结后的 `guards-r06-s3b-flex.json`。
