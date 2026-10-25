# G10@P5 G2（GJK/EPA 前置片）独立复核（QA）

- 任务：`2026-10-25-g10-p5-gjk-epa-independent-check`（依赖 `2026-10-25-g10-p5-gjk-epa`）
- 复核对象：**`900d1fe`**（G2 交付）+ **`735abaf`**（收口与证据口径修正）
- 复核基线：开工 HEAD `6ae95eb4`（review start 提交）；复核期间 HEAD 前移到 `48eaf399`，其间新增 `3b0cb082`（G0b 更正）、`475b4cf8`（公开 API + 复位集成测试）、`e190d5ec`、治理提交。**G2 窄相源码（`src/physics/collision/gjk/**`）自 `735abaf` 起未再改动**；`475b4cf8` 只改 `src/physics/mod.rs`（re-export）与 `tests/gjk_epa.rs`（重新落地）。
- 环境：Windows；pinned venv Python 3.12.13 / MuJoCo 3.15.0 / warp 1.15.0 / mujoco_warp 3.15.0；CPU 侧复核 + 一个约 2 分钟的 GPU 顺带项（见 §8）。
- 结论：**0 阻断，G2 数值判据从 QA 侧成立**。上游 f32 参考独立重采逐位一致（5/5）；5 case 判据表逐项复算与交付一致（全部 ≤4 ULP，其中 2 个 case 全字段位级）；27 ULP 根因（输入位姿口径）在同一二进制上以「换位姿不换代码」复现（native 27 ULP → 上游 f32 1 ULP）；native E 表独立复算与 `native-error-seed.json` 一致且仅作记录；正式门禁测试非 ignore、native 位级项 ignore；范围与不回归成立。另有 **1 条一般 + 4 条建议**（均为交付证据的陈旧/措辞问题，不影响判据结论，见 §9）。

> 口径：本复核只做独立复核与复跑，不改产品代码/测试/夹具/G2 原证据；本目录只写复核证据。**未跑全量 G01**（G2 无构建配置变更，按约定只跑证据门禁 fmt + json + forbid_claim）。

## 1. QA 独立口径（不复用交付对照代码）

| 项 | QA 自写 harness | 独立性 |
| --- | --- | --- |
| 上游 f32 参考重采 | `qa_capture_upstream.py`：自己用 mujoco_warp（warp **CPU** 后端）对 5 个 case 各捕获两次，输出 17 字段 + `geom_xpos/xmat` | 不读交付 `capture_mjwarp_f32.py`；只复用同一 pinned venv 与模型/qpos 夹具 |
| 判据复算 | `harness/main.rs`（Rust，path 依赖产品 crate，调用公开 `ccd_pair`）：自己构造 `Geom/MeshView` 输入、自己排序/ULP 比较 | 不读 `case_compare.rs` 的实现；输入来自 QA 自己的 JSON |
| E 表复核 | 同上 harness：自己算 `ours vs native`、`mjwarp vs native` 的 E 分布并与 `native-error-seed.json` 对账 | 不调用交付 `case_evidence.rs` |
| 证据/范围/门禁 | `qa_evidence_check.py`：31 项控制检查（blob、scope、gate 属性、seed 只写不读等） | — |
| GPU 顺带项 | `qa_g0b_cuda_recapture.py`：G0b 10 case CUDA 两次捕获（QA 自己实现） | 见 §8 |

产物：`qa-upstream-mjwarp-f32.json`（QA 重采 + 与交付逐位对账）、`qa-case-inputs.json`（QA 的输入/位姿/参考）、`harness-report.json`、`harness-run.log`、`evidence_check_report.json`、`unit-full.log`、`g0b-cuda-recapture.{json,log}`、`capture-rerun.log`、`delivered-unit-gate-735abaf.json`、`harness/{Cargo.toml,main.rs}`。

## 2. 上游 f32 参考：provenance 与两次捕获位等（独立重采）

- provenance 全部核实：`task_id`、`upstream_revision=71da24d9…`、脚本 sha256（= `capture_mjwarp_f32.py` 当前文件）、版本 `python 3.12.13 / mujoco 3.15.0 / warp 1.15.0 / mujoco_warp 3.15.0 / numpy 2.5.4`、`device=cpu`、`authoring_only/reads_product_tree=false`。
- 每 case `xml_sha256` 与仓库 XML 一致；`qpos_bits` 与夹具 qpos 位型一致。
- **QA 重采两次 → 逐位相同 5/5**；重采结果与交付 `upstream-mjwarp-f32.json` **逐位一致 5/5**（count、17 字段、`geom_xpos`、`geom_xmat` 全 17×f32 位比）；count = **8/2/2/2/2**。
- 结论：参考不是「读交付日志」，我能在 CPU 上逐位重建它。

