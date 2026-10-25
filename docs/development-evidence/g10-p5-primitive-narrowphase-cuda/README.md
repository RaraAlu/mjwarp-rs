# G10@P5 设备缺口片：primitive narrowphase 设备版（SPHERE-BOX / CAPSULE-BOX）

- 任务：`2026-10-25-g10-p5-primitive-narrowphase-cuda`（p55，Backend）。
- 状态：设备实现 + CPU 验证 + GPU 窗口对照完成；门禁回执见 §7。
- 上游：pinned `71da24d956378a87a703b6e1442b13aec0c4ac29`；本片引用的冻结源在
  `docs/evidence/d04-complex-physics/authoring-runs/run-5/frozen-source/_src/`
  （`collision_driver.py` 的 `MJ_COLLISION_TABLE`、`collision_primitive.py` 的 wrapper/dispatch）。

## 1. 结论（只覆盖本片 3 个冻结 case）

产品入口 `CollisionGpuProgram::new(0)` + `run(CollisionKernel::Primitive, ..)`（NVRTC 运行时编译）
在 `sphere-box-shallow` / `sphere-box-deep` / `capsule-box` 三个 case 上，与 QA 已提交的上游 f32 GPU
参考按 G5 口径逐字段比较（先按 `(worldid, dist, pos, frame)` 稳定排序，位级优先）：

| case | nacon | nccd | 位级比较 | max_dist_ulp | max_pos_ulp | max_frame_ulp | failures |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `sphere-box-shallow` | 2 | 0 | **80/80** | 0 | 0 | 0 | 0 |
| `sphere-box-deep` | 2 | 0 | **80/80** | 0 | 0 | 0 | 0 |
| `capsule-box` | 2 | 0 | **80/80** | 0 | 0 | 0 | 0 |

- 3 个 case 均 `nworld=2`、每 world 1 个有效接触（共 6 个接触点），`overflow` 全 0。
- 每 case 的 80 项 = 接触字段 70 项（`dist` 2、`pos` 6、`frame` 18，`includemargin`/`friction`/
  `solref`/`solreffriction`/`solimp`/`adhesion`/`dim`/`geom`/`worldid`/`type`/`geomcollisionid`
  共 44）+ `overflow` 2 项 + `efc_address` 8 项（= `nworld × efc_width`，核对产品初值 `-1`）。
- 明细：`window-output.txt`（三条命令原文）、`window-log.txt`（时间戳/基线/守卫/峰值/残留）。

同一窗口内还跑了 16 段拼装后的 **NVRTC launcher 回归**（防拼装/PTX 变化把既有 3 case 带崩）：
`hfield-sphere` 314/314、`sdf-box` 3122/3122、`ellipsoid-box` 80/80，均全位级、ULP 0、failures 0。

## 2. 实现对照表

上游 `MJ_COLLISION_TABLE`（frozen `collision_driver.py` L47-80）里 `CollisionType.PRIMITIVE` 的
dispatch 组合共 12 个；本片实现其中 2 个（必做项），其余 10 个如实记未覆盖（§6）。

| 上游 dispatch 组合 | 设备实现 | 入口/选择 | 对照 case | 本片结果 |
| --- | --- | --- | --- | --- |
| `(SPHERE, BOX)` / `(BOX, SPHERE)` | ✅ `g5_sphere_box` + `g5_sphere_box_wrapper` | `mjwarp_collision_primitive` 按 `type1/type2` 选路；盒在前时交换实参 | `sphere-box-shallow`、`sphere-box-deep` | 80/80 位级、ULP 0 |
| `(CAPSULE, BOX)` / `(BOX, CAPSULE)` | ✅ `g5_capsule_box` + `g5_capsule_box_wrapper` | 同上；胶囊轴取 `cap.rot[2],[5],[8]`（上游 mat33 第 3 列） | `capsule-box` | 80/80 位级、ULP 0 |
| `(PLANE, SPHERE)` | 未实现 | — | — | 未覆盖 |
| `(PLANE, CAPSULE)` | 未实现 | — | — | 未覆盖 |
| `(PLANE, ELLIPSOID)` | 未实现 | — | — | 未覆盖 |
| `(PLANE, CYLINDER)` | 未实现 | — | — | 未覆盖 |
| `(PLANE, BOX)` | 未实现 | — | — | 未覆盖 |
| `(PLANE, MESH)` | 未实现 | — | — | 未覆盖 |
| `(SPHERE, SPHERE)` | 未实现 | — | — | 未覆盖 |
| `(SPHERE, CAPSULE)` | 未实现 | — | — | 未覆盖 |
| `(SPHERE, CYLINDER)` | 未实现 | — | — | 未覆盖 |
| `(CAPSULE, CAPSULE)` | 未实现 | — | — | 未覆盖 |

补充说明：

