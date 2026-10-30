# U4 SDF 验证窗材料（条件式例外 amend）独立复核（QA，只读）

任务：`2026-10-31-render-sdf-validate-window-material-check`
被核：`2026-10-31-render-sdf-validate-window-material`（材料卡 verified）
被核 pin：**`285ba51f`**（amend：用户条件式例外 + 92 分解对账 + body 零编译口径）；核 start 时 `revision.before = ca88e04f`。
**结论：pass / 阻断 0 / 需修 4（全部证据/措辞级，不重跑、不跑 GPU）/ 观察 6。**

## 0. 一句话

材料把用户**条件式路径**（先常规 `util<40`，失败才启用已签 `validation_only util<60` 例外）写清楚了：9 实体定稿、`9 exe + 83 inputs = 92` 分解与 S3 的 68 对账（+10 scene-sdf、legacy 42→47）、判据零放宽、双签三件（user only）、不复用旧 guards、`aggregate` 冻结时生成、shell 固定 PowerShell 5.1、body 零编译口径都已落地。**92 分解我用冻结脚本的同一选取规则在现盘独立数了一遍：10/10/47/16 = 83 ✓**。4 条需修都是证据/措辞问题：receipt 自引用过期、签字时间占位 `00:5x`、README 标题仍写「无例外」、「5 条命令逐字」与实际 4 条+条件式命令未逐字。

## 1. 复核方法

- 全部数字由 `git cat-file` 读 committed blob 现算；92 的 83 inputs 用**冻结脚本同样的目录规则**在 `fixtures/render-bvh` 现盘独立统计。
- 重算脚本：`recompute.py`（11 项 check → `recompute.json`，problems 4）。

## 2. 结果（11 项 check，7 过 / 4 需修）

| # | check | 结论 | 关键实测 |
| --- | --- | --- | --- |
| 1 | `scope.amend_only_material` | ✓ | amend 只动材料 5 文件 + 卡；`entity-table.json` 与 amend 前**逐字节相同**；无 `src/tests/scripts/fixtures/Cargo` 改动 |
| 2 | `entities.final_selection` | ✓ | 9 实体 = `render_sdf_reference`（new）+ 8 回归，顺序与材料一致；**移出** `render_camera_rays`/`render_r08_cameras`，**换入** `render_mesh_bvh`，`render_context_lifecycle` 保留 |
| 3 | `guards.92_and_s3_reconciliation` | ✓ | `9 exe + 83 inputs = 92`；分解 `10 + 10 + 47 + 16 = 83`；与 S3 的 68 inputs 对账 `+10 scene-sdf、legacy 42→47` 写明；`aggregate` 明确「freeze 时生成、材料不预设」 |
| 4 | `commands.five_verbatim_as_claimed` | ✗（P4） | 材料声明「5 条命令逐字」，window-plan 实际 **4 条**完整 `powershell` 命令（monitor/freeze/guards-only/body）；条件式例外的 monitor 只描述 `-MaxUtil 60` + 新 CSV，**没有完整命令行/CSV 名** |
| 5 | `criteria.zero_relaxation` | ✓ | rgb ≤1 LSB（`beyond_1_lsb=0`、`max_byte_diff<=1`）、depth 位级 `mismatched=0` + `max_ulp≤4` + `beyond_ulp=0`、seg exact、两跑 sha、shadow 臂仅一致性、DepthSeg 跨路径位级；**与 amend 前逐字相同** |
| 6 | `exception.conditional_protocol` | ✓ | `validation_only_exception_util_lt_60`、`approved_by=user via Chief`、`authoring_exception_used=false`、五要素（mode/approved_by/reason/original attempts retained/`not_a_normal_threshold_pass=true`）、失败记录保留 + 不得写成常规通过 |
| 7 | `signature.user_and_artifacts` | ✗（P2） | `approved_by=user via Chief`、三件签字对象（窗口计划 / 基线 / 92 条 guards）齐全；但时间写成 **`2026-10-31T00:5x+08:00`**（4 个文件共 **10 处**占位），不是具体时间戳 |
| 8 | `discipline.shell_regular_zero_compile` | ✓ | 所有命令用 `powershell`（5.1）；常规路径 `default MaxUtil=40`、`do NOT pass -OpenUtilLimit or -MaxUtil 60`；body 零编译口径写明（freeze 的 `cargo test --no-run` 属预期构建，deps 快照取在 body 前） |
| 9 | `evidence.manifest_and_self_entry` | ✗（P1） | manifest **5/5** 行全等、6 文件、window-plan ASCII 0、无 UTF-16/BOM；但 `receipt.files.receipt.json` 自条目仍是旧的 `4ef24623…/2078 B/53 行`，实际 `87631dba…/3825 B` |
| 10 | `readme.heading_consistency` | ✗（P3） | README 有 §8 签字章节，但 §2 标题仍是「窗口纪律（**常规 util<40，无例外**）」，与该节条件式内容矛盾 |
| 11 | `card.checks_and_validate` | ✓ | 主卡 `normify check` 通过；`validate --fingerprints` = 0 错误 / 5 警告 |

