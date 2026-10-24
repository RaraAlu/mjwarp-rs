# 渲染/BVH S7a 独立复核（A′ 渲染门禁 f64 精确放行面 / 渲染发射 ABI / W15+NVRTC）

- 任务：`2026-10-25-render-bvh-s7a-independent-check`（QA；GPU 窗口内完成，GPU 实际占用约 2 s，W15 编译用例为 NVRTC/CPU）
- 复核对象：`a0d9af0`（S7a 功能提交，22 文件），基线 HEAD `74928dd`
- 日期：2026-10-25
- 结论：**验收面全部成立（0 阻断）**。A′ 精确放行面成立（真实渲染 PTX 全篇仅 2 个 f64 token，即 W15 两条死数据常量，符号+字面量逐字节；11 个注入/变异用例被独立参考实现与产品门禁**一致拒绝/放行**；物理门禁文件未被改动）；ABI 81 槽 / 13 类布局 / 56 B `RenderArray` 独立复算全部成立；GPU round-trip 与纹理句柄生命周期 2/2、host 4/4、render:: 42/0、532/0；10/10 产物哈希/字节/行数一致；`launch.rs` 恰 500 行。

## 1. A′ 渲染门禁的 f64 精确放行面（第一优先）

### ① 全 PTX 唯一性（成立）

QA harness 用产品渲染配方（`WARP_1_15_RENDER`，NVRTC，fmad=true、fast_math=false）重新编译 W15 头树探针，得到与交付**完全相同**的产物：

```text
ptx_bytes=25,996  ptx_sha256=d075f329132b5ffec750f004682fde52236880c7a85e1fde641edcedefba501e
source_sha256=0a319b89585e67d74c7c1e63d6e03f3903582221e62c58d6bb8b7c6654126a5d
```

QA 独立逐行扫描该 PTX 的全部 `.f64` token：

- `.global .align 8 .f64` 行 **恰 2 条**（第 32、33 行），且与放行清单**逐字节相等**：
  - `.global .align 8 .f64 _ZN2wp11_svd_configIdE11SVD_EPSILONE = 0d3D719799812DEA11;`
  - `.global .align 8 .f64 _ZN2wp11_svd_configIdE17QR_GIVENS_EPSILONE = 0d3D719799812DEA11;`
- 符号名、`= 0d...` 字面量、`;` 行尾均逐字节匹配；两条共享同一字面量 `0d3D719799812DEA11`；
- **非 `.global` 的 f64 行 0 条**（无 `.reg .f64`、无 sin/cos 归约三元组、无 `mul.f64`、无其它 f64 指令/声明）；
- 结论：这两条是**全 PTX 唯一的 f64 token**，是 W15 `svd.h` 随 `builtin.h` 带入的死数据常量，不参与任何 f32 运算。

### ② 注入/变异拒绝面（成立，独立重算 11 个用例）

QA 用与产品无关的 Python 参考实现（`qa_gate_abi_reference.py`，按允许清单语义独立编写）对同一批 PTX 文本逐条判定，并与产品 `check_render_ptx` 的判定 1:1 比对（**不读交付断言**）：

| 变体 | 期望 | QA 参考 | 产品 | 一致 |
| --- | --- | --- | --- | --- |
| 真实 W15 PTX | 放行 | 放行 | 放行 | ✅ |
| 注入第三条未知 f64 全局（`_ZN2wp11_rogue_configIdE...`） | 拒绝 | 拒绝 | 拒绝 | ✅ |
| 放行符号差一字符（`...EPSILONF`） | 拒绝 | 拒绝 | 拒绝 | ✅ |
| 字面量改动（`0d...DEA12`） | 拒绝 | 拒绝 | 拒绝 | ✅ |
| 放行常量重复出现 | 拒绝 | 拒绝 | 拒绝 | ✅ |
| 放行常量出现在非 `.global` 行（错位） | 拒绝 | 拒绝 | 拒绝 | ✅ |
| 注入 `f16` token | 拒绝 | 拒绝 | 拒绝 | ✅ |
| 注入独立 `mul.f64` 指令 | 拒绝 | 拒绝 | 拒绝 | ✅ |
| `.reg .f64` 但无 sin/cos 归约三元组 | 拒绝 | 拒绝 | 拒绝 | ✅ |
| `.reg .f64` + 合法 sin/cos 归约三元组（物理门禁同款例外） | 放行 | 放行 | 放行 | ✅ |
| 入口名缺失 | 拒绝 | 拒绝 | 拒绝 | ✅ |

- 产品错误类型与语义对应（`UnexpectedF64Global`/`DuplicateSvdConstant`/`MisplacedSvdConstant`/`UnaccountedF64Register`/`ForbiddenToken`/`MissingEntry`），全部在**编译产物入库前**拒绝；
- 11/11 QA 判定 == 产品判定 == 期望判定，0 分歧。

### ③ 物理门禁未被改动（成立）

```text
git diff a0d9af0^..a0d9af0 -- src/runtime/transfer/kernel/ptx.rs  → 空
```

- `check_physics_ptx` 源码与行为均未变；渲染门禁是**新增的独立函数**（`src/render/kernel/mod.rs::check_render_ptx`），物理门禁仍走原路径（`F32Physics`），两者门禁枚举分离（`F32Render` vs `F32Physics`）；
- 渲染配方 `WARP_1_15_RENDER`：`fmad=true`、`fast_math=false`、gate `F32Render`；Diagnostic 配方被拒绝用于 Render（独立核 `admit(Render)` 返回 Err）。

### ④ 结论口径（按要求）

