# U4 SDF 参考验证目标实现 独立复核（QA，只读）

任务：`2026-10-31-render-sdf-reference-target-check`
被核：`2026-10-31-render-sdf-reference-target`（实现片，主卡 verified）
被核 pin：**`4e761a7b`**；核 start 时 `revision.before = e83ed63b`。
**结论：pass / 阻断 0 / 需修 2（轻微：证据数字 / 判据覆盖）/ 观察 6。**
本卡只读：零 GPU、零窗口、零设备执行；四门禁 + `--no-run` + host 用例 + 冻结器静态检查为自跑（host 用例只读 JSON，不建 session）。

## 0. 一句话

新 target 结构正确、判据零放宽、host 用例把 U4a dump 语义与「SDF leaf 用 mesh bounds」钉住了，8 个用例（host 3 / GPU 5 ignored）与 `--list`/host 运行一致；冻结器 167 行、ASCII 0、PSParser 0 错、硬断言 9/83/92，且**我用它的选取规则在现盘重数 inputs 恰好 10/10/47/16=83**，本片未执行冻结（无 guards 文件）。两处需修：receipt 的 unit ignored 数写了旧值 392（实际 397）；shadow 臂「两臂两跑 sha 相等」在 off 臂缺 seg。

## 1. 复核方法

- 代码/身份/manifest 由 `git cat-file` 读 committed blob；判据逐条从 `sdf_compare.rs` 断言源码核对。
- 门禁自跑：fmt/clippy/clippy-cuda/unit/no-run + `cargo test --locked --features cuda-probe --test render_sdf_reference`（host 3 passed / 5 ignored）+ 冻结器 PSParser。
- 冻结输入计数：按 `freeze-guards-sdf.ps1` 同样规则在 `fixtures/render-bvh` 现盘统计。
- 重算脚本：`recompute.py`（10 项 check → `recompute.json`，problems 2）。

## 2. 结果（10 项 check，8 过 / 2 需修）

| # | check | 结论 | 关键实测 |
| --- | --- | --- | --- |
| 1 | `scope.pin_commit` | ✓ | 23 路径：5 个新文件（`tests/render_sdf_reference.rs` + `sdf.rs`/`sdf_scene.rs`/`sdf_compare.rs` + `freeze-guards-sdf.ps1`）+ 证据 17 + 卡 1；无 `src/`、`.cu`、`fixtures/`、`Cargo`、既有 tests 改动；**guards 文件未生成**（本片不冻结） |
| 2 | `files.line_counts` | ✓ | `render_sdf_reference.rs` **361**、`sdf.rs` **372**、`sdf_scene.rs` **323**、`sdf_compare.rs` **260**、冻结器 **167**；全部 ≤500；材料预算 260–360 被超 1 行（观察 O1） |
| 3 | `cases.eight_with_three_host` | ✓ | 8 用例：host 3（`model_dumps_match_signed_scene_semantics` / `capture_references_parse_and_cross_checks_hold` / `sdf_leaf_geometry_uses_mesh_bounds`）+ GPU 5（两组参考比对 / shadow / DepthSeg 跨路径 / 两跑确定性）；我自跑 host = **3 passed / 5 ignored**，与证据一致 |
| 4 | `criteria.zero_relaxation` | ✓ | `assert_group_criteria`：RGB `beyond_1_lsb=0`、depth `mismatched=0` + `max_ulp<=MAX_DEPTH_ULP(4)` + `beyond_ulp=0`、seg `mismatched=0`；shadow 臂 depth/seg 位级相等 + reversed==0 + changed>0；DepthSeg 跨路径 `0..ncam` 位级；两跑 rgb/depth/seg sha |
| 5 | `shadow.both_arms_two_run_sha` | ✗（P2） | on 臂断言 rgb/depth/seg 两跑 sha；**off 臂只有 rgb/depth，缺 seg** —— 与判据「both arms two-run sha equal」不完全一致 |
| 6 | `comments.no_stale_cu_reference` | ✓ | 5 个新文件均**不含** U1b 已过时的 `.cu` 注释语义（`host 仍拒绝` / `今天不可达` / `reject_sdf_geometry` / 旧错误串 0 命中） |
| 7 | `freezer.static_and_counts` | ✓ | 冻结器 167 行 / 非 ASCII 0 / PSParser 0 错；硬断言 9/83/92 三处；9 实体顺序与材料一致；**本片未执行**（`guards-sdf-validate.json` 不存在）；现盘按同规则重数 inputs = scene-sdf 10 + scene-hfield 10 + legacy 47 + scene-shadow 16 = **83** ✓ |
| 8 | `gates.self_run_and_receipt_number` | ✗（P1） | 我自跑：**137 result 行、1109 passed / 0 failed / 397 ignored**；六个门禁全 exit 0、`--no-run` 末行 `EXIT=0` 含 `Finished`；host 3/5 ✓。**但 receipt.gates.unit 文案写 392 ignored**（U1b 旧值；证据 `gate-unit.json` 是 397） |
| 9 | `evidence.manifest` | ✓ | manifest **12/12**、13 文件、无 UTF-16/BOM |
| 10 | `card.checks_and_validate` | ✓ | 主卡 `normify check` 通过；`validate --fingerprints` = 0 错误 / 5 警告 |

