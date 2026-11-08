# QA 结构级只读体检：S-3（G-L2）验证窗材料

- 任务：`2026-11-08-render-perworld-l-v2-validate-window-s3-structure-check`（owner QA、p83、P8-渲染与BVH）
- 性质：**结构级只读体检，不是完整预审、不做 sign-off、不给「可签名」结论**；零 GPU；只读材料 + 只写本记录
- 对象：S-3 材料落库 **`fc738234`**（README `5449d495ad4ac80896c25c112b85092316ad1dd2a9a657732c1026ea48527cf1`/14,773 B ✓ 与声称一致）+ 材料卡收口 **`34ce7b83`**（最终态 README `11cff9b3…`/15,049 B、receipt `b87ff779…`/12,235 B、manifest `3418a207…`/1,346 B）
- 依据：已签 spec v3 `bb064fe0…`/39,379 B §10.1/§10.2/§12 G-L2/§4/§5/§6/§8
- 基线：HEAD `34ce7b83`、树 clean、锁 FREE（Arch 收口期间我停写等待）

## 0. 结论

**结构级 PASS（可进入 fill 阶段）；这不是 sign-off，也不构成「可签名 / 可开窗」。**

- 与已签 spec **§10.2/§12 G-L2 全部对齐**：guards 算术、旧 suite 只读、aggregate 算法、参考体积、8 项窗口纪律（含 **authoring 例外不适用验证窗**）、实体预期 A/B/C 与 484/576/§2.4 对账/隔离/诊断/refit。
- **PENDING 覆盖完整**：96 处占位（运行材料子集 **78** = 声称口径）覆盖 8 组清单 + REFRESH/DECIDE 两类，**未发现「G-L1 后填不进去」的缺口**；签名卡明确「G-L1 前不定稿」。
- **无提前开窗/签声明**：材料卡 verified 仅表示计划材料完成；`signed_object=false`；「不得定稿 / 签名卡不得发 / guards 不得冻结 / 窗口卡不得 start」写死。
- **边界/门禁干净**：落库与收口两次提交都只动 S-3 材料目录 + 卡/runs；`src/fixtures/scripts/tests` 零 diff；manifest 12/12；四门禁通过、unit 1387/0/402。
- 需在 fill 阶段注意的点见 **§6 清单**（其中「冻结器脚本尚不存在」是 fill 前的一个实现依赖，不是材料结构缺口）。

## 1. 对象与身份

| 项 | fc738234（落库） | 34ce7b83（收口，最终） |
| --- | --- | --- |
| README | `5449d495…`/14,773 B（**与声称一致**） | `11cff9b33c227eea16e9e3546af9979165805212ace46d7a142ec744cd5c6c4c`/15,049 B |
| receipt | `6695b38b…`/11,619 B | `b87ff7798576c0077da105b6ff1f5c56f3853658607064c16855f4ce92228819`/12,235 B |
| manifest | `8f6606af…`/1,346 B | `3418a20728fedc177f1a17adf86085fc5703fb5e9ac811aea41b37adab6e86cb`/1,346 B，**12/12** |
| 卡 | in_progress（normify 禁止 proposed 提交） | **verified**（材料卡） |

- 收口 diff 只新增「卡性质与收口/只产材料」说明、PENDING 保留声明、`awaiting` 更新、`verification_note`、非声明补充；无任何验收/放行/数值结论。
- spec.md 工作树仍 `bb064fe0…`/39,379 B；收口与落库均未改 spec。

## 2. 与 spec 对齐（逐项核）

**guards 冻结算法**（`materials/guards-freeze-list.txt`）：

| 项 | spec §10.2 | 材料 | 结论 |
| --- | --- | --- | --- |
| 旧 suite | 96 = 11 exe + 83 输入 + 2 条 v2.2 清单 | 同 | ✅ |
| 新 exe | +1（`render_per_world_light`）→ 12 | 同 | ✅ |
| 新输入 | ≥10 必需、≤12 → 93..95 | 同（10 条逐条列出，含参考 5 文件/场景/3 脚本/spec） | ✅ |
| 合计 | 107..109 | 同 | ✅ |
| 旧 suite/stash | 只读保留、不重冻/不复用 | 同（明确路径与「不允许复用作证据」） | ✅ |
| aggregate | 沿用 96 条算法 | 同（路径排序 + 无分隔拼接 + sha256） | ✅ |
| 新 guards JSON 字段 | suite/task_id/.../entity_order(12)/frozen_exes/guard_inputs/outline_lists/stash/aggregate | 同 | ✅ |