## 3. 5 case 判据表（QA 逐 case 独立复算）

判据（唯一）：`ours` vs 同配方上游 `mujoco_warp` f32（CPU 后端）的 `dist/pos/frame/normal` ≤4 ULP、位级优先；pos 用向量尺度 ULP，frame/normal 用绝对误差（1 ULP@1.0 = 1.192e-7）。判据输入 = 上游 f32 运动学生成的 `geom_xpos/geom_xmat`（QA 从自己的重采 JSON 取，纯合成）。

| case | count | dist | pos | frame / normal | QA 判定 |
| --- | --- | --- | --- | --- | --- |
| `cylinder-box` | 8/8 | **位级 8/8，0 ULP** | **位级 8/8，0** | **位级 72/72，0** | 通过（全字段位级） |
| `ellipsoid-box` | 2/2 | **位级 2/2，0** | 1/2 位等，**2.00 ULP@尺度** | 13/18 位等，max abs **3.838e-7 = 3.22 ULP@1.0** | 通过 |
| `mesh-mesh` | 2/2 | 1/2 位等，**2 ULP** | **位级 2/2，0** | **位级 18/18，0** | 通过 |
| `convex-mesh` | 2/2 | **位级 2/2，0** | **位级 2/2，0** | **位级 18/18，0** | 通过（全字段位级） |
| `degenerate-mesh` | 2/2 | 0/2 位等，**1 ULP** | **位级 2/2，0** | **位级 18/18，0** | 通过 |

- 5/5 case 的 contact count 与 `geomcollisionid` 一一对应（`id_mismatch=0`）；每 case 全部分量 ≤4 ULP，且 **2 个 case 全字段位级**。
- 与交付 README §9 表逐项数字一致（dist/pos/frame、位等级数、max abs 均同值）；这次是 QA 自己的输入与比较器算出，不是读交付表。
- 判据实现澄清（非问题）：交付门禁把 frame 判据实现为 `frame_max_abs <= 4.8e-7`（= 4 ULP@1.0 的绝对上限），与「单位向量按 ULP@1.0」口径一致；对 frame 分量绝对值 <1 的分量，其在分量自身 ULP 意义下可 >4（单位向量近零分量 ULP 无意义），这是口径选择，已在交付 README/Chief 口径内。

## 4. 27 ULP 根因复核（同一二进制，只换输入位姿）

交付声称：native 位姿输入时 `degenerate-mesh` dist 27 ULP；改用上游 f32 位姿后 1 ULP；**未改窄相表达式**。QA 用**同一份 harness 二进制**、同一套 `ccd_pair` 选项，仅替换输入位姿：

| 输入位姿 | degenerate-mesh dist max | 其它 case（对照） |
| --- | --- | --- |
| QA 自算 native（`mj_forward` f64 → f32 位姿） | **27 ULP** | cylinder 0、ellipsoid 0、mesh-mesh 2、convex 0 |
| 上游 mjwarp f32 运动学位姿 | **1 ULP** | 同上（见 §3） |

- ① 27 ULP 可复现；② 换上游 f32 位姿 → 1 ULP；③ 两次只改输入、同一代码路径、同一选项，说明差异来自**输入位姿口径**而非窄相表达式；窄相源码中**没有按 case 名分支/调参**（对 `core/phase/ccd/epa/support/simplex/polytope*/contact/math` grep `degenerate|cylinder-box|mesh-mesh|convex-mesh|ellipsoid-box|27` 命中 0）；`git diff 900d1fe^..900d1fe` 是整包新增（无「先 27 后调表达式」的可审计历史，见 §10 限制）。
- 交付 `input_alignment` 里「用 numpy 复算 mjwarp FK」的对齐列更差（degenerate 34 ULP）与最终采用「直接取上游 f32 `geom_xpos/xmat`」不矛盾：前者确实不是 mjwarp 位级 FK；最终判据输入改用真实上游 f32 位姿后 1 ULP。QA 复算的是最终口径。

## 5. `native-error-seed.json`：只作记录，未作判据/容差

- QA 独立算 E 表（ours vs native；mjwarp vs native；每 case 每 world 按 `(dist,pos,id)` 排序后取 `|Δ|`，共 **208** 个分量）：`count=208, min=0, median=0, max=8.1928766e-7, over_1e6_ratio=0`；与交付 `native-error-seed.json` 两项**逐值一致**（count/max/over 全命中；max 与交付同 f32 值）。
- 判据测试的阈值是**源码常量**（`dist≤4`、`pos_scaled≤4.0`、`frame_abs≤4.8e-7`），没有从 seed 读阈值；`native_framework_observed_bounds` 只出现在 receipt/README 的「仅记录、非判据」上下文。全仓库 `grep native-error-seed` 只有 case_evidence.rs 的**写入**与证据文本，没有任何代码读取它作为容差。

