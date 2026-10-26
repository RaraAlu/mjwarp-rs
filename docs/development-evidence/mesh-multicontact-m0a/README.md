# G10@P5 M0a：mesh multicontact sidecar ABI（496→504）+ host 上传链 + 空 sidecar 语义 + T4 实测

任务：`2026-10-26-mesh-multicontact-m0a-abi-repin`（in_progress，owner Backend；纯 CPU，无 GPU 窗口）。
**非目标**：不实现 mesh 多接触内核（M2）、不改碰撞语义/判据/容差/参考/期望、不改 SDF 路径
（`CollisionUploadPlan` 14 字段不动）、不宣称 mesh 已支持。

## 1. 改动文件与行数（全部 ≤500 行）

| 文件 | 行数 | 说明 |
| --- | --- | --- |
| `src/physics/collision/cuda/product.cu` | 426 | 新增 `MjwarpMeshPolyArgs`（9 指针）与 args 的 `mesh_poly_args` 字段；入口数不变 |
| `src/runtime/collision_gpu/abi.rs` | 337 | `CollisionArgs` 504 布局 + sidecar 结构 + 空/不自洽校验 `validate_mesh_poly` |
| `src/runtime/collision_gpu.rs` | 448 | 9 路 upload + sidecar 构造/指针（全空→null）；缓冲 helper 拆出 |
| `src/runtime/collision_gpu/buffers.rs` | 70 | 新增：`zeroed/upload/input_pointer/output_pointer/readback/truncated`（行为不变） |
| `src/runtime/collision_gpu/types.rs` | 115 | `CollisionGpuInputs` 23→32（9 个可选 slice） |
| `src/runtime/collision_gpu/tests.rs` | 235 | 保留接触输出布局/拼装/入口/错误 seam 测试 |
| `src/runtime/collision_gpu/abi_tests.rs` | 169 | 新增：504 逐字段偏移、sidecar 72B 偏移、6 入口 by-value 唯一声明 |
| `src/runtime/collision_gpu/validate_tests.rs` | 291 | 新增：meta/slice 校验 + 空 sidecar 三条单测 |
| `tests/g10_p5_nvrtc_launcher.rs` | +15 | 输入构造 macro 补 9 个 `&[]` |
| `tests/g10_p5_primitive_plane_cuda.rs` | 450 | 补 9 个 `&[]`；size-gate **test-only 拆分**后 <500 |
| `tests/support/primitive_plane_cases.rs` | 63 | 新增：plane 测试 `PlaneCase`/`TETRA`/`CASES`（`#[path]` 引入，数据一字未改） |
| `tests/g10_p5_primitive_{sphere_capsule,branches}_cuda.rs` | +10 each | 每个 `CollisionGpuInputs` 补 9 个 `&[]` |
| `tests/support/{primitive_qa_support,multicontact_qa_support,stage2_equivalence_cases}.rs` | +10 each | 同上；`INPUT_BINS` 保持 23 项 |

`DEVICE_SOURCES` 段数保持 27（本片不新增 `.cu`）、`src/physics/mod.rs` 零 diff、
`fixtures/precision/` 零 diff（见 §6）。

**size-gate 拆分（test-only）**：M0a 给每个 `CollisionGpuInputs` 补 9 个 `&[]` 后
`tests/g10_p5_primitive_plane_cuda.rs` 由 493 涨到 503 行、超 500 提交上限；按 B3 的
`#[path]` 做法把 case 表/数据抽到 `tests/support/primitive_plane_cases.rs`（63 行），主文件
回到 **450 行**。断言/判据/字段集/期望一字未改，只动 `tests/`；`src/`（含 device 源）零新增改动，
**ABI 与冒烟指纹不变：`238969 / ec2e7acd… / 3786520 / entries 6`**；默认套件仍 **980/0/372**。

## 2. ABI 布局（496→504，写死进 `abi_tests.rs`）

`CollisionArgs`：align 8、size **504**；无隐式填充（标量块止于 344，`out` 8 对齐）。

