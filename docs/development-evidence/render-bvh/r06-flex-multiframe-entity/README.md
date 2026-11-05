# R06 多帧实体 A：`render_flex_multiframe`（3 帧逐帧口径 + 跨帧可见性 + 选项 A 审计）

- 任务：`2026-11-05-render-r06-flex-multiframe-entity`（owner Backend、p75、P8-渲染与BVH）
- 依赖：G-2 采集窗落库 3 帧参考（`2026-11-05-render-r06-flex-multiframe-capture-g2`，提交 `5f2bd0bb`，
  verdict `pass-margin`）+ refit 修复 r2（`ff8135e5`/`43295e49`）
- 边界：**零 GPU、不开窗、不执行 freeze**（冻结器只做 `-GuardsOnly` 结构自检）；不改 G-2 夹具/spec/判据；
  **现有 `tests/render_flex_validate.rs`（498 行）一行不动**
- **修订**：本 README 由小修卡 `2026-11-05-render-r06-flex-multiframe-entity-fix` 校正（跨帧判据改 **cam2-only**、
  选项 A 口径按 Chief 裁决改诚实措辞见 §5、行数/措辞同步）；实体判据语义与 G-2 夹具未变。

## 1. 结论

- 新增测试实体 `render_flex_multiframe`：GPU `#[ignore]` 用例 1 个；**普通 `cargo test` 可跑 CPU 自证 22 项**
  （本目标合计 `38 passed / 0 failed / 1 ignored`，其余来自共享 s7b1/allowlist 模块的自带测试）。
- 判据落地：逐帧 rgb ≤1 LSB 且 beyond=0；depth **frame-000 仅 cam2/(134,128)/flexid0 allowlist**、
  **frame-001/002 严格 bit-first**（allowlist 像素出现同像素同类型差 → 打印 flat/x/y/bits/ulp/seg 并大声失败，
  禁止自动扩 allowlist）；seg 逐像素全等；逐帧**整块多相机缓冲**两跑 sha 相等。
- 跨帧可见性由**实体自算**（从 npz 重算 **cam2-only** 的 `000→001`、`001→002` 变化像素各 **≥384**；
  全相机合计只作诊断打印；不读 `delta_selfcheck`；576 仅采集余量、不作验收阈）。
- **选项 A**（口径按 Chief 2026-11-05 裁决，见 §5）：**逐帧 state 刚性位姿建 3 个 context**，断言三者
  刚性签名（叶几何 + 位姿位型）一致（CPU 前提 + GPU 用例各一次），并断言 `RenderContext::refits()==0`
  （记录产品「每渲染重建合并 BVH」的架构事实）；另加「同 context 渲染对照」断言 frame-001/002 用
  `contexts[0]` 渲染与其自身 context 逐位相同。**不补 `body_pos` 偏移步骤**（产品路径不消费）。
- 冻结器/guards：`freeze-guards-r06-multiframe.ps1`（`-Attempt 1`，**2 exes + 26 inputs = 28**）与
  `guards-r06-multiframe.json`；`-GuardsOnly` → `GUARDS_STRUCTURE_OK suite=render-r06-multiframe
  exes_pending=2 inputs=26 inputs_sha256_ok=true classification=input=25,guard_script=1`。
- 行数棘轮：`tests/render_flex_multiframe.rs` **499**、`tests/support/render_bvh/multiframe.rs` **496**
  （均 ≤500；小修卡校正后）；`allowlist.rs` 210（+14 行仅新增 `assert_depth_strict`）；`render_flex_validate.rs` **498 未变**。

## 2. 实体结构

