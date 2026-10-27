# 产品碰撞输入接线 S1：独立 data 快照探针（宿主侧，无 GPU）

- 任务：`2026-10-27-product-collision-inputs-s1-data-snapshot`（owner Backend；HEAD 起点 `1f478513`）
- 父任务（伞，保持 backlog）：`2026-10-26-product-collision-inputs-wiring`
- 上游 pinned revision：`71da24d956378a87a703b6e1442b13aec0c4ac29`
- 本片范围：**只新增**独立 collision-inputs 探针与 Rust 侧拥有式快照；不做产品接线（S3 才接）。
- 结果一句话：新探针给出 `CollisionGpuInputs` 32 项字段的模型快照与 mjData `geom_xpos/geom_xmat` 帧快照；
  4 项 host 定向测试全过（32 项逐元素/oracle、2 组 qpos 帧位型、octree 与体积桥交叉、原始 ABI 负例）；
  fmt/clippy/clippy-cuda 通过、unit **994 过 / 0 败 / 376 ignored**；**18 个 pinned 源与 HEAD 逐字节一致（零改动）**。

## 1. 契约边界（硬条件，逐条）

1. **只新增文件**：新增 `include/mjwarp_collision_inputs_probe.h`、`native/model_probe_collision_inputs.cpp`、
   `src/model/collision_inputs.rs`、`src/io/collision_inputs.rs`、`tests/g10_p5_collision_inputs_snapshot.rs`
   与 `tests/support/collision_inputs_*.rs`；只改 `native/model_probe_sparse.cpp`（+1 行 include）与
   `build.rs`（只加 `rerun-if-changed`）。
2. **禁改 18 个 pinned 源**（`fixtures/precision/g01-inputs.json` 清单）：`git status` 与本片改动集不含其中任何
   一个；任务 machine check `unchanged` 会逐字节确认。
3. **既有 ABI 不动**：`include/mjwarp_native_probe.h` 的 `mjwarp_native_*` 声明、`mjwarp_collision_args`
   （504 B，`src/runtime/collision_gpu/abi.rs` 的静态布局注释与 `abi_tests`）、`DEVICE_SOURCES`=28
   （`multicontact_tests::device_sources_place_multicontact_and_boxbox_in_order`）、产品入口 6
   （`abi_tests`「entries 6→6，0 新增入口」）全部未触碰。本片新 ABI 使用**独立 header 与独立符号名**。
4. **不做新 feature**：沿用既有 `native-model-probe` feature；不新增依赖。
5. **无 GPU / 不开窗**：本片全部验证为宿主侧（可信 DLL + 冻结 MJB），不建 CUDA 上下文、不使用 GPU。
6. **状态输入只有 qpos**：`geom_xpos`/`geom_xmat` 的权威来源 = 调用方传 qpos 后执行一次 `mj_kinematics` 的
   mjData；mocap / act / ctrl **不在默认支持面**（mocap 模型另立任务评审 state 读取）。
7. 本片**不做产品接线**，不宣称 host↔device 等价、不宣称精度或验收结论；不写「与 forward 全流程等价」。

## 2. 原生 ABI（`include/mjwarp_collision_inputs_probe.h`）

独立 schema；字段号与 `src/runtime/collision_gpu/types.rs::CollisionGpuInputs` 的 32 项 1:1。
本 schema 的 kind 编号：**1 = int32，2 = double(mjtNum)，4 = float32**。

| 结构 | 大小/对齐 | 内容 |
| --- | --- | --- |
| `mjwarp_collision_inputs_field_info` | 16 B / 8 | `kind`、`reserved`、`count`（标量数） |
| `mjwarp_collision_inputs_info` | 96 B / 8 | `schema`、`reserved` + 11 个计数：`nq ngeom nmesh nmeshvert nmeshgraph nmeshpoly nmeshpolyvert nmeshpolymap nhfield nhfielddata noct` |
| `mjwarp_collision_inputs_data` | 不透明句柄 | `owner`、`mjData*`、`mj_kinematics`/`mj_deleteData` 符号 |

入口（错误码沿用现有约定：**0 = ok，8 = invalid**）：

- `mjwarp_native_collision_inputs_info(owner, info)`：写出 11 个非负计数。
- `mjwarp_native_collision_inputs_field_info(owner, id, info)`：32 个 ID 的 kind/count 描述（不分配）。
- `mjwarp_native_copy_collision_inputs_field(owner, id, info, target)`：复制**模型侧**字段（ID 2..31）；
  ID 0/1 是 mjData 字段，本入口返回 8，由 data 快照服务。所有 kind/count/来源/目标先 preflight，再写第一个字节。
- `mjwarp_native_collision_inputs_create(owner, out)`：`library_symbol` 从 owner 的 DLL 取
  `mj_makeData/mj_deleteData/mj_kinematics`，创建**一次** mjData 并缓存（只重跑运动学，不重复 makeData）。