**精确白名单放行 W15 死数据常量，不是容差放宽。** 放行对象是两条**逐字节完整匹配**的常量声明（符号+字面量+行尾），不是「允许任意 f64」「允许任何 `.f64` 行」「忽略 f64 差异」；任何第三条、改动一位、错位、重复或无归约解释的 f64 都拒绝；物理/数值路径的 `use_fast_math`、fmad、精度门禁均未放宽。

## 2. 渲染发射 ABI 独立复算（成立）

| 项 | 独立复算 |
| --- | --- |
| 槽位 | **81** = 1 `Bounds2D` + **73** 数组 + **7** 标量（另 4 个输出槽）；`RenderSignature::MEGAKERNEL` len=81、name=`_render_megakernel`、block=WarpGrid/256 |
| 13 类元素宽度/对齐 | i32/u32 4/4、u64 8/8、bool 1/1、f32 4/4、vec2f 8/4、vec3f 12/4、vec4f 16/4、vec2i 8/4、vec3i 12/4、mat33f 36/4、quatf 16/4、texture2d_t 24/8 —— **13/13 与预期一致** |
| 结构尺寸 | `size_of::<RenderArray>() = 56 B`；`Texture2d` 24 B / align 8（`Texture2d::BYTES=24`） |
| 每类槽数自洽 | 数组槽合计 73、标量槽合计 7，per-kind 计数与 81 槽一致（bool 6 + f32 8 + i32 22 + mat33f 2 + quatf 1 + texture2d_t 1 + u32 1 + u64 3 + vec2f 3 + vec2i 3 + vec3f 16 + vec3i 2 + vec4f 5 = 73；scalar int32 4 + uint64 2 + bool 1 = 7） |
| 写前拒绝面 | **10/10** 拒绝：数量不符、种类不符、首槽非 bounds、线程块规则、vec3 地址未对齐、texture2d 地址未对齐、容量不足、跨上下文、空形状、零维形状；错误类别与槽位信息逐项匹配 |

## 3. GPU / 测试独立重跑（成立）

- W15+NVRTC 用例原样重跑：**2 passed / 0 failed**（`source_sha256=0a319b89…`、`ptx_bytes=25,996`、`ptx_sha256=d075f329…`、`gate=f32-render`，与交付一致）；
- ABI round-trip（13 类数组 + 3 标量 + 4 输出逐位、两次发射 `module_loads=1`、CUtexObject 只读句柄往返 + 多轮 create/drop 零增长）：**2 passed / 0 failed**，纹理生命周期数字 `baseline=after_cycles=15,809,380,352 B`、`bytes_per_cycle=1,200,000`、`tex=0x1`，与交付一致；
- host ABI 用例：**4 passed / 0 failed**；`render::` 库内用例：**42 passed / 0 failed**；`cargo test --locked`：**532 passed / 0 failed**。

## 4. 证据 / 门禁 / 边界（成立）

- 交付回执 10 个产物 **10/10 sha256、bytes、lines** 与 git blob 一致（`launch.rs` **恰 500 行**，`--all-targets` 下 clippy 未破坏棘轮）；
- `git diff a0d9af0^..a0d9af0 -- src/runtime/transfer/kernel/ptx.rs` 为空（物理门禁未改）；
- `src/runtime/launcher/{render.rs,render/args.rs,recipe.rs}` 与 `src/render/kernel/*` 全部在 unchanged 列表内且未被本轮触碰；
- GPU：开始 `2,870–2,878 MiB` 桌面基线，GPU 用例 `2,878 → 2,871 MiB`，**结束 2,871 MiB**，无本任务 CUDA 计算进程。

## 5. 已知偏差（复核注明，不阻断）

- **裁决④ 模块登记偏差**：`mjwarp.render.kernel` 叶子模块被 normify 结构规则阻塞（父模块 `mjwarp.render` 已声明 API），S7a 把 kernel 源码登记进 `mjwarp.render`、launcher 进 `mjwarp.runtime`；`normify validate` 0 错误。Arch 已登记 backlog `2026-10-25-render-bvh-module-split`（S7b2 后处理），S7a 验收不因此重开。复核时以该已登记偏差为准，未当作缺陷。

## 6. 未满足项 / 限制

- 真实渲染 megakernel 与逐像素着色、AA 图像、阴影/天空盒/聚光灯、纹理/顶点法线/镜面/自发光求值属 S7b；本片只做发射 ABI 与 W15 门禁；
- `use_fast_math=true` 的渲染变体未实现（S7b 内容）；真实 megakernel 的多 world 参数属 S7b；
- W15 头树来源是 NVRTC 自带的 CUDA SDK include（`WP_NO_CRT` 提供 Jitify 替代），未覆盖其他 CUDA 版本；
- `texture2d_t` 设备数组按 6×u32 上传（会话只支持 f32/f64/i32/u32），与 C++ 24 B/align8 字节布局一致但语义类型是 u32；
- 未跑全量 G01（fast-loop 口径）。

## 7. 复核产物

- `w15-real.ptx` + `variant-*.ptx`：产品渲染配方编译的真实 PTX 与 11 个变异输入
- `harness-report.json`：产品门禁/ABI/recipe/rejection 原始判定
- `harness-compare.json`：QA 独立参考门禁逐条判定、f64 唯一性扫描、ABI 复算、拒绝面映射
- `qa_gate_abi_reference.py`：QA 的独立门禁参考实现（可重算）
- `artifact-check.json`：10 产物哈希/字节/行数（含 launch.rs=500）
- `w15-test.log`、`gpu-roundtrip.log`、`unit-tests-summary.txt`、`gpu-window.txt`
