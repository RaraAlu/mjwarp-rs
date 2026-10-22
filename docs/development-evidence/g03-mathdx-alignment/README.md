# G03 分块求解 MathDx 算序对齐：第一阶段可行性验证

任务：`2026-10-22-g03-tile-cholesky-mathdx-alignment`（第一阶段，只做可行性）
日期：2026-10-22
基线：`d850be0`（登记）/ `10913ce`（W-D4 与阶段证据之后）
授权：Chief 2026-10-22 授权的 A1 有界尝试；A2/B 不做；第二阶段需第一阶段可行且 Chief 确认。
**本轮结论：停手，不进入第二阶段。** 反推本身可行且已完成，但逐位一致的闭环在现有冻结数据下不可证明，且准入缺口主因不在 factor/solve 算法。

## 方法（全部离线，产品链无新依赖）

1. 用既有 `native-mathdx7-link` 先例的 native 路线，把 authoring 缓存中 11 维 MathDx LTO
   链接成 CUBIN：pinned `warp.dll` 的 `wp_cuda_compile_program` C ABI（MSVC + CUDA 12.8.1 头文件，
   **未启动 Python 解释器**）。识别结果：
   - `potrf_11` = `target/g03-block-reference/warp-cache/1.15.0/lto/cdd46856c1496194.lto`
   - `potrs_11` = `.../lto/004259c22f2bb44f.lto`
   - 通用 `FATBIN` = `.../lto/004259c22f2bb44f_fatbin.lto`
2. 用 CUDA 工具链的 `nvdisasm` 反汇编（`target/toolchains/cuda-12.8.1/nvdisasm/.../nvdisasm.exe`），
   读取 11×11 MathDx 代码完整指令序。
3. 写离线 oracle（CUDA driver 加载 CUBIN，喂冻结 fixtures 输入），把 MathDx 输出与冻结上游/产品值
   逐位比较；另用 `probe_mass_matrix` 的公开产品入口 dump 产品矩阵。

工具源码已收入 `tools/`（`link11.cpp`、`oracle.cpp`、`call-factor11.cu`、`call-solve11.cu`、
`dump_matrix.rs`），逐项数据见 `oracle-results.json`。

## 关键结果

### 1) 反推可行（已完成）

| 产物 | 尺寸 | 说明 |
| --- | ---: | --- |
| potrf_11 CUBIN | 2102928 B（sha256 `36E7F4…C765`） | kernel `mathdx11_factor`，32 线程 |
| potrs_11 CUBIN | 2100608 B（sha256 `0B09F0…E50C`） | kernel `mathdx11_solve`，64 线程 |

SASS 摘要：potrf_11 为协作式代码（`MUFU.RSQ` 12、`BAR.SYNC` 4、`SHFL` 58、`FFMA` 35 等）；
potrs_11 同样紧凑（`MUFU.RCP` 6、`BAR.SYNC` 2、`SHFL` 21、`FFMA` 31、`FADD` 4 等）。
指令序可完整读出，说明「离线反推到可对齐的指令序」这一步本身**可行**。

### 2) oracle 把缺口定位到矩阵链，而不是 factor/solve

| 实验 | 结果 |
| --- | --- |
| MathDx potrs(冻结上游 factor, rhs) vs 冻结上游 solution | **bit_diff 0/11（3 条 rhs 全部）** |
| MathDx potrf(产品矩阵) vs 产品 factor | **bit_diff 0/121，status=0** |
| MathDx potrf(原生 fixture 矩阵) vs 冻结上游 factor | bit_diff 56/121，max_abs 1.03e-4 |
| MathDx potrs(产品 factor, rhs) vs 冻结上游 solution | bit_diff 11/11，max_abs 7.42e-3；对原生 score **1.291/1.182**（红线 0.972/0.462） |
| 产品解 vs MathDx potrs(产品 factor) | rhs1 8/11、rhs2 10/11 位不同，max_abs 3.05e-5 |

解释：