## 6. 门禁真实性

- 正式判据测试：`physics::collision::gjk::case_evidence::upstream_f32_within_4ulp` —— **非 ignore**，断言 count 相等 + dist≤4 ULP + pos_scaled≤4.0 + frame_abs≤4.8e-7；另一个同名打印表测试 `our_contacts_match_upstream_f32_within_4ulp`（case_evidence/case_compare 各一个）只断言 count，属展示项，不构成放行。
- 探索项：`native_bitwise_exploration` 带 `#[ignore = "探索项：native 是 f64，f32 实现不可能位级一致，不作门禁"]`，确认仍为 ignore。
- QA 实跑：`cargo test --locked --lib upstream_f32_within_4ulp -- --nocapture` → **3 passed / 0 failed / 0 ignored**，打印的判据表与 QA 独立表逐项同值（见 §3）。
- 全量 `cargo test --locked`（在 G2 收口后的 HEAD `1be51332` 上实跑）：**561 passed / 0 failed / 1 ignored**（唯一 ignored 即 native 探索项）。逐提交口径：`735abaf` 自带 unit gate artifact = **558 passed / 0 failed / 1 ignored**（见 `delivered-unit-gate-735abaf.json`）；`475b4cf8` 重新落地 `tests/gjk_epa.rs` 的 3 个集成测试后为 561。

## 7. 分类更正与不回归（范围）

- 参考分类独立核实：读 pinned 上游 `collision_driver.py` 的 `MJ_COLLISION_TABLE` + `_narrowphase` 里 `NATIVECCD` 覆盖（`(BOX,BOX)→PRIMITIVE`）：`sphere-box(SPHERE×BOX)=PRIMITIVE`、`capsule-box(CAPSULE×BOX)=PRIMITIVE`、`box-box-margin`/`box-box-multiccd(禁 nativeccd→PRIMITIVE)`；`cylinder-box(CYLINDER×BOX)=CONVEX`、`ellipsoid-box(ELLIPSOID×BOX)=CONVEX`、`mesh-mesh`/`convex-mesh(BOX×MESH)`/`degenerate-mesh`=CONVEX；`hfield-*`=CONVEX+专用 hfield kernel（G4），`sdf` 独立分支（G3）。→ G2 只用 5 个 CONVEX case 正确；PRIMITIVE case 未被记成 GJK/EPA 覆盖（README §5、receipt `g0b_classification_correction` 一致），球/胶囊 shrink 分支无 CONVEX 直达参考已列 unmet。
- `git diff 900d1fe^..900d1fe` **27 文件全在声明路径**：`src/physics/collision/gjk/**`、`src/physics/collision.rs`、`tests/gjk_epa.rs`、G2 证据；**未改** `Cargo.toml/Cargo.lock/rust-toolchain.toml`、`src/physics/collision/narrowphase.rs`（plane-sphere/sphere-sphere/plane-box 语义代码）/`filter.rs`/`sap.rs`/`nxn.rs`/`tests.rs`、S2–S8 渲染证据、d04 证据、渲染夹具。
- `735abaf` 只改 `.normify`、G2 证据、移除 `tests/gjk_epa.rs`（按拆分移入 `target/g10-p5-g2/pending-api/gjk_epa.rs`，已确认存在 7,040 B）。
- 不回归：全量单测 561/0（含既有 narrowphase 套件）通过；`cargo fmt --check` 通过（QA 实跑）。

## 8. GPU 顺带项：G0b 的 CUDA 两次捕获（补 G0b 复核限制项）

- 开窗前状态：GPU **4,137 MiB**、`--query-compute-apps` 无 CUDA 计算进程；用 QA 自己的 harness 在 `cuda:0` 上对 G0b 10 case 各捕获两次（首次内核编译约 45 s，总占用约 2 分钟）。
- 结果：**10/10 count 与交付夹具一致**；**两次捕获原始位等 9/10**（`sdf-box` 原始顺序不稳定，与交付声明的 raw=false 一致）；**与交付夹具按「worldid 分组 + 全字段（含 geomcollisionid）排序」位等 10/10**。
- 原始行顺序与交付运行不同（跨运行/调度差异）→ 进一步证明 G0b 结论：**原始顺序不是夹具稳定属性，比较必须用排序判据**；G0b 证据里「原始顺序只对本次运行成立」的建议由此得到独立证据。
- 结束：脚本内采样 4,455 MiB（进程退出前）；进程退出后 **4,137 MiB（= 开窗前基线）**、无本任务计算进程。
- 说明：Arch 要求「开窗前先报」，本轮消息通道不允许中途单发（报错为「等你的结果」），因此在本报告中如实报开始/结束数据；若流程上必须等授权，请指出，我下轮按窗口重跑对齐。