- `mjwarp_native_collision_inputs_set_qpos(owner, data, qpos, nq)`：长度必须等于模型 `nq`；写 `d->qpos` 后
  执行 `mj_kinematics`。
- `mjwarp_native_collision_inputs_copy_xpos_xmat(owner, data, xpos, xmat, ngeom)`：把 `d->geom_xpos`（ngeom×3）
  与 `d->geom_xmat`（ngeom×9）转成 float32 写出；两个目标缓冲先 preflight。
- `mjwarp_native_collision_inputs_destroy(owner, data)`：防御性校验 `data->owner == owner`（外部句柄返回 8
  且不删除）；**必须在 owner close 之前 destroy**（Rust 侧用借用类型强制该顺序）。

32 项字段表（kind：1 int32 / 2 double / 4 float32；unit 即计数基数，count = unit × width）：

| id | 字段 | kind | unit | width | 来源 |
| --- | --- | --- | --- | --- | --- |
| 0 | `geom_xpos` | 4 | ngeom | 3 | mjData |
| 1 | `geom_xmat` | 4 | ngeom | 9 | mjData |
| 2 | `geom_size` | 2 | ngeom | 3 | mjModel |
| 3 | `geom_margin` | 2 | ngeom | 1 | mjModel |
| 4 | `geom_gap` | 2 | ngeom | 1 | mjModel |
| 5 | `geom_rbound` | 2 | ngeom | 1 | mjModel |
| 6 | `geom_aabb` | 2 | ngeom | 6 | mjModel |
| 7 | `geom_type` | 1 | ngeom | 1 | mjModel |
| 8 | `geom_dataid` | 1 | ngeom | 1 | mjModel |
| 9 | `mesh_vertadr` | 1 | nmesh | 1 | mjModel |
| 10 | `mesh_vertnum` | 1 | nmesh | 1 | mjModel |
| 11 | `mesh_graphadr` | 1 | nmesh | 1 | mjModel |
| 12 | `mesh_octadr` | 1 | nmesh | 1 | mjModel |
| 13 | `mesh_vert` | 4 | nmeshvert | 3 | mjModel |
| 14 | `mesh_graph` | 1 | nmeshgraph | 1 | mjModel |
| 15 | `mesh_polynum` | 1 | nmesh | 1 | mjModel |
| 16 | `mesh_polyadr` | 1 | nmesh | 1 | mjModel |
| 17 | `mesh_polynormal` | 2 | nmeshpoly | 3 | mjModel |
| 18 | `mesh_polyvertadr` | 1 | nmeshpoly | 1 | mjModel |
| 19 | `mesh_polyvertnum` | 1 | nmeshpoly | 1 | mjModel |
| 20 | `mesh_polyvert` | 1 | nmeshpolyvert | 1 | mjModel |
| 21 | `mesh_polymapadr` | 1 | nmeshvert | 1 | mjModel |
| 22 | `mesh_polymapnum` | 1 | nmeshvert | 1 | mjModel |
| 23 | `mesh_polymap` | 1 | nmeshpolymap | 1 | mjModel |
| 24 | `hfield_size` | 2 | nhfield | 4 | mjModel |
| 25 | `hfield_nrow` | 1 | nhfield | 1 | mjModel |
| 26 | `hfield_ncol` | 1 | nhfield | 1 | mjModel |
| 27 | `hfield_adr` | 1 | nhfield | 1 | mjModel |
| 28 | `hfield_data` | 4 | nhfielddata | 1 | mjModel |
| 29 | `oct_aabb` | 2 | noct | 6 | mjModel |
| 30 | `oct_child` | 1 | noct | 8 | mjModel |
| 31 | `oct_coeff` | 2 | noct | 8 | mjModel |

注：`mjModel` 3.12 没有 `nmeshpolycount` 字段；本 schema 用真实存在的计数覆盖全部 unit 需求
（`nmeshgraph` 与 `nmeshpolymap` 已显式列出，分别服务 `mesh_graph` 与 `mesh_polymap`）。

## 3. Rust 侧

- `src/model/collision_inputs.rs`：`CollisionInputsFieldKind`（Index/Num/Float，code 1/2/4）、
  `CollisionInputsUnit`、`CollisionInputsFieldSource`、`CollisionInputsFieldSpec`、32 项
  `COLLISION_INPUTS_FIELDS`、`CollisionInputsCounts::validate/expected`（负计数与溢出先拒绝）。
