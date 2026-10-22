# 三层迁移验收记录（Windows V1）

日期：2026-10-22。
范围：2026-10-18 三层设计的 Windows 集成验收。
基线提交：`ae89f43`（I1 数据交互层接入）。
Linux V2：需要 Linux 机器，未启动，保持未验收。

本记录只验收迁移范围。
它不转授完整产品验收。
它不宣称 NP01–NP04 精度达标。
Windows 结果不替代 Linux。

第 3、4 节保留 `ae89f43` 时点的历史快照，不改写。
2026-10-22 债务修复后的 G01 现状追加在第 4.1 节与第 5 节。

证据目录：[docs/evidence/three-layer/windows](evidence/three-layer/windows/summary.json)。

## 1. 环境

| 项 | 值 |
| --- | --- |
| 平台 | Windows x86_64 MSVC |
| rustc | 1.99.0 (b940084d7 2026-09-28) |
| cargo | 1.99.0 (5f94df478 2026-08-27) |
| GPU | NVIDIA GeForce RTX 4070 Ti SUPER |
| 驱动 | 596.36 |
| 设备显存 | 16376 MiB（本轮未采样峰值显存） |
| CUDA | target/toolchains/cuda-12.8.1 |
| NVRTC | CUDA 12.8（cudarc 0.18.2 动态加载） |
| MuJoCo | target/toolchains/mujoco-3.12.0/package |
| PowerShell | target/toolchains/powershell-7.6.7 |

原生 DLL 与 fixtures 哈希由验收脚本核对。
脚本显式不调用 Python。
脚本显式不启动模型编译器。

## 2. 三层迁移范围

- M1/M2：统一发射器、产物缓存与上下文模块缓存。
- M3：安全内核包装迁移（四项清单归零）；常驻/角速度/肌腱等调用点改走类型化包装。
- M4：L2 冻结产物接入统一发射器。
- M5：异步完成凭证与访问依赖账本。
- M6：捕获隔离、数值槽重绑与待回收列表。
- A1/A2：设备数据归属与渲染布局归属。
- I1：数据交互层安全面（不发射内核）。

## 3. 三层定向回归（debug 与 release，历史快照）

命令：`cargo test --locked --features cuda-probe …`。
`--ignored` 用例单独列出。
全部 0 失败。

| 用例 | debug | release |
| --- | --- | --- |
| layer_render_layout | 6 过 | 6 过 |
| layer_data（宿主/设备） | 10/1 过 | 10/1 过 |
| layer_io（宿主/设备） | 4/3 过 | 4/3 过 |
| launcher_frozen | 4 过 | 4 过 |
| launcher_async（宿主/设备） | 11/1 过 | 11/1 过 |
| launcher_graph（宿主/设备） | 10/1 过 | 10/1 过 |
| launcher_abi | 19 过 | 19 过 |
| launcher_sync | 5 过 | 5 过 |
| launcher_cache（设备） | 4 过 | 4 过 |
| t4 gravity upstream kinematics | 1 过 | 1 过 |
| t4 gravity upstream com | 1 过 | 1 过 |
| t4 gravity upstream abi | 3 过 | 3 过 |
| t4 gravity product chain | 1 过 | 1 过 |

迁移触达的 G01 完整链另见下节。
逐用例日志与耗时见证据目录。
上表未在 2026-10-22 修复后重跑；本轮新增代码只涉及 W-D7/W-D8 修复与证据重钉，不改变三层定向用例语义。

## 4. 全量 G01 回归（历史快照，`ae89f43`）

命令：`scripts/test-windows-g01.ps1 -AllFeatures -FullRegression`（debug 与 release 各一次）。
两次均在 `host-msvc` 阶段中止，退出码 1。

| 运行 | 退出 | 耗时 | 中止点 |
| --- | --- | --- | --- |
| debug | 1 | 102744 ms | host-msvc |
| release | 1 | 65368 ms | host-msvc |

中止原因：既有宿主用例
`runtime::transfer::kernel::recapture::tests_host::preserves_current_sources_and_all_thirty_variants`
失败，报 `unexpected lineage change`（W-D2）。

脚本中止后按阶段手工补跑：

