# G3 SDF 原语 sign 语义对齐（host `wp.sign`，`-0.0`/NaN → +1）

- 任务：`2026-10-25-g10-p5-sdf-primitives-sign`（P5 G3 修正片）
- 来源：G5 同类清扫发现 host `sdf/primitives.rs` 用 `f32::signum()`（Rust：`-0.0 → -1`、`NaN → NaN`），与冻结上游 `wp.sign`（`-0.0`/`0.0`/`NaN → +1`）语义不一致；G3 已验收的 `sdf-box` 80/80 未触发该分支，**G3 数值结论不变**，本片只对齐产品语义。
- 边界：不改已 verified 证据数字、不改判据/容差、不改 `Cargo.*`、`fixtures/mjwarp-complex-physics/*.xml`、S2–S8 证据；纯 CPU、**未开 GPU 窗口**；不宣称精度达标/上游完全等价/原生放行。

## 1. 缺陷与上游依据

| 位置（改前） | 改前语义 | 上游语义 |
| --- | --- | --- |
| `src/physics/collision/sdf/primitives.rs` `radial_field`（旧 L100-102，`x[i].signum()`） | `-0.0 → -1`、`NaN → NaN`、`0.0 → +1` | `wp.sign`：`-0.0`/`0.0`/`NaN → +1`、负数 → `-1` |
| 同文件 `grad_cylinder`（旧 L187，`p[2].signum()`） | 同上 | 同上 |
| `src/physics/collision/sdf/descent.rs::compute_grad`（旧 L117-125，手写 `>0→1,<0→-1,else→0`） | `0.0`/`NaN → 0` | 同上（`0.0`/`NaN → +1`） |

上游原文：

- pinned warp `include/warp-headers/builtin.h:765`：`template <typename T> inline CUDA_CALLABLE T sign(T x) { return x < T(0) ? -1 : 1; }`（浮点 `-0.0 < 0` 为假 → +1；NaN 比较为假 → +1）。
- 上游 `mujoco_warp/_src/collision_sdf.py`：`radial_field` 用 `wp.sign(x[i])`（L152-154）；`grad_cylinder` 用 `wp.vec3(0.0, 0.0, wp.sign(p[2]))`（L275）；`compute_grad` 用 `sign = wp.sign(max_val); gradient += max_grad * sign`（L629-630）。
- host `gjk/math.rs::sign3` 已是 `x < 0.0 ? -1 : 1` 语义，本片把 SDF 侧统一到同一定义。

## 2. 修改（行号按改后）

| 文件 | 改动 |
| --- | --- |
| `src/physics/collision/sdf/primitives.rs` | 新增 `pub(crate) fn upstream_sign`（L102-104，唯一 sign 实现，注释引用 builtin.h:765 与 `gjk/math.rs::sign3`）；`radial_field` 三处改用（L110-112）；`grad_cylinder` 的 `w` 改用（L197）；新增 `#[cfg(test)] mod tests`（L223 起，4 个测试） |
| `src/physics/collision/sdf/descent.rs` | `compute_grad` 的手写 sign 改为 `primitives::upstream_sign(max_val)`（L117-120）；旧注释「零 0」纠正为上游语义 |

语义对照（float）：

| 输入 | 改前 `f32::signum()`/手写 | 改后 `upstream_sign`（=上游） |
| --- | --- | --- |
| `-1.5` / `-inf` | `-1` / `-1` | `-1` / `-1` |
| `0.0` | `+1`（descent 处为 `0`） | `+1` |
| `-0.0` | `-1`（descent 处为 `0`） | **`+1`** |
| `NaN` | `NaN`（descent 处为 `0`） | **`+1`** |
| `+2.5` / `+inf` | `+1` / `+1` | `+1` / `+1` |

`descent.rs` 是同一次清扫发现的同类缺陷（零/NaN 分支与上游不符），一并修正；若它改变任何数值则按任务停下条件报告——**实测回归全部位级不变**（见 §4），故保留。

## 3. 反向判别力（改前会失败，改后通过）

