# 精确 f32 FMA helper + relmat 契约修复 · QA 独立复核（F1 已修复，复验 pass）

- 复核任务：`2026-10-25-exact-f32-fma-relmat-independent-check`（QA）
- 被复核任务：`2026-10-25-exact-f32-fma-relmat`（原交付 commit `f0dceb5b`）
- **F1 修复 commit：`3b7db351`**；本轮复验 HEAD：`3b7db351`（`src/math/fma.rs` sha256 `3c865f4b…`）
- 性质：CPU-only、无 GPU 窗口；自写探针（内联汇编 `vfmadd231ss` 参考）、自建 clean target；
  不改产品树/判据/容差/门禁
- **结论：pass（F1 已修复并独立复验）** —— 阻断 0 / 发现 1（原阻断 F1，已修、已复验）/ 观察 7。
  - 原 F1：兜底软件路径两处 `return c.bits as f32;`（u32→f32 **数值转换**）应为 `f32::from_bits(c.bits)`；
    我的 46 条独立边界集软件路径 **8/46 不符**（三构建一致），relmat 3-FMA 链样板被污染。
    `3b7db351` 将两处改为 `f32::from_bits`、边界集 58→77、新增默认 gate 逐类位型测试；
    **我的修后独立复验：boundary_seam 8/46 → 0/46，relmat 链 2/2/2/3 → 0/0/0/0，主路与随机 1e6 仍 0。**
  - 原交付「兜底（软件）路径 4096/1e6/边界集 0 差异」的声明**在两类边界输入上不成立**——这一条作为
    **历史事实**保留（修前实测），修后已由 77 例边界集、默认 gate 逐类测试与我的 46 例复验覆盖；
    本机（FMA3）产品始终走硬件主路，已交付 G5 数字不受 F1 影响。

## 1. F1（原阻断，已修）：兜底路径类型混淆，违反单次舍入契约

- 位置（修前）：`src/math/fma.rs:158`（`c == ±Inf` 分支）与 `:169`（乘积 ±0 分支），
  `return c.bits as f32;` 把位型当数值转换。
- 最小复现（修前，seam 强制软件路径）：`fma_f32(0.0, 1.0, -1.0)` 期望 `0xbf800000`（-1.0），
  实际 **`0x4f3f8000`（≈3.2e9）**；修后同调用返回 `0xbf800000`，三种构建一致。
- 修前 46 条独立边界集 **8 条不符**（三构建一致；硬件主路 0 条）：

| case | a | b | c | 修前 got | want |
| --- | --- | --- | --- | --- | --- |
| zero_product_neg_c | 00000000 | 3f800000 | bf800000 | **4f3f8000** | bf800000 |
| negzero_times_one_plus_two | 80000000 | 3f800000 | 40000000 | **4e800000** | 40000000 |
| one_times_zero_plus_five | 3f800000 | 00000000 | 40a00000 | **4e814000** | 40a00000 |
| zero_times_negone_plus_half | 00000000 | bf800000 | 3f000000 | **4e7c0000** | 3f000000 |
| zero_b_zero_c_nonzero | 00000000 | 80000000 | 3f800000 | **4e7e0000** | 3f800000 |
| c_inf_finite_ab | 3f800000 | 40000000 | 7f800000 | **4eff0000** | 7f800000 |
| c_neg_inf_finite_ab | 3f800000 | 40000000 | ff800000 | **4f7f8000** | ff800000 |
| zero_a_c_inf | 00000000 | 3f800000 | 7f800000 | **4eff0000** | 7f800000 |

- 修前影响面：本机 FMA3 → 产品走硬件主路，已交付 G5 数字不变；**无 FMA 机器**会全量走兜底，
  且 relmat 旋转矩阵含大量精确 0，3-FMA 链首项后 `c` 非零即触发 → identity×identity 2/9、
  rz90×rz90 2/9、ry45×rz90 3/9 单元错误。
- 修前漏检原因：交付 58 条边界集不含「零因子 + 非零有限 c」「有限 a,b + c=±Inf」两类；
  其 seam 测试 4154 样本 = 随机 4096 + 该 58 条，故全绿；门禁默认特性不启用 seam。
- 修复即我方建议：两处 `c.bits as f32` → `f32::from_bits(c.bits)`（`grep 'bits as f32'` = 0、
  `f32::from_bits` = 7 处），并把 8 类输入补进 `boundary_cases()`（58→77）与新增默认 gate 测试。
  我在修前用**产品快照副本**验证过该假设（`probe-fixed/`、`qa-fix-hypothesis.json`），**当时未改产品树**。

## 2. 修后独立复验（本轮核心；自写探针 + 三构建 + 两路径）

