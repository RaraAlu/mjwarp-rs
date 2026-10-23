# 批次 1 前四条独立复核（G27@P2 / M08@P3 / G05@P4 / G28@P7）

任务：`2026-10-23-batch1-independent-check`。日期：2026-10-23。执行：QA（独立复核，非交付人）。
对象提交：`70b36b7`（G27）、`e63b296`（M08）、`9850ee0`（G05）、`8764ced`（G28）。
范围：只读复核 + 本证据目录；未改产品/测试/容差/fixtures/回执，未碰 Backend 正在做的同族后继四条。

## 1. 结论（先给）

- **四条交付的核心数值声明均被证据与独立复跑支持，未发现伪造、放宽容差或验收口径削弱。**
- 独立复跑（CPU，默认特性 `cargo test --locked --lib <filter>`）全部通过：`constants` **7/0**、`state` **16/0**、`actuator` **4/0**、`query` **6/0**。
- 4 个模型源文件 sha256 与各 `authoring.json` 记录**全部一致**（4/4）；交付提交内 fixtures/证据/源码的 git blob 与回执一致（例外见 §5 的 5 个回执 blob 追溯差异）。
- 四条任务的 checks/files 已按规范细化（fmt/clippy/unit/unchanged/json/forbid_claim），`acceptance` 在细化与交付提交中**零改动**（0 行 diff），未放宽。
- `m08-p3` 的 3 条 `normify fix` 记录均为范围/模块登记与指纹刷新，check 种类前后一致；但回执 blob 与提交内 README 不一致（§5-②），建议刷新回执。
- 结论：**批次 1 前四条可接受，不阻塞后续**；差异为一般/建议级，不影响数值声明。

## 2. 四条声明逐项核对

| 任务 | 声明 | 证据位置 | 复核结果 |
| --- | --- | --- | --- |
| g27-p2 | 产品 vs MJWarp `set_const_fixed` 5 次逐位一致 | `authoring.json`：`runs=5`、`distinct_bit_patterns=1`、`max_abs_run_spread=0`；fixture `tolerance.upstream_abs=1e-6`；`src/physics/constants.rs` 测试按 `to_bits` 比较 | ✅ 证据自洽；复跑 `constants` 7/0 通过 |
| g27-p2 | vs 原生 f64 max_abs 2.98e-07 ≤ 1e-6 | `authoring.json.max_abs_vs_native=2.980232238769531e-07`；回执同值；测试读 fixture `native_abs` 断言 | ✅ 一致（值 ≤ 冻结 1e-6） |
| m08-p3 | 7/7 用例与上游逐位一致 | fixture `cases` 长度 7、`tolerance.abs=0`；测试逐 `to_bits`；回执 `product_vs_upstream=7/7` | ✅ 复跑 `state` 16/0 通过 |
| m08-p3 | 原生 `mj_resetData/Keyframe` max_abs 0 | `authoring.json.native_cross_check.max_abs_vs_native_mj_resetData=0.0`；fixture `native_cross_check` | ✅ 一致 |
| g05-p4 | 三组输出与上游逐位一致 | `src/physics/actuator/tests.rs:150-160` 对 `act_dot`、`actuator_force`、`qfrc_actuator` 三组 `assert_bits`；fixture `tolerance.upstream_abs=0` | ✅ 复跑 `actuator` 4/0 通过 |
| g05-p4 | 原生 f64 max_abs 9.54e-08 ≤ 1e-5 | `authoring.json.native_cross_check.max_abs_vs_native=9.536743172944284e-08`；fixture `native_abs=1e-5` | ✅ 一致 |
| g28-p7 | 9/9 geomid 一致 | fixture `nray=9`、`rays` 长度 9；测试断言 geomid 相等 + dist/normal 容差；回执 `geomid 9/9` | ✅ 复跑 `query` 6/0 通过 |
| g28-p7 | 原生 `mj_ray` max_abs 1.68e-07/1.83e-07 | `authoring.json.max_abs_dist=1.6763955057452762e-07`、`max_abs_normal=1.830425819249193e-07`、`geomid_mismatch=0` | ✅ 一致 |

补充：四条 README 均把未交付部分（其余 `set_const` 常量、完整 Data/历史、transmission/肌肉等、BVH/jac/contact_force）如实列为未验收边界，并明确「不宣称完整族或精度验收通过、NP04 未测」——与 verify 结果一致，无夸大。

## 3. 独立复跑（CPU，默认特性）

```text
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --lib constants   -> 7 passed; 0 failed; 298 filtered out
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --lib state       -> 16 passed; 0 failed; 289 filtered out
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --lib actuator    -> 4 passed; 0 failed; 301 filtered out
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --lib query       -> 6 passed; 0 failed; 299 filtered out
```

均为 host 侧切片（无 GPU 内核）；复跑不启动 Python、不重跑 authoring。

## 4. sha256 / fixtures / 回执抽验

- 模型源 sha256 与 `authoring.json` 比对：`set-const/model.xml`、`local-reset/model.xml`、`actuation/model.xml`、`ray-query/model.xml` **4/4 一致**。
- 输出 fixtures 存在且被测试读取：`g27-fixed.json`、`reset-cases.json`（7 用例）、`actuation-cases.json`（2 world/3 actuator）、`ray-cases.json`（9 射线）。
- 回执 `files[].blob` 与交付提交 `git ls-tree` 比对：**绝大多数一致**；差异仅在 4 个 `.normify/modules/**/core.md` 与 1 个 m08 README（§5）。`src/physics/mod.rs` 与当前 HEAD 不同但在交付提交时一致——由后续任务追加导出所致，非问题。

