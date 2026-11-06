# perworld L v2 灯判据 spec v2 QA 最小复核

**结论：PASS（0 阻断，0 观察；新签名对象 `6cfaa57e…` / 20,681 B）**

- 任务：`2026-11-06-render-perworld-l-v2-light-criterion-spec-v2-check`（owner QA、P8-渲染与BVH、p55；QA 自登记，依赖 spec v2 修订卡）
- 复核目标：HEAD **`dccd0591`**（修订卡 `verified`；登记 `493cb0ea` → 实现 `dccd0591`），基线 `e9b1181b`（上一轮 QA 预审证据提交）
- 复核方式：只读 + 独立复算（v1 spec 从 git blob 取、v2 取工作树，逐段与逐叶对比；阈值/条数/形状全部重算）；QA 脚本自检 **21/21 PASS**；零 GPU
- 本卡登记提交：`c51d4e7f`

## 1. 结论

v2 修订**精确落在五处锚点**上：§2.2 recipe 拆行、§2.3 改为实际编码、§10.2 guards 条数写死、新增 §11.1 四条采集窗前置、§12 G-L1 同步；其余与 v1 **逐段一致**（判据 A/B/C、384/576、失败 7 条、不继承 v2.2、事实地基、S-1/S-2/S-3、G-L2、非声明均未动）。签名对象与 receipt/manifest 三方一致，边界零改动。**无阻断、无观察。**

信息项（非观察、无需动作）：
- 证据目录沿用原判据卡 `task_id`（v1 卡的 `2026-11-06-render-perworld-l-v2-light-criterion`），v2 卡 id 记在 `receipt.revisions[0].task`，追溯链完整。
- §2.3 体积取整表述沿用 v1（「≈5.55 MB/tag」vs 实测 5,543,106 B ≈ 5.54 MB 十进制），上轮已记备注，本轮未变。

## 2. 逐项结果

### 2.1 五处锚点（PASS）

| 锚点 | 核验结果 |
| --- | --- |
| **§2.2** | 出现 `| recipe.parameters |`（**仅 `parameters` 块**逐字同 `nw3-hinge02040/capture.json.parameters`）与 `| recipe.scene_xml / recipe.script |`（**sha256/bytes 必然变化**，不得要求同源）两行；`recipe` 的 `nworld/hinge_deg/seed/repeat/versions/upstream_revision` 与旧参考同口径；旧单行 `| recipe | 逐字同…` **0 命中** |
| **§2.3** | 改为「产物布局与编码」：rgb/depth/seg **逐 world 独立键**（`rgb_w{0,1,2}_cam0` (240,320,3) f32、`depth_w{0,1,2}_cam0` (240,320) f32、`seg_w{0,1,2}_cam0` (240,320,2) i32），state **按 nworld 首维堆叠**（`qpos (3,8)`、`geom_xpos (3,7,3)`、`geom_xmat (3,7,3,3)`、`light_xpos/light_xdir (3,3,3)`）；旧 `[nworld, 240, 320, 3]` / `[nworld, 240, 320]` **0 命中**；表内形状/dtype 与既有参考 npz 实测逐项一致 |
| **§10.2** | 写死「**新增 1 个 exe**（`render_per_world_light`）+ **输入 ≤12 条**、**必须包含 9 条必需**（新 tag 5 文件 + 场景变体 1 + 采集脚本 1 + 新模块 1 + 本 spec 1 = 9）、其余至多 3 条；最终条数以验证窗卡冻结清单为准、不得少冻结 9 条必需」；算术 5+1+1+1+1=9、12−9=3 自洽；旧「预估 +14 条」**0 命中** |
| **§11.1** | 新增「S-2 采集窗前置（写死）」四条：①`command` 字段补写+采集后自核非空；②`captured_at` 由显式 `MJWARP_CAPTURE_AT` 产生非空 ISO8601（现状 `""`）+ 开窗/采集后自核；③`per_world_scope` 从旧口径 `does_not_cover: per-world light poses` 改为新覆盖口径并自核；④`recipe.task_id` 旧卡 id 二选一处置（参数化 / 新增 `reference_tag`）并写进采集窗证据；且「否则 S-2 不得签」；receipt `preconditions_collection_window` 恰 4 条对应 |
| **§12** | G-L1 前置追加「**§11.1 四项采集窗前置全部落实（command / captured_at+env / per_world_scope 文案 / task_id 或 reference_tag）**」；G-L2 行逐字未变 |

### 2.2 签名对象与 receipt（PASS）

