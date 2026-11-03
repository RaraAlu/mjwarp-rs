# R06-S2c flex shell/elem 数据层 QA 独立复核

任务：`2026-11-04-render-r06-s2c-flex-shell-check`（QA）。对象：`2026-11-04-render-r06-s2c-flex-shell-data`（verified），提交 `df6d74af`（23 文件 +706/−11）；复核时点 2026-11-04，HEAD `ebbba895`。零 GPU；只写本复核记录。

## 结论

**PASS-with-observations**：additive ABI/既有桶零改动、io 联合宽度校验、T4 7 行重钉、原生冒烟四计数、零设备 diff 全部通过。1 条观察（低）：`validate_topology_against_render` 信任调用方已校验 `render`（`flex_dim` 长度未在此函数内检查）；当前仅测试调用、正式接线应先 `FlexRenderFields::validate()`。不宣称 2D/3D 渲染可用、不宣称 NP01/NP03 通过。

## 一、additive 原生 ABI（A0）

- `include/mjwarp_native_probe.h` 与 `native/model_probe.cpp` 的 diff **删除行数 = 0**（只增不改）⇒ 既有 `mjwarp_flex_position_*` / `mjwarp_flex_render_*` 桶字节级零改动 ✓。
- 新增 `mjwarp_flex_topology_info`（**40 B / align 8**）：`schema/reserved` + 4 计数（`nflex@8`、`nflexvert@16`、`nflexelem@24`、`nflexshelldata@32`）。
- 新增 `mjwarp_flex_topology_targets`（**96 B / align 8**）：4 计数后 7 个字段/指针（`flex_elemadr@40`、`flex_elemnum@48`、`flex_elemdataadr@56`、`flex_elem@64`、`flex_shellnum@72`、`flex_shelldataadr@80`、`flex_shell@88`）。
- `static_assert` 覆盖 size/align/offset（7 项新增，含 `offsets … nflexshelldata == 32`、`flex_shell == 88`）；实现侧计数/指针/溢出/`SIZE_MAX` 预检，错误码 8 拒绝。
- `flex_vertadr`/`flex_vertnum` 未重复导（仍在 position 桶）。既有桶未改 ✓。

## 二、io/model 只读视图与联合宽度校验（A1）

- `src/io/flex/topology.rs`（471 行）：`NativeFlexTopologySnapshot`（capture/into_fields）+ `NativeModelProbe::flex_topology_snapshot`；非测试代码无 `unwrap/expect/panic!`（仅 FFI `unsafe extern "C"` 与调用处 `unsafe`）；本桶全整数，无 f64→f32。
- `FlexTopologyFields::validate()`（model/flex.rs）：4 计数溢出检查、7 个数组长度 = 对应计数（5 个 `nflex`、`flex_elem=nflexelemdata`、`flex_shell=nflexshelldata`）、全字段非负。
- `validate_topology_against_render()`（topology.rs:219）：先 `topology.validate()`；`topology.nflex == render.nflex`；`flex_elemadr` 首 0/连续/合计 = `nflexelem`；`flex_elemdataadr` 首 0/单调（`checked_sub`）/逐 flex `elem_data == elems × (dim+1)`；`flex_shelldataadr` 首 0/单调/逐 flex `shell_data == shells × dim`（**elem=dim+1、shell=dim** 与验收一致）；全部返回显式 `InputError`（`LengthMismatch`/`InvalidTopology`/`Overflow`），不 panic。
- 单测：`--features native-model-probe` 5 passed / 1 ignored（转换、7 类长度拒绝、坏计数与负地址、空拓扑、联合宽度 4 类不匹配）；本证据 `native-topology-tests.txt`。
- **观察（低）**：该函数对 `render.flex_dim[flex]` 的访问信任 `render` 已校验；若未来生产接线传入未 `validate()` 的 `FlexRenderFields`（`flex_dim` 短于 nflex）会 panic。当前唯一调用方是本文件单测（均已 `render_for` 构造合法值），且正式数据源来自 S2b 已验证快照；建议接线时在调用前显式 `render.validate()?`（或本函数内补 `flex_dim.len()` 断言）。

## 三、T4 身份重钉 7 行 / 4 文件（A2）

深度 JSON 对比（父提交 `fb785a88` vs `df6d74af`，4 个文件）：

| 文件 | 变化条数 | old → new |
| --- | --- | --- |
| `fixtures/precision/g01-inputs.json` | 4 | header `e40aff13…→ebaf3c4a…`、cpp `f5272854…→09bba512…`、`src/io/flex.rs` `f74256b3…→a54db081…`、`src/model/flex.rs` `477355d8…→8dd9f7ee…` |
| `fixtures/precision/g01-packing-routes.json` | 1 | `src/model/flex.rs` 同上单文件 |
| `docs/evidence/t4-g03-retest/flex-internal/map.json` | 1 | `model/flex.rs` 组合 `9834e3a0…→56a75a7b…` |
| `docs/evidence/t4-g03-retest/flex-matrices/map.json` | 1 | 同上组合 |

