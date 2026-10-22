# U038 三维运动学逐表达式 PTX 对照（工具与结论）

任务：`2026-10-23-u038-3d-ptx-diff-tool`。用户裁决 C：先做只读 PTX 对照工具（有界、不改产品代码），
拿到逐表达式差异量后再决定转录修复（A）还是登记已知债务（B）。
本任务**未改产品源码、容差、断言、fixtures、排除清单或 ABI**；未做 T4 repin；未启动 GPU 内核（只做 NVRTC 编译）。

## 一条命令复现

```powershell
pwsh -NoProfile -File scripts/u038-ptx-diff.ps1
```

前置（本机 pinned 环境，均为 `target/` 下既有产物，脚本会逐项检查并给出缺失原因）：

| 前置 | 默认路径 |
| --- | --- |
| 仓库根 | 脚本上两级目录 |
| NVRTC 12.8 工具链（只加载 `nvrtc64_120_0.dll`） | `target/toolchains/cuda-12.8.1/bin` |
| pinned Warp 缓存（生成 CUDA + 冻结 PTX） | `target/mjwarp-parallel/cache-quarantine/warp-1.15.0-double-version-2026-10-18/wp_mujoco_warp._src.smooth_ca013d6` |
| Warp wheel native 头文件 | `target/mjwarp-parallel/venv/Lib/site-packages/warp/native` |

工具实现：`scripts/u038-ptx-diff.ps1`（入口）+ `scripts/u038-ptx-diff-tool/`（Rust crate，约 900 行，
使用与产品相同的 `cudarc 0.18.2` 动态加载 NVRTC）。输出目录默认 `target/u038-ptx-diff-tool/out/`
（含 `ptx-diff.json`、产品/上游 PTX、各探针 `.cu/.ptx`）；本目录的 `ptx-diff.json`、产品 PTX 摘录与探针
源文件是从该输出复制来的证据副本。工具只读产品源码并按产品同源方式组合编译单元，不改任何产品文件。

## 方法

1. **产品侧 PTX**：从 `src/physics/mod.rs` 逐字提取 `KINEMATICS_CUDA` / `COM_POSITION_CUDA`，
   按产品组合方式拼上 `FIELD_BATCH_CUDA` 前缀（`#define MJWARP_MOCAP` 已定义，对应 3D 链的 `equivalent=false`），
   用产品同款 NVRTC 12.8.93（cudarc 0.18.2）与产品编译选项编译：`--std=c++17`、
   `--gpu-architecture=compute_70`、`fmad=true`、`fast-math=false`。只编译不跑内核。
2. **上游侧 PTX**：pinned 缓存中的生成 CUDA（sha256 `844dca14…`）与冻结 PTX
   （sha256 `4179acd0…`，编译器 `release 12.9, V12.9.86`）。按 Warp 1.15 `wp_cuda_compile_program`
   的 release 选项组合在本机 NVRTC 12.8 重编译：`--gpu-architecture=compute_89`、`--include-path=<native>`、
   `--std=c++17`、`-DNDEBUG`、`-UWP_VERIFY_FP`、`-DWP_ENABLE_MATHDX=0`、`--fmad=true`、
   `--device-as-default-execution-space`、`--extra-device-vectorization`、`--restrict`、
   `--diag-suppress=177,550`（PCH / `--Ofast-compile` 仅 12.9 支持，省略）。
3. **逐表达式探针**：从产品 `KINEMATICS_CUDA` 逐字提取 `rotate`/`multiply`/`normalize` 函数；
   从上游生成 CUDA 逐字提取 `rot_vec_quat_0`/`mul_quat_0`（`normalize` 用 `builtin.h` 的 `wp::normalize`），
   两侧套用同形状的最小内核包装，在**同一 NVRTC 与同一编译选项**下编译，比较各表达式的
   f32 算术指令序列、fma 融合计数与 fma 位置。
4. **内核内验证**：把探针的指令种类序列当作模式，在两端完整内核的 f32 算术序列里查找命中位置，
   判断探针模式是否代表内核实际收缩。
5. **上下文敏感性实验**：同一表达式源码只改变包装内核里的加载顺序（先加载 q / 先加载 v），
   观察 NVRTC 收缩是否变化。