## 5. 差异清单（按严重级别）

**① 一般（证据追溯）：4 个 `.normify/modules/**/core.md` 回执 blob 与提交内版本不一致**
- g27/g05/g28 的 `physics/core.md`、m08 的 `io/core.md`：回执记录的 blob 等于 `done` 刷新指纹**前/中间**版本，提交内是刷新后的版本。
- 影响：仅回执追溯（模块索引是共享文件，每次任务 done 都会刷新）；不影响数值声明与证据内容。建议回执在 `done` 之后生成，或不在回执中登记模块索引文件。

**② 一般（证据追溯）：`m08-p3` 的 README 回执 blob 与提交内文件不一致**
- 回执记录 `7b6f4259…`，提交/HEAD 为 `f9ef2abd…`（`git log` 显示 README 仅在 `e63b296` 添加，之后未改）。
- 回执与 README 内容声明一致，但字节级无法确认「fix 期间证据未改」（fix addenda 声称「验收口径、证据与检查不变」）。建议 Backend 刷新该回执 blob。
- 未发现声明/数值被改动：README 的 7/7、max_abs 0 与 fixture/测试一致。

**③ 建议：authoring 未独立重生成**
- 5 次运行位型、原生 f64 max_abs 依赖 `authoring.json` 与冻结 fixture 自洽；重新生成需要 pinned venv + GPU，且属离线证据链，本轮按范围未重跑（不占用 GPU）。现有证据链完整、可复现脚本齐全（4 个 `author-*.py` 均 pin `71da24d…`）。

**④ 无阻断/严重问题**：未发现容差放宽、断言弱化、fixture 与证据不符、CPU 复跑失败。

## 6. checks/files 与验收口径核对

- 四条 `normify show` 均含：gate fmt、gate clippy、gate unit、unchanged（含前序任务 fixtures 保护）、json（回执 task_id）、forbid_claim（`完整验收,精度已验收`）。gate 运行记录（`.normify/runs/<task>/`）fmt/clippy/unit 均 `passed=true, exit=0`；g27 unit 记录 412 过 0 败，与回执一致。
- 细化提交（`2354645`/`1309506`/`f421045`/`2d8d35a`）与交付提交对 `.normify/changes/<task>.json` 的 `acceptance` 数组**零改动**（diff 0 行）；只新增 checks、收窄 files、补细化说明。
- `m08-p3` fix 记录（3 条 addenda）：①把 `src/io/state/tests.rs` 作为模块源码登记；②刷新模块指纹；③范围修正（改由 `src/io/state/` 目录项覆盖，模块登记还原；「验收口径、证据与检查不变」）。前/后 check 种类一致（gate,gate,gate,unchanged,json,forbid_claim），files 7→8。

## 7. 边界

- 只做独立复核与证据记录；未改产品/测试/容差/fixtures/回执；未 start/回退任何任务。
- 未重新生成 offline authoring 参考（需 pinned venv + GPU）；未复核 GPU 内核路径（本批四条均为 host 侧切片）。
- 未碰 Backend 正在做的 g27-p7 / m08-p7 / g05-p7 / g28-p8 范围。

## 8. 证据索引

- 本文件：逐条声明核对、复跑计数、sha256 抽验、差异清单。
- 被复核证据：`docs/development-evidence/backlog-batch1/<task-id>/{README.md,authoring.json}`、`docs/development-evidence/changes/<task-id>.json`、`.normify/runs/<task-id>/`。
- 复跑命令见 §3。

## 9. 回执口径实践（2026-10-23 增补，任务 `2026-10-23-receipt-practice-fixup`）

本节把 §5 发现的回执追溯差异转为长期口径，不重写历史回执。

**规则（自 2026-10-23 起执行）**：

1. **回执在 `normify done` 之后生成**：`done` 会刷新模块指纹与任务字段，回执若在 `done` 前生成，`files[].blob` 可能记录中间版本。以后先 gate/check/done，再写回执并提交。
2. **不登记共享模块索引文件**：`.normify/modules/**/core.md`、`.normify/modules/**/index.md` 是跨任务共享、每次 `done` 都会刷新的文件，不应作为单个交付回执的文件哈希登记；如确需登记，只登记 `done` 之后的稳定值。
3. **回执 blob 以提交内版本为权威**：发现回执 blob 与提交不一致时，用 `normify fix` 追加修正记录并写明权威值，不重写历史回执、不改证据内容。

**已知追溯差异登记（不改历史）**：

| 任务 | 文件 | 差异 |
| --- | --- | --- |
| g27-p2 / g05-p4 / g28-p7 | `.normify/modules/mjwarp/physics/core.md` | 回执 blob 为 `done` 刷新前/中间版本；模块索引随每次任务 `done` 刷新，历史回执不重写 |
| m08-p3 | `.normify/modules/mjwarp/io/core.md` | 同上 |
| m08-p3 | `docs/development-evidence/backlog-batch1/2026-10-11-roadmap-m08-p3/README.md` | 回执 blob 曾为交付前中间版本；已用 `normify fix`（2026-10-23）声明权威值以 HEAD 为准，并把回执登记值修正为提交内版本 |

上述 4 条模块索引差异为已知项，不影响数值声明与验收结论；后续回执按本节规则执行。
