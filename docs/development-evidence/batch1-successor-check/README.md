# 批次 1 后继四条独立复核（G27@P7 / M08@P7 / G05@P7 / G28@P8）

任务：`2026-10-23-batch1-successor-check`。日期：2026-10-23。执行：QA（独立复核，非交付人）。
对象提交：`a1fa5b5`（G27@P7）、`b92e55d`（M08@P7）、`e7a7859`（G05@P7）、`00b8b60`（G28@P8）。
范围：只读复核 + 本证据目录；未改产品/测试/容差/fixtures/回执，未碰 Backend 正在做的批次 2。

## 1. 结论（先给）

- **四条交付的数值声明均被证据与独立复跑支持**：g27-p7（2 案例 × 8 组常量逐位一致、原生 max 1.19e-07≤1e-5）、m08-p7（6 用例 × 9 字段逐位一致、原生 max 0）、g05-p7（transmission 与端到端三组逐位一致、原生 9.54e-08≤1e-5）、g28-p8（2 世界 × 5 射线 geomid 一致、dist/normal 1e-6 内、原生 1.19e-08）。
- 独立复跑（CPU，默认特性）：`lib qpos` **5/0**、`lib extended` **3/0**、`lib transmission` **3/0**、`lib batch` **11/0**，全过。
- 模型源 sha256 与 4 个 `authoring.json` **4/4 一致**；交付提交内 evidence/fixtures/source blob 与回执一致（唯一差异为共享模块索引回执 blob，属已登记的已知追溯项）。
- checks 均含 fmt/clippy/unit/unchanged/json/forbid_claim 且 gate 记录 passed；**acceptance 未被放宽**（细化提交仅追加切片说明与边界，交付提交 0 行改动）。
- **G28@P8 模块范围从 `mjwarp.render` 改落 `mjwarp.physics.core` 的理由成立**：本片实现位于 `src/physics/query/batch.rs`，未引入 render/BVH；证据 §1 明确说明映射依据并在 §6 把 BVH/RenderContext、mesh/hfield/flex/SDF、过滤与设备视图列为未验收边界（见 §5）。
- 无阻断问题；仅 1 类一般级已知追溯差异（4 个共享模块索引回执 blob），与 `2026-10-23-receipt-practice-fixup` 登记的口径一致，不重复修正历史回执。

## 2. 四条声明逐项核对

| 任务 | 声明 | 证据位置 | 复核结果 |
| --- | --- | --- | --- |
| g27-p7 | 2 案例 × 8 组常量逐位一致（容差 0） | fixture `qpos-dependent.json`：2 cases、`tolerance.upstream_abs=0`；`src/physics/constants/qpos/tests.rs` 对 tendon_length0/cam_pos0/cam_poscom0/cam_mat0/light_pos0/light_poscom0/light_dir0/meaninertia 逐项 `assert_bits` | ✅ 复跑 `lib qpos` 5/0 |
| g27-p7 | 原生 max abs 1.19e-07 ≤ 1e-5 | `authoring.json.native_cross_check.max_abs_vs_native_mj_setConst=1.1920928955078125e-07`；fixture `native_abs=1e-5`；回执同值 | ✅ 一致 |
| m08-p7 | 6 用例 × 9 字段逐位一致（容差 0） | fixture `extended-cases.json`：6 cases、`tolerance.upstream_abs=0`；`src/io/state/extended/tests.rs` 断言 time/qpos/qvel/ctrl/act/qfrc_applied/xfrc_applied/mocap_pos/mocap_quat 九字段 | ✅ 复跑 `lib extended` 3/0 |
| m08-p7 | 原生 max abs 0 | `authoring.json.native_cross_check.max_abs_vs_native_mj_resetData=0.0`；回执同值 | ✅ 一致 |
| g05-p7 | transmission + 端到端三组逐位一致 | fixture `transmission-cases.json`：`transmission_reference`（length/velocity/moment）+ `expected_actuation`（act_dot/actuator_force/qfrc_actuator）；测试对两组均 `assert_bits`（`transmission/tests.rs:186-196`） | ✅ 复跑 `lib transmission` 3/0 |
| g05-p7 | 原生 max abs 9.54e-08 ≤ 1e-5 | `authoring.json.native_cross_check.max_abs_vs_native=9.536743172944284e-08`；回执同值 | ✅ 一致 |
| g28-p8 | 2 世界 × 5 射线 geomid 一致；dist/normal 1e-6 内 | fixture `batch-cases.json`：nworld=2、nray=5、`tolerance.dist_abs/normal_abs=1e-6`；两世界 `hits` 均含 2 条无命中且世界 1 旋转后结果不同；测试断言 geomid 相等 + 容差比较 | ✅ 复跑 `lib batch` 11/0 |
| g28-p8 | 原生 max abs 1.19e-08、geomid 0 不符 | `authoring.json.max_abs_dist=1.1920928966180355e-08`、`geomid_mismatch=0` | ✅ 一致 |

