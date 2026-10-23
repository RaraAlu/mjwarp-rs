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

修复提交后的两轮 full-debug 运行（`scripts/test-windows-g01.ps1 -AllFeatures -FullRegression`）：

- W-D5/W-D3/重钉后复跑目录 `target/g01-acceptance/b0dce6add88645cfa7e674f7a93a569e`（本机日志 `target/g01-final.log`）：all-probes lib 123 过 1 败，唯一失败为 W-D4。
- W-D4 f32 逐步证书重生成后复跑目录 `target/g01-acceptance/9bd3489f5b4542409f104f5feac1c4fb`（本机日志 `target/g01-wd4-final.log`）：

| 阶段 | 结果 |
| --- | --- |
| fmt | 通过 |
| native | 27 过 |
| host-msvc | **709 过**（repin-5 重钉 U038 后的 resident 源码身份后恢复） |
| g01 | 7 过 |
| native-g01 | **2 过** |
| native-only-g01 | 1 过 |
| resident | **102 过**（不再中止） |
| all-probes | lib **124 过 0 败**（461 filtered，重力位证书项已转绿）；all-probes 阶段整体仍**未通过**，见下 |
| all-probes 之后的阶段 | 本轮 all-probes 中止后未随脚本执行；已于 2026-10-22 用同一命令单独补跑，见 §4.2 |

all-probes 的剩余失败明细（非 W-D4、非 M3 回归）：

- G03 `mass_solve_probe` ×3（`gpu::gpu_factors_and_multiple_rhs_match_independent_native_solves`、`gpu::rhs_counts_one_two_three_and_five_reuse_each_world_factor`、`native::native_snapshots_feed_gpu_solves_without_native_physics_at_runtime`）在 NP03 两级判定上 `np03_admitted=false`（案例 4 world 4：native 归一化误差 1.318 超过 mjwarp 红线 0.972，`native_error_exceeds_mjwarp_redline`）。这是完整 G03 的已知验收缺口（`docs/windows-mass-solve-probe.md` 第 7 行「最新结果仍有数值门禁失败」、`docs/windows-g03-block-repair.md` 的 NP03「10 块失败、0 块待验收、不放行」），与 W-D4/M3 无关；此前 cargo 在 lib 的首个失败（W-D4）处 fail-fast 停止，未执行到该测试二进制，本轮 lib 转绿后才暴露。未修改这些测试、容差、断言语义或 fixtures，也不宣称 G03 通过；已登记跟踪任务 `2026-10-22-g01-g03-np03-admission-debt`（proposed，只登记不 start）。
- W-D4 ×1 已修复：`physics::gravity_compensation::tests::gpu_matches_independent_multi_body_reduction_certificate` 在 all-probes lib 中通过（`bit_differences=0`、`max_absolute=0`、`max_ulp=0`），证书按用户 2026-10-22 裁决改为 f32 逐步口径（§5）。
- 历史基线缺失 ×3（已显式排除，不算验收探针）：`runtime::transfer::kernel::recapture` 的 2 项重采集与 1 项归档。经独立复现确认不是 `MJWARP_PHYSICS_PTX_DIR` 语义问题，而是需要历史捕获产物（`target/t4-g03-retest` 等 10/17 冻结产物）与冻结源码基线（`fixtures/precision/t4-kernels.json` 有 7 个产品源身份漂移，历史回执明确保持历史身份）。用户 2026-10-22 裁决 A：从 all-probes 显式排除并写清债务边界，工具保留，手动运行命令见 §5.1。

W-D4 证书项转绿后的 lib 计数为 124（W-D5 语义冲突修复后为 123），all-probes 阶段脚本 `-Minimum` 与实际通过数一致（W-D6 收口），但阶段整体退出码仍受上述 G03 已知失败影响。

W-D5 语义冲突修复：测试侧把 `MJWARP_PHYSICS_PTX_DIR` 统一按捕获根解释，根存在时按 `module_path!()` 与标签分配唯一子目录；mass_solve wrapper 1、rne 4、rne_bias 1 共 6 项稳定转绿（G01 all-probes 等价命令两轮独立空捕获根：122 过 5 败、123 过 4 败）。证据 `docs/development-evidence/g01-wd5-ptx-dir/README.md`。

与历史快照对比：W-D1、W-D2、W-D7 造成的阻塞已消除；resident 阶段从 wake-guards 中止变为 102 项全绿；W-D3 rustdoc 0 警告；all-probes 从“未执行”推进到 lib 123 过 1 败（W-D5 收口）再到 124 过 0 败（W-D4 转绿）。
G01 release 已于 2026-10-22 补跑，结果见 §4.3；release 同样不宣称全量通过。

结论（当前）：三层迁移范围的 debug 回归无新增 M3 回归；G01 全量仍**不被宣称通过**：W-D4 证书项已转绿（all-probes lib 124 过 0 败），但 all-probes 阶段整体因 G03 `mass_solve_probe` 3 项既有门禁缺口未通过（跟踪任务 `2026-10-22-g01-g03-np03-admission-debt`）；其余六阶段已于同日单独补跑通过（§4.2）。release 已补跑，阶段结果与 debug 一致，同一债务处中止（§4.3）；Linux 未跑。