| 阶段 | 结果 |
| --- | --- |
| fmt | 通过（gate） |
| native 探针 | 通过（脚本在 host-msvc 前已完成） |
| g01 | 7 过 0 败（2084 状态 / 7704548 标量） |
| native_g01 | 1 过 1 败（W-D1） |
| resident 脚本 | 在 wake-guards 中止（W-D1） |
| host-gnu | 273 过 0 败（默认特性） |
| rustdoc -D warnings | 3 处既有 broken intra-doc links（W-D3） |
| git diff --check | 通过 |
| all-probes 全探针矩阵 | **未执行**（脚本提前中止，本轮未单独启动） |

结论（历史快照）：三层定向回归全绿；G01 全量**不通过**，失败项已登记为债务，不宣称全量通过。

## 4.1 2026-10-22 债务修复后 G01 现状

修复提交后的最新 full-debug 运行（`scripts/test-windows-g01.ps1 -AllFeatures -FullRegression`，工作区干净；本机日志 `target/g01-final.log`；W-D5/W-D3/重钉后最终复跑目录 `target/g01-acceptance/b0dce6add88645cfa7e674f7a93a569e`）：

| 阶段 | 结果 |
| --- | --- |
| fmt | 通过 |
| native | 27 过 |
| host-msvc | **709 过**（repin-5 重钉 U038 后的 resident 源码身份后恢复） |
| g01 | 7 过 |
| native-g01 | **2 过** |
| native-only-g01 | 1 过 |
| resident | **102 过**（不再中止） |
| rustdoc -D warnings | 通过（0 警告，W-D3 已修复） |
| all-probes | **123 过 1 败**（461 filtered，3 项历史捕获/归档工具按用户 2026-10-22 裁决 A 显式排除生效；唯一失败为 W-D4，见 §5.1） |

all-probes 的剩余失败明细：

- W-D4 ×1：`physics::gravity_compensation::tests::gpu_matches_independent_multi_body_reduction_certificate`（2 bit / 4 ulp，证书口径，基线同样失败；用户尚未裁决，未动容差与断言）。
- 历史基线缺失 ×3（已显式排除，不算验收探针）：`runtime::transfer::kernel::recapture` 的 2 项重采集与 1 项归档。经独立复现确认不是 `MJWARP_PHYSICS_PTX_DIR` 语义问题，而是需要历史捕获产物（`target/t4-g03-retest` 等 10/17 冻结产物）与冻结源码基线（`fixtures/precision/t4-kernels.json` 有 7 个产品源身份漂移，历史回执明确保持历史身份）。用户 2026-10-22 裁决 A：从 all-probes 显式排除并写清债务边界，工具保留，手动运行命令见 §5.1。

W-D5 语义冲突修复：测试侧把 `MJWARP_PHYSICS_PTX_DIR` 统一按捕获根解释，根存在时按 `module_path!()` 与标签分配唯一子目录；mass_solve wrapper 1、rne 4、rne_bias 1 共 6 项稳定转绿（G01 all-probes 等价命令两轮独立空捕获根：122 过 5 败、123 过 4 败）。证据 `docs/development-evidence/g01-wd5-ptx-dir/README.md`。

与历史快照对比：W-D1、W-D2、W-D7 造成的阻塞已消除；resident 阶段从 wake-guards 中止变为 102 项全绿；W-D3 rustdoc 0 警告；all-probes 从“未执行”推进到 123 过 1 败（仅 W-D4）。
本轮未重跑 G01 release；release 现状仍以历史快照为准，不宣称通过。

结论（当前）：三层迁移范围的 debug 回归无新增 M3 回归；G01 全量仍**不被宣称通过**，唯一剩余失败项为 W-D4（证书口径，等用户裁决）。

## 5. 债务清单

