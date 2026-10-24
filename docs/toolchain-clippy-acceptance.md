# 工具链治理完整验收门禁（Windows 单平台判定）

日期：2026-10-24。任务：`2026-10-18-toolchain-clippy-acceptance`。执行人：QA。
最终判定 HEAD：`245c461`（并行 a10-p11 交付落库后的干净树；`unchanged` 门禁覆盖 `src`/`tests`/`fixtures`/`Cargo.*`，本任务未改这些路径）。
平台：Windows 11、RTX 4070 Ti SUPER、`rustc 1.99.0 (b940084d7 2026-09-28)`。
边界：按用户 2026-10-24 决定 **Windows-only**；Linux 续接复核 deferred（并入 `2026-10-24-linux-batch-retest`），不参与本门禁判定，**不宣称 Linux 或双平台通过**。

## 结论（先给）

- **Windows 可判条款 17/17 通过**（16 条原验收 + 1 条 Windows-only 重划条款）；门禁四项 `fmt` / `clippy` / `clippy-cuda` / `unit` 在最终 HEAD `245c461` 全部通过，`unchanged` 与 `json` 检查通过；`245c461` 为并行 a10-p11 交付，落库后本门禁重跑干净树。
- 实际诊断数：**默认配置 0 处、`cuda-probe` 配置 0 处**（基线分别为 24 行/23 个位置、52 处）；71 项分类仍为 39 修复 / 22 窄范围豁免 / 6 暂缓（后继清零）/ 4 范围外（后继清零）。
- 三个续接任务（governance、evidence-revalidation、clippy-integration-tests）与 M1 补证任务均 verified；`+stable` 覆盖 0 处；锁定工具链 1.99.0 与实际 rustc/cargo/clippy 一致。
- 真实产物核对：物理核心组合摘要 `d7f6c7d1…` 与 repin-12 重钉值一致；gravity 两包 `ba40ffcd…`/`95b6818d…`、T4 当前清单 `8a4b6e1f…` 均一致；701 份内核回执逐份哈希通过、650 份仓内 PTX 实体逐份哈希通过（NP02 扫描器当前 HEAD 只读复跑）。
- 未满足项全部为**范围外/已 deferred**（Linux、release 形态 clippy、NP01–NP04、MathDx 再分发、缓存来源、5 条证据未重新登记），已逐条列明；本门禁只做治理汇总验收，**不转授产品精度/双平台/全量 T4 结论**。

## 17 条逐条判定（Windows 可判）