2026-10-23 更新（QA 最终验收）：上述 G03 `mass_solve_probe` NP03 缺口已由矩阵链对齐（`d5b02d5`）解决，跟踪任务已关闭；all-probes 在 env 接线（`6cb5a71`）与 RNE 重基线（`b13d1c5`，§4.4）后，debug/release 均为 **340 过 0 败**，仅余 §5.2 的历史捕获身份等文档化排除项。G01 现状以 §4.7 为准；接线后的 debug/release 双模式最终复验见 §4.7（QA）。

### 4.2 未执行阶段补跑（2026-10-22，任务 `2026-10-22-g01-stage-evidence-refresh`）

W-D4 复跑在 all-probes 中止后，其余六阶段用当前源码（`49fd4d9` 之后，本任务只改文档）单独补跑，命令与环境同 `scripts/test-windows-g01.ps1` 第 112-128 行（脚本未修改）：

| 阶段 | 结果 |
| --- | --- |
| host-gnu | **404 过 0 败**（78 个测试二进制，门槛 ≥200，20.9s） |
| clippy-gnu | 通过，0 警告（2.8s） |
| clippy-gnu-cuda | 通过，0 警告（7.6s） |
| clippy-msvc | 通过，0 警告（22.5s） |
| rustdoc -D warnings | 通过，0 警告（4.5s） |
| diff | 通过（`git diff --check`，仅 Git 的 LF/CRLF 提示，无空白错误） |

证据：`docs/development-evidence/g01-stage-refresh/`（`README.md`、`stage-refresh.json`；原始日志在 gitignored 的 `target/g01-stage-refresh/`）。
边界：只补跑本轮脚本未执行的阶段，未重复已通过阶段，未重跑 all-probes；不宣称完整 G01 通过。

### 4.3 G01 release 双模式证据（2026-10-22，任务 `2026-10-22-g01-release-mode-evidence`）

`scripts/test-windows-g01.ps1 -AllFeatures -FullRegression -Release`（脚本未修改；源码 `ca94aa0`；运行目录 `target/g01-acceptance/f7c4de41f1024372b74e0b60119824ef`，本机日志 `target/g01-release-final.log`，墙钟约 421 秒）在 **all-probes 阶段中止**：

| 阶段 | 结果 |
| --- | --- |
| fmt | 通过（exit 0） |
| native | 27 过 |
| host-msvc | **709 过**（`--all-features --release`） |
| g01 | 7 过（release） |
| native-g01 | 2 过（release） |
| native-only-g01 | 1 过（release） |
| resident | **102 过**（release） |
| all-probes | lib **124 过 0 败**（461 filtered）；阶段整体**失败**：`mass_solve_probe` 3 项既有准入缺口（与 debug 相同，NP03 失败块 factors 2/240、native 2/240、rhs-counts 6/152） |

脚本中止后未执行阶段按脚本第 112-128 行同一定义单独补跑（host-gnu 404 过 0 败、clippy-gnu/-cuda 0 警告、clippy-msvc 0 警告（`--release --all-features`）、rustdoc 0 警告、diff 通过）。
证据：`docs/development-evidence/g01-release/`（`README.md`、`run.json`；原始日志在 gitignored 的 `target/g01-release-final.log` 与 `target/g01-release-refresh/`）。
对比结论：release 与 debug 的阶段计数完全相同（27/709/7/2/1/102、lib 124 过 0 败），在**同一已知债务**处中止，**没有 release 专属的新失败**。release 仍不宣称 G01 全量通过。

### 4.4 RNE 加速度参考重基线（2026-10-23，用户裁决 A）

G01 all-probes 在 env 接线后首次执行 `resident_rne_acceleration::computes_both_flags_for_nonzero_accelerations_without_old_bias_replay`，报 521 个 native-tolerance 失败块（scalars=900288；5 models × 3 gravities × 8 states × worlds [1,2,5,513] × flags 2）。沿用 G03 打法离线 authoring 定位（`docs/development-evidence/g02-rne-authoring-diff/`）：

- 521 块全部是 `mixed-joints`/gravity2/state7/`flag=false`/worst_dof=3；本库 f32 值 `1.5473079681396484` vs 原生 MuJoCo 3.12 f64 `1.5471062036731382`（差 2.02e-4，normalized 3.96，ABS=REL=2e-5）。
- 失败块产品 qfrc 位串与冻结上游 MJWarp（`71da24d…`）逐位相同（0/11 位差）；上游自身对同一原生参考也失败同一块（240 条单世界记录中 1 条超限）。
- 根因：冻结原生 f64 参考 vs MJWarp f32 累加的同口径差异（与 W-D4 重力证书同类），不是产品/上游实现分歧。

用户 2026-10-23 裁决 A：把期望值重基线为冻结上游 MJWarp f32 参考，**容差、断言、产品代码零改动**，旧 f64 参考归档。任务 `2026-10-23-g02-rne-acc-fixture-rebaseline`（登记 `46bdba7`）：

