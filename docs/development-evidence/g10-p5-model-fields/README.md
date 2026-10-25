# G1 模型字段片：hfield/octree/`opt.sdf_*` 读取与跨版本位级 parity

任务：`2026-10-25-g10-p5-model-fields-r2`（范围含 native 导出扩展；首次登记因漏 `native/` 已 abandon）。
上游基线：`71da24d956378a87a703b6e1442b13aec0c4ac29`。本片无 GPU、不实装 GPU 内核、只读夹具。

## 1. 结论

- 产品读取链路已从 3.12 原生模型读出 hfield、mesh octree 与 `opt.sdf_*` 共 **14 个字段**；
  字段表在 `native/g01_fields.inc` 的 **45..58 号**，Rust 侧规范表 `model::volume::VOLUME_FIELDS` 逐项对应。
- 与 G0b 落盘 `fixtures/g10-p5/octree-parity.json` 做**位级**对照：`hfield-box`（noct=0）与
  `sdf-cylinder`（noct=63,497）两例的 12 个可比字段**全部全等**（小端原始字节 SHA-256 相等），
  无差异项。对照表见 §4。
- `oct_aabb`/`oct_coeff`/`hfield_size`（原生 f64）的 f64→f32 转换点唯一，见 §5；
  `hfield_data` 原生即 f32，不转换。
- `nworld>1` world 维上传口径只出**规格**，见 §6；未实装、未实测。
- ABI 不变：`mjwarp_g01_field_info` 尺寸/对齐/字段/`kind` 宽度均未动，`kind` 只新增取值；
  G01 既有 45 项字段的 kind/顺序/导出结果不变。

## 2. 改动文件

| 文件 | 改动 |
| --- | --- |
| `native/g01_fields.inc` | 字段与 kind 为**新增**（ID 45..58；kind 4=float32、5=int64）；另有两处宽度表达式抽成 `g01_scalar_bytes()` 的**等价重构**；既有 kind 1/2/3 语义不变（45 项既有字段前后 ABI 导出位比全等，见独立复核证据） |
| `include/mjwarp_native_probe.h` | 仅注释：kind 取值说明与 45..58 号用途；结构体未动（ABI 不变） |
| `src/model/volume.rs`（新） | 字段规范、原生计数、数据面与 f64→f32 唯一转换函数（≤500 行） |
| `src/model/volume_input.rs`（新） | 校验后的 `VolumeModelInput`：字段数/kind/长度/`noct` 一致性/下标范围校验（≤500 行） |
| `src/io/volume.rs`（新） | 原生读取：复用 `mjwarp_native_g01_field_info`/`mjwarp_native_copy_g01_field`，逐字段 kind 校验 |
| `src/io/mod.rs` | `mod volume` + `NativeVolumeSnapshot` 导出 + `NativeModelProbe::volume_snapshot()` |
| `src/model/mod.rs` | `pub mod volume` 与类型导出 |
| `tests/g10_p5_model_fields.rs`（新） | 位级 parity 集成测试（忽略项需可信 MuJoCo DLL；另有不依赖 DLL 的夹具解析测试） |
| `docs/development-evidence/g10-p5-model-fields/` | 本目录 |

**kind 取值更正（与本片开工令的表述差异）**：G01 字段表原有 kind 2 已是 `double`（f64），
缺少的是 **float32**（`hfield_data`）与 **int64**（`mjtSize` 的 `noct`）。
故本片新增 kind 4=float32、5=int64，未新增 f64 kind；`src/io/volume.rs` 对每个字段强制比对
原生返回的 kind 与规范 kind，不一致返回 `InvalidDimension`（不静默）。

## 3. 字段表（原生 ID / 类型 / 计数）

| ID | 字段 | 原生类型 | 分量宽度 | 计数基数 | kind |
| --- | --- | --- | --- | --- | --- |
| 45 | `hfield_nrow` | `int*` | 1 | `nhfield` | 1 |
| 46 | `hfield_ncol` | `int*` | 1 | `nhfield` | 1 |
| 47 | `hfield_size` | `mjtNum*` | 4 | `nhfield` | 2 |
| 48 | `hfield_adr` | `int*` | 1 | `nhfield` | 1 |
| 49 | `hfield_data` | `float*` | 1 | `nhfielddata` | 4 |
| 50 | `noct` | `mjtSize` | 1 | 常量 1 | 5 |
| 51 | `oct_depth` | `int*` | 1 | `noct` | 1 |
| 52 | `oct_child` | `int*` | 8 | `noct` | 1 |
| 53 | `oct_aabb` | `mjtNum*` | 6 | `noct` | 2 |
| 54 | `oct_coeff` | `mjtNum*` | 8 | `noct` | 2 |
| 55 | `mesh_octadr` | `int*` | 1 | `nmesh` | 1 |
| 56 | `mesh_octnum` | `int*` | 1 | `nmesh` | 1 |
| 57 | `opt.sdf_iterations` | `int` | 1 | 常量 1 | 1 |
| 58 | `opt.sdf_initpoints` | `int` | 1 | 常量 1 | 1 |

