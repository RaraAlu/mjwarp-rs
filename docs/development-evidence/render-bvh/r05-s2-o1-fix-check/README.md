# R05-S2 QA O1 修复轻复核（三角化阈值改全 f32 + 证据 O2/O3/O5）

任务：`2026-10-28-render-bvh-r05-s2-o1-fix-check`（P8 渲染与 BVH；p57；owner QA）
被复核提交：`1b876391` 修复(渲染)（HEAD；14 文件 +468/−387；`Normify-Fixes: …-r05-s2-hfield-intersect-r2`）
复核时 HEAD：`1b876391`（树冻结：Backend 不写不提交）
范围：只读轻复核 + 本证据目录；**不跑全量门禁、不开 GPU 窗、不做渲染数值结论**。

## 1. 结论

**verdict：pass / 阻断 0 / 观察 1**（观察为源码注释控制字符，建议级，不影响本片验收）。

O1 修复的 4 个点全部成立，O2/O3/O5 处置与要求一致：

| 复核点 | 结果 |
| --- | --- |
| 1. 阈值全 f32 | `let threshold = 1.0e-5f32;`；`planar` 用 `diff.abs() < threshold`；`fits_plane` 的预测/斜率判据用 `>= threshold`，**全部 f32**，无 `1.0e-5f64`/`f64::from(diff.abs())` 残留 |
| 2. 新增 4 条单测 | 逐条读 + 定向跑：钉住位型/顶点数/索引数/完整索引向量/点坐标，非 `is_ok()`；`cargo test --test render_hfield_mesh` **16 passed / 0 failed** |
| 3. README §2.1 | 旧「阈值不落在相邻 f32 之间 ⇒ 与 f64 比较等价」**已被明确否证并改写**为 f32 口径（含 `f32(1e-5)=9.999999747378752e-06`、边界 6V/12I 与 4V/6I）；`hfield_mesh.rs` 模块文档同步 |
| 4. 零 CUDA 改动 | `1b876391` 14 文件**无 `.cu`**；我顺带跑内核门：`72bc364c…` / `97467d83…` / **264015**，与 O1 修复前一致 |

## 2. 四点核验（我自己算/自己跑）

### 2.1 阈值全 f32 口径

- `src/render/bvh/hfield_mesh.rs`：`let threshold = 1.0e-5f32;`（L155）；`planar` = `(d00+d11)-(d01+d10)` 的 f32 差值 `diff.abs() < threshold`（L158）；`fits_plane` 的 `(cz00 - z_pred).abs() >= threshold`（L200）与 `(cslope_x - slope_x).abs() >= threshold || (cslope_y - slope_y).abs() >= threshold`（L205-206）。
- 阈值位型自证：`f32(1e-5)` = **`0x3727C5AC`** = **9.999999747378752e-06**，`f64::from(f32(1e-5)) < 1e-5` 为真（我算的）。即上游 numpy 2.5.4/NEP 50 的 `f32 vs f32(1e-5)` 语义与实现一致。
- **原 2/16 分歧点复现（我的探针）**：2×3、差值恰为 `f32(1e-5)` 的两例（planar 判据与 `fits_plane` 斜率判据各一）现在都是 **6 顶点/12 索引**（= 上游「非共面/不合并」行为）；低 1 ULP（`0x3727C5AB`）→ **4 顶点/6 索引**（合并）。修复前两例分别是 4/6 与 4/6（错误方向），已回到上游口径。

### 2.2 新增 4 条单测强度（`tests/render_hfield_mesh.rs`）

| 测试 | 我核到的可证伪量 |
| --- | --- |
| `threshold_constant_matches_upstream_f32_rounding` | `assert_eq!(1.0e-5f32.to_bits(), 0x3727_C5AC)` + `f64::from(1.0e-5f32) < 1.0e-5` |
| `diff_exactly_f32_threshold_is_non_planar_like_upstream` | 2×3 → **6 顶点/12 索引**、前 4 点精确坐标（含 `t*0.5` z）、**完整索引向量** `[0,1,3,0,3,2,1,4,5,1,5,3]` |
| `diff_just_below_f32_threshold_is_planar_and_merges` | 低 1 ULP → **4 顶点/6 索引** + 索引向量 `[0,1,3,0,3,2]` |
| `triangulation_is_deterministic_for_same_input` | 3×3 输入两次三角化结果完全相等（同输入确定性） |

定向跑（我执行）：`cargo test --locked --test render_hfield_mesh` → **16 passed / 0 failed**（exit 0）；4 条新测试均 `ok`。

### 2.3 README §2.1 与模块文档