- 只改 `fixtures/rne-acceleration/{mixed-joints,inertial-tree,rotated-tree}.json` 的 qfrc 期望值（改为上游 `qfrc_bias_hex` 的 f32 值，飞块新期望 `1.5473079681396484` 与产品实际值逐位一致）；`zero-dof`/`massless-tree` 因 `nv=0` 字节不变。`manifest.json` 同步 3 个 sha256 与期望来源 provenance（上游提交、pinned 环境、authoring 脚本/证据 SHA256、两次运行一致）。
- 旧 f64 参考归档 `docs/development-evidence/g02-rne-acc-rebaseline/historical-f64/`（含 README 与 SHA256，标注「历史 f64 原生参考，不再作为期望」）。
- 重基线工具 `docs/development-evidence/g02-rne-acc-rebaseline/rebaseline-fixture.ps1`（PowerShell、离线证据工具、非 Python；`-VerifyTwice` 两次运行逐字节一致）。
- 验证：`cargo +msvc test --locked --all-features --test resident_rne_acceleration -- --ignored`（debug）2 过 0 败，`failed_world_field_blocks=0`（此前 521）、scalars=900288、elapsed≈14.1s；`cargo +msvc test --locked --all-features --no-fail-fast` 709 过 0 败 346 ignored（EXIT=0）；all-probes debug 阶段等价命令（复刻脚本 106–215 行、排除清单不变）：cargo 本体 336 过 0 败（78 行），脚本口径含 collision-mocks 自检 4 过为 **79 行、340 过 0 败**（`-Minimum 335` 满足，退出码 0；重基线前同一结构为 339 过 1 败）。
- 债务：`2026-10-22-g02-rne-acc-native-tolerance-debt` 已按裁决关闭；`2026-10-22-g01-g03-np03-admission-debt` 已由矩阵链对齐（`d5b02d5`）解决并同批关闭（§4.1 的旧快照由 QA 验收文档统一刷新）。
- 边界：release 双模式最终验收由 QA 执行（本任务只跑 debug all-probes）；Linux/性能未测；产品 vs 上游 84/240 条 ULP 级差异（首差 `cacc`、最大 normalized 0.743<1）仍存在，属可选后续「RNE 逐位对齐」，不混入本次口径裁决；不宣称完整 G02 验收通过。

### 4.5 最终双模式验收（2026-10-23，QA）

RNE 重基线 `b13d1c5` 入库后，按交接文档执行完整脚本（debug 与 release），14 个阶段全部通过：

| 阶段 | debug | release |
| --- | --- | --- |
| native | 27 过 | 27 过 |
| host-msvc | 709 过 | 709 过 |
| g01 | 7 过 | 7 过 |
| native-g01 | 2 过 | 2 过 |
| native-only-g01 | 1 过 | 1 过 |
| resident | 102 过 | 102 过 |
| all-probes | **340 过 / 0 败**（78 个 result 行） | **340 过 / 0 败**（78 个 result 行） |
| host-gnu | 404 过 | 404 过 |
| clippy-gnu / clippy-gnu-cuda | 0 警告 | 0 警告 |
| clippy-msvc | 0 警告 | 0 警告 |
| rustdoc（`-D warnings`） | 0 警告 | 0 警告 |
| diff | 通过 | 通过 |
| 全脚本墙钟 | 789.1s | 652.9s |

- 运行目录：debug `target/g01-acceptance/90e611aeee0b4f1fbbefc2681bdcae84`、release `target/g01-acceptance/9bf374a9fd4c428bb65ef059442932bc`；本机日志 `target/g01-qa-20261023-012436/`。
- 排除清单见各 run 的 `probe-reports/exclusions.json`：W-D5 3 项（§5.1）、历史捕获身份 6 项（§5.2）、env/外部产物未接线 1 项；未改断言、容差或排除清单。
- fixtures 来源：`git log --oneline -- fixtures/rne-acceleration` 最新为 `b13d1c5`（RNE 重基线），`git status --short fixtures` 为空；QA 任务未改夹具。
- 证据：`docs/development-evidence/g01-fast-loop/`（README、`final-acceptance.json`、`baseline-timings.json`）。
- 结论：**G01 debug/release 双模式通过（含文档化排除项；RNE 已按裁决重基线）**。Linux、性能与完整上游等价仍未验收。此结论对应接线前版本（all-probes 340 过 0 败、10 项排除）；W14/W15/U038 新测试接线后的 debug/release 双模式最终复验见 §4.7（346 过 0 败、11 项排除）。

### 4.6 W14/W15/U038 新测试接线后的 debug 复跑（2026-10-23，任务 `2026-10-23-g01-script-new-tests-wiring`）

当前 HEAD 的 all-probes 一度有 3 个 target 失败：`launcher-tile`、`launcher-warp-headers` 的 ignored NVRTC 测试缺 `MJWARP_CUDA_INCLUDE`/CUDA bin PATH；`t4_gravity_3d_product_chain` 缺 `MJWARP_GRAVITY_3D_EVIDENCE_DIR`，接线后会到达 U038-3D 8/77 已知债务断言。修复：在 all-probes env 接线段落补 CUDA include/PATH、补 3D 证据目录；该 live 探针按用户 2026-10-23 裁决 B 加入显式 known-debt 排除（§5.3）；`-Minimum` 同步为 346。另为使脚本在本机 Windows PowerShell 5.1（codepage 936，无 pwsh7 且无法联网安装）可执行，4 个 G01 脚本补 UTF-8 BOM、原生 stderr 兼容包装与 collision 脚本的 PS 5.1 SHA API 等价改写；断言、容差与检查语义不变。