| 偏移 | 内容 |
| --- | --- |
| 0–176 | 既有 23 指针（顺序不变） |
| **184** | **`mesh_poly_args: *const MjwarpMeshPolyArgs`**（指针块末尾、i32 块之前） |
| 192–340 | i32/f32 标量块（`nworld`…`efc_width`，全部较 496 版 +8） |
| 344 | `out: CollisionContactOut`（136B） |
| 480 / 488 / 496 | `nacon` / `nccd` / `overflow` |
| **504** | 总大小（align 8） |

`MjwarpMeshPolyArgs`（sidecar）：align 8、size **72**，顺序与 pinned `collision_convex.py` L197-205 一致：
`0 mesh_polynum / 8 mesh_polyadr / 16 mesh_polynormal(f32*) / 24 mesh_polyvertadr /
32 mesh_polyvertnum / 40 mesh_polyvert / 48 mesh_polymapadr / 56 mesh_polymapnum / 64 mesh_polymap`。

设备侧 `product.cu`：`struct MjwarpMeshPolyArgs`（L25）+ `mjwarp_collision_args::mesh_poly_args`
（L63，位置与宿主 ABI 一致）。**0 新入口（entries 6→6）**；`all_six_entries_take_args_by_value`
逐入口断言 `extern "C" __global__ void <entry>(mjwarp_collision_args a)` 唯一声明。

## 3. host 上传链（逐环 file:line）

```text
[环0] 模型层字段定义/校验
  src/model/render.rs:473-481            mesh_polynum/polyadr/polynormal/polyvertadr/polyvertnum/
                                          polyvert/polymapadr/polymapnum/polymap 的 field 表
  src/io/render.rs:222 into_model        NativeRenderSnapshot → RenderModelInput
  src/io/render.rs:238 RenderModelInput::new
  src/io/render.rs:296 field(name)       → FieldData::{Index,Float}（按名读数组）
  src/io/render.rs:320-380 check_mesh_ranges   mesh_poly/polyvert/polymap 区间校验
        ↓  （环0→1 由调用侧负责；产品内无 model→碰撞输入接线，见 §8 已知债务）
[环1] src/runtime/collision_gpu/types.rs:48/65 CollisionGpuInputs 32 字段（9 个 mesh_poly* 可选 slice）
        ↓
[环2] src/runtime/collision_gpu.rs:148-157   9 路 upload()（空 slice → buffers.rs:24 的 1 元素零缓冲）
        ↓
[环3] src/runtime/collision_gpu.rs:187-209   sidecar 构造（9 设备指针）+ 上传 + 设备指针；
                                            全空 → mesh_poly_args = null
        ↓
[环4] src/runtime/collision_gpu/abi.rs:63/106  CollisionArgs.mesh_poly_args（504 布局）
        ↓
[环5] src/physics/collision/cuda/product.cu:25/63  mjwarp_collision_args::mesh_poly_args
        （M2 才消费；本片不实现 mesh 内核）
```

调用方兼容（G5 旧 case 只有 23 项 bin）：`tests/support/primitive_qa_support.rs`、
`tests/support/multicontact_qa_support.rs`、`tests/support/stage2_equivalence_cases.rs`、
`tests/g10_p5_nvrtc_launcher.rs`、`tests/g10_p5_primitive_{plane,sphere_capsule,branches}_cuda.rs`
均补 9 个 `&[]`；**`INPUT_BINS` 仍为 23 项**（未要求补齐 9 bin）。

## 4. 空 sidecar 语义（写死）

- 9 个 slice 全空（默认 `&[]`）→ sidecar 不构造、`mesh_poly_args = null`；
  语义 =「无 mesh polygon 数据、mesh 多接触功能不启用、行为与改动前完全一致」。
