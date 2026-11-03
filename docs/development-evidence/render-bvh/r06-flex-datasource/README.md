# R06-S2b（合并）：flex 渲染数据源 + T4 源码身份重钉（7 文件 12 行）

任务：`2026-11-04-render-r06-s2b-flex-datasource-t4-merged`（单卡单提交；原 S2b 与拆出的 T4 repin 卡已 abandon）。
日期：2026-11-04（空闲段）。**零 GPU 计算、零窗口、零参考生成**；`src/render/kernel/` 与全部 `.cu` 零改动（与 `src/render/kernel/cuda/` 一起零 diff）；fixtures 侧**只有 4 个登记 fixture**（`fixtures/precision/` 的 `t4-current-kernels.json`、`g01-inputs.json`、`g01-packing-routes.json`、`g01-state-packing.json`）与 3 个 `docs/evidence/t4-g03-retest/` map 做了身份重钉，**其余 fixtures 零 diff**。

## 1. 交付物（代码面）

| 文件 | 行数 | bytes | 作用 |
| --- | --- | --- | --- |
| `include/mjwarp_native_probe.h` | 145 | 6265 | additive ABI：`mjwarp_flex_render_info`(32 B) / `mjwarp_flex_render_targets`(72 B) + 2 个 C 函数声明 |
| `native/model_probe.cpp` | 327 | 15270 | 实现 `mjwarp_native_flex_render_info` / `mjwarp_native_copy_flex_render` + static_assert 偏移/大小校验 + preflight |
| `src/io/flex/render.rs` | 337 | 12769 | `NativeFlexRenderSnapshot::capture/into_fields`（FFI DTO + 校验 + 单测 + 1 个默认忽略的原生冒烟） |
| `src/io/flex.rs` | 466 | 16653 | 只加 `mod render;` + `pub use render::NativeFlexRenderSnapshot;` |
| `src/io/mod.rs` | 472 | 18846 | `NativeModelProbe::flex_render_snapshot(&self)` + `pub use flex::{...}` |
| `src/model/flex.rs` | 407 | 13759 | `FlexRenderFields`（计数 + dim/radius/rgba/edgeadr/edgenum）+ `validate()` + `geom_mapping()` + 单测 |
| `src/physics/flex.rs` | 439 | 16432 | `FlexRenderView`/`FlexRenderWorld` 只读视图（逐 world `flexvert_xpos`、半径、rgba、边拓扑、host 只读 `edge_lengths`）+ 单测 |

全部代码文件 ≤500 行（`src/io/flex.rs` 466、`src/io/mod.rs` 472 均在死线内）。

## 2. 原生 ABI（additive；不改 `mjwarp_flex_position_*`）

| struct | size/align | 字段偏移 |
| --- | --- | --- |
| `mjwarp_flex_render_info` | 32 B / 8 | `nflex@8`、`nflexvert@16`、`nflexedge@24` |
| `mjwarp_flex_render_targets` | 72 B / 8 | `nflex@8`…`nflexedge@24`、`flex_dim@32`、`flex_radius@40`、`flex_rgba@48`、`flex_edgeadr@56`、`flex_edgenum@64` |

- 只新导 **5 个字段 + 3 个计数**；`flex_edge` 由 G01 字段集提供，**不重复导**（单真相源，证据 README 口径）。
- `schema=1/reserved=0` 校验；`flex_radius` 原生 `mjtNum`(f64)，`flex_rgba` 原生 `float`；错误码 8 = preflight 拒绝。
- `static_assert` 覆盖 struct 大小/对齐/偏移；native 侧 `-W4 -WX` 下零警告（C++ 注释保持 ASCII）。
- 不改 `native/g01_fields.inc`、不改 G01/Render probe 既有字段 ID。

## 3. 字段与上游对应（对齐 S2 的 9 个 flex 通道数组）

