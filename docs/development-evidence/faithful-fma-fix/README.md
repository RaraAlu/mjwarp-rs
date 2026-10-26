# faithful FMA 处置 A 版：GJK `face_norm2` 显式非融合（对齐上游 CPU 参考 recipe）

- 任务：`2026-10-25-g10-p5-faithful-fma-fix-r2`（用户 2026-10-26 最终拍板 **A**；接替被放弃的 B 版
  `2026-10-25-g10-p5-faithful-fma-fix`）
- 原则：**标准依据是数据向最可靠的依据靠拢** —— host 数值链对齐上游 CPU 参考（两次舍入），
  device 对齐 pinned GPU PTX（`fma.rn` 融合），render / relmat 对齐各自的 GPU fused 参考。
- HEAD（改动前/后）：`7eac444f`（`7094363a` r2 登记、`7eac444f` QA 复核改 A 版）
- 改动：**仅 2 个 `src/` 文件，+8/−4 行**（`logs/src.patch`）：
  - `src/physics/collision/gjk/polytope.rs::attach_face`（`face_norm2`）：`mul_add` 链 → 上游 CPU
    源码顺序的**显式两次舍入**（`head = r[0]*r[0]; tail = r[1]*r[1] + head; face_norm2 = r[2]*r[2] + tail`），
    并写注释说明「对齐上游 CPU 参考 recipe（两次舍入）；GPU pinned ccd PTX `fma.rn` 属 device 侧口径」。
    不用 helper、无 f64、不出现 `mul_add`。
  - `src/render/kernel/megakernel_arrays.rs::light_data_poses`：光源方向归一化恢复
    `crate::math::fma::fma_f32(z, z, fma_f32(y, y, x * x))`（旧 B patch 的 megakernel 段）——render 参考是
    GPU `use_fast_math=true` fused，故保留精确 FMA helper 对齐单次正确舍入。
- **不改**：`relmat.rs`（已是 helper）、`render/bvh/aabb.rs`（实现不动）、判据/容差/期望值
  （`case_evidence.rs` 的 4.8e-7 与 4 ULP）、`case_compare.rs` 参考路径（仍读 CPU 参考
  `upstream-mjwarp-f32.json`）、`gates.toml`、G01 脚本、旧 CPU 参考、`upstream-mjwarp-f32-gpu.json`
  （保留归档、不参与判据）。
- 资源：**GPU 窗口 0**（全片纯 CPU；GPU 空闲）。未开窗、未跑 GPU 位级。
- 结论（**仅本片范围**）：5 个 case 与 CPU 参考**位级 0 差异**（三构建一致）；G5 位级不变；
  默认 gate 604/0/7 全绿；MSVC / GNU+`+fma` 的 `case_evidence` 由红转绿。**不宣称**精度达标/上游完全等价/
  双平台通过/原生放行；「真实无 FMA 机器未实机验证」照旧。

## 1. 为什么选 A：证据链

1. **GNU 默认 `f32::mul_add` 的库回退不是 IEEE 单次舍入 FMA**（`docs/development-evidence/case-evidence-frame-diagnosis/README.md`）：
   GNU 默认 frame `3.838112e-7`（< 4.8e-7 → pass）是**非合规库回退造成的偶合**；MSVC 默认 / GNU+`+fma` /
   MSVC+`+fma` 三者 frame 完全相同 = `6.351978e-7`（≈5.3 ULP，fail）。MSVC 默认与硬件 FMA 在 4096 组探针上
   **0/4096 差异**（GNU 默认对照见 `exact-f32-fma-relmat` §2/§3）。
2. **G2/G6 判据的参考是 CPU 后端、非融合 recipe**（`docs/development-evidence/g10-p5-g2-gpu-reference-capture/`）：
   采集自 warp 1.15.0 CPU 后端（`upstream-mjwarp-f32.json`，`device=cpu`）；参考实际生成的 C++ 中
   `mul`/`add` 是**独立语句**（不融合），同目录 4 个 `ccd_kernel*.o` 的 FMA 指令字节扫描 = 0（阳性对照 1 处命中，
   方法有效）。而 pinned GPU `ccd` PTX 是**全融合**：`fma.rn.f32 = 1480`、`mul.rn/add.rn = 0`
   （`docs/development-evidence/g10-p5-gjk-epa/README.md` §1/§4）。→ 两套参考语义不同，host 到底对齐谁必须先定。
