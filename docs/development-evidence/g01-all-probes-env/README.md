# G01 全探针环境接线（all-probes env wiring）

任务：`2026-10-22-g01-all-probes-env-wiring`。前置：`2026-10-22-t4-source-hash-repin-9`（`f1faa15`，源码身份收口）。
本轮结论：all-probes 从「25 项失败/6 个 target」收敛到 **仅 RNE 数值探针 1 项失败**；按约定口径**不宣称 G01 全绿**，表述为「除 RNE 数值探针缺口外全绿」。

## 本轮最终数字（debug 与 release 一致）

| 阶段 | debug | release |
| --- | --- | --- |
| fmt | 0 警告 | 0 警告 |
| native | 27 过 | 27 过 |
| host-msvc | 709 过 | 709 过 |
| g01 | 7 过 | 7 过 |
| native-g01 | 2 过 | 2 过 |
| native-only-g01 | 1 过 | 1 过 |
| resident | 102 过 | 102 过 |
| all-probes | **339 过 / 1 败 / 79 个 result 行** | **339 过 / 1 败 / 79 个 result 行** |
| host-gnu（debug 补跑） | 404 过 | — |
| clippy-gnu / clippy-gnu-cuda | 0 警告 | — |
| clippy-msvc / clippy-msvc --release | 0 警告 | 0 警告 |
| rustdoc（`-D warnings`） | 0 警告 | 0 警告 |
| diff（`git diff --check`） | 通过 | 通过 |

- debug 运行目录：`target/g01-acceptance/a947ee919daf48699d47016d684ebe70`（日志 `target/g01-debug-final4.log`，本机 `all-probes.log` 同目录）。
- release 运行目录：`target/g01-acceptance/3afadc22c5b74e9b8e6576aafdcad231`（日志 `target/g01-release-final2.log`）。
- all-probes 唯一失败：`tests/resident_rne_acceleration.rs::computes_both_flags_for_nonzero_accelerations_without_old_bias_replay`，报 `G02-RNE-ACC 521 native-tolerance blocks failed; complete coverage recorded`（models=5、gravities=3、states=8、worlds=[1,2,5,513]、flags=2、scalars=900288、约 13–19s）。
- 脚本在 all-probes 中止后不会执行后续阶段；上表后五行为本轮用与脚本相同的等价命令单独补跑（日志 `target/g01-stage-*.log`）。

## all-probes 环境接线（写在 `scripts/test-windows-g01.ps1`）

| env | 值/来源 |
| --- | --- |
| `MJWARP_T4_EVIDENCE_ROOT` | `target/t4-g03-retest`（含 debug-ptx/release-ptx 历史捕获） |
| `MJWARP_FROZEN_UPSTREAM_ROOT` | `target/mjwarp-parallel/upstream/mujoco_warp` |
| `MJWARP_FROZEN_WARP_ROOT` | `target/mjwarp-parallel/venv/Lib/site-packages/warp` |
| `MJWARP_T4_CURRENT_CAPTURE_ROOT` | 每轮新建的空目录（`<run>/t4-current-captures`） |
| `MJWARP_T4_AUDIT_REPORT` | `<run>/probe-reports/t4-audit.json` |
| `MJWARP_MACHINE_DIR` | `<run>/machine-code-captures`（路径保证**不存在**） |
| `MJWARP_GRAVITY_NONZERO_CHAIN_EVIDENCE_DIR` / `..._PRODUCT_CHAIN_...` | 每轮新建空目录 |
| `MJWARP_MIXED_FACTOR_REPORT` / `MJWARP_MIXED_SOLVE_REPORT` | `<run>/probe-reports/*.json` |
| `MJWARP_SPARSE_PREFLIGHT_EXE` / `MJWARP_NATIVE_SPARSE_OUTPUT` | 复用 `target/native-sparse-mass/attempt-*/sparse-preflight.exe`（既有构建产物，sha 随 receipt 记录），输出到每轮新建目录 |
| native 组 | 复用既有 `scripts/test-windows-native.ps1` 设置的 `MJWARP_*`（同进程生效） |
| collision 组 | 复用既有 `scripts/test-windows-collision-input.ps1`（离线 cl 编译 18 个读数替身；输出目录用相对 `target/g01-acceptance/<run>/collision-mocks`），env 同进程生效 |

接线状态落盘：`<run>/probe-reports/env-wiring.json`（t4_history/frozen_upstream/frozen_warp=t wired；sparse_preflight=reused；collision_mocks=wired）。collision 构建脚本要求输出目录不存在且位于 `target` 下，因此改为传相对路径。

## 三项「可接线项」结果

1. `native_sparse_mass`：复用既有 `sparse-preflight.exe`（未重复构建、未伪造），该 target 转绿。
2. `t4_machine_code`：`MJWARP_MACHINE_DIR` 改为「绝对路径且不存在、父目录在 target 下」，该 target 转绿。
3. `com_position_probe`：先前失败为 `os error 32`（杀软/索引器持有刚写入的 physics-precision-evidence 文件）瞬态锁；单跑复验通过；脚本对 all-probes 增加**仅在该瞬态错误**时用独立捕获目录重试一次的守卫（`env-wiring.json` 记录 `transient_file_lock_retry`；断言/计数不变）。

