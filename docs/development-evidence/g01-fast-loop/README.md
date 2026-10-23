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

> 接线后（新 HEAD、11 项排除）的 debug/release 双模式最终复验见 §5.1；本节保留接线前版本作为历史。

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

### 5.1 接线后双模式最终复验（2026-10-23，QA，任务 `2026-10-23-final-acceptance-rerun`）

背景：W14/W15/U038 新测试接线（Backend `45baa0e`：补 `MJWARP_CUDA_INCLUDE`/CUDA bin PATH、补 `MJWARP_GRAVITY_3D_EVIDENCE_DIR`、U038-3D live 探针按裁决 B 加入显式 known-debt 排除、`-Minimum` 346）后，由 QA 按同一命令重跑 debug 与 release，确认双模式全绿且仅剩文档化排除项。

命令与 §5 相同；源码 HEAD `6e8c79a`（脚本为 `45baa0e` 接线版本）；工作区无产品/测试/夹具改动（本任务只改证据与文档）。
运行目录：debug `target/g01-acceptance/fff94fdd2e7c4e489362480b369c454b`，release `target/g01-acceptance/f0bb413b3de5423d91a82484c8a9d5de`；本机日志 `target/final-accept-debug.log`、`target/final-accept-release.log`；机器数据 `final-acceptance-wiring.json`。

| 阶段 | debug | release |
| --- | --- | --- |
| fmt | 通过 | 通过 |
| native | 27 过 | 27 过 |
| host-msvc | 722 过 | 722 过 |
| g01 | 7 过 | 7 过 |
| native-g01 | 2 过 | 2 过 |
| native-only-g01 | 1 过 | 1 过 |
| resident | 102 过 | 102 过 |
| all-probes | **346 过 / 0 败**（83 个 result 行） | **346 过 / 0 败**（83 个 result 行） |
| host-gnu | 406 过 | 406 过 |
| clippy-gnu / clippy-gnu-cuda / clippy-msvc | 0 警告 | 0 警告 |
| rustdoc（`-D warnings`） | 0 警告 | 0 警告 |
| diff | 通过 | 通过 |
| **全脚本墙钟** | **689s（11.5 分钟）** | **656s（10.9 分钟）** |

- **排除清单（11 项文档化排除，`<run>/probe-reports/exclusions.json`，四组单列）**：W-D5 3 项、历史捕获身份 6 项、env/外部产物未接线 1 项、U038-3D 已知债务 1 项（`live::live_three_d_product_chain_matches_frozen_upstream`，reason 单列）；未改断言、容差或排除清单。
- **环境接线（`probe-reports/env-wiring.json`，两模式一致）**：`cuda_include=wired`、`cuda_bin_path=wired`、`gravity_3d_evidence=fresh-directory`；原 3 个失败 target 中 `launcher-tile`/`launcher-warp-headers` 真实执行通过，3D live 探针按 known-debt 显式排除。
- **NP03 两级判定汇总（两模式一致）**：9 个 scope、79,286 块（79,202 原生放行 + 84 二级参考），`failed=0`、`pending=0`，比较标量 8,527,795。
- **与 §5（接线前 340/0）的差异**：all-probes 通过数 340→346（+6：launcher-tile 2、launcher-warp-headers 1、其余新测试/计数同步），result 行 78→83；排除项 10→11（新增 U038-3D known-debt 1 项）；host-msvc 709→722；host-gnu 404→406；墙钟 789.1s/652.9s→689s/656s。
- **与 Backend debug 首跑（`g01-new-tests-wiring`）对比**：计数完全一致（346 过 0 败、83 result 行、11 项排除）；其 README/§4.6 记录的运行目录 `10a6e9fd…`（墙钟 728.8s）与消息中的 `d5308115…` 属两次不同运行，本次 QA 记录以本轮 `fff94fdd…`/`f0bb413b…` 为准。
- 结论：**G01 debug/release 双模式通过（含 11 项文档化排除项；RNE 已按裁决重基线；U038-3D 已知债务显式排除、未验收）**。Linux（deferred）、性能与完整上游等价仍未验收。（历史快照：批次 1–4 与三处修复后的最终复验见 §5.2）

### 5.2 重钉与三处修复后双模式最终复验（2026-10-23/24，QA，任务 `2026-10-23-g01-final-acceptance-3`）

背景：批次 1–4 共 20 条切片交付合法改动 `src/physics/mod.rs`/`history.rs` 等，使 T4 冻结捕获身份审计（`preserves_current_sources_and_all_thirty_variants`）漂移、G01 host-msvc 一度失败（失败记录见 `docs/development-evidence/g01-head-drift/`）。随后按用户裁决 A 依次完成：① T4 源码身份重钉（`2026-10-23-t4-source-hash-repin-12`，只改哈希字面量，组合摘要 `7eb6e7a3…`→`d7f6c7d1…`）；② rustdoc 私有项断链修复（`2026-10-23-g01-rustdoc-private-link-fix`，`collision.rs` 一行文档）；③ A10 measure release 计时断言修复（`2026-10-23-a10-measure-release-timing-fix`，测试负载改为可测的 `black_box` 自增，保留 `nanos > 0` 严格断言）。

命令与 §5 相同；源码 HEAD `66256c9`；工作区无产品/测试/夹具改动（本任务只改证据与文档）。
运行目录：debug `target/g01-acceptance/383d4b51c15c462982a5b06cd47503de`，release `target/g01-acceptance/7d4d70d46e7f4fc3a86f3963de4e878a`；本机日志 `target/final-g01-check/{debug,release}-final3.log`；机器数据 `final-acceptance-final.json`。

