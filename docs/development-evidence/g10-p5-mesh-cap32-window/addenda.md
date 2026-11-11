# A2-3 窗口收口补充（append-only；历史字节不改）

- 任务：`2026-11-12-w2-a2-3-m2-m3-s4-replay-window`（窗后 `normify fix` 记录）。
- 本文件针对 QA/Chief 已见的 3 个观察与 O3 处置，只追加说明，不改窗口证据历史字节。

## O3（fixture 注释 13→15）移出本卡

- `tests/support/branch_closure_case.rs:130` 注释「S3 签名集合里的 13 个 fixture」与数组 15 不符（A2-2 后未同步）。
- 尝试在本卡 `normify fix` 中直接修：commit-msg hook 拒绝——`tests/support/` 不在本卡登记写入范围（本卡 files 仅
  `docs/development-evidence/g10-p5-mesh-cap32-identity/` 与 `...-window/`），且修正提交需追加 addenda/证据文件。
- 处置：**本卡不加 tests 写入范围、不改冻结字段**；O3 注释修正改由独立 0 窗片
  `2026-11-12-w2-a2-3-o3-fixture-comment-fix` 落地（tests-only；重跑四门禁）。本窗口证据不受影响（窗口已在冻结 exe 上完成）。

## 窗口证据状态

- 窗口证据已入库（提交 `bd231ace`），manifest 20/20 自核；本 addenda 追加后 manifest 21 条。
- 本补充不改变窗口结论与限制（U7/U8 可观测子集结论、pass_rule 全条件未覆盖、不宣称全条件通过）。
