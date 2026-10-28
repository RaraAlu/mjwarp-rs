# R05-S3b hfield 验证窗 · attempt=2 窗口已执行（validation-only 例外）

任务：`2026-10-28-render-bvh-r05-s3b-hfield-validate`（P8 渲染与 BVH；owner Backend；卡 verified）
阶段（**2026-10-29 更新，QA O3**）：**attempt=2 验证窗已执行**（9 实体全 exit 0、`WINDOW_RUN_DONE`），
以 **validation-only 例外 `util<60`** 执行；本片**仍零渲染数值验收结论**（只记录数字与留痕）。

历史时点：本文件最初是 **离线 authoring** 文档（窗口未开、未跑 GPU、未跑 `--ignored`）——那是 S3a
authoring 阶段的实况；S3b 已于 attempt=2 实际开窗执行 9 实体，下文相应章节已更新并保留历史段落。

## 1. 实体清单（9，逐字命令）

| # | 实体 | source | 窗内命令 |
| --- | --- | --- | --- |
| 1 | `render_hfield_reference` | new | `cargo test --locked --features cuda-probe --test render_hfield_reference -- --ignored --test-threads=1 --nocapture` |
| 2 | `render_camera_rays` | regression | 同式（`--test render_camera_rays`） |
| 3 | `render_context_lifecycle` | regression | 同式 |
| 4 | `render_e2e` | regression | 同式 |
| 5 | `render_kernel_depth_seg` | regression | 同式 |
| 6 | `render_kernel_flags` | regression | 同式 |
| 7 | `render_kernel_rgb` | regression | 同式 |
| 8 | `render_r08_cameras` | regression | 同式 |
| 9 | `render_r10_multi_world` | regression | 同式 |

实体表原文（authoring 证据）已复制到 `materials/entity-table.json`；runner 按此顺序（new 先）逐条执行。

判据（沿用 R09-S3b，不新增不放宽）：RGB 每通道 ≤1 LSB（位级优先）；depth 位级优先
（`mismatched==0`）且 `max_ulp<=4`、`beyond_ulp==0`；seg 逐像素全等；两次运行 rgb/depth/seg sha256 相同。
每个实体必须在日志中给出每相机比较规模（像素数）与超判据计数，不能只写「通过」。

## 2. 脚本与冻结物

| 文件 | 字节 | 行数 | sha256 |
| --- | --- | --- | --- |
| `scripts/validate/render-bvh/run-window.ps1` | 18 187 | 386 | `31ac28bfcd8cce3bd10f6f5803f4a5573f218cccb88604e13d5537447d28a3c5` |
| `scripts/validate/render-bvh/freeze-guards.ps1` | 7 803 | 154 | `b01abff293d9bf328e35a7c279b74ee3260756d925ff9384f290e2137091f676` |
| `scripts/validate/render-bvh/monitor-until-clean.ps1` | 2 509 | 34 | `788d350555a48b8c94a9d85998725e67a5ad71e39f255f76c7fb6c8949f096d0` |
| `scripts/validate/render-bvh/guards.json` | 27 670 | — | `e2444a96128f33575bffab0f9420a1844c2f6cf2d3b60ef309f1fcad9b7eecb3`（attempt=2；attempt=1 旧值仅见 superseded 注记） |

（**attempt=2 现值**：`guards.json` `e2444a96…`/27 670 B/attempt=2/aggregate `7d24f036…`；
`freeze-guards.ps1` 为带 `-Attempt` 参数的 `b01abff2…`/7 803 B。）

**superseded 注记（QA O4）**：attempt=1（`2026-10-29T01:32:38.120+08:00` 重冻结）身份 = guards `210f8208…`/27 670 B、
aggregate `105ad0ab…`、9 exe 首项 `90d5becd…`/100 354 066 B、`freeze-guards.ps1` 旧版 `da5a0a1f…`/7 773 B；
均已被 attempt=2 取代，旧值仅在本注记与 `materials/guard-manifest.txt` 历史行保留。

- **runner 行为**：唯一硬门 = guard precheck（73 项 sha/bytes 照 `guards.json` 校验 **+ 9 个 stash 副本**逐一校验，
  任一不符 → 无效轮 exit 9 留档）；每实体**跑前**校验 live exe sha 与冻结值一致、**跑后**再校验一次，
  漂移即 abort → 无效轮 exit 9 留档（防止 `cargo test` 期间重链后静默使用别的字节）；起 0.25 s GPU 采样器
  （后台 job，写 `samples.csv`，util 连续 ≥80% 3 次写 abort 标记）；
  记录 3 点 baseline + `floor=min(10240, baseline_free−3072)`；逐实体运行上方命令并把原始 stdout/stderr
  写入 `logs/<entity>.log`；结束停止采样器、做 3 点 post baseline、`deps` 前后快照比对（零编译）、
  窗后 9/9 exe 复查、residual 进程计数，写 `facts.txt`/`entity-results.csv`/`sha-guards.txt`/`guard.log`/
  `guard_aggregate.json`；成功输出 `WINDOW_RUN_DONE` 且 exit 0。
