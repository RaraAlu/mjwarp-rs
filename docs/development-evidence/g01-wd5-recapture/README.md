# G01 历史捕获/归档工具显式排除（用户裁决 A）

任务：`2026-10-22-g01-wd5-recapture-baseline`
基线：`5ac1a00`
裁决来源：用户 2026-10-22 裁决 A（由 Chief 转达）：显式排除并写清债务边界，不做静默跳过。

## 排除对象

| 测试 | 文件（一行未改） |
| --- | --- |
| `runtime::transfer::kernel::recapture::tests_recapture::recompile_rigid_variants_without_launching_physics` | `tests/support/t4_rigid_capture/tests_recapture.rs` |
| `runtime::transfer::kernel::recapture::tests_recapture::recaptures_only_missing_rotation_variants` | `tests/support/t4_rigid_capture/tests_recapture.rs` |
| `runtime::transfer::kernel::recapture::tests_archive::archives_face_operator_repair_with_failed_numeric_gate` | `tests/support/t4_rigid_capture/tests_archive.rs` |

依据（W-D5 调查结论）：三项需要历史捕获产物（`target/t4-g03-retest` 等 10/17 冻结产物）与冻结源码基线（`fixtures/precision/t4-kernels.json` 有 7 个产品源身份保持历史），不是验收探针；在普通验收工作区会分别报 `source drift: src/physics/attached.rs`、`capture coverage mismatch`、归档计数 `left 0 right 2078`。详见 `docs/development-evidence/g01-wd5-ptx-dir/README.md`。

## 改动

1. `scripts/test-windows-g01.ps1` all-probes 阶段：排除清单集中在 `$script:excludedAllProbes` 一处，附注释（历史捕获/归档工具、需 10/17 冻结产物与冻结源码基线、不是验收探针、用户 2026-10-22 裁决 A），通过 libtest `--skip` 显式排除；不是静默跳过，日志仍记录全部命令与计数。
2. 三个测试文件一行未改（任务用 `unchanged` 检查机器核对两个文件）。
3. `docs/three-layer-acceptance.md`：债务表登记「历史基线缺失、all-probes 显式排除、需冻结产物才能复采」并引用用户裁决；§5.1 给出有冻结产物时的手动运行命令；§4.1 与 §7 更新最终状态。
4. W-D6 计数同步：all-probes `-Minimum` 由 169 改为实际通过数 123（3 项排除后仅 W-D4 仍红；W-D4 转绿后应核对 124）。

## 验证

| 命令 | 结果 |
| --- | --- |
| `cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features -- --ignored --test-threads=1 --skip <3 项>`（脚本脚本同口径，独立空捕获根） | 123 过 / 1 败；唯一失败为 W-D4 `gpu_matches_independent_multi_body_reduction_certificate` |
| `pwsh -File scripts/test-windows-g01.ps1 -AllFeatures -FullRegression` | all-probes 阶段日志 123 过 / 1 败（仅 W-D4），排除生效；脚本按设计因 W-D4 已知债务在该阶段停下 |
| 门禁 fmt/clippy/clippy-cuda/unit | 见 `normify gate` 记录 |

## 手动运行（有冻结产物时）

见 `docs/three-layer-acceptance.md` §5.1：`MJWARP_PHYSICS_PTX_DIR` 必须为绝对路径、位于 `target` 下且不存在的新目录，然后按测试全名执行 `cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features cuda-probe --lib <测试全名> -- --ignored --test-threads=1 --nocapture`。

## 边界

- W-D4 未碰（证书口径，等用户裁决）；不伪造/重写历史基线；不改内核、容差、fixtures、测试断言。
- 3 项作为历史债务保留在 `2026-10-22-g01-wd5-recapture-baseline`（本任务）与 W-D5 债务表；复采基线更新需另立证据任务。
- 未跑 release 与 Linux。