- `spec.md` 实测 sha256 **`6cfaa57e2c8d5317442f90fddc157c666430d726ad20d3ceace102fe0b435900`** / **20,681 B**，与 `receipt.spec`、`manifest.txt` 行三方一致。
- `receipt`：`version=v2`、`status=draft_pending_user_signature`、`effective=false`、`spec.sections=15`；`supersedes={version: v1, spec_sha256: 1821be98…, spec_bytes: 16625}` 与 **v1 实测逐字节一致**；`revisions[0].task` = v2 卡 id、`items` 5 条对应 O1/O4/O5/O2/O3、`unchanged` 声明与实测一致。

### 2.3 其余与 v1 逐段一致（PASS）

- 按 `## N.` 分段对比：**§1、§3、§4、§5、§6、§7、§8、§9、§13、§15 逐字节一致**。
- 变化面逐段收敛：preamble 仅「任务行/状态行」2 处改 + 新增「版本：v2」1 行；**§2 内仅 2.2/2.3 变**（2.1/2.4 逐字节同）；**§10 内仅 10.2 变**（10.1/10.3 同）；**§11 原 S-1/S-2/S-3 表为前缀完整保留**、其后新增 §11.1；**§12 仅 G-L1 行变**（G-L2 逐字同）；**§14 原 3 条保留**、其后新增 §14.1 修订记录。
- 语义关键点复核（belt & braces）：384/576 与 0.5%×76,800 在位；失败即停 7 条计 7；`cfd1038a`/`14893fc6`/`|ULP|≤128` 与 10 回归实体列表在位；事实地基四数值 + spot 0/0 在位；S-1/S-2/S-3、G-L1/G-L2 各唯一；非声明在位；v1 sha 仅出现于「版本/supersedes/修订记录」语境。

### 2.4 边界与证据（PASS）

- `git diff e9b1181b..dccd0591` = 恰 **8 路径**（spec v2 卡 + 4 gate runs + spec/receipt/manifest）；`fixtures/`、`src/`、`scripts/`、`tests/`、criterion v2.2 **零 diff**；未采集、未新增参考。
- 被复核卡 `status=verified`、`revision.before=493cb0ea…`、checks 含四门禁；其 `unit` gate 记录 **1359 passed / 0 failed / 400 ignored**。
- 证据目录 3 文件、manifest 首行 `#`、**2/2** 条目复算通过、不自列；UTF-8/LF/无 BOM/裸 CR·TAB 0。

## 3. 证据清单

| 路径 | 说明 |
| --- | --- |
| `scripts/verify_spec_v2_review.py` | 复核脚本（21 项只读复算，可复跑） |
| `logs/verify-spec-v2-review.json` | 21 项检查机器可读结果（PASS 21/21） |
| `logs/verify-spec-v2-review.log` | 脚本原始输出 |
| `artifacts/scope.json` | 变更面 8 路径 + 零越界探针 |
| `artifacts/signature-object.json` | v2 sha/bytes 三方一致、v2/supersedes、状态 |
| `artifacts/evidence-selfcheck.json` | manifest 2/2（3 文件）与编码 |
| `artifacts/anchor-2-2.json` | recipe 拆行锚点 |
| `artifacts/anchor-2-3.json` | 编码锚点 + 既有参考 npz 实测形状 |
| `artifacts/anchor-10-2.json` | guards 条数锚点（9 必需/≤12/至多 3） |
| `artifacts/anchor-11-1.json` | 采集窗四条前置锚点 |
| `artifacts/anchor-12.json` | G-L1 同步锚点 |
| `artifacts/section-identity.json` | 逐段一致矩阵（含 §2/§10/§11/§12/§14 收敛面与 G-L1 旧/新行） |
| `artifacts/semantic-core.json` | 阈值/失败条数/不继承/事实/清单/非声明关键点 |
| `artifacts/reviewed-card.json` | 被复核卡状态与 unit 1359/0 |
| `README.md` / `receipt.json` | 本报告与机器可读回执 |
| `manifest.txt` | 本目录 sha256 + bytes（首行 `#`，不自列） |

## 4. 边界与非声明

- 只读复核 + 只写本卡证据目录；不改被复核 spec/receipt/manifest、既有证据、代码、fixtures、criterion v2.2。
- 本复核**不构成用户签名**；v2 仍 `draft_pending_user_signature` / `effective=false`，签名前不得据此开窗；不写覆盖成立 / 上游等价 / 渲染数值结论 / 发布依据。

## 5. 复现

```powershell
cd D:\Rust\mjwarp-rs
target/mjwarp-parallel/venv/Scripts/python.exe -B docs/development-evidence/render-bvh/perworld-l-v2-light-criterion-spec-v2-check/scripts/verify_spec_v2_review.py
# 期望末行：SUMMARY PASS 21/21
```