`scripts/test-windows-g01.ps1 -AllFeatures -FullRegression`（debug；源码 `b0b4b8a` + 本任务脚本改动；运行目录 `target/g01-acceptance/10a6e9fd1b164c0fa84d3e0c56191188`；墙钟 728.8s）14 个阶段全部通过：

| 阶段 | 结果 |
| --- | --- |
| fmt | 通过（exit 0） |
| native | 27 过（受限 27） |
| host-msvc | 722 过 |
| g01 | 7 过 |
| native-g01 | 2 过 |
| native-only-g01 | 1 过 |
| resident | 102 过 |
| all-probes | **346 过 / 0 败**（83 个 result 行；W-D5 3 + 历史捕获 6 + env 1 + U038-3D known-debt 1 = 11 项文档化排除） |
| host-gnu | 406 过 |
| clippy-gnu / clippy-gnu-cuda / clippy-msvc | 0 警告 |
| rustdoc（`-D warnings`） | 0 警告 |
| diff | 通过 |

- 证据：`docs/development-evidence/g01-new-tests-wiring/`（README、`debug-run.json`、接线/排除项/前后失败记录）。
- release 双模式已由 QA 复验通过（§4.7）；§4.5 的双模式声明仍只对应接线前版本。Linux、性能与完整上游等价仍未验收。

### 4.7 接线后 debug/release 双模式最终验收（2026-10-23，QA，任务 `2026-10-23-final-acceptance-rerun`）

Backend 接线（`45baa0e`）入库后，QA 按同一命令执行 debug 与 release 两轮完整脚本，**14 个阶段均 exit 0**，all-probes **debug 与 release 各 346 过 / 0 败**（各 83 个 result 行），排除项为 **11 项文档化排除**（W-D5 3、历史捕获身份 6、env/外部产物 1、U038-3D 已知债务 1；§5.1–§5.3）。运行源码 HEAD `6e8c79a`，脚本为 `45baa0e` 接线版本。

| 阶段 | debug | release |
| --- | --- | --- |
| fmt | 通过 | 通过 |
| native | 27 过 | 27 过 |
| host-msvc | 722 过 | 722 过 |
| g01 | 7 过 | 7 过 |
| native-g01 | 2 过 | 2 过 |
| native-only-g01 | 1 过 | 1 过 |
| resident | 102 过 | 102 过 |
| all-probes | **346 过 / 0 败**（83 result 行；11 项排除） | **346 过 / 0 败**（83 result 行；11 项排除） |
| host-gnu | 406 过 | 406 过 |
| clippy-gnu / clippy-gnu-cuda / clippy-msvc | 0 警告 | 0 警告 |
| rustdoc（`-D warnings`） | 0 警告 | 0 警告 |
| diff | 通过 | 通过 |
| 全脚本墙钟 | 689s | 656s |

- 运行目录：debug `target/g01-acceptance/fff94fdd2e7c4e489362480b369c454b`、release `target/g01-acceptance/f0bb413b3de5423d91a82484c8a9d5de`；本机日志 `target/final-accept-{debug,release}.log`。
- 环境接线（两模式 `probe-reports/env-wiring.json`）：`cuda_include`、`cuda_bin_path`、`gravity_3d_evidence` 均 wired；`probe-reports/exclusions.json` 的 `known_debt` 单列 `live::live_three_d_product_chain_matches_frozen_upstream` 与理由（U038-3D 8/77 已知债务，用户 2026-10-23 裁决 B），不静默跳过。
- NP03 两级判定汇总（两模式一致）：9 个 scope、**79,286 块**（79,202 原生放行 + 84 二级参考），`failed=0`、`pending=0`，比较标量 8,527,795。
- 证据：`docs/development-evidence/g01-fast-loop/`（README §5.1、`final-acceptance-wiring.json`）与本文件。
- 结论：**G01 debug/release 双模式通过（含 11 项文档化排除项；RNE 已按裁决重基线；U038-3D 已知债务显式排除、未验收）**。Linux（按用户 2026-10-24 决定暂时终止，待 Windows 全量完成后集中回测，§7.1）、性能与完整上游等价仍未验收。（批次 1–4 交付与三处修复后的最终复验见 §4.8，HEAD `66256c9`。）

### 4.8 重钉与三处修复后 debug/release 双模式最终验收（2026-10-23/24，QA，任务 `2026-10-23-g01-final-acceptance-3`）

批次 1–4（20 条切片）交付后，T4 冻结捕获身份审计（`preserves_current_sources_and_all_thirty_variants`）因 `src/physics/mod.rs`/`history.rs` 合法改动而漂移，G01 host-msvc 一度失败（失败记录见 `docs/development-evidence/g01-head-drift/`）。按用户裁决 A 依次完成：T4 源码身份重钉（`2026-10-23-t4-source-hash-repin-12`，只改哈希字面量，组合摘要 `7eb6e7a3…`→`d7f6c7d1…`）、rustdoc 私有项断链修复（`2026-10-23-g01-rustdoc-private-link-fix`，`collision.rs` 一行文档）、A10 measure release 计时断言修复（`2026-10-23-a10-measure-release-timing-fix`，测试负载改为可测的 `black_box` 自增，保留 `nanos > 0` 严格断言）后，QA 在 HEAD `66256c9` 按同一命令执行 debug 与 release 两轮完整脚本，**14 个阶段均 exit 0**。

