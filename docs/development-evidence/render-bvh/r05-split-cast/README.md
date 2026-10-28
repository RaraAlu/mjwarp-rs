# R05 拆分②：`cast.rs` 的 `MeshDevice` 纯移动（494 → 411 行）

任务：`2026-10-28-render-bvh-r05-split-cast`（P8 渲染与 BVH；p57；owner Backend）
拆分前树 HEAD：`4a5a84498ce2e4022f4076f297198581e036ee0b`
阶段：**纯移动完成**；S2 前置（`cast.rs` 411 行 + S2 预计 +20~35 → ≈431–446 ≤450）。

## 1. 拆分前后行数（UTF-8/LF 行数口径，0x0A 字节计数实测）

| 文件 | 拆分前 | 拆分后 | 说明 |
| --- | --- | --- | --- |
| `src/render/kernel/cast.rs` | **494** | **411** | 抽出 L284-372（`MeshDevice` 类型文档 + 定义 + impl，89 行；块后 1 空行随块删除，共 −90）；插入 `#[path]` + `mod` + `pub use`（**+7 行**）；合计 **9 insertions / 92 deletions，净 −83**（494 − 83 = 411） |
| `src/render/kernel/cast_mesh.rs`（新） | — | **102** | 文件头 13 行 + 迁出块 89 行 |
| `src/render/kernel/mod.rs` | 474 | **474** | **零改动**（`#[path]` 声明放在 `cast.rs` 内） |

> 与批准方案差异（如实记录）：
> - 抽 **L284-372（89 行）** 而非 L293-372（80 行）：L284-291 是 `MeshDevice` 的类型文档注释，必须随类型一起迁出（留在 `cast.rs` 会错挂到下一个类型）。
> - `to_i32` **未随迁**：它同时被 `SceneDevice` 与 `CameraDevice`（均在 `cast.rs`）使用，故保留在 `cast.rs` 并改 `pub(super)`，新文件 `use super::to_i32;` 复用（批准方案二选一中的「`pub(super)` 复用」）。
> - `cast.rs` **411**（预算 ≈414，目标 ≤430 ✓）；新文件 **102**（预算 ≈90）。

## 2. 纯移动证据（逐条）

1. **迁出块字节级相同**：旧 `cast.rs` L284-372 与新 `cast_mesh.rs` 对应块 sha256 均为
   `0B68B0C930D1A09B15A819804AD61C3EE762D028F0C67C847690D44BF643C96E`、**3 296 B**
   ⇒ 设备 blob 布局、打包顺序、错误路径与布局文档逐字节未变。
2. **PTX/source sha 与拆分前一致**（同一内核编译门拆前/拆后各跑一次）：

| 项 | 拆分前（HEAD `4a5a8449`） | 拆分后 | 一致 |
| --- | --- | --- | --- |
| `source_sha256` | `2d2c27df985882c2b571514579dff19f9103bfc903ec786db42a592ce1bd8c2a` | 同左 | ✅ |
| `ptx_sha256` | `e955bb4b0280228639f3e544b459873c83d1dae672662a43a96cccc8dabc4ffa` | 同左 | ✅ |
| `ptx_bytes` | 258 660 | 258 660 | ✅ |

3. **公开 API 零变化**：`cast.rs` 保留 `pub use cast_mesh::MeshDevice;`，外部 `cast::MeshDevice` 路径不变
   （`renderer.rs`/`megakernel.rs`/`megakernel_arrays.rs`/`megakernel_uploads.rs` 的 import 与调用均零改动）；
   `tests/` 零改动（`git status --short -- tests/` 空）。
4. **`MEGAKERNEL_SOURCE`/CUDA 源拼接零变化**：`cast.rs`/`cast_mesh.rs` 是 Rust 侧打包，不参与
   `include_str!` 源码拼接（PTX sha 一致为直接证据）；未触碰任何 `.cu`。
5. **`git diff` 性质**：`cast.rs` 共 **4 个 hunk**（`git show 72a7de66 --unified=0` 实测）：
   - `@@ -17`（1−/1+）：import 由 `{MeshBvh, MeshNodeKind, NodeKind, SceneBvh}` 收窄为 `{NodeKind, SceneBvh}`（`MeshBvh`/`MeshNodeKind` 仅被迁出的 `MeshDevice` 使用）；
   - `@@ -22,0 +23,7`（+7）：2 行注释 + `#[path = "cast_mesh.rs"]` + `mod cast_mesh;` + 空行 + `pub use cast_mesh::MeshDevice;` + 空行；
   - `@@ -284,90`（−90）：89 行块（类型文档 + 定义 + impl）+ 块后 1 空行；
   - `@@ -465`（1−/1+）：`to_i32` 加 `pub(super)`。

   合计 **9 insertions / 92 deletions、净 −83**（494 − 83 = 411）。新文件 = 文件头 13 行 + 3 条 import + 原块 89 行原样。

## 3. 子模块声明方式（满足卡 scope）

卡片 files 只含 `cast.rs` + 新 `cast_mesh.rs` + 证据目录，`src/render/kernel/mod.rs` 不在 scope。
沿用拆分①先例：在 `cast.rs` 依赖区之后写
`#[path = "cast_mesh.rs"] mod cast_mesh;`（私有）+ `pub use cast_mesh::MeshDevice;`，
**父模块 `kernel/mod.rs` 一行不动**。

## 4. 模块源与指纹

- `normify module source mjwarp.render --add src/render/kernel/cast_mesh.rs`：**37 → 38**；
- 指纹过期由本任务 `normify done` 刷新；收口后 `normify validate --fingerprints` 错误 0。

## 5. 红线与不变量（零改动）

`render.cu` **499** / `launch.rs` **500** / `model/render.rs` **500** / `camera_output.rs` **499**：均未改动；
`DEVICE_SOURCES`=28（collision 侧）、入口=6、`CollisionArgs` 504 B、`MEGAKERNEL_SLOTS`=81、
`MEGAKERNEL_SOURCE` 拼接顺序均未改动；`args.rs` 494、`megakernel.rs` 406、`megakernel_uploads.rs` 384 未改动。

## 6. 验证

- `cargo fmt --check` → 通过（`logs/fmt.log` 空）
- `cargo clippy --all-targets --locked -- -D warnings` → 通过（`logs/clippy.log`）
- `cargo test --locked` → **1069 passed / 0 failed**（`logs/test.log`，171 236 B；各测试二进制合计 1069）
- 内核编译门拆前/拆后：`source_sha256=2d2c27df…`、`ptx_sha256=e955bb4b…`、`ptx_bytes=258 660`
  （`logs/kernel-gate-before.log`、`logs/kernel-gate-after.log`，两次完全一致）
- 四门禁 gate 全过：fmt 1611 ms / clippy 455 ms / clippy-cuda 405 ms / unit 11 624 ms（1069 过 0 败）
- 行数实测：`logs/lines.log`

## 7. 限制

- 本片为纯移动，**无行为/数值/判据变化**；S2 待本片 QA verified 后开写。
- `cast.rs`（411）与 `cast_mesh.rs`（102）均 ≤500 行棘轮；S2 的 hfield mesh blob 打包预计 +20~35 行（目标 ≤450）。
- 备选再抽 `SceneDevice`（原 L182-292）**未批，未动**。
