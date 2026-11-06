# perworld L v2 灯判据可签草案 QA 读前预审

**结论：PASS-with-observations（0 阻断；5 条低观察，其中 2 条为「采集窗前必须落实」的接口前置，不影响 S-1 签名）**

- 任务：`2026-11-06-render-perworld-l-v2-light-criterion-check`（owner QA、P8-渲染与BVH、p55；QA 自登记，依赖判据草案卡）
- 复核目标：HEAD **`1ef5bd5b`**（草案卡 `verified`；登记 `42748ac7` → 实现 `1ef5bd5b`），基线 `b6f5b724`
- 复核方式：只读 + 独立复算（不采信执行方结论；阈值与数值全部重算，事实地基回查 prep 证据原始 JSON，96 条 suite 事实回查归档 guards 本体）；QA 脚本自检 **21/21 PASS**；零 GPU
- 本卡登记提交：`271749bb`

## 1. 结论与定级

草案满足「可签」的实质条件：**签名对象机读一致、条款自洽、判据阈值算术正确、事实地基与 prep 证据逐值对齐、签名/GO 清单无歧义、边界零改动、未签不生效声明在位**。5 条观察均为**文本/接口口径**问题：O1/O4/O5 只关措辞与预估口径；**O2/O3 是「已签名规格的必含字段 vs 现有采集脚本输出」的接口差，必须在采集窗前补齐**（否则采集产物不满足已签名 §2.4，属窗口前置而非 spec 缺陷）。

| 编号 | 级别 | 区域 | 摘要 | 处置建议 |
| --- | --- | --- | --- | --- |
| O1 | 低 | §2.3 产物形状记法 | 用堆叠记法 `[nworld, 240, 320, 3]` 描述 rgb，但既有参考实际是**逐 world 独立键**（`rgb_w0_cam0` 等，各 `[240,320,3]`）；同节又写「与 nw3-hinge02040 同编码」，两者字面矛盾 | 签名前把记法改成「逐 world 键」或注明明是逻辑形状；不改语义 |
| O2 | 低 | §2.4 vs 现有脚本（采集前置） | §2.4 要求 capture.json 必含「命令」与「采集时间」，但现有脚本**不写 `command` 字段**；`captured_at` 从 `MJWARP_CAPTURE_AT` 读、v1 参考该字段为**空串**，而 §9.2 写死命令未含该 env；另 `recipe.task_id` 仍为旧卡 id（prep 复核 O3） | 采集窗卡/S-2 材料写明：补 `command` 写入、采集时设 `MJWARP_CAPTURE_AT`、确定 task_id/reference_tag 口径 |
| O3 | 低 | §2.4 的 per_world_scope（采集前置） | §2.4 要求 `capture.json.per_world_scope` 写明覆盖「per-world 灯位姿 + geom 位姿」并注明设备灯语义前提；但脚本该字段**硬编码旧口径** `does_not_cover: per-world light poses`（v1 实测），实现/采集窗不改文案则产物违约 | 实现卡或采集窗卡同步更新该文案（含 §10.3 前提），并在采集后自核 |
| O4 | 低 | §2.2 recipe 行措辞 | 行标写 recipe，正文却引「逐字同 `…/capture.json.parameters`」；recipe 本体（`scene_xml`/`script` sha）必然随新场景/脚本变化，实际只有 `parameters` 块可逐字同 | 明确为「`parameters` 块逐字同；recipe 的 scene_xml/script 按新场景与脚本记录」 |
| O5 | 低 | §10.2 预估口径 | 「新增 exe 与参考输入（预估 +14 条：新 tag 5 + 场景变体 1 + 脚本/模块 2 + 冻结器/证据若干）」未列明 +14 是否含新增 exe，「若干」未定数 | 已声明「以窗口卡冻结清单为准」，可在窗口卡落实际条数时一并写清 |

备注（非观察）：体积估算「≈5.55 MB/tag」与实测合计 **5,543,106 B（5.54 MB 十进制）** 相差 0.1%，属取整口径、不影响。

## 2. 逐项结果