- 探针：`probe-postfix/qa_fma_probe.rs`（与修前同一份自写探针，**不读** `src/math/fma/tests.rs`、
  不复用交付脚本）；对**修后** `src/math/fma.rs` 做纯机械快照 `probe-postfix/fma_snapshot.rs`
  （仅行首 `//!`→`//`，459 行、改动 39 行、**unexpected diff 0**，见 `qa-static-checks-postfix.json`）。
- 参考：内联汇编 `vfmadd231ss`；生成器 LCG mult=6364136223846793005 / inc=1442695040888963407 /
  seed=0x2026102500000001，映射 `f32(top24)/2^24*3-1 ∈[-1,2)`；规模 4096 + 1e6 + 46 边界。
- 修后结果（GNU 默认 / GNU `+fma` / MSVC 默认三构建全部一致，均 `cpuid_fma=true active_path=hardware`）：

| 路径 / 规模 | 4096 | 1e6 | 46 边界 | 修前对照 |
| --- | --- | --- | --- | --- |
| dispatch（真实主路，硬件） | 0 差异 | 0 差异 | **0/46** | 0/46（不变） |
| seam 强制（软件兜底） | 0 差异 | 0 差异 | **0/46** | **8/46 → 0/46** |
| 直接 `soft_fma`（同一兜底函数） | 0 | 0 | 0/46 | 8/46 → 0/46 |
| dispatch vs seam 互比（1e6） | — | 0 差异 | — | 0（随机样本无触发类） |
| relmat 3-FMA 链样板（4 组矩阵） | — | — | **0/9 全部** | 2/9、2/9、2/9、3/9 → 0 |

- 修后最小复现：`fma_f32(0.0, 1.0, -1.0)` = `0xbf800000`（`SEAM_ENV_CHECK … equal=true`，seam 开/关皆然）。
- 随机路径 digest 与修前一致（4096：`94042172a0100abf`；1e6：`cd439a5ae1784d4f`）→ 硬件主路未被改动。
- 判别力自检（探针确实能区分非合规 FMA）：GNU 默认 `f32::mul_add` vs 硬件 FMA **1025/4096**；
  GNU `+fma` / MSVC 均 0 —— 与修前一致。
- 汇编复核（修后快照、非 test 构建 `--emit=asm`）：GNU 与 MSVC 均命中 `vfmadd231ss` **各 1 处**
  （`asm-vfmadd-gnu-postfix.txt` / `asm-vfmadd-msvc-postfix.txt`）。
- 逐项日志：`probe-logs-postfix/`（6 份探针日志）；汇总 `qa-probe-summary-postfix.json`
  （含 before/after 对照矩阵）。

## 3. 「默认 gate 常态覆盖兜底」独立核（Arch 复跑要求第 2 项）

- 新增测试 `soft_fma_exact_bits_by_edge_class`（`src/math/fma/tests.rs:322`）：
  **直接调用 `pub(crate) soft_fma`**，不读 env、不用 seam、不依赖硬件 FMA；默认 `cargo test --locked --lib` 即执行。
- 我的独立证据（自建 clean target，`MJWARP_FORCE_SOFT_FMA` **未设置**）：
  `cargo test --locked --lib math::fma::tests::soft_fma_exact_bits_by_edge_class -- --exact --nocapture`
  → **1 passed / 0 failed**；日志 16 条 `F1 … OK`、0 条 FAIL（`logs-postfix/g5-lib-soft-fma-edge-class.log`；
  环境快照 `logs-postfix/env-snapshot-postfix.txt` 记录 `MJWARP_FORCE_SOFT_FMA_set=False`）。
- 静态核：该测试块内无 `env`/`var_os`/`MJWARP` 引用；`env` 读取全仓 `src/math/fma.rs` 只有 1 处，
  位于 `#[cfg(test)] mod test_seam`（L115–135）；`fma_f32` 的 seam 分支由 `#[cfg(test)]`（L76）门控。
- 因此「同类缺陷以后默认 gate 直接抓到」这一机制**成立**（我的 46 例边界集复核为独立第二判据）。

## 4. `fma.rs` early-return 分支独立审计（Arch 复跑要求第 3 项）

