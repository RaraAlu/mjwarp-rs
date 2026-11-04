# G10@P5 家族收口 QA 复核（只读）

任务：`2026-11-04-g10-p5-family-closeout-check`（owner QA）。复核对象：`2026-11-04-g10-p5-family-closeout`（verified；证据提交 `e5cdc542`；NP01 措辞 fix `afd5e6cc`）。日期：2026-11-05（空闲段）。
**边界：只读 + 只写本目录；零 GPU；不回改 summary、`g10-p5-acceptance-map/` 或任何已 verified 证据；问题只记录为观察。**

## 1. 结论

**PASS-with-observations（0 阻断）。** 12 条 requirement 计数、partial / uncovered 边界、设备 vs host 证据分界、NP01 措辞、非声明、`src/tests/fixtures` 零 diff、以及「fix 不改结论」全部通过。
唯一实质观察 **O1**：**「54 张子卡」没有成文索引/口径**（Chief 点名核对项），建议一次 addenda 写死；其余 O2–O5 为低观察，均不影响 met/partial 记账与收口结论。

## 2. 逐项核对

### 2.1 计数一致（PASS）
- `g10-p5-acceptance-map/map.json`（verified）`summary`：requirements 12 = **met 8 / partial 4 / unmet 0**，uncovered 6。
- summary `receipt.json.requirement_counts`：`total 12, met 8, partial 4, unmet 0, uncovered_extra 6`；`requirements[]` 17 项状态与 map.json 逐 id 相同。
- README §2 的 12 行状态列逐 id 与 map.json 一致：met = G6-1.1 / 1.3 / 1.4 / 1.5 / 1.6 / 2.2 / 2.3 / 3.1；partial = G6-1.2 / 1.7 / 2.1 / 2.4；unmet 0。
- uncovered 6 条 id 齐全（UNC-1 partial、UNC-2 closed、UNC-3 partial、UNC-4/5/6 unmet），与 map.json `uncovered[]` 一致。
- 注：QA 卡 acceptance 写的 `receipt.json.verification.map_summary` 在实际 receipt 中不存在（实际字段为 `requirement_counts` + `mapping_source`），本回复核按实际字段执行，见 O2。

### 2.2 partial 边界如实（PASS）
- G6-1.2：`HFIELD(32)` overflow=32 实测**只在 QA 独立复核**、G5 交付片 README 只写实现映射 → 记 partial；
- G6-1.7：容量截断 case（`sdf-box-naconmax40`、`ellipsoid-box-naccdmax1`）**只在 QA 独立复核**自造 → partial；
- G6-2.1：默认产品管线未接入、primitive 分支层未全覆盖、mesh 边/顶点分支仅静态核对 → partial；
- G6-2.4：NP01/NP02 已核对，NP03 native 侧按用户拍板 B 只诊断，NP04 随独占性能窗（相对基准）→ partial。
README §2/§6 逐条写明「谁实测、证据在哪」，未把 partial 写成 met；§6.1 另写死「4 条 partial 不得按 met 计」。

### 2.3 设备 / host 分界（PASS）
- 设备数值只引 `g10-p5-gpu-kernels/gpu-window/`：cases `hfield-capsule`、`hfield-sphere`、`sdf-box` 均存在；G6-1.4 的 `203/240`、`675/720`、`2.384e-7` 与 `hfield-sphere 8/8`、`hfield-capsule 7/7` 可在 `g10-p5-gpu-kernels` receipt/README 与其窗口产物链上对上。
- host 数值只引 `g10-p5-hfield-host`（8/8、7/7、72/72、63/63）、`g10-p5-sdf-host`（r2 卡：80/80、720/720）、`g10-p5-sdf-relmat-fix`（80/80、720/720）、`g10-p5-host-ccd-overflow-bit-fix`（272→256）。
- 无窗口记录的行为（overflow=32、容量截断）明确写「仅 QA 独立复核记录」，不冒充交付片实测/设备位级；G6-2.2 性能数字（峰值 4834 MiB、+1171）在 `g10-p5-acceptance-map/perf/perf-summary.json` 可对上。

