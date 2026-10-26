# G6 验收映射 UNC-3 治理更新（组合层 13/13、分支层部分覆盖）+ B2/B1 证据观察勘误

**任务**：`2026-10-26-g10-p5-unc3-governance`（owner Arch，phase 治理/验收映射）。
**基准 HEAD**：`507a576c`（QA B2 复核证据收口提交）。**本次为纯文档/证据治理提交，不改 `src/`、`tests/`、`fixtures/`、`Cargo.*`，不改任何判据/容差/期望/已 verified 数字与结论。**

## 1. 触发与依据

- B1（plane 族 6 组合）交付 `249a4dbd`、关闭 `f578b474`；QA 独立复核 pass（阻断 0 / 观察 7，收口 `ee92abe5`）。
- B2（sphere/capsule 族 4 组合）交付 `6ab5cb22`、关闭 `f578b474`；QA 独立复核 pass（阻断 0 / 观察 7，收口 `507a576c`）。
- Chief 定稿（2026-10-26）：G6 map 的 UNC-3 更新放两项 QA pass 后的治理提交、带证据指针；UNC-3 只能写「组合层 13/13、分支层部分覆盖、未宣称全路径」。

## 2. UNC-3 更新（前 → 后）

| 项 | 更新前 | 更新后 |
| --- | --- | --- |
| 组合层 | 3/13（sphere-box / capsule-box / box-box） | **13/13**（+ B1 plane 6、B2 sphere/capsule 4） |
| 分支层 | 未区分 | **部分覆盖**：本 case 实际覆盖 4 类分支；**8 类分支未覆盖** + swap 几何顺序无 case |
| status | `partial` | **`partial`（不变）** |
| 表述 | 「其余 10 组合未实现」 | 「组合层 13/13 已实现且各有上游 GPU 对照 case；分支层部分覆盖；未宣称全路径」 |

**文件改动**
- `docs/development-evidence/g10-p5-acceptance-map/README.md`：第 3 条未覆盖项改写 + 新增 **§6 UNC-3 更新** 台账（13 组合逐行、8 类未覆盖分支逐条、swap 路径、债务、证据指针、边界）。
- `docs/development-evidence/g10-p5-acceptance-map/map.json`：`uncovered[UNC-3]` 的 title/implementation/measured/evidence_paths/reason/impact 按实际覆盖改写（status 仍 `partial`）；`summary.note` 的「primitive 3/13 组合」「primitive 其余 10 组合」两处同步改为「组合层 13/13、分支层部分覆盖」。其它 5 条 uncovered 与 12 条 requirement 未动。

## 3. 13 组合台账（治理口径，与 README §6 一致）

| 组合 | 覆盖片（提交） | case（nacon） | 结果 |
| --- | --- | --- | --- |
| plane-sphere / plane-capsule / plane-ellipsoid / plane-cylinder | B1 `249a4dbd` | 各 1（nacon 1） | 4/4 pass |
| plane-box | B1 `249a4dbd` | 1（nacon 3） | pass |
| plane-mesh | B1 `249a4dbd` | 1（nacon 3） | pass（穷举分支，4 顶点） |
| sphere-sphere / sphere-capsule / sphere-cylinder | B2 `6ab5cb22` | 各 1（nacon 1） | 3/3 pass |
| capsule-capsule | B2 `6ab5cb22` | 1（nacon 1） | pass（平行分支；GPU 侧因 FMA 收缩走非平行但 1 接触） |
| sphere-box / capsule-box | 既有 `g10-p5-primitive-narrowphase-cuda`（`9d72159c`） | 3 | pass（80/80 位级） |
| box-box | 既有 `g10-p5-multicontact-cuda`（`9a282074`） | 1（5 接触/world） | pass（391/392 位级 + pos 1 ULP） |

## 4. 8 类未覆盖分支（fixture 未触发；不宣称已验证）

1. `plane_convex` graph 分支（mesh `vertnum≥10`；B1 fixture 为 4 顶点走穷举）；
2. `sphere_sphere` 零距离 fallback（`dist==0`）；
3. `sphere_capsule` 投影 clamp 路径（B2 case proj=0）；
4. `sphere_cylinder` 端面 / 角点分支（B2 case 只触发侧面）；
5. `capsule_capsule` non-parallel 分支（fixture 未融合 det=0 → 平行 4 端点；GPU 融合 det≈2.6e-12 → 非平行但 1 接触）；
6. `plane_cylinder` degenerate x 轴回退（len_sqr 未触发兜底）；
7. `plane_ellipsoid` 非上游退化保护（零长/非有限未触发）；
8. `plane-box` 8 角点中 5 个未 detected；`plane-capsule`/`plane-cylinder` 其余接触候选未 detected。

**另**：swap 几何顺序（调用方按 `(capsule,sphere)` 等传入）分派已实现但无 case；device `contact.geom` 用调用方顺序、pinned 按类型序规范化，仅同序时一致（QA B2 观察 O5）。

## 5. QA B2 观察收口（纯文字勘误，不改数字/判据/结论）

| 观察 | 级别 | 处理 |
| --- | --- | --- |
| O1 | 一般 | B2 README §11 陈旧 `35ba212d` 句改 `5c191046`（`35ba212d` 保留在作废链）；support/main 行数 417/419 → 实测 424/429；清 tab 残留 |
| O2 | 一般 | B2 README §5「设备对照测试尚未运行」→ 窗口已跑通并指向 §10 |
| O3 | 一般 | B2 README §6 补 capsule-capsule 的 FMA 收缩机制，并写明「count 期望对编译期 FMA 开关敏感」风险（判据/期望未动） |
| O4 | 建议 | B2 README §9 补「本 case 实际覆盖 / 未覆盖分支」逐条清单 |
| O5 | 建议 | B2 README §9 补 swap 几何顺序路径无 case + `contact.geom` 调用方顺序说明；写入本回执 §4 |
| O6 | 一般 | B1 receipt `fix_regate_note` unit 计数 973 → **974**（与 `.normify/runs/.../unit.json`、commit message 一致） |
| O7 | 建议 | B2 README §12 记「attempt2 launcher 二进制 `23ae4d82` 未归档，等价性为源码 + proof 级，不宣称二进制级复现」 |

QA B1 的 7 条观察已在 B1 关闭前收口（`686dc61d`/`ee92abe5`），本任务不改其语义。

## 6. 证据指针

- 交付：`docs/development-evidence/g10-p5-primitive-plane-cuda/`（B1）、`docs/development-evidence/g10-p5-primitive-sphere-capsule-cuda/`（B2，含 §6 机制 / §9 分支 / §10 窗口 / §11 行序修复 / §12 收口）。
- 独立复核：`docs/development-evidence/g10-p5-primitive-plane-cuda-independent-check/`、`docs/development-evidence/g10-p5-primitive-sphere-capsule-cuda-independent-check/`。
- 既有片：`docs/development-evidence/g10-p5-primitive-narrowphase-cuda/`、`docs/development-evidence/g10-p5-multicontact-cuda/`。
- 本 map：`docs/development-evidence/g10-p5-acceptance-map/{README.md,map.json}`。

## 7. 边界（不得声明）

- UNC-3 仍为 **partial**：只写「组合层 13/13、分支层部分覆盖」，**不宣称全路径 / 全分支 / 跨平台 / 精度结论 / 与上游等价**。
- 本 map 只做证据映射；不改任何已 verified 数字、判据、容差、期望；不覆盖 QA 未重跑的 device 侧结论。
- capsule-capsule 的 count 对编译期 FMA 收缩敏感（B2 README §6）：属已知风险记录，不改判据。
