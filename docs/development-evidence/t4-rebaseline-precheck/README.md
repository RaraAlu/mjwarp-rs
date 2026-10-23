# T4 历史捕获重基线可行性预检

任务：`2026-10-23-t4-rebaseline-precheck`。对象：`2026-10-22-t4-historical-capture-rebaseline`（proposed，未 start）。
方式：只读复跑 3 个被 all-probes 排除的重采集/归档测试 + 盘点历史捕获与工具 + 复刻模板映射逻辑；**未改历史捕获/脚本/产品/测试/容差，未跑 G01 全脚本，未 start 主任务**。明细见 `check.json`。

## 结论（先说结果）

1. **3 个工具当前全部失败，且失败点各不相同**（见下）；用现有捕获与工具**不能就地重生成**当前源码对应的捕获。
2. 根因不是单点：G03/G26/U038 多次有意改动后，历史模板锚点只覆盖上一次 rotation-grouping 变更场景——**10 个旋转族变体锚点全部失配、2 个 retained 变体（`rigid_mass_matrix`、`rigid_mass_solve`）模板已变**；另有 12 个变体的 previous 捕获与 `fixtures/precision/t4-current-kernels.json` 不一致；face 归档产物已被重生成（bit_differences 0）。
3. 恢复 6 项历史 audit 需要先做「更新重采集工具/锚点 + 重生成 30×2 模式捕获 + 重写 fixtures 身份」的系统性工作（选项 A），不是一次跑通即可。
4. 旧历史产物都在 `target/`（未入库，合计约 22.8MB），可先整目录归档再重生成；已入库的身份（`fixtures/precision/t4-kernels.json`、`docs/evidence/t4-g03-retest`）可经 git 历史回溯。**归档可行。**

## 3 个测试的真实失败点

| 测试 | 命令（均需 `--ignored`） | 退出码 | 首个失败点 |
| --- | --- | --- | --- |
| `recompile_rigid_variants_without_launching_physics` | `cargo test --locked --features cuda-probe --lib runtime::transfer::kernel::recapture::tests_recapture::recompile_rigid_variants_without_launching_physics -- --ignored --exact --nocapture` | 101 | `tests_recapture.rs:50` `verify_sources` → `"source drift: src/physics/attached.rs"`；测试只更新 3 个源身份，当前实际漂移 **10** 个源 |
| `recaptures_only_missing_rotation_variants` | 同上 `...::recaptures_only_missing_rotation_variants` | 101 | `tests_recapture.rs:161` `verify_capture(previous)` → `"capture coverage mismatch"`；`target/t4-final-face-compile/current` 与 `t4-current-kernels.json` 有 **12 个变体（36 文件）** 不一致 |
| `archives_face_operator_repair_with_failed_numeric_gate` | 同上 `...::tests_archive::archives_face_operator_repair_with_failed_numeric_gate` | 101 | `tests_archive.rs:35` 断言 `repaired_product.bit_differences == 2078` → 实际 **0**；`target/t4-face-operators/factorial.json` 已于 10/22 由 `d2d7b467` 重生成（`full_acceptance=false`），"失败数值门"前提不再成立 |

环境前置（本次实测必须）：
- `MJWARP_PHYSICS_PTX_DIR` 必须指向**不存在的唯一输出路径且父目录已存在**（`capture_output` 语义；否则在 `.expect("new capture directory")` 或 `parent().canonicalize()` 失败）。
- NVRTC 需要 `MJWARP_CUDA_INCLUDE=target/toolchains/cuda-12.8.1/include` 且 `target/toolchains/cuda-12.8.1/bin` 在 `PATH`。

## 能否就地重生成（复刻 recipe 映射的结论）

以 `fixtures/precision/t4-kernels.json`（BASE，`4dc88934`，30 变体）为旧身份，用当前源码复刻 `tests/support/t4_rigid_capture/recipe.rs` 的映射逻辑：

| 类别 | 数量 | 当前源码能否对回 BASE 身份 |
| --- | --- | --- |
| 旋转族（rigid_kinematics×3、attached×3、initialize×2、camlight×2） | 10 | **0/10 匹配**——模板锚点需重建 |
| retained（其余 direct/组合模板） | 19 | **17/19 匹配**；`rigid_mass_matrix`、`rigid_mass_solve` 需重建锚点 |
| flex_faces（face lineage） | 1 | 归档 `product.cu` 已与当前 FACE 模板一致；lineage 复核未执行（需测试内运行） |

