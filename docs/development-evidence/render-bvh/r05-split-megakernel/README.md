# R05 拆分①：`megakernel.rs` 设备上传段纯移动（490 → 406 行）

任务：`2026-10-28-render-bvh-r05-split-megakernel`（P8 渲染与 BVH；p57；owner Backend）
拆分前树 HEAD：`00ed9efb3ac36e716d1e2ddaefecf956922ff9ca`
阶段：**纯移动完成**；S2 前置（`megakernel.rs` 余量 94 行，S2 +10~20 后仍 ≤426 ≤ 500）。

## 1. 拆分前后行数（UTF-8 行数口径，`find /c /v ""` 实测）

| 文件 | 拆分前 | 拆分后 | 说明 |
| --- | --- | --- | --- |
| `src/render/kernel/megakernel.rs` | **490** | **406** | 抽出 L203-344（设备上传段 142 行）；加调用+输入构造+`let u` 引用+`#[path]` 子模块声明后净 −84 |
| `src/render/kernel/megakernel_uploads.rs`（新） | — | **384** | `UploadInputs` + `MegakernelUploads`（80 字段）+ `upload_megakernel_arrays` |
| `src/render/kernel/mod.rs` | 474 | **474** | **零改动**（见下：子模块用 `#[path]` 在 `megakernel.rs` 内声明） |

> **子模块声明方式（满足卡 scope）**：本卡 files 只含 `megakernel.rs` + 新 `megakernel_uploads.rs` + 证据目录，
> `src/render/kernel/mod.rs` 不在 scope 内。经 Arch 确认，采用先例（`lighting.rs` 同款）：
> 在 `megakernel.rs` 依赖区之后写
> `#[path = "megakernel_uploads.rs"] mod megakernel_uploads;`（私有 mod），
> **父模块 `kernel/mod.rs` 一行不动**；新文件内 `super::launch`/`super::megakernel_arrays`
> 相应改成绝对路径 `crate::render::kernel::{launch,megakernel_arrays}`（子模块父级由 `kernel` 变为 `megakernel`）。
> 这 5 行（2 行注释 + 属性 + mod + 空行）即 `406` 与 `401` 的差。

> 与批准行级预算的差异（如实记录）：批准方案预计 `megakernel.rs ≈345`、新文件 `≈170`。实际 406/384——
> 原因：`MegakernelUploads` 需携带 **80 个 `TransferBuffer<T>` 句柄**（每字段类型不同，无法用数组/泛型收敛），
> 结构体与构造合计 ~200 行；`megakernel.rs` 侧保留 34 行 `UploadInputs` 构造，且 args 段引用改为 `u.*` 短前缀
> （若用 80 字段解构需 ~96 行，若用 `uploads.*` 长前缀则 fmt 折行使文件涨到 477）。目标 ≤430 已达成（406）。

## 2. 纯移动证据（Chief 硬要求，逐条）

1. **PTX/source sha 与拆分前完全一致**（同一内核编译门 `megakernel_source_compiles_under_render_gate`）：

| 项 | 拆分前（HEAD `00ed9efb`） | 拆分后 | 一致 |
| --- | --- | --- | --- |
| `source_sha256` | `2d2c27df985882c2b571514579dff19f9103bfc903ec786db42a592ce1bd8c2a` | 同左 | ✅ |
| `ptx_sha256` | `e955bb4b0280228639f3e544b459873c83d1dae672662a43a96cccc8dabc4ffa` | 同左 | ✅ |
| `ptx_bytes` | 258 660 | 258 660 | ✅ |

