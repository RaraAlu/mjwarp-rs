# 批次 2 回执追溯与门禁命令名修正

日期：2026-10-23。执行：QA。任务：`2026-10-23-batch2-doc-receipt-fixup`。
来源：`2026-10-23-batch2-independent-check` §5 差异清单（均为不阻断项）。

## 1. 修正项 1：`g20-p7` 回执 README 权威 blob

- 现象：交付回执 `docs/development-evidence/changes/2026-10-11-roadmap-g20-p7.json`
  把 evidence README 的 blob 登记为 `ecd16262f509072c2aa7fe4519be11ca58aa0ef0`。
- 事实：该 README 仅在 `a10aea5`（G20@P7 交付提交）被添加，其后未再修改
  （`git log -- <路径>` 仅 1 条）；提交内实际 blob = `2638ad1c03c4a8f9b4d3400843df9fb0519958d8`
  （`git hash-object` 实测）。原登记值为交付前中间版本。
- 处置：按 §9 口径实践规则 3，用 `normify fix 2026-10-11-roadmap-g20-p7 --reason …`
  追加修正记录（`addenda` 首条，`at=2026-10-23T06:47:59.710Z`），写明权威值以提交内版本为准；
  回执登记值同步改为 `2638ad1c…`。
- 边界：只改回执登记与口径，**未改**证据内容、测试与验收结论；历史回执不重写。

## 2. 修正项 2：unit 门禁命令名标注

`.normify/gates.toml` 的 unit 门禁实际为 `cargo test --locked`（全目标）。原标注写成
`cargo test --locked --lib`（461），把定向过滤当成了门禁命令。数字本身无误（全目标 461 过 0 败；
`--lib` 单独 341），仅命令名标注错误。

| 位置 | 原标注 | 修正后 |
| --- | --- | --- |
| `docs/development-evidence/backlog-batch2/README.md` §3 | `cargo test --locked --lib`：461 过 0 败 | `cargo test --locked`（gates.unit，全目标）：461 过 0 败；`--lib` 单独 341，仅定向复跑过滤 |
| `docs/development-evidence/changes/2026-10-11-roadmap-a10-p10.json` `results.gates.unit` | 「tooling 5 过；全量 lib 461 过 0 败」 | 「gates.unit = cargo test --locked 全目标 461 过 0 败；--lib 单独 341；含 tooling 5 过」 |
| 同上 `results.verification.tests` | `cargo test --locked --lib tooling`：5 过 0 败 | 「定向复跑 cargo test --locked --lib tooling：5 过 0 败（门禁命令见 gates.unit）」 |

计数（461/0、341、tooling 5/0）一律不动。

## 3. 修正项 3：§9 口径实践增补

`docs/development-evidence/batch1-independent-check/README.md` §9：

- 新增规则 4「门禁命令名以 `.normify/gates.toml` 为准」：unit 门禁是 `cargo test --locked`
  （全目标 461），`--lib`（341）或其它过滤只是定向复跑，不能写成门禁命令。
- 已知追溯差异表新增 `g20-p7` 行：`ecd16262…` 为交付前中间版本，已用 `normify fix` 声明
  权威值 `2638ad1c…` 并修正回执登记值（本任务）。
- 表下说明改为「上述 4 条模块索引差异（g27-p2/g05-p4/g28-p7 的 `physics/core.md`、
  m08-p3 的 `io/core.md`）为已知项」，避免与新增的已修正行混淆。

## 4. 验证

- `normify gate 2026-10-11-roadmap-g20-p7`：fmt/clippy/unit 重跑通过（unit 464 过 0 败，
  含当时工作区在途改动；本次修正不改产品代码）。
- 本任务 gate fmt/unit 通过；`normify check --staged` 通过；`normify validate` 错误 0。
- 修正后回执登记值与提交内 blob 一致（`git hash-object` 复算）。

## 5. 边界

- 只做回执登记/命令名标注/口径增补；未改产品、测试、容差、fixtures、脚本与证据结论。
- 历史回执不重写；批次 1/2 的数值与验收结论不受影响。
- 未触碰 Backend 批次 3 WIP（`g08-p5` 等），提交一律 path-limited。
