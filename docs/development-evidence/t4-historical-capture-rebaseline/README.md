# T4 历史捕获重基线：可用性核查（夜间轻量段，2026-10-27）

任务：`2026-10-22-t4-historical-capture-rebaseline`（proposed → 本片 start，夜间只做轻量/CPU 段）。
方式：**只读核查**——归档完整性、冻结产物、工具/脚本存在性、3 个被 all-probes 排除的重采集/归档工具定向复跑、
身份漂移与缺口清单。**未改历史捕获、`fixtures/`、`scripts/`、`tests/`、产品代码；未跑 G01 全脚本；未开 GPU 内核。**

## 结论（先说结果）

1. **流水线物料齐**：归档脚本/清单 10/10 sha 与 manifest 一致；4 个历史捕获目录都在（合计 848 文件 / 22.8 MB）；
   toolchain 与原生 DLL 身份对得上（dll sha = `79b61d22…` = `fixtures/g01/manifest.json`）。
2. **3 个重采集/归档工具在当前源码下依然全部失败（3/3，exit 101）**，失败点与 2026-10-23 预检（`2026-10-23-t4-rebaseline-precheck`）完全一致：
   `source drift: src/physics/attached.rs`（`tests_recapture.rs:50`）、`capture coverage mismatch`（rotation 变体）、
   `bit_differences` 断言 0 vs 2078（`tests_archive.rs:35`）。
3. **不能就地重生成**：10 个旋转族模板锚点全部失配、2 个 retained 变体（`rigid_mass_matrix`/`rigid_mass_solve`）
   锚点已变、13/30 个 artifact 的历史身份与当前身份不同。恢复 6 项 audit 必须先重建
   `tests/support/t4_rigid_capture` 的模板/锚点 → **越出本任务 `files`**，需扩范围或另立前置片。
4. **夜间段到此为止**：不需要现在跑 G01 全量；全量只在「重基线（选项 A）收尾」时按 Chief/用户点名跑一次。
   本片不宣称任何 audit 恢复、不宣称 T4 完整。

## 1. 物料核查（全部实测）

| 项 | 结果 |
| --- | --- |
| 归档 manifest 完整性 | `docs/evidence/t4-g03-retest/manifest.json` 10 个文件 sha **10/10 一致** |
| 归档流水线脚本 | `run.ps1`（8909 B）/ `summarize.ps1`（5717 B）在，sha 与 manifest 一致 |
| 历史捕获目录 | `target/t4-g03-retest` 247 文件 / 7.6 MB；`t4-face-operators` 194 / 11.2 MB；`t4-final-face-compile` 94 / 1.3 MB；`t4-rigid-recapture` 189 / 2.7 MB（测试后未修改） |
| toolchain | `target/toolchains/cuda-12.8.1`（含 include/bin）、`mujoco-3.12.0/package`（dll sha `79b61d22…` = g01 manifest） ✓ |
| 身份清单 | 历史 `fixtures/precision/t4-kernels.json`：30 artifacts、`source_revision=4dc88934…`；当前 `t4-current-kernels.json`：30 artifacts、`source_revision=8a5bac00…` |
| 身份差异（按 artifact 序配对） | `ptx_sha256` 相同 17 / 不同 **13**；`source_sha256` 相同 17 / 不同 **13** |
| 产品源漂移（BASE 20 源） | **10/20 与 BASE 不同**（`attached`、`com_velocity`、`flex_face`、`mass_matrix`、`mass_solve`、`mod`、`resident`、`rne_bias`、`transfer`、`transfer/kernel`）——与 BASE 是历史身份这一事实一致 |
| 复跑产物 | `target/t4-availability-2026-10-27/{<test>.log,summary.json,capture-*}`（target 未入库） |

## 2. 3 个工具定向复跑（2026-10-27，当前 HEAD `be5bffd4`）

命令模板（`--ignored --exact --nocapture`，环境见 §3）：

```text
cargo test --locked --features cuda-probe --lib <filter> -- --ignored --exact --nocapture
```

| 工具 | exit | 秒 | 首个失败点 |
| --- | --- | --- | --- |
| `recompile_rigid_variants_without_launching_physics` | **101** | 0.8 | `tests_recapture.rs:50` → `"source drift: src/physics/attached.rs"`（常量只覆盖 3 个源身份，实际漂移 10 个） |
| `recaptures_only_missing_rotation_variants` | **101** | 0.2 | `"capture coverage mismatch"`（previous 捕获与 `t4-current-kernels.json` 不一致） |
| `archives_face_operator_repair_with_failed_numeric_gate` | **101** | 0.4 | `tests_archive.rs:35` 断言 `left=0 vs right=2078`（face 产物已于 10/22 重生成，失败数值门前提取消失） |

