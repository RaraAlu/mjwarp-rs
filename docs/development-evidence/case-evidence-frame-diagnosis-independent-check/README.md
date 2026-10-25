# case_evidence frame 环境敏感性诊断 · QA 独立复核

- 复核任务：`2026-10-25-case-evidence-frame-diagnosis-independent-check`（QA）
- 被复核任务/结论：`2026-10-25-case-evidence-frame-diagnosis`（交付 commit `7c401282`）
- 复核基线：HEAD `eac9af71`（任务登记时）；复核期间主干前进到 `00831281`（仅新增 `tests/` 文件，`src/`/`fixtures/` 零改动，已核）
- 性质：**CPU-only、不开 GPU 窗口**；自建探针与 harness、自建 clean target；不碰产品代码/判据/容差/G01 脚本
- 结论：**pass（阻断 0 / 发现 0 / 观察 7）**。交付的核心事实（GNU 默认 `f32::mul_add` 非 IEEE 单次舍入、GNU pass 系偶合、faithful FMA 下 frame `6.351978e-7` 红）**独立复现成立**；探针统计量因交付未记录输入生成器参数而不能逐位复现（并列见 §2）。

## 1. 产品级 frame 矩阵（全部自建 clean target，逐格原始命令/exit/结果行）

每个单元格：**新建并删空专用 `CARGO_TARGET_DIR`**（不复用交付 `diag-*`、Backend `verify-clean-001/002`、Arch `arch-*-clean`、主 `target/`），
清空 `CARGO*`/`RUST*` 并记录（清空前计数 0，见 `qa-exclusions.json.clean_build_record`），用仓库干净 `git worktree`
（`target/qa-frame-diag/tree` @ `eac9af71`；⑤ 用去掉 `.cargo/config.toml` 的另一 worktree）执行。

测试：`cargo <toolchain> test --locked [--release] --lib physics::collision::gjk::case_evidence::upstream_f32_within_4ulp -- --exact`
（断言格，exit/panic 给出 frame）+ `…our_contacts_match_upstream_f32_within_4ulp -- --exact --nocapture`（打印格，取 ellipsoid-box `max abs`）。

| # | 环境（三方并列） | 我的 exit / 结果行 | 我的 frame | 交付 frame | 一致 |
| --- | --- | --- | --- | --- | --- |
| ① | GNU 默认（repo config，`-C link-dead-code=yes`；deps 62） | **0 / `1 passed`** | **3.838112e-7** | 3.838112e-7（pass） | 是 |
| ② | MSVC 默认（deps 65） | **101 / `FAILED. 0 passed; 1 failed`，panic `ellipsoid-box frame 6.351978e-7`** | **6.351978e-7** | 6.351978e-7（fail） | 是 |
| ③ | GNU `RUSTFLAGS=-C target-feature=+fma`（deps 62） | **101 / FAILED** | **6.351978e-7** | 6.351978e-7 | 是 |
| ④ | MSVC `+fma`（deps 65） | **101 / FAILED** | **6.351978e-7** | 6.351978e-7 | 是 |
| ⑤ | GNU release（deps 62） | **0 / `1 passed`** | **3.838112e-7** | 3.838112e-7 | 是 |
| ⑥ | MSVC release（deps 65） | **101 / FAILED** | **6.351978e-7** | 6.351978e-7 | 是 |
| ⑦ | MSVC `RUSTFLAGS=-C link-dead-code=yes`（deps 67） | **101 / FAILED** | **6.351978e-7** | 6.351978e-7 | 是 |
| ⑧ | GNU 去掉 `.cargo/config.toml`（无 link-dead-code；deps 62） | **0 / `1 passed`** | **3.838112e-7** | 3.838112e-7 | 是 |
| ⑨ | ① 的同一二进制 + `G2_DUMP_NATIVE_ERROR=1` | **0 / `1 passed`** | —（同 ①） | — | — |

- 8/8 格的 exit、结果行、frame 值与交付逐项一致；deps 数（62/65/67）也与交付基线一致。
- 每格另有打印格原始行（含 `frame 位等分量 13/18 max abs …`）与完整 cargo 日志：`frame-matrix.json`、`frame-matrix-summary.txt`、`logs/*.log`。
- 关键推论（我独立得到）：**GNU 默认（含 release）与「去掉 link-dead-code」均为 `3.838112e-7`；任何 faithful FMA 构建（MSVC 默认 / GNU+`+fma` / MSVC+`+fma`）均为 `6.351978e-7`**。
  即 pass/fail 翻转只随 FMA 语义变化，link-dead-code、release/opt-level、环境变量、产物目录均不改变结论。

## 2. 独立微探针（自己的源码、自己的输入生成器、1e6 组）