### 2.1 签名对象（PASS）
- `spec.md` 实测 sha256 **`1821be98…`** / **16,625 B**，与 `receipt.spec.sha256/bytes`、`manifest.txt` 行**三方逐项一致**。
- `receipt.json`：`status=draft_pending_user_signature`、`effective=false`、`kind=criterion-spec-draft`、`task_id`=/卡 id（机读一致）；`date=2026-11-06`、`owner=Arch`。
- 证据目录 3 文件（spec/receipt/manifest），manifest 首行 `#`、**2/2** 条目复算通过、不自列；三文件 UTF-8/LF/无 BOM/裸 CR·TAB 0。

### 2.2 内容自洽（PASS）
- **结构**：15 节齐全（适用/边界 → 参考范围 → 事实地基 → 判据 A/B/C → v2.2 关系 → 失败条件 → 采集方案 → 验证方案 → 签名 → GO → 风险 → 变更控制 → 非声明）。
- **参考范围**：scene-a-light 变体、tag `nw3-light-hinge02040`、out-root `scene-a-light/perworld/`、nworld=3、hinge 0/20/40、仅 `cam-persp` 320×240、`cam-track` 不启用（`cam_active=[true,false]`）。
- **阈值算术**：0.5% × 76,800 = **384**；1.5 × 384 = **576**；320×240 = **76,800**，均复算正确；`[384,576)` 与 `<384` 分档处置写死。
- **recipe 对齐**：§2.2 列的 8 个 `parameters` 值与 v1 参考 `capture.json.parameters` 逐项一致（`use_fast_math=false`、`use_shadows=true`、`samples_per_pixel=1`、`enabled_geom_groups=[0,1,2,3]`、`background_color=[0,0,0,1]`、`shadow_light_fraction=0.3`、`flex_render_smooth=true`、`use_precomputed_rays=true`）；v1 `context.resolved.cam_res=[[320,240]]`、`total_rays=76800` 实证分辨率。
- **产物布局与体积**：v1 参考实测 rgb 2,765,524 / depth 922,336 / seg 1,843,924 / state 2,486 / capture.json 8,836 B，合计 5,543,106 B，与 §2.3 列值一致（体积取整见上备注）。
- **事实地基**：§3 三灯数值与 prep 证据 `light_pose_precheck` 逐值一致（sun 10.745878447°/21.255926664°、lamp 0.222378165/0.437999482 m、spot 0/0），spot 恒 0 的几何原因（pos/dir 落在 tri_body z 旋转轴）在位。
- **不继承 v2.2**：明写不继承两条已签清单（`cfd1038a…`/183 行、`14893fc6…`/172 行）与 `|ULP|≤128`，且 10 个回归实体名称逐一列全、声明其继续按 v2.2 判定。
- **失败即停**：§8 为 7 条编号项，覆盖帧内位级差/两跑不一致/判别力不足/采集余量不足/灯字段位级失败/未申报放宽/确定性或既有参考改动；处置写死「保留现场、停手报用户、不重试不改阈值不换参考不调步进」。
- **变更控制与非声明**：签名对象任何改动产生新 sha 需重签；未签不生效、不构成 L v2 启动、不产参考、不宣称覆盖/上游等价/性能/渲染数值/发布依据。

### 2.3 签名 / GO 清单（PASS）
- **S-1/S-2/S-3 各出现 1 次且内容明确**：S-1 = spec sha+bytes；S-2 = 采集窗计划用户双签（计划 + 开窗前三点基线 + signed floor 取严 + authoring 例外是否预批 util<60）；S-3 = 验证窗计划用户双签 + Chief token。
- **G-L1/G-L2 各 1 次**：采集窗前置（S-1+S-2 已签、spec/receipt/manifest 落库、脚本参数化 verified、QA 复核、基线/floor、材料贴群）；验证窗前置（采集证据 verified + ≥576 + 实体 verified + QA PASS + guards 冻结 + 基线/floor + 零编译纪律）。
- **数量事实核对**：§10.2「既有 perworld suite 96 条（11 exe + 83 输入 + 2 清单）」与归档 `perworld-impl-s1/window-attempt2-invalid/guards-perworld-s1.json` 本体一致（counts 11/83/2/total 96），且其 10 个回归实体与 §7 清单逐一吻合。

