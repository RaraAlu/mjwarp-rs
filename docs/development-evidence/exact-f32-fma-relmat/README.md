# 目标无关精确 f32 FMA helper + relmat 契约修复

- 任务：`2026-10-25-exact-f32-fma-relmat`（Backend）
- HEAD：`f5abcd7cc4fe77cede8d7d6f9d3327d2a94909e1`（实现/验证时；若之后 HEAD 前进仅影响行号引用）
- 性质：新增 `src/math/fma.rs` 共享 helper；把 `relmat.rs` 的私有 f64 fma 换成该 helper；
  `render/bvh/aabb.rs` **仅加边界注释**（实现不动）；`gjk/polytope.rs`、`render/kernel/megakernel_arrays.rs`
  **未碰**（另立 fix 片）。不改判据/容差/排除清单/`gates.toml`/G01 脚本/已 verified 证据。
- 证据文件：本 README、`receipt.json`、`probe-4096-1m-boundary.log`、`three-build.log`、
  `seam-boundary.log`、`asm-vfmadd.log`、`bitwise-regression.log`、`f64-audit.log`，
  原始运行日志 `gnu-default.log` / `msvc-default.log` / `gnu-fma.log` / `gnu-seam.log` /
  `relmat-{gnu,msvc,gnu-fma}.log`。

## 1. helper 设计（`src/math/fma.rs`，449 行；单测 `src/math/fma/tests.rs`，377 行）

- 唯一入口：`pub(crate) fn fma_f32(a,b,c) -> f32`，语义 = **IEEE-754 单次正确舍入的 `a*b+c`**
  （RNE），与 GPU `fma.rn.f32` 同语义；**全 f32、无 f64**（契约 L313/L344/L454）。
- 路径选择（每进程一次）：
  1. 主路：`is_x86_feature_detected!("fma")` 为真 → 内联汇编 `vfmadd231ss`；CPUID 结果由
     `OnceLock<bool>` 缓存（每进程只检测一次，缓存后不读 CPUID）。
  2. 兜底：无 FMA 机器自动退纯软件正确舍入（f32 位型分解 + 576 位整数精确乘积/求和 + RNE 舍入；
     无 f64、无平台假设）。
- unsafe 收薄：仅 `#[target_feature(enable="fma")]` 的 `hw_fma` 一个函数，含内联汇编；
  文档写明安全前提（CPUID 已确认 FMA、仅 xmm 标量寄存器、不访存/不改标志/不弹栈）。
- 模块文档写死三件事：① 全 f32/正确舍入/无 f64；② 主路/兜底切换条件与 `OnceLock` 语义；
  ③ 兜底 tie/次正规/溢出/特殊值的等价性依据（整数精确 + guard/sticky/RNE 说明）。
- **测试 seam 仅 test-only**：`MJWARP_FORCE_SOFT_FMA` 只在 `#[cfg(test)] mod test_seam` 内读取
  （`src/math/fma.rs` L104–121）与测试模块（`tests.rs` L310/L315）使用；默认/Release 产品构建
  **不编译、不读取**任何 env seam 分支。证据见 `seam-boundary.log`：
  - 源码层：env 读取位于 `#[cfg(test)]` 块内；
  - 构建层：`target/debug/mjwarp-rs.exe`、`target/release/mjwarp-rs.exe` 的 ASCII 扫描均
    `contains_env=false`，debug 测试二进制 `contains_env=true`；
  - 测试层：注入 seam 后强制走兜底（日志 `SEAM path=software(seam)`），与主路同批样本逐位一致。
- 模块归属建议：`mjwarp.math`（`src/math/fma.rs`）；`.normify/modules` 登记由 Arch 执行（本片未动）。

## 2. 「实际走了哪条路径」的证据

三个构建均实测 `cpuid_fma=true`、`active_path=hardware`（日志 `PATH` 行）：