- `src/io/collision_inputs.rs`：
  - `NativeCollisionInputsSnapshot::capture`：查 info → 校验 11 计数 → 逐 ID 查 field info、校验
    kind/count/来源 → 拥有式 `Vec` 快照（30 项模型字段：`Index/Num/Float`）。
  - `into_inputs()`：`Num` 走**既有唯一转换入口** `io::convert_f64_to_f32_into`（拒绝非有限/f32 溢出，
    错误映射到字段名），`Float` 原样保留，`Index` 原样；产出宿主侧 `CollisionInputsModelInput`。
  - `CollisionInputsData<'a>`：借用 `NativeModelProbe` 的 mjData 句柄（RAII，Drop 调用 destroy），
    `set_qpos` 校验长度，`capture_frame()` 得到 `geom_xpos`/`geom_xmat` 的 f32 快照（ngeom×3 / ngeom×9）。
  - `io::NativeModelProbe` 新增 `collision_inputs_snapshot()` / `collision_inputs_data()` 两个方法。
- 新 cpp 从 `native/model_probe_sparse.cpp` include（放在 `model_probe_collision.cpp` 之后，同 TU 复用
  owner/library_symbol）；**新 cpp 不再次 include `model_probe.cpp`**（避免同 TU 重复定义）。

## 4. 数据源与 oracle

三份冻结 MJB（只读，未新造；均为仓库既有 fixture）覆盖 mesh / hfield / octree：

| oracle 名称 | MJB | bytes | sha256 | 关键计数 |
| --- | --- | --- | --- | --- |
| hfield-box | `fixtures/g10-p5/mjb-312/hfield-box.mjb` | 5267 | `39e74291327e71cfe9c9ab42f7f15f781a2cf26b6050fe498146d2a0f1985082` | nq 7, ngeom 2, nhfield 1, nhfielddata 9 |
| scene-312 | `fixtures/render-bvh/scene-a/scene-312.mjb` | 101328 | `c6f5f9f17118067ad39fb2ea6be4f1c274e4d932b553bbe9a4e59b464c56aad1` | nq 8, ngeom 7, nmesh 1, nmeshpoly 4 |
| sdf-cylinder | `fixtures/g10-p5/mjb-312/sdf-cylinder.mjb` | 9405930 | `9dfd077ed4ab40877ea9380ae57e317200da70c1645c603d22b5896d5aefec43` | nq 7, ngeom 2, nmesh 1, noct 63497 |

- 生成脚本（authoring-only，入库）：`scripts/generate_collision_inputs_oracle.py`，环境为 pinned
  Python 3.12.13 + **mujoco 3.12.0（python wheel）** + numpy 2.5.4；产出
  `tests/support/collision_inputs_oracle.json`（12545 B / sha256 `b5a3fd29…`）。
- oracle 内容：11 计数、30 项模型字段的原生值、2 组 qpos 的 `geom_xpos`/`geom_xmat` f32 位型；
  sdf 模型的 oct_* 因体量过大只存计数，逐元素改由**独立 G01 体积桥**（同一 pinned 模型的另一条读取路径）
  交叉核对。
- **f64 期望值以 u64 位型存储**（`f64_bits`），测试用 `f64::from_bits` 还原：本仓库 serde_json 未启用
  `float_roundtrip`，浮点文本解析可能丢 1 ULP（实测 scene-312 `geom_size[10]` 差 1 ULP），位型存储消除该误差。
- f32 期望以 `f32_bits` 存储（numpy `astype(float32)` 同舍入），测试按 `to_bits()` 逐元素比较。

## 5. 验证（全部宿主侧，无 GPU）

| 项 | 命令 | 结果 |
| --- | --- | --- |
| 定向测试 | `cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features native-model-probe --test g10_p5_collision_inputs_snapshot -- --ignored --test-threads=1` | **4 passed / 0 failed**：`snapshot_matches_pinned_mjmodel_oracle_for_all_32_fields`、`data_snapshot_matches_pinned_mjdata_oracle_and_reuses_mjdata`、`snapshot_outlives_owner_and_octree_matches_volume_bridge`、`rejects_invalid_native_arguments_without_partial_writes` |
| oracle 一致性 | 同上 | 32 项计数与模型字段逐元素一致；宿主收窄后 f32 位型与 numpy astype 一致（0 差异）；2 组 qpos 的 `geom_xpos/geom_xmat` 位型一致；mjData 复用后同一 qpos 复现同一帧 |
| octree 交叉 | 同上 | sdf-cylinder：`noct=63497`，`oct_aabb/oct_child/oct_coeff` 与 G01 体积桥逐元素相等 |
| 既有 native_g01 期望 | `cargo ... test --locked --features native-model-probe --test native_g01 -- --ignored --exact converts_complete_native_g01_fields_after_owner_drop` | **1 passed / 0 failed**（78 项 G01 字段对冻结参考不变） |
| fmt / clippy | `cargo fmt --all -- --check`；`cargo clippy --locked --all-targets --features native-model-probe -- -D warnings` | exit 0 / exit 0 |
| C++ `/W4 /WX` | build.rs `warnings_into_errors(true)`，cc 1.6 对 MSVC 映射 `/W4` + `/WX` | 新 TU 编译通过、**零警告**（`logs/build-w4.log`） |
| 门禁（最终源） | `normify gate 2026-10-27-product-collision-inputs-s1-data-snapshot` | exit 0：fmt 通过、clippy 通过、clippy-cuda 通过、**unit 994 过 / 0 败 / 376 ignored**（与本片前基线一致，新测试在 `native-model-probe` 下不进默认 unit） |

