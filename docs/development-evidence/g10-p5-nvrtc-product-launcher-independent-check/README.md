# G10@P5 NVRTC 产品 launcher 独立复核（入口真实性 + 逐字节数值对照 + 错误路径）

- 任务：`2026-10-25-g10-p5-nvrtc-product-launcher-independent-check`（p57）
- 复核对象：`dfcc1bdc`（NVRTC 产品 launcher 接线；交付窗口起点为父提交 `49e2fab5`，本次复核两轮窗口均绑 HEAD `dfcc1bdc`）
- 判定：**pass（阻断 0、发现 0、观察 4）**
- 独立性：入口/拼装/ABI/错误路径全部从源码独立复算；数值对照**不复用交付测试的 bit_equal 结论**——QA 自建 dumper（只用产品公开 API）落盘 3 case 原始输出，再由自写 Python 比较器逐字节对 G5 探针 `out1`；两轮已批准 GPU 窗口内只跑预构建 dumper，窗口内不编译。

## 1. 入口真实性（关键项，已独立确认）

- `src/runtime/mod.rs`：`#[cfg(feature = "cuda-probe")] pub mod collision_gpu;`；`Cargo.toml` 里 `cuda-probe = ["dep:cudarc", …]` 且**不在 `default` features** → 默认构建不含 CUDA 产品入口（交付 §5.1 边界属实）。
- `CollisionGpuProgram::new(device)` → `compile_program()` → `compile_source_with` → **`cudarc::nvrtc::compile_ptx_with_opts`（真正 NVRTC 运行时编译）** → `load_module` + 三个 `load_function`；`run(kind, inputs)` 上传 23 个输入数组、按 `grid=(nworld,1,1)` 发射、同步、回读——**不是只有 ignored 测试、也没有绕回 nvcc 探针**（产品模块内无 `probe.exe`/`probe.cu` 引用）。
- 集成测试 `tests/g10_p5_nvrtc_launcher.rs` 是 `#[ignore]`，用产品 API `program.run(...)` 并与 `out1` 比对；测试存在本身不构成证据，数值结论见 §5（QA 自己的 dumper）。
- 编译选项独立核对：`fmad=Some(true)`、`use_fast_math=Some(false)`、`--std=c++17`、`--gpu-architecture=sm_89`、`include_paths=[]`、`name=mjwarp_collision_product.cu` —— 与交付声明的 nvcc 配方对齐。
- CPU 侧 NVRTC 冒烟我独立重跑（`--features cuda-probe --lib runtime::collision_gpu::nvrtc_smoke --include-ignored`）：`ptx_bytes=1863043 entries=3 source_sha256=dc862ec8…cad43`，与声明一致（不需要 GPU）。

## 2. `product.cu` 边界（不含数值内核）

- `product.cu` 顶层定义只有：`struct mjwarp_collision_args`、`struct LocalPolytope`、4 个胶水函数（`init_polytope`/`build_geom`/`build_params`/`sdf_type_of`）、3 个 `extern "C" __global__` 入口（`mjwarp_collision_contact/hfield/sdf`）。
- 它调用的数值函数（`g5_hfield_pair_contacts`、`g5_sdf_pair_contacts`、`g5_ccd_pair_write`、`g5_write_contact`、`g5_hfield_write_contacts`、`v3_make` 等）**全部定义在 `src/physics/collision/cuda/*.cu`**；`product.cu` **未重定义任何**数值内核（源码交叉核对 0 重叠）。
- 拼装：`source.rs` 的 14 个设备源（`common→geom→support→simplex→gjk_core→polytope→polytope_build→epa→ccd→sdf_octree→sdf_math→sdf_narrowphase→hfield→contact`）按本地 `#include "*.cu"` 依赖图独立验证为**合法拓扑序**，末尾追加 `product.cu`。

## 3. 设备源拼装指纹（独立复算）

按 `source.rs` 规则（删本地 `.cu` include、缺换行补齐、14 源顺序 + product.cu）在 Python 里独立重放：**len = 121566、sha256 = dc862ec884680349c6f38b40de0379082ec1b6dbf47d4dba9f22adfd8adcad43**，与交付声明 **逐位一致**（`matches_claim_len=true`、`matches_claim_sha256=true`）。

> **`121562` vs `121566` 说明（Arch 要求写入）**：交付证据（README L16、receipt L29）为 `len=121566, sha256=dc862ec8…cad43`；QA 独立复算同样为 **121566 + 同一 sha**，`matches_claim_len=true`。**不存在 4 字节差异；`121562` 是 Arch 转述时的笔误，不是证据差异。**

## 4. ABI 布局（从结构定义独立推导）

从 `abi.rs` 的 `#[repr(C)]` 字段类型（64 位：指针 8B、`i32`/`f32` 4B、数组按元素、内嵌结构 136B/align 8）独立计算：`CollisionArgs` = **496 B / align 8 / 55 字段**；`CollisionContactOut` = **136 B / align 8 / 17 字段**——与 `tests.rs` 的 `size_of`/`offset_of!` 断言一致（推导不是抄测试常量）。