- 源码：`qa_fmaf_probe.rs`（本目录，独立实现，未读交付探针源码），`rustc --edition 2024 -C opt-level=0 -C debug-assertions=on`，
  四种构建：GNU 默认 / MSVC 默认 / GNU `-C target-feature=+fma` / MSVC `-C target-feature=+fma`。
- 输入生成器（**为可复现而记录**）：64 位 LCG，`state ← state*6364136223846793005 + 1442695040888963407`，
  seed `0x2026102500000001`；每个值 = `f32(top24(state))/2^24 * 3 - 1`（值域 `[-1,2)`）；每组 `a,b,c` 取连续三次输出。
- 三个参考：
  1. **硬件 FMA**：内联汇编 `vfmadd231ss`（运行期 `is_x86_feature_detected!("fma")` 检查；**参考与构建 flags 无关**，
     四构建的参考 digest 在每个规模上完全一致 → 参考稳定）；
  2. 非融合 `a*b + c`（`black_box` 强制中间舍入）；
  3. f64 双舍入 `(f64 a * f64 b + f64 c) as f32`。
- 结果（不同项 / 4096，及 1e6 残余率）：

| 对比 | GNU 默认 | MSVC 默认 | GNU `+fma` | MSVC `+fma` |
| --- | --- | --- | --- | --- |
| `mul_add` vs 硬件 FMA（4096） | **1025（25.02%，全 1 ULP）** | **0** | **0** | **0** |
| `mul_add` vs 非融合（4096） | 1447 | 877 | 877 | 877 |
| `mul_add` vs f64 双舍入（4096） | **1025** | **0** | **0** | **0** |
| 硬件 FMA vs f64 双舍入（4096） | 0 | 0 | 0 | 0 |

| 1e6 组（残余率） | GNU 默认 | MSVC 默认 | GNU `+fma` | MSVC `+fma` |
| --- | --- | --- | --- | --- |
| `mul_add` vs 硬件 FMA | **246671（24.6671%）**：246670 个 1 ULP + **1 个 2 ULP** | **0 / 1e6** | **0 / 1e6** | **0 / 1e6** |
| 硬件 FMA vs f64 双舍入（团队先例 helper 的方法） | **0 / 1e6** | 0 / 1e6 | 0 / 1e6 | 0 / 1e6 |

- 结论（限定口径）：**在本目录记录的输入分布与 1e6 规模下**，GNU 默认 `f32::mul_add` 与硬件 FMA 的残余率约 **24.67%**，
  绝大多数 1 ULP、但**存在 2 ULP 的个例**（见 §2.2 样本）；MSVC 默认与两个 `+fma` 构建观测 **0 差异**。
  「0/1e6」只是该分布/该规模下的观测，**不能外推为跨分布保证**；2 ULP 个例说明差异并非严格限于 1 ULP。
- 四构建互比：`+fma` 两构建的 `mul_add` digest 与硬件 FMA digest 完全相同；MSVC 默认亦相同；**GNU 默认不同**。
- 已知向量（`fma(1+2^-23, 1-2^-23, -1)`、`fma(3,7,-21)`、`fma(0.1,0.2,0.3)` 等）四构建结果一致且与硬件一致 ——
  说明 GNU 默认不是「完全不融合」，而是特定输入下的低 ULP 偏差；与交付描述一致。

### 2.1 与交付探针统计的并列（不单方面采信）

交付 `probe-fmaf.log` 给出的 4096 组统计 vs 本复核独立结果：

| 对比（4096） | 交付 GNU | 我的 GNU | 差异 | 交付 MSVC | 我的 MSVC | 差异 |
| --- | --- | --- | --- | --- | --- | --- |
| vs 硬件 FMA | 991 | 1025 | +34 | 0 | 0 | 0 |
| vs 非融合 `a*b+c` | 986 | 1447 | +461 | 55 | 877 | +822 |
| vs f64 双舍入 | 991 | 1025 | +34 | 0 | 0 | 0 |
| `+fma` 两构建互比 | 0 | 0 | 0 | 0 | 0 | 0 |

**差异可能来源（并列记录）**：① 交付日志**未记录 LCG 参数/种子与样本输入**，其 991/4096 无法逐位复现（本复核用自己的
可复现生成器）；② 「非融合」参考的实现细节可能不同（是否 `black_box` 阻断、是否被工具链收缩），该类计数对实现敏感；
③ 输入值分布若不同（如指数跨度不同），会影响融合/非融合差异率。**结论方向不受影响**：GNU 默认与硬件 FMA 差异约 24–25%，
MSVC 默认与 `+fma` 构建为 0 —— 交付的核心判断在我独立数据上同样成立。

### 2.2 残余样本（按 Arch 要求给具体位型）

GNU 默认 vs 硬件 FMA，1e6 组内 **2 ULP** 个例（`qa-probe-summary.json` 的 `SAMPLE_WORST`）：