## 自然校验结果

| 校验 | 结果 |
| --- | --- |
| 本机 NVRTC 12.8.93 + Warp release 选项重编译生成 CUDA vs 冻结 12.9.86 PTX | **48/48 个入口指令流完全一致（含寄存器编号）** |
| 带 `--generate-line-info` 的重编译 vs 冻结 PTX | **48/48 个入口指令流完全一致** |
| `--generate-line-info` 是否改变产品 PTX 指令流 | 否（产品 `rigid_kinematics` 指令逐条相同；PTX 文本 sha 不同仅因 `.loc`/`.file` 指令） |

结论：冻结上游 PTX 可在本机完整复现，逐表达式对照不依赖 12.9 工具链；行号映射可信。
产品源码身份：`KINEMATICS_CUDA` sha256 `79459f6b…`，`COM_POSITION_CUDA` sha256 `b8599070…`，
`FIELD_BATCH_CUDA` sha256 `0d019385…`；产品 `rigid_kinematics` PTX sha256 `0bbc1f9a…`，
`rigid_com_position` PTX sha256 `f3baabf0…`；NVRTC DLL sha256 `02d2d7ef…`。

## 逐表达式结果（同 NVRTC、同选项、同形状包装）

| 表达式 | 产品 | 上游 | 分类 |
| --- | --- | --- | --- |
| `rotate` | 30 条算术：fma 13 / mul 12 / sub 4 / add 1 | 30 条算术：fma 13 / mul 12 / sub 4 / add 1 | **一致**（fma 位置完全相同；仅一条非融合 `add` 的调度位置不同） |
| `multiply`（四元数乘） | 22 条：fma 6 / mul 10 / sub 6 | 22 条：fma 6 / mul 10 / sub 6 | **一致**（fma 位置完全相同） |
| `normalize` | 10 条：fma 3 / mul 5 / sqrt 1 / rcp 1 | 10 条：fma 3 / mul 5 / sqrt 1 / rcp 1 | **一致**（fma 位置完全相同） |

即：**在受控的相同包装上下文下，三个表达式的产品源码与上游生成源码编译出的收缩量与指令计数完全一致**，
不存在"表达式写错/少融合"的源级差异。`rotate` 唯一差异是一条独立 `add`（`2*s = s+s`）被调度到不同位置，
不改变任何数值。

## 关键发现：收缩由内联上下文决定，而不是表达式源形态

1. **探针模式不能覆盖内核全部调用点**：`rigid_kinematics` 内 `rotate` 有 5 个调用点，
   产品探针模式只命中 3/5（位置 55/176/260 的算术指令序号）；`multiply` 有 3 个调用点，只命中 1/3；
   `normalize` 3/3 全命中。上游 `_kinematics_branch` 侧同样：`normalize` 3/3，`multiply` 2/3，
   `rotate`（q-first 探针）0/5。
2. **上下文敏感性实测**：同一上游 `rot_vec_quat_0` 源码，只把包装里的加载顺序从 q 先换成 v 先：
   - 上游源码：**13 fma → 10 fma**（算术总数仍 30，mul 12→15），且在冻结上游内核命中 3/5（位置 10/154/241）；
   - 产品源码：13 fma → 13 fma（不敏感）；
   - 候选转录（按上游语句与操作数顺序重写 rotate）：13 fma → 13 fma（不敏感），
     该候选模式在产品内核命中 3/5、在冻结上游内核 **0/5**。
3. **对两轮有界失败的对照解释**：
   - 第 1 轮（改成上游生成代码风格 straight-line 局部乘积写法）：本轮候选转录复现了同一现象——
     只改表达式源形态，产品侧 NVRTC 输出与现状**完全一致**（13 fma 模式、产品内核命中位置相同），
     因此 run7 的"8/77 不变、位差 123"是可预期的，不是尝试不足；
   - 第 2 轮（显式 `mul/add/sub` 禁止 FMA）：上游内核实际是混合收缩（`_kinematics_branch` 共
     99 fma / 284 条 f32 算术），强制 0 fma 与上游方向相反，故 run8 位差 122→147 变差。