- 新增单测（`primitives.rs` 内，4 个）：`upstream_sign_matches_warp_sign_semantics`、`pre_fix_signum_behaviour_differs_from_upstream`、`radial_field_uses_upstream_sign_for_negative_zero_and_nan`、`grad_cylinder_uses_upstream_sign_for_negative_zero`。改后 `cargo test --locked --lib physics::collision::sdf::primitives::tests` → **4 passed / 0 failed**。
- 改前源复算：`pre_fix_reverse_check.ps1` 用改前提交 `23661557` 的 `primitives.rs`（blob `34f7d1dd…`，嵌入文本 sha256 `be0ed554…`）原样嵌入临时 crate，复算本片新增的产品路径断言：**3 条上游语义断言 FAIL（`-0.0`/`NaN` 的 radial_field 与 `-0.0` 的 grad_cylinder）+ 1 条负值对照 PASS** → 改后同断言在 `cargo test` 全 PASS，判别力成立。输出见 `pre-fix-check-output.txt`（含 `REVERSE_OK failures=3/3`）。

## 4. 不回归（真实命令与数字）

| 命令 | 结果 |
| --- | --- |
| `cargo test --locked --lib physics::collision::sdf::tests::sdf_box_host_matches_upstream_f32_within_4ulp -- --nocapture` | `[SDF判据] sdf-box count 80/80 | dist 位等 80/80 max 0 ULP | pos max 0.00 ULP@尺度 | frame 位等分量 720/720 max abs 0e0` |
| `cargo test --locked --test g10_p5_sdf_relmat -- --nocapture` | **4 passed**；`sdf-box(G3 回归)` 与 `sdf-rot-ident`/`sdf-rot-sdf`/`sdf-rot-box`/`sdf-rot-both`/`sdf-rot-rz-rx` 均 **80/80 contacts bit-equal**，`dist/pos/frame max_abs 0e0` |
| `cargo test --locked --lib physics::collision::gjk::case_evidence -- --nocapture` | **3 passed / 2 ignored**；G2 五 case 仍满足 ≤4 ULP（`upstream_f32_within_4ulp` ok；ellipsoid-box pos max 2.00 ULP@尺度、frame max 3.838112e-7；mesh-mesh dist 2 ULP；degenerate-mesh dist 1 ULP —— 与 G2 已验收数字一致） |
| `cargo test --locked --lib physics::collision::hfield::tests -- --nocapture` | **3 passed / 1 ignored**；G4 `hfield-box` 8/8、`hfield-sphere` 8/8、`hfield-capsule` 7/7 全字段位级，dist 0 ULP、pos 0.00 ULP@尺度、frame 72/72·72/72·63/63，max abs 0e0 |
| `cargo test --locked --lib` | **444 passed / 0 failed / 5 ignored**（改前 440 passed/5 ignored；本片 +4 个非 ignored 新语义测试） |
| `normify gate 2026-10-25-g10-p5-sdf-primitives-sign` | fmt/clippy/clippy-cuda/unit 全绿（unit 592 过 0 败：G6 基线 588 + 本片 4） |

## 5. G3 加注（不改 G3 既有数值结论）

`normify fix 2026-10-25-g10-p5-sdf-host-r2 --reason "..."` 追加 addenda：写明「`sdf-box` 80/80 未触发 `-0.0`/`NaN` 分支，G3 数值结论不变；host 原语 sign 语义已由 `2026-10-25-g10-p5-sdf-primitives-sign` 修正为上游 `wp.sign`」，指向本片证据；G3 既有判据/数字均未改。

## 6. 未改 / 未覆盖

- 未改：G2–G5 已 verified 证据数字、已批准判据与容差、`Cargo.*`、`fixtures/mjwarp-complex-physics/*.xml`、S2–S8 证据；未新增/改动任何 GPU 路径（本片不开 GPU 窗口）。
- 未覆盖：设备侧 `cuda/common.cu::v3_sign` 与 `cuda/sdf_math.cu` 的同类修正已在 G5 `01690e53` 完成，本片不重复；`wp.sign` 反向传播（`adj_sign` 为空）不影响前向语义，不在本片范围。
- 声明边界：本片只对齐 sign 语义；不表示接触字段精度已达标、不表示上游完全等价、不提供原生放行信号。

## 7. 文件

| 文件 | 内容 |
| --- | --- |
| `README.md` | 本文件（缺陷、上游依据、修改、判别力、回归、加注、未覆盖） |
| `receipt.json` | 任务回执（task_id、改动、测试与回归数字、gate） |
| `pre-fix-check-output.txt` | 改前源复算输出（3 FAIL + 1 PASS + `REVERSE_OK`） |
| `pre_fix_reverse_check.ps1` | 从改前提交 `23661557` 取源、嵌入、编译并复算的脚本（authoring-only） |
- 源码改动：`src/physics/collision/sdf/primitives.rs`、`src/physics/collision/sdf/descent.rs`。