`noct` 由原生 `mjtSize` 读出后，用于 `oct_*` 各字段的期望计数与下标范围校验。
计数、长度、kind 与下标范围全部校验通过才构造 `VolumeModelInput`；无容差、无静默截断。

## 4. 位级 parity 对照表

期望来源：G0b `fixtures/g10-p5/octree-parity.json`（每字段 `sha256_312`，且 `sha256_315` 与之相等、
`equal=true`，即 MuJoCo 3.12 与 3.15 逐字节全等）。本实现读取 3.12 `mjb`，把小端原始字节做 SHA-256 比对。

| 字段 | dtype | hfield-box count | hfield-box 结果 | sdf-cylinder count | sdf-cylinder 结果 |
| --- | --- | --- | --- | --- | --- |
| `oct_aabb` | f64 | 0 | 全等（空） | 380,982 | 全等 |
| `oct_coeff` | f64 | 0 | 全等（空） | 507,976 | 全等 |
| `hfield_size` | f64 | 4 | 全等 | 0 | 全等（空） |
| `hfield_data` | f32 | 9 | 全等 | 0 | 全等（空） |
| `oct_depth` | i32 | 0 | 全等（空） | 63,497 | 全等 |
| `oct_child` | i32 | 0 | 全等（空） | 507,976 | 全等 |
| `mesh_octadr` | i32 | 0 | 全等（空） | 1 | 全等 |
| `mesh_octnum` | i32 | 0 | 全等（空） | 1 | 全等 |
| `hfield_nrow` | i32 | 1 | 全等 | 0 | 全等（空） |
| `hfield_ncol` | i32 | 1 | 全等 | 0 | 全等（空） |
| `hfield_adr` | i32 | 1 | 全等 | 0 | 全等（空） |
| `opt_sdf`（= `sdf_iterations`,`sdf_initpoints`） | i32 | 2 | 全等 | 2 | 全等 |

两例派生的原生计数亦与夹具一致：`hfield-box` =（nhfield 1 / nhfielddata 9 / nmesh 0 / noct 0），
`sdf-cylinder` =（nhfield 0 / nhfielddata 0 / nmesh 1 / noct 63,497）。

值级锚点（与上面的 SHA-256 互相印证）：

- `hfield-box`：`hfield_nrow=[3]`、`hfield_ncol=[3]`、`hfield_adr=[0]`、
  `hfield_size=[1.0, 1.0, 0.1, 0.1]`（f64 位型逐项比对）、
  `hfield_data=[0, 0.5, 0, 0.5, 1, 0.5, 0, 0.5, 0]`（f32 位型逐项比对）。
- `sdf-cylinder`：`noct=63_497`、`oct_depth.len=63_497`、`oct_child.len=63_497*8`、
  `oct_aabb.len=63_497*6`、`oct_coeff.len=63_497*8`、`mesh_octadr=[0]`、`mesh_octnum=[63_497]`。
- `opt_sdf`：两例均为 `sdf_iterations=10`、`sdf_initpoints=40`（拼接后 SHA-256 与夹具相等）。

差异项：无。若将来失败，集成测试给出字段名 + 计算得到的 SHA-256 与夹具期望；
夹具只存每字段摘要，逐元素首个差异索引需重采 3.15 原始值（本片不涉及）。

运行记录：`parity-run.log`（命令、工具链、环境变量、结果）。

## 5. f64→f32 一次转换点

- **位置**：`src/model/volume.rs::num_to_f32`（唯一转换函数）；唯一调用入口是
  `VolumeModelInput::oct_aabb_f32()` / `oct_coeff_f32()` / `hfield_size_f32()`
  （`src/model/volume_input.rs`）。原生读取层（`io::volume`）只保留 f64 原值，不做任何转换。
