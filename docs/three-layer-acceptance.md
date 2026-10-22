# 三层迁移验收记录（Windows V1）

日期：2026-10-22。
范围：2026-10-18 三层设计的 Windows 集成验收。
基线提交：`ae89f43`（I1 数据交互层接入）。
Linux V2：需要 Linux 机器，未启动，保持未验收。

本记录只验收迁移范围。
它不转授完整产品验收。
它不宣称 NP01–NP04 精度达标。
Windows 结果不替代 Linux。

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

## 3. 三层定向回归（debug 与 release）

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

## 4. 全量 G01 回归

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

结论：三层定向回归全绿；G01 全量**不通过**，失败项已登记为债务，不宣称全量通过。

## 5. 债务清单

| 编号 | 级别 | 内容 |
| --- | --- | --- |
| W-D1 | P0 | M3 唤醒安全包装元数据校验与打包布局不一致：`TendonWakeMetadata` 要求参数批次≥1、`7+3*nt+nw` 布局与 ntree>0，零树/零批次合法场景被拒（`Input(InvalidTopology{sleep_copy_metadata, invalid_or_stale_metadata})`）。由 `1551c59` 引入。基线 `8f903c5` native_g01 2 过，`64b79b5` 起 1 过 1 败。影响 sleep/tendon-wake 使能的 native G01 链与 wake-guards。 |
| W-D2 | P1 | T4 pinned lineage 漂移：`preserves_current_sources_and_all_thirty_variants` 报 `unexpected lineage change`，阻塞 G01 脚本 host-msvc。 |
| W-D3 | P2 | rustdoc `-D warnings` 三处既有 broken intra-doc links（kinetic_energy.rs:134、device_energy/plan.rs:47、mass_solve/sparse/resident/solve.rs:55）。 |
| W-D4 | 既有 | G01 重力位证书 2bit 差异：f32 输入舍入导致 4 ulp，证书用有理精确值；基线同样失败，属证书口径问题。 |
| W-D5 | 既有 | `MJWARP_PHYSICS_PTX_DIR` 双语义（NVRTC 捕获目录与固定 PTX 加载）；本轮用独立空捕获目录规避，未当门禁。 |
| W-D6 | 既有 | G01 脚本 resident/all-probes 期望计数需与拆分后布局同步；本轮 resident 在计数检查前被 W-D1 阻断。 |

W-D1 是迁移自身回归，不能当历史债务豁免；需在数值主线继续前单独立任务修复。

## 6. NP01–NP04 缺口

- NP01/NP02 仍为未验收；本轮只做迁移范围回归，不宣称精度等价通过。
- NP03/NP04 只记录适用性与未覆盖面；未提供新的双平台或性能证据。
- Windows 结果不替代 Linux；无 Linux 环境时保持未验收。
- 缺少证据时报告精度未验收，不以本记录豁免精度契约。

## 7. 未验收边界

- 只验收 Windows；Linux V2 未启动（需要 `linux-gpu:0` 与 `linux-acceptance`）。
- all-probes 全探针矩阵本轮未执行。
- 未测量 G01 阶段峰值显存。
- 不转授完整产品数值验收、性能验收或双平台准入。
- 不修改任何内核、容差、fixtures 或原生 ABI。
