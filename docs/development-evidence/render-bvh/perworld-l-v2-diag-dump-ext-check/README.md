# QA 复核：产品侧扩展像素 dump 两卡（主卡 `bb29e351` + 配套卡 `e7fc06f7`）

- 任务：`2026-11-11-render-perworld-l-v2-diag-dump-ext-check`（owner QA；零 GPU、只读复核，未改任何源码/判据/参考/容差）。
- 对象：主卡 `2026-11-11-render-perworld-l-v2-diag-dump-ext`（提交 **`bb29e351`**，28 文件 +425/−16）；配套卡 `2026-11-11-render-megakernel-slot-82-ratchet-support`（提交 **`e7fc06f7`**，10 文件 +144/−10）。两卡均 verified。
- 复核基线：登记 `435e39a3`（本卡），复核时工作树 HEAD 为 `26d3cada`（其上的 `dde275c1`/`26d3cada` 是 G-L2 后置 errata 批，**未触及**本卡复核的代码与证据）。
- 结论：**阻断 1 项（B1）+ 一般 4 项 + 低 5 项**。B1 = dump 顶层 `SCHEMA` 仍为 **1**，与 README / receipt / 两卡验收 / 两个提交信息声称的 **schema 2** 不符；修好前一行为一行，但**修好前不宜作为诊断轮 #7 的产品侧数据源放行**。其余关键面（算术未变、零副作用门控、82 槽 ABI、偏移编码、边界、门禁）均独立核实通过。

## 1. 阻断 B1：dump 顶层 `schema` 与声明不符

| 位置 | 声明/实现 |
| --- | --- |
| `tests/support/render_bvh/light/diag.rs:24` | `pub const SCHEMA: u32 = 1;`（未改） |
| 同文件 `:110` | `"schema": SCHEMA,` → 产物顶层实际输出 `"schema": 1` |
| 主卡 README §2 标题 / L61 | 「schema 2」/「顶层 `schema` 由 1 → 2」 |
| 主卡 receipt `dump_schema.json_location` | 「pixel-dump.json schema 2」 |
| 主卡验收、配套卡标题/验收 | 「dump schema 2」 |
| `bb29e351` / `e7fc06f7` 提交信息 | 「dump schema 2」 |

⇒ 数据契约与产物不一致：按「schema 2 = 带 `diagnostics`」消费该 dump 的对照/重放工具会误判版本。建议修法：`SCHEMA: u32 = 2` 一行 + errata 记一句（提交信息不可改，记录面同步更正）。

## 2. 一般项

- **G1 记录面仍指向不存在的 `render_diag.cu`**：主卡 README §1 表格写「`MEGAKERNEL_SOURCE` 在 `render_common.cu` 后插入 `render_diag.cu`」，主卡验收同句，`src/render/kernel/diag.rs:10`、`tests/support/render_bvh/light/diag_ext.rs:4` 注释亦引用 `cuda/render_diag.cu`；主卡卡记录的 scope 还列了该文件路径。**实际未新增任何 `.cu`**，`MEGAKERNEL_SOURCE` 仍 7 项（与本卡 README §4 的 7 文件 stitched source 自相矛盾），DIAG 常量与 store helper 定义在 `render_megakernel.cu` 顶部。设计本身没问题（正是为了保住 42 文件精度源面快照），只是记录面过期，建议 errata 更正。
- **G2 记录索引公式与实现不符 + 多 world 单次发射静默无诊断**：README §2 / receipt `record_index` 写「记录下标 = `worldid * total_rays + rayid`」，实现为 `diag + (long)rayid * DIAG_STRIDE`（`render_megakernel.cu:279`，无 world 基址）。唯一使用路径是 per-world 单 world 发射（`launch_worlds=1`、kernel `worldid` 恒 0、`DiagBuffer` 分配 `total_rays` 条），此处等价且正确；但 `render_impl(world=None)` 多 world 单次发射时各 world 写同一段前 `total_rays` 条，且 `DiagCapture.world=None` 与 `diag_record(world)` 查找的 `Some(world)` 不匹配 → 诊断字段全为 `null`（不产生错数据，但静默缺失）。建议记录「仅支持 per-world 单 world 发射」或按 world 加基址。
- **G3 README 行数/文件数写错**：README §6 写「`render_shade.cu` 473 行、`render_megakernel.cu` 414 行」，实测 **478 / 499**（receipt `line_budget` 的 478/499/500/496/494 才对）；§1 标题「11 个源文件」与主卡实际 8 个 src + 2 个 test 不符（配套卡另 5 个 test）。
- **G4 新解码路径无自证测试**：`DiagRecord::from_slice`（偏移/位型/`view_dir` 派生）与 `diag_ext` JSON 编码没有任何单元测试，只能靠诊断轮实测。作为逐站比对的数据源，建议补一个纯 CPU 合成 64×f32 记录的往返测试（无需 GPU）。