4. **结论**：上游收缩模式是"表达式源码 × 内联上下文（操作数加载顺序、寄存器分配、指令调度）"的共同产物，
   不是表达式本身可转录的属性。**源级转录（照抄上游 C++ 写法）无法改变产品侧收缩**；
   要逐位对齐只能按冻结 PTX 逐调用点写显式 `fma.rn`/`mul.f32`/`add.f32` 序列，
   且必须先提取每个调用点的窗口——本工具目前只能匹配到 5 个 rotate 调用点中的 3 个，
   剩余 2 个调用点的窗口未自动分离（需要 PTX 级调用点切片工具）。

## com/cdof 结果（结构性差异，非单点收缩）

| 侧 | 内核 | f32 算术 | 说明 |
| --- | --- | --- | --- |
| 上游 | `_subtree_com_init` | mul 6，无 fma | 初始化 mass·ipos |
| 上游 | `_subtree_com_acc` | **atom.global.add.f32 6**，无 fma | 按树层**原子累加**子树质心 |
| 上游 | `_subtree_div` | div.rn.f32 3、mul 3 | 质量归一 |
| 上游 | `_cdof` | mul 42 / sub 24 / add 0 | xmat 列 + `cross(xmat[i], offset)` |
| 产品 | `rigid_com_position`（单内核） | 112 条 com/cdof 算术：fma 23 / mul 50 / add 21 / sub 18，**0 原子** | 初始化 → 单线程反向遍历累加 → 除法 + 空间惯量 + cdof |

差异类型：**顺序（内核分解/累加顺序不同）**。上游把 subtree_com 拆成 init/acc/div 三个阶段并按树层
原子累加，产品在单线程内按 body 下标反向累加，浮点加法结合顺序不同；该差异无法用 FMA 转录消除。
`cdof` 两侧代数相同（xmat 列 + cross），其位差主要来自 `xmat` 与 `subtree_com` 输入差异。

## 最小修复点评估与建议

- `multiply` / `normalize`：**无需任何修改**（源形态收缩完全一致）。
- `rotate`：**没有可信的源级最小修复**。候选转录实测与现状同码（13 fma、产品内核命中位置相同），
  上游内核实际模式（v-first 类 10 fma）只能通过显式 PTX 算序逐调用点固定，且 5 个调用点中
  本工具只能分离 3 个。
- `com/cdof`：需要单独把产品 subtree_com 累加改成上游分层顺序，即使 rotate 对齐，该差异仍会留下。
- **建议：B（登记已知债务）**。理由：残留差异至少来自两类不可由"表达式转录"消除的原因
  （内联上下文决定的 FMA 收缩 + subtree_com 累加分解顺序），A 路线需要先另立
  「PTX 调用点切片 + 逐调用点显式算序转录 + subtree_com 分层累加对齐」系统任务，
  工作量与风险都超过本有界范围，且不保证 1477 项 0 超差。
- 若用户仍选 A：本工具已提供可复现基座（冻结 PTX 48/48 可本机复现、行号映射可信、
  逐表达式探针与上下文实验可重跑），下一步只需补"调用点窗口切片"与逐点模式导出。

## 边界与未验收

- 未跑 GPU 内核、未重跑 1477 项定向验收（8/77 超差维持 run6/run5 状态，本任务不改变产品数值）；
- 未改产品源码/容差/断言/fixtures/排除清单/ABI，未做 T4 repin；
- 逐表达式覆盖限 `rotate`/`multiply`/`normalize` 与 com/cdof 内核结构；未逐式覆盖矩阵、
  `axis_angle_to_quat`、`xipos` 等其他表达式；
- 内核内实例覆盖为"探针模式命中统计"，未完整分离全部调用点窗口（见上）；
- 工具依赖本机 pinned 环境（NVRTC 12.8 + pinned Warp 缓存 + wheel native 头文件），
  仅为离线诊断工具，不进入产品构建或测试链。

## 证据文件

- `ptx-diff.json`：完整机器可复核数据（输入/编译身份与哈希、校验、逐表达式指令序列与融合位置、
  内核命中位置、上下文敏感性、com/cdof 计数、汇总）。
- `product-kinematics.ptx` / `product-com.ptx`：产品侧编译产物摘录。
- `probes/`：逐表达式探针源码（产品/上游/候选），即实际编译输入。
