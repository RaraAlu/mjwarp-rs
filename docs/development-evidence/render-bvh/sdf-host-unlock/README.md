# SDF host 解锁（U1b）

任务：`2026-10-31-render-sdf-host-unlock`（Arch；P8；离线，零 GPU、零窗口）
前置：U1 `2026-10-30-render-sdf-mesh-path`（verified）、U3 `2026-10-30-render-sdf-capture`（verified）
后续：U4a 模型 dump → U4 材料落卡 → U4 验证窗（`2026-10-31-render-sdf-validate`）

## 1. 做了什么

U1 已让设备侧 `gtype==8` 走底层 mesh 路径，但 host 侧 `megakernel.rs` 的 `reject_sdf_geometry`
仍把含 SDF geom 的模型判为 `Unsupported`，能力不可达。本片只删 host 拒绝，让 U4 窗口能加载 SDF 场景：

- 删除 `fn reject_sdf_geometry`（含 doc）与 `render()` 内调用点；注释改为「SDF 走底层 mesh，与 MESH 同路径；数值验证在 U4 窗口」；
- 删除 `#[cfg(test)] mod sdf_geometry_guard_tests`（2 条 host-only 单测，钉的是「不可达前提」，解锁后前提消失，**不造假断言**）；
- **不删**任何其它拒绝（`use_precomputed_rays` / `splat_count` / `light_mode` / `enabled_geom_ids` / `nsample`/`ray_count` / non-finite 等全部原样）。

## 2. 改动面

| 项 | 值 |
| --- | --- |
| 源码 diff | **只 `src/render/kernel/megakernel.rs`**：1 文件、**+1 / −35** |
| 行数 | **440 → 406**（LF，≤500） |
| 残留引用 | 工作树 `reject_sdf_geometry` / `S7b3 不含 SDF 求交` = **0**（`materials/removed-guard.txt` 留 HEAD 旧 3 处） |
| 零改动 | 全部 `.cu`、`cast.rs`、`bvh/**`、ABI/81 槽、判据/容差/参考、`tests/**`、`scripts/**`（`materials/removed-guard.txt` 的 4 条 unchanged 均为空） |
| 内核身份 | **不变**：source `778216b2…` / PTX `34f91147…` / 268,589 B（U1 收口值，见 `materials/kernel-identity.txt`） |

## 3. 门禁

| 门禁 | 结果 |
| --- | --- |
| fmt | 通过 |
| clippy | 通过 |
| clippy-cuda | 通过 |
| unit | **1106 通过 / 0 失败 / 392 ignored**（U1 为 1108；本片删 2 条 host-only 单测 ⇒ 1106，与预期一致） |
| `cargo test --locked --no-run` | **EXIT=0**，日志含 `Finished`（`logs/no-run.txt`） |
| `normify validate --fingerprints` | 错误 **0**；警告在 `done` 刷新 `mjwarp.render.kernel` 指纹后回到既存 5 条 |

## 4. 边界与回滚

- 本片**不实现 SDF 求交、不跑 GPU、不宣称能力、不发布**；解锁后的能力验证全部由 U4 窗口承担。
- **回滚声明**：U4 验证窗未过 → `git revert` 本片提交（U1 保留，SDF 能力回到不可达）。
- 解锁后 `megakernel.rs` 不再有「8 不可达」的 host 断言；U4 的 host 用例（SDF 用 mesh bounds、model dump 语义）+ GPU 参考比对承接这一前提。

## 5. 证据清单

| 文件 | 内容 |
| --- | --- |
| `README.md` | 本文 |
| `receipt.json` | 任务 id、diff/行数、unit 数、门禁、no-run、内核身份、回滚声明 |
| `manifest.txt` | 本目录除自身外全部文件 sha256/bytes |
| `logs/gate-*.txt` | 四门禁 `.normify/runs/<task>/<gate>.json` 转写快照（权威= `runs/`） |
| `logs/no-run.txt` | `cargo test --locked --no-run` 原样日志（含 `EXIT=0`、`Finished`） |
| `materials/diff-stat.txt` / `diff.patch` | 源码改动 |
| `materials/removed-guard.txt` | HEAD 旧 guard 3 处 + 工作树 0 命中 + 4 条 unchanged 空 |
| `materials/gates.txt` | 四门禁 exit/wall/tests/scope_hash |
| `materials/kernel-identity.txt` | 内核三值不变与 cite |
