# R05 拆分①（`megakernel.rs` 设备上传段纯移动）独立复核

任务：`2026-10-28-render-bvh-r05-split-megakernel-check`（P8 渲染与 BVH；p56；owner QA）
被复核任务：`2026-10-28-render-bvh-r05-split-megakernel`（已 verified）
被复核提交：`5d4d23d0`（实现 + 证据，16 文件）+ `00ea2baf`（verified 收口，2 文件）
拆分前树：`00ed9efb3ac36e716d1e2ddaefecf956922ff9ca`；复核时 HEAD：`00ea2baf`
范围：只读复核 + 本证据目录；不开 GPU 窗口、不重跑窗口；不改代码/判据/夹具。

## 1. 结论

**verdict：pass / 阻断 0 / 观察 6**（O1–O3 建议级，O4–O6 记录级；均为证据措辞/结构债，不阻断本片验收）。

本片是「纯移动」，复核口径不是"能编译"，而是**逐项证明行为面零变化**：

| 复核点 | 结果 |
| --- | --- |
| 内核源字符串 / PTX | 6 个 `include_str` 源文件与拆分前逐字节相同；source sha 我自算 = 基线；PTX sha/bytes 我自跑门禁 = 基线 |
| 公开 API / 81 槽 ABI / CUDA 源 / 测试断言 | 全部零变化（`kernel/mod.rs` blob 相同、`args.rs` blob 相同、`tests/` diff 为空） |
| 上传语义 | 70 个 `upload(...)` 调用顺序 + 实参逐项等价；81 个 `args.push` 逐槽内容一致 |
| 行数 / `#[path]` | 406 / 384 / `mod.rs` 474 零改动；`#[path]` 仅 1 处私有声明；模块源 37、`validate --fingerprints` 错误 0 |
| 回归 | 我独立复跑四门禁：fmt 0 / clippy 0 / clippy-cuda 0 / unit **1069 passed / 0 failed / 387 ignored（132 result 行）** |
| 红线 / 证据 | 红线行数全对；`DEVICE_SOURCES=28`、入口 6、`CollisionArgs` 504 B 由 unit 断言全绿佐证；实现证据 manifest 7/7 对 committed blob、LF 无 BOM、禁词 0 |

## 2. 复核点逐条（数字全部我自己重算/自跑）

### 2.1 纯移动证据（最硬）

- `MEGAKERNEL_SOURCE` = `concat!` 6 个 `include_str`（`src/render/kernel/mod.rs:58-65`）。我按该顺序把 6 个文件字节拼接后自算 sha256 = **`2d2c27df985882c2b571514579dff19f9103bfc903ec786db42a592ce1bd8c2a`**，与基线一致；且 6 个文件相对 `00ed9efb` 的 blob **逐字节相同**（`git rev-parse` 两端相等）。
- 我自己跑内核编译门（NVRTC 纯编译、不发射、无 GPU）：
  `cargo test --locked --test render_kernel_rgb -- --ignored megakernel_source_compiles_under_render_gate --nocapture`
  → `source_sha256=2d2c27df… / ptx_sha256=e955bb4b… / ptx_bytes=258660`，exit 0（`logs/kernel-gate.log`）。
- 结论口径：source 字符串拆分前后相同（文件级字节相同 + sha 相同）⇒ NVRTC 输入相同；我的复跑 PTX 与基线逐位相同。**边界**：我未在拆分前树上重跑该门（见 §5）。

### 2.2 API / ABI / CUDA 源 / 测试零变化

- `src/render/kernel/mod.rs`：`00ed9efb` 与 HEAD 的 blob 相同（`151b7294…`），即拆分子模块声明**没有**进 `kernel/mod.rs`。
- `src/runtime/launcher/render/args.rs`（81 槽表）blob 相同；`render.cu`/`render_common.cu`/`render_shade.cu`/`render_megakernel.cu` 等 6 个 CUDA 源 blob 相同。
- `tests/` 与 `Cargo.toml`/`Cargo.lock`：`git diff 00ed9efb..HEAD` **为空**（`recompute.json` → `tests_and_manifest_untouched = true`）；抽查 `tests/render_kernel_rgb.rs`/`render_lighting_kernel.rs`/`render_launch_abi.rs` 两端 blob 相同。
- `MegakernelRenderer` 的 `pub fn` 签名行集合：拆分前/后一致（`public_fn_signatures_identical = true`）；新模块只有 `pub(super)` 类型与函数，无新增公开导出。
- unit 里 ABI 断言逐条绿：`megakernel_slots_signature_is_unchanged`、`megakernel_signature_covers_upstream_argument_classes`、`abi_collision_args_matches_device_layout_504`、`all_six_entries_take_args_by_value`、`device_sources_place_primitive_after_contact` 等（见 `logs/unit.log`）。

