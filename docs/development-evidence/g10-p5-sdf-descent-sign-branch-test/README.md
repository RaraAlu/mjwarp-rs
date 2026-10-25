# G3 `descent::compute_grad` sign 分支级反向判别测试（QA 观察 1）

- 任务：`2026-10-25-g10-p5-sdf-descent-sign-branch-test`（P5，纯 CPU，未开 GPU）
- 背景：`2026-10-25-g10-p5-sdf-primitives-sign`（`11696fc8`）把 `descent.rs::compute_grad` 的旧手写 sign（`>0→1, <0→-1, else→0`，即 `0.0`/`NaN → 0`）改为 `primitives::upstream_sign(max_val)`（上游 `wp.sign`：`0.0`/`-0.0`/`NaN → +1`）。QA 独立复核（`f187f701`）判 pass/0 阻断，但观察 1：该分支**没有直接反向判别用例**（当时 4 个新单测都在 `primitives.rs`，`descent.rs` 内无测试），只靠「G3 位级不变 ⇒ 未触发」间接说明。本片补分支级测试。
- 边界：**不改产品实现**（`sign` 仍调用 `primitives::upstream_sign`）、不改判据/容差/已 verified 数字/`Cargo.*`/fixtures/S2–S8；纯 CPU。

## 1. 改动

| 位置（`src/physics/collision/sdf/descent.rs`） | 内容 |
| --- | --- |
| L117–L129 | 扩写 `wp.sign` 注释：引 pinned `include/warp-headers/builtin.h:765`，并写明三条分支的**可达性与测试覆盖**（`+0.0`、`-0.0` 可构造并覆盖；`NaN` 不可构造为可断言场景，按任务口径记「不可构造、测试不增」） |
| L220 起 `#[cfg(test)] mod tests` | 3 个测试：`compute_grad_zero_max_val_uses_upstream_sign`（L261）、`compute_grad_negative_zero_max_val_uses_upstream_sign`（L306）、`nan_branch_is_not_constructible_but_helper_semantics_covered`（L346）；helper：`constant_volume`（单叶节点体积 SDF，盒内值恒定、梯度 0）、`old_three_branch_sign`（复刻旧分支，仅用于反向判别）、`identity` |

文件 356 行（≤500）；`cargo fmt --check` 已通过。

## 2. 测试构造与断言（为何这样测）

| 分支 | 构造 | 断言 |
| --- | --- | --- |
| `max_val == +0.0` | `shape1=Sphere(size=[1,0,0])`、`p1=[1,0,0]` ⇒ `sdf=0.0`；`shape2=Sdf` 带单叶体积（系数全 `-1.0`，盒内 `sdf=-1.0`）；`params.rel_mat=I、rel_pos=[1,0,0]` ⇒ 产品路径关系 `p1=rel_mat*p2+rel_pos` 成立 | `max_val=+0.0`；上游 `sign=+1` ⇒ `compute_grad == base + grad1_transformed == [2,0,0]`；旧三分支 `sign=0` ⇒ `[1,0,0]`（`assert_ne`） |
| `max_val == -0.0` | `shape1=Plane`、`p1=[0,0,-0.0]` ⇒ `sdf = p[2] = -0.0`；`shape2` 同上体积 `-1.0`；`max(-0.0,-1.0) = -0.0`；梯度有限（plane grad `[0,0,1]`、体梯度 `[0,0,0]`） | `a_.to_bits() == (-0.0).to_bits()`、`max_val.to_bits() == (-0.0).to_bits()`；上游 `sign=+1` ⇒ `[0,0,2]`；旧分支 ⇒ `[0,0,1]`（`assert_ne`） |
| `max_val == NaN` | **不可构造为可断言场景**：`f32::max` 单侧 NaN 会返回另一侧有限值；要 `max` 返回 NaN 需两侧 SDF 都是 NaN，而体 SDF 在 NaN 点用 NaN 权重算值/梯度，梯度同为 NaN，增量无方向可断言；NaN 输入是非物理的，合法有限几何模型不可达 | 不增测试；按任务口径记「不可构造、测试不增」；分支语义由 `primitives::upstream_sign(NaN)=+1` 单测覆盖（本测试内也直接断言该 helper 与旧分支行为） |