| 文件（新/改） | 行数 | 职责 |
| --- | --- | --- |
| `tests/render_flex_multiframe.rs`（新） | 499 | 22 项 CPU 自证 + 1 个 `#[ignore]` GPU 用例（逐帧/跨帧/选项 A/同 context 对照） |
| `tests/support/render_bvh/multiframe.rs`（新） | 496 | 3 帧 npz/文本夹具嵌入与解析、参考完整性与 provenance、刚性位等/位移、跨帧变化像素（**cam2-only 判据 + 全相机诊断**）、scene/flex/context 构造、逐帧判据实现 |
| `tests/support/render_bvh/allowlist.rs`（改） | 210 | 仅新增 `assert_depth_strict`（纯位等；`#[allow(dead_code)]` 供本实体使用） |
| `scripts/validate/render-bvh/freeze-guards-r06-multiframe.ps1`（新） | 205 | 冻结器（克隆 r06-s3b-flex；2 exes + 26 inputs = 28；`-GuardsOnly` 结构自检） |
| `scripts/validate/render-bvh/guards-r06-multiframe.json`（新） | — | 28 条目结构文件（2 exes 待冻结 + 26 inputs sha 自洽） |

## 3. 逐帧判据（GPU 用例）

| 项 | 判据 |
| --- | --- |
| 前置 | 逐帧 state_k 刚性数组（`geom_xpos`/`geom_xmat`/`light_xpos`/`light_xdir`/`qpos`）与 frame-000 **f32 位级相等**（证「只有 flex 动」） |
| rgb | 逐相机 `max_byte_diff <= 1` 且 `beyond_1_lsb == 0` |
| depth | frame-000：cam2 的 `(134,128)`（参考 seg=`(flexid0,9)`）允许 `ULP ≤4` 且 `beyond4=0`，其余严格位等；frame-001/002：**全严格 bit-first** |
| allowlist 边界 | frame-001/002 若 allowlist 像素出现同像素同类型差 → panic 打印 `flat/x/y/bits/expected/ulp/seg`，停手报用户；**不自动扩 allowlist** |
| seg | 逐相机逐像素全等（含 flex `(flexid,9)`） |
| 两跑 | 每帧渲染两次，**整块多相机缓冲** rgb/depth/seg sha256 相同（整块 sha 更强，等价逐相机相等） |

## 4. 跨帧可见性（cam2-only；实体自算，不读 `delta_selfcheck`）

- 从 3 帧 npz 重算**仅 cam2** 的相邻帧 `depth` 位级变化像素与 `seg` 对变化像素（`change_pixels_cam2()`，**判据口径**）；
- 全相机合计（`change_pixels_all_cameras()`）**只作诊断打印、不参与断言**（spec §3：cam0/cam1 帧间变化只诊断）；
  实测 cam0/cam1 变化为 0，cam2-only 与合计逐对相同：`000→001 = (1167, 1074)`、`001→002 = (1262, 1126)`；
- 判据 **各 ≥384**（spec §5：0.5% × 76,800）；**576 只是采集前置余量，不得当验收阈**；
- CPU 自证：`change_pixels_cam2(0)`、`change_pixels_cam2(1)` 与阈值常量 384 的静态断言。

## 5. 选项 A（三 context 审计；口径按 Chief 2026-11-05 裁决）

- **实现方式**：逐帧 state 的刚性位姿建 3 个 `RenderContext`（frame-k 的 `geom_xpos`/`geom_xmat` f32 位型
  直读为场景叶位姿；该帧刚性数组与 frame-000 位级相等，见 §3）；
- 断言三个 context 的**刚性签名一致**（叶几何 `geom_type`/`geom_id`/`geom_size` + 位姿 `xpos`/`xmat` 位型；
  CPU 前提 + GPU 用例各一次，把「模型偏移对 flex 渲染惰性」变成可审计证据）；
- 断言 `context.refits() == 0`（产品渲染上下文不 refit——「每渲染重建合并 BVH」的架构事实）；
- 对照：frame-001/002 用 `contexts[0]`（同一 context）渲染，整块多相机缓冲 sha 与各自 context 结果相同；
- **裁决留痕（Chief 2026-11-05）**：**不补 `body_pos` 偏移步骤**——产品渲染路径不消费 `body_pos`，补一步是演戏；
  口径以「逐帧 state 刚性位姿建 3 个 context + 刚性签名一致 + 同一 context 渲染对照 + `refits()==0`」为准
  （卡面 acceptance #4 已不可改写，裁决记录于小修卡 acceptance 与 §5 本段）；
