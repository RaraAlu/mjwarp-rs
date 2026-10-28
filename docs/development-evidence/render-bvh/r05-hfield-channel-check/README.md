# R05 hfield-channel 独立复核（适配器语义 / 单一 f64→f32 转换点 / 两层语义边界 / 零 native 红线 ABI / 证据）

任务：`2026-10-28-render-bvh-r05-hfield-channel-check`（P8 渲染与 BVH；scope 仅本目录）
被复核对象：`2026-10-28-render-bvh-r05-hfield-channel`（实现+证据 `9a6fd66e`、收口 `c8ccf66c`；判据树 HEAD `9a32bf87`）
上游锚点：冻结 MJWarp 3.15 `render.py`（`71da24d`）；复用通道 `VolumeModelInput`（G1/G10 链路）与 `R05-S1` 的 `resolve_hfield`
复核方式：只读 + 四门禁独立复跑 + **临时对抗性探针**（运行后删除、不入库）；无 GPU、无窗口、无渲染数值结论。

## 0. 结论

**verdict：pass；阻断 0；观察 5（全部记录级/接受，无必须修项）。**

方案 A（薄适配器复用已验证的体积通道）的语义、单一转换点、两层语义边界、零 native/零字段表/零红线/零 ABI、
7 条测试强度、证据完整性均独立复核通过；未发现阻断项。我另跑了 4 条临时对抗性探针（两层边界/极端值/多 hfield/借用）
补充实现片未覆盖的边界。

## 1. 独立复跑

| 项 | 命令 | 退出码 | 结果 |
| --- | --- | --- | --- |
| fmt | `cargo fmt --check` | 0 | 通过 |
| clippy | `cargo clippy --all-targets --locked -- -D warnings` | 0 | 通过 |
| clippy-cuda | `... --features cuda -- -D warnings` | 0 | 通过 |
| unit | `cargo test --locked` | 0 | **1069 passed / 0 failed / 387 ignored**（132 条 result 行求和） |
| 架构树 | `normify validate --fingerprints` | 0 | 错误 0（警告 5，既有项） |

- 实现片 gate 记录一致：`.normify/runs/<impl>/unit.json` = **1069 过 / 0 败 / 387 ignored**（at `13:44:35Z`）；四门禁记录 exit 0。
- 实现片 `normify check` exit 0；实现文件在收口后零改动；`unit 1069` 相对 R05-S1 基线 1060 = S1 加固片新测 + 本片 7 条。

## 2. 适配器语义（静态 + 探针）

`src/render/context/hfield_inputs.rs`（70 行）独立核读：

| 项 | 核验结果 |
| --- | --- |
| `nhfield==0 → Ok(None)` | `if model.counts().nhfield == 0 { return Ok(None) }` 在读取任何字段**之前**提前返回 |
| `size` 单一转换点 | 仅 `model.hfield_size_f32()?`；适配器代码**无** `as f32` / 无 `num_to_f32` 二次转换 |
| `data` 借用不复制 | `model.hfield_data()?` 返回 `&[f32]`，无 `to_vec()/clone()` |
| 错误直传 | `from_volume` 返回 `Result<Option<Self>, NativeProbeError>`，字段访问错误 `?` 直传 |
| 单份 | 代码无 `nworld` 读取/参数/循环 |
| 组合 | `as_inputs()` 五字段全 `Some`；`resolve()` = `resolve_hfield(&self.as_inputs())` |

我的临时探针（`tests/_qa_r05_channel_probe.rs`，运行后删除）**4/4 通过**（`logs/probe-adapter.log`）：
两层边界（见 §3）、负数/极端值（见 §3）、多 hfield + 单一转换（见下）、无 hfield 边界（见 §3）。

**多 hfield / 单一转换（探针）**：两 hfield（3×3 adr=0、2×2 adr=9，data 13）→ 适配后 `size` 共 8 个 f32，
与 `num_to_f32(原生 f64 数组)` **逐位相等**；`resolve()` 得 2 个视图（span 9/4、adr 9），
`params.data()` 指针与 `model.hfield_data()` **相同**（借用未复制）。

## 3. 两层语义边界（重点，显式核验，不静默改写）