| 编号 | 级别 | 状态 | 内容与证据 |
| --- | --- | --- | --- |
| W-D1 | P0 | **已修复** | M3 唤醒安全包装零树/零批次元数据回归，由 `1551c59` 引入。修复提交 `eef9321`（登记）+ `71638e2`（修复），任务 `2026-10-22-layers-m3-wake-native-chain-fix` 已验证。`native_g01 --include-ignored` 2 过；wake-guards 1 过；resident 脚本不再在 wake-guards 中止。 |
| W-D2 | P1 | **已修复** | T4 pinned lineage 漂移，阻塞 host-msvc。重钉提交 `28bd15f`（登记）+ `4ffda93`；W-D7 修复改变 resident 源码后由 `2026-10-22-t4-source-hash-repin-3` 再次重钉：`c5c47e0`（登记）+ `eac3df6`。host-msvc 697 过，lineage 不再中止。 |
| W-D3 | P2 | **已修复** | rustdoc `-D warnings` 三处既有 broken intra-doc links（kinetic_energy.rs、device_energy/plan.rs、mass_solve/sparse/resident/solve.rs）为中文量纲记号被当作 intra-doc link。任务 `2026-10-22-g01-wd3-rustdoc-links` 用代码文本包裹修复，未放宽门禁、未新增豁免；`cargo +1.99.0-x86_64-pc-windows-msvc doc --locked --all-features --no-deps`（`RUSTDOCFLAGS=-D warnings`）0 警告通过。 |
| W-D4 | 已知债务 | 未修复（用户未裁决） | G01 重力位证书 2 bit 差异：f32 输入舍入导致 4 ulp，证书用有理精确值；基线同样失败，属证书口径问题。定性与证据见 `2026-10-22-g01-numerics-audit` 回执。**容差未修改；不宣称通过；用户尚未裁决口径。** |
| W-D5 | 已知债务 | **部分修复 + 3 项显式排除** | `MJWARP_PHYSICS_PTX_DIR` 测试侧目录分配语义冲突：已统一为“捕获根 + 唯一子目录”，6/9 转绿（mass_solve wrapper 1、rne 4、rne_bias 1），修复提交 `3a44e10`（任务 `2026-10-22-g01-wd5-ptx-dir-semantics-2`），证据 `docs/development-evidence/g01-wd5-ptx-dir/README.md`。剩余 3 项重采集/归档为历史捕获与冻结源码基线缺失，**用户 2026-10-22 裁决 A**：all-probes 显式排除并写清边界（任务 `2026-10-22-g01-wd5-recapture-baseline`），工具保留、手动运行命令见 §5.1；未改 3 个测试、未伪造基线。 |
| W-D6 | 既有债务 | **已收口** | G01 脚本 resident/all-probes 期望计数与拆分后布局同步：`2026-10-22-g01-gate-script-sync` 已同步 resident 102；all-probes `-Minimum` 由 169 改为实际通过数 123（3 项历史工具显式排除后，仅 W-D4 仍红），由 `2026-10-22-g01-wd5-recapture-baseline` 脚本改动同步。 |
| W-D7 | P0 | **已修复** | M3 柔体边安全包装空 ABI 回归（`068fd14` 引入）：`FlexEdgesStaticDims` 误拒单世界体/零 nnz 合法输入。修复提交 `341a52c`（登记）+ `c98e246`（修复），任务 `2026-10-22-layers-m3-flex-edges-abi-fix` 已验证；`flex_edge_free_body...` 恢复、resident 脚本 102 项全绿。 |
| W-D8 | P1 | **已修复** | M3 稀疏求解包装空计划 ABI 回归（`38a4960` 引入）：`SparseSolveStaticDims` 误拒 n=0/nc=0 合法空计划。修复提交 `1e3dd2f`（登记）+ `24d7937`（修复），任务 `2026-10-22-layers-m3-sparse-solve-empty-abi-fix` 已验证；`tests/resident_sparse_solve -- --ignored` 2 过 0 败。 |

W-D1、W-D7、W-D8 是迁移自身回归，已在独立任务中修复，未被当作历史债务豁免。
W-D4 仍开放且用户未裁决；W-D5 语义冲突已部分修复（6/9），剩余 3 项历史工具已按用户裁决 A 显式排除；W-D3 已修复；W-D6 已收口（resident 102 与 all-probes `-Minimum 123` 对齐）。
不得以本记录豁免任何原容差或门禁。

### 5.1 历史捕获/归档工具手动运行（all-probes 显式排除项）

用户 2026-10-22 裁决 A：以下 3 项是历史捕获/归档工具，需要 10/17 冻结产物与冻结源码基线，不是验收探针，因此从 `scripts/test-windows-g01.ps1` 的 all-probes 阶段显式排除；测试文件一行未改，工具保留可手动运行：

