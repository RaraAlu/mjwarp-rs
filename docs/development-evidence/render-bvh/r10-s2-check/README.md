# R10-S2 独立复核（QA）：多 world 端到端窗口（两轮 + precheck abort）

- 任务：`2026-10-28-render-bvh-r10-s2-check`（复核对象 `2026-10-28-render-bvh-r10-s2-multi-world`）
- 判据树：HEAD `48ea01a2`；实现/窗口证据绑 `40e9238f`；结论：**pass / 阻断 0 / 观察 4（均建议级）**
- 只读复核：未重开门、未重跑 GPU、未改代码/判据/参考/容差

## 1. 总体判定

三段记账（attempt1 有效轮 1 未过 → attempt2-exit9 非有效轮 → attempt2-run 有效轮 2 全绿）全部由我从
`target/r10-s2-window/` 原始产物独立重算：attempt2-run **148 样本 / used 3283–3556(seq 87) / free_min 12506 /
util_max 9 / streak 0 / delta 270·90·273 / floor 9708**；attempt1 **156 / 4492(seq 92) / 11570 / 35 /
291·270·295 / floor 8789**；exit9 留档 **61 B `069d142d…` + 51 B `6c5acd89…` 一字未动**；
守卫 **46/46** 自行重哈希全 match、窗后 **8/8**；runner `4d636c39…`/14968 B、`guards.json 0c849e67…` 未变；
零编译 **1823→1823**、max_write 04:23:44 不变；红线/行数全部符合。判据语义从测试源码逐条核对（位级优先、
`max_ulp<=4` 兜底、world1 120000/120000 逐位、隔离、两次 SHA），且 40e9238f 的修复只改参考配方、断言零放宽。
四条观察均为记录层面，不阻断。

## 2. 窗口采样（我自算，raw 与入库副本逐项相同）

| 项 | attempt1（有效轮 1，未过） | attempt2-run（有效轮 2，全绿） |
| --- | --- | --- |
| 样本数 | **156** | **148** |
| used_min / used_peak(seq) | **4197** / **4492**(92) | **3283** / **3556**(87) |
| free_min / util_max / streak | **11570** / **35** / **0** | **12506** / **9** / **0** |
| delta vs baseline / 首样本 / 样本最小 | **291 / 270 / 295** | **270 / 90 / 273** |
| baseline_free → floor（min(10240, free−3072)） | 11861 → **8789** | 12780 → **9708** |
| 采样间隔 median（min–max） | 0.2818（0.2783–0.3007） | 0.2814（0.2773–0.2943） |
| body_wall（entity 区间） / window span | 44.1 s / 46.5 s | **41.7 s** / 44.1 s |
| 7 回归 + 新实体 exit | 7×0 + **101** | **8/8 全 0** |
| 有效轮 | 1 | 2（`attempt_valid_rounds=2`） |

`increment_limit 3072` 未触、floor 未破、guard 未 abort、residual 0、compute-apps raw 29 / filtered 0。

## 3. 判据语义（从测试源码逐条核，非抄数字）

- world0：depth 对**同配方**（`recipe_depth_seg()`，`use_fast_math=False`）参考做 `mismatched==0`（位级）+
  `max_ulp <= MAX_DEPTH_ULP(4)` 兜底；seg `mismatched==0` 全等；RGB `beyond_one_lsb_pixels==0`（各相机）。
  阈值/容差零放宽；attempt1 的参考配方错配（误用 fast-math `s7b1::REFERENCE`）已在 `40e9238f` 修正，
  `40e9238f` 的测试 diff 未改任何断言（只换参考）。
- world1：`&depth_a[depth_stride..] == &depth1[..]`、seg/RGB 同理——与 `nworld=1` 单独渲染同 state
  **逐位相同**（120000/120000）；且断言两 world 图像必须不同（per-world 相机生效）。
- 隔离：world1 几何 +1000 → world0 区段逐位不变、world1 depth 全 0、seg 全 `[-1,-1]`。
- 两次运行 depth/seg/RGB 分别 SHA 相同（三个断言）。
- attempt2-run 日志：`2 passed; 0 failed`，两条 GPU 用例均 ok；attempt1 日志 `0 passed; 2 failed`
  （`left: 19074` 与 `Capacity { slot: 0, required: 56, actual: 28 }` 两条根因如实留档）。

## 4. 守卫与冻结实体（自己重哈希）

