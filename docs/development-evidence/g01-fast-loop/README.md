# G01 验收快验环路（QA）

任务：`2026-10-23-g01-acceptance-fast-loop`。范围：`scripts/g01-fast.ps1`（新增）、本证据目录。

结论（阶段）：全脚本双模式耗时基线已统计；快验脚本已实现并通过自测，单 target 迭代从「全脚本一轮 11.1 分钟」降到「复用构建后 20 秒级」；最终双模式验收与并行评估待 RNE 重基线后执行（见 §5、§6）。

## 1. 耗时基线（2026-10-22 终轮全脚本，未受并发干扰）

来源：`target/g01-debug-final4.log`、`target/g01-release-final2.log` 与对应 `target/g01-acceptance/<run>/<stage>.log` 完成时间。
机器数据：`baseline-timings.json`。

| 阶段 | debug 秒 | release 秒 |
| --- | --- | --- |
| native | 10.8 | 7.6 |
| host-msvc | 13.0 | 5.9 |
| g01 | 18.9 | 17.0 |
| native-g01 | 5.0 | 4.5 |
| native-only-g01 | 0.7 | 0.7 |
| resident | 139.9 | 138.7 |
| all-probes | 475.5 | 404.6 |
| **到 all-probes 失败合计** | **663.8（11.1 分钟）** | **578.9（9.6 分钟）** |

- all-probes 占 debug 71.6%、release 69.9%；resident 占 21.1%、24.0%；其余六阶段合计不足 7%。
- all-probes 内 78 个测试二进制，debug 累计 461.4s、release 392.7s（逐项相加与墙钟差约 3%）。
- 最大单项：lib 测试（debug 133.3s / release 121.2s，124 项 ignored 探针），其后是 `t4_input_precision`（25.0/14.7）、`artifact_probe`（19.8/17.7）、`g01`（18.3/17.3）、`mixed_mass_factor`（17.6）、`cubecl_probe`（16.4/14.4），多数独立 target 在 10–25s。
- 两轮都在 all-probes 的 RNE 失败处中止，host-gnu/clippy/rustdoc/diff 未随脚本执行。
- 目录对应关系更正：`3afadc…` 是 debug（`all-probes.log` 含 77 条 `target\debug` 路径），`a947…` 是 release（76 条 `target\release` 路径）；Backend 交接 README 中两目录写反，本文以二进制路径为准。

## 2. 快验脚本 `scripts/g01-fast.ps1`

只用于开发迭代，**不替代** `scripts/test-windows-g01.ps1`：不校验 fixture/样本哈希、不跑排除清单、不归档验收证据、不改断言与容差。

```powershell
$pwsh = 'target\toolchains\powershell-7.6.7\bin\pwsh.exe'

# 一次构建（debug, all features）；构建清单写到 target/g01-fast/manifest-debug-all.json
& $pwsh -File scripts/g01-fast.ps1 -AllFeatures

# 复用构建，只跑受影响 target（按验收同口径 --ignored、单线程）
& $pwsh -File scripts/g01-fast.ps1 -NoBuild -AllFeatures -Target resident_rne_acceleration -IgnoredOnly

# 按测试名过滤（传给 libtest 的位置参数）
& $pwsh -File scripts/g01-fast.ps1 -NoBuild -AllFeatures -Target mjwarp_rs -IncludeIgnored -Nocapture -Test gravity_compensation::tests::gpu_matches

# release
& $pwsh -File scripts/g01-fast.ps1 -NoBuild -AllFeatures -Release -Target g01 -IgnoredOnly

# 列出构建清单里的 target
& $pwsh -File scripts/g01-fast.ps1 -NoBuild -AllFeatures -List
```

参数：`-Target`（支持 `*` 通配）、`-Test`、`-AllFeatures`、`-Release`、`-NoBuild`、`-BuildOnly`、`-List`、`-IncludeIgnored`、`-IgnoredOnly`、`-Nocapture`、`-TestThreads`（默认 1）、`-Parallel`（>1 时多 target 并行，PowerShell 7）、`-LogDir`。

实现要点：
- 用 `cargo test --no-run --message-format=json` 解析 `compiler-artifact` 的 `executable`，一次构建拿到全部测试二进制，后续 `-NoBuild` 直接运行，跳过 cargo 的重复 check/build。
- 每个 target 单独计时、单独日志；解析 `test result:` 行汇总 passed/failed；失败原样打印尾部，退出码非 0。
- 无测试命中的 target 视为通过（0 项）；进程崩溃（exit≠0）视为失败。
- `-NoBuild` 时比较源码 mtime 与清单 mtime，过期只告警，不静默重编译。
- 运行前若未设 `MJWARP_PHYSICS_PTX_DIR`，按本轮日志目录建唯一捕获根，避免并行写同一目录。

## 3. 快验提速实测

以「迭代一个受影响 target」为口径（构建成本两边都要付：全脚本在 native/host-msvc 阶段重编，快验一次构建后可反复跑）：

| 场景 | 全脚本 | 快验（复用构建） | 提速 |
| --- | --- | --- | --- |
| 单 target `g01`（7 项） | 789.1s（最终 debug 全脚本、14 阶段全绿） | 19.7s | 约 40 倍 |
| 单 target `resident_rne_acceleration` | 789.1s | 15.5s（all-probes 内逐项；单独跑 14–16s） | 约 50 倍 |
| 8 个 resident target（串行） | 789.1s | 80.7s | 约 9.8 倍（并行后更快，见 §6） |
| 只跑 CPU 型 target | 同上 | 0.0–0.2s | — |

