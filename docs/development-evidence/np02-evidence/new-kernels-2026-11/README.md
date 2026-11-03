# NP02 新增 12 个内核的精度证据（2026-11-04，`2026-11-04-precision-np02-new-kernel-evidence`）

本目录记录 2026-11-04 回执重算发现的 **12 个「无任何 NP02 回执」内核**的补采结果：逐个给出源单元、配方、`sourceSha256`、`ptxSha256`、门禁结论，以及两处必须解释清楚的配方差异（渲染配方内核、装配源内核）。采集**全程 CPU-only**：只调用 NVRTC 12.8.93，不创建 CUDA 设备上下文，**零 GPU、零窗口、零参考采集**。

对象来自 `../inventory.json` 当时 12 条 `evidence_state=missing` 行：`build_rays`、`mesh_bvh_ray`、`scene_bvh_ray`、`s7a_abi_probe`、`aa_resolve`、`sample_texture2d`、`mjwarp_collision_{boxbox,contact,contact_multi,hfield,primitive,sdf}`。

## 1. 结论

- 12 条回执全部采集成功、门禁全部 pass；登记后清单 **有证据 88→100、缺失 12→0、覆盖 88%→100%、源身份一致 77→89**；回执 **701→713**、PTX 实体 **697→709**、缺口 **44→32**（缺的 12 条是「缺失证据」缺口，补上即消失）。
- **仍不宣称 NP02 通过**：9 条历史源身份漂移与 4 条历史缺实体（`small_prefix_factor_{3,4,5,6}`）按原裁决保持不动；本片只补证据与计数。
- 4 条先前已有独立切片记录的单元，本次复算得到**逐字节相同的 ptxSha**（见 §2），因此「复用」是可复现的而非只凭旧记录。

## 2. 采集器与自校验

- 采集器：`scripts/np02-evidence/np02-capture-new-kernels.py`（CPU-only；`capture` 写实体、`index` 幂等登记补采索引）。
- 复刻的产品编译路径（逐字段对齐）：

| 配方 | 产品入口 | NVRTC 选项 |
| --- | --- | --- |
| physics | `src/runtime/transfer/kernel.rs::SynchronousKernel::compile` | `--fmad=true`、`--std=c++17`、`--gpu-architecture=compute_70`（`use_fast_math=false` 不下发）；`name="<entry>.cu"` |
| render | `src/render/kernel/launch.rs::compile_render_artifact` + `src/runtime/launcher/artifact/nvrtc.rs::nvrtc_compile` | 同上 + `--include-path=include/warp-headers` + `--device-as-default-execution-space`；门禁 `f32-render` |
| collision | `src/runtime/collision_gpu/source.rs::assemble_device_source` | 物理配方（见 §4） |

cudarc 0.18.2 的 `CompileOptions::build()` 顺序为 `--fmad`、`--include-path…`、`--gpu-architecture`，最后追加调用方 options；PTX 文本按 `Ptx::to_src()` 取 `CStr`（去掉尾部 NUL），`ptxSha256` = 该文本 UTF-8 的 sha256。

**自校验（4/4 逐字节命中）**：对已有独立切片记录的 4 个单元复算，ptxSha 与旧记录完全相等——

| 内核 | 旧记录位置 | 复算 ptxSha |
| --- | --- | --- |
| build_rays | `render-bvh/s5-independent-check/ptx-record.json` | `3666ff54b3dc052a0d0ea6a2317f447875445b78361f75285a1b73985e2d21a1` |
| mesh_bvh_ray | `render-bvh/s3-independent-check/ptx-record.json` | `f4e299fce72f98f178486cfa62f340800f3dd99349bca58702129c4577804665` |
| scene_bvh_ray | `render-bvh/s2b-independent-check/ptx-record.json` | `37cea6a50776f39c862ef2885dedf8b32d45d89392626bbe264b60d89f2699c3` |
| sample_texture2d | `render-bvh/s4-independent-check/ptx-record.json` | `a880edca421f32df956470e0f2ab37dc967f843f1d3135ab607bb1a4e6545ce8` |

## 3. 12 条回执

