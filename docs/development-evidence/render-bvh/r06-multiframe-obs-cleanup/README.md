# R06 多帧观察项治理（O2 + O6；仅证据层、CPU-only）

- 任务：`2026-11-06-render-r06-multiframe-obs-cleanup`（owner Backend、p60、P8-渲染与BVH）
- 来源：Arch 决策包零风险项（QA 实体复核 `8c9e7192` 的观察 O2/O6）
- 边界：**仅证据层**（receipt/README/manifest）+ 卡记录；不碰 tests、freeze 脚本、guards JSON、fixtures、spec；G-3 已冻结字节不变

## 1. 改动（逐文件前后 raw sha256/bytes 见 `artifacts/changed-files.json`）

| 文件 | 改动 |
| --- | --- |
| `r06-flex-multiframe-entity-fix/receipt.json` | `option_a.ruling_trace` 路径文案：`receipt.json.criteria.option_a.chief_ruling` → **`receipt.json.option_a.chief_ruling`**（修复卡 receipt 为**顶层 `option_a`**；`criteria.option_a` 只存在于实体卡 receipt） |
| `r06-flex-multiframe-entity-fix/README.md` | §4 同一路径文案改为顶层 `option_a`，并注明追溯口径 |
| `r06-flex-multiframe-entity/receipt.json` | `claims.statement` 去除相邻的「上游」「等价」组合词，改为「不构成与上游实现的一致性结论」；其余语义（不声明 R06 整族通过、动画通用性、不做验证窗结论）保持不变 |
| `r06-flex-multiframe-entity/manifest.txt`、`...-entity-fix/manifest.txt` | 重算自核（实体 **8/8**、小修 **6/6**） |

## 2. 冻结字节不变

- `scripts/validate/render-bvh/guards-r06-multiframe.json`：raw sha256 `64bfecdfd179c4b7160c4f853660671ec9fb570b4d6c7702aa6c35c2c3cb352d` / **11,899 B** 前后一致；`git status --porcelain -- <guards>` 为空（见 `artifacts/changed-files.json` 的 `guards` 段）。
- 说明：该文件未被 `.gitattributes` 的 `-text` 覆盖，HEAD blob 与 raw 工作树字节口径不同（QA O1）；本卡以 **raw 字节前后一致 + git 无改动** 证明不变。

## 3. 边界与自核

- `unchanged tests/, fixtures/, scripts/, docs/.../r06-flex-multiframe-criterion/, docs/.../r06-flex-multiframe-verify-g3/, docs/.../r06-flex-multiframe-capture-g2/`（normify checks）。
- 两目录 manifest 重算自核通过（`logs/manifest-recompute.log`）；UTF-8/LF 无 BOM、manifest 首行 `#`、裸 CR/TAB 自查通过。
- 其余 O1/O3/O4/O5 与 guards eol **不做**（等下一次验证窗前随重冻一起动）。
- 四门禁未跑（无代码改动，卡面 checks 不含 gate）。

## 4. 非声明

本卡只做证据层文案清理，不产生新验收结论；不写 R06 整族通过 / 动画一致性全广度覆盖 / 与上游实现的一致性结论 / 发布依据；不重开任何已 verified 判定。

## 5. 证据清单

| 路径 | 说明 |
| --- | --- |
| `README.md` | 本文件 |
| `receipt.json` | 机器可读回执（`/task_id` 供 normify `json` 检查） |
| `artifacts/changed-files.json` | 5 个改动文件与 guards 的前后 raw sha256/bytes、git status |
| `logs/manifest-recompute.log` | 两目录 manifest 重算自核与编码卫生 |
| `manifest.txt` | 本目录 sha256 + bytes（首行 `#`，不自列） |

## 6. 复现

```powershell
git diff --stat HEAD -- docs/development-evidence/render-bvh/r06-flex-multiframe-entity/ docs/development-evidence/render-bvh/r06-flex-multiframe-entity-fix/
target/mjwarp-parallel/venv/Scripts/python.exe target/r06-multiframe-entity-fix/gen_manifest.py "docs/development-evidence/render-bvh/r06-flex-multiframe-entity|r06-flex-multiframe-entity" "docs/development-evidence/render-bvh/r06-flex-multiframe-entity-fix|r06-flex-multiframe-entity-fix"
```