### 92 分解的独立核对（现盘，冻结脚本同一规则）

| 组 | 文件数 | 期望 |
| --- | --- | --- |
| scene-sdf（2 XML + 8 采集产物） | 10 | 10 ✓ |
| scene-hfield（2 XML + 8 capture） | 10 | 10 ✓ |
| legacy（排除 scene-shadow/hfield/sdf，含 4 skybox + 5 perworld） | 47 | 47 ✓ |
| scene-shadow `capture(-exact)?/` | 16 | 16 ✓ |
| **合计 inputs** | **83** | **83 ✓** |

## 3. 需修 4（证据/措辞级；不重跑、不跑 GPU）

1. **P1 receipt 自引用过期**：`receipt.files.receipt.json` 记 `4ef24623…/2078 B/53 行`，实际 receipt 是 `87631dba…/3825 B`。建议**去掉 receipt 自条目**（manifest 本身就不登记自己，自引用无法自证），或注明「自引用为 amend 前值」。
2. **P2 签字时间占位**：`2026-10-31T00:5x+08:00` 在 README/receipt/window-plan/expected-products 共 10 处。签字件应是具体时间；另建议在双签块注明「用户批准的是协议内容（regular + conditional），amend 文本由 Arch 按 Chief 转达落字」。
3. **P3 README §2 标题**仍写「常规 util<40，无例外」，应改为「先常规 util<40；仅当失败才启用已签条件式 validation-only 例外」。
4. **P4 「5 条命令逐字」不成立**：实际 4 条完整命令；条件式例外 monitor 缺逐字命令与新 CSV 文件名（窗口内现场容易写错或覆盖失败 CSV）。建议补第 5 条逐字命令（如 `-OutCsv ...monitor-pre-window-exception.csv ... -MaxUtil 60`）或改口径为「4 条 + 条件式命令模板」。

## 4. 观察 6（记录级接受）

- **O1 条件式协议写得完整**：先常规、失败才例外、现场按序执行、不预设路径、失败 CSV/快照/判定原样保留、`authoring_exception_used=false`、`not_a_normal_threshold_pass=true`、load-bearing 规则（util≥40 仍 clean/streak、ge60=0、util_max<60；全 <40 记 `exception_effectively_not_load_bearing=true`）——与我上轮 U3/per-world 的复核口径一致。
- **O2 判据零放宽**：与 amend 前逐字相同；shadow 臂明确「只一致性、无上游 shadow-off 参考、不宣称数值参考」；DepthSeg 跨路径允许「API 无法暴露双路径时退化为单路径 launch 比较」但判据不变——这是预先声明的 fallback，不是放宽。
- **O3 92 与 S3 的 68 对账**：`+10 scene-sdf + legacy 42→47(+5 perworld)`，scene-hfield 10 / scene-shadow 16 不变；我在现盘按冻结脚本规则重数，恰好 10/10/47/16=83（若窗口前 fixtures 再有变动，冻结会硬断言失败——fail-closed）。
- **O4 冻结器与材料对齐**：实体顺序、suite 名、guards 文件名、`aggregate` 生成方式一致（冻结器代码属实现片，已另行复核）；材料不预设 aggregate。
- **O5 双签对象是「用户 only」**：`approved_by=user via Chief`，三件（计划/基线/92 guards）都要签；不把 Chief 记为签字人。
- **O6 receipt 的 `not_claimed` 只列 3 项**，而 expected-products 列 7 项（含 SDF 求交/plugin SDF/per-world/AA）——不是矛盾，但建议 receipt 与材料主表对齐，免得窗口证据少列不宣称项。

## 5. 边界与不宣称

- 本卡只读复核材料：零 GPU、零执行、零参考写入、不改判据/容差/参考；不评价实现片（另有复核卡）。
- 不宣称渲染数值验收通过、不宣称上游等价、不宣称 SDF 渲染已覆盖——材料本身也只写计划。
- 条件式例外是 **validation-only**，只放宽 util 门槛、不放宽渲染判据；U4 结果不得写成「常规通过」。

## 6. 限制

- 92 的 83 inputs 我用现盘 + 冻结脚本同规则重数（10/10/47/16=83），但最终 freeze 由窗口内 PowerShell 5.1 冻结器生成 guards 文件，逐条 sha 要在窗口证据里核。
- 签字时间占位的「真实分钟」只能由 Chief 转达确认，我无法从仓库推断。
