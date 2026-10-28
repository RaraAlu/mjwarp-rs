# R05-S3b 零 mesh 场景 megakernel 发射修复（empty-mesh fix）

任务：`2026-10-28-render-bvh-r05-s3b-hfield-empty-mesh-fix`（proposed 由 Arch 预登记，Chief 批 A′ 后 start）。
本片**只修**「零 mesh（`nmesh=0`）场景下渲染发射被零长数组拒绝」这一产品缺口；
**不写任何渲染数值验收结论**，不宣称 hfield 渲染通路可用，不开 GPU 窗口，不重冻结 guards。

## 1. 范围与改动文件

| 文件 | 改动 | 行数（committed blob） |
| --- | --- | --- |
| `src/render/kernel/megakernel_uploads.rs` | 新增 `nonempty()` / `mesh_slot_shape()`；6 处 upload 用占位；新增 2 条纯 host 单测 | **446**（上界 ≤450） |
| `src/render/kernel/megakernel.rs` | 6 处 mesh 家族 shape 改 `&mesh_slot_shape(...)` | **412**（上界 ≤500） |

零改动（git 证据见 §7）：`src/runtime/**`、`src/render/kernel/cuda/**.cu`、`fixtures/**`、
`tests/support/render_bvh/**`（含 `hfield_scene.rs`）、`tests/render_hfield_reference.rs`、判据/容差/参考。

## 2. 根因链（三层代码 + 一条 ABI 限制）

1. `src/runtime/transfer/session.rs:176-186`：`upload()` 对空切片**不申请设备缓冲**（注释「空字段不申请设备缓冲」）→ `allocation=None, device_addr=0`。
2. `src/runtime/transfer/buffer.rs:60-67`：`launcher_span()` 在 `allocation.is_none()` 时返回 `InputError::InvalidDimension{field:"launcher_buffer"}`。
3. `src/render/kernel/megakernel_arrays.rs:97-102`：`array()` 把该错误包成 `LauncherError::Driver("s7b2-span: …")`；`span_addr()` 同源（:77-84）。
4. 附加硬限制：`src/runtime/launcher/render/args.rs:150-152`（`shape.contains(&0)` 拒绝）与 `:220-221`（`axis <= 0` 拒绝）
   ⇒ 零长字段不仅无设备跨度，**也不允许 shape [0]**；必须「1 元素占位 + shape ≥ 1」。

触发场景（签字 XML 的 3.12 dumper 产物，权威）：
`model-312.json`（hfield-only）与 `model-mixed-312.json`（hfield+box）均 `nmesh=0 / nmeshvert=0 / nmeshface=0`
（MJCF 里没有 `<mesh>` 元素），于是 `megakernel_uploads.rs` 里 mesh 家族 6 个槽的源数组长度为 0。
81 槽逐项枚举后**只有这 6 个槽为零长**（其余 75 槽：ngeom 1/2、nlight 2、ncam 2、nrender 2、rays 360000、mat 2/3、ntex 1、输出/AA 缓冲均非空）：

| 槽(0-based) | 槽名 | 构造处 | 期望 shape | 实际 len |
| --- | --- | --- | --- | --- |
| 22 | `mesh_faceadr` | `megakernel_uploads.rs:243-245` | `[u.nmesh]`（`megakernel.rs:276`） | 0（首失败点） |
| 23 | `mesh_normaladr` | `megakernel_uploads.rs:246-247` | `[u.nmesh]`（`:277`） | 0 |
| 24 | `mesh_normal` | `megakernel_uploads.rs:248-249` | `[len/3]`（`:278-282`） | 0 |
| 60 | `mesh_facetexcoord` | `megakernel_uploads.rs:250-252` | `[u.mesh_faces]`（`:319-323`） | 0 |
| 61 | `mesh_facenormal` | `megakernel_uploads.rs:253-254` | `[u.mesh_faces]`（`:324-328`） | 0 |
| 63 | `mesh_texcoord_offsets` | `megakernel_uploads.rs:265-266` | `[u.nmesh]`（`:335-339`） | 0 |

