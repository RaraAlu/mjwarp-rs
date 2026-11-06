# perworld L v2 灯备选准备片 QA 只读复核（准备片 1）

**结论：PASS-with-observations（0 阻断；7 条低观察：执行方 O1–O4 + 本轮新增 O5–O7）**

- 任务：`2026-11-06-render-perworld-l-v2-light-prep-check`（owner QA、P8-渲染与BVH、p55；QA 自登记，依赖准备片）
- 复核目标：HEAD **`b0b8cc02`**（准备片卡 `verified`；登记 `4ae66c81` → 补 files `b6b8bd89` → start `a27e54a7` → 实现 `b0b8cc02`）
- 基线（改动前）：`b0b8cc02^`（= `a27e54a7`；脚本 sha `67a760fa…`/24,008 B/499 行）
- 复核方式：只读 + 独立复算（不采信执行方日志结论；另做「同树旧脚本 vs 新脚本」控制变量对比与证据日志复跑比对）；QA 脚本自检 **22/22 PASS**；CPU-only、零 GPU、不写参考
- 本卡登记提交：`11371d9f`

## 1. 结论与定级

准备片 1 的 5 项要求全部成立：**默认行为逐字段不变（仅 4 项客观差异且各有理由）→ scene-a/src/criterion 零越界 → 行数棘轮 497+160+84（≤500）→ dry-run 三自检可复现且数值与证据一致、负例 exit 1 → 证据 16 条 manifest 自核/编码/非声明在案**。7 条观察均只涉及占位/命名/模板/措辞/健壮性，**不影响判据、数值与验收结论**，按 Chief 定级规则记 observation、不返工。

| 编号 | 级别 | 区域 | 摘要 |
| --- | --- | --- | --- |
| O1 | 低 | 死代码占位 | `perworld_light_prep.py` 返回体 `"world_hinge_deg": [float(v) for v in []]` 恒空，随后被调用方覆盖为真实值（执行方已记，不返工） |
| O2 | 低 | diff 分类命名 | `default-dryrun-diff.json` 把 `legacy_manifest_entries 106→107` 与 script sha/bytes 一律归为 `allowed-new-param-echo`，命名不精确（执行方已记） |
| O3 | 低 | TASK_ID 旧卡 id | 脚本 `TASK_ID = "2026-10-30-render-perworld-capture-authoring"` 沿用旧卡；采集窗前需决定「参数化 / reference_tag」（执行方已记） |
| O4 | 低 | spot 恒 0 | `spot` 的 pos=0 0 2、dir=0 0 -1 落在 `tri_body` z 旋转轴上，三 world 位移恒 0；仅 `sun`/`lamp` 有判别力（场景几何事实，执行方已记） |
| O5 | 低 | 卡面措辞 | 卡 acceptance #3 写三灯「增加 `body=\"tri_body\"`」，与实现（把三灯**嵌套**进 `<body name="tri_body">`）不符；MJCF 无 `<light body=…>` 属性（实测 schema 拒绝），实现方式正确、Chief 复核口径也是「嵌套」；acceptance 不可改写，仅记录 |
| O6 | 低 | 场景差异面 | 除三灯挂载外，变体还多出 1 段说明注释、并删掉 scene-a 末尾 1 个空行（纯注释/空白，`scene.diff` 可见）；不影响 MJCF 语义与属性逐字一致结论 |
| O7 | 低 | 健壮性 | `--scene`/`--out-root`/`--out-dir` 指向仓库**之外**的合法路径时，`_rel()`/`relative_to()` 抛 `ValueError` traceback（exit 1，但非 `fail()` 干净文案）；本片 acceptance 不涉及、属建议项 |

## 2. 逐项结果

### 2.1 默认行为不变（PASS）

- **同树控制变量对比**（当前工作树，旧脚本 sha `67a760fa…` 临时置回真实路径运行后立即恢复，恢复 sha 校验 = `3e1d207c…`）：
  `capture_render_perworld.py --dry-run --out-dir fixtures/render-bvh/scene-a/perworld/prep-default-dryrun` 两侧 exit 0、stderr 0；逐叶字段 diff = 新增 `/paths` 7 叶 + 变更 `/recipe/script/sha256`、`/recipe/script/bytes`，**removed=0、其余全部相同**（`legacy_manifest_entries` 两侧同为 107、`--repeat` 两侧同为 2）→ 脚本行为只多了路径回显与自身指纹（`artifacts/default-behavior-diff.json`）。
