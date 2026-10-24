# 工具链治理完整验收门禁证据（Windows 单平台判定）

任务：`2026-10-18-toolchain-clippy-acceptance`。日期：2026-10-24。执行人：QA。
最终判定 HEAD：`245c461`（并行 a10-p11 交付 `245c461` 落库后重跑；本任务未改 `src`/`tests`/`fixtures`/`Cargo.*`）。
范围：按用户 2026-10-24 决定 **Windows-only**；Linux 续接复核 deferred（并入 `2026-10-24-linux-batch-retest`），不参与判定、不宣称 Linux/双平台。

## 结论

- **Windows 可判条款 17/17 通过**（16 条原验收 + 1 条 Windows-only 重划条款）；四项门禁（fmt/clippy/clippy-cuda/unit）在最终 HEAD 全部通过，`unchanged`/`json` 通过。
- 实际诊断：默认配置 **0**（基线 24 行 / 23 位置）、`cuda-probe` **0**（基线 52）；71 项分类 39 修复 / 22 窄范围豁免 / 6 暂缓（后继清零）/ 4 范围外（后继清零）。
- 工具链锁定 1.99.0 与实际 `rustc`/`cargo`/`clippy` 一致；58 个脚本 `+stable` 覆盖 0 处。三个续接任务 + M1 补证均 verified。
- 真实产物核对：物理核心组合摘要 `d7f6c7d1…`、gravity 两包 `ba40ffcd…`/`95b6818d…`、T4 当前清单 `8a4b6e1f…` 均与重钉/冻结值一致；701 份回执逐份哈希通过、650 份仓内 PTX 实体逐份哈希通过。
- 未满足项全部为范围外/已 deferred（Linux、release 形态 clippy、NP01–NP04、MathDx 再分发、缓存来源、5 条继任证据未重登记、逐文件戳观察项），已在主文档逐条列明。

## 本目录文件

| 文件 | 内容 |
| --- | --- |
| `verification.json` | 机器记录：工具链与脚本覆盖、clippy 基线/当前、71 项分类、19 项复核与 50 条继任、身份锚、四项门禁、最终状态回归、17 条判定、缺口 |
| `toolchain-current.log` | `rust-toolchain.toml` 原文、`rustup show active-toolchain`、`rustc -Vv`、`cargo -V`、`clippy -V`、`rustfmt -V`、`Cargo.toml` `rust-version` |
| `anchors.log` | 物理核心组合摘要（LF-normalized 连接）、gravity 两包与 T4 清单原始字节摘要、`t4-current-kernels.json` 逐文件戳观察项 |

前序证据（均 verified 任务交付）：`../toolchain-clippy-governance/`、`../toolchain-evidence-revalidation/`、`../clippy-integration-tests/`、`../m1-bounds-evidence/`。
汇总文档：`docs/toolchain-clippy-acceptance.md`；回执：`docs/development-evidence/changes/2026-10-18-toolchain-clippy-acceptance.json`。
最终状态回归：`docs/development-evidence/g03-parity/`（G01 双模式 14/14 阶段 exit 0）。

## 门禁实测（最终 HEAD `245c461`）

| 门禁 | 结果 | 墙钟 |
| --- | --- | --- |
| fmt | 通过 | 1463ms |
| clippy（默认，全目标 `-D warnings`） | 通过，0 诊断 | 355ms（缓存命中；全量 5842ms） |
| clippy-cuda（`cuda-probe`，全目标 `-D warnings`） | 通过，0 诊断 | 355ms（缓存命中；全量 8460ms） |
| unit（`cargo test --locked`） | **485 过 0 败** | 2923ms（缓存命中；同内容全量 26356ms） |

## 边界

- 只做治理汇总验收，不转授产品精度、双平台或全量 T4 结论；NP01–NP04 仍未验收。
- 未改产品/测试/容差/fixtures；门禁 `unchanged` 保护。
- 未在本门禁重跑新鲜 GPU 摘要对比（无 GPU 资源）；最终状态 GPU 结论引用最终里程碑 G01（HEAD `c4c1058`，`all-probes` 353/0、11 项排除）与 repin-12 重钉值。