### 2.3 上传语义（顺序 / 实参 / 生命周期）

- 从 `00ed9efb:src/render/kernel/megakernel.rs` 提取 **70** 个 `upload(...)` 赋值调用，与新 `megakernel_uploads.rs` 的 70 个按顺序逐项比对：
  - 左值（写入的局部名）序列**完全相同**；
  - 实参在「`&self.session`→`session`、局部 `&X`→`inputs.X`」归一后**完全相同**；
  - 每个 `inputs.X` 都在 `UploadInputs` 字面量里绑定到**同名局部** `&X`（`bindings` 逐条核，无串位/交换）。
- 81 个 `args.push(...)`：去掉 `u.` 前缀与尾逗号归一后，**逐槽内容完全一致**（槽序 = `MEGAKERNEL_SLOTS` 顺序不变）。
- `blob_buffer`：从局部变量改为 `MegakernelUploads` 字段（`#[allow(dead_code)]`，只为保设备内存活到发射）。`uploads` 与旧局部变量同生命周期（都在 `render()` 作用域内、`launch_samples` 之后才 drop），我核过无提前 drop、无 moved-out。
- 错误路径：`Texture2dArray::upload`/`upload`/`ints`/`floats`/`flags` 的 `?` 传播点与顺序不变，错误值类型不变（`LauncherError`）。

### 2.4 行数与 `#[path]`

| 文件 | 行数（我的口径：committed blob 的 LF 行数） | 要求 |
| --- | --- | --- |
| `src/render/kernel/megakernel.rs` | **406**（拆分前 **490**） | ≤430 目标达成，≤500 |
| `src/render/kernel/megakernel_uploads.rs`（新） | **384** | ≤500 |
| `src/render/kernel/mod.rs` | **474**（零 diff） | — |

- 移动段实测 **L203-344（142 行）**；L345-347 的 `rgb_shape/depth_shape/seg_shape` 保留原文件（与证据 README §3 一致）。
- `#[path = "megakernel_uploads.rs"] mod megakernel_uploads;` 在 `megakernel.rs` 内**仅 1 处**、无 `pub`、`kernel/mod.rs` 无重复声明；模块源 `render.md` = **37** 条且含该文件；`normify validate --fingerprints` **错误 0**（警告 5 条为既有结构类警告，与本片无关）。

### 2.5 回归（我自己跑的四门禁）

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --check` | exit 0（`logs/fmt.log`） |
| `cargo clippy --all-targets --locked -- -D warnings` | exit 0（`logs/clippy.log`） |
| `cargo clippy --all-targets --locked --features cuda -- -D warnings` | exit 0（`logs/clippy-cuda.log`） |
| `cargo test --locked` | **1069 passed / 0 failed / 387 ignored**，132 result 行（`logs/unit.log`） |

- 含 S7b1/S7b2/S7b3/S8 位等回归（`render_e2e`/`render_kernel_rgb`/`render_kernel_flags`/`render_launch_abi` 等用例在 `logs/unit.log` 中逐条 `ok`）。
- `clippy -D warnings` 通过同时说明：拆分后**无未使用导入/死代码**（`blob_buffer` 的 `#[allow(dead_code)]` 是显式例外）。

### 2.6 红线与证据

- 红线行数（我的口径）：`render.cu` **499**、`launch.rs` **500**、`cast.rs` **494**、`model/render.rs` **500**、`camera_output.rs` **499**、`args.rs` **494**；均未改动（blob 相同）。
- `DEVICE_SOURCES=28`、入口 6、`CollisionArgs` 504 B、`MEGAKERNEL_SLOTS` 81：由 unit 断言全绿佐证（非我另写脚本数）；`hfield_bvh_id` 槽仍占位 `[0u32, 0]`（上传段原样）。
- 实现证据 `r05-split-megakernel/`：**8 文件**、`manifest.txt` **7 条**（不含自身），**7/7 对 committed blob**（sha+bytes，LF 归一）、**无自引用**、**LF 无 BOM**、**禁词 0**（6 条全局 + 本片 3 词表）；README 74 行 ≤500；`receipt.json` 的 `/task_id` 正确。
- 提交范围：`5d4d23d0` 16 文件、`00ea2baf` 2 文件，**全部落在卡 scope**（`megakernel.rs` + 新文件 + 证据目录 + `.normify/`），无越界文件。
- `plan_vs_actual` 差异**如实记录**（README §1 注/§3 + receipt）：抽出 142 行（批准稿 145）、新文件 384（批准 ~170）、`megakernel.rs` 406（批准 ~345，目标 ≤430 达成）、`#[path]` 替代 `mod.rs` 声明；原因（80 字段不可收敛、34 行 `UploadInputs` 构造、`u.` 短前缀避免 fmt 折行到 477）写明。

