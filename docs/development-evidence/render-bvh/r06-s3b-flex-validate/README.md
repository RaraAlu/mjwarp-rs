# R06-S3b② flex 渲染验证窗材料（离线；未开窗、未冻结）

任务：`2026-11-04-render-r06-s3b-flex-window-material`（p75，owner Arch；depends `2026-11-04-render-r06-s3b-flex-capture-r2`）。
性质：**只写窗口材料**（实体表/判据/守卫预期/Baseline 公式/失败停手/开窗前提），**不开 GPU 窗口、不执行真冻结、不写代码**。
参考数据来源：`2026-11-04-render-r06-s3b-flex-capture-r2`（提交 `71ec7c83`，卡 verified）产出的 `fixtures/render-bvh/scene-flex/reference/base/`。

## 1. 本片交付物

| 交付 | 路径 | 说明 |
| --- | --- | --- |
| 实体表 | `materials/entity-table.json` | 3 相机实体 + 1 flex 覆盖断言 + 回归集占位；判据逐条带出处 |
| 窗口计划 | `materials/window-plan.txt` | 逐字命令形态、守卫预期、floor 公式、时长、失败回滚 |
| 基线快照 | `materials/gpu-baseline.txt` | 只读 GPU/工具链/HEAD 快照；**签署基线必须开窗前 5 分钟内重取** |
| 回执 | `receipt.json` | 交付清单 + 依赖缺口 + 非声明 |

## 2. 参考与场景事实（取自采集片证据，不重算）

- 场景 `fixtures/render-bvh/scene-flex/scene.xml`：2 条 1D flex（`flex-line` r=0.02 Z 字线 4 顶点/3 边；`flex-line-fat` r=0.04 单边 2 顶点/1 边）+ 刚性几何（floor/box/mesh/capsule）+ 3 相机（`cam-persp`/`cam-track`/`cam-flex`）+ 2 灯（sun/lamp，均 castshadow）。
- flex 关键量：`nflex=2`、`flex_dim=[1,1]`、`flex_radius=[0.02,0.04]`、`flex_vertadr=[0,4]`、`flex_edgeadr=[0,3]`、`flex_edgenum=[3,1]`、`flex_elemadr=[0,3]`、`flex_shellnum=[2,2]`；`rgba=[[0.9,0.5,0.1,1],[0.2,0.4,0.9,1]]`；`bvh_nflexgeom=4`、`flex_geom_flexid=[0,0,0,1]`、`flex_geom_edgeid=[0,1,2,3]`。
- 参考 recipe（与产品同 recipe）：`use_fast_math=false`、`use_shadows=true`、`samples_per_pixel=1`、`flex_render_smooth=true`、`use_precomputed_rays=true`、`render_skybox=false`、`enable_backface_culling=true`。
- 参考文件 sha256：`rgb.npz 5ed50287…`、`depth.npz c3ee4adc…`、`seg.npz f3b00fa8…`、`state.npz 7f2fff50…`；`capture.json` `4713e2c7…`（两跑逐字节一致）。
- flex 覆盖实测（参考侧）：`cam-flex` = **654** 个 `(object_id, 9)` 像素（flexid 0/1 均出现）；`cam-persp`/`cam-track` 各 0（预期，flex 不在其视野）。
- 采集环境：python 3.12.13 / warp 1.15.0 / mujoco 3.15.0 / mujoco_warp 3.15.0 / numpy 2.5.4；GPU RTX 4070 Ti SUPER、driver 596.36；pinned upstream `71da24d…` clean。

## 3. 实体表（详见 `materials/entity-table.json`）

| # | 实体 | 类型 | 判据来源 |
| --- | --- | --- | --- |
| 1 | `render_flex_dim1_camflex` | **new**（flex 主实体，flex 覆盖 654 像素） | R05/R02 家族口径 + S3a 参考语义 |
| 2 | `render_flex_dim1_cam_persp` | new（同场景、flex 不在视野，等价刚性路径对照） | 同上 |
| 3 | `render_flex_dim1_cam_track` | new（同上，targetbody 相机） | 同上 |
| 4 | `render_regression_suite` | 回归（既有 render 实体集零漂移） | 既有位级基线 |