- `(BOX, BOX)` 在上游 table 中是 `CONVEX`（`# overwritten by NATIVECCD disable flag`），**不属于**
  primitive dispatch；本片不实现、也不引用 `box-box-margin` 的参考。
- `primitive.cu` 的数值内核按上游 f32 语义转写：`g5_sphere_box` 单接触；`g5_capsule_box` 输出两个
  接触（含 corner/edge/face 第二接触分支），无效第二接触 `dist = MJ_MAXVAL`，由 `g5_write_contact`
  的检出语义跳过（与上游 `write_contact` 计数一致）。
- 上游 `capsule_box_wrapper` 的平面盒面方向选择（`c1` 编码）与 `make_frame` 复用 `common.cu` 的既有
  实现（CPU 单测断言拼装源里 `make_frame` 只有一处定义）。
- 新内核走既有拼装管线：`src/physics/collision/cuda/{primitive.cu, primitive_wrapper.cu}` 追加进
  `DEVICE_SOURCES`（16 段），`product.cu` 末尾新增唯一入口 `mjwarp_collision_primitive`。

## 3. 参考与判据（均沿用既有口径，未改）

- **参考**：`docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-upstream-gpu.json`
  （QA 独立复核已提交）。该捕获 `cases` 含本片 3 个 case，且每个 case 带
  `two_captures_raw_bitwise_identical=true`；`provenance.reads_product_tree=false`，版本
  python 3.12.13 / mujoco 3.15.0 / warp 1.15.0 / mujoco_warp 3.15.0 / numpy 2.5.4 / device `cuda:0`，
  脚本 sha256 `5cf2f746…`。
- **复用依据（不新开 authoring 捕获）**：G5 冻结 case 的 23 项输入 bin 与 QA 参考 `input_bins` 逐项
  sha256、元素数相同；测试运行时重算 23 项 sha256 做守卫（`tests/support/primitive_qa_support.rs`
  L18-111）。任一项不同测试即失败，杜绝「输入换了还比数」。
- **判据**（G5 既有口径，本片不改）：
  - 结构/计数：`nacon` 对 `count` 位级；`nccd` 期望 0（primitive 不消耗 CCD 槽，参考为 null）；
    `overflow` 位级；`efc_address` 逐项期望 `-1`（上游 `write_contact` 只写 -1）。
  - 非位级：`dist` 自身量级 ≤4 ULP（`ref == 0` 用 4 ULP@1.0）；`pos` 按 `scale = max_i |ref_i|` 的
    `4·ULP(scale)`；`frame` 按 scale=1.0 的 4 ULP。
  - 参数/整数（`includemargin`/`friction`/`solref`/`solreffriction`/`solimp`/`adhesion`/`dim`/`geom`/
    `worldid`/`type`/`geomcollisionid`）按位比较、零容差；位级相等优先。
- 实现：`tests/support/primitive_compare_support.rs`；入口测试 `tests/g10_p5_primitive_cuda.rs`
  （按 case 早停：首个失败 case 打印明细后 panic，后续 case 不跑）。

## 4. GPU 窗口事实

- 窗口：2026-10-25 22:57:25 → 22:57:59（**elapsed 36.3 s**），命令与原文输出见
  `window-output.txt`、`window-log.txt`、`window-memory-samples.txt`（121 点）。
- 起点 HEAD **`7e5f4f4392fc0cbfa768e42300ba79a6690d8e50`**（`head_match=true`）；窗口只跑窗口外
  预构建的三个 test binary，**窗口内不编译产品代码**。
- 基线：used **3722 MiB** / free 12340 / 31°C（relative 提交前记录 3665 漂移 +57 MiB，<200 阈值）；
  `query-compute-apps` 中本任务相关进程 **0**；桌面 top5（工作集）vmware-vmx 17701 MiB、
  DeepSeek Harness 1634 MiB、Code 997 MiB、MsMpEng 757 MiB、explorer 668 MiB。
- 硬件/驱动/工具链：RTX 4070 Ti SUPER 16376 MiB、driver 596.36、nvcc 12.8.93、
  CPU AMD Ryzen 9 9950X3D 16C/32T；`PATH` 前置 `target/toolchains/cuda-12.8.1/bin`；
  NVRTC DLL sha256：`nvrtc64_120_0.dll=02d2d7ef…`、`nvrtc64_120_0.alt.dll=5464ea01…`、
  `nvrtc-builtins64_128.dll=a7afa2a4…`。
- 预构建 binary（sha256 / mtime）：primitive `3c34b809…` / 22:35:37；launcher `fbf3d439…` /
  22:35:36；lib 测试 `16d70395…` / 22:35:34。
