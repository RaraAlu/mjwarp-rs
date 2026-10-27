# UNC-3 S5 台账收口独立复核（requirement 12 条 + UNC-3 六项 + UNC-1 收窄 + mesh 汇总）

**任务**：`2026-10-27-unc3-ledger-closure-s5-independent-check`（owner QA）。
**被复核提交**：S5 主体 **`7b99cbf4`** + 补正 **`77055c95`**（HEAD；U8 差距 backlog `2a0de8ab`）。
**结论**：**pass / 阻断 0 / 观察 5（0 一般 + 5 建议）**。docs-only，无门禁项（gate 不适用，见 `receipt.json`）。

只读复核：未跑 cargo/GPU、未改任何 S5 数字/判据；`verify_s5.py` 只读 `git show` + 证据文件 + S4 窗口日志 + S2 branch-map。

## 1. 逐条结论（Chief 7 点 + 任务卡 acceptance）

| # | 复核点 | 结果 |
| --- | --- | --- |
| 1 | UNC-3 六项数字对 S4 窗口日志 | **一致**：U1 3/3；U3 = face-down 4/4 + face-up 4/4 + tilt-x 2/2 + tilt-y 2/2 + single 1/1（角点并集 8/8，5 pose nacon 合计 13）；U4 2/2；U5 4/4；U6 3/3（ContactMulti）；U9 launcher 275/275 bit、ULP 0。逐条取自 `.../branch-closure-validation/logs/validation-attempt1-*.log`，且每 case `nacon==ref_count`、`failures=0` |
| 2 | U2/U7/U8 措辞 | **一致**：U2「不可达 / 非上游防御（pinned 无保护，设备 `primitive_plane_shapes.cu` 才加 `!(length>0)||!isfinite`）」；U7「仅限制存在（count=1 与 degree>10 限制语义一致，同时记 dist/pos，不等于证明触发）」；U8「设备 1 vs 上游 4 = 行为差异 / 限制存在」，明示不进分母、不改判据。**新增行**中「与上游一致 / 已验证回退 / U8 已覆盖」只出现在否定句（如「不写『已验证回退行为与上游一致』」），无正面使用 |
| 3 | UNC-1 收窄 | **一致**：mesh-mesh 已覆盖（M3 4/4 bit）不再是未覆盖项；`nmesh>1` **仅 1 例**（S4 U6 3/3）；一般拓扑未穷举；degree>10/polygon>16 仅限制存在；mesh 边/顶点分支层静态计数 **3/4/2/5**（14 条 = 已 case 覆盖 3 / 仅静态核对 4 / 仅限制存在 2 / 未覆盖 5）与 S2 `branch-map.json` **逐项一致**（我按 family 过滤 entries 复算）；hfield-capsule 单 case 275/275 bit 计入已覆盖且明写不外推；efc_address / UNC-6 / 跨平台 / 性能写「不变」；status 仍 partial |
| 4 | 计数与回归对象 | **一致**：case 级 **33** = multicontact 3 + launcher 4 + primitive 3 + plane 6 + sphere_capsule 4 + branches 6+4 + M2 2 + M3 1；stage2 **17 dump**；「13 组合」在 S5 新增文字中标「旧口径（已作废，仅历史）」、launcher 写 4；回归对象写「M3 窗口冻结二进制（8 个 `target/m3-freeze`，构建 01:06）」，M3 提交 `f1e5cda1`（01:28）在冻结之后（含 O7 + 测试拆分），并写「不写成当前树重编通过」 |
| 5 | map.json 只增不改（按补正口径） | **一致**：`2a0de8ab→77055c95` 逐字段 diff 只有 `summary.note`、`uncovered[UNC-1/UNC-3].evidence_paths`（追加）、`.reason`/`.impact`（字符串追加）、新增 `s5_update`（条目级 + 顶层）；**requirement 12 条逐项不变**；**uncovered 其余 4 条（UNC-2/4/5/6）逐项不变**；summary 计数 12/8/4/0/6 不变；UNC-1/UNC-3 的 status 不变；旧 reason/impact **原文保留在 `s5_update.previous_reason/previous_impact`**，新串以旧串为前缀追加（我逐对核过） |
| 6 | backlog / S2 12 条 | **一致**：`2026-10-27-mesh-polygon-gt16-device-gap` status=**backlog**（未 start）、3 条 acceptance 全为「差距评估 + 设计方案 + 落文档」、明写「改设备代码或判据需用户批准」、不 start/不改代码/不开窗口；S5 文字保留「S2 的 12 条『可验证』需定向 case 才能转覆盖」 |
| 7 | 不宣称 / 禁词 / 门禁 | **一致**：S5 新增行内无正面「窗口通过」「任何分支全量覆盖」类断言（命中全在「不写/不宣称/not_claimed/边界」里）；U8 定死句在 §6.2/§7.3/§10；S5 任务 checks 只有 json+forbid_claim、**无 gate**（docs-only，gate 不适用） |

## 2. 关键数字复核（我重算/重取，不采信台账转述）