- **legacy 106→107 归因**：`git ls-tree` 统计 `fixtures/render-bvh` 排除 `scene-a/perworld/`：`b0b8cc02^` = **106**、`b0b8cc02` = **107**，新增集合恰为 `fixtures/render-bvh/scene-a-light/scene.xml` 一个文件 → 与 `--out-root` 参数化无关（`artifacts/legacy-counts.json`）。
- **证据日志复跑比对**：被复核 `logs/after-default-dryrun.json` 与本次复跑**逐叶完全一致**；`logs/before-default-dryrun.json` 与同树复跑仅差 `/legacy_manifest_entries`（106 vs 107，旧树无新夹具），其余逐叶一致（`artifacts/evidence-logs-reproduced.json`）。

### 2.2 边界零越界（PASS）

- `git diff --name-only b0b8cc02^..b0b8cc02` = **25 个路径**，全部落在「卡 + runs + 准备片证据 + 2 个脚本 + 1 个场景 XML」；`src/`、`fixtures/render-bvh/scene-a/`、`scene-a/perworld/nw3-hinge02040/`、`docs/.../sdf-criterion-v2.2/` **零 diff**；无 `scene-a-light/perworld/`。
- `fixtures/render-bvh/scene-a-light/` 恰 **1 个文件** `scene.xml`。
- 全部 dry-run/负例运行后 `git status --porcelain -- fixtures/` 为空、未创建 `prep-default-dryrun`/`scene-a-light/perworld` → 不写参考（`artifacts/scope.json`、`artifacts/negative-checks.json`）。

### 2.3 场景变体等价性（PASS）

- 三灯同名 `sun`/`lamp`/`spot`，属性字典**逐值相等**（sun 12 / lamp 14 / spot 16 个属性，含 type/pos/dir/diffuse/specular/ambient/attenuation/cutoff/exponent/softness/castshadow/active/bulbradius/intensity/range）；变体中三灯父元素均为 `tri_body`；**移除三灯后两份 XML 结构完全相等**；相机不变（`cam-persp` mode 0、`cam-track` mode 3，dry-run `model_static.cameras=[[0,0,"cam-persp"],[1,3,"cam-track"]]`）（`artifacts/scene-compare.json`）。

### 2.4 行数与采集语义（PASS）

- git blob 换行口径：`scripts/capture_render_perworld.py` **497**（基线 **499**）、`scripts/perworld_light_prep.py` **160**、`scene.xml` **84**，均 ≤500；sha256/bytes 与 `artifacts/prep-meta.json` 逐项一致（`artifacts/line-counts.json`）。
- 脚本 diff 仅 6 处：docstring 示例、模块 import、`legacy_manifest(out_root)` 参数化、`--scene/--out-root` 两个参数、dry-run plan 移入模块、捕获路径两处 `legacy_manifest(out_root)`；`--repeat` 仍 `choices=(1,2), default=2`，`--worlds 3`、`--hinge-deg 0,20,40`、`--seed` 默认不变，recipe 字段集合/判据/容差未动（`artifacts/script-semantics.json`）。

### 2.5 dry-run 三自检 + 负例 + 零 GPU（PASS）

- 显式 light dry-run：`--scene fixtures/render-bvh/scene-a-light/scene.xml --out-root fixtures/render-bvh/scene-a-light/perworld --out-dir …/nw3-light-hinge02040 --worlds 3 --hinge-deg 0,20,40` → **exit 0**；① `model_static`（njnt 2、nq 8、jnt_type/jnt_qposadr/qpos0、相机 mode）；② `light_pose_precheck`：三灯 attached_body 均 `tri_body`、`per_world` 3 条、`diff_keys` = **sun 10.745878447° / 21.255926664°、lamp 0.222378165 m / 0.437999482 m、spot 0/0**、`verdict.nonzero=true`（与被复核证据逐值一致）；③ `paths` 回显默认/显式、`scene_explicit/out_root_explicit=true`、生效 out_dir 在 out-root 之下；默认 run **不含** `light_pose_precheck`（默认行为不变）。
- 负例：`--out-dir` 已存在非空 → **exit 1**（"already exists and is not empty"）；`--out-dir` 不在 `--out-root` 下 → **exit 1**（"outside --out-root"）；两者无副作用。
- 零 GPU 静态证据：dry-run 分支调用 `check_environment(want_cuda_device=False)`，`wp.get_device("cuda:0")` 只在 `if want_cuda_device:` 分支内；dry-run 只用 CPU `mujoco.MjData`+`mj_forward` 做灯位姿前置自算，不渲染、不落参考。
- **不做渲染判别力断言**：本片输出无渲染数值断言；`spot` 恒 0 只记录并写明几何原因（O4）。

