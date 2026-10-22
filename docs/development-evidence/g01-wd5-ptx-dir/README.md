# G01 捕获目录双语义修复（W-D5 语义冲突部分）

任务：`2026-10-22-g01-wd5-ptx-dir-semantics-2`（原任务 `-semantics` 因范围遗漏重登记）
基线：`1dceff6`；修复提交 `3a44e10`
范围：测试侧目录分配与文档；不改内核、PTX 生成策略、容差、fixtures 与产品逻辑。

## 根因

`MJWARP_PHYSICS_PTX_DIR` 在产品捕获层只有一个语义：捕获根目录（`src/runtime/transfer/kernel/ptx.rs` 按 `<entry>-<hash>` 写入并 `create_dir_all`）。G01 all-probes 用一个已存在的 `physics-ptx` 目录作为根，测试侧却存在两种冲突假设：

1. 包装测试（mass_matrix、mass_solve、resident、rne、rne_bias）把变量当根，写固定子目录名（`foreign-session`、`zero-dof`、`root-only` 等）。同一进程内固定子目录名被不同测试复用，第二次 `assert!(!child.exists())` 必失败。
2. 重采集/归档测试把变量当“必须不存在的输出目录”，并断言其绝对路径、位于 `target` 下；根目录已存在时 `assert!(!out.exists())` 必失败。

## 修复

新增测试侧辅助 `src/runtime/transfer/kernel/capture_dir.rs`（仅 `cfg(test)`，经 `src/runtime/mod.rs` 测试重导出）：

- `capture_child(scope, label)`：根目录存在与否都返回 `<root>/<scope>-<label>`，调用方断言不存在；产品捕获按需创建父目录。
- `capture_output(scope, label)`：根目录存在时返回唯一子目录，根目录不存在时沿用旧口径（变量本身即唯一目录），重采集/归档由此兼容单测命令。

十个测试文件改用统一分配，标签按 `module_path!()` 加作用域，消除跨测试复用。

## 验证

命令（G01 all-probes 等价环境；`+1.99.0-x86_64-pc-windows-msvc`、`--locked --all-features`、`-- --ignored --test-threads=1`，两次独立空捕获根）：

| 轮次 | 结果 | 说明 |
| --- | --- | --- |
| 1 | 122 过 5 败 | 6 项原失败转绿；另发现 1 次捕获文件锁（os error 32），独立复跑通过 |
| 2 | 123 过 4 败 | 6 项稳定转绿；无文件锁复现 |

原 9 项失败处置：

| 项 | 结果 |
| --- | --- |
| `physics::mass_solve::tests_device_wrapper::runs_rigid_mass_solve_wrapper_paths_and_recovers` | 转绿 |
| `physics::rne::tests_{backward,cacc,cfrc,qfrc}_run` 4 项 | 转绿 |
| `physics::rne_bias::tests_gpu::runs_rne_bias_wrapper_and_recovers_after_real_metadata_write` | 转绿 |
| 重采集 2 项 + 归档 1 项 | **未转绿，另案**（见下） |
| W-D4 重力位证书 1 项 | 已知债务，未动 |

## 未解决：3 项需历史基线（另案跟踪）

三项在独立运行（绝对/相对、存在/不存在根目录、单测与应用两种环境）下均以相同方式失败：

- `recompile_rigid_variants_without_launching_physics`：`source drift: src/physics/attached.rs`。`fixtures/precision/t4-kernels.json` 有 7 个产品源身份与当前源码不同（attached、com_velocity、mass_matrix、mass_solve、resident、rne_bias、transfer），而 `2026-10-19-t4-input-precision-source-drift` 回执明确 t4-kernels 身份保持历史。
- `recaptures_only_missing_rotation_variants`：`capture coverage mismatch`。
- `archives_face_operator_repair_with_failed_numeric_gate`：归档计数 `left 0, right 2078`。

结论：它们是需要历史捕获产物（`target/t4-g03-retest` 等 10/17 冻结产物）与冻结源码基线的人工重采集/归档工具，不是 all-probes 验收探针。按约定未改其断言、未改脚本、未伪造基线；由 proposed 任务 `2026-10-22-g01-wd5-recapture-baseline` 跟踪，等用户对 A（脚本排除）/B（已知债务）/C（另立重生成任务）裁决。

## 边界

- 不改内核字符串/PTX 生成策略、数值容差、fixtures 与产品物理逻辑。
- 不宣称 all-probes 全绿；W-D4 与上述 3 项仍开。
- 未跑 release、Linux 与整脚本两遍；os error 32 判定为环境瞬态，记录不追。