```
a_bits=bf7d32d9 b_bits=3fd57e84 c_bits=3fd33468  got(fmaf)=39bf7dde  want(hw fma)=39bf7de0  ulp=2
```

另有 5 个 1 ULP 样本（输入位型 + 两结果位型 + ULP）见 `probe-gnu-default.log`；
其余三构建在 1e6 组内无差异样本（0 条）。

## 3. 代码定位独立核（自写全仓扫描）

`qa_scan_mul_add.py` 扫描 `src/` 与 `tests/` 的 `mul_add(` 调用、相关 helper 与注释（29 条命中）：

- **产品 `f32::mul_add` 调用点恰好 3 行 / 2 文件**：
  - `src/physics/collision/gjk/polytope.rs:69`、`:70`（`face_norm2` 点积链；**碰撞 host 路径的唯一 `mul_add` 使用点**——`src/physics/collision/` 下只有这 2 处调用）；
  - `src/render/kernel/megakernel_arrays.rs:189`（光源方向归一化 `z.mul_add(z, y.mul_add(y, x*x)).sqrt()`，产品站点但不是本 case 路径）。
- **测试站点 1 处**：`src/runtime/transfer/kernel/tests/precision.rs:72`（`assert_eq!(values[0], a.mul_add(b, -1.0))`）。
- **团队先例 helper**：`src/physics/collision/relmat.rs:26` `fma_f32`、`src/render/bvh/aabb.rs:132` `fused_mul_add`
  （均为「f64 承载乘积 + 单次回舍」，注释明确避免 `f32::mul_add`）；测试侧 `tests/g10_p5_sdf_relmat.rs:416` 同名 helper。
- 交付 README 的定位与我的扫描一致；只发现一处**注释行号引用漂移**（见观察 3）。

## 4. 排除项自测（逐项命令/结果，原始输出在 `logs/`）

| 排除项 | 我的做法 | 结果 |
| --- | --- | --- |
| `CARGO*`/`RUST*` 环境变量 | 每格清空前/后计数记录（均为 0）；仅受控设置 `RUSTFLAGS`/`CARGO_TARGET_DIR` | 结果不由环境变量改变 |
| `-C link-dead-code=yes` | MSVC 显式 `RUSTFLAGS=-C link-dead-code=yes`（⑦）；GNU 用删除 `.cargo/config.toml` 的 worktree（⑧） | ⑦ 仍红 `6.351978e-7`；⑧ 仍绿 `3.838112e-7` → 非原因 |
| `--release/opt-level` | GNU release（⑤）、MSVC release（⑥） | GNU 绿 `3.838112e-7`、MSVC 红 `6.351978e-7`，与 debug 同结论 |
| 产物来源 | 每格全新专用 target（deps 62/65/67），删空重建 | 两侧结果均复现，与旧产物无关 |
| fixture/输入 | 4 个关键输入 `git hash-object` 与 `HEAD:eac9af71` 逐字节比对 | 4/4 一致（`qa-exclusions.json.input_identity`） |
| `G2_DUMP_NATIVE_ERROR` | 同二进制设置该变量重跑 | exit 0 / `1 passed`（结果不变；该变量只在 `native_error_seed_is_recorded` 里写 dump） |

## 5. 口径核对（交付证据未改判据/未宣称已修复）

- **未宣称已修复/已通过**：交付 README/receipt/commit 里没有「已修复」「修复完成」「已通过」「fixed」等表述；
  `pass` 只用于描述 **GNU 默认构建的测试结果**，并立即注明是「非合规 `fmaf` 的偶合」；处置候选（①~④）明确标注「不实施」。
- **明确「G6 绿系偶合 / G01 host-msvc 红是忠实结果」**：README §0.3/§0.5/§4、receipt `root_cause`/`r2_impact` 均写明；与我的复现一致。
- **未改产品/判据/门禁**：`git diff eddad9c0..7c401282 -- src tests fixtures scripts .normify/gates.toml Cargo.toml Cargo.lock` **为空**；
  交付 receipt `boundaries.modified_product_code/modified_judgment_tolerance_exclusions/modified_gates_or_g01_scripts` 均为 false，与 diff 相符。
- 交付 `head` 记为 `47f1f7a3`、注明实验执行时 `eddad9c0` → `7c401282`；我核对 `eddad9c0..7c401282` 产品路径零改动，成立。
- 本复核自跑门禁（`normify gate 2026-10-25-case-evidence-frame-diagnosis-independent-check`）：
  fmt / clippy / clippy-cuda 通过、**unit 597 passed / 0 failed**，EXIT 0；`normify check` 全部检查通过（EXIT 0）。
  说明：该树（HEAD `00831281`）已被交付前一轮 gate 覆盖，clippy/fmt 属 cargo 缓存命中；unit 测试真实执行。