| 阶段 | debug | release |
| --- | --- | --- |
| fmt | 通过 | 通过 |
| native | 27 过 | 27 过 |
| host-msvc | 796 过 | 796 过 |
| g01 | 7 过 | 7 过 |
| native-g01 | 2 过 | 2 过 |
| native-only-g01 | 1 过 | 1 过 |
| resident | 102 过 | 102 过 |
| all-probes | **346 过 / 0 败**（83 result 行；11 项排除） | **346 过 / 0 败**（83 result 行；11 项排除） |
| host-gnu | 480 过 | 480 过 |
| clippy-gnu / clippy-gnu-cuda / clippy-msvc | 0 警告 | 0 警告 |
| rustdoc（`-D warnings`） | 0 警告 | 0 警告 |
| diff | 通过 | 通过 |
| 全脚本墙钟 | 709s | 627s |

- 运行目录：debug `target/g01-acceptance/383d4b51c15c462982a5b06cd47503de`、release `target/g01-acceptance/7d4d70d46e7f4fc3a86f3963de4e878a`；本机日志 `target/final-g01-check/{debug,release}-final3.log`。
- 排除清单（`probe-reports/exclusions.json`）：仍为 **11 项**（W-D5 3 + 历史捕获身份 6 + env/外部产物 1 + U038-3D 已知债务 1），与 §4.7 一致，**未新增排除**；`probe-reports/env-wiring.json` 两模式一致（`t4_current_captures`/`gravity_3d_evidence` = fresh-directory，其余 wired/reused）。
- 与 §4.7 差异（HEAD `6e8c79a`→`66256c9`）：host-msvc 722→796、host-gnu 406→480（批次 1–4 新增 20 条切片测试）；all-probes 346/0、83 result 行、11 项排除均不变；rustdoc 0 警告；墙钟 689s/656s→709s/627s。
- 证据：`docs/development-evidence/g01-fast-loop/`（README §5.2、`final-acceptance-final.json`）与本文件。
- 结论：**G01 debug/release 双模式通过（含 11 项文档化排除项；HEAD 66256c9；U038-3D 已知债务显式排除、未验收）**。Linux（按用户 2026-10-24 决定暂时终止，待 Windows 全量完成后集中回测，§7.1）、性能与完整上游等价仍未验收。

## 5. 债务清单

| 编号 | 级别 | 状态 | 内容与证据 |
| --- | --- | --- | --- |
| W-D1 | P0 | **已修复** | M3 唤醒安全包装零树/零批次元数据回归，由 `1551c59` 引入。修复提交 `eef9321`（登记）+ `71638e2`（修复），任务 `2026-10-22-layers-m3-wake-native-chain-fix` 已验证。`native_g01 --include-ignored` 2 过；wake-guards 1 过；resident 脚本不再在 wake-guards 中止。 |
| W-D2 | P1 | **已修复** | T4 pinned lineage 漂移，阻塞 host-msvc。重钉提交 `28bd15f`（登记）+ `4ffda93`；W-D7 修复改变 resident 源码后由 `2026-10-22-t4-source-hash-repin-3` 再次重钉：`c5c47e0`（登记）+ `eac3df6`。host-msvc 697 过，lineage 不再中止。 |
| W-D3 | P2 | **已修复** | rustdoc `-D warnings` 三处既有 broken intra-doc links（kinetic_energy.rs、device_energy/plan.rs、mass_solve/sparse/resident/solve.rs）为中文量纲记号被当作 intra-doc link。任务 `2026-10-22-g01-wd3-rustdoc-links` 用代码文本包裹修复，未放宽门禁、未新增豁免；`cargo +1.99.0-x86_64-pc-windows-msvc doc --locked --all-features --no-deps`（`RUSTDOCFLAGS=-D warnings`）0 警告通过。 |
| W-D4 | 已知债务 | **已修复（用户 2026-10-22 裁决口径第 1 项）** | G01 重力位证书 2 bit 差异：f32 输入舍入导致 4 ulp，证书用有理精确值。任务 `2026-10-22-g01-gravity-cert-f32-regen` 按上游公式 + f32 输入逐步舍入重生成证书：生成器 `docs/development-evidence/g01-gravity-cert-f32/gen.rs`（rustc 直接编译、非 Python、不读取 GPU 输出；精确有理自检 28/28 逐位复现旧证书），world 1/4 的 dof 2 由 `0x3f000000` 变为 `0x3efffffc`（4 ulp，max_abs 1.1920929e-7 ≤ 2e-6），其余 26 槽逐位一致；旧证书归档为 `docs/development-evidence/g01-gravity-cert-f32/legacy-exact-certificate.json`，口径变更与差异见同目录 `README.md`；容差、断言逻辑、内核/PTX、fixtures 未改。all-probes lib 124 过 0 败，该项测试转绿。 |
| W-D5 | 已知债务 | **部分修复 + 3 项显式排除** | `MJWARP_PHYSICS_PTX_DIR` 测试侧目录分配语义冲突：已统一为“捕获根 + 唯一子目录”，6/9 转绿（mass_solve wrapper 1、rne 4、rne_bias 1），修复提交 `3a44e10`（任务 `2026-10-22-g01-wd5-ptx-dir-semantics-2`），证据 `docs/development-evidence/g01-wd5-ptx-dir/README.md`。剩余 3 项重采集/归档为历史捕获与冻结源码基线缺失，**用户 2026-10-22 裁决 A**：all-probes 显式排除并写清边界（任务 `2026-10-22-g01-wd5-recapture-baseline`），工具保留、手动运行命令见 §5.1；未改 3 个测试、未伪造基线。 |
| W-D6 | 既有债务 | **已收口** | G01 脚本 resident/all-probes 期望计数与拆分后布局同步：`2026-10-22-g01-gate-script-sync` 已同步 resident 102；all-probes `-Minimum` 由 169 改为实际通过数 123（3 项历史工具显式排除后 lib 通过 123，W-D4 转绿后 lib 通过 124，`-Minimum 123` 仍满足），由 `2026-10-22-g01-wd5-recapture-baseline` 脚本改动同步。 |
| W-D7 | P0 | **已修复** | M3 柔体边安全包装空 ABI 回归（`068fd14` 引入）：`FlexEdgesStaticDims` 误拒单世界体/零 nnz 合法输入。修复提交 `341a52c`（登记）+ `c98e246`（修复），任务 `2026-10-22-layers-m3-flex-edges-abi-fix` 已验证；`flex_edge_free_body...` 恢复、resident 脚本 102 项全绿。 |
| W-D8 | P1 | **已修复** | M3 稀疏求解包装空计划 ABI 回归（`38a4960` 引入）：`SparseSolveStaticDims` 误拒 n=0/nc=0 合法空计划。修复提交 `1e3dd2f`（登记）+ `24d7937`（修复），任务 `2026-10-22-layers-m3-sparse-solve-empty-abi-fix` 已验证；`tests/resident_sparse_solve -- --ignored` 2 过 0 败。 |