| 上游 `_render_megakernel` 形参 | 本卡来源 |
| --- | --- |
| `flex_vertadr` / `flex_edge` | 既有 flex-position / G01 输入（本卡不重复导） |
| `flex_radius` | 新原生桶（`mjtNum`→f32 单点转换） |
| `flexvert_xpos_in` | physics 数据面 `FlexPositionOutput::world(w).flexvert_xpos` |
| `flex_rgba` | 新原生桶（float 直拷 + finite 校验） |
| `flex_bvh_id` / `flex_group_root` | 留 S3（flex BVH 构建）填 |
| `flex_geom_flexid` / `flex_geom_edgeid` | `FlexRenderFields::geom_mapping()` 派生（1D 每边一个胶囊图元 `edgeid=edgeadr+e`；2D/3D 每 flex 一个盒图元 `edgeid=-1`） |
| `bvh_nflexgeom` | `geom_mapping().len()` |

**f64→f32 转换集中一处**：`io::kinematic::native::convert("flex_radius", …)`（非有限/超 f32 范围错误带字段名与下标）；
`flex_rgba` 原生即 float，仅逐项 finite 校验；其余字段/拓扑全 i32。

## 4. 校验面（不 panic、不静默回退）

- **io**：计数非负且 ≤`i32::MAX`、按数组展开不溢出；五数组长度 = 计数×宽度（`check_length`）；`flex_rgba` 逐项 finite。
- **model `FlexRenderFields::validate()`**：`flex_dim∈{1,2,3}`、`flex_radius` 有限且 ≥0、`flex_rgba` 有限、`nflex>0` 时 `nflexvert>0`、
  `flex_edgeadr[0]==0`、`edgeadr[f]+edgenum[f]==edgeadr[f+1]`、末项和 == `nflexedge`、`nflex≤i32::MAX`。
- **physics `render_view`**：`positions`/`edges`/`model` 与输出同源（nflex/nflexvert/nflexedge/edge 长度）否则 `InputError`；
  `edge_lengths` 顶点号越界显式拒绝（host 只读视图，**非渲染判据**）。

## 5. T4 源码身份重钉（7 文件 **12 行**，逐行 old→new）

> 口径更正：任务卡/接管文书写「7 文件 11 行」是**我方早期口径少计 1 行**（`g01-inputs.json` 实为 4 行），
> 实际改动 = **7 文件 12 处身份字面量**；**未少改任何行**。逐行见下表。

全部为 **LF 归一化 sha256**（`digest` 口径；`include/mjwarp_native_probe.h` 与 `native/model_probe.cpp` 工作树为 CRLF，取归一化值）。
只改身份期望字面量：**不改断言逻辑、容差、排除清单、上游 pin、数值参考、判据**；precedent `61fb6baf`/`097c54b9`。

| # | 文件 | 条目 | 旧值 | 新值 |
| --- | --- | --- | --- | --- |
| 1 | `fixtures/precision/t4-current-kernels.json` | `src/physics/flex.rs` | `cb2977a3…` | `0299be863b318168b5b391af4b921e1db3b5420b2da1a68b4abc93bb9249612d` |
| 2 | `fixtures/precision/g01-inputs.json` | `include/mjwarp_native_probe.h` | `03151bf5…` | `e40aff139ec2af3e67e49a65c922bacd80672a16c4826b7c03788273042c7adf` |
| 3 | `fixtures/precision/g01-inputs.json` | `native/model_probe.cpp` | `6a01eb01…` | `f52728543ae9d44d7c3fbe948645c0ab8572c5c15a1c5efe90a607d58150ff28` |
| 4 | `fixtures/precision/g01-inputs.json` | `src/io/flex.rs` | `c0f3e0bb…` | `f74256b311642e1ac17c74b8776291a37e3a17bba6c0f0b23a6f6b7c35ad9ce4` |
| 5 | `fixtures/precision/g01-inputs.json` | `src/model/flex.rs`（单文件） | `8355b0f1…` | `477355d8a47b1a3350a30c167f3b02683dc4fca7f42bf2e94e24ac5828ca2cb7` |
| 6 | `fixtures/precision/g01-packing-routes.json` | `src/physics/flex.rs` | `cb2977a3…` | `0299be86…` |
| 7 | `fixtures/precision/g01-packing-routes.json` | `src/model/flex.rs`（单文件） | `8355b0f1…` | `477355d8…` |
| 8 | `fixtures/precision/g01-state-packing.json` | `src/physics/flex.rs` | `cb2977a3…` | `0299be86…` |
| 9 | `docs/evidence/t4-g03-retest/flex-internal/map.json` | `src/physics/flex.rs` | `cb2977a3…` | `0299be86…` |
| 10 | `docs/evidence/t4-g03-retest/flex-internal/map.json` | `src/model/flex.rs`（组合：flex.rs+edge+face+input+position，`\n` 连接） | `b3603225…` | `9834e3a0931abfe58f10050ee5623a0951e3927643c06a2ab5ae3f4b24a99e25` |
| 11 | `docs/evidence/t4-g03-retest/flex-matrices/map.json` | `src/model/flex.rs`（同组合） | `b3603225…` | `9834e3a0…` |
| 12 | `docs/evidence/t4-g03-retest/rigid-rotate-recapture/previous-kernels.json` | `src/physics/flex.rs` | `cb2977a3…` | `0299be86…` |