## 6. 观察（7 项，均非阻断）

1. **探针可复现性不足（最重要）**：交付 `probe-fmaf.log` 只写「4096 组确定性 LCG」，**未记录 LCG 参数/种子、未给差异样本输入位型**，
   故其 `991/4096` 无法被独立复现；我用自己的生成器得 1025/4096（25.02%）。建议后续诊断/实现片把生成器参数与至少 5 条差异样本位型写入证据，
   否则「残余率」无法作为修复验收门槛（这正是 Arch 要求的口径）。
2. **「非融合」参考计数差异较大**（交付 GNU 986 / MSVC 55；我 1447 / 877）：参考实现细节与输入分布都可能影响该计数；
   该行不宜作为修复门槛，应以「vs 硬件 FMA」为主判据。
3. **注释行号引用漂移**：交付 README §3.6 引 `relmat.rs:10-15` 的「刻意不用 `mul_add`」注释，实际在 `relmat.rs:19-24`（`aabb.rs:126` 正确）。
4. **交付证据内部数字不一致（非 clean 反例）**：同一「主 `target/`」的 deps 文件数，README 写 14534、receipt 写 13262（应为不同时刻测量）。
   该对象只是反例、不参与结论，但建议标注测量时刻或统一。
5. **1e6 规模出现 2 ULP 个例**：交付只报 4096 组（全 1 ULP）；我在 1e6 组找到 1 个 2 ULP 差异（位型见 §2.2），
   说明非合规 `fmaf` 偏差并非严格 1 ULP 上界——若以「≤N ULP」作为修复验收门槛，需按规模/分布定界。
6. **f64 双舍入先例在观测规模内未被证伪**：团队先例 helper（relmat/aabb 的 f64 承载法）与硬件 FMA 在 4096 与 1e6 上均 0 差异；
   这支持「纯 f32 软件正确舍入」候选（选项 b），但属样本观测、不是数学证明（双舍入理论角落未穷举），验收应保留独立样本/规模要求。
7. **G01 host-msvc 全量数字未在本片重跑**：`689 passed / 1 failed / 137 ignored` 引用 Arch 复现；本片以 MSVC 单元格独立复现了同一失败与同一
   frame 值（`6.351978e-7`），两者一致；如需全量复核可另开 CPU 复核格。

## 7. 覆盖与未覆盖

已覆盖：微探针 4 构建 ×（4096 + 1e6）× 3 参考 + 已知向量；产品级 frame 矩阵 8 格 + 环境变量格；
排除项 6 项；全仓 `mul_add` 扫描（产品/测试分开）；交付口径与 diff 边界核对。

未覆盖（如实登记）：Linux/其它 CPU/其它 rustc 版本；mingw-w64 `fmaf` 的具体实现/反汇编（交付亦未覆盖，行为证据已足）；
「无 FMA3 的 CPU」回退路径；微探针输入分布之外的残余率外推；G01 全量重跑（见观察 7）；数值机制之外的 `render/` 光源路径影响（站点已定位，未测）。

## 8. 复现与显式路径

```powershell
& target\toolchains\powershell-7.6.7\bin\pwsh.exe -NoProfile -File docs\development-evidence\case-evidence-frame-diagnosis-independent-check\run_probe.ps1
& target\toolchains\powershell-7.6.7\bin\pwsh.exe -NoProfile -File docs\development-evidence\case-evidence-frame-diagnosis-independent-check\run_frame_matrix.ps1
& target\mjwarp-parallel\venv\Scripts\python.exe docs\development-evidence\case-evidence-frame-diagnosis-independent-check\qa_scan_mul_add.py
& target\mjwarp-parallel\venv\Scripts\python.exe docs\development-evidence\case-evidence-frame-diagnosis-independent-check\qa_summarize.py
```

本目录入库文件（`docs/development-evidence/case-evidence-frame-diagnosis-independent-check/`）：
`README.md`、`receipt.json`、`artifacts-sha256.json`、`qa_fmaf_probe.rs`、`qa_scan_mul_add.py`、`qa_summarize.py`、
`qa_artifacts_sha256.py`、`qa-probe-summary.json`、`qa-exclusions.json`、`qa-mul-add-scan.json`、`frame-matrix.json`、
`frame-matrix-summary.txt`、`probe-gnu-default.log`、`probe-msvc-default.log`、`probe-gnu-fma.log`、`probe-msvc-fma.log`、
`probe-toolchains.txt`、`run_probe.ps1`、`run_frame_matrix.ps1`、
`logs/`（8 格 × 断言/打印原始 cargo 日志 + 环境变量格，共 17 份；env/toolchain 记录已并入 `qa-exclusions.json`）。

不入库（可由脚本重建）：`target/qa-frame-diag/{tree,tree-noconfig,frame,probe}/**`。