| # | 条款要点 | 判定 | 证据与数字 |
| --- | --- | --- | --- |
| 1 | 记录实际工具链与诊断基线；选明确版本锁定；不靠降级；不改宿主目标/平台边界 | 通过 | 基线 `baseline-toolchain.log`、`baseline-clippy-default.json`（23 位置/24 行）、`baseline-clippy-cuda-probe.json`（52）；锁定 1.99.0 = 原 `stable` 实际版本；rustup 默认宿主 gnu 未改，G01 脚本显式 msvc 未改 |
| 2 | 加 `rust-toolchain.toml`；核对脚本 `+stable` 覆盖；验证脚本实际选用版本；`rust-version` 只声明最低版本 | 通过 | `rust-toolchain.toml` `channel = "1.99.0"` + `rustfmt`/`clippy`；`Cargo.toml` `rust-version = "1.99"`；58 个脚本中 `+stable` **0** 处、`+1.99.0-…-msvc` 字面量 16 个、`+$toolchain` 2 个（g01-fast/compare-split-ptx）、3 个 authoring/比对脚本直接 `cargo`（遵循 `rust-toolchain.toml`）；实际 `rustc 1.99.0`/`cargo 1.99.0`/`clippy 0.1.99`；Linux 部分 deferred 不判定 |
| 3 | 按锁定工具链修复诊断；只改宿主逻辑与测试；保留所有权/别名/整数边界/浮点位型；不批量屏蔽/删测试 | 通过 | 39 项修复 + 22 项窄范围豁免（单条/单循环/单测试）见 `diagnostics.json`；当前两配置 clippy 0；`unchanged` 保护 `src`/`tests` 不被本门禁改动 |
| 4 | 默认与 cuda-probe 的 clippy 全目标；保留 `-D warnings`；记录真实诊断数与未覆盖配置 | 通过 | `cargo clippy --all-targets --locked -- -D warnings` 与 `--features cuda-probe` **各 exit 0 / 0 诊断**（5842ms/8460ms）；未覆盖：release 形态 clippy、Linux |
| 5 | 重跑默认库/M1/受影响宿主测试；影响物理输入或调度的改动跑相关 GPU 验证；资源独占 | 通过 | 本门禁 `unit` 门禁 485 过 0 败（2923ms 缓存命中；同内容全量 26356ms）；GPU 侧由 governance/revalidation 的 68 程序/279 内核摘要前后一致，及最终里程碑 G01 `all-probes` 353 过 0 败覆盖（GPU 独占） |
| 6 | 逐项对比实际生成内核源码摘要；固定配方与工具链对比 PTX；保留原静态产物；不能只证文件未改 | 通过（附观察项） | 物理核心组合摘要 `d7f6c7d1…` 当前 HEAD 实测一致；gravity nonzero/three-d 包 `ba40ffcd…`/`95b6818d…` 原始字节一致；T4 当前清单 `8a4b6e1f…` 一致；701 回执 + 650 仓内 PTX 实体逐份哈希通过（`np02-evidence.ps1 -Check`）；静态产物未改（`unchanged`）。观察项：`fixtures/precision/t4-current-kernels.json` 的 `src/physics/mod.rs` 条目在 repin-12 被写成**组合摘要**而非该文件摘要，仅被 all-probes 显式排除的 recapture 工具消费，不影响 G01 身份锚与 PTX 实体核对 |
| 7 | 内核字符串/编译选项/精度保持；不放宽容差；漂移先诊断 | 通过 | `unchanged`（src/tests/fixtures/Cargo.*）；容差字面量仍在（`2e-5` 76 行、`2e-4` 30 行、`2e-6` 28 行；RNE/G01/G03/T4/G26/U038 口径与文档一致）；本轮无容差改动 |
| 8 | 交付诊断清单、测试日志、逐文件回执；列明 Linux/release/NP01–NP04 缺口；不把治理当全产品验收 | 通过 | 三份前序证据目录（diagnostics/日志/回执）+ 本门禁证据；缺口逐条列在本文末尾 |
| 9 | `fluid_force.rs` 四项 `excessive_precision` 只允许窄范围豁免；保留字面量与位型断言；不得全文件屏蔽 | 通过 | 5 处函数/常量级 `#[allow(clippy::excessive_precision)]`（测试函数 1 + 常量条目 4，覆盖 8 个字面量），**无文件级 allow**；字面量与位型断言未改（`unchanged` + 当前 clippy 0） |
| 10 | 默认基线 24 项、cuda-probe 52 项；逐项注明修复或豁免；新增 6 项来源引用第 2 项报告 | 通过 | `baseline-clippy-*` 23 位置（24 行）/52；71 项 = 39/22/6/4 逐项 `diagnostics.json`；6 项暂缓与 4 项范围外由后继任务清零，来源引用 governance 第 2 项报告与 revalidation/集成任务 |
| 11 | 前十条原文承接；不得当已通过；逐条关联证据与源码版本；任一未满足不得关门禁 | 通过 | 本表逐条关联证据；未满足项均为范围外/deferred，Windows 可判条款无未满足项 |
| 12 | 三个续接任务须 verified；重新核对最终组合源码；历史证据不自动转授；源码变化补验证 | 通过 | governance/evidence-revalidation/clippy-integration-tests 与 m1-bounds-evidence 均 verified；最终组合源码在本 HEAD 重跑 clippy/unit 与 PTX 实体核对；后续合法交付由 G03 里程碑 G01 14/14 覆盖 |
| 13 | 默认与 cuda-probe 全目标 clippy 必须通过；四项集成诊断全部覆盖；不得缩小目标或降级 | 通过 | 两配置 `--all-targets -D warnings` exit 0；集成测试诊断见 `clippy-integration-tests/validation.json`（0 诊断）并在本 HEAD 复验通过 |
| 14 | 核对六项暂缓真实处置、19 项宿主失败复核、50 条证据继任链、最终状态 GPU 比较 | 通过 | 六项：revalidation 表逐项修复（当前 clippy 0）；19 项：`review-19.json` 19 行全部处置，当前 `unit` 485/0、里程碑 host-msvc 835/0，缓存项 `t4_hessian_generation::checks_cached_artifact_boundary_and_rejects_forgery` 本 HEAD 实测 ok；50 条：45 已复验 + 5 未复验（当时既有 GPU 失败，现 `all-probes` 353/0 中对应测试通过）；最终 GPU 比较取最终里程碑 G01（HEAD `c4c1058`）14/14 |
| 15 | 保留原精度硬指标、全部旧阶段门禁、未验收 release/产品边界；不自动完成 NP01–NP04 | 通过 | 容差口径保留（见 #7）；G01 14 阶段脚本自 `45baa0e` 未改，`gates.toml` 的 g26-energy/g01-full/g01-full-release 均在；NP01–NP04 与 release 边界保留 |
| 16 | 本门禁只做汇总验收；实现各走独立交付；使用独立审查报告与回执 | 通过 | 实现与证据分属 governance/revalidation/integration/m1 任务；本门禁只新增汇总文档、证据与回执 |
| 17 | Windows-only 重划：Linux 续接 deferred、不参与判定；列明 Linux 缺口、不宣称 Linux/双平台 | 通过 | 本节与“缺口”节列明 Linux deferred；无 Linux/双平台结论 |

