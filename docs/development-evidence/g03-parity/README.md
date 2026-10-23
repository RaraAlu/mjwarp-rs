# G03 完整分支与精度 Windows 单平台收口

任务：`2026-10-16-next-g03-parity`（按用户 2026-10-24 决定重划为 Windows 单平台收口）。日期：2026-10-24。执行人：QA。
最终 HEAD：`c4c10580455d05e4fc5dc90b28ac9f6ce6a105ac`（G01 两轮 `report.json` 的 `gitRevision` 均为该值）。
平台：Windows 11（10.0.26200）、RTX 4070 Ti SUPER（驱动 596.36，SM89）、`rustc 1.99.0 (b940084d7 2026-09-28)`、`--all-features`。

## 结论（只对 Windows 已覆盖范围）

- **Windows 单平台收口完成**：Tile 发射（W14）、正式编译路线（W13 nvJitLink + W15 头文件 + LTOIR 输出接口 + 内核来源路线 + MathDx LTO 生成/链接对照）、Small 与稀疏实际算序对照均引用对应 verified 交付与证据。
- **里程碑终验（本轮唯一一次全量 G01）在最终 HEAD `c4c1058` 双模式全绿**：debug 与 release 各 **14/14 阶段 exit 0**，host-msvc **835 过 0 败**、all-probes **353 过 0 败（87 条 result 行、11 项文档化排除）**、rustdoc **0 警告**、diff 通过；两模式数字逐项一致。
- 墙钟：debug **775.5s**、release **641.1s**（运行目录 `target/g01-acceptance/b9fb775e067e451daa99fbdaec33e922`、`0c2d6e4355794126b65e0a383c09c7d5`）。
- 与上次全绿（`4f362f2`，`docs/three-layer-acceptance.md` §4.11）相比：host-msvc 823→**835**（+12 内核来源路线单测）、all-probes 352→**353**（+1 ignored GPU 探针，`kernel_source_route_probe` 自动纳入）、result 行 86→**87**、排除仍 **11 项**；host-gnu 480 不变；rustdoc/ diff 保持 0。
- **不宣称双平台、不宣称完整上游等价、不宣称 g03-parity 全量通吃**：Linux 按用户 2026-10-24 决定 deferred（并入 `2026-10-24-linux-batch-retest`）；未满足项见下。

## 收口清单（引用 verified 交付与证据）

| 项 | 交付（提交） | 证据 | 收口要点 |
| --- | --- | --- | --- |
| Tile 发射约束（W14） | `f98dc34` | `docs/development-evidence/layers-w14*` / `.normify/runs/2026-10-23-w14-tile-launcher/` | tile 共享内存声明与发射约束落地；本收口引用其 verified 记录，不复用旧结论 |
| nvJitLink 绑定（W13） | `1de8c11` | `docs/development-evidence/layers-w13.md`、`layers-w13-ltoir.json` | 自建动态绑定 12.9.86；CUBIN 路径可用 |
| W15 头文件 | `b1de2db` | `docs/development-evidence/warp-headers*`、`include/warp-headers/` | 上游头文件原样复制 + NVRTC include 路径 |
| LTOIR 输出接口 | `f9da799` | `docs/development-evidence/compile-ltoir/`、`ltoir-independent-check/` | 调用 LTOIR 3632 B（`1415be10…`）、CUBIN 2,086,544 B |
| 内核来源路线标准化 | `687754c` | `docs/development-evidence/kernel-source-route/`、`kernel-source-route-independent-check/` | 四类路线 + `RouteKey`；默认不启用、不切物理配方；两环境探针各 1/0、route key `27c33a4e…` |
| MathDx LTO 生成与对照 | `2025a57` / `e5f1559` / `4f362f2` | `docs/development-evidence/mathdx-lto/`、`mathdx-independent-check/` | libmathdx 0.3.1 组件与许可、Solver LTO 绑定与冻结对照、NVRTC 12.9.86 组件化 |
| Small 分解与 MathDx 链接 | 历史交付 `2026-10-16-done-small-mathdx`（verified） | `docs/development-evidence/g03-matrix-chain-alignment/` | 首差循环归零：xpos/xquat/xmat/xipos/ximat/xanchor/xaxis/subtree_com/cinert/crb/cdof/matrix_dense_block 全部 0 位差；MathDx potrf **0/121** 位差，产品 GPU 解 **0/33** 位差 |
| 稀疏分解与回代 | 历史交付 `2026-10-16-done-sparse`（verified） | `docs/development-evidence/g03-matrix-chain-alignment/`、`docs/evidence/resident-sparse-solve/` | 稀疏算序与冻结上游一致；`mass_solve_probe` 6 过 0 败，NP03 汇总 factors/native-snapshots/rhs-counts 全 0 失败 |
| 三路混合分解与桥接 | 历史交付 `2026-10-16-done-mixed`（verified） | `docs/evidence/resident-mixed-mass/`、`docs/evidence/mixed-mass-solve/` | 本收口只引用其 verified 状态与证据目录 |

