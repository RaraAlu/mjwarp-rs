# R05-S3b hfield 验证窗 · 离线 authoring 材料（窗口未开）

任务：`2026-10-28-render-bvh-r05-s3b-hfield-validate`（P8 渲染与 BVH；owner Backend；in_progress）
阶段：**离线 authoring 完成；窗口未开、未跑 GPU、未跑 `--ignored`、零渲染数值结论**。

本片把设备验证窗需要的东西全部离线写好：9 实体 runner、73 项 `guards.json`、0.25 s 采样器、
预窗监测脚本与窗口材料；窗内只跑实体与比对，不再写代码。

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
| `scripts/validate/render-bvh/freeze-guards.ps1` | 7 773 | 154 | `da5a0a1ff8a8991790d1a0eb229aa38d4614396b2ae1ebce985f1e8266976c8c` |
| `scripts/validate/render-bvh/monitor-until-clean.ps1` | 2 486 | 34 | `ec2b959ca4b067dcab251f67506800d78f1078df233ed6a2da8e00b13e818814` |
| `scripts/validate/render-bvh/guards.json` | 27 670 | — | `210f82088702ce0ea00bb357a88b1c75948f94b9ef48a632ad76546be1ba287d` |

（本表为 **2026-10-29T01:32:38.120+08:00 重冻结**后的值；此前 01:13 的冻结被 QA 构建导致 exe 漂移而作废，见 §8。）

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
- **guards.json sha256 = `210f82088702ce0ea00bb357a88b1c75948f94b9ef48a632ad76546be1ba287d`（27 670 B）**
- **aggregate_sha256 = `105ad0ab2cce4ce9eaa0e226089fe9e3f27bfb5ae38f247b9a1656c99efb226e`**
- 捕获产物 8 文件：**5 784 264 B**，聚合 `c6705a80a6ffe6b0c4a01974aa3e534983b5408782377dfdf665b2da5261fbbb`（重冻结前后一致）
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
- 监测脚本 sha `ec2b959c…`（ASCII-only）；CSV 原样留档。

## 5. 窗口命令与事实（逐字）

1. 开窗前最后构建后冻结：`powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate/render-bvh/freeze-guards.ps1`
2. 预窗监测：见 §4；达标后（并完成 Arch→Chief 双签）才开窗。
3. 开窗：`powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate/render-bvh/run-window.ps1 -Attempt 1`
4. 成功判据：exit 0 + `guard.log` 含 `WINDOW_RUN_DONE`；`sha_guards=73/73`（含 9 个 stash 副本校验）；
   每实体跑前/跑后 live exe sha 均等于冻结值（无 drift）；`post_window_exe_recheck=9/9`；
   `zero_rust_compile=true`；`entity_count=9`（1 new + 8 regression）且全部 exit 0；`floor_breached=false`。

- 规划 3 点基线（只读 `nvidia-smi`，见 `materials/gpu-baseline.txt`）：free_min **13 272 MiB**、util 38 %、
  **floor = min(10240, 13272−3072) = 10 200 MiB**（仅规划值；开窗按 runner 现场 baseline 重算）。
- `attempt = 1`（验证窗轮次）；wall 预估 **60–180 s**（依据：R09-S3b 8 回归实体 48.1 s；本窗多跑新 hfield
  目标且每实体经 cargo 调用）。
- 冻结对象清单：`materials/guard-manifest.txt`（73 行）+ `scripts/validate/render-bvh/guards.json`。

## 6. 本片边界

- 只做 authoring 与材料：**未开窗、未跑 GPU 实体、未跑 `--ignored`**；`src/` 与既有 fixture 零改动
  （新增仅 `scripts/validate/render-bvh/` 与本文档目录）。
- 零渲染数值结论：不写任何 rgb/depth/seg 比较数字；判据数字全部留窗口轮。
- 数字与差异只写可复核项；窗口若出现超判据、镜像/转置形差异或 BVH tie-break 可疑差异 → 停手报 Arch，
  不改容差/参考/fixture。

## 7. 遗留

- 等 Arch/Chief 对 **2026-10-29T01:32 重冻结后的材料**双签后再开窗；窗口产物（facts/logs/samples/
  guard_aggregate/entity-results 等）届时入证据并两段提交 + 收口。
- **冻结后纪律**：重冻结完成后零 cargo/rustc/测试，只做哈希与只读检查；任何构建都会使签名再次失效。
- `hfield_scene.rs` 已贴 500 行上限（authoring 卡终稿，LF 499），本片未再加行；若需改测试先拆。

## 8. 冻结修订记录（重冻结，QA 构建漂移后）

- **2026-10-29T01:13:03 首冻**：`guards.json` sha `39a6c1fe…`（25 023 B）；9 exe 冻结值（历史）见
  `materials/guard-summary.txt` 的 superseded 记录。
- **作废原因**：QA 复核构建在冻结后重链 9 个测试 exe（mtime 01:20:47–01:21:20；`target/debug/deps` 出现
  QA 探针 `_qa_s3b_auth_probe-…exe`）。test exe 重链不可复现（同尺寸不同字节），签名失效。
- **2026-10-29T01:32:38.120+08:00 重冻结**：`guards.json` sha `210f8208…`（27 670 B），
  aggregate `105ad0ab…`；新增 9 个 **stash 副本**（`target/render-bvh/r05-s3b-window/frozen-exes/<entity>.exe`）；
  `run-window.ps1` 新增 stash precheck + 每实体跑前/跑后 live sha 复查（sha `31ac28bf…`）。
- **64 项 guard 输入未变**：capture `c6705a80…`/5 784 264 B、legacy `b5cb50af…`/38·25 320 515 B、
  scene-shadow `cd990c20…`/16·11 561 363 B（重冻结前后逐项一致，未触发「输入变化即停手」）。
- 重冻结后 `run-window.ps1 -GuardsOnly` → **`GUARDS_OK=73/73`**、exit 0；之后零 cargo/rustc/测试。
- 新签名待 Arch→Chief 重签；签名前不得开窗。