- 步骤结果：
  - **A CPU 侧 NVRTC 冒烟**（不创建 CUDA 上下文）`exit 0`，1 passed（9.72 s）：
    拼装源 **len=138520 / sha256=`a45169bc…`**，PTX **1,983,583 B / 4 入口**。
  - **B NVRTC launcher 回归** `exit 0`，1 passed（13.64 s）：hfield-sphere 314/314、
    sdf-box 3122/3122、ellipsoid-box 80/80，ULP 全 0。
  - **C primitive 3 case** `exit 0`，1 passed（9.63 s）：见 §1 表。
- 峰值/守卫/残留：0.25 s 采样 121 点，峰值 used **3919 MiB（+197）**、util max 57%、temp max 32°C；
  守卫口径「增量 ≥3072 MiB 或 >总显存 50%（8188 MiB），取更严」**未触发**；结束时 used **3689 MiB**，
  跑前跑后 `query-compute-apps` 本任务相关进程均 **0**，无残留。
- 证据 sha256：`artifacts-sha256.txt`（17 项：源码/测试/脚本/输出）。

## 5. 设备源与 ABI

- 设备源拼装保持确定性：`DEVICE_SOURCES` 16 段，顺序
  common→geom→support→simplex→gjk_core→polytope→polytope_build→epa→ccd→sdf_octree→sdf_math→
  sdf_narrowphase→hfield→contact→primitive→primitive_wrapper，最后追加 `product.cu`；
  拼装前删除本地 `#include "*.cu"`，其余 `#include`/`#pragma once` 原样保留。
- PTX 4 入口：`mjwarp_collision_contact`、`mjwarp_collision_hfield`、`mjwarp_collision_sdf`、
  `mjwarp_collision_primitive`；入口唯一性、`make_frame` 单一定义由 CPU 单测
  `collision_gpu/primitive_tests.rs` 覆盖（4 条）。
- **ABI 未改**：`collision_gpu/abi.rs` 204 行未修改；`CollisionArgs` 496 B/55 字段、
  `CollisionContactOut` 136 B/17 字段与 NVRTC launcher 片一致；primitive 复用 `contact.cu` 的
  `g5_write_contact` 与 `common.cu` 的 `make_frame`，不新增输出缓冲。
- launcher/入口在 `#[cfg(feature = "cuda-probe")]` 下启用；默认构建不含 CUDA 产品入口（§6 未覆盖）。

## 6. 未覆盖与边界

1. **上游 primitive dispatch 其余 10 个组合未实现**（§2 表）：plane-sphere、plane-capsule、
   plane-ellipsoid、plane-cylinder、plane-box、plane-mesh、sphere-sphere、sphere-capsule、
   sphere-cylinder、capsule-capsule。
2. **没有既有 host 参考可做交叉核对**：G5 host 交付 case 只有 9 个（convex-mesh、cylinder-box、
   degenerate-mesh、ellipsoid-box、hfield-box、hfield-capsule、hfield-sphere、mesh-mesh、sdf-box），
   不含 SPHERE-BOX / CAPSULE-BOX；按任务验收「造不出参考的 case 记未覆盖，不伪造」记录，未伪造
   host 参考、未把 QA 上游捕获当作 host 参考。
3. **CAPSULE-BOX 第二接触分支未被样本触发**：本片 3 个 case 每 world 只产生 1 个有效接触
   （另一行 `dist = MJ_MAXVAL` 被检出语义跳过）；`g5_capsule_box` 的 corner/edge/face 第二接触
   分支已按上游转写但没有值级证据。
4. **未接入默认产品管线**：产品入口经 `cuda-probe` 特性启用；默认构建不含 CUDA 产品入口。
5. **样本规模**：3 个冻结 case × 2 world × 1 接触/world = 6 个接触点，且只取上游 primitive
   dispatch 的 2/12 组合；不覆盖旋转/尺度/深穿透边界之外的新姿态，不据此扩展结论。
6. 未做性能测试（本片无性能判据）；未新增 authoring 捕获（复用 QA 已提交参考 + 输入 sha 守卫）。
7. 不改判据、不改已 verified 证据数字、不改 `Cargo.*` / fixtures XML；不声明本片 3 case 之外
   任何精度/等价结论。

## 7. 验证与门禁

- CPU 侧：
  - `cargo test --locked --features cuda-probe --lib runtime::collision_gpu -- --include-ignored`
    → **17 passed / 0 failed**（12 CPU 单测 + NVRTC 冒烟；含 `primitive_tests` 4 条）。
  - `cargo test --locked --features cuda-probe --test g10_p5_primitive_cuda --no-run`、
    `... --test g10_p5_nvrtc_launcher --no-run`、`... --lib --no-run` 均通过（预构建）。
  - `cargo fmt --check` 通过。
- GPU 窗口：见 §4（A/B/C 全部 exit 0；无 case 超判据，无需早停）。
- `normify gate` / `normify check`：见 `receipt.json` 的 `gate` 字段（门禁绑定源码指纹，改动后已重跑）。