- **spec §2 的实现映射**：模型 `body_pos` 偏移对 flex 渲染惰性（产品每次渲染用通道 positions 重建合并 BVH）。

## 6. 参考完整性 / provenance（CPU 自证）

- 12 个 npz sha256 对齐 `capture.json.files_sha256` + 3 个文本夹具 sha256 锚定记录值 = **15 文件**；
- spec `1d9fa5ce…`/10,506 B；三脚本 `authoring_script_sha256` 逐项；`refit_calls=6`；
  `capture_checks.refit.actual_sequence == [0,1,2,0,1,2]`、`ok=true`、`refit_policy=every-frame`、`refit_bvh=true`；
- capture 自身：`two_captures_bitwise_identical=true`、`legacy_reference_untouched=true`、overall `pass-margin`。

## 7. 冻结器与 guards 结构自检

```text
GUARDS_STRUCTURE_OK suite=render-r06-multiframe exes_pending=2 inputs=26 inputs_sha256_ok=true
aggregate_inputs_sha256=b8110cee547a6a6a12dbc4b8201f1571bebb31f7450e2d252f0a4015484afb7f classification=input=25,guard_script=1
```

26 个 inputs = 12 帧 npz + 3 多帧文本 + scene.xml/model-312/model-315/scene-312.mjb/scene-315.mjb/provenance.json
（7）+ spec.md + 3 authoring 脚本 + 冻结器自身（`guard_script`）；exes = `mjwarp-rs` bin + `render_flex_multiframe`
test（待验证窗真冻结）。本卡**只跑 `-GuardsOnly`**，不执行 freeze、不重冻任何既有 guards。

## 8. CPU 自检（零 GPU）

`cargo test --locked --features cuda-probe --test render_flex_multiframe` → **38 passed / 0 failed / 1 ignored**
（1 ignored = GPU 用例；其中本实体 22 项见 §3–§6 与 `logs/cpu-tests.log`）。

## 9. 边界与非声明

- 现有 `tests/render_flex_validate.rs` **498 行零 diff**（`artifacts/render_flex_validate-unchanged.txt`）；
- 四门禁全过（fmt/clippy/clippy-cuda/unit，unit **1359 过 0 败**，含本实体新增 38 项）；零 GPU、未开窗、未执行 freeze；
- 不改 `src/`、`fixtures/`、G-2 夹具、spec/判据/阈值；
- **非声明**：本实体只覆盖「该 3 帧集上的逐帧口径 + 跨帧可见性」；不写 R06 整族通过、不写动画通用性、
  不写与上游实现的等价结论；GPU 用例的真实结论留验证窗执行（token 由 Chief 发）。

## 10. 证据清单

| 路径 | 说明 |
| --- | --- |
| `README.md` | 本文件 |
| `receipt.json` | 机器可读回执（`/task_id` 供 normify `json` 检查） |
| `logs/cpu-tests.log` | CPU 自检原始输出（command + exit_code + 38 用例） |
| `logs/guards-structure.log` | `-GuardsOnly` 结构自检输出（GUARDS_STRUCTURE_OK） |
| `logs/line-counts.txt` | 各文件 git blob 口径行数（新文件 ≤500、validate 498 不变） |
| `artifacts/allowlist.diff` | `allowlist.rs` 唯一改动（新增 `assert_depth_strict` + dead_code 注记） |
| `artifacts/render_flex_validate-unchanged.txt` | 现有 498 行用例 unchanged 对照 |
| `artifacts/script-identity.json` | 三文件行数/bytes/sha256 |
| `manifest.txt` | 本目录 sha256 + bytes + 证据目录内相对路径（首行 `#` 表头，不自列） |

## 11. 复现

```powershell
# CPU 自检（零 GPU）
cargo test --locked --features cuda-probe --test render_flex_multiframe
# 结构自检（不跑 cargo、不冻结）
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate/render-bvh/freeze-guards-r06-multiframe.ps1 -GuardsOnly
# 验证窗（需 Chief token，不在本卡）
cargo test --locked --features cuda-probe --test render_flex_multiframe -- --ignored --test-threads=1
```