**结论：3/3 失败，失败点与 10-23 预检逐字一致 → 工具代码没有退化，是「历史模板锚点 vs 当前源码」的结构性缺口。**

## 3. 环境前置（复跑必需，实测）

- `MJWARP_CUDA_INCLUDE=target/toolchains/cuda-12.8.1/include`，`target/toolchains/cuda-12.8.1/bin` 进 `PATH`（NVRTC）。
- `MJWARP_PHYSICS_PTX_DIR` 必须指向**不存在**的唯一输出、且父目录已存在（`capture_output` 语义）。
- 归档 `run.ps1` 另需 `MJWARP_NATIVE_MOCKS`（含 `report.json`），由 `scripts/test-windows-native.ps1` 生成 mock DLL；
  该 env 未设时 `run.ps1:93` 直接 `throw '原生环境缺失'`。
- **归档 `run.ps1` 不可直接执行**：内部硬编码 `$root = 'C:\Rust\mjwarp-rs'`（本机是 `D:\Rust\mjwarp-rs`），且带
  「`calls.json` 存在即禁止重跑」守卫。重基线若用全量捕获，须**复制一份适配当前路径的新 runner**（不改归档件，
  归档件是 manifest 哈希绑定的证据）。

## 4. 缺口清单（恢复 6 项 audit 所需，按依赖序）

1. **模板锚点重建**（越界 `files`）：`tests/support/t4_rigid_capture/{recipe.rs,lineage.rs}` 的 10 个旋转族
   + `rigid_mass_matrix`/`rigid_mass_solve` 两个 retained 变体锚点与期望数量。
2. **重生成 30×2 模式捕获**（compile-only，NVRTC）：debug-ptx / release-ptx 全变体 + 历史模板锚点。
3. **重写身份**：`fixtures/precision/t4-kernels.json`（BASE 身份 30 artifacts）与 `t4-current-kernels.json`。
4. **重跑 6 项 audit**：`audit_local_t4_precision_without_gpu_execution`、`rigid_internal::…`、
   `field_internal::flex_matrices::…`、`packing_routes::flex_internal::…`、`packing_routes::routes::…`、
   `topology_frames::tests::…`（排除清单在 `scripts/test-windows-g01.ps1:112-119`，本片未改）。
5. **移除排除项**：按项从 `$script:excludedHistoricalAudits` 删除；另 1 项 env/外部产物
   （`captures_loaded_face_machine_code_without_launching_physics`，需 `target/t4-face-operators/product.ptx`）
   仍单列在 `$script:excludedEnvUnwired`，不随本任务自动解除。
6. **repin 影响面**：BASE 身份与 T4 483 双模式历史捕获绑定，改写身份需文档注明、只能 git 回退；
   历史 repin 链（`t4-source-hash-repin*` 共 13 轮）说明此类身份改写会连带多片，须先评估再动。

## 5. 剩余工作与是否要早上全量

- **不需要**现在或早上跑 G01 全量：全量只在「重基线（选项 A）实现完成、准备收口」时按 Chief/用户点名跑一次，
  并绑定最终 HEAD。
- 裁决结果（2026-10-27，用户/Chief）：**走 B**——维持 6 项历史捕获身份 audit + 1 项 env/外部产物 audit 的显式排除，
  不改身份 fixture、不改判据、不做 A/C、不跑全量 G01。A 的定义与代价（约 3–5 天、重建 `tests/support/t4_rigid_capture`
  锚点、改写 BASE 身份、需用户批准）保留为**恢复路径**；C（冻结历史身份对照）**不默认采用**（比较口径变化需用户单独批准）。
- **A 留 backlog**：`2026-10-27-t4-historical-capture-rebaseline-decision`（不 start）；原任务
  `2026-10-22-t4-historical-capture-rebaseline` 已 abandon。B 的落盘位置：`docs/three-layer-acceptance.md` §5.2
  末段与 `scripts/test-windows-g01.ps1` 的 `$script:excludedHistoricalAudits` 注释块（仅注释，排除成员与 `--skip`
  行为未变）。

## 6. 边界

- 只读核查段未改历史捕获、`fixtures/precision`、`docs/evidence/t4-g03-retest`、`tests/`、产品代码；B 落盘只改
  `docs/three-layer-acceptance.md` 与 `scripts/test-windows-g01.ps1` 的注释文字（不改排除成员、不改断言/容差/判据）。
- 未运行 G01 全脚本；未使用 GPU 内核（仅 NVRTC 编译与宿主读取）。
- 不宣称任何 audit 恢复、不宣称 T4 完整或数值口径变化；本片结论只到「可用性 + 缺口 + 裁决 B 落盘」。