## 5. 数值对照（QA 自建 dumper + 自写比较器，3/3 逐字节一致）

方法：QA 的 `dumper/`（独立 crate，只依赖 `mjwarp-rs` 的公开 API + `cuda-probe` feature）在窗口内对每个 case 调 `CollisionGpuProgram::new(0)` + `run(...)`，把 **dist/pos/frame/includemargin/friction/solref/solreffriction/solimp/adhesion/dim/geom/efc_address/worldid/geomcollisionid/type/nacon/nccd/overflow 共 18 个原始数组**落盘；随后 `qa_nvrtc_check.py` 用自己的比较器逐 4 字节对 G5 探针 `out1` 比较，并按 `bit_equal` 口径（`nworld(overflow) + nacon*(35+efc_width)`）独立复算分母。

| case | kernel | nacon/nccd | 逐字节相等 | 独立复算分母 | 交付分母 | 观测窗口 |
| --- | --- | --- | ---: | ---: | ---: | --- |
| `hfield-sphere` | Hfield | 8 / 0 | **314/314** | 314 | 314 | run1 |
| `sdf-box` | Sdf | 80 / 0 | **3122/3122** | 3122 | 3122 | run2 |
| `ellipsoid-box` | Contact | 2 / 2 | **80/80** | 80 | 80 | run2 |

- 三 case **分母独立复算与交付一致**，分子由 QA 自己的 dump 计算，**未引用交付的 `bit_equal` 结论**；逐字段（f32/i32）全部 4 字节相等，`nacon`/`nccd` 整值相等。
- 早停规则已执行：run1 只跑 `hfield-sphere`（见 §6 原因）；run2 依次 `sdf-box`→`ellipsoid-box`，两 case 均一致才继续；任一处不等即会停且不写「其余通过」。

## 6. 窗口事实（两轮，均 HEAD 绑 `dfcc1bdc`，窗口内不编译）

| 项 | run1（hfield-sphere，0.5 s 采样） | run2（sdf-box + ellipsoid-box，0.25 s 采样） |
| --- | --- | --- |
| 时间 / elapsed | 22:10:09–22:10:19 / **9.8 s** | 22:11:57–22:12:19 / **21.1 s** |
| HEAD | `dfcc1bdc…`（符合） | `dfcc1bdc…`（符合） |
| PATH / nvcc / NVRTC DLL sha | `target\toolchains\cuda-12.8.1\bin`；`release 12.8, V12.8.93`；3 个 DLL sha 已记 | 同左（同 sha） |
| dumper exe sha256 | `85550689…fc492`（预构建，窗口内未编译） | 同左 |
| 基线 used（漂移 vs 3669） | 3662（**7**） | 3691（**22**） |
| 图形上下文 / 本任务相关计算进程 | 34 / **0** | 34 / **0** |
| 采样点数 | 19（0.5 s） | **75（0.25 s）** |
| 采样峰值 / 相对基线 | **4987 MiB / +1325 MiB**（context 瞬时） | **3691 MiB / +0 MiB** |
| 软阈值 / 硬守卫 | 旧 +500 预期触发 → 停并报（Arch 后澄清 +500 是预期非批准阈值） | 软 +1536 仅记录（未超）；硬 3072/50 未触发 |
| 结束 used / 计算进程 / 残留 | 3662 / 34 / `qa-nvrtc-dumper` **0** | 3665 / 34 / **0** |
| 结果 | `hfield-sphere` 314/314 逐字节一致 | `sdf-box` 3122/3122、`ellipsoid-box` 80/80 逐字节一致 |

> run1 的 +1325 MiB 峰值：18 点 3662（NVRTC 编译 ~9 s 无 GPU 分配）→ 1 点 4987（CUDA context/模块创建瞬间）→ 结束 3662；硬守卫（3072 MiB/50%）未触发、残留 0。run2 用 0.25 s×75 点未再捕到该瞬时（每 case 进程中 context 存活仅约 30–100 ms，见观察 1），故 run2 序列峰值为 +0。

## 7. 错误路径审查（12 个 CPU 单测，**非假覆盖**）

- 编译失败：注入返回 `Err` 的编译闭包 → `CollisionGpuError::Compile`（含 `source_sha256`）；
- 编译期 `panic`：`catch_unwind` 兜底 → `Compile("NVRTC编译异常")`，**不 panic**（测试内 `panic!` 注入 + 断言错误映射）；
- 无设备：`program_from_parts(Err(..), Ok(Ptx))` → `NoDevice`；并验证**编译错误优先于设备错误**；
- 参数非法：`InvalidArgument`（`nworld/ngeom/naconmax/efc_width` 非正、`g1/g2` 越界、基础数组长度不匹配）；
- ABI 锁死：size/offset 断言（55/17 字段）；
- 源拼装：14 源依赖序、只删本地 `.cu` include、确定性 sha256；`product.cu` 三个 `extern "C"` 入口。
- 缺口（观察 3）：`CollisionGpuError::Load`（`load_module`/`load_function` 失败）与 `Launch` 分支**无 CPU 可注入 seam**，只有 Display 覆盖，未做函数级失败单测（不影响本片结论，建议后续补 seam）。