- **规则**：每个标量恰好一次 `f64 as f32`（就近舍入、ties-to-even），之后不再参与同类转换；
  不做 f64→f32→f64 往返、不二次量化。
- **哪些字段转换**：`oct_aabb`、`oct_coeff`、`hfield_size`（原生 f64 → 上传 f32）；
  `hfield_data` 原生即 f32，**不转换**；整数（`noct` 及各 i32 数组）**不转换**
  （`noct` 超出 i32 时由 `noct_i32()` 报 `Overflow`，不截断）。
- 与 mjwarp `put_model` 对齐：模型静态数组上传为 f32 时使用同一就近舍入规则；
  本片只提供宿主侧唯一转换入口，不实装设备上传。
- 位型示例（真实夹具值）：`hfield_size[2]` 的 f64 `0.1` = `0x3FB999999999999A`
  → 一次转换后 f32 `0x3DCCCCCD`；`1.0` = `0x3FF0000000000000` → `0x3F800000`；
  `hfield_data` 的 `0.5` 原生即 f32 `0x3F000000`，不经过转换。
- 集成测试对两例的每个 f64 分量断言：`converted.to_bits() == (raw as f32).to_bits()`，
  且 `f64::from(converted) as f32 == converted`（二次量化不改变位型）。

## 6. `nworld>1` world 维上传口径（只出规格，未实装）

**静态共享（广播）**：`hfield_data`、`hfield_nrow/ncol/size/adr`、`oct_depth/oct_child/oct_aabb/oct_coeff`、
`mesh_octadr/mesh_octnum`、`noct`、`opt.sdf_*` 都是模型静态数据，不随 world 变化，
按**一份共享缓冲**上传（所有 world 使用同一基址），不做逐 world 复制。

理由：与 mjwarp `put_model` 的模型数组语义一致；逐 world 复制只增加显存与上传时间，无物理收益；
G4/G5 的 hfield/SDF 查询只读同一份数据。

**接触容量维度预留**：`naconmax` 属接触/约束容量，与 world 维相关，不在本片实装；
规格为「每 world 的 naconmax 按 world 分段布局，总行数 = `naconmax_per_world * nworld`，
world 间行偏移在 kernel 侧按 world 计算」，供 G4/G5 引用；本片只登记该预留，
不写多世界打包结果、不声明多世界已验证。

## 7. 验证

- `cargo +1.99.0-x86_64-pc-windows-msvc check --locked --all-targets --features native-model-probe`：通过。
- `cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features native-model-probe --test g10_p5_model_fields -- --ignored --nocapture`：
  `1 passed; 0 failed`（两例位级 parity 全过；日志见 `parity-run.log`）。
- 默认工具链 `cargo test --locked`：**565 过 0 败 1 忽略**（G2 基线 561 + 本片宿主单测 4；
  忽略项为 G2 的 native 位级探索项）。
- 四门禁 fmt/clippy/clippy-cuda/unit 由 `normify gate 2026-10-25-g10-p5-model-fields-r2` 记录，
  结果绑定本提交范围（runs 目录）。
- 无 GPU、无 Python、无 `fixtures/**` 与 `Cargo.*` 改动。

## 8. 未覆盖 / unmet

- **GPU 内核未实装**：本片只做宿主读取、位级 parity 与转换点，设备上传与 hfield/SDF 内核属 G4/G5。
- **多世界未实测**：§6 只是上传口径规格，未实装、未跑 `nworld>1`。
- **Linux / T4 未验证**：本片只在 Windows + MuJoCo 3.12 原生库上执行；双平台未验证。
- **T4 精度清单未刷新**：`tests/t4_input_precision.rs`（`cuda-probe` 特性）用
  `fixtures/precision/g01-inputs.json` 里的源文件摘要约束 G01 输入面；该夹具在范围外且当前 HEAD
  已多处漂移，本片未刷新、未把新字段纳入该清单。故 T4 的「78 项输入面」口径**未扩展**，属已知缺口。
- **`src/io/g01.rs::FIELDS` 保持 45 项不变**（新字段不进 G01 快照）：G01 快照会一次性复制全部字段，
  若把 63,497 节点的 octree 数组并入，会给既有产品路径增加 MB 级拷贝；且上述 T4 清单把该表与
  78 项输入面硬绑定。新字段改走独立的 `io::volume` 子系统，复用同一原生 ABI 函数与字段号空间。
