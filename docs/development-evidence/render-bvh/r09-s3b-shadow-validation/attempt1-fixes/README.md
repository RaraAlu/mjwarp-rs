# R09-S3b attempt1 红项修复：b 项（`render_e2e` 旧断言）

本目录收纳 R09-S3b 验证窗 attempt1 红项 **b** 的修复证据（红项定义见 `target/s3b-validation-window/attempt1/` 原始 facts/logs，未改动）。

## 背景

attempt1 中 `render_e2e` exit 101：`tests/render_e2e.rs` 的 `negative_controls_hold_with_product_poses` 仍断言「`use_shadows=true` 必须在写前被拒绝」，而 `2026-10-28-render-bvh-r09-s2-host-guard-unlock` 已解锁该路径（实际返回 `Ok(())`）。这是 S2b 同类旧断言遗漏的第三处（flags/rgb 已在 S2b 内同步）。

## 处置与留痕

- 修复：`use_shadows` 从拒绝面移出、改为「接受」断言；其余两类拒绝面保持全强度。
- 全仓清查：`use_shadows` 94 处命中，旧契约共 3 处（flags/rgb 已修 + e2e 本处），**无第四处**；产品源码无拒绝逻辑。证据见 `e2e-negative-control/grep-report.md`（6 466 B）与 `grep-report-raw.txt`。
- 验证：host `1 passed / 5 ignored`；GPU 回归 `render_e2e` **5/5**（含 negative control 的 `use_shadows` 接受断言真跑通过）。
- 判据/容差/参考/夹具零改动。

## 登记卡的去向（如实记录）

本项原本单独登记为 `2026-10-28-render-bvh-r09-s2b-e2e-negative-control`（in_progress）。提交时遇到结构性互锁：S3b validation 卡处于 in_progress 且其冻结 scope 为 `tests/`，与 e2e 卡的 `tests/render_e2e.rs` 前缀冲突，normify 以 `schedule/running-conflict` 拒绝任何一方提交；而运行中任务的声明不可改（`change/transition` 冻结）。Arch 裁决：**撤销 e2e 卡登记（abandon，原因写明），把 b 项代码改动（`tests/render_e2e.rs` 在 S3b scope `tests/` 内）与证据并入 S3b 提交流水**。
- 撤销卡文件保留为未跟踪历史记录（不删除内容），本 README 与提交信息共同构成留痕；
- attempt2 窗口仍落在 `2026-10-28-render-bvh-r09-s3b-shadow-validation` 卡下，任务 id 不变（Chief 签名锚点不变）。

## 目录内容

| 文件 | 说明 |
| --- | --- |
| `e2e-negative-control/README.md` | 修复说明与原卡面口径 |
| `e2e-negative-control/grep-report.md` + `grep-report-raw.txt` | 全仓旧断言清查清单与结论 |
| `e2e-negative-control/logs/host-test.log` | host 用例日志 |
| `e2e-negative-control/receipt.json` + `manifest.txt` | 原卡证据（task_id 指向已撤销的卡，作历史保留） |
