# SDF host 解锁（U1b）独立复核（QA，只读）

任务：`2026-10-31-render-sdf-host-unlock-check`
被核：`2026-10-31-render-sdf-host-unlock`（主卡 verified）
被核 pin：**`af96c048`**；核 start 时 `revision.before = abbb19ed`。
**结论：pass / 阻断 0 / 需修 1（证据措辞，轻微）/ 观察 7。**
本卡只读：零 GPU、零窗口、零设备执行；四门禁 + `--no-run` + NVRTC 纯编译门为自跑。

## 0. 一句话

U1b 只删 host 侧 SDF 拒绝：`megakernel.rs` 440→406（+1/−35），`reject_sdf_geometry` 与 2 条 host-only 单测删除，其它拒绝全部原样；`.cu` 零改动 ⇒ 内核身份仍是 U1 三值（我独立拼接 + NVRTC 自跑确认 `778216b2…`/`34f91147…`/268,589）；unit 1106/0/392 我复现。唯一需修是 README 把「残留引用」写成 0 —— 实际 `src` 里还有 **2 行 `.cu` 注释**提到已删除的 guard（且语义已被本片反转），`removed-guard.txt` 自己已列出，属措辞/口径问题，不改代码不重跑。

## 1. 复核方法

- 改动/身份/manifest 由 `git cat-file` 读 committed blob 现算；NVRTC 与四门禁用本卡自跑日志。
- 重算脚本：`recompute.py`（8 项 check → `recompute.json`，problems 1 = P1）。

## 2. 结果（8 项 check 全过，另 1 项需修见 §3）

| # | check | 结论 | 关键实测 |
| --- | --- | --- | --- |
| 1 | `scope.pin_commit` | ✓ | `af96c048^..af96c048` 共 **20** 路径 = `megakernel.rs` 1 + `.normify` 6 + 证据 13；`.cu`/`cast.rs`/`bvh`/`tests`/`scripts`/`Cargo` 0 改动 |
| 2 | `src.line_counts` | ✓ | `megakernel.rs` **440 → 406** 行；numstat **+1 / −35**，源码只此一文件 |
| 3 | `guard.removed` | ✓（另 P1） | 可执行引用：`.rs` 里 `reject_sdf_geometry` **0**、旧错误串 `S7b3 不含 SDF 求交` **0**；但 `render_common.cu:391/:397` 仍有 2 行注释提到 `reject_sdf_geometry`（`removed-guard.txt` 已记录） |
| 4 | `host.other_rejections_kept` | ✓ | `splat`/`light_mode`/`use_precomputed_rays`/`enabled_geom_ids`/`splat_count`/`ray_count`/`nsamples` 拒绝全在 |
| 5 | `kernel.identity_unchanged` | ✓ | 6 个 `.cu` 拼接 **88,865 B / `778216b2…`**（= U1 收盘值）；`git diff -- src/render/kernel/cuda` 空；我自跑 NVRTC：PTX **`34f91147…` / 268,589 B**、`aa_resolve` `b28e155c…`/27,536 B 不变 |
| 6 | `gates.self_run` | ✓ | fmt/clippy/clippy-cuda/unit/no-run/nvrtc 全 exit 0；unit **136 result 行、1106 / 0 / 392**（U1 为 1108，删 2 条 host-only 单测，与卡面声明一致）；`--no-run` 末行 `EXIT=0` 且含 `Finished`；unit 日志里已无 `sdf_geometry_guard_tests` |
| 7 | `evidence.manifest_and_removed_guard` | ✓ | 证据 manifest **12/12**、13 文件、无 UTF-16；`removed-guard.txt` 含 HEAD 旧 9 处（含 `.cu` 2 行）与 4 条 unchanged 空 |
| 8 | `card.rollback_and_checks` | ✓ | 主卡 note 明写「U4 未过 → `git revert` 本片（U1 保留）」；`normify check` 通过；`validate --fingerprints` = 0 错误 / 5 警告 |

## 3. 需修 1（证据措辞，轻微，不重跑）

**P1 `README §2` 的「残留引用 = 0」与事实不符。** 实际：可执行引用确实 0（`.rs` 0 命中、旧错误串 0 命中），但 `src/render/kernel/cuda/render_common.cu:391` 与 `:397` **仍有 2 行注释**引用已删除的 `reject_sdf_geometry`，且这两行注释的语义已被本片反转（现在写「host 仍拒绝 8 / 本片改动今天不可达」）。`materials/removed-guard.txt` 的「working tree after this slice」段已经如实列出这 2 行，说明记录是诚实的，只是 README 概括口径写成 0 造成不一致。
修法（不改代码、不重跑）：把 README §2 改为「可执行引用 0（`.rs` 0 命中、`S7b3 不含 SDF 求交` 0 命中）；`.cu` 注释仍有 2 行引用，因本片零 `.cu` 改动（内核身份必须不变）保留，语义待下一次内核重基线时更新」。

## 4. 观察 7（记录级接受）

- **O1 `.cu` 注释语义已过时**：`render_common.cu:391/397` 说 host 仍拒绝、改动用不可达 —— U1b 之后相反。因任何 `.cu` 改动都会改 `MEGAKERNEL_SOURCE`/PTX（本片硬约束是身份不变），**不能在本片修**；建议 U4 之后或下一次有意重基线内核身份时一并更新。**U4 材料与复核不得引用这两行注释当作事实**。
- **O2 `removed-guard.txt` 记录方式很好**：HEAD 旧 9 处（含 `.cu` 2 行）+ 工作树残留 2 行 + 4 条 unchanged 空，问题（如有）一眼可见；建议以后冲突类改动沿用。
- **O3 解锁前提转移**：删掉 host guard 后「SDF 不可达」的断言消失，前提改由 U4 的 host 用例（模型 dump 语义 / SDF leaf 用 mesh bounds）+ GPU 参考比对承接；本片零 GPU，无法、也未宣称任何设备能力。
- **O4 unit 数可解释**：1108→1106 恰为删除的 2 条 host-only 单测；136 result 行不变；我自跑与证据 receipt 一致。
- **O5 内核身份不变是「不碰 `.cu`」的正向验证**：我独立拼接 sha 与 NVRTC PTX 都等于 U1 收盘三值，说明本片确实没动设备代码。
- **O6 门禁证据**：`gates.txt`/`gate-*.txt` 是 `.normify/runs/2026-10-31-render-sdf-host-unlock/` 的转写快照（runs at 2026-10-30T16:41Z），计数与我自跑一致；`.normify/runs` 权威记录在库。
- **O7 回滚声明四处一致**（commit message / 卡 note / README §4 / receipt）：U4 未过 → `git revert` 本片、U1 保留；U4 复核时我会按「失败即回滚」检查这条路径。

## 5. 边界与不宣称

- 本卡只读：零 GPU、零窗口、零设备执行；不改 `src/`、不改他人证据。
- 确认主卡口径：本片只删 host 拒绝，不实现 SDF 求交、不改内核身份、不宣称能力；**U4 之前不得发布/宣称 SDF 能力**。
- 不宣称渲染数值验收通过、不宣称上游等价、不宣称 SDF 渲染已覆盖。

## 6. 限制

- 解锁后的实际能力（含 SDF 场景能否正确加载/渲染）只能由 U4 窗口验证；本卡只能确认「host 不再拒绝 + 设备代码未变」。
- `.cu` 注释残留的语义修正需要改内核身份，本片范围内不可做（见 O1）。