- **退出码**：`0` = 有效轮 + `WINDOW_RUN_DONE`；`9` = 无效轮（precheck/guard sha/stash 不符、跑前或跑后
  live exe 漂移）；`10` = 实体失败或采样器 abort；`7` = 检出窗内编译（deps 快照变化）。所有轮次与 exit 码原样留档。
- **ASCII-only 三项自检**（材料 `materials/runner-ascii-check.txt`）：非 ASCII 字节 0、ANSI 解码行数 ==
  UTF-8 解码行数、无 `$` 代码行并入注释行；三个脚本均通过。
- **guards-only 干跑**（不跑实体、只校 guard）：**`GUARDS_OK=73/73`**、`GUARDS_ONLY_DONE`、**exit 0**
  （重冻结后输出在 `target/render-bvh/r05-s3b-window/guards-only-refreeze/`）。

## 3. `guards.json`（73 项，逐文件 sha 固定 + 9 个 exe stash 副本）

组成：**9 实体 exe + 8 个 capture-hfield 产物 + 2 个签字 scene XML + 38 个 legacy 参考文件 +
16 个 scene-shadow 采集产物 = 73**；`frozen_exes` 9 项带 `entity`/`source`/`stash_path`/`stash_sha256`/
`stash_bytes`，`guard_inputs` 64 项。**stash 副本**位于
`target/render-bvh/r05-s3b-window/frozen-exes/<entity>.exe`，与 live exe 逐字节一致（runner precheck 校验）；
后续任何构建只影响 `target/debug/deps`，不破坏签名字节。
逐项清单见 `materials/guard-manifest.txt`，汇总见 `materials/guard-summary.txt`：

- `aggregate_alg`：73 项 sha256 十六进制串按仓库相对路径排序拼接后再 sha256
- **guards.json sha256 = `e2444a96128f33575bffab0f9420a1844c2f6cf2d3b60ef309f1fcad9b7eecb3`（27 670 B；attempt=2，`generated_at 2026-10-29T04:10:37.668+08:00`）**
- **aggregate_sha256 = `7d24f0368f62e07cb4ef54a1978efba7b806b53d2a19c20c998d326648e90997`（attempt=2）**
- 注（QA O4）：attempt=1 的 `210f8208…`/aggregate `105ad0ab…`/exe `90d5becd…` 已 superseded，
  仅在 `materials/guard-manifest.txt` 历史行与本节注记保留；attempt=2 的 9 个新 exe sha 见
  `materials/freeze-attempt2.txt` §4。
- 捕获产物 8 文件：**5 784 264 B**；历史记录聚合 `c6705a80…`。
  **注（QA O6）**：在 64/64 输入逐项 sha/bytes 未变的前提下，该旧聚合值用已记录算法均无法复现；
  现按冻结脚本口径（`guard_inputs` 顺序 hex 串拼接）复算为 `d405ae21…`。故 `c6705a80…` 视为
  **不可复现的历史值**，不作复算依据；输入未变以 64/64 逐项比对为准（`receipt.refreeze_attempt2`）。
- legacy 38 文件：**25 320 515 B**，聚合 `b5cb50afcc608a67ca9a7b83222029402719f2770a3510ab423df13601e50d8b`（一致）
- scene-shadow 16 产物：**11 561 363 B**，聚合 `cd990c203112e44baf284c0da11c543f3a21612428e9396a1303671617a1fded`（一致）
- 9 实体 exe 的 live sha/bytes 与 stash sha/bytes 均记录在 `materials/guard-manifest.txt`（每实体 2 行），
  两者必须一致；冻结脚本从 `cargo test --locked --no-run --message-format=json` 的 compiler-artifact
  解析 live exe，避免误取旧 exe。
- **重新冻结规则**：若最后构建后任何冻结物变化，重跑 `freeze-guards.ps1`（会同时重建 stash 副本）并重新签名；
  冻结后到开窗禁跑 cargo/rustc/cargo test。

## 4. 预窗监测（设备验证窗不适用 authoring 例外）