## 精度四类（Windows 侧）

| 类别 | 结论数字 | 证据 |
| --- | --- | --- |
| NP01 逐字段映射 | 191 行 = 字段 167 + 内核 19 + 编译选项 5；字段上游命中 **147/167**；类型偏离 **0**、未映射产品字段 **0**；内部布局字段 **20**（语义映射未做）；f64 命中 20 全部已分类（测试断言 2、诊断/读回校验 17、辅助校验 API 1）、未分类 0；编译选项差异 **1**（产品 `compute_70` vs 冻结上游设备架构 89）；缺口 10（未实现 7 + 部分覆盖 3） | `docs/development-evidence/np01-field-mapping/` |
| NP02 证据清单 | Windows 已实现路径 **88 条**（内核 84 + 冻结链 3 + MathDx 路线 1），有证据 **88/88（100%）、缺失 0**；源身份一致 77、已知漂移 **9**（用户 2026-10-24 裁决 B 归档，不重采不重基线）、派生 2；回执 701 份文件哈希全通过，PTX 实体在仓 650 份通过、51 份不在仓（历史捕获）；缺口 32 条（未实现 7、部分 3、deferred 4、未判定/不适用 5、部分缺失 2、已知漂移 9、派生 2） | `docs/development-evidence/np02-evidence/`（`3dcc1f2`+`147a922`+`c4c1058`） |
| NP03 Windows 数值等价 | 9 个 scope、**79,286 块、0 失败、0 待验收**；容差本收口未改（G01 `2e-5+2e-5|expected|` 等）；U038-3D 全链 `qfrc_gravcomp` **8/77 超 2e-6**（max 4.29e-6）按用户裁决 B 登记已知债务、未验收 | `docs/development-evidence/np03-windows-evidence/`（`0c4db70`） |
| NP04 Windows 性能 | **180 次运行全通过**（9 套件 ×5 样本 ×冷/热 ×debug/release）；稳态中位耗时 debug 4.90–17.56s、release 4.00–16.36s（release 快 3.5%–27.1%）；峰值显存增量 216–219 MiB、宿主峰值工作集 151.6–202.2 MiB；采样分辨率不足以细分套件差异（如实标注） | `docs/development-evidence/np04-windows-performance/`（`b0b4b8a`） |

Linux 侧：deferred（用户 2026-10-24 决定终止，等 Windows 全量完成后集中回测），不宣称双平台、不宣称 NP03/NP04 完整通过。

## 里程碑终验：G01 debug/release 双模式（最终 HEAD `c4c1058`）