因此：
- 可复用：17 个 retained 变体的模板仍能对上；`flex_faces` 模板本身可重建。
- 必须先重建：10 个旋转族 + 2 个 G03 改动变体的**历史模板锚点**（`recipe.rs`/`lineage.rs` 的模板与期望数量、source 身份列表都要改）；`tests/support` 不在主任务 files 内，需扩范围或另立任务。
- 之后才能：重生成 debug-ptx/release-ptx 全 30 变体（compile-only）→ 重写 `fixtures/precision/t4-kernels.json` 与 `t4-current-kernels.json` 身份 → 重跑 6 项 audit → 移除脚本排除项。

## 旧产物归档与覆盖范围

- 历史捕获均在 `target/`（未入库），合计约 **22.8MB**：`t4-g03-retest` 247 文件/7.6MB、`t4-face-operators` 194/11.2MB、`t4-final-face-compile` 94/1.3MB、`t4-rigid-recapture` 189/2.7MB。
- 测试代码只读历史目录（`verify_capture`/`read_under`），写入目标为 `capture_output` 的新目录；本次复跑前后**文件数与最新 mtime 均未变化**，且未创建覆盖历史的写入（新目录在 `target/qa-t4-precheck/`）。**历史产物未被修改。**
- 重基线会**新增**新捕获目录并**改写**两个 `fixtures/precision/*-kernels.json`；`docs/evidence/t4-g03-retest` 快照与脚本排除清单按主任务范围更新。旧身份可先复制归档（建议 `target/archive/<date>/`）并经 git 历史回溯已入库部分。

## A/B/C 三种裁决的代价（按「历史捕获债务处置」口径）

| 选项 | 工作量 | 恢复 6 项 audit | 改写已入库身份 | 产物 | 主要风险 |
| --- | --- | --- | --- | --- | --- |
| **A 全量重基线**（重建工具/锚点 + 重生成 + 重写 fixtures） | **3–5 天**（含一次双模式回归；捕获编译分钟级，工具改动与验证为主） | 是 | 是 | `tests/support/t4_rigid_capture` 更新、`fixtures/precision` 两文件身份重写、新捕获目录 + 旧目录归档、`docs/evidence/t4-g03-retest` 快照、脚本排除清单缩项、three-layer 文档 | BASE 身份重写（T4 483 双模式与旧捕获绑定，需文档注明、只能 git 回退）；工具改动越出主任务范围；需固定工具链防新 hash 漂移；W-D5 工具自身要改，独立性需重新说明 |
| **B 维持排除**（登记长期债务） | **0.5 天**（文档） | 否 | 否 | three-layer 债务小节 + 证据登记（可复用本预检） | G01/T4 永久带 10 项排除、不能宣称完整；身份分叉随时间放大，未来重基线更贵 |
| **C 冻结历史身份对照**（不重采集，审计基准切到冻结源码身份） | **1–2 天** | 部分（只证明历史捕获自洽，不证明当前源码） | 否（新增冻结快照） | 冻结源码快照/锚点、6 audit 输入切换、脚本排除清单 + 文档 | 比较口径变化需用户批准；仍不提供当前源码身份证据；新增仓储体积与维护点 |

> 说明：A/B/C 的定义按「历史捕获债务处置」维度整理；若用户的 A/B/C 指其他口径，可按上表同字段替换评估。

## 证据文件

- `check.json`：3 次测试的命令/退出码/失败点、环境前置、历史目录盘点与完整性、身份漂移 10 源、模板映射逐变体结果、test2 差异、face 归档事实、A/B/C 明细。
- 本次复跑日志在 `target/qa-t4-precheck/`（target 未入库）。

## 复现

```powershell
$env:MJWARP_CUDA_INCLUDE = "D:\Rust\mjwarp-rs\target\toolchains\cuda-12.8.1\include"
$env:PATH = "D:\Rust\mjwarp-rs\target\toolchains\cuda-12.8.1\bin;$env:PATH"
$env:MJWARP_PHYSICS_PTX_DIR = "D:\Rust\mjwarp-rs\target\qa-t4-precheck\<fresh-name>"   # 父目录需存在、目标不存在
cargo test --locked --features cuda-probe --lib runtime::transfer::kernel::recapture::tests_recapture::recompile_rigid_variants_without_launching_physics -- --ignored --exact --nocapture
# 另两个测试把末尾过滤名换成 recaptures_only_missing_rotation_variants / ...tests_archive::archives_face_operator_repair_with_failed_numeric_gate
```

## 边界

- 只读预检：未改历史捕获、`scripts/`、`tests/`、`fixtures/`、产品代码；未 start `2026-10-22-t4-historical-capture-rebaseline`；未跑 G01 全脚本。
- 不宣称任何 audit 恢复或 T4 通过；A/B/C 代价为基于实测的估算，不是承诺工期。
- 未使用 GPU 内核（仅 NVRTC 编译与宿主检查），与 Backend 的 GPU 任务无冲突。