- **部分为空或长度不自洽 → `InvalidArgument`**（不静默禁用；`abi.rs validate_mesh_poly`）。
- 单测（`validate_tests.rs`，全部通过）：
  `empty_mesh_poly_sidecar_is_null_and_valid`（全空 + 非 mesh 组合 SPHERE-BOX，validate 通过、不 panic）、
  `rejects_partial_mesh_poly_sidecar`、`rejects_inconsistent_mesh_poly_sidecar`、
  `rejects_inconsistent_mesh_poly_sums`、`accepts_real_cube_mesh_poly_shapes`。
- 说明：M0a 为纯 CPU 片，「发射级」验证（GPU 上 null sidecar 不改行为）在 M2 验证窗执行。

> **已被修正片 `2026-10-26-mesh-multicontact-m0a-fix-mesh-poly-shapes` 更新（2026-10-26）**
>
> 本文档此前「9 个非空数组**逐 mesh 等长**」的表述是**错误的**，已作废。pinned
> `mujoco_warp/_src/types.py`（rev `71da24d9`）L1329-1337/L1864-1872 的真实形状是：
> `mesh_polynum`/`mesh_polyadr` = `(nmesh,)`；`mesh_polynormal` = `(nmeshpoly,3)`；
> `mesh_polyvertadr`/`mesh_polyvertnum` = `(nmeshpoly,)`；`mesh_polyvert` = `(nmeshpolyvert,)`；
> `mesh_polymapadr`/`mesh_polymapnum` = `(nmeshvert,)`；`mesh_polymap` = `(nmeshpolymap,)`。
> 修正后校验按逐 mesh / 逐 polygon / 逐 vertex 三段写死（规则审计表与红/绿对照见修正片证据
> `docs/development-evidence/mesh-multicontact-m0a-fix-mesh-poly-shapes/`）。
> 影响面仅 CPU 校验层 + 注释 + 单测：ABI 504 / sidecar 72B / entries 6 / 冒烟指纹
> `238969/ec2e7acd…/3786520`、`DEVICE_SOURCES` 27、device 源与 fixtures **零改动**。
> 修正片后行数（`Measure-Object -Line`）：`abi.rs` 378 / `types.rs` 94 / `validate_tests.rs` 374
> （§1 表为 M0a 提交时历史值）。

## 5. CPU NVRTC 冒烟（窗口外，不建 context）

```text
$env:PATH="D:\Rust\mjwarp-rs\target\toolchains\cuda-12.8.1\bin;$env:PATH"
cargo test --locked --features cuda-probe --lib runtime::collision_gpu::nvrtc_smoke -- --include-ignored --nocapture
```

```text
nvrtc_smoke_ok ptx_bytes=3786520 entries=6 source_bytes=238969 source_sha256=ec2e7acddc71eec2b8cd6b07cf70ba4a4a3e7eab258982815f2199349e1a047a
```

- **entries 6→6（0 新入口）**；拼装源 237922（B3 收口态 `92ee236e…`）→ 238969（product.cu 加
  sidecar 结构/字段 + 探针同步注释，预期变化；中间态 `238728/5ab32eaa…` 已被本次注释更新取代，
  不在证据中作为最终值）；`make_frame`/27 段顺序断言不变。
- 冒烟日志：`logs/post-smoke-final.log`。

### 5.1 探针 `ProbeArgs` / `DeviceArrays` 同步口径

- 冻结 G5 探针 `ProbeArgs`（496 字节、23 指针）与旧 G5 证据**零改动**；产品侧
  `mjwarp_collision_args` 是其**超集**：前 23 指针 + i32/f32 标量块顺序/类型一致，只在指针块
  末尾插入 `mesh_poly_args`（504 字节），因此探针 `out1` 参考语义对既有 23 项输入仍成立。
- 测试侧 `DeviceArrays` 语义 = 23 项输入：`INPUT_BINS` 仍 23 项，9 个 mesh_poly 数组以空
  slice 传入（sidecar 不构造）；见 §5.1。
