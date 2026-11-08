# QA 最小复核：S-2 材料 fix r2（`0b2b4cc4`；零 GPU）

- 任务：`2026-11-08-render-perworld-l-v2-capture-window-s2-material-fix-r2-check`（owner QA、p81、P8-渲染与BVH）
- 复核对象：S-2 采集窗材料 fix r2，提交 **`0b2b4cc4`**（parent `be12145e`；卡 `2026-11-08-render-perworld-l-v2-capture-window-s2-material`）
- 依据：已签 spec v3 `bb064fe0…`/39,379 B §2.4、§11.1、§12 G-L1、§9.3、§10.2；capture-counts 卡 `311b5a84` + QA `fefe45a5`
- 基线：HEAD `0b2b4cc4`、树 clean、锁 FREE；**只读被复核内容 + 只写本卡证据；零 GPU、不采集、不开窗**

## 0. 结论

**PASS（0 阻断）→ 可发新 S-2 双签卡。**

- **P1a 三处齐全且与 §2.4 对齐**：README §1 表、`materials/preconditions.txt`、`receipt.json.preconditions` 均含 P1a（`c1_rgb_changed_px` 两对 world 的 bitwise+magnitude、`c2_depth_changed_px`、`c2_seg_changed_px`、诊断两项、结构 `{"0->1":…,"1->2":…}`）；锚点 `311b5a84`+`fefe45a5`；pilot 交叉数字与我的复核一致。
- **脚本身份三值逐位吻合**：三脚本 sha/bytes/行数独立重算 = 材料声明；pilot 旧值标 **OBSOLETE**，`command.txt` 第 1 步改为现场 `Get-FileHash` 记录实际值。
- **采后自查增量可执行**：README §4 第 6 步 + `command.txt` 6b（python 语法编译通过、capture/readout 键名正确、两对 world、`assert all(...)`、打印 OK/MISMATCH）+ evidence-checklist `window/counts-crosscheck.txt` + window-plan 第 8 条门槛。
- **S-2b/例外/失败条款零改动**：`13,302`/`10,230`/`floor_effective=max` 与 authoring 例外、失败条款的受保护子句在 diff 中 **0 行变更**。
- **签名卡字段可用**：S-2a 已补 §2.4 计数行；S-2b/例外/失败条款/Signs NOTHING here 均在；新增内容不改结论口径。
- **边界/卫生/门禁**：仅 S-2 证据目录（9 文件）+ 卡 + 4 runs = **14 文件**；`src/fixtures/scripts/tests` 零 diff；manifest **12/12**；编码卫生；四门禁通过、unit **1387/0/402**。

## 1. 材料 sha/bytes 与 manifest（独立重算）

| 文件 | sha256 | bytes |
| --- | --- | --- |
| `README.md` | `c8bb71ef7c863c22b48f4abb2237c520d982f9e59c37513c48faed2e1787e9f9` | 18,299 |
| `receipt.json` | `79c7a88a160ea6c55c71577140caaa2efc10e65f8baae8acbfd1544523f99171` | 11,681 |
| `manifest.txt` | `23aa135ea48c22c98eb656f084921b6fae56810870eaee8e9d85b0eeb6148f22` | 1,342 |

manifest **12/12** 独立重算通过；目录内无 BOM/裸 CR/TAB。

## 2. P1a（§2.4 判别力计数前置）三处核对

| 位置 | P1a 在位 | 5 组字段 | 结构 `0->1`/`1->2` | 锚点 `311b5a84`/`fefe45a5` | pilot 数字 |
| --- | --- | --- | --- | --- | --- |
| README §1 表 | ✅ | ✅ | ✅ | ✅ | ✅ |
| `materials/preconditions.txt` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `receipt.json.preconditions` | ✅ | ✅ | ✅ | ✅ | ✅ |

pilot 交叉数字（与我的 capture-counts 复核卡 `fefe45a5` 逐项一致）：C1 magnitude 38,285/40,644、bitwise 43,604/41,815；C2 depth 35,828/33,883、seg 16,041/16,754；诊断 cam-persp depth/seg 0/0；cam-hinge rgb magnitude 50,363/52,329。

## 3. 脚本身份（独立重算 = 声明值）

| 脚本 | sha256 | bytes | 行数 |
| --- | --- | --- | --- |
| `scripts/capture_render_perworld.py` | `f3af09ab992981f2dc5d7f2e3479aaf9eebb97df0d2a774b247c8e0772ad6f07` | 23,637 | 490 |
| `scripts/perworld_capture_meta.py` | `3fdb85953681a1a695a9611fc48ddf0862318acf0e0767e831029a820127b87e` | 19,154 | 379 |
| `scripts/perworld_light_prep.py` | `9fd5f8c25894380992986ffecaa584ee4fdff53a042438a70812f04699113e2c` | 7,206 | 160 |

