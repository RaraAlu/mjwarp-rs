# R05 拆分②（`cast.rs` 的 `MeshDevice` 段纯移动）独立复核

任务：`2026-10-28-render-bvh-r05-split-cast-check`（P8 渲染与 BVH；p56；owner QA）
被复核任务：`2026-10-28-render-bvh-r05-split-cast`（已 verified）
被复核提交：`72a7de66`（实现 + 证据，17 文件）+ `3e8628b6`（verified 收口，2 文件）
拆分前树：`4a5a84498ce2e4022f4076f297198581e036ee0b`；复核时 HEAD：`3e8628b6`
范围：只读复核 + 本证据目录；不开 GPU 窗口、不重跑窗口；不改代码/判据/夹具。

## 1. 结论

**verdict：pass / 阻断 0 / 观察 6**（O1–O2 建议级，O3–O6 记录级；均不阻断本片验收）。

复核口径是「证明行为零变化」，不是"能编译"。关键结论：

| 复核点 | 结果 |
| --- | --- |
| 迁出块字节一致 | `cast.rs` 旧 **L284-372**（89 行）与新 `cast_mesh.rs` **L14-102** 逐字节相同：sha256 `0b68b0c9…`、**3 296 B**（我自算，与实现声明一致） |
| 全量重构等价 | 旧文件经「import 收窄 + 插入 7 行 + 删 89 行块 + 1 空行 + `to_i32` 可见性」后**逐字节等于** HEAD `cast.rs`（16 378 B == 16 378 B） |
| PTX / source | 6 段 `include_str` 拼接 sha 自算 = 基线 `2d2c27df…`；CUDA 源逐字节未改；我自跑内核编译门 = `2d2c27df…`/`e955bb4b…`/258 660 |
| API / 对外路径 | `pub use cast_mesh::MeshDevice;` 1 处、`#[path]` 1 处、`mod.rs` blob 相同、4 个外部引用文件 blob 未改、公开项集合不变（6 项） |
| 行数 / 模块 | `cast.rs` **411**（≤430）、`cast_mesh.rs` **102**（≤500）、`mod.rs` 474 零 diff；模块源 **38**、`validate --fingerprints` 错误 0 |
| 回归 | 我独立复跑四门禁：fmt 0 / clippy 0 / clippy-cuda 0 / unit **1069 passed / 0 failed / 387 ignored**（132 result 行） |
| 红线 / 证据 | 红线行数全对；实现证据 9 文件、manifest **8/8 对 committed blob**、LF 无 BOM、禁词 0；提交 17+2 文件全在卡 scope |

## 2. 复核点逐条（数字全部我自己重算/自跑）

### 2.1 迁出块字节级一致（最硬）

- 旧 `cast.rs`（`4a5a8449`）**L284-372** 共 **89 行**（`MeshDevice` 类型文档 + 定义 + impl）；新 `cast_mesh.rs` 的 **L14-102** 同为 89 行。
- 我取两段**连续行字节**（含行尾 LF）自算：旧 `0b68b0c9…`/3 296 B，新 `0b68b0c9…`/3 296 B，**逐字节相等**。
- `cast_mesh.rs` = 文件头 13 行 + 迁出块 89 行 = **102** 行；import 为 `{MeshBvh, MeshNodeKind}` + `LauncherError`，第 12 行 `use super::to_i32;`。

### 2.2 `cast.rs` 全量重构等价（4 个 hunk，无第四类改动）

`git diff -U0 4a5a8449..HEAD -- src/render/kernel/cast.rs` 只有 **4 个 hunk**（removed 92 / added 9 = 净 −83）：

| hunk | 内容 | 我核 |
| --- | --- | --- |
| 1 | 旧 L17 `use …{MeshBvh, MeshNodeKind, NodeKind, SceneBvh};` → `{NodeKind, SceneBvh};` | 逐字核 |
| 2 | 旧 L22 之后**插入 7 行**（2 行注释 + `#[path]` + `mod cast_mesh;` + 空行 + `pub use cast_mesh::MeshDevice;` + 空行） | 逐行核（7 行，不是 9 行，见 O1） |
| 3 | 旧 **L284-373 删除 90 行** = 89 行块 + 块后 1 空行 | 前 89 行拼接 sha = 迁出块 sha；第 90 行为空行 |
| 4 | 旧 L465 `fn to_i32(…)` → `pub(super) fn to_i32(…)`（函数体不变） | 逐字核 |

- **全量重构验证**：`old[0:16] + 收窄 import + old[17:22] + 插入 7 行 + old[22:283] + old[373:]`，再替换 `to_i32` 行 → **逐字节等于** HEAD `cast.rs`（16 378 B）。⇒ 除上述 4 类外无任何字节变化（保留区零改动，含 `SceneDevice` 未动）。

