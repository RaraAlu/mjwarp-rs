# 拆分前后 PTX 逐字节对照 · 证据

任务：`2026-10-22-split-ptx-identity`（工程治理 / 拆分验收，2026-10-23）。
验收：在拆分前基线 `360718f` 与拆分收口 `e9ae451` 各捕获全部产品配方的 PTX，记录配方数/文件数/摘要；
`report.json.identical` 必须为 true，有差异逐项列出且不改判通过；记录主机/GPU/驱动/CUDA/耗时/显存；
产品源码、原生、测试与依赖不变。

结论：**identical = true**。两侧各 **96 个配方**、**276 个 PTX 文件**、`6 270 230` 字节，
PTX 摘要同为 `32968ad63b539d64f136ca4e213270c63f67c0efbcf1ea9036a3d264af0df7a0`；
独立复验（按文件名与 sha256 逐文件比较）0 缺失、0 多余、0 位差。
唯一差异是 1 个测试探针的 `.cu` 注释含每次运行临时路径（PTX 相同），已如实记录、不影响 PTX 身份。

## 1. 参照提交

| | 拆分前基线 | 拆分收口 |
| --- | --- | --- |
| 提交 | `360718f521a2728f50e1c207c9d7a91d9a5a0eca` | `e9ae451b60218cf904e9dcae2f69f16b94cfed5a` |
| 标题 | `文档(layers): 评审W15原样复制内核来源` | `治理(重构): 收口超长文件拆分` |
| 时间 | 2026-10-21T19:29:34+08:00 | 2026-10-22T03:10:13+08:00 |
| tree | `c3a1663a278afd29113cd8a802e45429e3f50f57` | `1a2e938063baa7df4664e9883a3a4dfa2f436864` |

两提交间 `Cargo.toml`、`Cargo.lock`、`rust-toolchain.toml`、`build.rs` 完全一致（拆分只动源码/测试文件组织）。
基线是收口的祖先（`git merge-base --is-ancestor` 通过）。

## 2. 捕获口径

- 产品捕获层原样使用：`src/runtime/transfer/kernel.rs::record_physics_ptx`（拆分后为
  `src/runtime/transfer/kernel/ptx.rs`，函数体两提交逐字节相同），
  `MJWARP_PHYSICS_PTX_DIR` 作为**全新空捕获根**，每个编译的 f32 物理配方写
  `<entry>-<ptxSha256>.{cu,ptx,json}`。
- 套件：`cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features cuda-probe --no-fail-fast -- --ignored --test-threads=1 --nocapture`。
  - 两提交使用同一条命令、同一选择器、同一环境；`--no-fail-fast` 让已知失败的 ignored GPU 测试
    只进入 `testSummary`，不静默跳过，也不能中断捕获。
  - 用 `cuda-probe` 而非 `--all-features`：产品 f32 物理配方只在 cuda-probe 路径编译；
    `native-model-probe`/`cubecl-*` 探针不编译物理配方，纳入只会拉长构建、不增加配方覆盖。
    该口径写在 `report.json.command.featureScope`。
- 配方数 = 去重后的内核入口（entry）数；文件数 = `<entry>-<ptxSha256>.ptx` 变体数；
  摘要 = 对排序后的 `文件名|PTX sha256` 行取 sha256。每个 `.ptx` 都与同名 `.json`、`.cu` 交叉校验
  （文件名前缀、PTX sha、`precisionGate`、`fmad`/`fastUseMath`、CU 源 sha、NVRTC 版本）。

## 3. 方法（`scripts/compare-split-ptx.ps1`）

1. `git archive --format=tar <commit>` 导出两提交到 `target/split-ptx/<short>/src`，`tar` 解包；
   **不用 `git worktree`**，不 checkout，不污染 `.git/worktrees`、不改工作区。
2. 每提交独立 `CARGO_TARGET_DIR=target/split-ptx/<short>/target`；捕获根
   `target/split-ptx/<short>/ptx` 每次运行前清空重建。
3. 运行期间每 2 秒采样 `nvidia-smi memory.used` 记录峰值显存；记录耗时、日志与命令。
4. 逐文件比较两侧 PTX 文件名集合与 sha256；`identical` 还要求两侧无捕获异常、配方集合相同、
   PTX 摘要相同。存在差异时脚本非零退出并列出差异，不改判通过。