对照：槽 62 `mesh_texcoord` 的 dump 源也是 0，但其 upload 已有 2-float 占位（`megakernel_uploads.rs:256-263`，`len/2=1`），**本次保持原样不动**；
`mesh_vertadr`/`mesh_graph` 等模型字段没有对应内核槽，不产生零长绑定。

## 3. 为什么此前没有暴露

此前跑过的渲染场景（`scene-a`、`scene-shadow`、e2e 等）**都恰好有 1 个 `<mesh>`**；
hfield 两个场景是首次出现的 `nmesh=0` 场景。上游对 0 长度 `wp.array` 合法（同一签字 XML 已由 pinned 上游渲出 R05-S3a 参考），
我们的宿主/绑层层在零长数组上无法发射 —— 属**产品缺口**，不是测试装配 bug（`hfield_scene.rs::fields()` 逐字段忠实读取 dump）。

## 4. 修复内容

`megakernel_uploads.rs` 新增两个纯 host 函数（`pub(super)`，不涉及 `TransferSession`/CUDA）：

- `nonempty<T: Copy + Default>(values: &[T], fill: usize) -> Vec<T>`：空源补 `fill` 个零值元素，非空源**原样透传**。
- `mesh_slot_shape(count: usize) -> [usize; 1]`：`[count.max(1)]`（shape 各维 ≥1，规避 ABI 零维拒绝）。

6 处 upload 改为 `nonempty(...)`：`mesh_faceadr`/`mesh_normaladr`/`mesh_texcoord_offsets` 填 1×i32；
`mesh_facetexcoord`/`mesh_facenormal` 填 3×i32（Vec3I）；`mesh_normal` 填 3×f32（Vec3）。
`megakernel.rs` 6 处 shape 改 `&mesh_slot_shape(...)`（`nmesh`、`len/3`、`mesh_faces` 三种来源）。
容器核对：占位后 4B≤4B（i32×1）、12B≤12B（vec3/vec3i 各 1 元素），满足 ABI `required <= span.bytes()`。

新增 2 条 `#[cfg(test)]` 单测（无 CUDA、无 `TransferSession`，普通 `cargo test --locked` 可跑）：
① 空源 → 6 槽占位非空且元素宽度正确（I32=1×i32、Vec3I=3×i32、Vec3=3×f32）+ shape 各维 ≥1；
② 非空源 → 长度与内容逐项不变（钉住 `scene-a`/`scene-shadow`/e2e 的字节）。
注释写明「占位约定回归，非渲染数值验收」。

## 5. 零影响论证（有 mesh 场景）

- 占位分支**只在源为空时进入**；有 mesh 场景 source 非空 → 走原路径，字节与改动前一致。
- 内核只在 mesh geom（gtype 7/8）解引用 mesh 数组：
  `src/render/kernel/cuda/render_megakernel.cu:217-230` 顶点法线块整体在 `vertex_normal_gtype=(hit_geom_type==7||8)` && flag && `mesh_id>=0` && `face>=0` 内；
  `src/render/kernel/cuda/render_shade.cu:285-287` 只有 `gtype==7||8` 才调 `texture_mesh`（唯一解引用 `mesh_faceadr/mesh_facetexcoord/mesh_texcoord/mesh_texcoord_offsets` 的路径）。
- **kernel 不读 mesh 数组 shape**：`.cu` 中 `.shape` 仅 5 处 —— `dim.shape[0]`（`render_megakernel.cu:101`）、`cam_xpos.shape.dims[1]`（`:148`）、
  `rgb/depth/seg_out.shape.dims[1]`（`:172-174`）、`light_type.shape.dims[1]`（`:279`）；`nmesh`/`mesh_faces` 只喂实参 shape、不进内核标量。
- 不触碰 81 槽签名、`DEVICE_SOURCES=28`、`CollisionArgs` 504 B、`src/runtime`、任何 `.cu`。

