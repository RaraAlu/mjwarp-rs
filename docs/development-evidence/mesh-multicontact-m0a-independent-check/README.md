# M0a（sidecar ABI 496→504 + 上传链 + 空 sidecar 语义 + T4）QA 独立复核

结论：**pass（阻断 0 / 观察 6）**。复核对象：交付 `097c54b9`（复核末 HEAD `83fe9f54`，仅 `.normify` M2 验收文案变更），证据 `docs/development-evidence/mesh-multicontact-m0a/`。本复核纯 CPU：未开 GPU 窗口、不建 CUDA context；独立重算 ABI 布局、重跑 ABI/校验单测、T4 重采/身份、冒烟、默认/all-features/no-default 门禁、源拼装，并逐条核 `validate_mesh_poly` 规则与上传链证据。

## 1. 范围与不变量

- 交付 diff：`src/physics/collision/cuda/product.cu`、`src/runtime/collision_gpu.rs`、`abi.rs`、`types.rs`、`tests.rs`（拆分）、新增 `buffers.rs`/`abi_tests.rs`/`validate_tests.rs`、tests 侧补 9 个 `&[]` 与 test-only 拆分；共 32 文件 +1401 −406。
- 独立核：`src/physics/mod.rs`、`fixtures/precision/*`、`fixtures/g10-p5-primitive*`、`docs/development-evidence/g10-p5-gpu-kernels/*`、`src/io/render.rs`、`src/model/render.rs` 本提交**零 diff**；`src/runtime/collision_upload.rs` 零 diff（`CollisionUploadPlan` specs 仍 14）；`DEVICE_SOURCES` 仍 27 段；`product.cu` 恰好 6 个 `extern "C" __global__` 入口；未新增 `.cu`。
- 判据/容差/期望/参考零改动；`prepare_cases.py` 与旧 G5 证据/旧参考零 diff；旧窗口结果与旧 sha 只作历史记录（receipt 已显式写明 ABI 变更使其失效、M2 需全量重验）。

## 2. ABI 布局（独立重算，非只看 README）

我用**独立 rustc 程序**（复制 `abi.rs` 三个 `#[repr(C)]` 定义，`std::mem::{size_of,align_of,offset_of}`）重算，与交付 `abi_tests.rs`/README 完全一致：

| 项 | 实测 |
| --- | --- |
| `CollisionArgs` | size **504**、align **8** |
| 指针块 | 0–176 既有 23 指针，**184** `mesh_poly_args` |
| 标量块 | `nworld` **192** … `efc_width` **340**（全部较 496 版 +8） |
| `out` | **344**（`CollisionContactOut` size 136、align 8；344 为 8 对齐，标量结束即 344，无隐式填充） |
| 尾指针 | `nacon` **480** / `nccd` **488** / `overflow` **496** |
| `MjwarpMeshPolyArgs` | size **72**、align **8**；9 指针 offset 0/8/16/24/32/40/48/56/64 |
| 旧版对照（git 097c54b9~1 测试） | size 496；`nworld` 184；`out` 336；`nacon/nccd/overflow` 472/480/488 → 新=旧+8 ✓ |

- sidecar 9 指针顺序 = pinned `collision_convex.py` L197-205（`polynum/polyadr/polynormal/polyvertadr/polyvertnum/polyvert/polymapadr/polymapnum/polymap`）✓；host 与 `product.cu` 结构字段顺序一致（我逐字段对照）。
- `abi_tests.rs` 的 `offset_of!` 断言我重跑通过；`all_six_entries_take_args_by_value` 断言 6 个入口各恰有一处 `extern "C" __global__ void <entry>(mjwarp_collision_args a)`，且结构/sidecar 字段各仅一处（非空转）。另有既有 `product_source_exposes_three_extern_c_entries` 负向检查产品源不含探针 host 代码。

## 3. 上传链与债务标注（逐环核）

