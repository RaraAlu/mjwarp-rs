# R09-S2b-e2e negative control 修复（use_shadows 解锁同步 + 全仓旧断言清查）

任务：`2026-10-28-render-bvh-r09-s2b-e2e-negative-control`（owner Backend）
判据树 HEAD（开写时）：`cbee1127`
背景：S2b 解锁 `use_shadows=true` 后，S3b 验证窗 attempt1 的 `render_e2e` negative control 仍断言「写前拒绝」（窗口内实测 `tests/render_e2e.rs:475` panic：`use_shadows 必须在写前拒绝（实际 Ok(())）`）。本卡把该旧断言同步为「接受」，并按 Chief 要求全仓清查是否有第四处。

## 1. 改动

| 文件 | 改前 | 改后 | 内容 |
| --- | --- | --- | --- |
| `tests/render_e2e.rs` | 499 | **498** | negative control 的 `use_shadows=true` 从 cases 循环（断言 `Unsupported`）移出 → 独立「接受 → `Ok(())`」断言（与 flags/rgb 用例同口径）；其余 2 类（`use_precomputed_rays=false`、`render_skybox`）保持断言 `Unsupported` 全强度；`rejected` 计数 3 → 2；用例注释与 `println!` 更新 |

未改判据/容差/参考/夹具；未删除或降级任何拒绝面用例；产品源码未改。

## 2. 全仓旧断言清查（Chief 要求，非「只改一行」）

- 明细见 `grep-report.md` / `grep-report-raw.txt`（`src/` + `tests/` + `scripts/` 的 `.rs`/`.cu`/`.py` 全量扫描）
- **旧契约「`use_shadows=true` 必须写前拒绝」全仓共 3 处**：
  1. `tests/render_kernel_flags.rs`（S2b 已改为接受）✓
  2. `tests/render_kernel_rgb.rs`（S2b 已改为接受）✓
  3. `tests/render_e2e.rs:445`（**本卡修复**）
- **修复后复查：无第四处**。`use_shadows` 其余 94 处命中均为「接受断言 / `ResolvedContext` 接线 / 参数解析 / 参考读取 / 诊断打印 / authoring 脚本参数」，不是拒绝契约
- 产品源码（`src/`）无 `use_shadows=true` 拒绝逻辑（S2b 已移除 `megakernel.rs` 前置拒绝，L97 仅剩解锁说明注释）
- 拒绝面关键词（`Unsupported`/「写前拒绝」/「不含阴影」/`rejected` 且含 shadow/阴影）命中全部为**注释或 println 说明**，无第三类断言

## 3. 验证（本阶段编译 + host；GPU 用例在 attempt2 窗口内跑）

- `cargo test --locked --features cuda-probe --test render_e2e` → **1 passed / 0 failed / 5 ignored**（`logs/host-test.log`；host 用例 `fixtures_cover_frozen_scene_and_closure_evidence_paths` 通过）
- `cargo clippy --all-targets -- -D warnings` → 通过
- 四门禁：`normify gate 2026-10-28-render-bvh-r09-s2b-e2e-negative-control`（绑最终树；结果见 `.normify/runs/`）

## 4. 限制与口径

- 本卡**不跑 GPU 用例**：negative control 的「`use_shadows=true` → 接受 → `Ok(())`」断言将在 attempt2 验证窗内由 `render_e2e` 实体真实执行
- 不写任何窗口结论、覆盖声明、精度结论或等价结论；判据/容差/参考/夹具零改动