定向测试日志：`logs/targeted-test.log`；native_g01 日志：`logs/native-g01.log`；fmt/clippy：`logs/fmt-clippy.log`；
oracle 生成：`logs/oracle-generate.log`；门禁：`logs/gate.log`（run 目录
`.normify/runs/2026-10-27-product-collision-inputs-s1-data-snapshot/`）。

## 6. 边界与不宣称项

- 本片**不做产品接线**（S3 才接）；不宣称 host↔device 等价、不宣称精度或验收结论、不宣称与上游等价。
- `geom_xpos/geom_xmat` 权威来源仅指「调用方 qpos → 一次 `mj_kinematics` 后的 mjData」；**不代表 forward
  全流程等价**（未跑 position/velocity/constraint 等阶段）。
- 状态输入只有 qpos；mocap/act/ctrl 不在默认支持面。
- `sdf-cylinder` 的 oct_* 只与同 pinned 模型的 G01 体积桥交叉核对（不是第二份独立实现的外部参考）；
  oct_* 的 f32 收窄位型在定向测试中只做有限/非空校验。
- **pinned sha 与工作树行尾差异说明（QA S1 O1 更正）**：`fixtures/precision/g01-inputs.json` 的 pinned 值按
  **LF 归一化**记录，而本机 `core.autocrlf=true` 使工作树为 CRLF，故 raw byte sha 不同：
  `include/mjwarp_native_probe.h` raw `2068caf1…` / **LF `03151bf5…` = pinned**；
  `native/model_probe.cpp` raw `f60047c0…` / **LF `6a01eb01…` = pinned**。
  **不存在身份漂移**：两文件 blob 在本片前后一致、内容未被本片或既有 render/volume 片改动；
  T4 审计本身按 `replace("\r\n","\n")` 取 hash（`tests/support/t4_evidence/sources.rs:13`）。
  `cargo test --test t4_input_precision -- --include-ignored` 的 12 个失败原因是缺
  `MJWARP_FROZEN_UPSTREAM_ROOT` / `MJWARP_FROZEN_WARP_ROOT`（panic「set upstream root」），不是 hash 差异。
  本片对这 18 个路径零改动，machine check `unchanged` 以 HEAD 为准。
- 本 README/代码均 ≤500 行；`receipt.json` 为机器可读证据（可超 500 行）。

## 7. 交付文件

| 文件 | 行/字节 | 说明 |
| --- | --- | --- |
| `include/mjwarp_collision_inputs_probe.h` | 69 行 | 独立 schema + data 快照入口 |
| `native/model_probe_collision_inputs.cpp` | 236 行 | 模型字段复制 + mjData 句柄（同 TU，复用 owner/library_symbol） |
| `native/model_probe_sparse.cpp` | +1 行 | 只加 include |
| `build.rs` | +2 行 | 只加两个 `rerun-if-changed`（新 cpp / 新 header），不加 `.file()`/feature |
| `src/model/collision_inputs.rs` | 211 行 | 32 项字段规范 + 11 计数 |
| `src/io/collision_inputs.rs` | 474 行 | 拥有式快照 + mjData 句柄 + 唯一 f64→f32 入口 |
| `src/model/mod.rs` / `src/io/mod.rs` | +6 / +32 行 | 模块声明与 probe 方法 |
| `tests/g10_p5_collision_inputs_snapshot.rs` | 234 行 | 4 项 host 定向测试（≤500） |
| `tests/support/collision_inputs_oracle_support.rs` | 105 行 | oracle 解析/位型工具 |
| `tests/support/collision_inputs_preflight.rs` | 313 行 | 原始 ABI 负例预检 |
| `tests/support/collision_inputs_oracle.json` | 12545 B | pinned oracle（机器可读） |
| `scripts/generate_collision_inputs_oracle.py` | 159 行 | authoring-only oracle 生成脚本 |

## 8. S2：快照 → `CollisionGpuInputs` adapter + M0b 32 bins CPU 对账

- 任务：`2026-10-27-product-collision-inputs-s2-adapter`（owner Backend，HEAD 起点 `6a113daf`）；无 GPU、未开窗；
  **不做产品入口接线**（S3 才接）、不改判据/参考/容差/设备源/ABI。
- 结果一句话：S1 的 32 项宿主快照经新 adapter 组装为 `CollisionGpuInputs` 并复用既有
  `validate_collision_inputs`；对三份 case 的 32 数组做到**逐字节 + sha256 相等（0 不符）**；
  旧 23 项输入 → 9 项空 slice、sidecar null、validate 通过；部分为空/长度/meta 非法 → `InvalidArgument`。