| 构建 | CPUID fma | active_path | 4096 digest | 1e6 digest | 边界集 |
| --- | --- | --- | --- | --- | --- |
| GNU 默认（1.99.0-...-gnu） | true | hardware | `a14ba3531fc6a6c6` | `54ec8d0836fa2a10` | 0 mismatches |
| MSVC（1.99.0-...-msvc） | true | hardware | `a14ba3531fc6a6c6` | `54ec8d0836fa2a10` | 0 mismatches |
| GNU + `-C target-feature=+fma` | true | hardware | `a14ba3531fc6a6c6` | `54ec8d0836fa2a10` | 0 mismatches |
| GNU 默认 + `MJWARP_FORCE_SOFT_FMA=1`（seam） | true | **software(seam)** | —（同批 4154 样本 digest `c0cf1bfdd35c42f9` = 硬件 digest） | — | 0 mismatches |

- 主路确实发出 `vfmadd` 的独立证明：`cargo rustc --lib -- --emit=asm` 的生成汇编中命中
  `vfmadd231ss %xmm2, %xmm1, %xmm0`（`asm-vfmadd.log`，含上下文行）；同一 helper 输出与内联汇编
  参考 `vfmadd231ss` 在 4096/1e6/边界集上 0 差异、digest 完全相同（§3）。
- 注：若某构建静默走了兜底，则「位级一致」只证明兜底；本片用上面的 `PATH` 行 + 汇编命中 +
  硬件参考 digest 三件事排除该情况。

## 3. 探针（QA 模板：生成器参数 / 规模 / 差异样本 / 残余率范围）

- 生成器（QA 复核口径）：LCG `mult=6364136223846793005, inc=1442695040888963407,
  seed=0x2026102500000001`；映射 `f32(top24)/2^24*3-1 ∈ [-1,2)`（top24=(state>>40)&0xffffff）。
- 规模：**4096 组 + 1e6 组** + 边界集（58 例：次正规、近半 ULP tie、指数差极大、抵消到
  次正规/零、±0、溢出、NaN/Inf）。
- 参考：内联汇编 `vfmadd231ss`（硬件 FMA；不使用 `#[target_feature]` 包装的 `mul_add`，
  避免 opt-level 0 下退化成库回退而同源）。
- 结果（三构建一致，详见 `probe-4096-1m-boundary.log`）：
  - `soft_fma` vs 硬件 FMA：**4096 = 0 差异、1e6 = 0 差异、边界集 = 0 差异**；
  - `fma_f32`（按 CPUID 派发）vs 硬件 FMA：同上 0 差异，同批 digest 与兜底一致；
  - 1e6 digest：soft/hw 均 `54ec8d0836fa2a10`；4096 digest：`a14ba3531fc6a6c6`。
- 差异样本（≥5 条，用于证明探针有判别力；来源 GNU 默认的 `f32::mul_add` 库回退 vs 硬件 FMA）：
  ```
  a=3d832940 b=3fe7d4e4 c=3e428720 | mul_add=3e9ca735 want=3e9ca736
  a=3f3de1da b=bec38e5e c=3fac3010 | mul_add=3f87ece8 want=3f87ece7
  a=3f43b054 b=bf7ec3a1 c=3e6a95d8 | mul_add=bf081907 want=bf081908
  a=bed00a12 b=3f3df5bc c=3f7ff0dc | mul_add=3f32c14c want=3f32c14b
  a=bf61bed2 b=3f697998 c=beb166ce | mul_add=bf934aa8 want=bf934aa7
  ```
  GNU 默认 4096 组中 1025 条差异；MSVC 默认 0 条；GNU+`+fma` 0 条。
- **残余率声明**：0 残余率仅对上述生成器/规模/边界集有效，不外推到其它分布/规模。
  已知 QA 在 1e6 中观测到 GNU `mul_add` vs 硬件 FMA 有 1 个 **2 ULP** 个例；本 helper 兜底
  对硬件 FMA 差异 = 0，未出现 >1 ULP 个例。

## 4. 三构建位级一致（目标无关证明）