pilot 旧值 `55b91f23…`/23,483 B、`296c2b12…`/12,508 B 已在 README/command 标 **OBSOLETE**；`command.txt` 第 1 步「record actual values」+ 现场 `Get-FileHash`（且注明 S-2 材料 identity 期望值）。

## 4. 采后计数交叉自核（可执行性）

- README §4 第 6 步：`capture.json` 的 `c1_rgb_changed_px`（magnitude/bitwise）、`c2_depth_changed_px`、`c2_seg_changed_px` 与两个诊断项，须与 readout 产物（`c1_cam_persp`/`c2_cam_hinge`/`diagnostics_no_threshold`）**两对 world 逐项相等**，不一致停手。
- `command.txt` 6b：`python -B -c` 单行命令**语法编译通过**；引用 capture 侧 5 组键（含 `bitwise`/`magnitude`）与 readout 侧正确键名；`assert all(f(p)==g(p) for p in ('0->1','1->2'))`；逐对打印 `OK`/`MISMATCH`；引用路径为正式参考根于 `target/…/readout.json`。`readout_c1_c2.py` 确有 `--pilot-dir` 与 `--json-out`。
- `evidence-checklist.txt` 增 `window/counts-crosscheck.txt`；`window-plan.txt` 门槛第 8 条「capture.json section 2.4 counts equal the readout counters for both world pairs」在位。

## 5. S-2b / 例外 / 失败条款未被动（diff 受保护子句 0 行变更）

- `baseline_free = 13,302 MiB`；signed floor **10,230 MiB**；`floor_effective = max(signed floor, 现场 floor)`（取严=取更高）。
- authoring 例外：仅本采集窗、仍需 S-2 双签、`exception_load_bearing=true`、floor 不放松、pilot/验证窗不适用、失败即停不重试不放宽。
- 失败条款：停手报用户、不重试、不改阈值/相机/recipe、不换参考、失败前只清本次新建 out-dir。
- 用 diff token 扫描（`13,302`、`10,230`、`floor_effective = max`、authoring/exception_load_bearing、Failure clauses、no retry、pilot or the validation window）：**0 行**变更。

## 6. 签名卡字段（可直接粘贴；与上一版差异仅 S-2a 补计数行）

- **窗口**：G-L1 per-world L v2 灯采集窗
- **依据**：spec v3 sha256 `bb064fe0…`/39,379 B（S-1 已签 2026-11-08）
- **S-2a（计划）**：执行者 Backend、**单次**、`--repeat 2` 两遍逐位一致、正式参考根 `fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040/`、tag/seed/fastmath/shadows/显式 `MJWARP_CAPTURE_AT`；5 文件 **≈11.08 MB**；**`capture.json` 含 spec §2.4 判别力计数**（5 组字段、两对 world；已验证卡 `311b5a84` / QA `fefe45a5`）
- **S-2b（suggested，开窗重测，stricter = higher wins）**：`baseline_free 13,302 MiB`；`signed floor 10,230 MiB`；`floor_effective = max(signed floor, on-the-spot floor)`
- **authoring 例外**：capture-window-only util<60 fallback，逐样本记录、不提前解锁、不用于 pilot/L2
- **失败条款 / 证据与 QA 安排**：同上（不重试、不放宽、证据目录 `…/perworld-l-v2-capture-window-g-l1/`）

## 7. 观察项（均不阻断）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | 低 | `command.txt` 6b 只打印 OK/MISMATCH，未把输出**重定向/落盘**到 `window/counts-crosscheck.txt`（evidence-checklist 要求该证据文件；第 7 步「按 checklist 归档」隐含人工保存） | 下次 fix 在 6b 末尾加 `> window/counts-crosscheck.txt`（或注释说明），消除执行歧义；不阻断签名 |
| O2 | 信息 | 派工描述「14 文件（证据 10 + 卡 + 4 runs）」；实际 14 = **证据 9** + 卡 1 + runs 4（README、manifest、command、evidence-checklist、preconditions、qa-prereview、signature-summary、window-plan、receipt） | 仅数字口径，不影响结论 |
| O3 | 信息 | P1a 的诊断 cam-hinge rgb 只列 magnitude（50,363/52,329）；capture.json 实际同时记录 bitwise（54,380/54,250），诊断项非门槛 | 无需改 |
| O4 | 信息 | 材料任务卡 acceptance 仍含历史 `floor_effective = min(...)` 文本；README §8 r2 第 6 点已说明以 `ade516b5` 为准 | 历史任务定义，不阻断 |

## 8. 非声明

本卡只核材料一致性与可执行性：**不宣称** G-L1 已可开（仍需 QA PASS + **用户 S-2 双签** + Chief token）、不宣称采集有效/判别力恢复/任何渲染数值结论；建议 baseline 仅为参考值，signed floor 以用户签名卡为准。