### 8.1 adapter（`src/runtime/collision_gpu/inputs_from_snapshot.rs`，215 行）

- `CollisionInputsSnapshot<'a>`：32 项**借用 slice** 视图，字段顺序/宽度与
  `src/runtime/collision_gpu/types.rs::CollisionGpuInputs` 1:1（`geom_xpos/geom_xmat` 为单 world 的
  ngeom×3 / ngeom×9 帧快照）。
- `inputs_from_snapshot(&view, meta) -> Result<CollisionGpuInputs<'_>, CollisionGpuError>`：组装 32 项 +
  调用方给的 `CollisionGpuMeta`，然后调用**既有** `validate_collision_inputs`（与
  `CollisionGpuProgram::run` 首步校验同一实现）；不 panic。
- `CollisionInputsSnapshot::from_model_input(&CollisionInputsModelInput, &CollisionInputsFrame)`
  （`native-model-probe` feature 下）：把 S1 的拥有式模型快照 + mjData 帧快照按字段号映射为视图；
  先按 `COLLISION_INPUTS_FIELDS` 校验 30 项类型/长度（与 `CollisionInputsCounts` 口径一致）与帧长度，
  失败返回 `InvalidArgument`。
- `mesh_poly_sidecar_present(&inputs)`：与 `run` 内构造 sidecar 前的 `!inputs.mesh_polynum.is_empty()`
  条件一致（**`run` 未改**），供 host 测试断言「旧 23 项 → null」。
- `src/runtime/collision_gpu.rs` 只加 `mod inputs_from_snapshot;` 与 `pub use`（+4 行），既有入口/ABI 不动。

### 8.2 CPU 对账（写死口径，实际结果 0 不符）

- case 与路径（只读）：
  - `mesh-box` → `docs/development-evidence/mesh-multicontact-m0b/cases/mesh-box/`
  - `mesh-cylinder` → `docs/development-evidence/mesh-multicontact-m0b/cases/mesh-cylinder/`
  - `mesh-mesh`（fixture = `fixtures/g10-p5-mesh-multicontact/mesh-mesh.xml`，`xml_sha256=bb9ffb2f…`、
    qpos z=0.18、两份 cube：cube + cube_b 缩放 0.9；bins 实测 nmesh=2 / nmeshvert=16 / nmeshpoly=12）→
    `docs/development-evidence/mesh-multicontact-m3/cases/mesh-mesh/`
  - **路径更正（QA S2 O1）**：M0b `cases/` 只落了 `mesh-box` / `mesh-cylinder` 两个 case；第三份 case 的
    32 数组 bins 由 M3 捕获窗产出。**注意 fixture 归属**：本 case 用 `mesh-mesh.xml`（`bb9ffb2f…`），
    **不是** `mesh-mesh-two-assets.xml`（`1d7115…`，cube+tetra，为 S3/S4 U6 的 fixture）；此前笔误已更正。
    本片只读该目录、未改动（`unchanged` 检查只覆盖 M0b 目录，M3 case 只读）。
- 对账方式（测试 `tests/g10_p5_collision_inputs_adapter.rs` + support）：
  按字段名读 32 个 `.bin` 与 `meta.txt`，按字段 id 建 `CollisionInputsSnapshot`，经 adapter 组装后
  逐数组断言 **LE 字节相等** 且 **sha256 相等**；同时断言每项长度 ==
  `CollisionInputsCounts::expected(spec) × 4`（host 侧 f32/i32 均 4 字节标量），并断言
  `validate_collision_inputs` Ok、`mesh_poly_sidecar_present` 为 true。
- 结果：**3 case × 32 数组 = 96/96 项字节与 sha 全部相等，0 不符**；`meta.nworld == 1`、
  `meta.ngeom == counts.ngeom` 一致；三 case 均 validate 通过。
- 旧 23 项输入：以 `mesh-box` 的 23 项基础数组（去掉 9 项 mesh_poly）构造视图 →
  9 项空 slice、`mesh_poly_sidecar_present == false`（`run` 侧即 `mesh_poly_args = null`）、
  validate Ok、其余 23 项原样透传。
- 负例（均返回 `InvalidArgument`、不 panic）：
  - 只给 9 项中的 1 项 → 「mesh_poly 输入不完整…」；
  - `geom_xpos` 空（与 `meta.ngeom` 不一致）→ 「geom_xpos 长度须为 nworld*ngeom*3」；
  - `geom_type` 少 1 项 → 「geom_type 长度须为 ngeom」；
  - `meta.nworld`/`naconmax`/`efc_width` 置 0 → 对应长度/容量错误。