- helper 输出 digest（4096/1e6）与边界集结果三构建相同（§2/§3）。
- `relmat` 相关测试输出三构建一致：
  - `cargo test --locked --lib physics::collision::relmat` → 三构建均 **4 passed / 0 failed**；
  - `cargo test --locked --test g10_p5_sdf_relmat` → 三构建均 **4 passed / 0 failed**
    （4 场景 40/40 位级 + sdf-box 80/80 + rel_mat 编码约定）。
- 日志：`three-build.log`、`relmat-{gnu,msvc,gnu-fma}.log`。

## 5. G5 位级回归（换实现前 vs 换实现后，纯 CPU，未开 GPU 窗口）

| 测试 | 换实现前基线 | 换实现后 | 位型 |
| --- | --- | --- | --- |
| `--lib physics::collision::relmat` | 4 passed | 4 passed | 逐行 diff = 0 |
| `--test g10_p5_sdf_relmat` | 4 passed | 4 passed | 逐行 diff = 0（40/40 + 80/80 位级断言通过） |
| `--test g10_p5_sdf_host` | 1 passed | 1 passed | 逐行 diff = 0 |
| `--test gjk_epa` | 3 passed | 3 passed | 并行 stdout 行序交错；排序/多重集内容一致 |

→ **rel_mat 10/10 world 与 SDF 40/40 位级不变**（测试自身逐位断言通过），无位型变化，未触发红线。
本片不需要 GPU 窗口（上述均为 CPU host 测试；未开窗、未跑 GPU）。

## 6. 产品数值链内部 f64 审计（只列不改；清单 `f64-audit.log`，共 227 处）

| 分类 | 数量 | 说明/代表位置 | 是否物理链 |
| --- | --- | --- | --- |
| 允许-测试/诊断 | 139 | `src/physics/**/tests*`、`collision/tests.rs`、`case_evidence.rs`（`#[cfg(test)]`）、`constants.rs`（仅 `#[cfg(test)] mod tests` L157+）、`samples/perf/tooling` 诊断 | 否 |
| 允许-诊断/校验 | 42 | `device_checks.rs`（`cfg(any(cuda-probe,test))` 输出校验）、`kinematics.rs::norm_squared`（仅 `check_kinematics` 校验）、`history.rs::validate_times` 输入校验、`perf_report.rs`、`case_compare` 注释 | 否 |
| 允许-ABI/类型支持 | 13 | `runtime/transfer.rs` f64 TransferElement（公开传输类型）、`launcher/abi.rs`/`warp.rs` F64 WarpArray 变体（上游 ABI 支持） | 否（ABI 类型，不参与物理数值计算） |
| 允许-PTX 精度门禁（诊断） | 7 | `runtime/transfer/kernel/ptx.rs` 检测/拒绝 PTX 中 f64 | 否（门禁检查器） |
| 允许-错误诊断负载 | 7 | `mass_solve/sparse/*` 的 `TransferError::InvalidPivot { value: f64 }` | 否（错误信息字段） |
| 允许-ABI 转换口径 | 4 | `collision_upload.rs` 注释（f64→f32 仅走 G1 唯一转换入口） | 否 |
| 允许-回读校验（cuda-probe） | 1 | `resident/output.rs::read_output`（`cfg(cuda-probe)` 回读校验） | 否 |
| 允许-输入校验 | 2 | `kinematics.rs`（仅 `check_kinematics`） | 否 |
| 允许-夹具 provenance 注释 | 4 | `case_data.rs`/`gjk/mod.rs`/`octree.rs` 注释（authoring f64→f32 说明） | 否 |
| **已修复（本片）** | 2 | `relmat.rs` 的 f64 fma 实现删除，改为 `crate::math::fma::fma_f32`；剩余 2 处仅为文档说明 | **原违规，已删除** |
| 待人工复核 | 0 | — | — |

结论：除已修复的 `relmat.rs` 外，**未发现新的「产品物理数值链私有 f64 工作区」**；其余 f64 均属测试、
诊断/校验、错误信息、ABI 类型支持或 PTX 门禁。`render/bvh/aabb.rs` 的 f64 在加速结构（不属物理
数值链），本片只加边界注释、实现不动（归 render 侧口径另评估）。