| 项 | 台账值 | 我的核对 |
| --- | --- | --- |
| case 级比较 | 33 | 3+4+3+6+4+10+2+1 = **33**（与 S4 日志 case 行逐一对应） |
| stage2 dump | 17 | S4 stage2 日志 17 行（8 case × Contact/ContactMulti + sdf-box），全「两次一致 true」 |
| S4 窗口采样 | 871 / 3144 / 95·96·109 / 12918 / 9942 / 46 / streak 0 | 与 `samples.csv`+`guard.log` 独立重算一致（见 S4 复核 `unc3-branch-closure-s4-independent-check`，本片只引用其已核数字） |
| mesh 分支层静态计数 | 3/4/2/5（共 14） | S2 `branch-map.json` family=mesh-multicontact entries 复算 = 已 case 覆盖 3 / 仅静态核对 4 / 仅限制存在 2 / 未覆盖 5 |
| S2「可验证」条数 | 12 | `branch-map.json.counts.by_status.可验证 = 12` |
| M3 冻结/提交时序 | 冻结 01:06、提交 01:28 | `git show -s --format=%ci f1e5cda1` = 2026-10-27 01:28:13；M3 冻结构建 01:06:17/01:06:25 |
| s5-receipt 变更文件 sha | 5 条 | 4 个文档 sha+bytes 全对；第 5 条为 receipt 自身（path-only，注「自引用不进 manifest」） |

## 3. 补正提交（`77055c95`）专项

- 改动文件 5 个：`.normify` S5 任务记录、交付记录、acceptance-map README、map.json、s5-receipt.json；
  **无 `src/`、`tests/`、`fixtures/`、判据/容差/参考**（`git diff 09f6551e 77055c95 --name-only` 只列 docs/.normify）。
- map.json 的 UNC-1/UNC-3 `reason`/`impact`：新值 = 旧值 + 「（S5 更新…）」追加段；`previous_*` 与旧值逐字节相等；
  「仍未覆盖 3 类」「hfield-capsule 未覆盖」等旧口径已不再作为**最新结论**出现（只在 previous/旧文里）。
- s5-receipt 新增 `map_text_correction`（what/why/traceability/scope），交付记录 blob+sha 5/5 与工作树一致；
  S5 任务 JSON 有 `normify fix` addenda（reason：补正台账自相矛盾，不改 status/数字/判据）。
- 补正前后 `git diff 7b99cbf4 77055c95 -- map.json` 只涉及上述 reason/impact 追加 + s5_update 字段。

## 4. 观察（全部建议级）

- **O1 建议｜s5-receipt `changed_files` 自引用条目**用「path-only + note」表达（`{path, note}`），
  机器校验会把「缺 bytes/sha」当异常；建议改为独立 `self_entry` 键或省略，避免外部复核脚本误报。
- **O2 建议｜S5 范围文字在两处以上重复**（`summary.note`、UNC-1/UNC-3 的 `s5_update`、顶层 `s5_update`、
  README §6.2/§7.3），本次补正正是因 reason/impact 未同步而触发；建议定一处权威（如顶层 `s5_update`）
  其余改为指针/引用，减少复制漂移。
- **O3 建议｜建议给 map 更新加一步 lint**：`reason`/`impact` 必须包含最新窗口 task id 或与 `summary.note`
  同源，否则自动报错——可避免再出现「首次提交后才发现旧口径」的补正提交。
- **O4 建议｜历史「13 组合」文字仍散落在 acceptance-map 旧章节**（S5 新增文字已标「旧口径作废」，
  但旧章节原句未改）；为避免 S5 汇总被误引用，建议后续统一加「旧口径」前缀或指向 33 新口径（低优先）。
- **O5 建议｜S5 侧未显式写「gate 不适用」**：S5 任务 checks 只有 json+forbid_claim（docs-only，无 gate），
  但 s5-receipt 边界段未单列该句；建议后续 docs-only 片在 receipt 明写「无门禁项（gate N/A）」，
  与 QA 复核片口径一致（本 QA receipt 已写明）。

## 5. 边界 / 不宣称

- 本片只读复核，不重算设备数值（S4 窗口数字引用我在 `unc3-branch-closure-s4-independent-check` 已独立核过的结果）；
  S5 是台账文字更新，不引入新设备实验。
- 不宣称「窗口通过」「任何分支全量覆盖」「与上游一致」；UNC-1/UNC-3 仍 partial；U8 差距为真实债务
  （backlog 未 start，改代码/判据需用户批准）；hfield-capsule 仅单 case，不外推。

## 6. 证据清单 / 复现

| 文件 | 内容 |
| --- | --- |
| `README.md` | 本报告 |
| `claims-audit.json` | 7 点 + acceptance 逐条核对结果（含数字） |
| `verify_s5.py` / `verify_s5.json` | map.json 逐字段 diff、S2 计数、UNC-3 六项 vs S4 日志、receipt sha、补正追加校验、新增行措辞 |
| `manifest.json` | 本目录逐文件 sha256 |
| `receipt.json` | 任务 id / 判定 / 计数 / 关键数字 / gate N/A |

复现（只读）：`python verify_s5.py`（读 `git show 2a0de8ab/7b99cbf4/77055c95` + S4 日志 + S2 branch-map）。