### 2.4 NP01 措辞（PASS）
`afd5e6cc` 把 README §5 与 `receipt.np_check.NP01` 改为「已核对；E 指标用户 2026-11-05 已签（同码跨后端核对，A2 `b9cb2993`）」，并写明判据为「实测 worst ≤ 已签 atol」、不用 E 比值、不宣称 native 口径——**与 A2 提交号 `b9cb2993` 一致**，符合 A2「自 A2 起写已签」的措辞规则。卡面 acceptance 保留签发前的「E 待签」快照，由 `addenda` 记录本次更新（见 O4）。

### 2.5 零 diff 与「结论未被 fix 改动」（PASS）
- `git diff --stat 65fc778a..HEAD -- src tests fixtures docs/development-evidence/g10-p5-acceptance-map` 为空。
- `e5cdc542` / `afd5e6cc` 各只碰 summary 3 件 + 本卡 JSON；`afd5e6cc` numstat：README `+1/-1`、receipt `+1/-1`、manifest `+2/-2`、卡 `+9/-1`，改动行仅 NP01 措辞与 addenda/updated_at。
- 非声明 4 条在案（不声明整族通过 / 设备全路径覆盖 / 上游等价 / 对外发布依据）；卡面 `forbid_claim` 的 4 个词在 README 与 receipt 中 **0 命中**。

## 3. 「54 子卡」核对（Chief 点名项）

**没有成文的 54 项索引。** summary 目录只有 `README.md`、`receipt.json`、`manifest.txt` 三件；README 仅一句「54 张子卡全部 verified」，`receipt.subcards_verified = 54`，均未列成员、未给口径。

能对上 54 的两种候选口径，都**不等于**「54 张 verified 子卡」：

| 口径 | 实际值 | 与「54 张 verified 子卡」是否一致 |
| --- | --- | --- |
| `docs/development-evidence/g10-p5-*` 证据目录数 | **54**（逐目录均可解析到 ≥1 张 verified 卡，0 悬空；目录≠卡） | 数量对、语义不对 |
| `.normify` 中 g10-p5 命名卡（快照 `e5cdc542`/`afd5e6cc`） | 58 张 = **52 verified + 4 abandoned + 2 proposed**；`58−4=54` 恰为「非 abandoned 卡数（仍含 2 张 proposed）」 | 数量对、语义不对 |
| g10-p5 命名 **verified** 卡 | **52** | 不一致（差 2） |

结论：**「54」的来源未写死、成员未列，按卡口径实际是 52 张 verified。** 建议 Arch 用一次 `normify fix`（addenda）写死口径（例如「54 = 证据目录数」）或直接贴 54 项清单；只改口径说明、不动 met/partial 与结论，故**不阻断**收口。

## 4. 观察清单（按严重度）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | 建议修（低） | 「54 张子卡全部 verified」无索引/口径；卡口径实际 52 verified、证据目录 54 | 一次 addenda 写死口径或贴清单；不阻断 |
| O2 | 低 | QA 卡 acceptance 引用的 `receipt.json.verification.map_summary` 不存在（实际 `requirement_counts`/`mapping_source`） | 卡模板措辞，下次同类卡对齐实际字段 |
| O3 | 低 | README §4 host 证据写 `g10-p5-sdf-host-r2`（是卡 id；实际证据目录为 `g10-p5-sdf-host`） | 与 §2 G6-1.3 用卡 id 的写法统一即可 |
| O4 | 低（预期） | NP01 措辞与卡面 acceptance/note 的「E 待签」快照不一致 | 已由 `afd5e6cc` 的 addenda 记录、结论未动，符合 A2 规则，不算违规 |
| O5 | 低 | UNC-1 写 backlog `2026-10-27-mesh-polygon-gt16-device-gap`「不 start」，该卡实际已 verified/closed（交付评估+设计，设备实现未 start） | 措辞可改为「评估已闭、设备实现未 start」 |

## 5. 复现与证据

- 脚本：`scripts/g10_closeout_check.py`（只读；核 manifest 2/2、map/receipt/README 计数、引用解析、54 口径、零 diff、fix numstat、forbid_claim）。
- 日志：`logs/g10-closeout-check.log`（`RESULT: PASS`，0 FAIL；repo 快照 `90bc1169`，复核对象提交 `e5cdc542`/`afd5e6cc`）。日志由 Python 直写 UTF-8/LF（本机 PowerShell `>` 重定向会写成 UTF-16LE+BOM+CRLF，已规避；脚本末尾含证据目录编码自审）。
- `manifest.txt` 4 条（README + facts.json + 脚本 + 日志），LF、无 BOM，首行 `#`。