3. **B（只切 GPU 参考）实测不可行**（Arch 2026-10-26 纯 CPU scratch dry check，已记入任务 note）：
   同一度量下两套上游参考自身就差超限——ellipsoid-box frame `8.192876e-7`、mesh-mesh `5.364418e-7` + dist 5 ULP、
   convex-mesh pos `6.00 ULP@尺度`、degenerate-mesh dist `45 ULP`（仅 cylinder-box 一致）；scratch
   （HEAD + 旧 helper patch + 判据参考临时切 GPU JSON）跑 `case_evidence` **4/5 case 红**：
   ellipsoid-box `8.1928766e-7`（>4.8e-7）、mesh-mesh `5.364418e-7`、convex-mesh pos `6.00 ULP`、
   degenerate-mesh dist `26 ULP` + frame `2.3841858e-6`，仅 cylinder-box 过，exit 101。
   → host 当时的 helper（全融合）与 GPU 整核收缩差 8.19e-7 量级，**不能靠切参考解决**。
4. **A 为何是「数据向最可靠依据靠拢」**：判据给的是 CPU 参考（4 ULP/4.8e-7 就是按它冻结的），
   所以 host 链应复刻 CPU 的两次舍入 recipe；device 侧仍按 pinned PTX 的 `fma.rn` 语义（不属 host 判据）。
   render/relmat 的参考分别是 GPU fast-math 与 warp `mat.h`（都显式 fused），故那两处保留精确 FMA helper。
   **C（容差重推）/ D（按 case 缩小 4 ULP 范围）**都要动判据/验收范围，在参考侧已可位级复刻的前提下没有依据，
   因此不放宽——本片不改判据/容差/期望值/参考路径。

## 2. 硬验收①：默认门禁 + 5 case 位级（三构建）

`case_evidence`（`cargo test --locked --lib physics::collision::gjk::case_evidence -- --nocapture`；GNU 默认 /
MSVC / GNU+`-C target-feature=+fma`）：

| 构建 | 改动前（HEAD `7eac444f`） | 改动后（本片） |
| --- | --- | --- |
| GNU 默认 | pass；ellipsoid-box frame **3.838112e-7**（13/18 位等） | **pass**；frame **18/18，max abs 0e0** |
| MSVC | **FAIL**；frame **6.351978e-7**（13/18） | **pass**；frame **18/18，max abs 0e0** |
| GNU + `+fma` | **FAIL**；frame **6.351978e-7**（13/18） | **pass**；frame **18/18，max abs 0e0** |

改动后三构建的 5 个 `[判据]` 行**逐位相同**（三个日志）：

| case | count | dist 位等 / max ULP | pos 位等 / max ULP@尺度 | frame 位等分量 / max abs |
| --- | --- | --- | --- | --- |
| cylinder-box | 8/8 | 8/8，0 | 8/8，0.00 | 72/72，0e0 |
| ellipsoid-box | 2/2 | 2/2，0 | 2/2，0.00 | **18/18，0e0** |
| mesh-mesh | 2/2 | 2/2，0 | 2/2，0.00 | 18/18，0e0 |
| convex-mesh | 2/2 | 2/2，0 | 2/2，0.00 | 18/18，0e0 |
| degenerate-mesh | 2/2 | 2/2，0 | 2/2，0.00 | 18/18，0e0 |

→ 5 个 case 与 CPU 参考 **dist/pos/frame 全部位级 0 差异**；`test result: ok. 3 passed; 0 failed; 2 ignored`
（ignored 为探索/性能项，非判据）。默认全量 `cargo test --locked` = **604 passed / 0 failed / 7 ignored**
（111 个 test binary；lib 454/0/5；与改动前基线相同，无新增测试）。

## 3. 硬验收②：G5 位级不变（relmat / SDF / gjk_epa）