判据**零放宽**：rgb `mismatched=0 / max_byte_diff=0 / beyond_1_lsb=0`（bit-first）；depth `mismatched=0 / max_ulp≤4 / beyond_0_ulp=0`（bit-first）；seg 0 差异且 `(flexid, 9)` 样本计数与参考一致（cam-flex 654）；两跑 sha 相同；参考判读 `geom_id=-2`、`mesh_id=flexid`、depth `dist * -ray_dir_local_cam[2]`、颜色 `flex_rgba[mesh_id]`。

## 4. 开窗前提（逐条核，缺一不开）

| # | 前提 | 当前状态 |
| --- | --- | --- |
| 1 | 参考数据收口（采集卡 verified） | ✅ `71ec7c83` |
| 2 | **设备侧 GPU 用例与 `render_with_flex` 生产调用点**（见 §5 缺口 A） | ✅ `0a3058ee`（卡 `…-s3b-flex-device-harness` verified） |
| 3 | **本 suite 的冻结器 + guards 文件**（见 §5 缺口 B） | ❌ 缺（开窗卡内落地） |
| 4 | 本材料 QA 独立复核 pass | 只读复核 **PASS-with-observations**（证据落库中，待 commit） |
| 5 | 开窗前 5 分钟三点基线达标、floor 取严、compute apps filtered=0、worktree clean | 窗口期 |
| 6 | 开窗前把「计划 + 基线 + floor」贴开发团队群留痕；Chief GO 且为唯一执行会话 | 窗口期 |
| 7 | **BOM 审计脚本已落库并通过**（`2026-11-04-governance-evidence-bom-audit`） | ✅ `f8a0fa75`（`scripts/validate/check-evidence-bom.ps1`；基线 128 non-`.ps1` WARN + 4 manifest 违规。曾因 S3b③ 新文件 `compile.log` 漂移到 129，已由 `d034eb43` 剥离回 128） |

## 5. 依赖缺口（本片不解决，需另行登记/派活）

- **缺口 A（代码，已闭合）**：`render_with_flex` 曾无生产调用点、仓内无 flex GPU 用例；已由 `0a3058ee`（`…-s3b-flex-device-harness`）补齐——`src/render/context/flex_host/render.rs::render_with` 为生产调用点（`flex=None`/`flex_mesh=None` 与父版同构等价），`tests/render_flex_validate.rs`（`#[ignore]`，`--no-run` 编译通过）读 `reference/base/` + `state.npz` 按 §3 判据比对；执行命令：`cargo test --locked --features cuda-probe --test render_flex_validate -- --ignored --test-threads=1`。
- **缺口 B（窗口工具）**：新 suite 需要 `scripts/validate/render-bvh/freeze-guards-r06-s3b-flex.ps1` + `guards-r06-s3b-flex.json`（R08 先例：9 exe + 输入条目，`-GuardsOnly` 断言 `GUARDS_OK=N/N`）。**需在开窗卡（S3b validate）内落地。**
- 缺口 A/B 未收口前**不得开窗**；本片交付的是「材料齐、可开窗条件清单」。

## 6. 窗口计划要点（详见 `materials/window-plan.txt`）

