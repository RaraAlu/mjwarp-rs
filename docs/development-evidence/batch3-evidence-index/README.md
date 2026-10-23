# 批次 3 独立证据目录建立与两条已关闭任务证据迁移

任务：`2026-10-23-batch3-evidence-index`。日期：2026-10-23。执行：Backend。
性质：证据组织治理（不改证据内容、结论、数字与产品/测试/容差）。

## 1. 问题与处置

批次 3 三条任务（`g08-p5`、`g18-p6`、`g24-p7`）的证据误放
`docs/development-evidence/backlog-batch2/` 下，而批次 1/2 各自有独立证据目录。
按 Chief 要求：

1. 新建 `docs/development-evidence/backlog-batch3/README.md` 批次 3 索引
   （G08/G18/G24 三条已交付摘要 + M10/A10 待追加说明 + 目录约定）。
2. 把 `g08-p5`、`g18-p6` 证据目录**原样**从 `backlog-batch2/` 迁到
   `backlog-batch3/`；`backlog-batch2/` 只保留批次 2 七条，并在其 README
   增补迁移说明。
3. 同步更新两条回执（`docs/development-evidence/changes/<task-id>.json`）中
   证据 README/authoring 的路径为 `backlog-batch3`。
4. 对两条已关闭任务执行 `normify fix` 追加迁移修正记录（含原因与范围），
   修正提交带 `Normify-Fixes: <task-id>`；未手改 `.normify/runs/` 历史文件。
5. `g24-p7` 证据在交付时已直接落 `backlog-batch3/`（未经历迁移）。

## 2. 迁移文件与内容不变证明

| 文件（迁移后路径） | 迁移前/后 blob（git hash-object） |
| --- | --- |
| `backlog-batch3/2026-10-11-roadmap-g08-p5/README.md` | `37386fd763c2f52bab426a7e6cb4b88b6df37a1c` |
| `backlog-batch3/2026-10-11-roadmap-g08-p5/authoring.json` | `8e4af0a67e65a5e1bfebb2b63dd868dc77815bb8` |
| `backlog-batch3/2026-10-11-roadmap-g18-p6/README.md` | `5d4f772dbde1c48b6309f1e1293ead9d6024cf12` |
| `backlog-batch3/2026-10-11-roadmap-g18-p6/authoring.json` | `910c52a4f586ce784da6205f83384f01324ce3fa` |

四份文件均 `git mv` 迁移，内容与 blob 不变；g18 两份与回执原登记 blob
（`5d4f772d…`/`910c52a4…`）一致。回执只改路径字符串，blob 登记值不变。

## 3. 变更文件

- 新增：`docs/development-evidence/backlog-batch3/README.md`、本目录 README、
  `docs/development-evidence/changes/2026-10-23-batch3-evidence-index.json`。
- 迁移：上述 4 个证据文件（git rename）。
- 修改：`docs/development-evidence/backlog-batch2/README.md`（迁移说明）、
  `docs/development-evidence/changes/2026-10-11-roadmap-g08-p5.json` 与
  `...-g18-p6.json`（证据路径字符串）。
- 任务记录：本任务 `.normify/changes/2026-10-23-batch3-evidence-index.json`；
  g08/g18 任务记录的 `addenda` 由 `normify fix` 生成。

## 4. 验证（收口时实测）

- `cargo fmt --check`：通过。
- `cargo test --locked`（gates.unit，全目标）：**470 过 0 败**。
- `normify check --staged`（本任务）：通过。
- 校验无残留旧路径：`docs/development-evidence/` 下除历史回执/历史目录外不再有
  指向 `backlog-batch2/2026-10-11-roadmap-g08-p5` 或 `...g18-p6` 的有效引用；
  `backlog-batch2/` 目录只剩 7 条批次 2 证据目录。
- 未改任何证据结论/数字、产品代码、测试、容差、fixtures 与 authoring 脚本。

## 5. 未做与边界

- 不改写历史提交中已存在的旧路径字符串（历史提交只读，回执以当前路径为准）。
- 不改 `.normify/runs/` 下历史门禁文件内容；g08/g18 的修正提交仅含
  `normify fix` 生成的 addenda 与重跑门禁结果。
- 本任务不改变 g08/g18/g24 的验收状态（保持 verified）。

## 6. 复现

```powershell
git log --follow -- docs/development-evidence/backlog-batch3/2026-10-11-roadmap-g08-p5/README.md
normify show 2026-10-11-roadmap-g08-p5
normify show 2026-10-11-roadmap-g18-p6
```