- 冻结上游 `solution` 恰好等于 MathDx potrs(上游 factor, rhs)，**求解目标已被 oracle 精确锁定**；
- 产品分块因子实现已经与 MathDx potrf(产品矩阵) **逐位一致**（现有 rsqrt.approx/融合约束仿真已对齐）；
- 因此产品 factor 与上游 factor 的 20/121 位差异全部来自 **产品矩阵 ≠ 上游矩阵**；
- 即使换成 MathDx 求解，用产品 factor 的 score 仍是 1.291/1.182 > 红线，说明**只对齐求解不足以过准入**；
- `reconstruct(U_up^T U_up)` 与产品矩阵有 75/121 位、max_abs 1.907e-6 的差异（reconstruct 自身含舍入，只能当残差上界）；
  冻结参考行**不含上游 matrix**，无法得到精确上游矩阵作为对齐目标。

## 可行性判断与停手原因

- **反推可行**：11 维 MathDx 代码已离线取得 CUBIN 与 SASS，无外部运行时组件要求。
- **求解对齐可做**：目标明确（oracle 已验证），产品回代仅余 8/11、10/11 位末位差，可继续对齐；
  但它**单独不能**让 3 项探针过准入（见上表 1.291/1.182）。
- **不可证明闭环**：准入缺口的主因是上游 matrix 的末位差，而上游 matrix 不在冻结数据中；
  在「不启动 pinned Python/authoring、不改容差与准入规则」的约束下，无法构造可信的逐位一致判据。
- 按 Chief 规则：**第一阶段判为不可保证可控闭环，停手并登记债务，不进入第二阶段实现。**

## 解除阻塞的决策项（需 Chief/用户裁决）

1. **离线 authoring 例外**：批准用 pinned 环境（Python 3.12 + mujoco 3.15.0 + warp-lang 1.15.0 +
   固定 mujoco_warp 提交）离线生成上游 `matrix` 静态参考，作为新冻结证据（定义范围与边界；产品测试仍禁 Python）；或
2. **有界搜索**：允许以上游 factor 为目标函数，对矩阵链（kinematics/com_pos/crb/matrix）的末位
   舍入点做有界对齐搜索（不保证闭合，需另立任务与预算）；或
3. **用户裁决 A2/B**：改数值口径或引入外部组件（本轮授权之外）。

## 复现命令（本机离线）

```powershell
# 1) 编译离线链接器（MSVC + CUDA 头文件）
#    tools/link11.cpp -> target/mathdx11/link11.exe
# 2) 链接 11 维 MathDx LTO（pinned warp.dll，无 Python 解释器）
target/mathdx11/link11.exe <venv>\warp\bin\warp.dll <venv>\warp\native `
  target/g03-block-reference/warp-cache/1.15.0/lto/cdd46856c1496194.lto `
  target/g03-block-reference/warp-cache/1.15.0/lto/004259c22f2bb44f_fatbin.lto `
  tools/call-factor11.cu <outdir> potrf_11_11_1_89_32_1_1_5_x_x_0
# 3) 反汇编
nvdisasm -c <outdir>\potrf_11_11_1_89_32_1_1_5_x_x_0.cubin
# 4) oracle（factor/solve 输入为冻结 fixtures 转出的 f32 原始数组）
target/mathdx11/oracle.exe <cubin> factor <matrix.f32> <out> mathdx11_factor
target/mathdx11/oracle.exe <solve.cubin> solve <factor+rhs.f32> <out> mathdx11_solve
```

## 边界

- 只做第一阶段：未改产品 `src`、测试、容差、准入余量、fixtures、scripts（unchanged 检查）；
  未重钉 T4 源码身份；未跑 release/Linux。
- 不宣称 G03、精度契约或 NP01–NP04 验收通过。
- 离线分析沿用既有 `native-mathdx7-link` 先例的 pinned `warp.dll` 作为 LTO 链接提供者，
  未启动 Python；该 dll 只用于离线证据，不进入产品构建或产品运行链。若要求完全不触 venv，
  需外部 nvJitLink 组件（本轮未授权）。