- 现有探针一致性测试（默认 gate 内、非 ignored）通过：`g10_p5_nvrtc_launcher::
  world_order_normalization_turns_row_swap_into_zero_diff`、
  `g10_p5_sdf_host::sdf_cpu_reference_is_well_formed_and_matches_g0b_structure`（含在
  §7 unit 980/0/372 内）；GPU 型 `*_matches_*` 测试需窗口，M2 验证窗执行。

### 5.2 旧 case 兼容（不重生成）

- `docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases/` 的 12 个已 verified case
  保持 23 项输入；`INPUT_BINS` 仍为 23 项（`tests/support/primitive_qa_support.rs:18`、
  `tests/support/multicontact_qa_support.rs:19`），9 个 mesh_poly 数组以 `&[]` 传入
  （sidecar 不构造）；**未改 `prepare_cases.py`、未动 G5 证据/旧参考**（`git status` 对这些
  路径零改动）。
- 旧窗口结果与旧二进制 sha 的绑定：B3 收口的 5 个签名二进制 sha（branches `25410b3d…` /
  plane `115b5c99…` / B2 `280f2e47…` / primitive `ec56ff74…` / launcher `84ee6074…`）对应
  496 版 ABI 源码；本片 ABI **496→504** 后，旧窗口结果与旧 sha 只作历史记录，
  **M2 验证窗必须全量位级重验 G5 12 case**，不沿用旧结论。

## 6. T4 repin：前置枚举 + 改前/改后实测（0 漂移，未改任何身份字面量）

前置枚举（事实核对）：`fixtures/precision/t4-current-kernels.json::product_sources` 20 条 =
`fixtures/mjwarp-mass-solve/manifest.json` + 17 个 `src/physics/*.rs` + `src/runtime/transfer.rs` +
`src/runtime/transfer/kernel.rs`；**不含** `src/runtime/collision_gpu/*`、`product.cu`、`tests/`。
M0a 改动集合与身份集交集为空 → 预计 0 漂移；不改 `EXPECTED_PATHS`、不新增 collision 文件进身份面。

| 命令（原文） | 改前 | 改后 |
| --- | --- | --- |
| `cargo test --locked --lib runtime::transfer::kernel::recapture -- --nocapture` | 2 passed / 0 failed / 3 ignored，exit 0 | 2 passed / 0 failed / 3 ignored，exit 0 |
| `cargo test --locked --test t4_identity_live_check` | 2 passed / 0 failed，exit 0 | 2 passed / 0 failed，exit 0 |

结论：**漂移 0 → `fixtures/precision/t4-current-kernels.json` 等身份文件零改动**。
原始日志：`logs/pre-*.log`、`logs/post-*.log`。

## 7. 门禁与验证数字

| 项 | 命令 / 版本 | 结果 |
| --- | --- | --- |
| fmt | `cargo fmt --all -- --check` | exit 0 |
| clippy | `cargo clippy --locked --all-targets -- -D warnings` | exit 0 |
| clippy-cuda | `cargo clippy --locked --all-targets --features cuda-probe -- -D warnings` | exit 0 |
| unit（gate） | `cargo test --locked` | **980 passed / 0 failed / 372 ignored**，exit 0（基线 975/0/372：+3 mesh sidecar 校验单测 + 3 ABI/sidecar/入口单测 − 1 旧 ABI 测试移入新模块） |
| no-default | `cargo test --locked --no-default-features --lib` | 455 passed / 0 failed / 5 ignored，exit 0 |
| collision_gpu 子集 | `cargo test --locked --features cuda-probe --lib runtime::collision_gpu -- --include-ignored` | 31 passed / 0 failed（含 NVRTC 冒烟） |
| all-features | 见 §7.1 | **1029 passed / 0 failed / 430 ignored**，exit 0 |

工具链：`rustc 1.99.0 (b940084d7 2026-09-28)`、`cargo 1.99.0 (5f94df478 2026-08-27)`
（gnu 与 msvc 两个工具链同版本）。

### 7.1 all-features 口径（环境前置，写死）