## 3. 低项

- **L1**：`tests/render_launch_abi.rs` 断言 82 但消息仍写「1 bounds + 73 数组 + 7 标量」（实为 8 标量）；`tests/render_lighting_kernel.rs` 测试名 `megakernel_slots_signature_is_unchanged` 与注释「81 槽 S7a 冻结签名：R09-S2 不增槽」已误导。
- **L2**：README §4 离线 PTX 统计 344,182 / 347,071 B（sha `5abc0b4e…`/`c2d8a885…`）与 §3 身份用例 344,181 / 347,070 B（`7fd86b85…`/`1fd53131…`）不同；我独立编译复现出后者，差 1 字节 = 离线脚本把 NVRTC 输出尾部 NUL 计入（sha 因此不同），PTX 内容相同。建议注明口径。
- **L3**：算术未变门只归档了输出日志，未归档抽取脚本/命令，该门不可逐字复算（我已用自写脚本独立复现）。
- **L4**：`nlight >= 4` 时诊断灯槽冲突（`diag_slot = light < 3 ? light : 3`，第 3 盏之后全落 slot 3 与 headlight 同槽）；本 G-L2 场景（2 盏 + headlight）不受影响，记录限制即可。
- **L5（观察）**：Backend 收口期间 CPU `cargo build`/测试与 Arch 分钟级 GPU 窗口时间重叠——CPU-only 负载、GPU 计算未重叠，不违反 GPU 窗口规则；按团队「同一时间单构建者」纪律记一笔。本复核的门禁为串行执行。

## 4. 已独立核通过的关键面（附实测数字）

1. **算术未变门（关键）**：用仓库 pinned `target/toolchains/cuda-12.8.1/bin/nvrtc64_120_0.dll`（nvrtc 12.8）+ product 选项（`--fmad=true --std=c++17 --gpu-architecture=compute_70 --device-as-default-execution-space --include-path=include/warp-headers`）分别编译 old（`755b3d07` 七个 blob，按 `mod.rs` `include_str!` 顺序 LF 逐字节拼接）与 new（工作树同序）：
   - source：old `02160146de4e…`/104,676 B、new `624ab298e82e…`/109,384 B（与我独立复算逐位一致）；
   - PTX：old **`7fd86b85…`/344,181 B**、new **`1fd53131…`/347,070 B**（与身份用例打印逐位一致）；
   - f32 算术 opcode 序列 **完全相同**（我的两种抽取口径 3,124==3,124 与 3,151==3,151，sequence/multiset 均 identical；卡自报口径 3,148==3,148）。详见 `artifacts/arith-independent.txt`。