W-D1、W-D7、W-D8 是迁移自身回归，已在独立任务中修复，未被当作历史债务豁免。
W-D4 已按用户 2026-10-22 裁决（f32 逐步口径）修复；W-D5 语义冲突已部分修复（6/9），剩余 3 项历史工具已按用户裁决 A 显式排除；W-D3 已修复；W-D6 已收口（resident 102 与 all-probes `-Minimum 123` 对齐；W-D4 转绿后 lib 实际通过数为 124）。
W-D4 证书项转绿后，all-probes 阶段暴露的 3 项 `mass_solve_probe` NP03 门禁失败属完整 G03 已知缺口（见 §4.1、§7），不是 W-D4/M3 回归，未在本轮修改。
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

### 5.2 历史捕获身份债务（all-probes 显式排除，2026-10-23）

以下 6 项 audit 以历史 BASE 捕获身份/历史模板对照当前源码，从 all-probes 显式排除（`<run>/probe-reports/exclusions.json` 的 `historical_audits`），不是验收探针：

1. `audit_local_t4_precision_without_gpu_execution`
2. `rigid_internal::tests::audit_frozen_rigid_internal_and_output_sources`
3. `field_internal::flex_matrices::tests::audits_frozen_matrices_and_generated_face_artifacts`
4. `packing_routes::flex_internal::audits_frozen_flex_sources_and_retained_artifacts`
5. `packing_routes::routes::audit_remaining_packing_sources_and_captures`
6. `topology_frames::tests::audit_frozen_topology_and_frame_sources`

原因：G03 矩阵链对齐（`d5b02d5`）有意变更 f32 算序，这些 audit 的历史捕获/模板先于该变更；重生成需要被排除的重采集工具与冻结捕获流水线（跟踪任务 `2026-10-22-t4-historical-capture-rebaseline`，proposed）。
证据：debug/release 运行目录的 `probe-reports/exclusions.json`（三组单列可审计：W-D5 原 3 项、历史捕获身份 6 项、env/外部产物未接线 1 项）。
恢复条件：重基线任务完成重采集/重基线后解除排除；在此之前不把它们当静默跳过，也不伪造基线。

用户 2026-10-23 裁决 B（维持排除 + 归档，不重基线）：上述 10 项排除维持现状，
`scripts/test-windows-g01.ps1` 的排除清单保持不动；4 个历史捕获目录
（`target/t4-g03-retest`、`target/t4-face-operators`、`target/t4-final-face-compile`、`target/t4-rigid-recapture`）
已归档为证据 `docs/development-evidence/t4-historical-capture-archive/t4-historical-captures.tar.gz`
（724 文件、原始 23 842 480 字节、压缩后 4 006 399 字节、sha256 `95e7b430…a915`），逐文件
sha256/size/mtime 清单在同目录 `manifest.json`，可用 `make-archive.ps1 -Verify` 一条命令校验；
归档与 QA 预检证据见 `docs/development-evidence/t4-historical-capture-archive/README.md` 与
`docs/development-evidence/t4-rebaseline-precheck/`。跟踪任务 `2026-10-22-t4-historical-capture-rebaseline`
保持 proposed（不 abandon），恢复条件不变：将来完成重采集/重基线后解除排除；在此之前不伪造历史身份、不宣称完整通过。