| 阶段 | debug | release |
| --- | --- | --- |
| fmt | 通过 | 通过 |
| native | 27 过 | 27 过 |
| host-msvc | 796 过 | 796 过 |
| g01 | 7 过 | 7 过 |
| native-g01 | 2 过 | 2 过 |
| native-only-g01 | 1 过 | 1 过 |
| resident | 102 过 | 102 过 |
| all-probes | **346 过 / 0 败**（83 个 result 行） | **346 过 / 0 败**（83 个 result 行） |
| host-gnu | 480 过 | 480 过 |
| clippy-gnu / clippy-gnu-cuda / clippy-msvc | 0 警告 | 0 警告 |
| rustdoc（`-D warnings`） | 0 警告 | 0 警告 |
| diff | 通过 | 通过 |
| **全脚本墙钟** | **709s（11.8 分钟）** | **627s（10.5 分钟）** |

- **排除清单（11 项文档化排除，四组单列）**：W-D5 3 项、历史捕获身份 6 项、env/外部产物未接线 1 项、U038-3D 已知债务 1 项；与 §5.1 完全一致，**未新增排除**，未改断言/容差/排除清单/G01 脚本。
- **环境接线（`probe-reports/env-wiring.json`，两模式一致）**：`cuda_include`/`cuda_bin_path`/`t4_history`/`frozen_upstream`/`frozen_warp`/`collision_mocks` = wired，`t4_current_captures`/`gravity_3d_evidence` = fresh-directory，`sparse_preflight` = reused。
- **与 §5.1 的差异**：HEAD `6e8c79a`→`66256c9`；host-msvc 722→796、host-gnu 406→480（批次 1–4 新增 20 条切片测试）；all-probes 346 过/0 败、83 个 result 行两轮不变；排除项 11 不变；墙钟 689s/656s→709s/627s。
- 结论：**G01 debug/release 双模式通过（含 11 项文档化排除项；HEAD 66256c9；U038-3D 已知债务显式排除、未验收）**。Linux（deferred）、性能与完整上游等价仍未验收。

### 5.3 W13 后双模式复验（2026-10-24，QA，任务 `2026-10-24-g01-acceptance-after-w13`）

背景：W13（`1de8c11`）新增 `cuda-probe` 下的 nvJitLink 绑定源码、8 项单元测试与 2 项 GPU 测试（`tests/nvjitlink_probe.rs`），并把工具链 nvJitLink 改为 12.9.86。T4 源码身份清单不覆盖这些文件，未重钉。

命令与 §5 相同；运行前重跑 `scripts/prepare-cuda-probe.ps1`，`components.json` 含 `libnvjitlink` 12.9.86。
源码 HEAD `f9f1f66`（W13 `1de8c11` 之上只多一条任务登记提交）；工作区无产品/测试/夹具改动。
运行目录：debug `target/g01-acceptance/355a5a9e66504d288702e3ab6e87e24a`，release `target/g01-acceptance/a73659c1f1fd4acb835f902447b95d98`；本机日志 `target/final-g01-check/{debug,release}-w13.log`；机器数据 `final-acceptance-w13.json`。

| 阶段 | debug | release |
| --- | --- | --- |
| fmt | 通过 | 通过 |
| native | 27 过 | 27 过 |
| host-msvc | 804 过 | 804 过 |
| g01 | 7 过 | 7 过 |
| native-g01 | 2 过 | 2 过 |
| native-only-g01 | 1 过 | 1 过 |
| resident | 102 过 | 102 过 |
| all-probes | **348 过 / 0 败**（84 个 result 行） | **348 过 / 0 败**（84 个 result 行） |
| host-gnu | 480 过 | 480 过 |
| clippy-gnu / clippy-gnu-cuda / clippy-msvc | 0 警告 | 0 警告 |
| rustdoc（`-D warnings`） | 0 警告 | 0 警告 |
| diff | 通过 | 通过 |
| **全脚本墙钟** | **839s（14.0 分钟）** | **656s（10.9 分钟）** |

- **排除清单**：仍为 11 项（W-D5 3、历史捕获身份 6、env 未接线 1、U038-3D 已知债务 1），两模式逐项相同，**未新增排除**，未改断言/容差/排除清单/G01 脚本。
- **环境接线**：与 §5.2 相同；`cuda_include`/`cuda_bin_path`/`t4_history`/`frozen_upstream`/`frozen_warp`/`collision_mocks` = wired，`t4_current_captures`/`gravity_3d_evidence` = fresh-directory，`sparse_preflight` = reused。
- **G01 浮点比较**：两模式均 `states=2084 scalars=7704548 max_abs_error=1.2514791880302578e-6`、原始 `scalars=50512 max_abs_error=5.553492985477249e-7`，与 §5.2 相同。
- **W13 项**：all-probes 内 `nvjitlink_probe` 2 项通过；冻结 MathDx 链接后设备输出与离线证据逐字节一致（两模式各 1 次）。
- **与 §5.2 的差异**：HEAD `66256c9`→`f9f1f66`；host-msvc 796→804（W13 单元测试 8 项）；all-probes 346→348、result 行 83→84（W13 新测试二进制 2 项）；host-gnu 480 不变（W13 只在 `cuda-probe` 下编译）；排除项 11 不变；墙钟 709s/627s→839s/656s（未分析墙钟变化原因）。
- 结论：**G01 debug/release 双模式通过（含 11 项文档化排除项；HEAD f9f1f66；U038-3D 已知债务显式排除、未验收）**。Linux（deferred）、性能与完整上游等价仍未验收。


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