- `attempt2-run/sha-guards.txt` **46/46**：8 exe + 38 只读输入，逐条重算 sha256+bytes **全 match**；
  与 `guards.json`（`frozen_exes` 8 + `guard_inputs` 38，`0c849e67…`）集合逐项一致。
- 窗后 8 exe 复查 **8/8**（当前盘上文件对登记 sha/bytes 全 match）；`frozen-shas.txt` 8 个 exe + 39 个
  夹具条目逐条 match。
- `guards-check-r2/sha-guards.txt` 46 行且与窗口守卫清单逐条一致；旧 `guards-check*/` 仅历史（README 已注明）。
- runner `target/r10-s2-window/run-window-attempt1.ps1` **14968 B / `4d636c39…`**，窗口内零编辑。

## 5. 三段记账与 exit9 留档（不删不改）

- `attempt1/`（有效轮 1）：floor 8789、baseline `4201/11861/38…`、8 exits（101 + 7×0）、156 样本。
- `attempt2-exit9/`：`facts.txt` **61 B / `069d142d…`**（含 `precheck_abort open_util=40>=40`）、
  `guard.log` **51 B / `6c5acd89…`**；**与 target 原件逐字节相同**，无 baseline/实体/采样（非有效轮）。
- `attempt2-run/`（有效轮 2）：8/8 exit 0、floor 9708、baseline `3286/12776/23…`、148 样本。
- 证据副本与 target 原件（attempt1 的 facts/samples/baseline/sha-guards）逐字节相同。

## 6. 零编译与红线/行数

- `deps_before/after_count=1823/1823`、`deps_max_write` 前后 `10/28/2026 04:23:44` 不变、
  `zero_rust_compile=True`、`feature_profile=cuda-probe`、`env_cuda_bin` 在 facts。
- `camera_output.rs` **499**、`model/render.rs` **500**、`launch.rs` **500**、`cast.rs` **494**、
  `render.cu` **499（贴线事实已记录）**、`megakernel.rs` 491、新测试 `render_r10_multi_world.rs` 298。
- `git diff 40e9238f..HEAD` 对 `camera_output.rs`、`model/render.rs`、`launch.rs`、`cast.rs`、
  `kernel/cuda`、`fixtures`、`Cargo*` **为空**；`DEVICE_SOURCES=28` 锚点仍在；判据/容差/参考/夹具零改动。

## 7. 门禁独立复跑

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --check` | exit 0（1.6 s） |
| `cargo clippy --all-targets --locked -- -D warnings` | exit 0（2.0 s） |
| `cargo clippy --all-targets --locked --features cuda -- -D warnings` | exit 0（2.4 s） |
| `cargo test --locked` | exit 0：**1040 passed / 0 failed / 384 ignored**（128 result 行，13.9 s） |
| `cargo test --locked --test render_r10_multi_world`（host） | **3 passed / 2 ignored**（GPU 两条 window-only） |

与后端 gate 归档（unit 1040/0）一致；窗口内 GPU 结果按只读日志复核（不重跑）。

## 8. 观察（4 条，均建议级）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | 建议 | `48ea01a2` 用 `git add -A '.normify'` 把 4 张登记卡（r09-s2-check / r09-s2-lighting-kernel / r10-s2-check / per-camera-rgb-gap）一并入库（内容未改；Arch 已接受，Chief 已禁 `git add -A`） | **接受并保持**；后续一律显式路径（已记规范） |
| O2 | 建议 | 证据 `manifest.txt` 含**自引用行** `… 8325 manifest.txt`，而文件现为 **8286 B**（自哈希无法自洽且数值过期） | **修**：去掉自引用行，或在最终写完后重算 |
| O3 | 建议 | README §1 写「日志…`mismatched=19074`」，实际 attempt1 日志是断言 `left: 19074 right: 0`（无 `mismatched` 字面）；根因叙述本身正确 | **修**（措辞对齐）或**接受** |
| O4 | 建议 | 卡面要求写明「上游 nworld>1 数值**未覆盖**」；证据用「无上游 nworld=2 数值参考：不对上游 nworld>1 做任何数值结论」表述（语义等价但无「未覆盖」字面） | **修**（补字面）或**接受**（语义已明确） |

## 9. 边界与不宣称

只读复核：未重开门、未重跑 GPU/实体、未改代码/判据/参考/容差。world0/world1 的窗口数值来自
attempt2-run 只读日志与测试源码语义核对（不重跑）；本片不宣称上游 nworld>1 数值等价（无该参考），
不写「窗口通过/已覆盖/精度达标/上游等价」。