**窗口纪律 8 项**（`materials/window-discipline.txt`）：monitor PASS 后冻结、body 零编译、`cargo test --no-run` exit 0、三点基线 5 分钟内、`baseline_free=min(三点)`（采样规则与合并规则区分）、`floor_effective=max(signed,现场)`（更高更严）、monitor 60/60 util<40 + compute_filtered=0 + build=0、树 clean、`live==guard==stash`、单次失败即停、**authoring 例外明确不适用验证窗** — 全部在位。

**实体预期**（`materials/entity-expectations.txt`）：A（两台相机逐 world rgb 0 字节差/depth 位级/seg 全等/两跑 sha）、B（灯字段位型）、C（C1/C2 两对 ≥576、验收 384、[384,576) 停/<384 拒）、**§2.4 计数与实体逐项对账且 `check_capture_c1` 必须 Compared（Skipped ⇒ 证据不完整停手）**、world 隔离、诊断只记录、v2.2 不继承、C2 依赖 refit — 全部在位。

参考体积 ≈**11.08 MB / 5 文件**（pending-items 记 `11,076,054 B` 期望值）在位。

## 3. PENDING 覆盖（96 处 = 运行材料 78 + 汇总/清单 18）

| 文件 | PENDING token |
| --- | --- |
| README.md | 26 |
| receipt.json | 30 |
| materials/guards-freeze-list.txt | 9 |
| materials/preconditions.txt | 9 |
| materials/command.txt | 3 |
| materials/evidence-checklist.txt | 1 |
| **运行材料子集小计** | **78**（= 派工口径「78 处」） |
| materials/pending-items.txt | 5 |
| materials/qa-prereview-checklist.txt | 5 |
| materials/signature-summary.txt | 7 |
| materials/window-plan.txt | 1 |
| **总计** | **96** |

分类：`PENDING-G-L1` 73、`PENDING-REFRESH` 7、`PENDING-DECIDE` 8。

`materials/pending-items.txt` 的 **8 组**与所有占位一一对应：

1. G-L1 窗卡 id/提交/证据路径/QA 锚点（README P1 / receipt preconditions[P1]）
2. 参考 5 文件 sha256+bytes+总字节（README §2.3 #1–5 / receipt guards.inputs）
3. `capture.json` §2.4 判别力计数实测值（README §4 / receipt entity_expectations）
4. 新 guards JSON 路径/suite 名/条目数/aggregate/stash（README §2.2/§2.5 / receipt guards）
5. `render_per_world_light` exe live+stash 路径/bytes/sha + 三重一致（README §2.2）
6. S-3b suggested baseline 三点 + suggested signed floor（§3.2/§3.3 / signature-summary）
7. 实体最终 verified 提交 + QA PASS 锚点（`PENDING-REFRESH`，README P2）
8. 可选输入 0–2 最终取舍（`PENDING-DECIDE`，§2.3）

**未发现「G-L1 后填不进去」的缺口**：所有需 G-L1 产物的字段（参考 5 文件、§2.4 实测计数、G-L1 证据/QA 锚点、signed floor 测量时刻）都有明确落点；其余 PENDING（exe hash、guards JSON、baseline）虽标记为 G-L1 后填，但技术上也可提前准备，属保守标记。

## 4. 不可提前开窗 / 不可提前签

- README：缺 PENDING 项 → **S-3 不得签、G-L2 不得开**；§8 「G-L1 未收口前：本材料不得定稿、S-3 签名卡不得发、guards 不得冻结、窗口卡不得 start」。
- receipt：`signed_object=false`；`card_status_required` = verified **材料卡**、验证窗未开、G-L2 执行窗另卡登记；`verification_note` = verified 仅表示计划材料完成。
- §9 非声明：不构成 G-L2 启动、不宣称采集有效/灯验收/上游等价/渲染结论。
- 与卡状态一致：材料卡 **verified**、无窗口动作 ✅。