| # | 分支 | 位置（修后） | 我的结论 |
| --- | --- | --- | --- |
| 1 | 任一输入 NaN → `f32::NAN` | L142–144 | 规范 quiet NaN；IEEE 允许非规范 payload → 不作位级判据（见 §8-7） |
| 2 | `a`/`b` 含 Inf（0×Inf、异号 Inf+Inf → NaN） | L147–156 | 符号/NaN 规则正确；`f32::from_bits`/`inf()` |
| 3 | `c`=±Inf 且 a,b 有限 | L157–159 | **F1 修复点**：`f32::from_bits(c.bits)` ✓ |
| 4 | `a`/`b`=±0（c=0 走 IEEE 符号规则；c 非零→c） | L160–170 | **F1 修复点**：`f32::from_bits` ✓；±0 相加符号规则 ✓ |
| 5 | `c`=0 → 精确积 RNE | L172–176 | 整数精确积 + `round_to_f32` ✓ |
| 6 | 同号 → 精确和 RNE | L183–187 | 576 位整数精确加 ✓ |
| 7 | 异号 → 相等→+0；不等→精确差 + 对应符号 | L188–204 | RNE 下精确零为 +0 ✓ |
| 8 | `round_to_f32`：无有效位→±0；指数>127→±Inf；正规/次正规 tie+sticky；进位归一/升级最小正规；深下溢→±0 | L383–456 | 全部经 `inf()`/`zero()`/`from_bits`；上溢半 ULP tie 与次正规 tie 由我的 46 例覆盖（0 差异） |
| 9 | 全文件 `as` 用法 | — | 位型→浮点仅原 2 处（已修）；其余 `as` 均为整数移位/索引/位段（L181–182/L236–237/L347/L387/L425/L436/L439），取值范围核对无越界 |
| 10 | 测试侧生成器 `top24 as f32` | tests.rs | 真实数值转换（映射用），非位型还原 ✓ |

## 5. G5 位级独立复跑（修后；自建 clean target `target/qa-fma-check-postfix`）

`CARGO_TARGET_DIR=target/qa-fma-check-postfix/tests`（全新；CARGO/RUST env 清空前计数 0）：

| 测试 | 结果 |
| --- | --- |
| `--lib math::fma` | **7 passed / 0 failed** |
| `--lib physics::collision::relmat` | **4 passed / 0 failed** |
| `--test g10_p5_sdf_relmat --nocapture` | **4 passed**；`rel_mat_warp aᵀb 位级 10/10 world`（8 world 与 bᵀa 可区分）；**6 条 80/80 位等行**（sdf-box G3 + 5 个 sdf-rot 场景），dist/pos/frame `max_abs` 均 **0e0** |
| `--test g10_p5_sdf_host` | **1 passed** |
| `--test gjk_epa --nocapture` | **3 passed** |
| `--lib math::fma` + `MJWARP_FORCE_SOFT_FMA=1` | **7 passed / 0 failed**（seam 测试 4173 样本 0 mismatches） |

→ 「rel_mat 10/10 与 SDF 40/40 位级不变」在修后仍成立；F1 修复未改变本机硬件主路数值。

## 6. seam 边界修后复查

- 自建 clean target 的产品二进制字节扫描（`logs-postfix/seam-binary-scan-postfix.txt`）：

| 对象 | 大小 | contains `MJWARP_FORCE_SOFT_FMA` |
| --- | --- | --- |
| 产品 `debug/mjwarp-rs.exe` | 20,656,234 B | **false** |
| 产品 `release/mjwarp-rs.exe` | 2,590,274 B | **false** |
| lib 测试 harness（`mjwarp_rs-bbf617de7cd0b294.exe`） | 54,819,147 B | **true** |

- 源码核：env 读取只存在于 `#[cfg(test)] mod test_seam`；产品 `fma_f32` 的 seam 分支被 `#[cfg(test)]` 包裹；
  无「运行时读 env 再分支」的宽门控。修后 release 体积 +17 B（2,590,257→2,590,274），debug 不变。

## 7. 门禁与 `normify check`（主树 HEAD `3b7db351`，产品树干净）

- `normify gate 2026-10-25-exact-f32-fma-relmat-independent-check --repo D:\Rust\mjwarp-rs`
  → fmt / clippy / clippy-cuda 通过、**unit 604 passed / 0 failed / 7 ignored**、**EXIT 0**；
  runs 记录在 `.normify/runs/2026-10-25-exact-f32-fma-relmat-independent-check/`。
- `normify check …` → **全部检查通过、EXIT 0**。
- 门禁口径说明：本轮 gate/check 在**全部证据冻结后**、主树执行；gate 的 scope 就是本证据目录，
  **gate 运行期间不能往本目录写日志**（否则 check 会判「范围或定义已变，需重跑」——本轮踩过一次并已定位）。
  故 gate/check 原始 stdout 写入 `target/qa-fma-check/gate-check/`（不入库），提交证据以 runs JSON 与
  本节结果为准；`run_gate_check.ps1` 已按此修正。
- 修前门禁（603/0）与修后（604/0）差异来自新增默认 gate 测试，非缓存复用。

## 8. 观察（非阻断）

1. （历史，已修）交付原 58 条边界集缺 F1 两类触发输入；`3b7db351` 已补到 77 例。
2. （历史，仍未补）交付 `seam-boundary.log` 仍只有 3 个 `contains_env=False`，缺 harness 的 `contains_env=true`
   行；本轮我的独立扫描已补该证据（§6）。属交付证据完整性小缺口，不影响 F1 结论。