参考加载器（`sdf.rs`）我另核了硬断言范围：`capture.json` 的 task_id/kind/group/`two_captures_bitwise_identical`/`legacy_reference_untouched`/legacy 69/`use_fast_math=false`/`use_shadows=true`/`repeat=2`/`recipe.scene_xml.sha256`/`recipe.script.sha256`（U3 收盘 `80645968…`）/npz 清单/`samples_per_pixel=1`，以及每个 npz 文件 sha 对 `files_sha256` —— 参考被动过会直接失败。

## 3. 需修 2（轻微，不跑 GPU）

1. **P1 receipt 的 unit ignored 数错**：`receipt.gates.unit` 写「1109 passed / 0 failed / **392 ignored**」，实际是 **397**（U1b 后 392 + 本片 5 条新 ignored GPU 用例）；证据 `logs/gate-unit.txt` 与 `.normify/runs/.../unit.json` 都是 397。改 receipt 文案即可，不重跑。
2. **P2 shadow off 臂缺 seg 两跑 sha**：判据写「both arms two-run sha equal」，`shadow_toggle_consistency` 的 off 臂只断言 rgb/depth。建议补 1 行 seg 断言（改后行数 361→362，仍 ≤500；host/unit 门禁重跑一次，期望仍 1109/0/397）；或把判据措辞收窄为「shadow 两臂 rgb/depth 两跑 sha 相等」。

## 4. 观察 6（记录级接受）

- **O1 材料行预算**：材料写 `render_sdf_reference.rs` 预算 260–360，实际 **361**（超 1 行）；500 行棘轮满足。若按 P2 补断言会到 362，建议把材料预算改成「≤400」以免每次都踩线解释。
- **O2 场景装配与 U3 配方对齐**：`capture_params` 用 `ContextParams::default()` + 显式 `use_fast_math=false`/`use_shadows`/几何组 0..3 + 三输出 true；我对照 `params.rs` 默认值，与 U3 PARAMS_28 一致（spp=1、precomputed rays、backface culling、vertex normals、shadows fraction 0.3、background 0/0/0/1）——比对配方一致性有据。
- **O3 SDF leaf 规则**：`sdf_scene.rs` 对 `geom_type==8` 用 `mesh_half_extents` 作为 leaf size（断言 `geom_dataid==0`）、plane/sphere/box 用 `geom_size`；host 用例再断言 `LeafGeometry::from_geom_type(8, half).kind == Mesh` 且 `size == half` —— 与 U4a dump 的 `geom_size=0.35` 与 mesh bounds ±0.35 并存相呼应。
- **O4 host 用例是「读 JSON + 纯算术」**：3 条 host 用例不建 CUDA session（我自跑 0.12s 完成），GPU 5 条全部 `#[ignore]` 带明确理由；窗口 `--ignored --test-threads=1` 才会执行它们。
- **O5 冻结器行为**：`cargo test --no-run --message-format=json` 解析 exe 名 → 9 实体映射 → stash 到 `target/render-bvh/sdf-validate-window/frozen-exes` → 文件/输入 sha → 硬断言计数 → 聚合 = 92 条按 path 排序后 sha 串接再 sha256（与 run-window 同语义）；fail-closed（任一实体缺失/计数不符即 throw）。
- **O6 无产品源码改动**：本片只加测试/脚本/证据，`git diff PIN^..PIN -- src` 为空——不影响内核身份与 U4 窗口 exe 之外的行为。

## 5. 边界与不宣称

- 本卡只读：零 GPU、零窗口、零设备执行；GPU 用例仅静态审阅，未运行。
- 不宣称渲染数值验收通过、不宣称上游等价、不宣称 SDF 渲染已覆盖——判据是否真正满足只能由 U4 窗口证据给出。
- 冻结器本片未执行，92 条 guards 身份与 aggregate 需在窗口内生成并另核。

## 6. 限制

- GPU 5 条用例只能静态审阅（是否通过要等 U4 窗口）；本卡不能证明 SDF 渲染与参考一致。
- 材料预算/措辞问题与本片代码无冲突（O1）。