- 阈值**原样**：`util<40`、`free≥10240`、compute-apps 过滤 `0`、构建进程 `0`；**5 s 采样、60 个连续干净样本**、
  上限 30 分钟（不达标就等/重试，不降门槛）。
- 逐字命令：
  `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate/render-bvh/monitor-until-clean.ps1 -OutCsv target/render-bvh/r05-s3b-window/monitor-pre-window.csv -Required 60 -MaxMinutes 30`
- 监测脚本 sha `788d3505…`/2 509 B（ASCII-only；`-MaxUtil` 参数化见提交 `01e36a54`：param 默认 40、阈值判断 `($util -lt $MaxUtil)`，默认行为不变）；CSV 原样留档。
- **superseded 注记**：monitor 旧版 sha `ec2b959ca4b067dcab251f67506800d78f1078df233ed6a2da8e00b13e818814`/2 486 B 已由提交 `01e36a54` 的 `-MaxUtil` 参数化版本替代（现 `788d3505…`/2 509 B）；旧值仅在本注记保留。
- **`-MaxUtil` 参数化 diff 摘要（提交 `01e36a54`）**：`param` 新增 `[int]$MaxUtil=40`；唯一阈值判断
  `($util -lt 40)` → `($util -lt $MaxUtil)`；free/filtered/build/Required/MaxMinutes/CSV 字段/exit 20 原样。
  默认 40 行为不变（烟测记录：不带 `-MaxUtil`、`-MaxMinutes 1` 跑 12 样本、CSV 10 列、ready=False、exit 20）。
- **例外轮逐字命令（实际使用）**：
  `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate/render-bvh/monitor-until-clean.ps1 -OutCsv target\render-bvh\r05-s3b-window\monitor-pre-window-attempt2-exception.csv -Required 60 -MaxMinutes 30 -MaxUtil 60`

## 5. 窗口命令与事实（逐字）

1. 开窗前最后构建后冻结：`powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate/render-bvh/freeze-guards.ps1`
2. 预窗监测：见 §4；达标后（并完成 Arch→Chief 双签）才开窗。
3. 开窗（**attempt=2，实际执行**）：`powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate/render-bvh/run-window.ps1 -Attempt 2 -OutDir target\render-bvh\r05-s3b-window\attempt2 -OpenUtilLimit 60`
   （`-OpenUtilLimit 60` 为 Chief 批准的 validation-only 例外；`-OpenUtilLimit` 不写时默认 40。逐字见 `attempt2/window-command.txt`。）
4. 成功判据：exit 0 + `guard.log` 含 `WINDOW_RUN_DONE`；`sha_guards=73/73`（含 9 个 stash 副本校验）；
   每实体跑前/跑后 live exe sha 均等于冻结值（无 drift）；`post_window_exe_recheck=9/9`；
   `zero_rust_compile=true`；`entity_count=9`（1 new + 8 regression）且全部 exit 0；`floor_breached=false`。

- 规划 3 点基线（只读 `nvidia-smi`，见 `materials/gpu-baseline.txt`）：free_min **13 272 MiB**、util 38 %、
  **floor = min(10240, 13272−3072) = 10 200 MiB**（仅规划值；实际窗按 runner 现场 baseline 重算为 10 233 MiB）。
- 实际轮次：**`attempt = 2`**（attempt1 为无效轮，原样保留）；实际 wall 51.2 s（实体 body），见 `attempt2/facts.txt`。
- 历史规划：`attempt = 1`、wall 预估 60–180 s（S3a authoring 时点；已由 attempt=2 实际执行取代）。
- 冻结对象清单：`materials/guard-manifest.txt`（73 行）+ `scripts/validate/render-bvh/guards.json`。

## 6. 本片边界

- **S3a authoring 阶段（历史时点）**：只做 authoring 与材料，未开窗、未跑 GPU 实体、未跑 `--ignored`；
  `src/` 与既有 fixture 零改动（新增仅 `scripts/validate/render-bvh/` 与本文档目录）。
- **S3b attempt=2（现行事实）**：已开窗并执行 9 实体（全 exit 0）；本文件记录数字与留痕，
  **仍零渲染数值验收结论**（不写门槛通过/精度判定的结论性表述）。
- 数字与差异只写可复核项；窗口若出现超判据、镜像/转置形差异或 BVH tie-break 可疑差异 → 停手报 Arch，
  不改容差/参考/fixture。

## 7. 遗留