（12 行口径：`t4-current-kernels.json` 1 + `g01-inputs.json` 4 + `g01-packing-routes.json` 2 + `g01-state-packing.json` 1 +
`flex-internal/map.json` 2 + `flex-matrices/map.json` 1 + `previous-kernels.json` 1 = **12**；文件数 7。）

**重钉前 FAIL → 重钉后 PASS 原始输出**：
- `repin-before-fail.txt`：全量 `cargo test --locked --no-fail-fast` 在重钉前的失败块（lib `unexpected lineage change`；
  `t4_input_precision` 5 个 `source drift` / `product source drift`）。
- `repin-before-fail-identity.txt`：用 `T4_IDENTITY_LIVE_FIXTURE=<HEAD 旧 fixture>` 定向复现，原始输出含
  `expected=cb2977a3… actual=0299be86…`（exit 101）。
- `repin-after-pass.txt`：重钉后定向 PASS 原始输出——`t4_identity_live_check` 2 passed / 0 failed；
  `t4_input_precision` 36 passed / 0 failed / 12 ignored；`recapture::tests_host` 2 passed / 0 failed。
- 全量 PASS 由门禁 `unit` 记录（`.normify/runs/<task>/unit.json`）。

## 6. 单测与原生验证

- 默认特性新增单测 **10 个**（model `render_tests` 4 + io `flex::render` 4 + physics `render_view` 2），全部 PASS：
  - 空/单/多 flex、长度/世界数/映射/单调拒绝、`max(1)` 与 2-D 形状口径、u64 字对承载、只读视图 + host 边长。
- **原生验证记证据即可（Chief 口径，不加 gate）**：`native-verify.txt` 原始输出
  - 命令：`cargo +1.99.0-x86_64-pc-windows-msvc test --locked --no-default-features --features native-model-probe --lib flex -- --ignored --test-threads=1`
  - 环境：VsDevCmd x64 + `MJWARP_MUJOCO_ROOT=target/toolchains/mujoco-3.12.0/package`、`MJWARP_MUJOCO_DLL=…/bin/mujoco.dll`
  - 结果：`captures_flex_render_fields_from_flex_fixture ... ok`，**1 passed / 0 failed**；
    夹具 `fixtures/flex-position/flex-tree.mjb`：nflex=4、映射长度 = 上游 `bvh_nflexgeom` 口径、1D 图元带边号 / 2D/3D 边号 -1。

## 7. 门禁

fmt / clippy / clippy-cuda / unit（见 `.normify/runs/2026-11-04-render-r06-s2b-flex-datasource-t4-merged/`）；
`unchanged` = `src/render/kernel/cuda/` 零改动，全部 `.cu` 零改动。

## 8. 基线差异与非声明

- `fixtures/flex-position/flex-tree.mjb` 是 **MuJoCo 3.12.0** 样本；pinned render 上游参考是 **3.15 采集**——
  本卡只做字段管道与拓扑校验，**不做数值结论**，不宣称 flex 渲染可用。
- S2b 只交数据层：`render_with_flex` 真实调用方接线**并入 S3 前置**（Chief 口径），不新增 S2c。
- `edge_lengths` 是 host 只读诊断视图，S3 设备侧仍按上游公式自行计算，不作为判据。
- 未做：`FlexHostInputs` 组装、设备 cast/BVH/seg、GPU/窗口。