| 目标 | 改动前 | 改动后 | 结论 |
| --- | --- | --- | --- |
| `cargo test --locked --lib -- relmat` | 4 passed | 4 passed | 同 |
| `--test g10_p5_sdf_relmat` | 4 passed；`rel_mat_warp` **10/10 world**（8 world 可区分 b·a）；sdf-rot-ident / sdf-rot-sdf / sdf-rot-box / sdf-rot-both / sdf-rot-rz-rx / sdf-box(G3 回归) 各 **80/80 contacts bit-equal，dist/pos/frame max_abs 0e0** | 同上，逐值相同 | **diff=0** |
| `--test g10_p5_sdf_host` | 1 passed | 1 passed | 同 |
| `--test gjk_epa` | 3 passed（duplicate_keyed_rows：cylinder 1、其余 0） | 3 passed（同） | 同 |

（`relmat`/`sdf` 实现本片未改，G5 位级不变属预期；日志 `logs/pre-fix-g5-*.log` / `logs/post-fix-g5-*.log`。）

## 4. 硬验收③：三构建 + 工具链无关 + G01 定向

- 三构建 `case_evidence` 全绿且 5 case 判据行逐位一致（§2）——显式两次舍入不依赖 libm 回退，
  是本片的关键卖点（改动前 GNU 默认与 MSVC/`+fma` 结论相反）。
- **G01 定向（不跑全量 G01）**：
  - `host-gnu`（G01 内为 `cargo test --locked`，GNU 默认工具链）：改动前后均 exit 0 / 604/0/7。
    注意：**改动前它也是绿的**，但绿的原因是非合规 libm 回退偶合（frame 3.838112e-7 恰低于 4.8e-7）；
    改动后仍绿且变为**位级 0 差异**。因此「host-gnu 由红转绿」的说法不成立，如实记为本条。
  - `host-msvc`（G01 内为 `cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features native-model-probe,cuda-probe`，
    此处只跑定向的 `case_evidence`）：改动前 **FAIL**（ellipsoid-box frame 6.351978e-7）→ 改动后 **pass**（0e0）。
- G01 全量（`scripts/test-windows-g01.ps1 -AllFeatures -FullRegression`）按 acceptance「只跑定向」未执行；
  留待 Arch/QA 在片外里程碑按需跑。
- **门禁**：`normify gate 2026-10-25-g10-p5-faithful-fma-fix-r2` exit 0（fmt / clippy / clippy-cuda / unit 全通过，
  `unit` = 604 过 0 败）；`normify check …` → **全部检查通过**（最终一轮，见 `logs/normify-gate.log`）。

## 5. 硬验收④：`src/` 站点审计（改动后重扫）

命令：`grep -rn 'mul_add|fma_f32\(|fused_mul_add' src/`（结果见 `logs/src.patch` 与下方分类）。

| 分类 | 站点 | 语义 | 依据 |
| --- | --- | --- | --- |
| 产品 host（非融合） | `src/physics/collision/gjk/polytope.rs::attach_face`（`face_norm2`） | **两次舍入**（`*` + `+` 独立） | 对齐上游 CPU 参考 recipe（本片；G2/G6 判据参考） |
| 产品 host（fused） | `src/render/kernel/megakernel_arrays.rs::light_data_poses` | `fma_f32` 单次正确舍入 | render 参考 = GPU `use_fast_math=true`（fused） |
| 产品 host（fused） | `src/physics/collision/relmat.rs`（3 处 `fma_f32`） | 单次正确舍入 | warp `mat.h` `muladd→fmaf`（fused；G5 判据） |
| 产品 host（fused，本地 helper） | `src/render/bvh/aabb.rs::fused_mul_add`（6 处调用） | 单次正确舍入 | 冻结 S7 参考的 `fma.rn`；**本片实现不动** |
| device 侧 | CUDA 内核（`src/physics/collision/cuda/mod.rs` 等注释引 pinned ccd PTX） | `fma.rn` 全融合 | pinned GPU PTX（`fma.rn=1480`），不在 host 判据内 |
| 测试/诊断 | `src/math/fma/tests.rs`（`f32::mul_add` 判别力自证、硬件参考）、 `src/runtime/transfer/kernel/tests/precision.rs:72` | 测试专用 | 非产品数值链 |

