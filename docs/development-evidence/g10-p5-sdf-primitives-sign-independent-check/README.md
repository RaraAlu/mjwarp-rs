# G3 SDF 原语 sign 语义修正的独立复核（`wp.sign`：`x < 0 ? -1 : 1`）

- 任务：`2026-10-25-g10-p5-sdf-primitives-sign-independent-check`（p60，纯 CPU）
- 复核对象：修正片 `11696fc8`（`primitives.rs` +77、`descent.rs` 12 行改动、+4 单测）、G3 加注 `8f25e883`（仅 normify 记录）；HEAD `8f25e883`
- 判定：**pass（阻断 0、发现 0、观察 1）**
- 独立性：上游语义自己读 pinned 头/上游源核对；反向判别用**自写最小 Rust 复现**（未复用 Backend 的 `pre_fix_reverse_check.ps1`）；回归测试自己跑（独立 `CARGO_TARGET_DIR`）。

## 1. 上游语义（独立核对，非引述）

| 项 | 独立核对结果 |
| --- | --- |
| sign 定义 | `include/warp-headers/builtin.h:765`：`inline CUDA_CALLABLE T sign(T x) { return x < T(0) ? -1 : 1; }` → `-0.0`/`0.0`/`NaN` 均 **+1**（`NaN < 0` 为 false） |
| 上游源 provenance | `target/mjwarp-parallel/upstream` 的 HEAD = `71da24d956378a87a703b6e1442b13aec0c4ac29`（pinned）；`docs/evidence/d04-complex-physics/authoring-runs/run-5/frozen-source/_src/collision_sdf.py` 与该 pinned 树的同名文件 **sha256 逐字节相同**（`FE51F981…96D02`） |
| radial_field | `collision_sdf.py` L148–155：`field[i] *= wp.sign(x[i])`（3 个分量，L152–154） |
| grad_cylinder | L275：`w = wp.vec3(0.0, 0.0, wp.sign(p[2]))` |
| compute_grad | L624–630：`max_val = wp.max(A, B)`；`sign = wp.sign(max_val)`；`gradient += max_grad * sign` |

## 2. 产品实现（逐分支核对）

- `primitives.rs`：`pub(crate) fn upstream_sign(x: f32) -> f32 { if x < 0.0 { -1.0 } else { 1.0 } }` —— 与 `builtin.h:765` 逐字同构（`-0.0`/`NaN` 落到 `+1`）。
- 调用点齐全：`radial_field` 3 处（`upstream_sign(x[0..2])`）、`grad_cylinder` 1 处（`[0.0, 0.0, upstream_sign(p[2])]`）、`descent.rs::compute_grad` 1 处（`sign = primitives::upstream_sign(max_val)`）——与上游 5 个调用点一一对应，**全链只有一份 sign 实现**。
- 产品 `src/` 内**无残留活跃 `signum()`**（扫描时排除 `#[cfg(test)]` 测试块；剩余 `signum` 只出现在新加的反向判别测试里，是刻意保留的对照）。
- 顺带核对 `gjk/math.rs::sign3`（GJK 的 `wp.sign(vec3)`）：`if a[i] < 0.0 { -1.0 } else { 1.0 }` —— 与上游同语义（`support.rs` 使用）；上游另有 `collision_gjk.py` 2 处、`collision_primitive_core.py` 2 处 `wp.sign`，primitive core 属未实装路径（G5 未覆盖），不在本片范围。

## 3. 反向判别力（自写最小复现，不复用 Backend 结论）

从提交 `23661557` 取改前代码：`primitives.rs` 有 **4 处 `f32::signum()`**（radial_field 3 + grad_cylinder 1）；`descent.rs::compute_grad` 是手写三分支（`>0→1, <0→-1, else→0`，即 0/NaN→0）。自写 `qa_sign_repro.rs` 用 rustc 编译后逐输入比较「改前 / 改后 / 上游期望」：

| 输入 | 改前 `signum()` | 改后/上游 | 改前是否错 |
| --- | --- | --- | --- |
| `-0.0` | `-1`（0xbf800000） | `+1`（0x3f800000） | **错** |
| `0.0` | `+1` | `+1` | 对 |
| `NaN` | `NaN`（0x7fc00000） | `+1` | **错** |
| `-1.5` / `2.5` / `±inf` | `-1` / `+1` / `∓1` | 同 | 对 |

- `compute_grad` 的改前三分支：`max_val ∈ {0.0, -0.0, NaN}` 时 `sign=0`（upstream 应为 `+1`）；实测 gradient 由 `[1,2,3]`（旧）变为 `[11,22,33]`（新，`max_grad=[10,20,30]`），`±5.0` 两侧一致。
- `radial_field` 形态：`field=[0.25,0.5,0.75]`、`x=[-0.0,-0.0,-0.0]` → 旧 `[-0.25,-0.5,-0.75]` vs 新 `[+0.25,+0.5,+0.75]`（符号翻转）；`x[0]=NaN` → 旧分量 NaN vs 新 `+0.25`。
- 复现判定：**改前恰好只在 `-0.0`/`NaN`（primitives）与 `0`/`-0.0`/`NaN`（compute_grad 分支）与上游不一致，改后 7 个输入全部与上游一致**（`old_fails_exactly_expected=true`、`new_matches_all=true`）；两条产品路径的差异都被最小复现直接证明（不依赖 Backend 的脚本与结论）。

