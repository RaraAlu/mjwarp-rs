# G01 重力位证书 2 槽差异审计

任务：`2026-10-22-g01-numerics-audit`。
日期：2026-10-22。
设备：NVIDIA GeForce RTX 4070 Ti SUPER，驱动 596.36，Windows x64 MSVC。
只诊断：不改 src/tests/fixtures，不改断言与容差。
结论不自动批准修复。

## 结论

2 个槽各差 4 ulp，来源是 **f32 输入与中间值的舍入**，不是归约顺序、不是 FMA/编译选项、不是设备特性。
精确解析证书用有理算术算得精确值；内核按 f32 语义计算，结果必然不同。
差异与拆分无关：基线 `360718f` 同样失败。
数值容差通过（1.19e-7 远小于 2e-6）；位证书要求零差异，所以测试失败。

## 复现

固定命令（acceptance 指定）：

```text
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --lib -- --ignored gravity_compensation::tests::gpu_matches --nocapture --test-threads=1
```

运行前设置 `CUDA_PATH`、`PATH`（CUDA 12.8.1 bin）与 `MJWARP_MUJOCO_ROOT`、`MJWARP_MUJOCO_DLL`（`--all-features` 需要 MuJoCo 3.12.0 包）。
设置 `MJWARP_GRAVITY_MULTI_BODY_REPORT` 导出诊断报告。

结果：

| 项 | 值 |
| --- | --- |
| 失败 | `left: 2 right: 0` |
| 差异槽 | world 1 dof 2、world 4 dof 2 |
| actual bits | `0x3efffffc`（0.49999988079071045） |
| expected bits | `0x3f000000`（0.5） |
| 每槽 ulp | 4 |
| max_abs | 1.1920928955078125e-7 |
| 容差 | 2.0e-6（通过） |
| 28 槽中其余 | 逐位一致 |
| 归一 | 7 世界 × 4 dof |

world 1 与 world 4 的 COM/cdof 输入完全相同（质量批 1、gravcomp 批不同但不影响 COM），所以两槽结果一致。

## 定位方法

### 1. 贡献者与归约顺序

源码依据：

- `src/physics/gravity_compensation.rs` 的 `roots`：`root[body]` 取到父体为 0 的祖先（body3 的 root 是 body1）。
- 同文件 `ancestry`：`ancestor[b][d] = 1` 当且仅当 dof d 的驱动体在 b 的祖先链上。
- dof 2 的驱动体是 body3；只有 body3 自身满足，所以该槽只有一个贡献者。
- `src/physics/gravity_compensation/cuda.rs` 的 `gravity_compensation_compute` 用 `atomicAdd` 累加；`clear` 先把输出清零。单贡献者时原子加等于一次写入，求和顺序无关。

PTX 证据：`gravity_compensation_compute` 的 PTX 只有一个 `atom.global.add.f32`。

### 2. FMA 与编译选项

捕获 PTX（`MJWARP_PHYSICS_PTX_DIR`）：

| 指令 | 数量 |
| --- | --- |
| `fma.rn.f32` | 0 |
| `mul.f32` | 15 |
| `add.f32` | 3 |
| `add.rn.f32` | 2 |
| `sub.f32` | 8 |
| `atom.global.add.f32` | 1 |

该内核编译产物没有 FMA 融合；乘法与加减都是独立 f32 指令。
主机模拟在「无 FMA」「FMA」「精确后单次舍入」三种写法下得到同一 bits，进一步排除 FMA 影响。

### 3. 设备特性

用主机 f32 逐步模拟同一表达式，得到与 GPU 完全相同的 bits（`0x3efffffc`）。
即差异可由 f32 语义完整解释，不需要任何设备相关行为。
PTX 指令语义固定，`mul.f32`/`add.f32` 默认 round-to-nearest，结果确定。

## 根因推导

该槽的精确值与 f32 路径：

| 量 | 精确有理值 | f32 表示 |
| --- | --- | --- |
| body1 子树 COM | 17/7 = 2.428571428571… | 2.4285714626312256 |
| dof2 的 cdof 线 y | −25/7 = −3.571428571428… | −3.5714285373687744 |
| 力臂 ox = xipos(body3) − subtree_com(root) | 32/7 = 4.571428571428… | 4.5714282990 |
| jy = cdof4 + cdof2·ox | 1 | 0.99999976158 |
| 结果 = jy·fy，fy = 0.5 | 0.5 | 0.49999988079（4 ulp 低） |

f32 输入与精确值的差：

- COM：+3.406e-8（f32 舍入）。
- cdof4：+3.406e-8。
- ox：−2.725e-7（两个 f32 值相减后放大误差，超过单个输入误差）。

因此内核结果与证书差 4 ulp 是 f32 表示的必然结果。
证书 oracle 用的是精确解析力臂（17/7、25/7、32/7），不是 f32 逐步模拟。

## 影响评估

| 项 | 判断 |
| --- | --- |
| 数值容差 | 通过：1.19e-7 ≤ 2e-6 |
| 位证书 | 失败：要求零差异，实际 2 槽各 4 ulp |
| 拆分回归 | 否：基线 `360718f` 同样失败，日志保留在 `target/baseline-gravity.log` |
| 设备缺陷 | 否：主机 f32 模拟可复现，与硬件无关 |
| 产品精度 | 不构成产品路径偏差判定；本测试是辅助诊断证书，不是冻结上游产物对照 |
| NP01 至 NP04 | 不改变验收状态；NP 验收仍未完成，本任务不作精度达标结论 |
| 精度声明 | 不得据此宣称精度达标或失败；差异属证书口径与 f32 实现的固有差距 |

## 后续建议

登记后继任务处理证书口径，本任务不实现：

1. 方案 A：位证书改为 ULP 容差（例如 ≤8 ulp），保留零差异之外的其余断言。
2. 方案 B：证书改为「按内核 f32 表达式逐步模拟」生成，而非精确有理值。
3. 方案 C：保留位证书，但把该 2 槽记录为已知设备无关边界。

三个方案都涉及改测试期望值或断言，须另立任务并单独评审；本任务不改。

## 本轮验证

| 命令 | 结果 |
| --- | --- |
| `cargo fmt --check` | 通过 |
| `cargo test --locked` | 通过，退出码 0 |
| `unchanged` 检查 | `src`、`tests`、`fixtures`、`native`、`include`、`Cargo.toml`、`Cargo.lock`、`rust-toolchain.toml` 未改动 |
| 复现 | `left: 2 right: 0`，失败稳定 |
| PTX 捕获 | `gravity_compensation_compute` 无 fma，单次原子加 |
| 主机模拟 | 与 GPU bits 完全一致 |

## 证据

- `runs/run-01-repro-head.log`：复现日志。
- `runs/run-01-report-head.json`：28 槽诊断报告（actual/expected bits、ulp、COM/cdof）。
- `runs/run-01-ptx-summary.txt`：PTX 指令统计与 compute 主体片段。
- `runs/run-01-sim.log`：精确值、f32 模拟与误差。
- `sim.rs`：主机 f32 模拟脚本（不参与构建）。
- 交付回执：`docs/development-evidence/changes/2026-10-22-g01-numerics-audit.json`。