- 快验一次构建实测 93.3s（`--all-features` 全量测试二进制，冷增量窗口）；之后 `-NoBuild` 复用清单只跑二进制。
- 2026-10-23 00:34–01:00 的 authoring 并发窗口内，同一 RNE target 实测 78.3–135.5s、g01 阶段约 33s；authoring 结束后复测 g01 19.7s（最终验收 g01 阶段 18.2s），确认争用消失。该窗口数据只作环境对照，不作为性能结论。

## 4. 环境备注

- 2026-10-23 00:30–00:44 有离线 authoring 占 GPU，验收与快验都被拖慢（见 §3）。
- 期间观察到 session 0 的 python 进程（uv 启动，约 4.4GB 工作集，显存占用曾报 15.5GB），`taskkill` 拒绝访问；00:50 后显存回落到约 1.6GB，温度 31–35C，恢复。
- 最终双模式验收（01:24–01:48）在 GPU 空闲窗口执行，未与 authoring 重叠；debug 789.1s、release 652.9s。
- 结论：慢窗口来自 authoring 及其残留上下文，环境未重启即恢复；未改动任何断言/容差/排除清单。

## 5. 最终双模式验收（2026-10-23，RNE 重基线后）

命令与交接文档一致：`scripts/test-windows-g01.ps1 -AllFeatures -FullRegression` 与 `... -Release`。
源码：RNE 重基线 `b13d1c5` 已入库（HEAD `f266802` 时执行），工作区无产品/夹具改动。
运行目录：debug `target/g01-acceptance/90e611aeee0b4f1fbbefc2681bdcae84`，release `target/g01-acceptance/9bf374a9fd4c428bb65ef059442932bc`；本机日志 `target/g01-qa-20261023-012436/{debug,release}.log`；机器数据 `final-acceptance.json`。

| 阶段 | debug | release |
| --- | --- | --- |
| native | 27 过 | 27 过 |
| host-msvc | 709 过 | 709 过 |
| g01 | 7 过 | 7 过 |
| native-g01 | 2 过 | 2 过 |
| native-only-g01 | 1 过 | 1 过 |
| resident | 102 过 | 102 过 |
| all-probes | **340 过 / 0 败**（78 个 result 行） | **340 过 / 0 败**（78 个 result 行） |
| host-gnu | 404 过 | 404 过 |
| clippy-gnu / clippy-gnu-cuda | 0 警告 | 0 警告 |
| clippy-msvc | 0 警告 | 0 警告 |
| rustdoc（`-D warnings`） | 0 警告 | 0 警告 |
| diff | 通过 | 通过 |
| **全脚本墙钟** | **789.1s（13.2 分钟）** | **652.9s（10.9 分钟）** |

- all-probes 逐项累计：debug 479.3s、release 387.8s；最大单项 lib 143.5s / 120.0s，其后 `t4_input_precision`、`artifact_probe`、`mixed_mass_factor`、`g01`、`cubecl_probe` 等 13–26s。
- 排除清单（`<run>/probe-reports/exclusions.json`，三组单列）：W-D5 3 项、历史捕获身份 6 项、env/外部产物未接线 1 项；未改断言、容差或排除清单。
- fixtures 来源：`git log --oneline -- fixtures/rne-acceleration` 最新为 `b13d1c5`（Backend 重基线），`git status --short fixtures` 为空；本任务未改任何夹具。
- 结论：**G01 debug/release 双模式通过（含文档化排除项；RNE 已按裁决重基线）**。Linux、性能与完整上游等价仍未验收。

## 6. 并行评估（开发迭代用途）

子集：8 个独立 resident target（`resident_rne_bias`、`resident_tendon_wake`、`resident_flex_edge`、`resident_flex_position`、`resident_flex_face`、`resident_sparse_mass`、`resident_kinematics`、`resident_spatial_tendon`），每进程 `--test-threads=1`，`-NoBuild`。

| 方式 | 墙钟 | 逐项累计 | 结果 |
| --- | --- | --- | --- |
| 串行（`-Parallel 1`） | 80.7s | 79.6s | 36 过 0 败 |
| 4 路并行 | 28.1s | 85.7s（+7.7%） | 36 过 0 败 |
| 8 路并行 | 17.4s | 95.9s（+20.4%） | 36 过 0 败 |

- 提速：4 路 2.87 倍、8 路 4.64 倍；单 target 平均变慢 8%–20%，说明单卡上内核仍被串行调度，收益主要来自 CPU 侧准备/启动重叠。
- 前提：脚本为本轮建唯一 `MJWARP_PHYSICS_PTX_DIR`（`<log>/physics-ptx`），配合 W-D5「根 + 唯一子目录」语义，避免并行写同一捕获目录；未改测试、断言与排除清单。
- 风险：单 GPU 多上下文有显存/调度风险，8 路本次通过但不作保证；并行只用于开发迭代，验收判定仍按串行完整脚本；GPU 满载任务须串行（authoring 争用实测拖慢 5–10 倍）。

## 7. 流程偏差记录

本任务在 `normify start` 之后调整过 `acceptance` 与 `schedule.files`（把 `docs/three-layer-acceptance.md` 移出范围、把 `fixtures` 从 unchanged 中 waive），原因是用户分工变更与并发窗口保护。
规范路径应为 **abandon + 重新登记**，而不是直接改任务 JSON；本次因范围收缩、有 waiver 与提交记录而保留，后续不再直接改启动后的任务声明。

## 8. 未验收边界

- 快验工具只服务开发迭代，不产生验收证据；不替代 `scripts/test-windows-g01.ps1`。
- 并行模式只用于迭代，不用于验收判定。
- Linux、性能、完整上游等价未验收；RNE 已按用户裁决重基线，但不宣称完整 G02 验收通过。
- 本任务未改 `scripts/test-windows-g01.ps1`、断言、容差与排除清单。