## 6. 门禁与固定量（全部本机实测）

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --all-targets --locked -- -D warnings` | exit 0 |
| `cargo clippy --all-targets --locked --features cuda -- -D warnings`（clippy-cuda） | exit 0 |
| `cargo test --locked`（unit） | **1091 passed / 0 failed / 389 ignored**（134 个 `test result:` 行） |
| `cargo test --locked --no-run` | exit 0 |
| 内核编译门（`render_kernel_rgb -- --ignored megakernel_source_compiles_under_render_gate`） | exit 0；`source_sha256=72bc364c4b732e278396d2aebd09625872f6dcecba469a98a3e40aba04851746`、`ptx_sha256=97467d835e27107a4a0803678e194f4868fb552707543a544e270cbde8a9bd13`、`ptx_bytes=264015`；**改动前/后逐位相同** |
| `normify validate --fingerprints` | exit 0，**错误 0**（6 条既有结构告警，非本片引入） |

固定量表（unit 中既有断言覆盖）：81 槽签名（`megakernel_slots_signature_is_unchanged` 通过）、
`DEVICE_SOURCES=28`（`*device_sources_place*` 系列通过）、`CollisionArgs` 504 B（`src/runtime/collision_gpu/abi_tests.rs:41` 断言通过）。

## 7. 零改动 git 证据

`01e36a54..dce78b1f` 变更文件仅 3 个（2 源文件 + 本任务卡）；以下路径在该区间**零 diff**：

```
git diff --stat 01e36a54..HEAD -- src/runtime src/render/kernel/cuda fixtures tests/support/render_bvh tests/render_hfield_reference.rs
（空）
```

内核 `source_sha256`/`ptx_sha256`/`ptx_bytes` 与改动前逐位相同（见 §6），是「只动宿主 Rust」的独立证据。

## 8. 行数口径（LF 权威）

| 文件 | LF 行数（committed blob；`git cat-file -p | find /c /v ""` 与 `[IO.File]::ReadAllLines` 一致） | 上界 | 父提交行数 | 余量 |
| --- | --- | --- | --- | --- |
| `megakernel_uploads.rs` | **446** | 450 | 405 | **仅 4 行**（后续要改先拆） |
| `megakernel.rs` | **412** | 500 | 404 | 88 行 |

口径说明：Get-Content 在本环境（ANSI 解码 UTF-8 源文件）会少计行，其数值作废、不用于证据；
本片统一以 committed blob 的 LF 换行计数为准（2026-10-29 附录修正，Chief 指出）。
## 9. impact 记录

- `normify impact src/render/kernel/megakernel_uploads.rs` → `mjwarp.render`（active，源码 39，API 11）；
  依赖 `mjwarp.model.core`、`mjwarp.runtime.completion`、`mjwarp.math`、`mjwarp.diagnostics`；
  规则 `core-no-deprecated-target`、`boundary-render`。
- `normify impact src/render/kernel/megakernel.rs` → **无模块对应**（`src/render/kernel/` 下部分 .rs 未登记模块源的既有治理缺口，已有 backlog 卡 `2026-10-28-arch-volume-module-sources`；**本片不顺手登记**）。

## 10. 本地调试事实（**本地调试、非验收**；数字只作调试信息）

命令：`cargo test --locked --features cuda-probe --test <name> -- --ignored --test-threads=1 --nocapture`，
注入 `target\toolchains\cuda-12.8.1\bin` 到 PATH（与窗口 runner 同环境），按窗口 runner 实体顺序跑 9 条。

| 实体 | exit |
| --- | --- |
| `render_hfield_reference` | 101（**launcher 不再 panic**；完成比较后断言判据红） |
| `render_camera_rays` / `render_context_lifecycle` / `render_e2e` / `render_kernel_depth_seg` / `render_kernel_flags` / `render_kernel_rgb` / `render_r08_cameras` / `render_r10_multi_world` | 8/8 **exit 0** |

hfield 目标比较事实（判据数字，本地调试）：

| 组 | RGB | seg | depth cam0 | depth cam1 |
| --- | --- | --- | --- | --- |
| hfield-only | cam0 76800 / cam1 43200 像素，mismatched=0、max_byte_diff=0、beyond_1_lsb=0 | mismatch=0 | mismatch=0、max_ulp=0、beyond=0 | **mismatch=4423、max_ulp=7、beyond=20**（首差异 cam1 pixel=16913 x=113 y=70） |
| hfield-primitive | 同上，位级全等 | mismatch=0 | mismatch=0、max_ulp=0、beyond=0 | **mismatch=4389、max_ulp=7、beyond=20**（同一首差异像素） |

**本片不掩盖**：cam1 depth 判据红（4423/4389、max_ulp=7、beyond=20）是**独立未解缺陷**，
另立只读 RCA 片 `2026-10-28-render-bvh-r05-s3b-hfield-cam1-depth-rca` 处理；不调整判据/容差/参考，
不宣称 hfield 渲染通路可用，不把本次调试跑当作验收。

## 11. 前序 RCA 摘要（窗口 attempt1 invalid round；RCA 文件在 `target/` 不入库）

- 窗口 attempt1：`render_hfield_reference` exit 101，panic `s7b2-span: 传输输入错误：维度无效：launcher_buffer`
  （`tests/support/render_bvh/hfield_scene.rs:329` 的 `fire()`），8 回归 exit 0。
- 机制链与 6 槽枚举见 §2；判性为产品缺口（上游可渲零 mesh 场景、测试装配忠实、此前场景都有 mesh）。
- 修复方案预核曾比较 A（launcher 层哨兵，需改 `src/runtime` 且 shape[0] 仍被拒 → 不可行）、
  B（upload 分配 1 元素但 len=0，动 runtime 红线且不解决 shape → 不采用）、C（跳过槽，81 参固定 ABI 必错位 → 不可行）；
  最终 A′（本片）：megakernel 侧占位 + `count.max(1)` shape，采用模块既有占位约定。

## 12. 复现命令

```text
$env:PATH = 'D:\Rust\mjwarp-rs\target\toolchains\cuda-12.8.1\bin;' + $env:PATH
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo clippy --all-targets --locked --features cuda -- -D warnings
cargo test --locked
cargo test --locked --no-run
cargo test --locked --features cuda-probe --test render_kernel_rgb -- --ignored megakernel_source_compiles_under_render_gate --test-threads=1 --nocapture
cargo test --locked --features cuda-probe --test render_hfield_reference -- --ignored --test-threads=1 --nocapture
```

## 13. 局限

- 本片**零渲染数值验收结论**；hfield 目标的判据红（cam1 depth）未解决，窗口 attempt2 与 guards 重冻结全部暂缓。
- 本地调试跑是调试信息，不构成窗口证据；`target/render-bvh/r05-s3b-window/attempt1/` 原样保留未覆盖（attempt=1 已作废）。
- 未开窗、未重冻结、未改判据/容差/参考/fixture。

## 14. 收口

- 任务卡 `2026-10-28-render-bvh-r05-s3b-hfield-empty-mesh-fix` 已由 `normify done` 置 **verified**（本片无 gate 引用；四门禁与本机门禁数字见 §6 与 materials）。
- 提交链：代码 `dce78b1f`（2 源文件 + 本任务卡）→ 证据 `e262e682`（本目录）→ 本收口提交（卡状态 + 模块指纹 + README/manifest）。
- 本片**未开窗、未重冻结 guards、未改判据/容差/参考**；`target/render-bvh/r05-s3b-window/attempt1/`（attempt=1 无效轮）原样保留。
- 后续：hfield cam1 depth 判据红由只读 RCA 片 `2026-10-28-render-bvh-r05-s3b-hfield-cam1-depth-rca` 处理；重冻结与 attempt=2 窗口待该 RCA 有结论后再议。