## 7. 范围与行数

- 修改：`src/math/mod.rs`（+`pub(crate) mod fma;`）、`src/physics/collision/relmat.rs`（删 f64 helper，
  改调共享 helper，文档更新）、`src/render/bvh/aabb.rs`（**仅注释** +6 行边界说明）。
- 新增：`src/math/fma.rs` 449 行、`src/math/fma/tests.rs` 377 行（均 <500 行棘轮）。
- **未改**：`gjk/polytope.rs`、`render/kernel/megakernel_arrays.rs`、任何判据/容差/排除清单、
  `gates.toml`、G01 脚本、Cargo 清单、已 verified 证据。
- `git diff --stat`（实现文件）：5 files changed, 58 insertions(+), 26 deletions(-)（含两个任务 JSON；
  `polytope`/`megakernel_arrays` 不在 changed 列表）。

## 8. 边界与未覆盖

- **无 FMA 实机未验证**：兜底路径通过 seam 在 FMA 机器上强制运行并与硬件 FMA 0 差异；未在真实
  无 FMA 机器上跑（如老 CPU/非 x86），如实登记未覆盖。
- NaN payload 不作为位级判据：测试按「两边均为 NaN」比较；解析/传播 payload 未承诺。
- 性能未做正式基准：兜底路径仅在无 FMA 机器启用（本机不启用）；helper 主路为单条指令。
- 未跑 GPU/Linux；未宣称精度达标/跨环境全绿；G5 位级回归以仓库内既有 CPU 测试为准。
- 模块源码登记（`normify module source mjwarp.math --add src/math/fma.rs` + `.normify/modules` 提交）
  由 Arch 执行，本片未动 `.normify`。

## 9. 复现

```powershell
# GNU 默认（仓库默认工具链）
cargo test --locked --lib math::fma -- --nocapture --test-threads=1
# MSVC
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --lib math::fma -- --nocapture --test-threads=1
# GNU + 硬件 FMA 强制
$env:RUSTFLAGS='-C target-feature=+fma'; cargo test --locked --lib math::fma -- --nocapture --test-threads=1
Remove-Item Env:RUSTFLAGS
# seam 强制兜底（仅测试）
$env:MJWARP_FORCE_SOFT_FMA='1'
cargo test --locked --lib math::fma::tests::forced_soft_seam_matches_hardware -- --nocapture
Remove-Item Env:MJWARP_FORCE_SOFT_FMA

# G5 位级回归（CPU，无窗口）
cargo test --locked --lib physics::collision::relmat -- --nocapture
cargo test --locked --test g10_p5_sdf_relmat -- --nocapture
cargo test --locked --test g10_p5_sdf_host -- --nocapture
cargo test --locked --test gjk_epa -- --nocapture

# seam 边界 / 反汇编
cargo build --locked; cargo build --locked --release
cargo rustc --lib -- --emit=asm   # 然后在 target/debug/deps/mjwarp_rs-*.s 中查 vfmadd
```

## 10. 合规论证（NP01 类型映射：整数中间量不构成类型偏离，2026-10-25 追加）

- 兜底路径的整数**只用于精确模拟硬件 FMA 的单次舍入**：整数承载 f32 位型分解后的有效位乘积/和
  （576 位精确整数），不引入新的浮点精度层级、不产生第二个浮点结果，`fma_f32` 输入/输出始终 `f32`。
- 契约禁止的是「内部 f64 工作区/升精度规避原生误差」（L313/L344/L454）；整数精确运算不是浮点升精度，
  是对 `f32` 单次正确舍入的实现，不进入 NP01「实际精度偏离」计数。
- 落点：`src/math/fma.rs` 模块文档新增「合规论证」段；`docs/numerical-precision-contract.md` NP01 段
  增加同义注记；审计清单见 `f64-audit.log`（227 处，无新违规）。
- 行数棘轮提示：`src/math/fma.rs` 追加后 **459 行**（原 449），后续任何新增按棘轮**先拆文件**再加，
  不得逼近 500。