```powershell
$env:MJWARP_MUJOCO_ROOT = (Resolve-Path 'target/toolchains/mujoco-3.12.0/package').Path
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --no-fail-fast
```

- 结果：**pass=1029 / fail=0 / ignored=430，exit 0**；摘要 `logs/all-features-summary.txt`；
  原始日志留在 `target/m0a/all-features-raw.log`（sha256
  `b6ce8def369c2d11c645aeb1df406a9b5d7e97216efc653b59f848bc50cf13b8`，因证据文件 ≤500 行约束
  不入证据目录，摘要由该日志解析）。
- 裸跑（gnu host、无 env）会失败，已留证 `logs/all-features-bare-fail.log`：实测失败点是在依赖编译阶段——
  `dlltool.exe` 缺失 → `getrandom` / `windows-sys` 编译失败（**未进入 build.rs 阶段**）；
  「需 `MJWARP_MUJOCO_ROOT` + msvc 工具链」的结论仍成立，但此前「env 未设 → build.rs panic」的表述无原文证据，已按实测更正（QA M0a 观察 O1）。
  故验收 item 6 采用上面这条命令（与 `61fb6baf` 重钉流程同一口径）。


### 7.2 size-gate 拆分的边界（QA M0a 观察 O6）

- 提交前 `tests/g10_p5_primitive_plane_cuda.rs` 因新增 9 个 `&[]` 从 493 增至 503 行，触发 500 行棘轮；
  已做 **tests-only 拆分**：`PlaneCase`/`TETRA`/`CASES` 抽到 `tests/support/primitive_plane_cases.rs`（63 行），
  主文件回到 **450 行**，断言/字段集/判据/期望逐字未改。
- 该拆分为 `tests/` 内改动，不影响 device 源与 ABI：提交后树实测冒烟仍 `238969 / ec2e7acd… / 3786520 / entries 6`、
  默认套件 980/0/372——即「ABI/冒烟数字不变」是**提交后树实测确认**；
  单 commit 内无法独立还原「拆分前/拆分后」的先后，此点如实记录。

## 8. 已知债务 / 未覆盖（不得误读）

- **环0→1 映射是调用侧职责**：产品 API 为 slice 入参（`CollisionGpuInputs`），产品内当前
  **没有** model→碰撞输入的接线；现有 model→inputs 映射只在 `tests/support/*`。**本片不扩
  scope、不新增 adapter**（`src/io/render.rs`/`src/model/render.rs` 未改动）。架构片
  `2026-10-26-product-collision-inputs-wiring` 已由 Arch 登记为 backlog（依赖 M3），不 start。
  **不能据此认为产品管线已能端到端喂 32 数组。**
- mesh 多接触内核未实现（M2）；GPU 上 null/非 null sidecar 的发射级行为在 M2 验证窗验证。
- `--all-features` 需要 §7.1 的环境前置；M2 验证窗必须全量位级重验 G5 12 case（旧窗口
  产物与旧二进制 sha 绑定因 ABI 496→504 失效，只作历史记录；本片未改 `prepare_cases.py`、
  未动 G5 证据）。
- `capsule-capsule` 等既有语义未变（本片不改任何碰撞数值路径）。

## 9. 红线声明

- 0 个新入口（entries 6→6）；不改 ABI 既有字段顺序/类型（只在指针块末尾插入 sidecar 指针）。
- 不改 SDF 路径（`CollisionUploadPlan` 保持 14 字段）；不改 `DEVICE_SOURCES` 段数（27）；
  `src/physics/mod.rs` 零 diff；判据/容差/期望/参考零改动；不宣称 mesh 已支持。

## 10. 复现命令

```powershell
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --all-targets --features cuda-probe -- -D warnings
cargo test --locked
cargo test --locked --no-default-features --lib
cargo test --locked --features cuda-probe --lib runtime::collision_gpu -- --include-ignored
cargo test --locked --lib runtime::transfer::kernel::recapture -- --nocapture
cargo test --locked --test t4_identity_live_check
# all-features 见 §7.1
```