### 5.3 U038-3D 已知债务排除（all-probes 显式排除，2026-10-23）

`t4_gravity_3d_product_chain` 的 live 探针 `live::live_three_d_product_chain_matches_frozen_upstream` 在环境接线后会到达全链 `qfrc_gravcomp` 8/77 超 2e-6 的已知债务断言（`assert_eq!(total_outside, 0)`）。按用户 2026-10-23 裁决 B 登记为 U038-3D 已知债务，加入 `scripts/test-windows-g01.ps1` 的 `excludedKnownDebt` 清单，并在 `<run>/probe-reports/exclusions.json` 的 `known_debt` 字段单列理由（`known_debt_reason`）；不静默跳过、不改断言与容差。

手动复核命令（需要 GPU、冻结产物与全新的证据目录；预期在 8/77 断言处失败，属已知债务，不作为验收探针）：

```powershell
$env:MJWARP_GRAVITY_3D_EVIDENCE_DIR = '<repo>\target\<全新目录>'
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --test t4_gravity_3d_product_chain -- --ignored --test-threads=1 --nocapture
```

证据：`docs/development-evidence/u038-3d/known-deviation.md`、`docs/development-evidence/g01-new-tests-wiring/`。

## 6. NP01–NP04 缺口

- NP01/NP02 仍为未验收；重复修复不改变该状态。
- NP03/NP04 只记录适用性与未覆盖面；未提供新的双平台或性能证据。
- Windows 结果不替代 Linux；无 Linux 环境时保持未验收。
- 缺少证据时报告精度未验收，不以本记录豁免精度契约。
- 本轮修复不修改任何内核、容差、fixtures 或原生 ABI。

## 7. 未验收边界

- 只验收 Windows；Linux 侧按用户 2026-10-24 决定**暂时终止**（开发、验收与回测均暂停），等 Windows 全量开发完成后集中回测一次，不零散开跑；当前程序以 Windows 单平台完成为目标；Linux V2 未启动（原需 `linux-gpu:0` 与 `linux-acceptance`），细节见 §7.1。
- G01 现状（2026-10-23/24，§4.8，最终 HEAD `66256c9`）：debug/release 双模式通过（含 11 项文档化排除项；T4 源码身份已重钉、rustdoc 私链与 A10 release 计时断言已修复；U038-3D 已知债务显式排除、未验收）；两模式 14 阶段均 exit 0，host-msvc **796 过**、all-probes **346 过 0 败**（各 83 result 行）；仅余 W-D5 3 项（§5.1）、历史捕获身份 6 项（§5.2）、env/外部产物未接线 1 项、U038-3D 已知债务 1 项（§5.3），共 11 项文档化排除，未新增。不宣称 G02 完整验收。（§4.7 的 722/346 数字对应历史 HEAD `6e8c79a`。）
- all-probes `-Minimum 346` 满足（debug/release 均 346 过，§4.8）；旧 `-Minimum 123`/`335` 记录属历史阶段，W-D6 已收口。
- 历史记录：2026-10-22 的 full-debug 曾中止在 all-probes（§4.1–§4.3），相关债务已由 `d5b02d5`、`6cb5a71`、`b13d1c5` 收口；保留原文作为变更轨迹。
- W13（2026-10-24）在 §4.8 的 HEAD `66256c9` 之后新增 `cuda-probe` 下的 nvJitLink 绑定源码与 2 项 GPU 测试（`tests/nvjitlink_probe.rs`）；T4 源码身份清单不覆盖这些文件，未重钉。all-probes 会纳入这 2 项，运行前须重跑 `scripts/prepare-cuda-probe.ps1`（取 nvJitLink 12.9.86），对照测试还需 `target/native-mathdx7/attempt-1/` 的离线证据输入。§4.8 结论不覆盖 W13 之后的 HEAD；G01 双模式复跑由 QA 另立任务，复跑前不宣称新 HEAD 的 G01 通过。
- 未测量 G01 阶段峰值显存。
- 不转授完整产品数值验收、性能验收或双平台准入。
- 不修改任何内核、容差、fixtures 或原生 ABI（RNE 期望值按用户裁决重基线，见 §4.4）。
- U038-3D 三维链：全链 `qfrc_gravcomp` **8/77 超 2e-6**（max 4.29e-6、总位差 122；运动学 1400 项 0 超差，`gravity_operator` 77/77 0 超差）。用户 2026-10-23 裁决 B 登记为**已知债务**，**U038-3D 未验收、不宣称通过**（`docs/development-evidence/u038-3d/known-deviation.md`；PTX 逐表达式对照工具 `scripts/u038-ptx-diff.ps1`、证据 `docs/development-evidence/u038-3d/ptx-diff/`）。根因：三表达式源形态收缩一致但内核内 FMA 收缩由内联上下文决定，且 com/cdof 与上游分层原子累加存在内核分解/累加顺序差异；债务跟踪任务 `2026-10-23-u038-3d-gravity-propagation-debt-2`（proposed，不 start，验收条件不变）。