补充：四条 README 都如实列出未验收边界（g27 其余 invweight0/eq_data0/执行器常量；m08 G24 历史/其余 Data 字段；g05 free/ball/JOINTINPARENT 与 tendon/site/body 传动、设备内核；g28 BVH/过滤/设备视图），并明确「不宣称完整族或精度验收通过、NP04 未测」。

## 3. 独立复跑（CPU，默认特性）

```text
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --lib qpos          -> 5 passed; 0 failed
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --lib extended      -> 3 passed; 0 failed
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --lib transmission  -> 3 passed; 0 failed
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --lib batch         -> 11 passed; 0 failed
```

均为 host 侧切片；复跑不启动 Python、不重跑 authoring。复跑时 Backend 的批次 2 WIP 尚未进入 `src/`（仅任务 JSON 与 fixture 目录），lib 编译目标与 HEAD 一致。

## 4. sha256 / 回执抽验

- 模型源 sha256 与 `authoring.json`：`set-const/qpos-model.xml`（`707a599f…`）、`local-reset/extended-model.xml`（`413e5728…`）、`actuation/model.xml`（`f6d484da…`）、`ray-query/batch-model.xml`（`e03a503b…`）**4/4 一致**。
- 输出 fixtures 存在且被测试读取：`qpos-dependent.json`（2 cases）、`extended-cases.json`（6 cases × 9 字段）、`transmission-cases.json`、`batch-cases.json`（2 世界 × 5 射线）。
- 回执 `files[].blob` 与交付提交 `git ls-tree` 比对：除 4 个共享模块索引（`.normify/modules/mjwarp/{physics,io}/core.md`）外**全部一致**；模块索引差异属 `2026-10-23-receipt-practice-fixup` 已登记的已知追溯项（done 刷新导致），不重写历史回执。

## 5. G28@P8 模块范围改落 physics.core 的核对

- 变更：`1ed0f7a` 把该任务 `modules` 从 `["mjwarp.render"]` 改为 `["mjwarp.physics.core"]`，同时 files 收窄到 `src/physics/query.rs`、`src/physics/query/`、`fixtures/ray-query/`、`scripts/authoring/`、证据与回执。
- 实现位置：`src/physics/query/batch.rs`（`rays_batch`），复用 P7 `ray` 逐世界求交，未引入 render/BVH 代码或依赖。
- 证据说明：`docs/development-evidence/backlog-batch1/2026-10-11-roadmap-g28-p8/README.md` §1 明确写「原任务模块范围指向 mjwarp.render（BVH）；按当前代码布局本片不引入 render/BVH，只做批量；映射为 mjwarp.physics.core 并在 §6 登记 BVH/过滤/设备视图边界」；§6 逐项登记未验收边界。
- 判定：**理由成立**——模块映射应与实际代码归属一致；原 render/BVH 映射对应的能力未在本片实现，已作为未验收边界明确登记，未以改名替代实现。验收口径只增不减（细化提交新增切片说明与边界，交付提交 acceptance 0 行改动）。

## 6. checks / acceptance 核对

- 四条 checks 均含 gate fmt/clippy/unit、unchanged、json（回执 task_id）、forbid_claim；`.normify/runs/<task>/` 记录 fmt/clippy/unit 均 `passed=true`。
- 细化提交对 acceptance 的改动只有：末条「再验收适用NP03与NP04。」改为带逗号的连接行 + 追加切片说明/边界条目（新增 48 行级内容）；无删除验收要求、无放宽容差。交付提交对 acceptance 0 行改动。

## 7. 差异清单（按严重级别）

**① 一般（已知追溯项）：4 个共享模块索引回执 blob 与提交版本不一致**
- g27-p7/g05-p7/g28-p8 的 `physics/core.md`、m08-p7 的 `io/core.md`：回执为 done 刷新前/中间版本。与批次 1 前四条同源，已在 `batch1-independent-check/README.md` §9「回执口径实践」登记为已知差异，本复核不重复修正历史回执。

**② 建议：authoring 未独立重生成**
- 位型一致性与原生 max_abs 依赖 `authoring.json` 与冻结 fixture 自洽；重生成需 pinned venv + GPU，按复核范围未重跑（未占 GPU）。4 个 `author-*.py` 均 pin `71da24d…`。

**③ 无阻断/严重问题**：未发现伪造、容差放宽、断言弱化、fixture 与证据不符、CPU 复跑失败。

## 8. 边界

- 只做独立复核与证据记录；未改产品/测试/容差/fixtures/回执；未 start/回退任务。
- 未重新生成 offline authoring 参考（需 pinned venv + GPU）；未复核 GPU 内核路径（四条均为 host 侧切片）。
- 未碰 Backend 正在做的批次 2（g07 等 8 条）。

## 9. 证据索引

- 本文件：逐条核对、复跑计数、sha256 抽验、G28 模块范围核对、差异清单。
- 被复核证据：`docs/development-evidence/backlog-batch1/<task-id>/{README.md,authoring.json}`、`docs/development-evidence/changes/<task-id>.json`、`.normify/runs/<task-id>/`。