- README §3 与 receipt 逐环 file:line 我抽查核对属实：`src/model/render.rs:473-481` 确有 9 个 `mesh_poly*` field；`src/io/render.rs:320 fn check_mesh_ranges` 与 347-359 的 mesh_poly 区间校验存在；`types.rs` 32 个输入 slice（我数：23 既有 + 9 `mesh_poly*` = 32，另 `meta`）；`collision_gpu.rs:148-157` 9 路 `upload`、`:187-209` sidecar 构造（全空→null）、`abi.rs` 入口、`product.cu:25/63` 设备结构。
- 债务表述**显式**：§8 写「产品内当前没有 model→CollisionInputs 接线；现有映射只在 tests/support/*；本片不扩 scope；不能认为产品管线已能端到端喂 32 数组」，并引用 backlog `2026-10-26-product-collision-inputs-wiring`；我核该 backlog JSON 存在、status=backlog、depends_on M3、未 start ✓。
- 代码层核：`CollisionGpuInputs` 只在 `collision_gpu` 内部与测试中出现；`src/io/render.rs`/`src/model/render.rs` 本提交零 diff → 「环0→1 调用侧职责」属实，不是已接线。
- 上传实现：`mesh_poly_present = !mesh_polynum.is_empty()`；`validate_inputs` 在 `run()` 入口先执行（`collision_gpu.rs:119`），9 项不全/不自洽在分配前返回 `InvalidArgument`；全空时 `sidecar_storage=None`、`mesh_poly_args=null`；guard 声明顺序保证 sidecar 存储晚于 guards 释放（borrow 检查通过，注释与代码一致）。

## 4. 空 sidecar 语义与单测（规则逐条核）

规则实现（`abi.rs::validate_mesh_poly`）我逐条读过：①9 项必须同时提供/同时为空；②逐 mesh 数组等长（polynum/polyadr/polyvertadr/polyvertnum/polymapadr/polymapnum）；③`polynormal % 3 == 0`；④`sum(polynum) == nmeshpoly`；⑤polyadr/polynum 与 polyvertadr/polyvertnum、polymapadr/polymapnum 的区间界（含负值拒绝）。逻辑本身正确。

单测（`validate_tests.rs`，我重跑通过）覆盖：全空 + 非 mesh 组合通过；部分为空报错（1 例）；自洽 sidecar 通过、`polynormal%3` 报错、逐 mesh 长度不等报错。**覆盖缺口**：④sum 不等、⑤区间越界/负值未被单测覆盖；`empty_mesh_poly_sidecar_is_null_and_valid` 只在 CPU 层断言 slice 为空，未断言 `mesh_poly_args==null`（README 已声明发射级在 M2）→ 观察 O2。

## 5. T4 身份（0 漂移）

- `t4-current-kernels.json` 的 20 条 `product_sources` 与本次改动文件**交集为空**（脚本比对）；身份文件/期望/receipt 字面量零改动。
- 我独立重跑：`recapture` = **2 passed / 0 failed / 3 ignored**；`t4_identity_live_check` = **2 passed / 0 failed**，与交付 pre/post 日志一致；身份文件 git 零 diff。
- 结论：改前/改后 0 漂移成立（且我这次是在改动后的树上跑）。

## 6. 冒烟与源拼装（窗口外，无 context）

- 我重跑 `cargo test --locked --features cuda-probe --lib runtime::collision_gpu -- --include-ignored`：**31 passed / 0 failed**，`nvrtc_smoke_ok ptx_bytes=3786520 entries=6 source_bytes=238969 source_sha256=ec2e7acd…`，与交付一致。
- 自写 27 段拼装：工作树字节 = **238969 / ec2e7acd…**（与冒烟逐字节一致）；git blob（LF）= **238489 / 0272ccaf…**（交付未列 LF 口径，我补记）。相对 B3 的 237922，变化仅为 `product.cu` 加了 sidecar 结构/字段与探针同步注释。
- `product.cu` 恰好 6 个入口、0 新增；mesh_poly_args 在设备侧只有声明，无任何消费（M2 未实现）。

## 7. 门禁数字（我独立重跑）

| 项 | 我实测 | 交付 |
| --- | --- | --- |
| unit（gate，默认特性） | 见 §9 门禁 run 记录（预期 980/0/372） | 980/0/372 |
| `--no-default-features --lib` | **455 passed / 0 failed / 5 ignored** | 455/0/5 ✓ |
| `--all-features`（`MJWARP_MUJOCO_ROOT=target/toolchains/mujoco-3.12.0/package` + `+1.99.0-x86_64-pc-windows-msvc`） | **1029 passed / 0 failed / 430 ignored**，exit 0 | 1029/0/430 ✓ |
| collision_gpu（cuda-probe） | **31 passed** | 31 ✓ |
| 裸跑（无 env、默认工具链） | **exit 101**，失败在依赖编译（dlltool 缺失：getrandom/backtrace/windows-sys） | 交付也记失败，但原因写「env 未设→build.rs panic；dlltool 缺失」→ 见 O1 |
| fmt/clippy/clippy-cuda | gate 全过（§9） | exit 0 ✓ |

工具链实测 `rustc 1.99.0 (b940084d7 2026-09-28)`；`target/toolchains/mujoco-3.12.0/package` 存在。

## 8. size-gate test-only 拆分

- 交付记录明确：plane 测试 493→**503** 超 500 → 抽 `PlaneCase/TETRA/CASES` 到 `tests/support/primitive_plane_cases.rs`（63 行），主文件 **450** 行；我实测行数一致（450/63），`TETRA` 数据 old vs new **逐 token 相同**，其余断言/字段/期望未变；只动 `tests/`。
- 「ABI 与冒烟指纹不变」：当前树重跑冒烟 = 238969/ec2e7acd/3786520/6，与交付一致；拆分为 tests-only，src/device 源不受影响 → 结论成立（单 commit 内先后顺序不可由 git 独立还原，按证据记录采信；见 O6）。

## 9. 门禁记录（主树，stdout 写 `target/qa-m0a/gate-check/`）

- `normify gate 2026-10-26-mesh-multicontact-m0a-independent-check --repo D:\Rust\mjwarp-rs`：**exit 0**，fmt/clippy/clippy-cuda/unit 全过（unit 计数见 `.normify/runs/<task>/unit.json`）。
- `normify check`：**exit 0**（紧跟最终 gate；stdout 存 `target/qa-m0a/gate-check/check.log`，不拷入证据目录以免范围变化使记录过期）。

## 10. 未覆盖 / 边界

- 未开 GPU 窗口；GPU 上 null/非 null sidecar 的发射级行为按交付边界留给 M2 验证窗。
- 未重跑全部 12 个 G5 device case（ABI 变更后按交付口径 M2 全量重验；本片只核源拼装/冒烟与旧证据零改动）。
- 不宣称 mesh 已支持/全路径/跨环境；本片只做 ABI 接线与 CPU 可验证语义。

## 11. 复现命令

```powershell
rustc --edition 2021 target/qa-m0a/layout_check.rs -o target/qa-m0a/layout_check.exe; target/qa-m0a/layout_check.exe
cargo test --locked --features cuda-probe --lib runtime::collision_gpu -- --include-ignored --nocapture
cargo test --locked --lib runtime::transfer::kernel::recapture -- --nocapture
cargo test --locked --test t4_identity_live_check
cargo test --locked --no-default-features --lib
$env:MJWARP_MUJOCO_ROOT=(Resolve-Path 'target/toolchains/mujoco-3.12.0/package').Path
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --no-fail-fast
python target/qa-m0a/assemble_m0a.py
normify gate 2026-10-26-mesh-multicontact-m0a-independent-check --repo D:\Rust\mjwarp-rs
normify check 2026-10-26-mesh-multicontact-m0a-independent-check --repo D:\Rust\mjwarp-rs
```

## 12. 观察清单（详见 `qa-observations.json`）

| id | 级别 | 摘要 |
| --- | --- | --- |
| O1 | 一般 | 裸跑失败原因证据只支持 dlltool/依赖编译失败；「env 未设→build.rs panic」在留证日志与我的复现中均未出现 |
| O2 | 一般 | `validate_mesh_poly` 的 sum(polynum)==nmeshpoly、polyvert/polymap 区间与负值规则实现正确但未被单测覆盖；empty 测试未断言 null 指针（M2 已声明） |
| O3 | 建议 | README §1 行数小偏差（product.cu 424 vs 实测 426；abi.rs 334 vs 337）；§5.2 排在 §5.1 之前 |
| O4 | 建议 | 证据 9 个 `.log` 为 UTF-16LE（git 视为 Binary），建议转 UTF-8 文本留档 |
| O5 | 建议 | 旧 `abi_collision_args_matches_probe_args_layout` 已移除；`ProbeArgs` 496 冻结现在只靠 probe.cu 零 diff，无测试/static_assert 锁 496 |
| O6 | 建议 | size-gate 拆分的「ABI/冒烟不变」为 tests-only + 当前树重跑吻合，单 commit 内先后不可独立还原；证据已显式记录，可接受 |
