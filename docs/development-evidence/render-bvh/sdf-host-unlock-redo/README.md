# 重做 U1b host 解锁（离线，零 GPU）

任务：`2026-10-31-render-sdf-host-unlock-redo`（Arch；P8）

## 1. 为什么重做
criterion v2 已由用户签发（B=128 ULP，2026-10-31T11:04:37+08:00），SDF 要发布；U1b 曾在 `07252cd2` 因 U4 attempt1/2 失败按红线回滚，attempt4 必须先让 host 重新放行 `gtype==8`。

## 2. 改动（与 af96c048 同形态）
- `src/render/kernel/megakernel.rs`：**440 → 405 行**，删除 `fn reject_sdf_geometry` 与文档注释、`render()` 调用点（含 1 行注释）、`#[cfg(test)] mod sdf_geometry_guard_tests`（2 条 host-only 单测）。
- 零 `.cu` 改动（`git diff --stat -- src/render/kernel/cuda` 为空）；零 ABI/81 槽/判据/参考/fixtures 改动。diff 见 `materials/src-diff.patch`。

## 3. 内核身份（不变）
render megakernel source `778216b2…` / PTX `34f91147…` / 268,589 B；aa_resolve `0bf8af5a…` / `b28e155c…` / 27,536 B（本片未动设备源，见 `materials/kernel-identity-unchanged.txt`）。

## 4. 门禁与构建
fmt/clippy/clippy-cuda 通过；unit **1111 过 / 0 败 / 397 ignored**（1113 − 2 条被删的 host-only 单测）；`cargo test --locked --no-run` **EXIT=0** 含 Finished；`validate --fingerprints` 错误 0。

## 5. 窗口准备 checklist（新增项）
**开窗前必须核对「host 解锁状态与 target 预期一致」**：attempt3 失败的根因之一是 U1b 仍处回滚态、`render_sdf_reference` 在发射前 panic，导致整轮无判据依据。

## 6. 边界
零 GPU、零窗口、零冻结；不改判据/参考/历史证据；`attempt4` 材料与窗口计划在下一片（第 4 片）。
