# 渲染/BVH S7a：渲染发射 ABI、W15 编译与 f32 渲染门禁 · 证据

任务：`2026-10-24-render-bvh-s7a`（plan S7a，依赖 S2b/S5/S6b，均 verified）。
参考：冻结上游 MJWarp 3.15 `71da24d956378a87a703b6e1442b13aec0c4ac29` 的
`_render_megakernel` 真实签名（Warp 1.15 生成 CUDA 源）与 W15 头树（`include/warp-headers`）。

## 1. 范围与结论

本片只做发射 ABI / 编译 / 参数 round-trip，不含像素内核（S7b）。

| 验收（plan S7a） | 结果 |
| --- | --- |
| 约 83 参的打包/对齐/拒绝面（13 类数组元素 + 标量 int32/bool/uint64 + 4 输出） | **通过**：冻结上游签名 **81 槽**（1 bounds + 73 数组 + 7 标量），13 类元素宽度/对齐逐项核对；数量/种类/对齐/容量/上下文/线程块在发射前拒绝 |
| NVRTC + W15 头树编译；Render recipe 冻结；PTX f32-only 门禁与 entry | **通过**：`WARP_1_15_RENDER`（fmad=true、fast-math=false、C++17、compute_70、`PrecisionGate::F32Render`）；W15 三头文件编译出 25,996 B PTX，entry 正确，`tex.2d` 存在；渲染门禁精确放行 2 条 W15 常量后通过 |
| session 同流/同上下文仅 crate 可见；CUtexObject 只读句柄；句柄存活 + drop | **通过**：`TransferSession::{stream_ref,run_crate,span_context}` 均 `pub(crate)`，公共 API 未变；`TextureHandle2d` 只读；kernel 读回 `tex/width/height/channels` 与句柄一致；5 轮 1,200,000 B 数组 create/drop 显存零增长 |
| 登记 `mjwarp.render.kernel` 叶子模块；不实现像素内核；不改 S2–S6 证据；新文件 ≤500 行 | **部分**：源码登记在 `mjwarp.render`（见 §7 偏离说明）；其余全部通过 |

测试：GPU 2 passed / 0 failed（0.45 s），NVRTC/W15 2 passed，主机 4 passed / 2 ignored（ABI 签名/元素/拒绝面/recipe），
lib `render::` 42 passed（含 8 条渲染门禁单测）。GPU 窗口：RTX 4070 Ti SUPER / 596.36 / SM89，
开窗 2,870 MiB → 跑完 **2,870 MiB**（桌面基线，无 CUDA 计算进程）。

## 2. 冻结上游签名（81 槽）

从 S7a 预检缓存的 Warp 1.15 生成源
`wp__build_megakernel__locals___render_megakernel_ff1db96d_31afbcf/*.cu` 的 `extern "C"` 签名逐参抄录：
1 个 `wp::launch_bounds_t<2>` + 73 个 `wp::array_t<T>` + 7 个标量（4×int32 + 1×int32 `ray_base` +
1×uint64 `bvh_id` + 1×uint64 `splat_bvh_id`），其中 4 个输出为 `rgb_out(u32)`、`aa_accum_out(vec3f)`、
`depth_out(f32)`、`seg_out(vec2i)`。`MEGAKERNEL_SLOTS` 逐槽声明，主机测试核对数量与类覆盖。