- **体积层**：`volume_input.rs::check_ranges` 对 hfield 只要求 `nrow/ncol/adr >= 0`（**负数拒绝、零接受**），
  且 `end = adr + nrow*ncol <= nhfielddata`；`nrow=0` 可构造。
- **S1 层**：`hfield.rs::resolve_hfield` 对 `nrow<=0 || ncol<=0` 返回 `HfieldError::NonPositiveRowsCols{index,nrow,ncol}`。
- **适配器**：不读改写，错误直传。
- **实现片测试已显式覆盖**：`boundary_errors_are_rejected_by_volume_construction` 断言「体积层接受 `nrow=0` → `resolve()` 返回
  `NonPositiveRowsCols{index:0,nrow:0,ncol:3}`」，并在注释写明两层口径差异、不做静默改写。
- **我的探针独立补充**：`nrow=0`、`ncol=0`、多 hfield 中第 2 个 `nrow=0`（index=1）三种样例——体积层均接受、
  `resolve()` 均返回对应 index/值的 `NonPositiveRowsCols`；`nrow=-1`/`ncol=-1`/`adr=-1`/`adr+span` 越界在**体积层**即拒绝；
  `i32::MAX × i32::MAX` 极端乘积 `catch_unwind` 不 panic、在体积层被拒。

## 4. 零 native / 零字段表 / 零红线 / 零 ABI（自己 diff）

- 实现提交 `9a6fd66e` 仅 12 路径：4 个源码/测试路径（`hfield_inputs.rs` 新增、`params.rs` +4、`tests/render_hfield_inputs.rs` 新增、
  `render.md` +3）+ 治理/证据；**无** `src/model/render.rs`、**无** `src/io/render.rs`、**无** native/include、**无** `*.cu`。
- `git diff 9a6fd66e^..HEAD` 对 `src/model/render.rs`、`src/io/render.rs`、`native/`、`include/`、`src/render/kernel/`、
  `fixtures/`、`src/runtime/`、`Cargo.*` **为空**。
- `RENDER_FIELDS` 仍 **116**；`MEGAKERNEL_SLOTS` 81；`hfield_bvh_id` 仍占位 `[0u32,0]`；`DEVICE_SOURCES=28`、入口 6、`CollisionArgs=504 B`。
- 行数（LF 实测）：`hfield_inputs.rs` 70、`tests/render_hfield_inputs.rs` 218、`params.rs` 197、`hfield.rs` 372、`mod.rs` 384、
  `model/render.rs` 500、`io/render.rs` 453、`render.cu` 499、`launch.rs` 500、`cast.rs` 494、`camera_output.rs` 499、`megakernel.rs` 490——全部 ≤500。

## 5. 7 条测试强度（逐条读）

| 用例 | 断言强度核读 |
| --- | --- |
| `adapts_volume_hfield_into_inputs` | 五字段（size/nrow/ncol/adr/data）逐个 `assert_eq`，非空断言 |
| `resolve_full_chain_matches_s1_views` | 全链路视图逐字段 + `data_of` 内容断言 |
| `no_hfield_returns_none` | `nhfield=0 → None` |
| `g10_fixture_arrays_match_volume_adapter` | 读 `fixtures/g10-p5/hfield-f32/hfield-box-cpu.json`：size 与 `num_to_f32` 逐位、nrow/ncol/adr/data 同源数组相等（**输入一致性**，见 §6） |
| `size_conversion_is_single_f64_to_f32` | 0.1 位型 `0x3DCC_CCCD` + 与 `num_to_f32` 整体相等 |
| `boundary_errors_are_rejected_by_volume_construction` | adr 越界体积层拒绝；`nrow=0` 两层差异显式断言（见 §3） |
| `adapter_is_stable_and_copy_free` | 两次适配 `PartialEq`、`data` 指针相等、视图数=nhfield |

均无空断言/自证式断言；未发现以「输入一致性」冒充渲染数值结论的写法。

## 6. G10 夹具比对口径（必须守住）