```powershell
$env:CUDA_PATH = '<repo>\target\toolchains\cuda-12.8.1'
$env:PATH = "$env:CUDA_PATH\bin;$env:PATH"
# 输出目录必须是绝对路径、位于 target 下且不存在（历史产物不覆盖）。
$env:MJWARP_PHYSICS_PTX_DIR = '<repo>\target\<全新目录>'

cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features cuda-probe --lib `
  runtime::transfer::kernel::recapture::tests_recapture::recompile_rigid_variants_without_launching_physics `
  -- --ignored --test-threads=1 --nocapture
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features cuda-probe --lib `
  runtime::transfer::kernel::recapture::tests_recapture::recaptures_only_missing_rotation_variants `
  -- --ignored --test-threads=1 --nocapture
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features cuda-probe --lib `
  runtime::transfer::kernel::recapture::tests_archive::archives_face_operator_repair_with_failed_numeric_gate `
  -- --ignored --test-threads=1 --nocapture
```

前提与边界：需要 `target/t4-g03-retest` 等冻结捕获产物与匹配的冻结源码基线；当前源码身份已前进时这些工具会报 source drift/capture coverage/归档计数不符，属预期历史债务，不修改断言、不伪造基线。复采成功后的基线更新另立证据任务。

## 6. NP01–NP04 缺口

- NP01/NP02 仍为未验收；重复修复不改变该状态。
- NP03/NP04 只记录适用性与未覆盖面；未提供新的双平台或性能证据。
- Windows 结果不替代 Linux；无 Linux 环境时保持未验收。
- 缺少证据时报告精度未验收，不以本记录豁免精度契约。
- 本轮修复不修改任何内核、容差、fixtures 或原生 ABI。

## 7. 未验收边界

- 只验收 Windows；Linux V2 未启动（需要 `linux-gpu:0` 与 `linux-acceptance`）。
- all-probes 全探针矩阵已执行但未通过：唯一剩余失败为 W-D4（证书口径，用户未裁决）；3 项历史捕获/归档工具按用户 2026-10-22 裁决 A 显式排除（§5.1）。
- all-probes `-Minimum` 已按实际通过数 123 同步（W-D6 收口）；W-D4 转绿后需再核对为 124。
- G01 release 未在修复后重跑；release 不宣称通过。
- 未测量 G01 阶段峰值显存。
- 不转授完整产品数值验收、性能验收或双平台准入。
- 不修改任何内核、容差、fixtures 或原生 ABI。

## 8. 引用

- 修复任务与回执：
  - `2026-10-22-layers-m3-wake-native-chain-fix`：`docs/development-evidence/changes/2026-10-22-layers-m3-wake-native-chain-fix.json`
  - `2026-10-22-t4-lineage-repin`：`docs/development-evidence/changes/2026-10-22-t4-lineage-repin.json`
  - `2026-10-22-t4-source-hash-repin-3`：`docs/development-evidence/changes/2026-10-22-t4-source-hash-repin-3.json`
  - `2026-10-22-layers-m3-flex-edges-abi-fix`：`docs/development-evidence/changes/2026-10-22-layers-m3-flex-edges-abi-fix.json`
  - `2026-10-22-layers-m3-sparse-solve-empty-abi-fix`：`docs/development-evidence/changes/2026-10-22-layers-m3-sparse-solve-empty-abi-fix.json`
  - `2026-10-22-g01-wd5-ptx-dir-semantics-2`：`docs/development-evidence/changes/2026-10-22-g01-wd5-ptx-dir-semantics-2.json`
  - `2026-10-22-g01-wd5-recapture-baseline`：`docs/development-evidence/changes/2026-10-22-g01-wd5-recapture-baseline.json`
  - `2026-10-22-g01-wd3-rustdoc-links`：`docs/development-evidence/changes/2026-10-22-g01-wd3-rustdoc-links.json`
  - `2026-10-22-t4-source-hash-repin-5`：`docs/development-evidence/changes/2026-10-22-t4-source-hash-repin-5.json`
- 提交：`eef9321`、`71638e2`、`28bd15f`、`4ffda93`、`c5c47e0`、`eac3df6`、`341a52c`、`c98e246`、`1e3dd2f`、`24d7937`、`3a44e10`、`a433067`、`e83db38`、`5a636a4`、`f917530`。
- 运行证据：`target/g01-final.log`（本机日志，gitignored）、`target/resident-kinematics/` 下的 resident 报告。
- 历史证据：`docs/evidence/three-layer/windows/summary.json` 与同目录日志。