| entry | 配方 | 源单元 | `sourceSha256` | `ptxSha256` | PTX 字节 | 门禁 |
| --- | --- | --- | --- | --- | --- | --- |
| `build_rays` | physics | `src/render/camera/cuda/build_rays.cu` | `e38cdf2a29d529ee3cae76f88db476fc999ff555f992b8219c938664ac063577` | `3666ff54…d21a1` | 9,427 | f32-physics pass（sin/cos 归约 1 处） |
| `mesh_bvh_ray` | physics | `src/render/bvh/cuda/mesh.cu` | `75ae35acc44125e4c4fa18eac7086f68be325167c22b5a3443ca841eef42fd46` | `f4e299fc…804665` | 16,944 | f32-physics pass |
| `scene_bvh_ray` | physics | `src/render/bvh/cuda/scene.cu` | `8dad4d61235a8686d8b43fecef450535816c5a9423f15d256be4aab9a714d56d` | `37cea6a5…2699c3` | 34,938 | f32-physics pass |
| `sample_texture2d` | physics | `src/render/texture/sample.cu` | `145087f2de1c7127ed3cbc36a97bfef8e3ab227927bae053c812b513583e42da` | `a880edca…545ce8` | 1,689 | f32-physics pass |
| `aa_resolve` | **render** | `src/render/kernel/cuda/aa_resolve.cu` | `0bf8af5a68f9cae50d0cb9cc4da2a01da2607066cf1fe3072c8345e6e104bc76` | `b28e155c…66ec8a70` | 27,536 | f32-render pass（见 §5） |
| `s7a_abi_probe` | **render** | `src/render/kernel/launch.rs`（内联 `PROBE_SOURCE`） | `4c796b38331a0ca2821142e24eaf67829754c20436f337836da9ba000b360125` | `06a6c542…1558ec4` | 30,091 | f32-render pass（见 §5） |
| 6× `mjwarp_collision_*` | physics（装配源） | `src/physics/collision/cuda/product.cu`（片段） | `12d50d68d3e7f155e688f9dab2f69d6d30720a6b07804c7a0a026dd38dbd34b7`（装配源） | `54ab59e3…3369a40b` | 3,883,769 | f32-physics pass（6 entry 共用同一 PTX） |

实体按审计约定写成同目录 `<entry>-<ptxSha>.{json,cu,ptx}`；回执 JSON 与 `src/runtime/transfer/kernel/ptx.rs::record_physics_ptx` 写出的字段同构（`entry`/`sourceSha256`/`ptxSha256`/`precisionGate`/`cudaMathReduction`/`passed`/`nvrtcVersion`/`fmad`/`useFastMath`/`options`）。

## 4. collision：编译单元是装配源，不是片段

`src/runtime/collision_gpu/source.rs` 把 `DEVICE_SOURCES` 登记的 **28 个片段**按依赖拓扑序拼接（删掉各片段里的本地 `#include "*.cu"` 行、其它 `#include`/`#pragma once` 原样保留），最后追加 `product.cu`：

- 装配源 **254,263 字节**、`sourceSha256 = 12d50d68…`（采集器从 `source.rs` 解析片段顺序后独立重放，非手抄）。
- 而 `inventory.json` 该 6 行的 `source.path` 仍是片段 `product.cu`（`cad08d3e…`，18,302 字节）——这是扫描器「按 const 定位源单元」的口径，不是编译单元。
- 因此回执的 `sourceSha256` 记装配源；`np02-evidence.ps1` 对该情形走既有回退 `captured-cu-contains-current-const（参数化/拼接编译单元）`：要求回执旁 `.cu` 实体哈希 = 回执 `sourceSha256`，且其文本**完整包含**当前片段文本——两条都成立，故 6 行均为 `verified`。
- **6 个 entry 的 PTX 完全相同**（`54ab59e3…`，3,883,769 B）：entry 名不影响字节，一次编译产出 6 条回执。审计按 `<entry>-<ptxSha>.ptx` 逐条找实体，所以目录里有 **6 份同内容 .ptx + 6 份同内容 .cu**；工作树因此 **+23.3 MB**，但 git 对相同内容只存 1 个 blob（`git add` 去重），仓对象体积约 +4.1 MB。
- 该目录（`new-kernels-2026-11/**`）已在 `../.gitattributes` 标 `-text -diff`，避免检出时 EOL 改写破坏实体哈希。

## 5. `aa_resolve` / `s7a_abi_probe`：两条 f64 token 是 W15 死数据常量

两者源码都 `#include "builtin.h"`，真实编译入口是渲染配方（`WARP_1_15_RENDER`，`f32-render` 门禁），不是物理配方。逐条事实：

- **token 定位**：两份 PTX 各**只有 2 个** `.f64` token，就是第 32/33 行：

  ```text
  .global .align 8 .f64 _ZN2wp11_svd_configIdE11SVD_EPSILONE = 0d3D719799812DEA11;
  .global .align 8 .f64 _ZN2wp11_svd_configIdE17QR_GIVENS_EPSILONE = 0d3D719799812DEA11;
  ```