### 2.3 PTX / source 零变化

- `MEGAKERNEL_SOURCE` = `concat!` 6 个 `include_str`；我按顺序拼接自算 sha256 = **`2d2c27df…`**（= 基线）；6 个文件相对 `4a5a8449` blob 逐字节相同。
- 我自跑内核编译门（NVRTC 纯编译、无 GPU）：`source_sha256=2d2c27df…` / `ptx_sha256=e955bb4b…` / `ptx_bytes=258660`，exit 0（`logs/kernel-gate.log`）。
- **边界**：未在拆前树上重跑该门（见 §5）。

### 2.4 API / 对外路径

- `cast::MeshDevice` 对外路径不变：`cast.rs` 内 **1 处** `pub use cast_mesh::MeshDevice;`；`pub struct MeshDevice` 已不在 `cast.rs`（定义迁入新文件）。
- `#[path = "cast_mesh.rs"] mod cast_mesh;` **1 处**、无 `pub`；`kernel/mod.rs` blob 与拆前**相同**且不含 `cast_mesh` 字样（父模块零改动）。
- 全部引用 `MeshDevice` 的文件：`cast.rs`/`cast_mesh.rs` 为改动本体；其余 **4 个**（`megakernel.rs`/`megakernel_arrays.rs`/`megakernel_uploads.rs`/`renderer.rs`）blob 相对拆前**未改**。
- 公开项集合（`pub struct/enum/fn` 名）拆前 6 项 = 拆后并集 6 项：`CameraDevice/CameraPose/MeshDevice/ModelRenderArrays/PoseError/SceneDevice`。
- `to_i32` 仍在 `cast.rs` 且改为 `pub(super)`（仅 kernel 模块内可见，crate 外不可见）；函数体逐字节不变；`cast.rs` 10 处（1 定义 + 9 调用）、`cast_mesh.rs` 12 处调用、经 `use super::to_i32;` 复用。

### 2.5 行数 / `#[path]` / 模块源

| 文件 | 行数（committed blob 的 LF 行数） | 要求 |
| --- | --- | --- |
| `src/render/kernel/cast.rs` | **411**（拆前 **494**，净 −83） | ≤430 目标达成，≤500 |
| `src/render/kernel/cast_mesh.rs`（新） | **102** | ≤500 |
| `src/render/kernel/mod.rs` | **474**（零 diff） | — |

- 行数算术自校：`494 − 89（块） − 1（块后空行） + 7（插入） = 411`，与实测一致。
- `#[path]` 私有子模块无重复定义；模块源 `render.md` = **38** 条且含 `cast_mesh.rs`；`normify validate --fingerprints` **错误 0**（警告 5 条为既有结构类）。
- 备选的 `SceneDevice`（原 L182-292）**未抽**：`SceneDevice` 的 struct 与 impl 仍完整在 `cast.rs`，且全量重构等价证明该区间零字节变化。