3. （历史，已更新）交付 README §1 原写 `#[target_feature(enable="fma")]` 的 `hw_fma`；`3b7db351` 已改为
   「纯内联汇编、**未用** `#[target_feature]` 包装」——观察已闭环。
4. （历史，已缓解）原 seam 模式产品测试全绿（4154 样本）抓不到 F1；现由新增默认 gate 逐类位型测试（16 例）
   在默认特性 + 无 env 下常态覆盖，我独立复跑确认。
5. （历史）`c.bits as f32` 这类位型/数值类型混淆 clippy 不拦；现由默认 gate 测试兜底，评审要点
   「位型变量必须 `from_bits`」建议保留。
6. （新）交付 README §11 表 1/2 与 commit message 写「17 例」，实际测试 `cases` = **16**（README §11 末段又写
   16 条）；计数不一致属文档小瑕疵，不影响行为与门禁（我按源码与日志独立计数）。
7. （保留）NaN payload 不作位级判据：`soft_fma` 统一返回规范 `f32::NAN`，硬件可能返回带符号/载荷的 quiet NaN；
   按 IEEE 语义等价，本轮边界集 NaN 类按「均为 NaN」比较。

## 9. 口径与边界

- **真实无 FMA 机器仍未实跑**；兜底语义由 test seam + 新增默认 gate 直调测试覆盖。不宣称跨机器/跨平台全绿。
- 本机 FMA3 → 产品路径始终硬件主路；F1 修前修后**已交付 G5 数字不变**（修后复跑已核）。
- 原交付「兜底 0 差异」声明在两类边界输入上不成立 → 作为历史事实保留；修后 0/46 是**修后**结论，二者不混写。
- 未改产品代码/判据/容差/排除清单/`gates.toml`/G01；复验只读产品树，证据仅写本目录。
- 不入库：`target/qa-fma-check/**`、`target/qa-fma-check-postfix/**`、gate/check 原始 stdout。

## 10. 复现与显式路径

```powershell
# 修后探针（三构建 × 两路径；含 env seam；约 1 分钟）
& target\toolchains\powershell-7.6.7\bin\pwsh.exe -NoProfile -File docs\development-evidence\exact-f32-fma-relmat-independent-check\run_probe_postfix.ps1
# 修后产品构建 + seam 字节扫描 + G5 复跑 + 默认 gate 逐类测试
& target\toolchains\powershell-7.6.7\bin\pwsh.exe -NoProfile -File docs\development-evidence\exact-f32-fma-relmat-independent-check\run_g5_and_builds_postfix.ps1
& target\toolchains\powershell-7.6.7\bin\pwsh.exe -NoProfile -File docs\development-evidence\exact-f32-fma-relmat-independent-check\qa_scan_seam_binary_postfix.ps1
# 静态核 + 汇总
python docs\development-evidence\exact-f32-fma-relmat-independent-check\qa_static_checks_postfix.py
python docs\development-evidence\exact-f32-fma-relmat-independent-check\qa_summarize_postfix.py
# 门禁/check（证据冻结后；stdout 写 target/）
& target\toolchains\powershell-7.6.7\bin\pwsh.exe -NoProfile -File docs\development-evidence\exact-f32-fma-relmat-independent-check\run_gate_check.ps1
# 清单
python docs\development-evidence\exact-f32-fma-relmat-independent-check\qa_artifacts_sha256.py
```

证据目录 `docs/development-evidence/exact-f32-fma-relmat-independent-check/`：
- 修前（F1 历史）：`probe/`（自写探针 + 修前机械快照）、`probe-fixed/`（修前快照副本上的修复假设）、
  `probe-logs/`、`logs/`、`qa-finding.json`、`qa-probe-summary.json`、`qa-static-checks.json`、
  `qa-fix-hypothesis.json`、`asm-vfmadd-{gnu,msvc}.txt`、`probe-build-info.txt`。
- **修后（本轮复验）**：`probe-postfix/`、`probe-logs-postfix/`、`logs-postfix/`、
  `qa-probe-summary-postfix.json`、`qa-static-checks-postfix.json`、`qa-g5-postfix.json`、
  `asm-vfmadd-{gnu,msvc}-postfix.txt`、`probe-build-info-postfix.txt`。
- 脚本：`run_probe.ps1`、`run_g5_and_builds.ps1`（修前）；`run_probe_postfix.ps1`、
  `run_g5_and_builds_postfix.ps1`、`qa_scan_seam_binary_postfix.ps1`、`run_gate_check.ps1`、
  `qa_static_checks_postfix.py`、`qa_summarize_postfix.py`；`qa_make_snapshot.py`、
  `qa_make_snapshot_postfix.py`、`qa_summarize.py`、`qa_fix_hypothesis.py`、`qa_artifacts_sha256.py`。
- `README.md`、`receipt.json`、`artifacts-sha256.json`。
