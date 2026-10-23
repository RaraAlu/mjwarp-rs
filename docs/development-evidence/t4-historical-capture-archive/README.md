# T4 历史捕获归档（用户 2026-10-23 裁决 B）

任务：`2026-10-23-t4-historical-capture-archive`。
背景：用户 2026-10-23 对 T4 历史捕获债务裁决 **B + 归档**——维持 all-probes 的 10 项显式排除，
不重基线、不伪造；把 4 个历史捕获目录作为证据归档保存，并在
`docs/three-layer-acceptance.md` §5.2 登记为长期已知债务。排除清单
（`scripts/test-windows-g01.ps1` 的 `excludedAllProbes`/`excludedHistoricalAudits`/`excludedEnvUnwired`）保持现状不动。

本目录是该裁决的归档与可校验清单。归档只读取 `target/` 下历史捕获，未修改它们；
未改产品源码、测试、容差、fixtures 或脚本。

## 1. 归档内容

| 源目录（gitignored） | 文件数 | 原始字节 | 最新 mtime（UTC） | 目录树摘要（前缀） |
| --- | --- | --- | --- | --- |
| `target/t4-g03-retest` | 247 | 7 935 339 | 2026-10-12T01:45:48Z | `c736690127d53ceb…` |
| `target/t4-face-operators` | 194 | 11 719 026 | 2026-10-22T17:48:15Z | `bdf8d44f5fbd420e…` |
| `target/t4-final-face-compile` | 94 | 1 396 687 | 2026-10-13T17:41:48Z | `4a29d1c1d28f3e91…` |
| `target/t4-rigid-recapture` | 189 | 2 791 428 | 2026-10-12T16:23:56Z | `0e1f24b186ff0793…` |
| **合计** | **724** | **23 842 480** | — | — |

文件数与最新 mtime 与 QA 预检（`docs/development-evidence/t4-rebaseline-precheck/check.json` 的
`historicalInventory`/`historicalIntegrity`）逐项一致（247/194/94/189；mtime 到秒一致）。

归档文件：

| 文件 | 大小 | sha256 |
| --- | --- | --- |
| `t4-historical-captures.tar.gz` | 4 006 399 字节（3.82 MiB） | `95e7b430eebeb1986e2815faa5e2b566818baf89c91feeabc7b427059797a915` |
| `manifest.json` | 323 770 字节 | 逐文件 sha256/size/mtime + 目录级 treeSha256 清单（724 行） |
| `make-archive.ps1` | 归档生成与校验脚本（只读源目录） |

打包前后 `.git` 对象增长（归档 + 清单两个对象，含 zlib 压缩）为 **3 972 480 字节（3.79 MiB）**，
低于 15 MB 阈值，因此用单压缩包，未拆包、未裁剪内容。

## 2. provenance 与生成方式

- 4 个目录都是 `target/` 下的**运行时捕获产物**（未入库）：`t4-g03-retest` 为 10/12 的
  debug/release PTX 捕获；`t4-face-operators` 为 face 算子修复捕获（10/22 由 `d2d7b467` 重生成，
  `factorial.json` 的 `bit_differences=0`）；`t4-final-face-compile` 为 previous face 捕获；
  `t4-rigid-recapture` 为 rigid 重采集捕获。
- 它们与 git 历史的关系：已入库的**身份**在 `fixtures/precision/t4-kernels.json`、
  `fixtures/precision/t4-current-kernels.json` 与 `docs/evidence/t4-g03-retest/`；本归档是这些身份
  指向的**原始运行时文件**。按预检结论（`docs/development-evidence/t4-rebaseline-precheck/README.md`），
  当前源码身份已前进，现有工具无法就地重生成，且重生成需要重建历史模板锚点——因此按裁决 B
  保存原始捕获、不重基线。
- 生成：`make-archive.ps1` 先按源目录逐文件计算 sha256/size/mtime 生成 `manifest.json`，
  再 `tar -czf` 打包（bsdtar，保留目录结构与文件内容），最后解包回读并与清单逐文件比对。
  脚本只读 `target/`，只写本目录。
- 未包含任何密钥、环境变量或用户数据；均为工程捕获产物。

## 3. 校验（一条命令）

```powershell
docs\development-evidence\t4-historical-capture-archive\make-archive.ps1 -Verify
```

校验内容：归档大小与 sha256 与 `manifest.json` 一致；解包后 724 个文件的 sha256/size 与清单一致；
文件总数一致。输出 `verified: 724 files, archive 4006399 bytes, sha256 95e7b430…` 即通过，失败时非零退出。
`manifest.json` 中的 `mtimeUtc` 仅作溯源信息，校验以 sha256 与 size 为准。

## 4. 边界

- 这是**归档**，不是重基线：不改变任何历史捕获内容，不恢复被排除的 audit，不宣称 T4/G01 完整通过。
- 归档文件是二进制压缩包，位于 `docs/development-evidence/`（Normify size 豁免）；体积已如实记录，
  且提交前实测 `.git` 增长 3.79 MiB（< 15 MB）。
- 源目录 `target/` 仍保留原文件（gitignored）；归档是独立保留副本，可用于离线核对。
- 不覆盖 Linux、性能、精度验收等其它范围。