5. 只写 `target/split-ptx/` 与证据目录；仓库历史只读。

## 4. 环境

| 项 | 值 |
| --- | --- |
| 主机 | `DARKSEA` |
| OS | Microsoft Windows NT 10.0.26200.0 |
| CPU | AMD Ryzen 9 9950X3D 16-Core |
| GPU / 驱动 | NVIDIA GeForce RTX 4070 Ti SUPER / 596.36 / 16376 MiB |
| CUDA / NVRTC | `target/toolchains/cuda-12.8.1`；NVRTC DLL `6.14.11.9000`（捕获记录 NVRTC 12.8） |
| Rust | rustc 1.99.0 (b940084d7 2026-09-28)，cargo 1.99.0 |

## 5. 结果

| 项 | 基线 `360718f` | 收口 `e9ae451` |
| --- | --- | --- |
| 配方数（entry） | 96 | 96 |
| PTX 文件数 | 276 | 276 |
| PTX 总字节 | 6 270 230 | 6 270 230 |
| PTX 摘要 | `32968ad6…0df7a0` | `32968ad6…0df7a0` |
| 捕获异常 | 0 | 0 |
| 耗时 | 353.66 s | 357.40 s |
| 峰值显存 | 3062 MiB | 3068 MiB |
| ignored 套件 | 72 套件，235 过 42 败 | 72 套件，236 过 42 败 |
| 运行退出码 | 不使用（已知失败测试使其非零，见 `report.json` 说明） | 同左 |

比较：`missingInCurrent=0`、`extraInCurrent=0`、`changedPtx=0`、`ptxDigestsEqual=true`，
**`identical=true`**。独立复验：基线 276 文件、收口 276 文件、缺失 0、多余 0、位差 0。

差异清单（完整记录在 `report.json.comparison.changedSources`）：

- `baseline_probe-1f6a35c2…e92d.ptx`：两侧 PTX 字节相同，但 `.cu` 源字节不同。
  该条目是 launcher-baseline 测量探针，源注释里嵌入每次运行的临时目录
  （`%TEMP%\mjwarp-compile-baseline-gpu-<pid>-<seq>`），属测试探针噪声；
  `sourceDigestsEqual=false` 只反映这一条，**不影响 PTX 身份判定**。
- 除此之外 275 个 `.cu` 源与全部 276 个 PTX 逐字节相同。

ignored 套件的 42 个失败是既有的已知失败/依赖外部证据目录的测试（与 PTX 捕获无关），
两侧数量一致（42 vs 42），只记录、不隐藏、不用来判定配方身份。

## 6. 复现

```powershell
# 完整重跑（导出 + 独立构建 + 两次捕获 + 比较，写 report.json）
scripts\compare-split-ptx.ps1

# 已有捕获与 run-meta 时只重做比较/报告（本次证据即先完整重跑、后修正报告字段复算）
scripts\compare-split-ptx.ps1 -ReuseRuns
```

参数：`-Baseline`/`-Current`（默认上表两提交）、`-Features`（默认 `cuda-probe`）、
`-WorkRoot`（默认 `target/split-ptx`）、`-EvidenceDir`（默认本目录）。完整运行日志与捕获文件在
`target/split-ptx/<short>/`（gitignored，可按上命令重生成）。

## 7. 边界

- 只证明：在本机、本 CUDA/NVRTC、本 `cuda-probe` ignored 套件下，`360718f → e9ae451`
  的源码拆分未改变捕获到的全部产品配方 PTX 字节。
- 不覆盖 Linux、性能、release、NP01–NP04 等精度验收，也不替代完整上游等价；ignored 套件中的
  既有失败按原样记录。
- 不改产品源码/原生/测试/Cargo 清单；脚本与证据之外无仓库改动。

## 8. 文件

- `scripts/compare-split-ptx.ps1`：一条命令可复现的对照工具（只读仓库历史，独立 target 目录）。
- `docs/development-evidence/split-ptx-identity/report.json`：机器可判定报告（`identical: true`）。
- `docs/development-evidence/split-ptx-identity/README.md`：本证据说明。
- 运行日志/捕获产物：`target/split-ptx/{360718f,e9ae451}/`（gitignored）。