- `from_model_input` 桥接测试（`native-model-probe` + pinned DLL，ignored）：1 passed——S1 拥有式快照
  映射出的 32 项与视图逐字节一致（含 geom 帧）；测试内 meta 为手工构造（仅 `ngeom`/`noct` 取自 bins 计数，
  其余字段按单 world 场景给定），通过 validate（QA S2 O4 更正：原「由 30 项字段/帧导出」表述偏强）。

### 8.3 门禁与边界

- `normify gate` exit 0：fmt 通过、clippy 通过、clippy-cuda 通过、unit **997 过 / 0 败 / 376 ignored**
  （+3 项本片 host 测试；基线 994/0/376）。clippy 在默认与 `--features native-model-probe` 两套均零警告。
- 不改 18 个 pinned 源、`src/runtime/collision_gpu/abi.rs`、`src/physics/collision/cuda/`、
  `fixtures/g10-p5-mesh-multicontact/`、`fixtures/g10-p5-primitive-branches/`、M0b 证据（machine check
  `unchanged`）；`CollisionArgs` 504 B 与既有 ABI 未动；无新 feature、无 GPU、未开窗。
- 本片不做产品接线；不宣称 host↔device 等价、不宣称精度/上游等价。

### 8.4 S2 交付文件

| 文件 | 行/字节 | 说明 |
| --- | --- | --- |
| `src/runtime/collision_gpu/inputs_from_snapshot.rs` | 215 行 / 9657 B | adapter 视图、组装+校验、S1 桥接、sidecar 判据 |
| `src/runtime/collision_gpu.rs` | 462 行（+4） | `mod` 声明 + `pub use`，既有入口/ABI 不动 |
| `tests/g10_p5_collision_inputs_adapter.rs` | 233 行 / 8687 B | 32 bins 逐数组对账 + 旧 23 项 + 负例（≤500） |
| `tests/support/collision_inputs_adapter_support.rs` | 319 行 / 11596 B | bins/meta 读取、字段 id 视图、字节/摘要工具 |
| `logs/s2-targeted-default.log`、`logs/s2-targeted-native.log` | — | 定向测试原始输出（UTF-8 无 BOM） |
| `logs/s2-fmt-clippy.log`、`logs/s2-gate.log` | — | 本地检查与门禁记录 |

## 9. S3：产品入口（`cuda` 门 + 三态回退）+ 端到端 target（窗口材料另行报签）

- 任务：`2026-10-27-product-collision-inputs-s3-entry-window`（owner Backend；HEAD 起点 `41fa049c`）。
- 本片**不开验证窗**：只交付入口、端到端 target、CPU/宿主侧证据与**窗口候选冻结清单**（§9.5）；
  窗口材料按 Arch 要求另行一次齐报 Chief 签。**本 README 不写「窗口通过」**。
- 结果一句话：S1 快照 + S2 adapter 接到产品**显式设备入口**（`cuda` 门、三态回退不 panic、默认 Host
  路径不变、只接受 `nworld=1`）；端到端 target 用 pinned 3.12.0 MJB 走 S1→S2→设备，对 **M1**
  （`mesh-box`/`mesh-cylinder`）与 **M3**（`mesh-mesh`）冻结参考做到 **12/12 接触行位级相等、0 ULP**、
  结构/参数零容差；开发自测 1 轮（非验证窗，见 §9.3 注）；gate/scope 见 §9.4。

### 9.1 产品入口（`src/runtime/collision_gpu/product_entry.rs`，174 行）

- 请求：`ProductCollisionRequest<'_> { snapshot: CollisionInputsSnapshot<'_>, meta: CollisionGpuMeta,
  kernel: CollisionKernel }`——S1 快照视图 + 运行时配对参数包 + 既有设备内核种类。
- 结果：`ProductCollisionOutcome::Device(Box<CollisionGpuOutput>)` 或
  `Host(ProductHostReason)`；`device()`/`host_reason()` 提供只读判定。
- 步骤固定：**先纯 CPU 校验**（`nworld=1` 口径 + S2 `inputs_from_snapshot` 复用既有
  `validate_collision_inputs`，先于任何设备探测）→ Host 选择直接返回 `Selected`、不触碰驱动/NVRTC/设备
  → Device 选择查 [`backend::product_collision_availability`] → 通过后取缓存设备程序并发射。
- **三态回退（不 panic、不重试）**：

  | 状态 | 触发 | 结果 |
  | --- | --- | --- |
  | ① 未编译 `cuda` | `--no-default-features`（或 `cuda-probe` 无 `cuda`）构建 | `Host(FeatureDisabled)`；编译期分支，不触碰设备 |
  | ② 设备/程序不可用 | 驱动/设备/NVRTC/PTX 编译探测失败，或探测通过但程序取不到 | `Host(DeviceUnavailable(原因))` / `Host(DeviceFailure("设备程序不可用…"))` |
  | ③ 运行期失败 | 发射/同步/回读返回错误 | `Host(DeviceFailure(错误文本))` |
  | 纯 CPU 非法输入 | `nworld>1`、快照/长度/meta 不自洽 | `Err(CollisionGpuError::InvalidArgument)` |