2. **env 未设零副作用**（读码）：`DiagBuffer::new` 在 `enabled()==false` 时 `Ok(None)`（零分配/零上传）；`arg_addr(None)→0`；`finish_diag(None)` 空操作；kernel `diag_rec=nullptr` 且 4 个 store helper 全部首行 `if (d == nullptr) return;`；测试 dump 本身由同一 `MJWARP_DIAG_PIXEL_DUMP` 门控（未设 → 无打印/无文件）。
3. **82 槽 ABI 一致**：脚本解析 kernel 形参 **82 个**（末参 `wp::uint64 diag_ptr`）== `MEGAKERNEL_SLOTS` 82（**1 bounds + 73 数组 + 8 标量**，末槽 `S(SU)`=Scalar(U64)）；`RenderSignature::MEGAKERNEL.len()==82`；launcher `args.len() != slots.len()` / `!= signature.len()` 兜底；上游 81 槽顺序未动。
4. **schema 偏移/编码四方一致**：README §2 表 == kernel `DIAG_*` == `src/render/kernel/diag.rs` 解码常量 == `diag_ext` JSON 键（0–2 color、3–5 base_color、6–7 tex_uv、8–10 tex_color、11/12 bary、13 t、14 face、15 mesh_id、16–18 hit_point、19–21 normal、22–24 ray_dir、25–56 lights 4×8、57–63 保留、stride 64）；`face`/`mesh_id` 以 `__int_as_float` 存、host `to_bits as i32` 解回；`view_dir = -ray_dir_world` 为精确符号位翻转（含 ±0）。
5. **门禁/边界/证据**：我独立重跑 fmt/clippy/clippy-cuda/`cargo test --locked` 全 exit 0，unit **1,387 passed / 0 failed / 402 ignored**（`artifacts/gates.txt` + `.normify/runs/…-check/`）；两提交只含 `src/`（8 文件，含新 `diag.rs`）、`tests/`（2+5）、证据与卡记录，`fixtures/`/`scripts/` 零 diff；guards JSON `fad2ed37…`/43,729 B 与 12 个冻结 exe stash 未动；**无新增 `.cu`**（42 文件 CUDA 源面快照测试通过，`MEGAKERNEL_SOURCE` 7 项）；行数 `render_shade.cu` 478 / `render_megakernel.cu` 499 / `megakernel.rs` 500（棘轮内）；主卡证据 manifest **10/10 独立重算全对**（README `c5e21972…`/9,646 B、receipt `fc0494f2…`/7,381 B、manifest `78c4db9f…`/1,173 B），10 文件全 UTF-8/无 BOM/裸 CR 0/TAB 0。
6. **测试改动只服务诊断**：主卡 `tests/` diff 3 行（`DumpInput.renderer` + `diagnostics` 键 + 模块挂载）；配套卡 4 处槽数断言 + 新 `diag_ext` 模块；无阈值/参考/相机归属/判据 A 比较逻辑改动。
7. **口径**：主卡 README §6 与 receipt `non_claims` 明写不宣称修复/不宣称数值等价/不改判据/阈值/参考/容差/G-L2 判定维持未通过（判据 A FAIL）；无正面「已修复/已通过/等价」表述。

## 5. 复现方法（零 GPU）

1. 算术门：`git show 755b3d07:<7 个 .cu>` 与工作树文件分别按 `src/render/kernel/mod.rs` 的 `include_str!` 顺序字节拼接 → 用 pinned nvrtc DLL（ctypes）以 product 选项编译 → 取 PTX → 抽 `(add|sub|mul|fma|div|sqrt|rcp|neg|min|max|abs)*.f32` 与 `cvt.*.f32` opcode 序列比对；产物见 `artifacts/arith-independent.txt`。
2. 交叉核对：`artifacts/crosscheck.txt`（含 SCHEMA 行、`render_diag.cu` 引用、kernel 形参计数、槽位尾部、偏移常量、行数、guards/stash、两提交文件清单）。
3. 门禁：`cargo fmt --check` / `cargo clippy --locked --all-targets -- -D warnings` / `+ --features cuda-probe` / `cargo test --locked`；另 `normify gate …-check fmt clippy clippy-cuda unit`。

## 6. 非声明

- 本复核**不判定修复效果**、不改判据/阈值/参考/容差、不改 G-L2 判定（维持未通过）。
- B1 的修法是**数据契约一致性修复**，不属于放宽判据；G1–G4/L1–L5 只涉及记录面、注释与测试覆盖建议。
- 未在 GPU 上运行任何东西；未重冻 guards、未改冻结件与已签材料。

## 7. artifacts

- `artifacts/arith-independent.txt`：算术未变门独立复算（source/PTX 哈希、三种抽取口径、NUL 口径说明）。
- `artifacts/crosscheck.txt`：SCHEMA、`render_diag.cu` 引用、82 槽 ABI、偏移常量、行数、边界实测。
- `artifacts/gates.txt`：四门禁（自跑 + normify gate 记录）。
- `receipt.json` / `manifest.txt`：本卡凭据与证据清单（本文件不在 manifest 内）。