- **产品代码里已无 `f32::mul_add` 调用点**：`mul_add` 仅剩注释/测试/判别力自证；产品乘加站点即上表 4 类。
- render `aabb.rs` 与 `relmat.rs` 的实现本片未改（仅 polytope 改语义、render 光源归一化恢复 helper）；
  render 站点恢复 helper 的数值影响面依据 `docs/development-evidence/faithful-impact-measurement/README.md`：
  20 个定向项中 19 项在两口径下不翻转（render S7b1/S7b2/S7b3/S8 位级/哈希相同），唯一翻转项即本片的
  `case_evidence`。本片未开 GPU 窗口复跑 render GPU 位级（窗口 0 的既定依据即该影响面实测）。

## 6. 硬验收⑤：NP01 站点级口径登记

- 清单：`docs/development-evidence/np01-type-mapping/inventory.json` 新增 `hostSiteRegistration`（新增块、
  不改原 30 条 CUDA 内核行的分类/计数）：
  - `siteSemantics` 3 条：`gjk_ccd_epa_host`（host=CPU recipe 两次舍入、device=pinned ccd PTX `fma.rn` 融合）、
    `render_light_dir_normalize`（fused，GPU fast-math 参考）、`relmat_relative_matrix`（fused，warp `mat.h`）；
  - `hostCoverage` 3 条（gjk / ccd / epa host 路径）：补上原 30 条未覆盖的 host 数值链，状态如实写
    「站点口径已登记；逐字段输入→输出类型表/逐表达式位级对照待补」。
- README：`docs/development-evidence/np01-type-mapping/README.md` 追加「host 站点口径与 gjk/ccd/epa
  host 覆盖（2026-10-26）」一节，引用本证据。
- **只登记口径，不改判据/容差**；NP01 仍未验收（30 条逐字段表与 host 覆盖待补项照旧如实列出）。

## 7. 边界 / 未覆盖 / 红线

- **真实无 FMA 机器未实机验证**：本片未新增/未实机验兜底路径（沿用 `exact-f32-fma-relmat` 的
  seam + 默认 gate 覆盖结论）；不写「兜底已实机验收」。
- NaN payload 不作位级判据；边界测试计数以 **16** 为准（本片未新增边界测试）；gate unit 引当前值 **604**。
- 不宣称精度达标 / 跨环境全绿 / 上游完全等价 / 双平台通过 / 原生放行 / G10 已实现 / 完整物理通过。
- 不宣称「host 与 device 数值等价」：本片是「host 对齐 CPU 参考、device 按 pinned PTX」的两口径并存；
  `upstream-mjwarp-f32-gpu.json` 仍是归档证据，不是判据。
- 未跑全量 G01、未开 GPU 窗口；render GPU 位级/性能未复跑（依据影响面实测）。

## 8. 复现

```powershell
# 三构建 case_evidence（本片核心验收；均在 CPU）
cargo test --locked --lib physics::collision::gjk::case_evidence -- --nocapture
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --lib physics::collision::gjk::case_evidence -- --nocapture
$env:RUSTFLAGS='-C target-feature=+fma'; cargo test --locked --lib physics::collision::gjk::case_evidence -- --nocapture
Remove-Item Env:RUSTFLAGS

# G5 + 默认门禁
cargo test --locked --lib -- relmat --nocapture
cargo test --locked --test g10_p5_sdf_relmat -- --nocapture
cargo test --locked --test g10_p5_sdf_host -- --nocapture
cargo test --locked --test gjk_epa -- --nocapture
cargo test --locked
```

## 9. 证据文件

- 本 README、`receipt.json`（含 `task_id`）
- `logs/src.patch`（`src/` 改动原文，+8/−4）
- `logs/pre-fix-case-evidence-{gnu-default,msvc,gnu-fma}.log`、`logs/post-fix-case-evidence-{gnu-default,msvc,gnu-fma}.log`
- `logs/pre-fix-g5-*.log`、`logs/post-fix-g5-*.log`（relmat lib / sdf_relmat / sdf_host / gjk_epa）
- `logs/gate-unit-full-summary.log`、`logs/gate-fmt-check.log`、`logs/gate-clippy-default.log`、
  `logs/gate-clippy-cuda.log`、`logs/normify-gate.log`
- 上游与前置证据：`docs/development-evidence/case-evidence-frame-diagnosis/`、
  `docs/development-evidence/g10-p5-g2-gpu-reference-capture/`、`docs/development-evidence/g10-p5-gjk-epa/`、
  `docs/development-evidence/exact-f32-fma-relmat/`、`docs/development-evidence/faithful-impact-measurement/`
