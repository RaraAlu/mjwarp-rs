# NP02 回执重算（2026-11-04，QA；WIP 记录，棘轮未改）

本目录记录 `2026-11-03-precision-np02-receipt-rehash` 的重算执行结果与发现。**棘轮（`inventory.json` 计数与
`tests/precision_evidence_gate.rs` 快照）尚未改写**——重算发现的计数漂移需要 owner 定夺后再同步，避免静默改发布口径。

## 1. 前置恢复（已完成，可复现）

| checkout | 期望 pin | 实测 `git rev-parse HEAD` |
| --- | --- | --- |
| `target/mjwarp-parallel/upstream` | `71da24d956378a87a703b6e1442b13aec0c4ac29` | 一致 |
| `target/research/warp` | `ffa038b1adf3927a8e893d4536f1a7562f17d749`（v1.15.0） | 一致 |

`target/research/warp` 由 `git clone --depth 1 --branch v1.15.0 https://github.com/NVIDIA/warp
target/research/warp` 恢复（约 5.8 MB，2026-11-04 04:2x，耗时 7.1 s）；两个 pin 均由工具强校验通过。

## 2. 重算执行

- 工具：`scripts/np02-evidence/np02-evidence.ps1`（**只读扫描**），临时 `-OutDir target/np02-rehash-out`
  （未直接覆盖已提交证据，便于先比对）。
- **工具脚本编码缺陷**：仓库内该脚本（及 `np02-lib.ps1`）为 UTF-8 **无 BOM** 且含中文，Windows PowerShell 5.1
  无法解析（`Unexpected token`），与 prepare 脚本同类问题。本次用同目录结构的 BOM 临时副本
  （`target/np02-tool/`，含 spec/lib）执行，**仓库脚本未改**；建议另立小卡做 ASCII/BOM 硬化。
- 输出：`回执索引 701 份，逐份文件哈希通过 701 份，异常 0 份`；`PTX 实体缺失 51 条`（与旧清单同值）。

## 3. 与已提交清单的差异（关键发现）

| 计数 | 已提交 inventory | 重算 | 说明 |
| --- | --- | --- | --- |
| windows_implemented_paths | 88 | **100** | +12 个新内核行（下表） |
| kernel_rows | 84 | **96** | 同上 |
| evidence_verified | 77 | 71 | 证据匹配口径变化所致 |
| evidence_missing | 0 | **12** | 新增 12 个内核无证据 |
| source_identity_drift | 9 | **15** | 快照后源码变更累积 |
| coverage_pct_evidence | 100.0 | **88** | 因 12 新内核无证据 |
| coverage_pct_source_identity | 87.5 | **71** | 同上 |
| gap_items | 32 | **50** | +18（12 缺失证据 + 漂移新增等） |

新增 12 个内核行（全部 `evidence=missing`）：`build_rays`、`mesh_bvh_ray`、`scene_bvh_ray`、`s7a_abi_probe`、
`aa_resolve`、`sample_texture2d`、`mjwarp_collision_{boxbox,contact,contact_multi,hfield,primitive,sdf}`
（来源：`src/render/{camera,bvh,kernel,texture}` 与 `src/physics/collision/cuda/product.cu`）。
这些是 **2026-10-24 清单生成之后新增的产品内核**，尚未采集 NP02 回执，属新缺口（`genuine-gap`/`缺失证据`）。

## 4. 51 份缺失 PTX 实体定性（逐条）

`missing-51-classification.json` 给出逐条：`entry`、`ptx sha256`、receipt 路径、期望实体路径、分类、以及同名
实体在仓内的其它位置。汇总：

- **47 条**：同名 `<entry>-<sha>.ptx` **存在于仓内其它目录**（多为 `docs/evidence/**` 历史采集目录）。它们不是
  “不在仓”，而是**回执旁没有兄弟实体**——可归为「should-commit/relocate（把实体放到回执旁或修回执指向）」
  或按历史证据处理，需 owner 定夺（直接复制会增仓体积并影响证据路径口径）。
- **4 条**：仓内任何位置都找不到同名 `.ptx` → `historical-target-artifact`（历史采集产物未入库，保留回执哈希即可）。

旧 README 的「51 份实体不在仓」表述**不准确**：其中 47 份实体在仓内其它目录。

## 5. 结论与待决

- 重跑本身成功：两 pin 一致、回执 701/701 哈希通过、缺失 51 条复现。
- **已提交的 NP02 棘轮数字已过期**：按重算结果改写会使覆盖率 100%→88%、缺口 32→50。
  这属「计数变化」，按卡 acceptance 需在提交说明写清来源；但**是否现在改写 `inventory.json` 与
  `tests/precision_evidence_gate.rs` 快照、以及 47 条实体如何处置（入库/改指向/降级为历史）**，需 owner 定夺。
  本 WIP 记录先入库，卡保持 in_progress。

## 6. 证据文件

| 文件 | 内容 |
| --- | --- |
| `inventory.new.json` | 重算生成的 inventory（未覆盖已提交版） |
| `gap.new.json` | 重算生成的 gap（50 条，含 12 条缺失证据） |
| `missing-51-classification.json` | 51 条缺失 PTX 的逐条分类与仓内其它位置 |
| `comparison.txt` | 新旧计数差异、新增行、gap 分类、结论摘要 |

## 7. 边界

零 GPU；`src/`、`fixtures/`、判据/容差/参考零改动；仓库脚本未改（BOM 副本仅存在于 `target/`）；不声称 NP02
通过或双平台等价。