### 2.6 回归（我自己跑的四门禁）

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --check` | exit 0（`logs/fmt.log`） |
| `cargo clippy --all-targets --locked -- -D warnings` | exit 0（`logs/clippy.log`） |
| `cargo clippy --all-targets --locked --features cuda -- -D warnings` | exit 0（`logs/clippy-cuda.log`） |
| `cargo test --locked` | **1069 passed / 0 failed / 387 ignored**，132 result 行（`logs/unit.log`） |

### 2.7 红线与证据

- 红线行数（我的口径）：`render.cu` **499**、`launch.rs` **500**、`model/render.rs` **500**、`camera_output.rs` **499**、`args.rs` **494**、`megakernel.rs` **406**、`megakernel_uploads.rs` **384**、`megakernel_arrays.rs` **326**；相对拆前 blob 全部未改。
- `tests/`（`render_kernel_rgb.rs`/`render_lighting_kernel.rs`/`render_launch_abi.rs` 抽查）+ `Cargo.toml`/`Cargo.lock` blob 与拆前相同；变更路径只有 `cast.rs` + `cast_mesh.rs`（+ 证据/`.normify`）。
- 实现证据 `r05-split-cast/`：**9 文件**、`manifest.txt` **8 条**（不含自身），**8/8 对 committed blob**、**无自引用**、**LF 无 BOM**、**禁词 0**（6 全局 + 本片 3 词表）；README 75 行；`receipt.json` 的 `/task_id` 正确。
- `plan_vs_actual` 差异已记录：抽 **L284-372（89 行，含类型文档）** 而非批准稿 L293-372（80 行，理由：类型文档须随类型走）；`to_i32` **未随迁**（`pub(super)` 复用，批准稿二选一）；`cast.rs` 411（预算 ≈414）/新文件 102（预算 ≈90）；备选 `SceneDevice`（L182-292）未批未动。
- 提交范围：`72a7de66` 17 文件、`3e8628b6` 2 文件，**全部落在卡 scope**（`cast.rs` + 新文件 + 证据目录 + `.normify/`）。

## 3. 我的证据与复算方法

- `recompute.py`：一次跑完 6 个复核点（迁出块字节、4 hunk + 全量重构、源拼接 sha、API/引用/blob、行数与红线、证据自校与提交范围），输出 `recompute.json`；本次 `problems = []`。
- manifest 双规则：**本目录文件按 index blob（提交后 = committed blob，LF）**；被复核仓库文件按 `git show HEAD:<path>`。`manifest.txt` 不自引用。
- 原始日志：`logs/{fmt,clippy,clippy-cuda,unit,kernel-gate}.log`（我自己的运行，`cmd /c` 直出、LF 归一化），`logs/recompute.log`（脚本完整输出）。

## 4. 观察（6 条，均不阻断）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| **O1** | 建议（证据算术/描述） | 证据称「插入 `#[path]` + `pub use`（**+9 行**）」（README §1、receipt `git_diff_nature`），并由此得「预算 ≈414」；实测插入 **7 行**（2 注释 + `#[path]` + `mod` + 空行 + `pub use` + 空行），另有块后 **1 空行**随块删除，净 −83 = 494−89−1+7 → **411**。另 README §2.5 说「仅 3 类改动」但随后列举了 4 项（import 收窄 / 插入 / 块删除 / `to_i32` 可见性），实际 diff 为 4 个 hunk | **修**（一行数字与措辞；实际 411/102 与结论不变） |
| **O2** | 建议（既有缺口，非本片引入） | 模块源登记不完整：`src/render/kernel/` 下 **6 个 .rs 未登记**为 `mjwarp.render` 源（`cast.rs`、`megakernel.rs`、`megakernel_aa.rs`、`megakernel_arrays.rs`、`output.rs`、`renderer.rs`）；拆分前树 37 条源时 `cast.rs` 同样未登记，本片只按卡面登记了新文件 `cast_mesh.rs`（37→38）。`validate --fingerprints` 错误 0 只校验已登记项，不校验完整性 | **接受 + 建议登记治理项**（若 Arch 认为需补） |
| **O3** | 记录 | `to_i32` 由私有 `fn` 改为 `pub(super) fn`：仅 kernel 模块内可见、crate 外不可见；函数体逐字节不变；调用点 `cast.rs` 9 处、`cast_mesh.rs` 12 处 | **接受** |
| **O4** | 记录 | 对外路径 `cast::MeshDevice` 由 `pub use` 保持；4 个外部引用文件 blob 未改；公开项集合不变 | **接受** |
| **O5** | 记录 | `#[path]` 私有子模块第三次同款先例（拆分①/`lighting.rs`），`kernel/mod.rs` 零改动；既有结构债延续 | **接受** |
| **O6** | 记录 | `plan_vs_actual` 与批准方案的差异（L284-372 vs L293-372、`to_i32` 未随迁、411/102、备选未批）均如实记录并给原因 | **接受** |

## 5. 边界与未覆盖

- **未在拆前树（`4a5a8449`）重跑内核编译门**：PTX 一致性由「6 个源文件相对拆前逐字节相同 + 源拼接 sha 自算 = 基线」推断，我的复跑 PTX 与基线逐位相同。若需绝对实机复核，需另建 worktree 全量编译（本片按只读 + 轻量口径未做）。
- 无 GPU / 无验证窗口 / 无渲染数值结论（本片为 Rust 侧设备打包段移动，不发射内核）。
- 未重跑实现片 `.normify/runs/` 内四门禁记录的原始运行；只核其绑定树与结论一致（`normify check` exit 0）。
- `validate --fingerprints` 的 5 条警告为既有结构类警告，未逐条追溯是否早于本片。

## 6. 收口状态

- `normify gate 2026-10-28-render-bvh-r05-split-cast-check` → **exit 1「任务没有引用任何门禁」属预期**（QA 复核卡无门禁；receipt `gate_note` 已记）。
- `normify check 2026-10-28-render-bvh-r05-split-cast` → exit 0；`normify validate --fingerprints` → 错误 0。
- 两段提交：证据（本目录 + 任务卡）→ `normify done` → verified 记录单独提交；显式路径，无 `git add -A`。