### 2.6 证据自核（PASS）

- `perworld-l-v2-light-prep/manifest.txt`：首行 `#`、**16 条**逐条 sha256/bytes 独立重算通过、不自列（目录共 **17** 文件含 manifest 自身）；17 个文件 UTF-8/LF/无 BOM/裸 CR·TAB 0。
- `receipt.json /task_id` = `2026-11-06-render-perworld-l-v2-light-prep`；非声明在位（不构成 L v2 启动、不产参考、不宣称灯位姿覆盖/上游等价/渲染数值结论）；证据目录内三禁词 0 命中。
- 被复核卡 `status=verified`、`revision.before=b6b8bd89…`、checks 含四门禁（`artifacts/evidence-selfcheck.json`）。

## 3. 观察清单（全部低、不阻断）

见 §1 表格 O1–O7。执行方 O1–O4 按 Chief 指示记录不返工；O5–O7 为本轮新增，均只涉及措辞/注释/健壮性，**不要求返工**（O3 建议在采集窗开启前定参数化/reference_tag 口径）。

## 4. 证据清单

| 路径 | 说明 |
| --- | --- |
| `scripts/verify_light_prep_review.py` | 复核脚本（22 项只读复算；含同树旧/新脚本对比与证据日志复跑比对，可复跑） |
| `logs/verify-light-prep.json` | 22 项检查机器可读结果（PASS 22/22） |
| `logs/verify-light-prep.log` | 脚本原始输出 |
| `artifacts/scope.json` | 变更面 25 路径 + 零越界探针 + scene-a-light 目录 |
| `artifacts/line-counts.json` | 497/160/84 与基线 499、prep-meta 一致性 |
| `artifacts/legacy-counts.json` | legacy 106→107 的 +1 归因 |
| `artifacts/scene-compare.json` | 三灯属性/父元素/结构等价 |
| `artifacts/script-semantics.json` | 默认参数、repeat/worlds/hinge/seed、TASK_ID |
| `artifacts/default-behavior-diff.json` | 同树旧/新脚本逐叶 diff（仅 3 项客观差异） |
| `artifacts/evidence-logs-reproduced.json` | 被复核 before/after 日志与复跑比对 |
| `artifacts/light-precheck.json` | 显式 light dry-run 三自检数值/路径/相机/legacy |
| `artifacts/negative-checks.json` | 两负例 exit 1 与无副作用 |
| `artifacts/evidence-selfcheck.json` | manifest 16/17、编码、task_id、禁词、卡状态 |
| `README.md` / `receipt.json` | 本报告与机器可读回执 |
| `manifest.txt` | 本目录 sha256 + bytes（首行 `#`，不自列） |

## 5. 边界与非声明

- 只读复核 + 只写本卡证据目录；不改被复核脚本/场景/证据、既有证据、代码、fixtures、criterion。
- 复核期间为控制变量曾临时把旧脚本内容写回 `scripts/capture_render_perworld.py` 并在同一进程 `finally` 恢复（sha 校验一致）；工作树最终 `fixtures/`、`scripts/` 零改动。
- 本复核不产生新验收结论、不构成 perworld L v2 启动；不写灯位姿覆盖成立 / 与上游实现等价 / 任何渲染数值结论 / 发布依据。

## 6. 复现

```powershell
cd D:\Rust\mjwarp-rs
target/mjwarp-parallel/venv/Scripts/python.exe -B docs/development-evidence/render-bvh/perworld-l-v2-light-prep-check/scripts/verify_light_prep_review.py
# 期望末行：SUMMARY PASS 22/22
```
