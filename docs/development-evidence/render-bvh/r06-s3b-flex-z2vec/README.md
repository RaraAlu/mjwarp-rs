# R06 z2vec 潜伏缺陷诊断（只读证据；G-L2 后修复）

- 任务：`2026-11-11-render-r06-flex-z2vec-diagnosis`（owner Arch；依据 backlog 卡 `2026-11-04-render-r06-s3b-flex-z2vec-parallel-branch`，p40）
- 诊断时点：HEAD **`d1927155dc73df9564ae17862084a866f7dec2f6`**，`src/render/kernel/cuda/render_flex.cu` sha256 **`fe88cc4c9e80f673768148a854a6da17fdd6d776227e6beb55f2f9866e2ad822`** / 7056 B
- 性质：**只读诊断**。本卡**不改** `src/` / `tests/` / `fixtures/` / `scripts/`，不改判据/阈值/参考/spec，零 GPU、不采集、不开窗。
- 结论一句话：**缺陷属实、可定位到两行、触发面极窄、当前场景不触发；修复排 G-L2 收口后做。**

## 1. 缺陷定位

| 项 | 内容 |
| --- | --- |
| 文件 | `src/render/kernel/cuda/render_flex.cu`（`fe88cc4c9e80f673768148a854a6da17fdd6d776227e6beb55f2f9866e2ad822` / 7056 B） |
| 函数 | `flex_quat_z2vec`（第 **35–64** 行；上游 `math.py::quat_z2vec` 的移植） |
| 出错分支 | 第 **47–53** 行：`if (a < FLEX_MINVAL)` 近平行分支内的 `if (unit.z < 0.0f)` |
| 现状（产品） | 初始化 `quat=(0,0,0,1)`；该分支只写 `quat[0]=1.0f` → 落成 **`(w,x,y,z)=(1,0,0,1)` = 绕 z 轴 90°**（归一化后） |
| 上游真值 | pinned `math.py::quat_z2vec` 该分支 `quat = wp.quat(1.0, 0.0, 0.0, 0.0)` → **`(w,x,y,z)=(1,0,0,0)` = 单位四元数** |
| 实测差异 | `check_z2vec.log`：`vec=(0,0,-1)` → 矩阵差 **max\|prod−up\| = 2.00e+00（match=False）**；其余 6 个探针 ≤5.96e-08 |
| 正确修法 | 该分支内 `quat[0]=1.0f; quat[3]=0.0f;`（其余分支一字不动）——与 backlog 卡 acceptance 一致 |

- 代码摘录（只读，含行号）：`artifacts/render_flex-z2vec-excerpt.txt`。

## 2. 上游真值与「注释≠实现」陷阱（必须写死）

- 分量布局：pinned 上游 `axis_angle_to_quat` 返回 `wp.quat(c, axis[0]*s, axis[1]*s, axis[2]*s)`，`quat_to_mat` 以 `quat[0]` 作 `q00`；`probe_quat.log` 实测 `vec=(0,0,-1)` → quat `[1,0,0,0]` 且 mat = identity、`vec=(0,0,1)` → quat `[0,0,0,1]` 且 mat = diag(−1,−1,1) → **布局是 `(w,x,y,z)` 位置序**。
- **陷阱**：上游 `quat_z2vec` 该分支的注释写「opposite: 180 deg rotation around x axis」，但**实现返回的是单位四元数 `(1,0,0,0)`**（注释与实现不符）。判据目标是**与上游实际输出位级对齐**，因此修法是 `(1,0,0,0)`；若照注释「修正」成 `180° 绕 x = (0,1,0,0)`，反而把分歧放大。
- 同向近平行（`vec→+z`）产品与上游**都**返回 `(0,0,0,1)`（绕 z 180°），逐位一致——**不是缺陷**，不得顺手改。
- 环境：pinned upstream `71da24d9`、Warp **1.15.0**（CPU 设备）、`types.MJ_MINVAL = 1e-15`。

## 3. 触发条件（精确）