该用例只作**输入一致性**：`size` 经唯一 `num_to_f32` 与夹具数组比对，`nrow/ncol/adr/data` 与夹具同源数组相等；
`num_to_f32` 实现即 `values.iter().map(|&v| v as f32)`（一次就近舍入，ties-to-even），结论里**不含**任何渲染数值/等价口径。
证据 README §8 与 receipt `uncovered` 均写明「设备侧 BVH/相交未实现、无上游 2D 数值/等价」——未夸大覆盖。

## 7. 模块源

- `.normify/modules/mjwarp/render.md`：`[[source]]` 共 **36** 条，含 `src/render/context/hfield_inputs.rs`；
  `hfield.rs` 也在（上一片 O1 缺口已由 R05-S1 加固片补登记，本片主动预防同类缺口）。
- `params.rs` 的 `#[path = "hfield_inputs.rs"]` 挂载与 `lighting.rs`/`hfield.rs` 同构——既有结构债（R09-S1 O2 已记录），本片无新增债务类型。

## 8. 证据完整性（实现片）

- 目录 **7 文件**（README + receipt + manifest + 4 logs）；`manifest.txt` **6 条**逐条对 committed blob 相符、不自引用、LF 无 BOM、禁词 0。
- `logs/test.log` = 7 passed；`logs/lines.log` 行数字节与我的实测一致；`logs/fmt.log` 空 = fmt 通过。

## 9. 观察（全部记录级/接受）

- **O1（记录｜接受）顶层导出缺失**：`VolumeHfieldInputs` 仅可从 `mjwarp_rs::render::context::params::hfield_inputs` 取得，
  未 `pub use` 到 `mjwarp_rs::render`（R05-S1 的 `HfieldInputs/HfieldParams/HfieldError` 是顶层导出）。本卡 scope 只含
  `src/render/context/`，证据已说明「S2 需要时加一行 re-export」；建议 S2 接口落地时统一导出层级。
- **O2（记录｜接受）`NativeProbeError` 直传路径实际不可达**：`VolumeModelInput::new` 成功后 hfield 字段访问器不会失败，
  故 `from_volume` 的 `Err` 分支为防御性；保留 `Result` 形态合理，建议在 doc 注明「构造期已保证」。
- **O3（记录｜接受）`#[path]` 挂载结构债**：与 `lighting.rs`/`hfield.rs` 同构，沿用既有模式（R09-S1 O2）。
- **O4（记录｜接受）夹具比对侧转换**：测试侧把夹具 JSON 的 f64 数组 `as f32` 后与 `hfield_data` 比较；`num_to_f32` 实现亦为
  `as f32`，两者同一语义，且适配器路径只有 `hfield_size_f32()` 一次转换——不构成第二转换点；口径为输入一致性。
- **O5（记录｜接受）两层口径已显式化**：体积层 `>=0`（接受 0）与 S1 层 `>0` 的差异由实现测试 + README §3 + 我的探针三方一致确认，
  无静默改写/放宽；`adr` 越界在体积层构造期拒绝，适配器不会见到非法区段。

## 10. 边界（我没有做的）

- 无 GPU、无窗口、无渲染数值结论；设备侧 hfield BVH/相交、上游 2D 数值等价均为 S2 范围。
- 「谁在 render context 创建时提供 `VolumeModelInput`」留给 S2 的接口决定（证据已列未覆盖项）。
- `CollisionInputs` 通道本片未用（证据已说明）；我未评测其与 `VolumeModelInput` 的一致性。
- 我的适配器探针为临时测试文件（运行后删除、未入库），结果见 `logs/probe-adapter.log`。

## 11. 复算入口

```text
<repo>/target/render-bvh-precheck/venv312/Scripts/python.exe docs/development-evidence/render-bvh/r05-hfield-channel-check/recompute.py
```

`recompute.json` 含：实现证据完整性、行数/红线、实现提交范围与零 native diff、`RENDER_FIELDS` 116、ABI 计数、
适配器静态核对（提前返回/单一转换/借用/无 nworld/错误直传）、两层语义静态核对（`>=0` vs `<=0`、`num_to_f32` 实现）、
模块源 36、门禁记录（1069/0）、实现 7 条单测日志、我的 4 条探针、我的四门禁聚合、本证据 manifest 自校——`problems = []`。
