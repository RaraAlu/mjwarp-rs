# NP02 回执重算（2026-11-04，QA；已收口）

本目录记录 `2026-11-03-precision-np02-receipt-rehash` 的执行结果：恢复两个上游 pinned checkout、重跑回执/实体复核、51 份缺失实体逐条定性、工具可移植性缺陷修正、以及按结果同步 NP02 棘轮（`inventory.json` 计数与 `tests/precision_evidence_gate.rs` 快照）。旧清单（2026-10-24）原样副本存于 `../history/`，**不得覆盖**。

## 1. 前置恢复（已完成，可复现）

| checkout | 期望 pin | 实测 `git rev-parse HEAD` |
| --- | --- | --- |
| `target/mjwarp-parallel/upstream` | `71da24d956378a87a703b6e1442b13aec0c4ac29` | 一致 |
| `target/research/warp` | `ffa038b1adf3927a8e893d4536f1a7562f17d749`（v1.15.0） | 一致 |

`target/research/warp` 由 `git clone --depth 1 --branch v1.15.0 https://github.com/NVIDIA/warp target/research/warp` 恢复（约 5.8 MB，2026-11-04，耗时 7.1 s）；两个 pin 均由工具强校验通过。

## 2. 重算执行

- 工具：`scripts/np02-evidence/np02-evidence.ps1`（**只读扫描**），输出到 `target/` 下的临时目录，核对后再覆盖已提交清单。
- 运行环境：仓库没有 pinned PowerShell 7（仓瘦身删除了 `target/toolchains/powershell-7.6.7`），使用 Windows PowerShell 5.1；脚本是 UTF-8 无 BOM + 中文，PS 5.1 直接解析失败（`Unexpected token`），故用临时 BOM 副本执行（`target/np02-tool/`，执行后删除）。**仓库脚本在本次修正工具缺陷后仍是 UTF-8 无 BOM**，ASCII/BOM 硬化另立卡（`2026-11-04-prepare-scripts-bom-hardening-np02`）。
- 输出格式：PS 5.1 `ConvertTo-Json` 的缩进/分隔与仓库既有清单（PS 7 生成）不同，本次用等价序列化器把 `inventory.json` / `gap.json` 重排为仓库既有风格（2 空格、`": "`、CJK 不转义），并用旧 `inventory.json` 做 round-trip 校验：**旧文件经该序列化器可逐字节复现**，故仅格式等价重排、不改语义。工具原始输出保留在本目录 `inventory.new.json` / `gap.new.json`。

## 3. 工具可移植性缺陷（本次发现并修正）

`np02-evidence.ps1` 两处 `$x = if (…) { @(…) } else { @() }`：

- PowerShell 7：`if` 输出单元素数组时保持数组，`$x.Count` 正确；
- Windows PowerShell 5.1：单元素数组被解包为标量，`$x.Count` 为 `$null`。后果：
  1. 20 个「只有 1 份回执」的内核行 `artifact.receipts` / `artifact.t4_variants` 变 `null`；
  2. 6 个「参数化/拼接编译单元」（`camlight_com`、`camlight_rigid`、`passive_force_add_fluid`、`spatial_moment`、`spatial_site`、`spatial_wrap`）因 `$evidence.Count -gt 0` 为假而跳过「捕获 `.cu` 包含当前 const」回退，被误判 `source-drift`。

修正：改为 `$x = @(if (…) { … } else { @() })`（两种 PowerShell 下结果一致）。修正后重跑，旧 88 行与已提交清单**逐字段 diff 0 变化**。

**首轮 PS 5.1 结果作废**：首轮报 `verified 71 / drift 15 / gap 50`（`comparison.txt` 的 first-run 段），是上述缺陷产物；正确值为 `verified 77 / drift 9 / gap 44`。

## 4. 与已提交清单的差异（最终）

| 计数 | 已提交（2026-10-24） | 重算（2026-11-04） |
| --- | --- | --- |
| windows_implemented_paths | 88 | **100** |
| kernel_rows | 84 | **96** |
| evidence_verified | 77 | 77 |
| evidence_present | 88 | 88 |
| evidence_missing | 0 | **12** |
| source_identity_drift | 9 | 9 |
| coverage_pct_evidence | 100.0 | **88** |
| coverage_pct_source_identity | 87.5 | 77 |
| receipts_total / file_hash_ok | 701 / 701 | 701 / 701 |
| receipt_ptx_entities_verified | 650 | **697** |
| receipt_ptx_entities_missing | 51 | **4** |
| gap_items | 32 | **44** |