- 代码条件：`norm=|vec| ≥ 1e-15` 且 `a = sqrt(unit.x² + unit.y²) < 1e-15` 且 `unit.z < 0`。
- 用 f32 语义：等价于「边方向在 f32 下就是 `(0,0,-1)`」（x、y 为 0 或相对 z 低至 1e-15 以下的次正规量）。
- 现实触发面：**轴对齐的竖直向下 flex 边**（布料/杆件以 z 为轴或整体被旋转到该姿态）。只要不是严格轴对齐，就落入常规 `atan2` 分支（与上游 ≤6e-8 一致）。
- 现有场景核查：`fixtures/render-bvh/scene-a-light/scene.xml` **无 flex**；`fixtures/render-bvh/scene-flex/` 的边方向非 ±z 对齐。故**已验收的 R06/渲染线证据不含该分歧，当前零数值影响**。

## 4. 影响路径

`flex_quat_z2vec` → `flex_quat_to_mat` → `ray_capsule(pos, mat, size = (radius, 0.5·|vec|, 0))`（`render_flex.cu:141-155`）：变换矩阵错 → 该 flex 边的 capsule 姿态错（差 90°）→ 射线求交的 dist / normal 错 → 该像素的 rgb/depth/seg 可能错。

## 5. 现有覆盖 / 未覆盖（CPU harness 边界）

**已覆盖**（`target/s3b-d8/` 原始跑动，本卡已复制入库）：
- `probe_quat.log`：pinned 上游 CPU warp（Warp 1.15.0）对 **7 个向量**（`(0,0,−1)`、`(0,0,1)`、`(0,0,0)`、`(0,0.5,0)`、`(0.5,0,0)`、`(0.6,0.8,0)`、`(−0.6,−0.8,0)`）实测 `quat_z2vec`+`quat_to_mat` 的真值矩阵。
- `check_z2vec_equivalence.py` + `check_z2vec.log`：把 `render_flex.cu` 的公式**逐行转写**为 Python，与上游矩阵逐项比 → 6 个 match、`(0,0,−1)` 红。

**未覆盖**（因此不能宣称设备等价）：
1. 不是执行真 `.cu`——转写不证明 NVCC/NVRTC 下 `atan2f/sinf/cosf` 与 `fmaf` 收缩的实际位型；
2. 无阈值边界（`a` 恰在 `1e-15` 两侧、次正规 x/y、`−0.0`、`unit.z` 正负临界、`norm≈0`）；
3. 无端到端比较（只到矩阵，未到 `ray_capsule` 的 dist/normal/seg）；
4. 仓库内**没有** Rust host 单测（`tests/render_flex_z2vec.rs` 当前不存在，属修复卡产物）；
5. 无设备侧证据。

## 6. CPU 断言与设备证据的边界

- 现有 harness 足以**证明缺陷存在**并**支撑公式级修复**；
- **不足以**宣称「设备端等价/修复已验证」。要拿设备可观测差异，必须构造含 −z 边的 flex 场景 → 属 `fixtures/` 新场景（并可能牵动判据）→ **签名级**，**不建议**为这个 p40 缺陷做。
- 建议收口方式：CPU 断言矩阵（下游修复卡）+ 与上游位型对齐 + 内核身份留痕；**不宣称设备等价**。

## 7. 建议时机：**G-L2 收口后**（判定与理由）

- 该分支**不被 G-L2 场景触发**（`scene-a-light` 无 flex）→ 改它对 G-L2 判据数值零收益；
- 改 `.cu` 会经 `include_str!` 进入 `MEGAKERNEL_SOURCE`（`src/render/kernel/mod.rs:65`）→ **渲染内核 source 哈希与 exe/PTX 身份必变**；G-L2 冻结件/审计口径应绑定已评审代码形态，不宜混入未做设备验证的改动；
- 修复要真正闭环需要设备侧证据，而 G-L2 不能兼做该分支验证 → 前改会留下「已改未验」悬空态；
- 若你/用户要求「下一窗前清掉所有已知缺陷」，则前改的最低要求：四门禁 + CPU 断言矩阵 + 记录内核 source/PTX 前后基线 + 在 G-L2 证据注明「冻结 exe 含 z2vec 修复，但该分支未被场景触发、不构成设备验证」。

## 8. 决策要点