2. **公开 API 零变化**：`MegakernelRenderer` 的 pub 方法（`new`/`kernel`/`render`）签名不变；`kernel/mod.rs` **零改动**（子模块经 `#[path]` 在 `megakernel.rs` 内私有声明），无新 pub 导出。
3. **`MEGAKERNEL_SLOTS` 81 槽签名零变化**：槽表（`src/runtime/launcher/render/args.rs`）未改动；`render_launch_abi`/`render_lighting_kernel` 的 81 槽断言全绿。
4. **`MEGAKERNEL_SOURCE` 与 CUDA 源拼接零变化**：`megakernel_uploads.rs` 是 Rust 侧上传模块，**不参与** `include_str!` 源码拼接（PTX sha 一致为直接证据）。
5. **测试断言零改动**：无测试文件改动；`cargo test --locked` **1069 passed / 0 failed**（与拆分前一致），含 S7b1/S7b2/S7b3/S8 位等回归（`logs/test.log`）。
6. **`git diff` 性质**：`megakernel.rs` 删除 L203-344 段、新增 34 行输入构造 + 1 行 `let u` + 5 行 `#[path]` 子模块声明、args 段引用改为 `u.*`（引用改名）；新文件为原段内容（仅 `self.session`→`session`、`context`→`inputs.context` 等引用改名 + 两条 import 改绝对路径）；`kernel/mod.rs` **零改动**；imports 调整（`bool_words`/`flags`/`pack_blob`/`span_addr`/`upload` 移入新文件）。

## 3. 与批准行级方案的逐项对照

| 批准项 | 实际 |
| --- | --- |
| 抽出 L215-347（上传段 133 行）+ L203-213（纹理段 11 行）≈145 行 | 抽出 **L203-344（142 行）**：L345-347 的 `rgb_shape/depth_shape/seg_shape` 属 args 段输入，保留在原文件（3 行），边界微调 |
| 新文件 `≈170` 行（`MegakernelUploads` ~65 + 函数 ~105） | **384 行**：结构体 80 字段（~105 行）+ 函数（~250 行，原段逐行 + 引用改名） |
| `megakernel.rs ≈345` | **406**（原因见 §1 注） |
| 子模块声明位置（批准稿隐含 `kernel/mod.rs`） | `kernel/mod.rs` 不在卡 scope → 改 `#[path]` 在本文件内声明（Arch 确认的先例做法），`mod.rs` 零改动 |
| 零行为变化 | PTX/source sha 一致 + 1069 测试 + clippy/fmt 全绿 |

## 4. 模块源与指纹

- `normify module source mjwarp.render --add src/render/kernel/megakernel_uploads.rs`：**36 → 37**；
- 指纹过期警告由本任务 `normify done` 刷新；收口后 `normify validate --fingerprints` 错误 0。

## 5. 红线与不变量（零改动）

`render.cu` **499** / `launch.rs` **500** / `cast.rs` **494** / `model/render.rs` **500** / `camera_output.rs` **499**：均未改动；`MEGAKERNEL_SLOTS`=81、`DEVICE_SOURCES`=28（collision 侧）、入口=6、`CollisionArgs` 504 B、`MEGAKERNEL_SOURCE` 拼接顺序均未改动；`args.rs` 494 未改动。

## 6. 验证

- `cargo fmt --check` → 通过（`logs/fmt.log` 空）
- `cargo clippy --all-targets --locked -- -D warnings` → 通过（`logs/clippy.log`）
- `cargo test --locked` → **1069 passed / 0 failed**（`logs/test.log`，171 185 B；各测试二进制合计 1069）
- 内核编译门：`source_sha256=2d2c27df…`、`ptx_sha256=e955bb4b…`、`ptx_bytes=258 660`（`logs/kernel-gate-post.log`，与拆前一致）
- 行数实测：`logs/lines.log`

## 7. 限制

- 本片为纯移动，**无行为/数值/判据变化**；S2 仍待拆分②（`cast.rs`）后开写。
- `megakernel_uploads.rs`（384）与 `megakernel.rs`（406）均 ≤500 行棘轮；S2 的 `hfield_bvh_id` 槽填充预计改 `megakernel.rs`/`megakernel_uploads.rs` 共 +10~20 行（目标 ≤426）。
