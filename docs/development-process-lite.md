# 精简开发流程（L0 / L1 / L2）

日期：2026-11-12。状态：生效（用户已批准全量采用精简流程）。
本文件只定流程，不放宽任何数值/验收口径。与 `AGENTS.md`、`docs/numerical-precision-contract.md` 冲突时，以后者为准。

## 1. 三级流程

| 级别 | 适用 | 提交数 | 门禁 | 证据 | 复核 |
| --- | --- | --- | --- | --- | --- |
| **L0 轻量** | 文档、勘误、治理、注释 | **1 个提交** | 不跑 | 不建证据目录 | 不做 QA |
| **L1 标准** | 常规代码/测试、定义类产物 | 1 卡 **≤2 个提交**（登记 + 工作/收口） | 单提交跑 `fmt + clippy`；整批收口时跑一次 `clippy-cuda + unit` | 只留 README + 测试/分析日志 + 产物 JSON；**不要** manifest、逐文件 sha、policy 扫描 | 每卡**复核一次** |
| **L2 重型** | 判据数字、GPU 窗口、验收、冻结集、ABI、跨设备声明 | 沿用现有全套 | 四门禁 + 全套 | manifest、逐文件 sha、policy_scan、receipt | QA 逐字节复核 + 签名 |

**升级规则（自动）**：任何改动一旦触碰 **判据 / 容差 / 参考 / ABI / 冻结证据 / 对外声明**，自动升 L2，不得按 L0/L1 走。
**降级规则**：L2 片内的纯文字勘误，若不改语义，可由 QA 判为 L0 子提交并入该片。

L0 提交说明必须写清「改了什么、为什么、影响面」；不需要卡、不需要证据目录。

> **L0 提交引用**：L0 docs 提交统一引用常驻卡 **`2026-11-12-process-lite-standing-v2`**（`schedule.files` 为显式 L0 文档路径，不建证据、不排期、不占窗口）作为 `Normify-Task` 尾部字段；**禁止 `--no-verify`**。v1（`2026-11-12-process-lite-standing`）因 `unchanged src/...` 检查与 P1 代码片冲突、且运行中冻结无法收窄，已 abandon（`93d9a033b`）。历史两次一次性 `--no-verify` 提交（`5f9c1bb9`、`117d2299`）保留、不重写。长期工具白名单方案见 errata E-009（需用户批）。

## 2. 复核一次 + errata 台账

- 每张 L1 卡**只复核一次**；复核结论只分「阻断 / 一般 / 低 / 观察」。
- **阻断 / 一般**：当场退回返工，不进台账。
- **低项 / 观察**：不返工，登记到 `docs/development-evidence/process/errata-ledger.md`（OPEN → 批次 → DONE）。
- 修复时机：**每批收口或开窗前批量修一次**，不逐条打断。
- 台账每行必须有：来源（卡/复核 ID）、等级、状态、批次；修完注明提交号。

## 3. append-only 的边界

- **只用于已冻结的历史证据**（已 verified 的 L2 证据、已归档快照）。
- 新文件、未冻结证据：**允许直接改**，不必 addenda。
- 已冻结证据确实要改：append-only（addenda 或新版本文件），不改历史字节，manifest 重算。

## 4. QA 风险抽查

- **必查**：判据数字、断言、边界条件、冻结件身份（sha/字节）。
- **其余按 30% 抽样**；抽样面覆盖：证据与文档一致性、命令可复跑、声明用语。
- 只有发现**系统性问题**（同一错误重复 ≥3 处或影响判据结论）才升级为全量复核。
- 抽查结论同样只记一次，低项进台账。

## 5. 提交纪律

| 级别 | 提交形态 |
| --- | --- |
| L0 | 1 个提交，显式 pathspec，只加本次文件；`Normify-Task` 引用常驻卡 `2026-11-12-process-lite-standing-v2` |
| L1 | 1 卡 = 登记提交（卡 JSON，proposed→start）+ 工作/收口提交（内容 + 证据），共 2 个 |
| L2 | 沿用现有（登记 / 内容 / fix 等），两段式 |

`normify fix` **只用于已 verified 的 L2 卡**；L1 卡未 verified 前直接改，verified 后若确需修正按台账批次统一处理。

## 6. 自动化

| 脚本 | 作用 | 接口（草案，以 Backend 实现为准） |
| --- | --- | --- |
| `scripts/process/slice.ps1` | L0/L1 片的开/收：登记、收口、跑门禁、打印待办 | `-Action register\|close\|status -Id <card> -Kind L0\|L1 [-Reason <text>]` |
| `scripts/process/review.ps1` | L1 复核与台账：一次复核、30% 抽样、把低项写入台账 | `-Action review\|ledger -Id <card> -Sample 0.3 [-Ledger docs/development-evidence/process/errata-ledger.md]` |

- 脚本未就绪时按本文件手工执行；脚本落地后以 `-Help` 输出为准，接口变更需同步改本节。

## 7. 不精简项（任何级别都不放宽）

1. 判据 / 容差 / 参考的**签名级**变更：必须用户拍板，不得流程化。
2. GPU 窗口**四前提**：单窗互斥、monitor 60/60、三点基线、floor 取严。
3. 冻结集身份：sha/字节/重冻清单，一律按 L2。
4. 仓库**锁单写者**：写仓前 `repo-write-lock` 取锁，只 add 自己的文件。
5. 「不宣称验收通过 / 不宣称与上游等价」类声明纪律。

## 8. 台账

初始条目见 `docs/development-evidence/process/errata-ledger.md`（O2/O3/O4/O5、G.5/G.6 双规则、L4 等，均 OPEN）。
新低项由复核人登记；每批收口前由 Arch 汇总、派修、更新状态。