| 问题 | 结论 |
| --- | --- |
| 是否必须修 | **不必须立即修**（不触发、非阻塞）；但**保留卡、不要 abandon**——它是与上游的位级对齐缺陷 |
| 是否重采参考 | **不需要**（不涉 `fixtures/`/参考/判据） |
| 是否需新 PTX/内核基线 | **需要**：改后在新渲染窗记录 source/PTX 基线（属窗口证据，不是重新签名） |
| 片数 | **2 片**：本诊断卡（已固化）+ 修复卡（`.cu` 两行 + `tests/render_flex_z2vec.rs` + 四门禁 + 证据） |
| GPU 窗口数 | **0**（CPU 断言即可收口）；设备实证需新场景 → **+1 窗且签名级，不建议** |
| 需用户签事项 | 改 `.cu` 本身**不需要**（判据/参考未变）；**新增 −z 边场景做设备实证需要** |

## 9. 后续修复卡最小清单（写死，供修复卡直接引用）

1. **代码**：`render_flex.cu:47-53` 分支内 `quat[0]=1.0f; quat[3]=0.0f;`，**其它分支零改动**；`flex_quat_to_mat` 与 `ray_capsule` 不动。
2. **断言**：新增 `tests/render_flex_z2vec.rs`（host，CPU-only），矩阵至少覆盖：
   - 7 个已实测向量（含 `(0,0,−1)` 必须与上游矩阵 `identity` 一致）；
   - **阈值两侧**（`a` 略小于/略大于 `1e-15`）；
   - **次正规 x/y**（如 `vec=(1e-30,0,-1)`）→ 仍走近平行分支；
   - **`−0.0`**（`vec=(−0.0,−0.0,−1.0)`）；
   - **`unit.z` 正负两向**（+z 保持现状、不得改）；
   - **`norm≈0`**（`vec=(0,0,0)` 与 `1e-20` 量级）。
3. **留痕**：改后在新一轮渲染窗证据记录渲染内核 **source sha256 + PTX 字节** 前后值（所有冻结 render exe 的窗口以新身份重冻）。
4. **引用**：本目录证据（`probe_quat.log` / `check_z2vec.log` / `check_z2vec_equivalence.py` / 代码摘录）可直接作为修复卡对照基线。

## 10. 边界与非声明

- 本卡**零 GPU、不采集、不开窗**；**未改** `src/`、`tests/`、`fixtures/`、`scripts/`；未动判据/阈值/参考/spec；四门禁通过；manifest 自核、UTF-8/LF/无 BOM。
- **不宣称**：设备端等价、缺陷已修、任何渲染数值结论；`target/` 原始跑动为 gitignored 工作区，本卡只固化其输出与脚本。
- 修复前该缺陷仍存在于 HEAD；本卡只是诊断与证据。

## 11. 证据清单与来源

| 文件 | 内容 | 来源 `target/s3b-d8/` 原 sha256 |
| --- | --- | --- |
| `artifacts/probe_quat.log` | pinned 上游 CPU warp 7 向量真值（Warp 1.15.0） | `a92d2e8fabac41b86dc293d286ce943a9aee856694a3e458c9219846549b50d9` |
| `artifacts/check_z2vec_equivalence.py` | 产品公式转写 + 与上游矩阵对照脚本 | `3bbc4fb69413b1ccb7dafcb0551453ccdd2632e90e4a6c5eace50bb954fd864c` |
| `artifacts/check_z2vec.log` | 对照输出（6 match / 1 red，差 2.0） | `13701e860c24febea13ff28256c71e99b04dfcf462c57ca3353f6302c23cc92d` |
| `artifacts/render_flex-z2vec-excerpt.txt` | `render_flex.cu` 34–64 行只读摘录（含行号） | （摘自 `fe88cc4c9e80f673768148a854a6da17fdd6d776227e6beb55f2f9866e2ad822`） |
| `receipt.json` / `manifest.txt` | 机读回执 / sha256+bytes 清单 | — |

- 复制方式：源码文件从 `target/s3b-d8/` 读出后**行尾统一为 LF**（内容其余字节不变），源 sha256 见上表/`receipt.json`；`target/` 是 gitignored 工作区，入库后以本目录为准。