元素宽度/对齐（W15 device C++ static_assert 口径，S7a 预检与集成测试双重钉死）：
`i32/u32/f32`=4/4、`u64`=8/8、`bool`=1/1、`vec2/vec2i`=8/4、`vec3/vec3i`=12/4、`vec4/quat`=16/4、
`mat33`=36/4、`texture2d_t`=24/**8**、`array_t<T>`=56。渲染侧 `RenderArray`（56 B `repr(C)`、`unsafe impl DeviceRepr`）
与既有 `WarpArray` 做 strides/形状 parity。

## 3. 渲染 f32 门禁（A′ 裁决）

`WARP_1_15_RENDER` 的 `PrecisionGate::F32Render` 由 `src/render/kernel/{mod,launch}.rs` 执行；
`compile_render_artifact` 与 `RenderKernel::compile` 是唯一编译入口，均先过门禁再加载模块，无公开绕过路径。

### 3.1 精确放行的两条 W15 常量（PTX 原文）

```text
.global .align 8 .f64 _ZN2wp11_svd_configIdE11SVD_EPSILONE = 0d3D719799812DEA11;
.global .align 8 .f64 _ZN2wp11_svd_configIdE17QR_GIVENS_EPSILONE = 0d3D719799812DEA11;
```

来源：W15 原样复制的 `svd.h` 中 `template <> struct _svd_config<double>` 的两条
`static constexpr double` 成员；`builtin.h` 无条件包含 `svd.h`，而 `array.h`/`bvh.h`/`texture.h`
又都包含 `builtin.h`，因此任何渲染内核的 PTX 都会带这两条声明。

**这是 W15 头文件自带的死数据常量，不被任何 f32 运算使用；属渲染门禁的精确放行，不是容差放宽。**
放行条件逐字节：符号名 + 字面量 `0d3D719799812DEA11` + 行尾 `;` 全等，且各只允许出现一次；
其他任何 f64 全局声明（不同符号/不同字面量/第三条）、f64 算术、f16/f16x2/bf16/bf16x2/tf32 一律拒绝；
`.reg .f64` 仍按物理门禁口径只接受 CUDA-12.8 sinf/cosf 整数角归约三元组。

### 3.2 门禁单测（8 条，主机）

`src/render/kernel/mod.rs` 的 `render::kernel::tests`：①两条都在→通过；②第三条未知 f64 全局→拒绝；
③符号差一个字符（`_E11SVD_EPSILON`）→拒绝；④字面量改（`…DEB11`）→拒绝；⑤无归约三元组的 `.reg .f64`→拒绝；
⑥`f16`/`mul.f64`→拒绝；⑦重复/位置错误→拒绝；⑧`WARP_1_15_RENDER.gate == F32Render` 且标注 `f32-render`。
集成测试另有 W15 真产物通过 + 注入第三条 f64 全局必须被拒。

### 3.3 W15 编译产物

| 项 | 值 |
| --- | --- |
| 源（含 `builtin.h`+`bvh.h`+`texture.h`、`WP_NO_CRT`/`WP_NO_BFLOAT16`、调用 `wp::texture_sample`）sha256 | `0a319b89585e67d74c7c1e63d6e03f3903582221e62c58d6bb8b7c6654126a5d` |
| PTX 字节 | 25,996 |
| PTX sha256 | `d075f329132b5ffec750f004682fde52236880c7a85e1fde641edcedefba501e` |
| 精度标签 | `precisionGate: f32-render` |
| 头树 | `include/warp-headers`（36 文件，内容进产物键）；**无需 CUDA SDK include**（`WP_NO_CRT` 下 crt.h 自带 Jitify 替代） |

## 4. 发射前拒绝面

`RenderSignature::new` 拒绝空槽位/首槽非 `launch_bounds_t<2>`/非首槽边界；
`RenderPlan::plan` 校验数量、逐槽种类、数组描述符（数据指针、1..4 维、连续步长、元素对齐、容量）、
跨缓冲同会话上下文、线程块=签名规则；`plan.launch` 复核数量、会话标识、句柄与流同上下文。
错误类型：`RenderArgCount`、`RenderArgKind{slot,expected,actual}`、`RenderArrayLayout`、`RenderArrayAlign`、
`RenderBounds`、`Capacity`、`ContextMismatch`、`ModuleContext`。

## 5. 参数 round-trip（GPU）

探针内核逐类读取实参并把位型写入 4 个输出（u32/u64/vec2i/f32），主机逐字段比对：

- 13 类数组：i32(4)/u32(1)/u64(1)/bool(4)/f32(4)/vec2(2)/vec3(2)/vec4(1)/vec2i(2)/vec3i(1)/mat33(1)/quat(1)/texture2d_t(1)；
- 3 类标量：int32 `7`、uint64 `0x1122334455667788`、bool `true`；
- 4 输出：u32 27 个语义值、u64 2 个、vec2i 1 个、f32 2 个——全部逐位相等（含 f32 位型与纹理字段）；
- 两次发射只加载 1 个模块（`module_loads=1`），第二轮走模块缓存。

## 6. 纹理句柄存活与释放（GPU）

`TextureObjects::handle(0)` → `TextureHandle2d { tex, width, height, channels }`（只读视图，不转移所有权）；
`Texture2dArray` 把句柄编码为 24 B `texture2d_t` 设备数组（会话缓冲只支持 f32/f64/i32/u32，故按 6×u32 打包，
与 C++ 布局逐字节一致）。实测：`tex=0x1`、`width/height/channels=2/2/4`，kernel 读回与句柄一致；
5 轮 1,200,000 B 数组 create/drop：基线 free `15,809,380,352 B`，结束 `15,809,380,352 B`（零增长，drop 释放到驱动池）。
`TextureObjects` 仍负责销毁 `CUtexObject`/`CUarray`（S4 路径）。

## 7. 偏离与阻塞（需 Arch 复核）

**④ `mjwarp.render.kernel` 叶子模块登记被 normify 结构规则阻塞**：
`normify module add mjwarp.render.kernel` 报「父模块 mjwarp.render 声明了 API；容器不能有 API，先把 API 移到子模块」；
手动建模块文件后 `normify validate` 报 7 个结构错误（`api/non-leaf`、`dep/from-api-non-leaf`）。
把 `mjwarp.render` 的 API 迁移到子模块属架构治理任务，超出 S7a files 与「不做像素内核」范围，故本片改为：
- `src/render/kernel/{mod.rs,launch.rs}` 源码登记到现有 `mjwarp.render` 模块（`normify module source`，25 个 source）；
- `src/runtime/launcher/render.rs`、`render/args.rs` 登记到 `mjwarp.runtime`（75 个 source）；
- `normify validate` = **0 错误**、5 条既有 warning；指纹在 done 时刷新。
建议另立治理任务做 `mjwarp.render` API 拆分后再建 `mjwarp.render.kernel` 叶子。

其它说明：
- 渲染配方冻结 `use_fast_math=false`（确定性优先）；上游 `create_render_context` 的 `use_fast_math` 默认 true
  属 S7b 内核选项，本片不引入 fast-math 变体，S7b 需按参考决定是否增设变体（已记入未满足项）。
- `Texture2dArray` 空句柄列表当前拒绝（上游纹理数组至少 1 项）；天空盒/3D/mipmap 不在本片。

## 8. 门禁

| 门禁 | 结果 |
| --- | --- |
| fmt | 通过 1259 ms |
| clippy | 通过（`--all-targets`，0 警告） |
| clippy-cuda | 通过（`--all-targets --features cuda-probe`，0 警告） |
| unit | 通过，**532 过 0 败**（默认 feature；渲染门禁/ABI 用例在 cuda-probe 下另跑） |

GPU 命令：

```text
$env:CUDA_PATH = (Resolve-Path 'target\toolchains\cuda-12.8.1').Path
$env:PATH = "$env:CUDA_PATH\bin;$env:PATH"
cargo test --locked --features cuda-probe --test render_launch_abi -- --ignored --test-threads=1 --nocapture
cargo test --locked --features cuda-probe --lib render::kernel -- --ignored --test-threads=1 --nocapture
```

## 9. 产物

| 路径 | 行数 | bytes | sha256 |
| --- | --- | --- | --- |
| `src/runtime/launcher/render.rs`（新） | 211 | 7,337 | `4e38c5ed059f4d3a86d06d44751f04e3a8ecd11feb8c37995553aa91d864c901` |
| `src/runtime/launcher/render/args.rs`（新） | 494 | 14,078 | `c89124071b787e85d8b904816d055bcf97624aabae9302ca8098281677186f5c` |
| `src/render/kernel/mod.rs`（新） | 310 | 11,746 | `e13cc4c7bbecbf7abf511f9ca7077f15f39c3123b6b01f180c89fb9be9ed52b1` |
| `src/render/kernel/launch.rs`（新） | 500 | 20,677 | `04c8617c84f6122272cfb5e15d28cf5b877cb329a66936ac21b82b2e7ab5217a` |
| `tests/render_launch_abi.rs`（新） | 302 | 11,629 | `fba0b6df8c00d5bd638e311a2f2387b40a5e6a09db22fdc023cf0207eba35c66` |
| `src/runtime/launcher/recipe.rs`（改） | 359 | 11,937 | `3f1ad960de35fe665863e35ac8b76c450cf4cb77fb79a2341b654443f0d0215c` |
| `src/runtime/launcher/mod.rs`（改） | 250 | 10,984 | `425994efe1ecae29c3035978de85cbee5e3a9442968c7828cb1fd957cb6fea22` |
| `src/runtime/transfer/session.rs`（改） | 240 | 8,062 | `8ab4e9ceca9a03653daf8c499b04eb3be8b6bc9c2f9d8eb90ef667538ab9b71c` |
| `src/render/texture/cuda.rs`（改） | 323 | 11,511 | `0d678b1d6ab296fdcf722f42fc94f6ac7cd3abce8b61837269f03bc4a6305d11` |
| `src/render/mod.rs`（改） | 370 | 11,440 | `5b83a8a99cdcc3b586916675244cfbf6f52d2fbe853f7f498fe7607560edb298` |

新文件行数全部 ≤500；既有文件均未超限（最大 494）。行数口径为 LF（`wc -l` / `ReadAllLines`）。

## 10. 未满足项与 deferral

- **像素渲染内核未实现**：RGB/depth/seg 着色、AA resolve、阴影/天空盒/聚光灯、纹理采样求值属 S7b；
  本片只交付 ABI/编译/门禁/round-trip，`tex.2d` 仅证明采样路径可编译。
- `mjwarp.render.kernel` 叶子模块（见 §7 阻塞）待治理任务。
- 上游 `use_fast_math=true` 的渲染变体未引入（S7b 决定）。
- 多 world / 全网格 megakernel 实参填充由 S7b 完成；本片不发射真实 megakernel。
- 未跑全量 G01（fast-loop 口径：本片主机/GPU 用例 + 四门禁）；未改 S2–S6 证据与夹具。