- **nworld 口径（QA S2 O2 结论）**：本片只接受 `nworld=1`；`nworld>1` 返回明确
  `InvalidArgument`（「多 world 状态读取另立」），**不宣称多 world**；拒绝发生在任何设备探测之前。
- **默认数值路径仍 Host**：Host 选择不探测设备、不编译 NVRTC；Device 必须显式选择（UNC-2 限定语：
  入口不自动切换，设备不可用按既有约定回退 Host）。
- `CollisionGpuProgram::run` 与 `CollisionArgs`（504 B）**零改动**；入口只组合既有组件。
- 单测（`product_entry_tests.rs`，346 行，纯 CPU、无 GPU）：Host 选择不碰设备、①②③ 各档回退原因、
  程序缺失、成功路径内核透传、`nworld=0/2` 拒绝、非法快照拒绝、`cuda` 构建可用性与缓存窄相状态
  一致；`--no-default-features --features cuda-probe` 构建下额外覆盖 ①编译期分支（9 passed）。

### 9.2 后端接线（`src/runtime/backend.rs`，184 行）

- 新增 `product_collision_availability() -> Result<(), BackendUnavailable>`：
  `cfg(not(feature = "cuda"))` → `FeatureDisabled`（编译期回退，不碰驱动/NVRTC/设备）；
  `cuda` 构建 → 复用 `narrowphase_backend_status()` 的进程级缓存（`CudaReady` → `Ok(())`，
  `HostOnly(原因)` → `Err(原因)`）。只读缓存结论，不改变默认 Host 数值链、不改变任何设备内核。

### 9.3 端到端 target（`tests/g10_p5_product_collision_wiring_cuda.rs`，162 行 + support 122 行）

- 真实模型：pinned MuJoCo **3.12.0**（python wheel，authoring-only）从冻结 XML 编译 3 份 MJB，
  存于 `fixtures/mjb-312/`（machine-readable 证据，测试只读、**不启动 Python**）：

  | case | MJB | bytes | sha256 | XML | qpos |
  | --- | --- | --- | --- | --- | --- |
  | `mesh-box` | `mesh-box-312.mjb` | 8800 | `70b2fdc3…` | `fixtures/g10-p5-mesh-multicontact/mesh-box.xml`（`88b8d7af…`） | z=0.14 |
  | `mesh-cylinder` | `mesh-cylinder-312.mjb` | 8815 | `ceef075c…` | `…/mesh-cylinder.xml`（`d0b75f91…`） | z=0.19 |
  | `mesh-mesh` | `mesh-mesh-312.mjb` | 12032 | `298b8089…` | `…/mesh-mesh.xml`（`bb9ffb2f…`） | z=0.18 |

  生成/对账脚本（入库）：`scripts/generate_collision_inputs_mjb.py`（245 行；pinned mujoco 3.12.0 校验，
  否则直接 FAIL）；provenance：`fixtures/mjb-312/provenance.json`（MJB/XML/reference/qpos/32 数组
  逐项 sha）；日志 `logs/s3-mjb-generation.log`。
- **输入侧对账（CPU，无设备工作）**：3.12.0 MJB 的 32 数组与 M0b/M3 冻结 bins **96/96 项逐字节相等**
  （0 mismatch；脚本 `RESULT=PASS`）；target 内 `s1_snapshot_of_pinned_mjb_matches_frozen_case_bins`
  再走真实 S1 探针 → `ConfigInputsModelInput` + qpos 帧 → S2 adapter，得到
  **3 case × 32 项 = 96/96 与冻结 bins 逐字节相同**（不依赖近似相等）。
- **设备侧对照（窗口执行）**：`product_entry_device_output_matches_m1_and_m3_frozen_references`：
  S1 快照 → S2 adapter → `run_product_collision(Device, …)`（`ContactMulti`）→ 对 **M1**（`mesh-box`/
  `mesh-cylinder`，参考 `c83baffe…`）与 **M3**（`mesh-mesh`，参考 `96e7895d…`）冻结参考；判据沿用
  `tests/support/mesh_multicontact_compare_support.rs`（零放宽）：位级优先 + dist 4 ULP@量级 +
  pos 4·ULP(scale) + frame 4 ULP@1.0 + 结构/参数零容差 + `nacon` 与参考 count 零容差 +
  `contact.geom` 按 pinned 类型序 + overflow 全 0 + efc 全 -1；参考 sha 守卫写死在 support，只读。
- 命令（需 `MJWARP_MUJOCO_DLL` + CUDA bin/NVRTC + NVIDIA GPU）：
  `cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features native-model-probe --test
  g10_p5_product_collision_wiring_cuda -- --ignored --test-threads=1 --nocapture`