### 2.4 边界与证据（PASS）
- `git diff b6f5b724..1ef5bd5b` = 恰 **8 个路径**（草案卡 + 4 个 gate runs + spec/receipt/manifest）；`fixtures/`、`src/`、`scripts/`、`tests/`、criterion v2.2 **零 diff**；未采集、未新增参考。
- 被复核卡 `status=verified`、`revision.before=42748ac7…`、checks 含四门禁；其 `unit` gate 记录 **1359 passed / 0 failed / 400 ignored**。

### 2.5 与 prep 证据的对齐（补充核）
- 判据 A（rgb 0 字节差 / depth 位级 mismatched=0 / seg 全等 / 两跑 sha）与 perworld S1 实际口径一致，且明写不引用 v2.2 清单洗通过；判据 B（`light_xpos/light_xdir` f32 位型 0 例外）与 `state.npz` 形状 `[nworld,nlight,3]` 一致（v1 实测 `light_xpos` 为 `(3,3,3)`，入径正确）；判据 C 参考侧自算（避免循环论证）口径正确。

## 3. 观察清单（低，不阻断）

见 §1 表格 O1–O5。**O2/O3 建议在 G-L1 材料或 S-2 采集窗计划里写成显式前置项**（补 `command`、设 `MJWARP_CAPTURE_AT`、更新 `per_world_scope` 文案），避免「spec 已签、产物不符」。其余为措辞/预估口径，可在签名卡整理时顺手明确，不返工。

## 4. 证据清单

| 路径 | 说明 |
| --- | --- |
| `scripts/verify_light_criterion_review.py` | 预审脚本（21 项只读复算，可复跑） |
| `logs/verify-criterion-review.json` | 21 项检查机器可读结果（PASS 21/21） |
| `logs/verify-criterion-review.log` | 脚本原始输出 |
| `artifacts/signature-object.json` | spec sha/bytes 三方一致与机读状态 |
| `artifacts/scope.json` | 变更面 8 路径 + 零越界探针 |
| `artifacts/evidence-selfcheck.json` | manifest 2/2（3 文件）与编码 |
| `artifacts/content-consistency.json` | 15 节、阈值算术、参考范围要点 |
| `artifacts/recipe-parameters.json` | 8 个 parameters 与 v1 对齐、320×240 实证 |
| `artifacts/fact-base-vs-prep.json` | 三灯数值与 prep 证据逐值对齐 |
| `artifacts/v22-non-inheritance.json` | 不继承 v2.2 清单核对 |
| `artifacts/fail-stop.json` | 失败即停 7 条 |
| `artifacts/checklists.json` | S-1/S-2/S-3、G-L1/G-L2 唯一性 |
| `artifacts/perworld-suite-fact.json` | 96 条 suite 与归档 guards 对齐 |
| `artifacts/capture-interface-gaps.json` | O2/O3 接口差数据（command/采集时间/per_world_scope 旧文案） |
| `artifacts/npz-shape-vs-spec.json` | O1 形状记法 vs 实际逐 world 键 |
| `artifacts/volume.json` | 体积实测与取整 |
| `artifacts/reviewed-card.json` | 被复核卡状态与 unit 1359/0 |
| `README.md` / `receipt.json` | 本报告与机器可读回执 |
| `manifest.txt` | 本目录 sha256 + bytes（首行 `#`，不自列） |

## 5. 边界与非声明

- 只读预审 + 只写本卡证据目录；不改被复核 spec/receipt/manifest、既有证据、代码、fixtures、criterion v2.2。
- 本预审**不构成用户签名**、不产生生效判据、不构成 perworld L v2 启动；不写覆盖成立 / 上游等价 / 渲染数值结论 / 发布依据。
- 用户选定 L-v2 方向后，S-1 由 Chief 整理签名卡、用户签名方生效。

## 6. 复现

```powershell
cd D:\Rust\mjwarp-rs
target/mjwarp-parallel/venv/Scripts/python.exe -B docs/development-evidence/render-bvh/perworld-l-v2-light-criterion-check/scripts/verify_light_criterion_review.py
# 期望末行：SUMMARY PASS 21/21
```