## 9. 问题清单（非阻断，建议 `normify fix`）

**一般**

1. **`gjk-epa-run.log` 是中间态日志，不构成最终判据的证据**：它不含最终 f32 判据的 `[判据]` 表，且仍引用**已删除**的测试 `case_compare::input_aligned_contract` / `ccd_contacts_match_pinned_reference`；README §7 把它写成「单测 + 集成完整输出」。期望：重跑最终套件生成日志，或注明该日志为中间态、最终判据以 `case_evidence::upstream_f32_within_4ulp` 运行为准。

**建议**

2. **判据测试模块路径写错**：receipt `final_criterion.gate_test` 与 README §9 写 `case_compare::upstream_f32_within_4ulp`；实际测试在 `case_evidence::upstream_f32_within_4ulp`（`case_compare` 的同名测试只断言 count）。同 receipt `reproduction.strict_contract_ignored` 指向已删除的 `input_aligned_contract`。期望：改成可执行的正确命令。
3. **unit 计数三处不一致**：receipt `gates.unit.tests_passed=555`、README §7 写 557、同提交 gate artifact 为 **558**（我的实跑在 `735abaf` 后 HEAD 为 561，含 API 片重新落地的 3 个测试）。期望：统一为同提交实测值。
4. **README §3 仍保留旧 native 对照表**（dist 18 ULP 等）且 §7 命令 `cargo test --locked --test gjk_epa` 在 `735abaf` 已无该测试目标，与 §9 最终口径/§10 说明冲突，易误读。期望：标注 §3 为探索性/历史，或更新命令。
5. **小数字陈旧**：receipt `artifacts.test.lines=445`，实际 `case_compare.rs` 455 行（仍 <500）。期望：刷新。

## 10. 限制与未做

- **未审计 G2 开发期未提交历史**：「未改窄相表达式」无法用单一新增提交 diff 证明；我用「同一二进制只换输入位姿」+「窄相无 case 名分支」证明差异来源，但无法证明开发过程中是否存在过临时调参后回退（无对应提交可查）。
- **位级未全达成**：5 case 全部分量 ≤4 ULP、2 个全字段位级；`ellipsoid-box` normal 3.22 ULP、`mesh-mesh` dist 2 ULP、`degenerate-mesh` dist 1 ULP 仍是数值差，不是位级；不声称与上游「完全等价」。
- **G2 的 5 case 是采样面**，不是全组合矩阵；MESH 图 hillclimb（大 mesh）、HFIELD/TRIANGLE/FLEX 支撑点、hfield 四接触、SDF、flex 路径未由本片数值覆盖（交付 unmet 已列）。
- **判据测试只在 CPU 宿主机**；G5 GPU 内核同口径对照未做。
- QA 的 E 表复算用于校核 seed（只作记录）；**native 容差未批准**，本轮没有把它当放行依据。
- 未跑全量 G01（G2 无构建配置变更）；GPU 仅用于 §8 的 G0b 顺带项，未用于 G2 数值判据。

## 11. 复现

```powershell
# 1) 上游 f32 参考重采 + 与交付逐位对账（CPU，不占 GPU）
target\mjwarp-parallel\venv\Scripts\python.exe -B target\qa-g2\qa_capture_upstream.py
# 2) QA Rust harness（path 依赖产品 crate；把 harness 目录放到 target/ 下即可：
#    Cargo.toml 的 mjwarp-rs path 指向仓库根；--offline）
cd target\qa-g2-harness; cargo run --offline   # -> target/qa-g2/harness-report.json
# 3) 证据/范围/门禁控制检查
target\mjwarp-parallel\venv\Scripts\python.exe -B target\qa-g2\qa_evidence_check.py
# 4) 交付正式门禁实跑（在可编译的 HEAD 上）
cargo test --locked --lib upstream_f32_within_4ulp -- --nocapture
# 5) GPU 顺带项（约 2 分钟，需 GPU 空闲）
target\mjwarp-parallel\venv\Scripts\python.exe -B target\qa-g2\qa_g0b_cuda_recapture.py
```

> 备注：复核期间共享工作区有与本任务无关的未提交 WIP（G1 模型字段）导致产品 crate 暂不可编译；QA harness 的最终运行是在 `git worktree add --detach target/qa-g2-head HEAD`（HEAD `48eaf399`，G2 窄相源码与 `735abaf` 相同）上构建的，运行后已移除该 worktree。仓库根 `Cargo.toml/Cargo.lock` 未被本复核改动（逐字节核对）。