### 7.1 Linux 终止与 Windows 完工后集中回测（用户 2026-10-24 决定）

用户 2026-10-24 决定：**Linux 相关暂时终止**——开发、验收与回测全部暂停；等 Windows 全量开发完成后，再在 Linux 环境集中回测一次，不零散开跑。此前 2026-10-23 的 deferred 登记由本节取代。

排程调整（`.normify`，2026-10-24）：以下任务 status 由 proposed 改为 **backlog**（远期待办；不 abandon、不伪造、不算阻塞，不再占就绪队列），`note` 已记录依据、集中回测入口与恢复路径：

- `2026-10-18-layers-linux`（V2 Linux 三层集成）
- `2026-10-18-linux-toolchain-validation`（Linux 锁版工具链验证）
- `2026-10-23-np03-dual-platform`（Windows 单平台数值等价已由 `2026-10-23-np03-windows-evidence` 交付，剩余仅 Linux 侧）
- `2026-10-23-np04-performance`（Windows 单平台耗时与峰值显存已由 `2026-10-23-np04-windows-performance` 交付，剩余仅 Linux 侧）
- `2026-10-18-toolchain-clippy-acceptance` 保持 proposed/blocked（其 Linux 前置已转 backlog）

集中回测载体：**`2026-10-24-linux-batch-retest`**（backlog）。触发条件 = Windows 全量开发完成并经用户确认；届时一次性申请 Linux RTX 5080 与 `linux-acceptance` 资源，按 `scripts/prepare-linux-native.sh`、`scripts/prepare-linux-cuda.sh` 与 `scripts/test-linux-g01.sh --all-features --full-regression`（debug 与 `--release`）集中执行，合并覆盖上列四条范围，产出单一证据 `docs/development-evidence/linux-batch-retest/`。

边界：Windows 结果不替代 Linux；集中回测实际跑通前，不宣称 Linux、双平台或 NP03/NP04 完整验收；不复用历史 Linux 数字当作本次结果；不改容差、断言、排除清单与 fixtures。

恢复条件：Windows 全量开发完成（用户确认）后 promote 上述 backlog 任务，安排 Linux RTX 5080 与 `linux-acceptance`，一次性集中回测；在此之前 Linux 保持未验收。

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
  - `2026-10-22-g01-gravity-cert-f32-regen`：`docs/development-evidence/changes/2026-10-22-g01-gravity-cert-f32-regen.json`（W-D4 证书 f32 逐步口径；生成器与证据在 `docs/development-evidence/g01-gravity-cert-f32/`）
  - `2026-10-22-g01-stage-evidence-refresh`：`docs/development-evidence/changes/2026-10-22-g01-stage-evidence-refresh.json`（未执行阶段补跑；证据在 `docs/development-evidence/g01-stage-refresh/`）
  - `2026-10-22-g01-release-mode-evidence`：`docs/development-evidence/changes/2026-10-22-g01-release-mode-evidence.json`（release 双模式证据；证据在 `docs/development-evidence/g01-release/`）
  - `2026-10-23-g01-acceptance-fast-loop`：`docs/development-evidence/changes/2026-10-23-g01-acceptance-fast-loop.json`（快验脚本、耗时基线与最终双模式验收证据，`docs/development-evidence/g01-fast-loop/`）
  - `2026-10-23-g01-acceptance-doc-refresh`：`docs/development-evidence/changes/2026-10-23-g01-acceptance-doc-refresh.json`（本文档现状刷新与历史捕获身份债务小节）
  - `2026-10-22-g01-g03-np03-admission-debt`：proposed 跟踪任务，登记提交 `de25d4d`，不 start；指代 all-probes 剩余的 3 项 G03 `mass_solve_probe` 准入缺口
  - `2026-10-23-u038-3d-known-deviation`：U038-3D 已知债务登记（用户 2026-10-23 裁决 B，不宣称通过），证据 `docs/development-evidence/u038-3d/known-deviation.md` 与 `docs/development-evidence/changes/2026-10-23-u038-3d-known-deviation.json`；PTX 对照证据提交 `71d9d42`、`722e979`
  - `2026-10-23-u038-3d-gravity-propagation-debt-2`：proposed 债务跟踪任务（承接已 abandon 的 `2026-10-23-u038-3d-gravity-propagation-debt`，验收条件不变，不 start）
- 提交：`eef9321`、`71638e2`、`28bd15f`、`4ffda93`、`c5c47e0`、`eac3df6`、`341a52c`、`c98e246`、`1e3dd2f`、`24d7937`、`3a44e10`、`a433067`、`e83db38`、`5a636a4`、`f917530`、`de25d4d`；W-D4 修复提交 `49fd4d9`、阶段补跑提交 `10913ce`；2026-10-23 RNE 重基线 `b13d1c5`、债务关闭 `8a0bfde`。
- 运行证据：`target/g01-final.log`、`target/g01-wd4-final.log`、`target/g01-stage-refresh/`、`target/g01-release-final.log`、`target/g01-release-refresh/`（本机日志，gitignored）、`target/resident-kinematics/` 下的 resident 报告。
- 历史证据：`docs/evidence/three-layer/windows/summary.json` 与同目录日志。
