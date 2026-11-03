# R06-S2c：flex shell/elem 数据层（additive 原生字段 + io/model 只读视图 + T4 重钉）

任务：`2026-11-04-render-r06-s2c-flex-shell-data`（p75，owner Backend；依赖 `…-s2b-flex-datasource-t4-merged`）。
日期：2026-11-04（空闲段）。基线 HEAD：`fb785a88`。**离线、零 GPU、零参考采集**；只交数据层。

按卡面口径：`flex_shell*` / `flex_elem*` 以 **additive** 方式进原生 ABI（不改既有
`mjwarp_flex_position_*` / `mjwarp_flex_render_*` 结构），io/model 提供只读视图，T4 源码身份按
precedent `61fb6baf`/`097c54b9` 重钉。**不实现设备逻辑、不宣称 flex 渲染可用**；不改上游 pin /
数值参考 / 判据 / 容差。

## 1. 交付面（全部 ≤ 500 行）

| 文件 | 行数 | 内容 |
| --- | --- | --- |
| `include/mjwarp_native_probe.h` | 174 | 新增 additive 桶 `mjwarp_flex_topology_info`(40 B) / `mjwarp_flex_topology_targets`(96 B) 与两个导出函数；既有结构体一字未改 |
| `native/model_probe.cpp` | 386 | 新桶的 `static_assert`（size/align/offset 7 项）+ `_info`/`_copy_flex_topology` 实现（计数/指针/溢出逐项预检） |
| `src/io/flex/topology.rs`（新） | 459 | `NativeFlexTopologySnapshot`（capture/into_fields）+ `validate_topology_against_render`（元素/壳逐维宽度）+ `NativeModelProbe::flex_topology_snapshot` 访问器 + 5 单测 + 1 原生冒烟（忽略） |
| `src/io/flex.rs` | 473 | `mod topology;` 注册与 re-export（访问器同文件，避免动 `src/io/mod.rs`） |
| `src/model/flex.rs` | 478 | `FlexTopologyFields` 只读字段 + `validate()`（计数/长度/非负；地址链与逐维宽度在 io 边界联合校验） |
| `src/physics/flex.rs` | 未改 | 卡面在 scope 内但本片**无需改**（只读视图落在 io/model；不改则无身份漂移） |
| `fixtures/precision/g01-inputs.json` | 4 行重钉 | 身份字面量（见 §4） |
| `fixtures/precision/g01-packing-routes.json` | 1 行重钉 | 同上 |
| `docs/evidence/t4-g03-retest/flex-internal/map.json` | 1 行重钉 | 同上（model flex 组合摘要） |
| `docs/evidence/t4-g03-retest/flex-matrices/map.json` | 1 行重钉 | 同上（model flex 组合摘要） |

## 2. additive ABI（新桶，既有桶未改）

`mjwarp_flex_topology_info`（40 B / align 8）：`schema,reserved` + 4 个 `int64_t` 计数
（`nflex,nflexelem,nflexelemdata,nflexshelldata`）。

`mjwarp_flex_topology_targets`（96 B / align 8）：`schema,reserved` + 4 个 `uint64_t` 计数 +
7 个指针（C++/Rust 双侧 `static_assert`/`const assert` 覆盖 size/align 与以下 offset）：

| offset | 32 | 40 | 48 | 56 | 64 | 72 | 80 | 88 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 字段 | `flex_elemadr` | `flex_elemnum` | `flex_elemdataadr` | `flex_elem` | `flex_shellnum` | `flex_shelldataadr` | `flex_shell` | — |

计数区 offset：`nflex=8`、`nflexshelldata=32`。约束与 S2b 桶一致：零计数允许空指针；非零计数
必须给齐源/目标指针；`nflexelemdata`/`nflexshelldata` 有 `SIZE_MAX/sizeof(int32_t)` 溢出预检；
`schema != 1 || reserved != 0` 一律拒绝（错误码 8）。

字段语义（`mujoco/mjmodel.h` L569-591）：`flex_elemadr`=首元素地址（元素空间）、
`flex_elemnum`=元素数、`flex_elemdataadr`=首元素顶点 id 地址（`flex_elem` 下标）、
`flex_elem`=扁平顶点 id（每元素 `dim+1` 个）、`flex_shellnum`=壳片段数、
`flex_shelldataadr`=首壳片段顶点 id 地址、`flex_shell`=扁平顶点 id（每片段 `dim` 个）。
`flex_vertadr`/`flex_vertnum` 仍在 `flex_position` 桶，不重复导（单一真相源）。

## 3. io/model 只读视图

- `src/io/flex/topology.rs::NativeFlexTopologySnapshot`：与 S2b `render.rs` 同构的 FFI 布局、
  `capture()`（计数预检→staging→一次拷贝）与 `into_fields()`（长度逐项核对→
  `FlexTopologyFields::validate`）。**本桶全整数，无 f64→f32 转换点**（转换仍只在 render 桶
  `flex_radius` 一处）。
- `src/model/flex.rs::FlexTopologyFields`：7 个只读数组 + 4 个计数；`validate()` 负责计数/长度/
  非负与地址非负（结构性，不依赖 `flex_dim`）。
- `validate_topology_against_render(topology, render)`：跨桶完整校验 —— 元素空间地址链
  （首 0、逐 flex 连续、总和 = `nflexelem`）、两个顶点 id 空间链（首 0、单调、宽度 =
  `elemnum*(dim+1)` / `shellnum*dim`）。该函数放在 io 边界是因为逐维宽度需要 `flex_dim`
  （在 render 桶），model 层不重复携带。