## 门禁实测（最终 HEAD `245c461`）

| 门禁 | 命令 | 结果 | 墙钟 |
| --- | --- | --- | --- |
| fmt | `cargo fmt --check` | exit 0 | 1211ms |
| clippy | `cargo clippy --all-targets --locked -- -D warnings` | exit 0，0 诊断 | 355ms（缓存命中；全量 5842ms） |
| clippy-cuda | `cargo clippy --all-targets --locked --features cuda-probe -- -D warnings` | exit 0，0 诊断 | 355ms（缓存命中；全量 8460ms） |
| unit | `cargo test --locked` | **485 过 0 败**（a10-p11 新增 5 项单测后） | 2923ms（缓存命中；同内容全量跑 26356ms） |
| unchanged | `src`/`tests`/`fixtures`/`Cargo.toml`/`Cargo.lock` | 通过 | — |
| json | 回执 `/task_id` = `2026-10-18-toolchain-clippy-acceptance` | 通过 | — |

回归背景（不作为本门禁新结论，只证明治理结果在最终状态未被后续交付破坏）：G03 里程碑在 HEAD `c4c1058` 双模式 14/14 阶段 exit 0，host-msvc 835 过 0 败、all-probes 353 过 0 败（87 result 行、11 项排除）、rustdoc 0。`4e07852` 之后有并行 a10-p11 交付 `245c461`（新增 5 项单测）；本门禁在其落库后的干净树上重跑全部通过，`unchanged` 保持成立。

## 缺口与未满足项（逐条）

1. **Linux deferred**：用户 2026-10-24 决定终止 Linux，续接复核并入 `2026-10-24-linux-batch-retest`；本门禁不判定、不宣称 Linux/双平台。
2. **release 形态 clippy**：治理时即列为未覆盖；本门禁只跑默认与 `cuda-probe` 的 clippy 全目标，release clippy 未运行（G01 release 覆盖构建/测试，不等价于 release clippy 门禁）。
3. **NP01–NP04 未验收**：本门禁不自动完成；各自缺口见 `np01-field-mapping`/`np02-evidence`/`np03-windows-evidence`/`np04-windows-performance`。
4. **50 条证据继任链中 5 条**未在 revalidation 时复验（当时既有 GPU 失败）；当前 `all-probes` 353/0 显示对应测试通过，但未重新登记继任证据（历史证据保持原样）。
5. **缓存集合来源未查清**：`t4_hessian_generation` 缓存来源未知的遗留项；该测试当前通过，来源问题未解。
6. **`t4-current-kernels.json` 逐文件戳异常**（观察项，见 #6）：`src/physics/mod.rs` 条目存的是组合摘要；仅被排除的 recapture 工具消费，未修（`fixtures` 受 `unchanged` 保护）。
7. **MathDx 再分发口径**未裁决（不影响本地证据与运行）。
8. 本门禁**未重跑新鲜 GPU 摘要对比**（无 GPU 资源）；最终状态 GPU 结论引用最终里程碑 G01 `all-probes` 353/0 与 repin-12 重钉值。

## 证据索引

- `docs/development-evidence/toolchain-clippy-acceptance/`：`README.md`（汇总）、`verification.json`（机器记录：工具链、基线/当前诊断、71 项分类、19 项与 50 条核对、身份锚、门禁）、`toolchain-current.log`（版本与锁文件）、`anchors.log`（身份锚实测）。
- 前序证据：`toolchain-clippy-governance/`、`toolchain-evidence-revalidation/`、`clippy-integration-tests/`、`m1-bounds-evidence/`（均含逐项诊断/日志/回执）。
- 回执：`docs/development-evidence/changes/2026-10-18-toolchain-clippy-acceptance.json`（含 `task_id`）。
- 最终状态回归：`docs/development-evidence/g03-parity/`（G01 双模式 14/14）与 `docs/three-layer-acceptance.md` §4.13。

## 复现

```powershell
cd D:\Rust\mjwarp-rs
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo clippy --all-targets --locked --features cuda-probe -- -D warnings
cargo test --locked
```