## 8. 观察（4 项，均非阻断）

1. **交付窗口峰值可能被低估（Arch 要求写入）**：交付 NVRTC 窗口报 **+185 MiB**（25 点 × 0.5 s），而本次同配方复跑 run1 实测 context 创建瞬时 **+1325 MiB**（4987 vs 3662）。该量级与 CUDA context 预留吻合（G6 perf 探针 `cudaMemGetInfo` 同样显示 ~1.2 GiB），属 **CUDA context 固有开销，不改变功能/数值结论**；但「窗口峰值」数字应以更密采样（本次 0.25 s 也仅偶然捕获）或多轮采样为准。Arch 将让 Backend 在 NVRTC 证据里补注（fix 记录）。
2. **`121562` 是 Arch 笔误**：交付与独立复算均为 **121566 + 同 sha**（`matches_claim_len=true`），不存在 4 字节差异。
3. **`Load`/`Launch` 错误分支无可注入 seam**（见 §7 末），未单测；Display 已覆盖。
4. **行数口径**：交付 README 写 464/495/491 是 PowerShell 5.1 默认 ANSI 解码下的**少计**；按 UTF-8 `wc -l` 实际为 **collision_gpu.rs 498、tests.rs 499、launcher 499、product.cu 296**——仍 **≤500**（余量 1–2 行），不违反限制；建议后续按 UTF-8 口径记录。

## 9. 边界与未覆盖

- 默认构建不含 CUDA 产品入口（`cuda-probe` feature 门控），**「接入默认产品管线」未覆盖**（交付 §5.1 已声明，本复核确认属实）。
- 数值对照仅 3 个 case（任务下限）；multicontact、primitive narrowphase 设备版不在本片范围。
- 仅 Windows + RTX 4070 Ti SUPER + NVRTC 12.8 实测；不跨硬件/跨 NVRTC 版本外推；未作精度/等价/放行类结论。
- 未改产品代码/判据/已 verified 数字/`Cargo.*`/fixtures/S2–S8。
- QA dumper 是独立 harness（公开 API），**不是产品管线**；其输出仅用于本复核逐字节对照。

## 10. 复现

```powershell
# 0) CPU 侧（不占 GPU）：构建 dumper 与运行 CPU 检查
cd D:\Rust\mjwarp-rs\docs\development-evidence\g10-p5-nvrtc-product-launcher-independent-check\dumper
$env:CARGO_TARGET_DIR='D:\Rust\mjwarp-rs\target\qa-nvrtc\dumper-target'; cargo build --offline
# 1) 已批准窗口（仅预构建 dumper；0.25 s 采样/软阈值/硬守卫见脚本）
powershell -NoProfile -ExecutionPolicy Bypass -File ..\run_window2.ps1
# 2) CPU 比较
python ..\qa_nvrtc_check.py
```

## 11. 提交路径与不入库

入库（`docs/development-evidence/g10-p5-nvrtc-product-launcher-independent-check/`）：

```
README.md  receipt.json  artifacts-sha256.json  qa_artifacts_sha256.py
qa_nvrtc_check.py  qa-nvrtc-check.json  qa-nvrtc-check.log
run_window.ps1  run_window2.ps1
window-log.txt  window-memory-samples.txt  window-output.txt
window-log2.txt  window-memory-samples2.txt  window-output2.txt
dumper/Cargo.toml  dumper/Cargo.lock  dumper/src/main.rs
```

不入库：`target/qa-nvrtc/`（dumper-target 构建缓存、`prod-out/` 产品原始 dump、临时脚本）；证据目录内 **0 个 `.pyc/.bin/.o/.obj`**、无 `__pycache__`、无 `dumper/target`。

## 12. 结论

- **阻断 0 / 发现 0**：入口真实走 NVRTC 运行时编译与产品 API；`product.cu` 胶水-only；拼装指纹 121566/同 sha 独立复现；ABI 496/136 独立推导一致；错误路径 12 个 CPU 单测为真覆盖；**3/3 case 用 QA 自己的 dump 逐字节对 `out1` 全部一致（314/314、3122/3122、80/80），分母独立复算一致**；两轮窗口自洽、无残留。
- **观察 4**：交付峰值采样可能低估（context 瞬时 ~+1.3 GiB）、`121562` 笔误、`Load/Launch` 无 seam 单测、行数按 UTF-8 实际 498/499/499（仍 ≤500）。