- `NativeModelProbe::flex_topology_snapshot()` 访问器与快照同文件（R06-S2c 卡 `schedule.files`
  不含 `src/io/mod.rs`，同 crate inherent impl 保持 scope 不越界）。

## 4. T4 源码身份重钉（7 行 / 4 文件；LF 归一化口径）

S2b 的 7 文件重钉面里，本片实测**只有 4 个文件漂移**（改动的源文件：header、cpp、
`src/io/flex.rs`、`src/model/flex.rs`）；`src/physics/flex.rs` 未改，
`fixtures/precision/t4-current-kernels.json` / `g01-state-packing.json` /
`docs/evidence/t4-g03-retest/rigid-rotate-recapture/previous-kernels.json`
经全量测试确认**无本片文件的身份行**、未触碰（卡面 3 条 docs scope 中该条为“预留”而非必改）。

逐行 old→new（`sha256(LF 归一化字节)`）：

| 身份 | old | new |
| --- | --- | --- |
| `include/mjwarp_native_probe.h` | `e40aff139ec2af3e67e49a65c922bacd80672a16c4826b7c03788273042c7adf` | `ebaf3c4a087bded8e5164543a634d258a4e68416edcef532fffd3499f46e24dc` |
| `native/model_probe.cpp` | `f52728543ae9d44d7c3fbe948645c0ab8572c5c15a1c5efe90a607d58150ff28` | `09bba512abcdfce8fbf2201d9482c4d0efe0e4db18fefe89b0bce0271b0ed639` |
| `src/io/flex.rs` | `f74256b311642e1ac17c74b8776291a37e3a17bba6c0f0b23a6f6b7c35ad9ce4` | `a54db08142b01cfe147d8e8147b82cb616bdd4830e6fadd11d39682360577b32` |
| `src/model/flex.rs`（单文件） | `477355d8a47b1a3350a30c167f3b02683dc4fca7f42bf2e94e24ac5828ca2cb7` | `8dd9f7ee1e59b70c3c804be843424f906cdc59b37590d7f89153983ada946c7c` |
| `src/model/flex.rs`（组合摘要：`model/flex.rs`+`flex/{edge,face,input,position}.rs` 以 LF 归一化 `\n` 连接） | `9834e3a0931abfe58f10050ee5623a0951e3927643c06a2ab5ae3f4b24a99e25` | `56a75a7ba559eb368a40dce6e3d2ed8155b8e81d5cf8ab2862a616f762431898` |

替换位置：`fixtures/precision/g01-inputs.json`（header/cpp/io_flex/model_single 4 行）、
`fixtures/precision/g01-packing-routes.json`（model_single 1 行）、
`docs/evidence/t4-g03-retest/flex-internal/map.json`（model 组合 1 行）、
`docs/evidence/t4-g03-retest/flex-matrices/map.json`（model 组合 1 行）= **7 行**。
重钉前后 FAIL/PASS 原始输出：`repin-before-fail.txt`（4 个测试：source drift header /
model/flex.rs / 组合“product source drift”）与 `repin-after-pass.txt`
（`t4_input_precision` 36 passed/0 failed/12 ignored；`t4_identity_live_check` 2 passed/0 failed）。
**不改上游 pin、数值参考、判据、容差、断言逻辑**。

## 5. 验证

1. **门禁（默认特性）**：fmt / clippy / clippy-cuda / unit 全过；unit **1298 passed / 0 failed /
   398 ignored**（与 S3a 后基线持平：本片新增测试在 `native-model-probe` 特性下，默认门禁只编
   设备无关的 model 结构，无新增默认测试）。
2. **io 单测（`--features native-model-probe`）**：`io::flex::topology` **5 passed / 0 failed /
   1 ignored**（`native-topology-tests.txt`）：转换与计数、7 类长度变体拒绝、坏计数/负地址拒绝、
   空拓扑、与 render 联合宽度校验（含 4 类不匹配拒绝）。
3. **原生冒烟（离线，仅 MuJoCo 3.12 DLL，无 GPU）**：`captures_flex_topology_from_flex_fixture`
   **1 passed**（`native-smoke.txt`）：`fixtures/flex-position/flex-tree.mjb` → nflex=4、
   nflexelem=150、nflexelemdata=588、nflexshelldata=388；逐 flex 校验元素宽度 = `dim+1`
   （1D→2、2D→3、3D→4）与壳宽度 = `dim`（1D→1、2D→2、3D→3），地址链与合计一致。
4. **身份**：`t4_input_precision` 36/0/12、`t4_identity_live_check` 2/0（见 §4 PASS 日志）。
5. **零设备改动**：`git diff` 不含 `src/render/kernel/cuda/` 与任何 `.cu`（卡面 `unchanged`）；
   未跑 GPU、未开窗口、未生成参考。文件行数全部 ≤ 500（478/473/459/386/174）。

## 6. 边界与遗留

- 本片只交数据层：不接设备逻辑、不建 BVH、不宣称 2D/3D flex 渲染可用；S3 的 2D/3D 机制接真实
  数据预计在 S3c（依赖本片）。
- `fixtures/` / `docs/evidence/` 的改动**仅限身份字面量**（4 个 fixture + 2 个 map 共 7 行），
  无数值期望/容差/参考语义变化；`t4-current-kernels.json`、`g01-state-packing.json`、
  `previous-kernels.json` 未触碰（全量测试验证）。
- 原生冒烟依赖受信 MuJoCo 3.12.0 DLL（`MJWARP_MUJOCO_DLL`）与 `native-model-probe` 特性；
  属证据命令，不加入默认门禁（与 S2b 同口径）。