- **不参与计算**：两个符号在各自 PTX 中各**只出现 1 次**（即只有声明行，无任何指令/操作数引用）；字面量 `0d3D719799812DEA11` 解码为 `1e-12`。
- **来源**：`include/warp-headers/svd.h:42-44` 的 `template <> struct _svd_config<double>` 两条 `static constexpr double` 成员，经 `include/warp-headers/builtin.h:2804` `#include "svd.h"` 带入。
- **产品代码引用检查**：`src/**` 内除渲染门禁白名单与文档注释外 **0 处**引用这两个符号；两个内核源文本内 `svd/SVD` 字样 **0 处**。
- **放行依据**：`src/render/kernel/mod.rs::check_render_ptx` 只允许 f64 两种形态——逐字节精确匹配的两条 `.global` 常量声明（各至多一次；重复 / 错位 / 改字面量 / `other_config` / `.const` 形态一律拒绝）+ sin/cos 整数角归约三元组，并要求 `.visible .entry <entry>` 存在。QA 把该判定独立移植后跑这两份 PTX，**均 pass**；同一移植实现对 `mod.rs` 内 11 个注入/变异用例的判定与产品交付的独立复核一致（见 `render-bvh/s7a-independent-check/`）。
- 因此 `inventory.json` 这两行的 `compile.precision_gate` 记 `f32-render`；`scripts/np02-evidence/np02-spec.json` 的 `product_recipe.gate_overrides` 显式声明这两个 entry，重跑扫描器可复现，不靠手工改清单。

## 6. 棘轮同步（本次实际改动）

| 计数 | 之前 | 之后 |
| --- | --- | --- |
| `evidence_present` | 88 | **100** |
| `evidence_verified` | 77 | **89** |
| `evidence_missing` | 12 | **0** |
| `coverage_pct_evidence` | 88 | **100** |
| `coverage_pct_source_identity` | 77 | **89** |
| `receipts_total` / `receipts_file_hash_ok` | 701 / 701 | **713 / 713** |
| `receipt_ptx_entities_verified` | 697 | **709** |
| `receipt_ptx_entities_missing` | 4 | 4（历史缺实体不动） |
| `gap_items` | 44 | **32** |

- 补采索引 `../receipt.json` 追加 12 条（幂等：重复执行不会重复插入；当前 13+12=25 条）；主索引 `docs/development-evidence/frozen-inventory/receipts.json`（688 条）**未改动**。
- `inventory.json` / `gap.json` 用**已证明逐字节等价**的序列化器重写（先用仓内旧清单做 round-trip：旧文件经该序列化器可逐字节复现 258,028 / 20,923 B），因此格式与仓内既有风格一致，不是 PS 5.1 的原始输出。
- `tests/precision_evidence_gate.rs` 快照同步（含新增 `np02_new_kernel_rows_stay_verified` 断言：这 12 行必须保持 `verified` 且至少 1 份回执），`cargo test --locked --test precision_evidence_gate` **4/4 通过**。

## 7. 复现

```powershell
cd D:\Rust\mjwarp-rs
# 1) 采集（CPU-only，约 90 s；不需要 GPU/窗口）
target\mjwarp-parallel\venv\Scripts\python.exe scripts\np02-evidence\np02-capture-new-kernels.py capture
# 2) 幂等登记补采索引
target\mjwarp-parallel\venv\Scripts\python.exe scripts\np02-evidence\np02-capture-new-kernels.py index
# 3) 重算清单（Windows PowerShell 5.1；脚本已 BOM 硬化，可直接执行）
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\np02-evidence\np02-evidence.ps1 `
  -ProductRoot D:\Rust\mjwarp-rs -OutDir target\np02-p63-out
# 4) 门禁
cargo test --locked --test precision_evidence_gate
```

注意：PS 5.1 的 `ConvertTo-Json` 缩进/分隔与仓内既有清单（PS 7 生成）不同；重算输出请写临时 OutDir，再按 §6 的等价序列化器落盘，**不要**直接用 5.1 原始输出覆盖本目录清单。

## 8. 边界

- 零 GPU、零窗口、零参考采集、零网络；不改 `src/`、`fixtures/`、判据/容差/参考。
- 不宣称 NP02 通过、不宣称双平台；9 条历史漂移按用户 2026-10-24 裁决 B 保持不重采不重基线；4 条历史缺实体（`small_prefix_factor_{3,4,5,6}`）保持 `historical-unrecoverable`。
- `receipt_ptx_entities_missing` 仍为 4 是历史遗留，与本批无关。