| 阶段 | debug | release |
| --- | --- | --- |
| fmt | exit 0 | exit 0 |
| native | 27 过 | 27 过 |
| host-msvc | **835 过 0 败** | **835 过 0 败** |
| g01 | 7 过 | 7 过 |
| native-g01 | 2 过 | 2 过 |
| native-only-g01 | 1 过 | 1 过 |
| resident | 102 过 | 102 过 |
| all-probes | **353 过 0 败（87 result 行；11 项排除）** | **353 过 0 败（87 result 行；11 项排除）** |
| host-gnu | 480 过 | 480 过 |
| clippy-gnu / clippy-gnu-cuda / clippy-msvc | 0 警告 | 0 警告 |
| rustdoc（`-D warnings`） | 0 警告 | 0 警告 |
| diff | 通过 | 通过 |
| 全脚本墙钟 | **775.5s** | **641.1s** |

- 运行目录：debug `target/g01-acceptance/b9fb775e067e451daa99fbdaec33e922`、release `target/g01-acceptance/0c2d6e4355794126b65e0a383c09c7d5`；脚本 stdout：`target/g03-parity/g01-debug.log`、`g01-release.log`。
- 数值报告（两模式相同）：`completeStates=2084`、`completeScalars=7,704,548`、`maxAbsoluteError=1.25147918803026e-06`；`rawScalars=50,512`、`rawMaxAbsoluteError=5.55349298547725e-07`。
- 排除清单 **11 项、两模式逐项一致、本轮未新增**：W-D5 3（recapture×2 + archive×1）、历史捕获身份 6、env 未接线 1、U038-3D 已知债务 1；脚本写入 `probe-reports/exclusions.json`，显式排除而非静默跳过。
- `report.json` 的 `workingTreeDirty=true` 仅因为 `.normify/changes/2026-10-16-next-g03-parity.json` 处于 in_progress（任务登记状态），`src`/`tests`/`fixtures`/`Cargo.*` 未被改动（`unchanged` 门禁通过）。
- 未改 G01 脚本、排除清单、断言与容差。

## 未满足项（逐条）

1. **NP01 内部布局字段 20 个**（内部布局字段语义映射未做）；字段上游命中 147/167，其余按「内部字段/缺口」登记。
2. **NP02 已知漂移 9 条**（裁决 B 归档，不重采不重基线）；51 份历史捕获 PTX 实体不在仓（依赖 `target/`）；NP02 未实现子系统 7、部分覆盖 3。
3. **MathDx 再分发口径未定**（libmathdx 0.3.1 / pinned NVRTC 12.9.86 的再分发许可与 provenance 仍待用户裁决），不影响本地证据与运行。
4. **性能与完整上游等价未验收**：NP04 只有 Windows 单平台采样（GPU 非独占、分辨率有限）；无完整上游数值等价证明。
5. **产品固定 `compute_70` 与设备架构差异（设备 SM89）未评估**：产品按冻结配方编译 `compute_70`、Warp 冻结链为 arch 89，PTX 前向兼容可用，但架构级指令差异风险未单独评估。
6. **Linux deferred**：不宣称双平台；并入 `2026-10-24-linux-batch-retest`。
7. **U038-3D 已知债务**：`qfrc_gravcomp` 8/77 超 2e-6（max 4.29e-6），未验收、不宣称通过（§5.3 显式排除）。

## 复现

```powershell
cd D:\Rust\mjwarp-rs
scripts\test-windows-g01.ps1 -AllFeatures -FullRegression
scripts\test-windows-g01.ps1 -AllFeatures -FullRegression -Release
```

本轮两轮各执行一次（GPU 独占，调试约 12.9 分钟、release 约 10.7 分钟）。

## 边界

- 只对 Windows 单平台已覆盖范围下结论；不宣称双平台、不宣称完整上游等价、不宣称完整 g03-parity 全量通过。
- 未改产品物理配方/内核字符串/容差/断言/排除清单/G01 脚本；未改 `src`/`tests`/`fixtures`/`Cargo.*`。
- 本记录引用的是各 verified 交付与证据目录，不复用更早的旧结论冒充本轮实测。