## 4. 回归独立复跑（数字不改，只核对）

| 命令 | 结果 | 与既有 verified 证据 |
| --- | --- | --- |
| `cargo test --locked --test g10_p5_sdf_relmat` | **4 passed / 0 failed**；测试内断言每场景 `total==80` 且 `bit_equal==total`（旋转 4 场景 dist/pos/frame `max==0.0`；`sdf-box` 回归 80/80 位级） | 与 G3 `sdf-box` 80/80 位级及 4 旋转 80/80 位级一致 |
| `cargo test --locked --lib physics::collision::gjk::case_evidence` | **3 passed / 2 ignored**（含 `upstream_f32_within_4ulp`、`our_contacts_match_upstream_f32_within_4ulp`） | 五 case ≤4 ULP 判据未变 |
| `cargo test --locked --lib physics::collision::hfield::tests` | **3 passed / 1 ignored**（含 8/8、8/8、7/7 位级断言） | G4 结论未变 |
| `cargo test --locked --lib physics::collision::sdf::primitives` | **4 passed / 0 failed**（本片新增 4 个单测全过） | 与修正片声明 +4 单测一致 |
| 修正片 `unit` 门禁记录 | 592 passed / 0 failed / 7 ignored | 与 Arch 转述 592/0 一致；本任务 gate 复跑同口径 |

- 已 verified 数字未改：`git diff 23661557..HEAD -- docs/development-evidence/g10-p5-sdf-host{,-r2}` 与 `g10-p5-sdf-relmat-fix` **均为空**；G3 加注提交 `8f25e883` 只改 `.normify` 记录（5 个文件全在 `.normify/` 下），G3 结论不变。
- 修正片范围：产品只改 `src/physics/collision/sdf/primitives.rs`、`src/physics/collision/sdf/descent.rs`；其余为 `.normify` 记录与本片证据目录；无 `Cargo.*`/`rust-toolchain.toml`/`fixtures/mjwarp-complex-physics`/S2–S8 改动。

## 5. 观察（1 项，非阻断）

**`descent.rs::compute_grad` 的旧分支没有直接反向判别用例**：`descent.rs` 内 `#[test]` 数量为 **0**，本片新增的 4 个单测都在 `primitives.rs`（`upstream_sign` 语义 / 改前 `signum` 对照 / `radial_field` / `grad_cylinder`）。`compute_grad` 的 `0/NaN→0` 旧分支目前只能靠「G3 sdf-box 位级不变 ⇒ 该分支未被触发」间接说明未回归，缺少像 `grad_cylinder` 那样的直接断言（例如构造 `max_val==0.0`/`NaN` 的输入并断言 `gradient += max_grad`）。实现本身已核对正确（`upstream_sign(max_val)`），复现也证明旧分支确与上游不一致；建议后续小片补一条 `compute_grad` 分支级反向测试，或在注释里说明该分支在产品路径中不可达/由何覆盖。

## 6. 方法与复现

```powershell
cd D:\Rust\mjwarp-rs\docs\development-evidence\g10-p5-sdf-primitives-sign-independent-check
python qa_sign_check.py     # 上游语义/产品实现/改前代码/最小复现/范围 → qa-sign-check.json
```

`qa_sign_check.py` 内部：读 `builtin.h` 与 pinned 上游树 → 校验 frozen-source 逐字节一致 → 解析产品 `upstream_sign` 与 5 个调用点、扫描活跃 `signum` → `git show 23661557:…` 取改前代码 → `rustc -O qa_sign_repro.rs` 编译并运行最小复现 → 核对修正片/加注提交范围与 G3 已 verified 证据 diff → 输出 JSON。

## 7. 提交路径与不入库

入库（`docs/development-evidence/g10-p5-sdf-primitives-sign-independent-check/`）：

```
README.md  receipt.json  artifacts-sha256.json  qa_artifacts_sha256.py
qa_sign_check.py  qa_sign_repro.rs  qa-sign-check.json  qa-sign-check.log
qa-test-relmat.log  qa-test-case-evidence.log  qa-test-hfield.log  qa-test-primitives.log
```

不入库：本目录无 `.bin`/`.pyc`/`.o`/warp-cache；`qa_sign_repro.exe` 在 `target/qa-sdf-sign/`（构建产物，可由脚本重建）。

## 8. 结论

- **阻断 0 / 发现 0**：上游 `wp.sign` 语义（含 `-0.0`/`NaN` → +1）经 pinned 头与上游源独立确认；产品 `upstream_sign` 与 5 个调用点逐一对应、全链单份实现、无残留 `signum`；改前 `signum()`/三分支与上游的差异被自写最小复现精确复现（`-0.0`/`NaN`/`0`），改后 7 输入全对齐。
- **回归**：relmat 4 passed（4 旋转 + sdf-box 均 80/80 位级断言）、case_evidence 3/2、hfield 3/1、primitives 4 passed；已 verified 数字与证据零改动。
- **观察 1**：`compute_grad` 的 0/NaN 分支缺直接反向用例（实现已核对正确、复现已证旧分支错误），建议后续补一条分支级测试。