## 3. 反向判别（退回旧三分支 ⇒ 本片测试必失败）

`pre_fix_reverse_check.ps1`：把 `descent.rs` 的 `let sign = primitives::upstream_sign(max_val);` **临时替换**为旧三分支 → 跑两个分支测试（预期 FAIL）→ **finally 还原源码并校验 sha256** → 复跑（预期 PASS）。真实输出见 `pre-fix-reverse-output.txt`：

- 旧分支：`old_branch_exit_code=101`，`old_branch_failed_tests=2`（两个分支测试都 FAILED）；
- 还原：`source_sha256_after == source_sha256_before`（`restored_ok=True`）；
- 复跑：`restored_passed_tests=2`、exit 0；`REVERSE_OK old_branch_fails_2/2 restored_passes_2/2`。

## 4. 不回归（真实命令与数字）

| 命令 | 结果 |
| --- | --- |
| `cargo test --locked --lib physics::collision::sdf::tests::sdf_box_host_matches_upstream_f32_within_4ulp -- --nocapture` | `sdf-box count 80/80 | dist 位等 80/80 max 0 ULP | pos max 0.00 ULP@尺度 | frame 位等分量 720/720 max abs 0e0` |
| `cargo test --locked --test g10_p5_sdf_relmat -- --nocapture` | **4 passed**；`rel_mat_warp aᵀb 位级 10/10 world`（8 world 可区分 bᵀa）；`sdf-box(G3 回归)` 与 `sdf-rot-ident/sdf/sdf-box/both/rz-rx` 均 **80/80 contacts bit-equal**，dist/pos/frame max_abs 0e0 |
| `cargo test --locked --lib physics::collision::gjk::case_evidence -- --nocapture` | **3 passed / 2 ignored**；G2 五 case 数字与已验收一致（cylinder-box 8/8 全位级；ellipsoid-box dist 2/2、pos 1/2 max 2.00 ULP@尺度、frame 13/18 max 3.838e-7；mesh-mesh dist 1/2 max 2 ULP；convex-mesh 2/2 全位级；degenerate-mesh dist 0/2 max 1 ULP） |
| `cargo test --locked --lib physics::collision::hfield::tests -- --nocapture` | **3 passed / 1 ignored**；hfield-box 8/8、hfield-sphere 8/8、hfield-capsule 7/7，dist 0 ULP、pos 0.00 ULP@尺度、frame 72/72·72/72·63/63 max abs 0e0 |
| `cargo test --locked --lib` | **447 passed / 0 failed / 5 ignored**（改前 444/5；本片 +3 测试） |
| `normify gate 2026-10-25-g10-p5-sdf-descent-sign-branch-test` | fmt/clippy/clippy-cuda/unit 全绿（unit **595 过 0 败** = 原 592 + 本片 3） |

## 5. 结论

- QA 观察 1 已闭环：`0.0`/`-0.0` 分支有**产品路径可达**的分支级反向判别测试（旧实现下 2/2 失败、现实现 2/2 通过）；
- `NaN` 按任务要求如实记录「不可构造、测试不增」并在代码注释给出依据，未伪造用例；
- 无产品实现变更、无已 verified 数字/判据变更；G3/G2/G4 数字逐位不变。

## 6. 未覆盖 / 边界

- 未覆盖 `NaN` 的 `compute_grad` 级用例（不可构造，原因见 §2/注释）；其 sign 语义由 `primitives::upstream_sign` 单测覆盖。
- 本片不表示精度/等价/放行结论；不覆盖设备侧路径（设备 EPA 的同类语义已在 G5 修正）。

## 7. 文件

| 文件 | 内容 |
| --- | --- |
| `README.md` | 本文件 |
| `receipt.json` | 任务回执（task_id、改动、测试/反向判别/回归、gate） |
| `pre-fix-reverse-output.txt` | 旧分支失败 2/2、还原 sha256 校验、复跑 2/2 通过的原始输出 |
| `pre_fix_reverse_check.ps1` | 反向判别脚本（临时改实现→跑测试→finally 还原并校验 sha256） |
- 源码改动：`src/physics/collision/sdf/descent.rs`（注释 + 测试模块；实现未改）。