## 排除清单（单列、可审计；不并入 W-D5）

落盘：`<run>/probe-reports/exclusions.json`。

- **W-D5 原 3 项**（保持不变，用户 2026-10-22 裁决 A）：`recompile_rigid_variants_without_launching_physics`、`recaptures_only_missing_rotation_variants`、`archives_face_operator_repair_with_failed_numeric_gate`。
- **历史捕获身份债务 6 项**（新单列）：`audit_local_t4_precision_without_gpu_execution`、`rigid_internal::tests::audit_frozen_rigid_internal_and_output_sources`、`field_internal::flex_matrices::tests::audits_frozen_matrices_and_generated_face_artifacts`、`packing_routes::flex_internal::audits_frozen_flex_sources_and_retained_artifacts`、`packing_routes::routes::audit_remaining_packing_sources_and_captures`、`topology_frames::tests::audit_frozen_topology_and_frame_sources`。
  原因：G03 矩阵链对齐（`d5b02d5`）有意变更 f32 算序；这些 audit 以历史 BASE 捕获身份/历史模板对照当前源码，历史捕获先于该变更；重生成需被排除的重采集工具与冻结捕获流水线（跟踪任务 `2026-10-22-t4-historical-capture-rebaseline`，proposed）。
- **env/外部产物未接线 1 项**：`captures_loaded_face_machine_code_without_launching_physics`。原因：`target/t4-face-operators/product.ptx` 需由既有 face-operator 探针重新生成并重钉机读码期望，本轮不伪造、不硬跑。

## release 下 `gpu_shared_resident_ellipsoid_matches_independent_reference` 的原因判断

- 首轮 release 出现该 lib 失败，但报错是 `Backend(Compilation { stage: "physics-precision-evidence", detail: "attached_kinematics: ... (os error 32)" })`，即**文件锁瞬态**，不是 FMA/精度差异；debug 同名测试未失败。
- 加入瞬态锁重试守卫并复跑后，release all-probes 与 debug 完全一致（339/1/79，仅 RNE 失败），该测试不再失败。
- 结论：**环境瞬态（Windows 文件锁），不是 release 构建 FMA/精度差异**；不混入接线任务。若后续再次稳定复现，再按独立问题登记。

## 判读方法（供 QA 接手）

- 全脚本：`target/toolchains/powershell-7.6.7/bin/pwsh.exe -NoProfile -ExecutionPolicy Bypass -File scripts/test-windows-g01.ps1 -AllFeatures -FullRegression [-Release]`。
- 阶段结论看日志开头 `fmt/native/host-msvc/g01/native-g01/native-only-g01/resident passed=…`；all-probes 若失败会在 `all-probes.log` 里出现 `test result: FAILED`；本任务的「除 RNE 外全绿」判定 = all-probes 汇总 passed=339/failed=1，且失败集合恰为 `computes_both_flags_for_nonzero_accelerations_without_old_bias_replay`。
- 因为脚本在第一个失败 target 处抛错，all-probes 已加 `cargo --no-fail-fast`（放在 `--` 之前）以跑完全部 target 并留下完整 result 行。
- 迭代建议（按 Chief 口径）：日常只跑受影响 target（如 `cargo +1.99 test --locked --all-features --test <target> -- --ignored`），完整双模式只在收口/里程碑跑一次。

## 交接给 QA 的事项

1. 在 RNE 处置（用户裁决）前，G01 只能声明「除 RNE 数值探针缺口外全绿」，不得声明首次全绿。
2. 历史捕获身份债务 6 项与 env-unwired 1 项已在脚本单列；QA 复跑时按 `exclusions.json` 判读，不要把它们当静默跳过。
3. 建议在 `docs/three-layer-acceptance.md` 增「历史捕获身份债务」一节的现成文本见本文件「排除清单」段（QA/文档任务落盘）。
4. RNE 证据补齐（521 块清单、失败分布、基线归属/矩阵对齐前是否已存在、与冻结上游逐阶段定位）由 `2026-10-22-g02-rne-acc-native-tolerance-debt`（proposed）承接；本轮只记录汇总与日志路径：`target/g01-acceptance/a947ee919daf48699d47016d684ebe70/all-probes.log`。
5. `target/run-g01-both.ps1` 是本地一次性驱动（gitignored，不入库），做法是：同一 pwsh 进程内先跑 native/collision 脚本（env 同进程保留），设好上表 env 后以 `--no-fail-fast` 跑 ignored 全 target；复现请按本文命令与 env 表，不依赖该本地脚本。

## 未验收边界

- RNE 数值缺口未解决（不宣称 G01 全绿）。
- Linux、性能、完整上游等价未验收；历史捕获重基线未做。
- 未改断言/容差/W-D5 排除清单；未改产品源码。