## 5. 边界 / 证据 / 门禁

- 落库 `fc738234`：18 文件（S-3 材料 12 + 卡 + 4 runs），全部新增；收口 `34ce7b83`：8 文件（README/manifest/receipt + 卡 + 4 runs）；两次均 **`src/`/`fixtures/`/`scripts/`/`tests/` 零 diff**。
- 材料 manifest **12/12** 独立重算；编码卫生（UTF-8/LF、无 BOM、裸 CR·TAB 0）。
- 四门禁：fmt/clippy/clippy-cuda exit 0；unit **1387/0/402**。

## 6. fill 阶段注意事项（交付清单）

1. **按 8 组顺序填**（§3 顺序）：每填一处同步更新 README 对应节 + receipt 对应块 + 重算 manifest；填完再跑四门禁与材料 QA 完整预审。
2. **冻结器脚本尚不存在**：`freeze-guards-perworld-l2-light.ps1` 只有 PENDING 路径/名——这是 fill 阶段的**实现依赖**（不在 8 组清单内）；登记 fill 任务时先落冻结器（含 aggregate 复算与旧 suite/stash 只读校验），再冻结。
3. **11 个旧 exe 的名字/顺序不要从材料照抄**（材料只给数量），从旧 suite `guards-perworld-s1.json` 枚举；新 guards JSON `entity_order` 必须 12 条且与旧 11 条可追溯。
4. **aggregate 复算**：填完后用冻结器按「路径排序 + 无分隔拼接 + sha256」独立复算一次，写入 `aggregate_algorithm` 原文与 `aggregate_sha256`；`counts.total` 必须落 107..109 并与各列表自洽。
5. **可选输入 0–2 条**在冻结前定；不得用可选名额替换任何必需行；候选（冻结器本身 / guards JSON 副本或证据输入）要写入清单并记录选择理由。
6. **S-3b 数值**：suggested baseline 可 GPU-free 预量，但用户签名值必须在开窗前 5 分钟内重测；`signed floor = baseline_free − 3,072`；现场 `max(signed,现场)`；**不要复用 S-2 的 13,302/10,230**（不同窗口）。
7. **§2.4 对账前置**：G-L1 参考的 `capture.json` 必须是字典结构（P1a 已验证）、实体最终版本含 dict 适配（`be12145e` 链）；fill 后先在 CPU 侧用实体 counts 预演一次 `check_capture_c1`（期望 `Compared`），避免 G-L2 现场才发现 `Skipped`。
8. **证据目录名**在 fill 时定稿（建议 `…/perworld-l-v2-validate-window-g-l2/`）并写入 receipt；G-L2 **执行窗口卡在 G-L1 收口后另卡登记**，材料卡不承载执行。
9. **零编译纪律**：`cargo test --no-run` 只在冻结阶段跑一次并留 `cargo-artifacts.jsonl`；body 内只运行冻结 exe，不在 body 阶段重编译/改测试。
10. **receipt `awaiting.qa_prereview`** 现写「G-L1 前不做预审」——本结构体检记录已存在，fill 提交时建议把该字段指向本记录，并保留「完整预审在 fill 后」。
11. **签名卡定稿**：8 组填完、QA 完整预审 PASS 后才定稿；保持 S-3a/S-3b 可分、「signing does not open window」、失败条款与证据/QA 安排。
12. **旧 suite/stash 只读**：fill 与窗口期都要前后校验旧 `guards-perworld-s1.json` 与 `target/render-bvh/perworld-s1-window/frozen-exes/` sha 不变，防止误重冻/误复用。

## 7. 非声明

本记录只是**结构级只读体检**：不构成 S-3 材料的完整预审、不做 sign-off、**不给出「可签名 / 可开窗」结论**；不宣称 G-L2 可开、不宣称采集有效/灯验收/上游等价/任何渲染数值结论；未修改被体检材料、spec、S-2/S-3 签名记录。fill 后仍需完整 QA 预审 + 用户 S-3 双签 + Chief G-L2 token + 现场条件。