- 新增 12 行（全部 `evidence=missing`）：`build_rays`、`mesh_bvh_ray`、`scene_bvh_ray`、`s7a_abi_probe`、`aa_resolve`、`sample_texture2d`、`mjwarp_collision_{boxbox,contact,contact_multi,hfield,primitive,sdf}`（来源 `src/render/{camera,bvh,kernel,texture}` 与 `src/physics/collision/cuda/product.cu`；逐行见 `inventory.new.json`）。
- 旧 88 行：`evidence_state`、`source.sha256`、`evidence_paths` 等逐字段不变；仅 `artifact.ptx_verified` 在实体补回后按回执复核更新（`ptx_verified` 计数来自兄弟实体复核）。
- 缺口 44 = 缺失证据 12 + 已知漂移 9 + 未实现 7 + deferred 4 + 部分覆盖 3 + 部分缺失 2 + 未判定 2 + 派生 2 + 不适用 3。

## 5. 51 份缺失 PTX 实体处置（逐条完成）

`missing-51-classification.json` 逐条给出 `entry`、`ptxSha`、`receipt`、`expected_ptx`、分类、仓内其它位置、以及本次 `resolution`：

- **47 条 `should-commit-or-relocate` → `restored-to-expected-path`**：从仓内同名实体复制到回执旁的约定路径（`<entry>-<sha>.ptx`），复制前后 sha256 逐一复核等于回执 `ptxSha256`；总 1,073,875 B。逐条 `restored_from`、`bytes`、`sha256_verified` 见该 JSON，汇总见 `missing-capture-restore.json`。
  - 回执 JSON 无路径字段（工具按兄弟文件解析），所以「改回执指向」不可行；补回实体不是新增证据，只是把仓内已有同 sha 实体放回约定位置，不伪造任何内容。
- **4 条 `historical-target-artifact` → `historical-unrecoverable-filename`**：`small_prefix_factor_{3,4,5,6}`（同一 ptx sha `92561e5d…`），按文件名全仓检索 0 命中；保留回执哈希，不补造。
  - 如实记录：`small_prefix_factor_2` 的同 sha 实体在仓内存在（`layers-m3-resident-mixed-gather/capture-raw/small_prefix_factor_2-92561e5d….ptx`，哈希一致）。按 owner 2026-11-04 指令「4 条真缺标 historical-unrecoverable、不许补造」，**未**据 sha 等价恢复 3–6；是否恢复待 owner 决定。

## 6. 棘轮同步

- `../inventory.json`：按重算结果改写（100 路径 / 88 有证据 / 12 缺失 / 88% / 9 漂移 / 44 缺口 / 697 实体 / 4 缺失）。
- `../gap.json`：同步为 44 条。
- `tests/precision_evidence_gate.rs`：计数快照、`rows`/`gaps` 条数、指针下限、`evidence_state` 词表（补 `missing`，新增 12 行状态即 `missing`）同步；旧快照留 `../history/`。
- 本目录保留重算原始输出与逐条证据；`receipt.json` 记录命令、pin、两轮对比、修正前后数字与复算时间。

## 7. 证据文件

| 文件 | 内容 |
| --- | --- |
| `inventory.new.json` | 重算生成的 inventory（工具原始输出，最终轮：已修正工具缺陷） |
| `gap.new.json` | 重算生成的 gap（44 条，含 12 条缺失证据） |
| `missing-51-classification.json` | 51 条缺失 PTX 的逐条分类、仓内其它位置与本次处置 |
| `missing-capture-restore.json` | 47 条实体补回汇总（来源/目标/sha/字节） |
| `comparison.txt` | 新旧计数差异、两轮（作废/最终）对比、新增行、gap 分类 |
| `receipt.json` | 本次重算收口收据（命令、pin、计数、复算时间、边界） |
| `manifest.txt` | 本目录文件 sha256 + bytes（不含自身） |

## 8. 边界

零 GPU；`src/`、`fixtures/`、判据/容差/参考零改动；不重采参考；不声称 NP02 通过或双平台等价。工具脚本仅修正 2 处可移植性写法（PS 5.1/7 结果一致），未改扫描逻辑、规格、输入索引。