- `docs/development-evidence/render-bvh/r05-s2-hfield-intersect/README.md` §2.1：旧等价结论被标为「**不成立**」，改为 f32 阈值口径，并写入两条边界单测的期望（6V/12I、4V/6I）；§8 计数改 **1086**；§9 限制项改为「f32 结果 × f32 阈值」。
- `hfield_mesh.rs` 模块文档（L15-18）同步：阈值先转 f32、`f32(1e-5) < 1e-5`、差值恰为阈值时上游判非共面；实文件头注释与上图一致（但见观察 O1）。
- `receipt.json`：`precision_convention` 改写为 f32 口径、`triangulation.unit_tests` **16**、`kernel_compile_gate.after_o1_fix` 记录 sha 与 O1 前一致且 `identical_to_before_fix: true`、`qa_fixes` 记 O1/O2/O3_O5、`tests.passed` **1086**。

### 2.4 零 CUDA 源改动与内核门

- `git show --name-only 1b876391`：14 文件，`.cu` 数量 **0**；改动 = `src/render/bvh/hfield_mesh.rs` + `tests/render_hfield_mesh.rs` + 证据 + `.normify`（卡 addenda/门禁记录）。
- 我顺带跑内核门（`megakernel_source_compiles_under_render_gate`，NVRTC 纯编译、无 GPU）：`source_sha256=72bc364c…` / `ptx_sha256=97467d83…` / `ptx_bytes=264015`，exit 0 —— 与 O1 修复前记录**完全一致**（O1 只在 host Rust）。

## 3. O2 / O3 / O5 处置确认

- **O2（如实记录）**：README §5 与 receipt `none_path_evidence` 均写明「槽 64 上传值由 `u64(0)` 变 `u64::MAX`；形状 `[1]`/类型 `Array(U64)` 不变；设备只在 `gtype==1` 读且该组合被 host 前置拒绝 ⇒ 渲染行为零差异；「上传内容不变」字面不成立」。✅ 与我的原观察一致。
- **O3**：README §9 与 receipt `limits` 记为「遗留 O3（带进 S3）：used hfield 集合未含上游 `geom_enabled_mask`；当前 host 要求 `enabled_geom_ids` 覆盖全部 geom，故当前语义一致；部分使能需补」。✅
- **O5**：README §9 与 receipt `limits` 记为「遗留 O5（带进 S3）：`SceneLeaf.size` 必须与三角化同一份 `half`；S3 fixture 侧保证或加 debug 断言」。✅
- 计数一致性：`1086 = 1082 + 4`；卡 `.normify/runs/…-r2/unit.json` 记录 `passed: 1086 / failed: 0 / ignored: 387`、exit 0（修复提交内），与实现证据/README §8 一致。✅

## 4. 观察（1 条，不阻断）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| **O-new** | 建议（源码卫生） | `src/render/bvh/hfield_mesh.rs` 第 153 行注释里两处 `f32(1e-5)` 的 `f` 字节是 **0x0C（FF 控制字符）**：实际内容为 `// \x0c32(1e-5) = 9.999999747378752e-06 …` 与 `… 差值恰为 \x0c32(1e-5) 时 …`。疑为编辑工具把 `f` 当转义处理；不影响编译/行为，但正好落在 O1 修复的关键注释处，属源码卫生问题 | **修**（可选）：把两处控制字符改回普通 `f32`；不改判据/逻辑 |

## 5. 边界与未覆盖

- 按指示**未重跑全量门禁**：全量 1086 由（a）我定向跑 `render_hfield_mesh` 16 测试全绿、（b）修复提交内的 unit 门禁记录（1086/0/387）、（c）实现证据 README/日志三方一致确认；未独立复跑 `cargo test --locked` 全量。
- 无 GPU / 无窗口 / 无渲染数值结论；本片只核阈值口径与证据处置。
- 未跑 `fmt`/`clippy` 门禁；按修复提交记录与证据日志（`docs/…/logs/{fmt,clippy}.log`）采信，属轻复核口径。
- 未重新复刻上游全 16 例三角化比对（只复跑原 2 个分歧点 + 低 1 ULP 样例）；其余 14 例在 S2 复核已逐位比对通过，本次修复只改阈值比较、不涉坐标/索引生成。

## 6. 收口状态

- `normify gate 2026-10-28-render-bvh-r05-s2-o1-fix-check` → exit 1「任务没有引用任何门禁」属预期（receipt `gate_note`）。
- 两段提交：证据（本目录 + 任务卡）→ `normify done` → verified 记录单独提交；显式路径，无 `git add -A`。