- 执行器唯一（`window-execution-policy.md`）：只有 Chief 直接 GO 给唯一执行会话才开；群内帖仅作 record-only。
- 流程：`monitor-until-clean.ps1`（常规 `-Required 60 -MaxMinutes 30`，默认 `MaxUtil=40`，**不带** `-OpenUtilLimit`）→ 冻结 `-Attempt 1` → `-GuardsOnly` 断言 → stage B 实体体。
- floor 公式：`floor_mib = max(开窗前三点基线 free_min − 3072, 已签 floor)`，取严者生效；采样点 free < floor、或 used 增量 >3072 MiB、或 util>40 即中止。
- **floor 必须显式传给 runner**：`run-window.ps1` 的 `-OpenFreeMiB` 默认 **10240**、`monitor-until-clean.ps1` 的「干净」判据也硬编码 free≥10240（**脚本没有 floor 参数**）。按本材料基线 14772 算出的当次 floor 是 **11700**——若照默认参数开窗，实际门槛比签署值松 1460 MiB。所以：① 开窗时必须传 `-OpenFreeMiB <当次 floor>`（数值按开窗前 5 分钟内三点基线重算，11700 只是示例）；② monitor 阶段执行者必须**人工逐点核对 free ≥ floor**，低于即停；`monitor-until-clean.ps1` 的 `-MaxMinutes` 默认 12，plan 显式传 30（「硬顶 30」是参数，不是脚本内置上限）。
- 时长预期：monitor 5–10 min（硬顶 30）+ 冻结/自检 1–2 min + 实体体 60–120 s ≈ 整窗 30–60 min。
- **失败即停**：monitor 不达标 / freeze 不符 / GuardsOnly 不符 / 任一实体红 → 停手报 Chief，不重试、不重冻「跑到过」、不放宽阈值；attempt 原文留档。
- **窗口规则 v2（Chief 2026-11-04 批准，attempt5 起生效）**：窗口第一步做**一次** canonical rebuild（`cargo test --locked --features cuda-probe --no-run --test render_flex_validate`，随后确认无构建进程）→ **立即记录该次 exe sha256+bytes**（PTX 由离线 precheck 记录）→ 随即 `freeze -Attempt 1` 冻结该产物（期望 `GUARDS_OK=16/16 exes=2 inputs=14`、aggregate 重算、exe 条目不再 `pending`）→ **冻结之后零编译**跑 monitor 与实体体；**冻结后出现任何编译/relink 即判 INVALID_ROUND**（停手、留档）。依据（实测）：`CARGO_INCREMENTAL=0` + `cargo clean -p mjwarp-rs` 连建两次产物同大小但 sha 不同（`d05e4c64…` vs `574f8969…`）；`-C link-arg=/Brepro` 会导致 `generic-array`/`serde` build script 编译失败、不可用；Backend 侧 canonical vs 强制 relink 亦不同（`3A096B52…` vs `3CF1EA02…`）。**exe 身份 = 冻结时刻的 canonical rebuild 产物**，任何预录 sha 仅作参考、不作冻结卡口。详见 `materials/window-plan.txt` §0b。

## 7. 窗口期附加检查（吸取本日两处教训）

- **EOL**：跑任何带 `docs/`/`fixtures/` 的 `unchanged` 前先跑 `scripts/validate/normalize-evidence-eol.ps1 -DryRun`，再实跑（实测存量 1,227 stale，`core.autocrlf=true` 检出后会复发）。
- **BOM**：开窗前跑 `scripts/validate/check-evidence-bom.ps1`（已落库，`2026-11-04-governance-evidence-bom-audit`）；基线 **128 non-`.ps1` WARN + 4 manifest 违规**（3 个 BOM + 非 BOM 的 `render-bvh/r05-summary/manifest.txt`），**新增或超基线即停窗报 Chief**；`manifest.txt` 第 1 字节必须是 `#`。基线漂移史：S3b③ 的 `compile.log` 曾带 BOM 使计数变 129，已由 `d034eb43` 剥离回 128。

## 8. 非声明

- 本片不宣称 flex 渲染数值可用；不宣称 NP01/NP02/R06 通过；不动判据/容差/参考/夹具；零 GPU。
- `fixtures/render-bvh/scene-flex/provenance.json.model_parity.known_difference` 记录的 `nbuffer`（312=37552 vs 315=37296、去 flex 对照 32368==32368）**未放宽、未宣称 parity pass**，判据分类裁定归 Chief/QA，不进入本窗口判据。