- 合计 **7 处 sha 字面量、4 文件、0 处其它字段变化**；每条 old = 父提交对应文件原始 blob LF-sha、new = HEAD 原始 blob LF-sha（单文件与 5 文件 `\n` 组合均独立复算一致）。
- 未触碰核实：`fixtures/precision/t4-current-kernels.json`、`g01-state-packing.json`、`rigid-rotate-recapture/previous-kernels.json` 不在本提交；`src/physics/flex.rs` 未改 ⇒ 与本片 README §4「无本片身份行」一致（卡面第 3 条 docs scope 为预留）。
- 重钉前 FAIL（4 个测试 drift）/重钉后 PASS（`t4_input_precision` 36/0/12、`identity_live` 2/0）原始输出在 `repin-before-fail.txt`/`repin-after-pass.txt`。

## 四、原生冒烟与门禁（A3/A4）

- `native-smoke.txt`：`flex-tree.mjb` → **nflex=4 / nflexelem=150 / nflexelemdata=588 / nflexshelldata=388**，`elemadr=[0,4,6,102]`、`elemnum=[4,2,96,48]`、`elemdataadr=[0,8,12,396]`、`shellnum=[2,2,80,48]`、`shelldataadr=[0,2,4,244]`；**1 passed / 0 failed**；离线（VsDevCmd + MuJoCo 3.12 DLL，无 GPU）。
- 门禁 runs：fmt/clippy/clippy-cuda passed；unit **1298 passed / 0 failed / 398 ignored**。
- 零设备 diff：`.cu` 0 文件、`src/render/kernel/cuda/` 0 文件；文件行数 ≤500（topology 471、model/flex.rs 478、io/flex.rs 473、cpp 386、header 174）。
- 证据 manifest **6/6 sha+bytes 通过**；README §4 逐行表与上述复算一致。

## 五、边界

零 GPU、零窗口、零参考采集；未改产品源码/判据/容差/fixtures（重钉为登记的 4 个身份文件）；本文件与检查卡状态是唯一写入。不宣称 flex 2D/3D 渲染可用、不宣称 NP01/NP03 通过。

## 六、闭合 addenda：S2c 修卡 `b4a7e827` 增量核（2026-11-04，QA）

本卡原观察「`validate_topology_against_render` 先 `topology.validate()` 但未校验 render 侧长度，短 render 向量可 panic」已由修卡 `2026-11-04-render-r06-s2c-topology-render-validate-fix`（提交 `b4a7e827`，卡 verified）闭合。QA 增量核四点全过：

1. **入口顺序**：`src/io/flex/topology.rs::validate_topology_against_render` 首部即 `render.validate()?`（在读取任何 render 字段之前），随后 `topology.validate()?`，再进入 `topology.nflex != render.nflex` 与按 flex 索引的宽度判定。
2. **新单测断言具体变体**：`rejects_short_render_vectors_before_indexing` 断言 `Err(InputError::LengthMismatch { field: "flex_dim", expected: 2, actual: 1 })`，并覆盖 `flex_rgba → { expected: 8, actual: 7 }`；不是只断 `is_err()`。
3. **既有 5 条拓扑单测断言未改**：该提交对 `topology.rs` 的 diff **删除行 = 0**（仅新增 2 行入口校验 + 1 条新单测）。QA 独立重跑 `cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features native-model-probe --lib io::flex::topology`（env：`MJWARP_MUJOCO_ROOT`/`MJWARP_MUJOCO_DLL` 指向 `target/toolchains/mujoco-3.12.0/package`）→ **6 passed / 0 failed / 1 ignored**（既有 5 + 新增 1；ignored 是需受信 MuJoCo DLL 的原生冒烟，按证据命令显式跑）。
4. **范围与行数**：源码只改 `src/io/flex/topology.rs`（+28 行），文件 **499 行** ≤ 500；`src/render/kernel/cuda/`、全部 `.cu`、`fixtures/` 零 diff；四门禁 runs 记录 fmt/clippy/clippy-cuda passed、unit **1303 passed / 0 failed / 398 ignored**。

补充说明：S3c（`181eaa7a`）目前**不调用**该函数（其 host 入口用 `FlexMeshAssembly::build` → `channel.topology.validate()`，且 `FlexMeshChannel` 不含 `FlexRenderFields`）；因此「S3c 调用前必须先 `render.validate()`」这一验收条件当前按「不适用即满足」处理——将来若 S3c/S3b 正式接线复用 `validate_topology_against_render`，入口已保证先校验 render 侧。

结论：原观察 **closed**；本卡结论仍为 PASS-with-observations（该条观察转为已闭合记录）。