## 3. 我的证据与复算方法

- `recompute.py`：一次跑完全部数字（行数、6 源拼接 sha、移动段逐行归一对照、70 调用顺序/实参、81 槽、API/ABI blob、测试零改动、实现证据 manifest 自校、提交范围、门禁日志解析），输出 `recompute.json`；本次 `problems = []`。
- manifest 双规则：**本目录文件按 index blob（提交后即 committed blob）**；被复核仓库文件按 `git show HEAD:<path>`（LF 归一）。`manifest.txt` 不自引用。
- 原始日志：`logs/{fmt,clippy,clippy-cuda,unit,kernel-gate}.log`（我自己的运行，`cmd /c` 直出，UTF-8 LF），`logs/recompute.log`（脚本完整输出）。

## 4. 观察（6 条，均不阻断）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| **O1** | 建议（证据完整性） | README §2.6 / receipt `git_diff_nature` 称「新文件为原段内容，仅引用改名 + 两条 import 改绝对路径」，但实测移动段归一后仍有 **3 行**不是字面搬运：三个输出 scratch 分支由「中段前算好的局部 bool（`rgb_enabled/depth_enabled/seg_enabled`）」改为**现场** `context.outputs().is_enabled(ImageOutput::X)`。行为等价（`is_enabled` 是 `Option::is_some` 只读访问、同一不可变 `context`、两步之间无写入），但差异面应写全 | **修**：证据补一句「3 处 scratch 开关改为现场 `is_enabled` 读取，只读同值」 |
| **O2** | 建议（注释准确） | `megakernel_uploads.rs` 文件头两处（L3、L146）写「原 L203-347」，实测移动段为 **L203-344**；L345-347 保留在原文件（README §3 已正确写明） | **接受或修**：改注释为 L203-344，避免后续核查按 347 找边界 |
| **O3** | 建议（措辞精度） | README §1 注 / receipt 称「80 个 `TransferBuffer<T>` 句柄」；实测 `MegakernelUploads` **80 字段** = 74 个 `TransferBuffer` + 4 个 `Option<TransferBuffer>`（AA/输出 scratch）+ `texture_array: Option<Texture2dArray>` + `blob_addr: u64` + 4 个 `usize` 计数 | **接受**（若改措辞更准：「80 字段，其中 78 个 TransferBuffer 句柄」） |
| **O4** | 记录 | `blob_buffer` 由局部变量改为结构体字段 + `#[allow(dead_code)]` 保活；`unwrap`/`expect` 无新增，`?` 传播不变 | **接受** |
| **O5** | 记录 | `#[path]` 私有子模块与 `lighting.rs`/`hfield.rs` 同款既有结构债（R09-S1 O2 已记录），本片无新增债务类型 | **接受** |
| **O6** | 记录 | `plan_vs_actual` 与批准行级预算差异（142/145、406/~345、384/~170、`#[path]` 替代 `mod.rs`）已如实记录并给原因；目标 ≤430 达成 | **接受** |

## 5. 边界与未覆盖

- **未在拆分前树（`00ed9efb`）重跑内核编译门**：PTX 基线的一致性由「6 个源文件与拆分前逐字节相同（`git rev-parse` 相等）+ source sha 自算等于基线「推断；我的复跑 PTX 与基线相同。若要求绝对实机复核，需另建 worktree 全量编译（本片按只读+轻量口径未做）。
- 无 GPU / 无窗口 / 无渲染数值结论：本片是 Rust 侧上传段移动，不涉及内核发射与像素。
- 未复核 `.normify/runs/` 内四门禁记录的原始运行（我只核它们绑的代码树与结论一致；`normify check` 通过）。
- 其余 `validate --fingerprints` 的 5 条警告为既有结构类警告，未逐条追溯是否早于本片（与本片 diff 无关）。

## 6. 收口状态

- `normify gate 2026-10-28-render-bvh-r05-split-megakernel-check` → **exit 1「任务没有引用任何门禁」属预期**（QA 复核卡无门禁；已在 receipt `gate_note` 记录）。
- `normify check 2026-10-28-render-bvh-r05-split-megakernel` → exit 0；`normify validate --fingerprints` → 错误 0。
- 两段提交：证据（本目录 + 任务卡）→ `normify done` → verified 记录单独提交；显式路径，无 `git add -A`。