- ~~等双签后再开窗~~（历史）：attempt=2 已按 Chief 有条件预批执行，窗口产物已入库（`attempt2/`、`monitor/`）
  并两段提交 + 收口；QA 复核 pass / 阻断 0 / 观察 10（O1–O6/O8/O10 已在本轮证据修正）。
- **冻结后纪律**：重冻结完成后零 cargo/rustc/测试，只做哈希与只读检查；任何构建都会使签名再次失效。
- `hfield_scene.rs` 已贴 500 行上限（authoring 卡终稿，LF 499），本片未再加行；若需改测试先拆。

## 8. 冻结修订记录（重冻结，QA 构建漂移后）

- **2026-10-29T01:13:03 首冻**：`guards.json` sha `39a6c1fe…`（25 023 B）；9 exe 冻结值（历史）见
  `materials/guard-summary.txt` 的 superseded 记录。
- **作废原因**：QA 复核构建在冻结后重链 9 个测试 exe（mtime 01:20:47–01:21:20；`target/debug/deps` 出现
  QA 探针 `_qa_s3b_auth_probe-…exe`）。test exe 重链不可复现（同尺寸不同字节），签名失效。
- **2026-10-29T01:32:38.120+08:00 重冻结（attempt=1，已被 attempt=2 superseded）**：`guards.json` sha `210f8208…`（27 670 B），
  aggregate `105ad0ab…`；新增 9 个 **stash 副本**（`target/render-bvh/r05-s3b-window/frozen-exes/<entity>.exe`）；
  `run-window.ps1` 新增 stash precheck + 每实体跑前/跑后 live sha 复查（sha `31ac28bf…`）。
- **64 项 guard 输入未变**：capture `c6705a80…`/5 784 264 B、legacy `b5cb50af…`/38·25 320 515 B、
  scene-shadow `cd990c20…`/16·11 561 363 B（重冻结前后逐项一致，未触发「输入变化即停手」）。
- 重冻结后 `run-window.ps1 -GuardsOnly` → **`GUARDS_OK=73/73`**、exit 0；之后零 cargo/rustc/测试。
- 新签名待 Arch→Chief 重签；签名前不得开窗。
- **2026-10-29T04:10:37.668+08:00 attempt=2 重冻结**（实际执行）：`guards.json` `e2444a96128f33575bffab0f9420a1844c2f6cf2d3b60ef309f1fcad9b7eecb3`/27 670 B、
  aggregate `7d24f0368f62e07cb4ef54a1978efba7b806b53d2a19c20c998d326648e90997`；旧 attempt=1 `210f82088702ce0ea00bb357a88b1c75948f94b9ef48a632ad76546be1ba287d`/`105ad0ab2cce4ce9eaa0e226089fe9e3f27bfb5ae38f247b9a1656c99efb226e` 已 superseded
  （详情 `materials/freeze-attempt2.txt`、提交 B `309a68b9`）。

## 9. attempt=2 窗口（validation-only 例外，**例外实际生效**；零渲染数值验收结论）

### 9.1 例外块（不得表述为常规阈值通过）

```text
mode=validation_only_exception_util_lt_60
approved_by=Chief 2026-10-29T04:5x（条件预批：常规 util<40 监测失败后改用本例外）
# O10：批准时间只有原消息的分钟精度（04:5x），无秒级时间戳可补；此处照实记录，不虚构秒值。
reason=persistent desktop GPU graphics contexts; filtered compute-apps=0; build=0; bitwise criteria + two-run sha unchanged
original_normal_threshold_attempts_retained=monitor/monitor-pre-window-attempt2-final.csv + -final-result.txt
  （samples=354、longest_clean=45<60、util 1-49、ge40=33、ge60=0、free_min 13163、decision=STOP、window_commands_executed=0）
not_a_normal_threshold_pass=true
authoring_exception_used=false
signature_unchanged=true（guards.json e2444a96…/27 670 B/attempt=2；9 exe live==stash）
```

**采样口径（QA O1 修正）**：
- `attempt2/guard.log` 原文：`PRECHECK_OK open=2757/13305/37/30;2757/13305/37/30;2757/13305/37/30`
  ⇒ **precheck 三样本 util = 37/37/37**（`run-window.ps1` L111-129）。
- `attempt2/baseline.txt` 原文：`37 / 40 / 40` ⇒ 这是 **precheck 之后另一次 3 点 baseline 采样**
  （`run-window.ps1` L131-135），与 precheck 是**两次不同采样**；两者都不能单独证明例外生效。