- **本片开发自测（非验证窗）实际结果**（`logs/s3-targeted-native.log`）：2 passed / 0 failed；
  `mesh-box`/`mesh-cylinder`/`mesh-mesh` 各 `nacon=4 = ref_count`，**bit_equal=12/12 行**，
  `max_dist_ulp=0 / max_pos_ulp=0 / max_frame_ulp=0`，`contact.geom=[1,0]/[1,0]/[0,1]`，**failures=0**；
  S1 快照对账 `s1_fields=32/32` × 3 case、`mismatches=0`。
  **口径**：这是一次本地开发自测（单 target 直接跑，无窗口 guard/负载声明/冻结 exe），
  只用于确认 target 可执行与数值落点；**窗口结论以窗口任务冻结 exe 的独立重跑为准**，本片不写
  「窗口通过」。

### 9.4 门禁与范围

- `normify gate 2026-10-27-product-collision-inputs-s3-entry-window` **exit 0**：fmt 通过、clippy 通过、
  clippy-cuda 通过、**unit 1006 过 / 0 败 / 376 ignored**（基线 997/0/376，+9 项本片纯 CPU 单测）。
- clippy 在默认与 `--features native-model-probe` 两套均零警告（`logs/s3-fmt-clippy.log`）；
  `--no-default-features --features cuda-probe` 构建下 ①编译期回退用例 9 passed
  （`logs/s3-nocuda-feature.log`）。
- `scope-selfcheck`：**17 路径 / 越界 0**；`normify check` 全部通过。
- 不改 18 个 pinned 源、`src/runtime/collision_gpu/abi.rs`、`src/physics/collision/cuda/`、
  `fixtures/g10-p5-mesh-multicontact/`、`fixtures/g10-p5-primitive-branches/`、M0b/M1/M2/M3 证据目录
  （machine check `unchanged` 覆盖 26 个路径）；`CollisionArgs` 504 B 与既有 ABI 未动；无新 feature。

### 9.5 窗口候选冻结清单（材料另行报签，本片不开窗）

- **新 target（1）**：`g10_p5_product_collision_wiring_cuda`（`--features native-model-probe` 构建；
  运行时需 `MJWARP_MUJOCO_DLL` 与 CUDA/NVRTC bin；本机开发自测墙钟约 42 s，其中 NVRTC 编译约 41 s）。
- **回归实体（9）**：`multicontact` / `primitive` / `primitive_plane` / `primitive_sphere_capsule` /
  `primitive_branches` / `mesh_multicontact_m2` / `mesh_multicontact_m3` / `nvrtc_launcher`（含
  hfield-capsule）/ `stage2_equivalence_window`（record-C）。
- 冻结后窗口口径沿用既有纪律：sha 守卫 + `SHA_OK` 落盘、负载 precheck、`floor=min(10240, baseline_free−3072)`、
  0.25 s 采样、pre/post/结束基线、零 Rust 编译、≤2 有效轮、日志 UTF-8 无 BOM。
- 本片新增只读 fixture 需纳入窗口 fixture 守卫：3 份 MJB + `provenance.json`（+ 既有 M0b/M3 case bins）。

### 9.6 S3 交付文件

| 文件 | 行/字节 | 说明 |
| --- | --- | --- |
| `src/runtime/collision_gpu/product_entry.rs` | 174 行 / 7623 B | 产品入口：显式后端选择 + 三态回退 + nworld 口径 |
| `src/runtime/collision_gpu/product_entry_tests.rs` | 366 行 / 13038 B | 三态回退/Host/成功/拒绝单测（纯 CPU） |
| `src/runtime/collision_gpu.rs` | 466 行（+4） | `mod product_entry` + `pub use`（既有入口/ABI 不动） |
| `src/runtime/backend.rs` | 184 行（+19） | `product_collision_availability`（`cuda` 门接线） |
| `tests/g10_p5_product_collision_wiring_cuda.rs` | 162 行 / 7453 B | 端到端 target（S1→S2→设备→M1/M3） |
| `tests/support/product_collision_wiring_support.rs` | 122 行 / 5092 B | MJB/参考/qpos 只读守卫 |
| `tests/support/collision_inputs_adapter_support.rs` | 320 行（+1） | 只加 `#![allow(dead_code)]`（被两个 target 复用） |
| `fixtures/mjb-312/*.mjb`（3）+ `provenance.json` | 8800/8815/12032 B + 35941 B | pinned 3.12.0 编译产物与 32 数组对账证据 |
| `scripts/generate_collision_inputs_mjb.py` | 245 行 / 10115 B | authoring-only MJB 生成 + 96 数组 bin 对账 |
| `logs/s3-targeted-native.log` 等 4 份 | — | 定向测试、fmt/clippy、no-cuda 分支、MJB 生成（UTF-8 无 BOM） |