- **例外实际生效的唯一机器证据**在 `monitor/monitor-pre-window-attempt2-exception.csv`：5 个 util≥40 样本
  被记 `clean=True` 且 streak 连续增长——
  `04:50:55` util=**40** clean=True streak=2；`04:52:32` util=**41** clean=True streak=21；
  `04:54:24` util=**41** clean=True streak=43；`04:54:55` util=**40** clean=True streak=49；
  `04:55:45` util=**40** clean=True streak=59 ⇒ **生效门槛为 60**（`-MaxUtil 60`）。
- 例外监测逐字命令：
  `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate/render-bvh/monitor-until-clean.ps1 -OutCsv target\render-bvh\r05-s3b-window\monitor-pre-window-attempt2-exception.csv -Required 60 -MaxMinutes 30 -MaxUtil 60`
- 窗口以 `-OpenUtilLimit 60` 执行（逐字见 `attempt2/window-command.txt`）⇒ 本窗**不是常规阈值通过**；
  其余签字物（guards/exe/runner/monitor/floor 公式/判据/容差/参考）全未变。

### 9.2 窗口六项数字（原始产物在 `attempt2/`）

| 项 | 值 |
| --- | --- |
| 9 实体 exit | 全 **0**（wall_ms：3812/1314/7845/8563/4568/9123/3571/8051/3041） |
| `sha_guards` | **73/73 (9 exes + 64 read-only inputs), mismatches=0** |
| `post_window_exe_recheck` | **9/9 mismatches=0** |
| `zero_rust_compile` | **True**（deps 1836→1836，max write 2026-10-29 04:08:28） |
| 采样 | samples=**171**、util **11–43%**、`util_streak_ge_80=0`、free_min=**13 035 MiB**、used_peak=3 027 |
| 阈值 | baseline_free=13 305、`floor=10 233`、`floor_breached=False`、`increment_exceeded=False`、`guard_aborted=False`、`residual_processes=0` |
| 轮次/目录 | `attempt=2`、`dir=target/render-bvh/r05-s3b-window/attempt2`、window 04:56:10.798→04:57:05.184 |

### 9.3 两组 hfield 判据数字（每组 2 相机 ×120 000 像素）

- **hfield-only**：cam0（76 800 px）/cam1（43 200 px）RGB 位级 mismatched=0、max_byte_diff=0、beyond_1_lsb=0；
  depth mismatch=0、max_ulp=0、beyond_ulp=0；seg mismatch=0；depth_zero=29 776 / 37 050
- **hfield-primitive**：同上全 0；depth_zero=29 776 / 37 050
- 两次运行三 sha（hfield-only）：rgb `c58b9f178df3c8a9c7cf25ac266252825d8fcb19ebc99db3d2c38da80ff4daa0`、
  depth `87fb8b3aa3bb6bece2692c5fa1de92470ff6a5ed5e06276d433d2513db24b5dd`、
  seg `727bcf2ba8cc2e50aa0f1c1ae848f001b2afbe6fbdb10464fd2696c85f688a93`
- 两次运行三 sha（hfield-primitive）：rgb `0d494b5aaf537f140dcf214b93bff6c046af26f1d2cd857f5dde5fa54a258d40`、
  depth `e8ca22fe9488815017bcf4d4df44960d213e8ad77a242862bef189d5ebdbb64e`、
  seg `343c8978b0b1df4ed897d6560cf7bd43e549873647d6fa8768a602675b9324e7`

### 9.4 产物与留痕

- `attempt2/`：facts.txt、guard.log、entity-results.csv、guard_aggregate.json、baseline.txt、samples.csv（171 行）、
  sha-guards.txt、window-command.txt、exception-block.txt、logs/*.log（9 份，均 `test result: ok` / 0 failed）
- `monitor/`：常规与例外两轮 CSV + result 文件（数值见 9.1）
- **证据副本 vs 原始件（QA O8）**：`attempt2/window-command.txt`、`attempt2/exception-block.txt`、
  `attempt2/facts.txt` 原始件带 BOM+CRLF（`facts.txt` 为 CRLF），入库副本统一归一为 UTF-8 LF 无 BOM，
  内容逐行一致（仅行尾/编码归一）；`monitor/` 四个文件与 `target/` 原始件逐字节相同。
- **记录项**：O7 = `.git/info/attributes` 的 `-text` 使 guards.json 的 LF 归一 sha（`25cd1524…`）
  不等于现盘/blob sha（`e2444a96…`），fresh clone 无该本地属性时不可复现，按记录接受；
  O9 = QA 构建导致 HEAD 漂移，记录接受。
- 说明：以上为**数字与留痕**；本片**零渲染数值验收结论**（不写门槛通过/精度判定的结论性表述）。
