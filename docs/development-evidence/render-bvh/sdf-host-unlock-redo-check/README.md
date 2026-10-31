# U1b redo 独立复核（QA，只读）

任务：`2026-10-31-render-sdf-host-unlock-redo-check`
被核：`2026-10-31-render-sdf-host-unlock-redo`（pin **`013d8d4a`**）
复核 start 时 `revision.before` = **`2006544f`**
**结论：pass / 阻断 0 / 需修 2（均证据措辞，轻微，不重跑）/ 观察 7。**
只读：零 GPU、零窗口、零编译、零冻结；未改判据/参考/代码/历史证据。

## 0. 一句话

本片只删 host 侧 SDF 拒绝并把 `megakernel.rs` 从 440 行缩到 405 行：`reject_sdf_geometry` 与其 2 条 host-only 单测已删（`.rs` 可执行引用 0、旧错误串 0），其余 4 处 `Unsupported` 拒绝原样；`.cu` 零改动 ⇒ 我独立按 6 段拼接重算 `MEGAKERNEL_SOURCE` = **88,865 B / `778216b2…`**（与证据一致），`aa_resolve` source = **2,364 B / `0bf8af5a…`**；`013d8d4a + fa604008` 叠加后 host 放行 `gtype==8` 与设备侧 `a = dot3_ba` 同时在场，attempt4 前置成立。需修只有两处措辞：README 未提 `.cu` 里 2 行语义已反转的注释；「与 af96c048 同形态」与 405/406 差 1 行说明注释未写清。

## 1. 复核方法（全部只读命令，可复跑）

- 范围：`git show --stat 013d8d4a`、`git diff --name-only 1735d062..013d8d4a -- src tests fixtures ...`
- 行数/守卫：`git show <rev>:src/render/kernel/megakernel.rs`、`git diff --numstat`、`git grep -n reject_sdf_geometry <rev> -- '*.rs' / '*.cu'`
- 内核身份（独立重算）：按 `src/render/kernel/mod.rs` 的 `MEGAKERNEL_SOURCE = concat!(scene.cu, render_math.cu, render_common.cu, render_shadow.cu, render_shade.cu, render_megakernel.cu)`，用 `git cat-file blob 013d8d4a:<seg>` 取 **committed blob** 逐段拼接，算 sha256 与总字节；`AA_RESOLVE_SOURCE` 同法单独取 `aa_resolve.cu`。
- 证据卫生：上游 `manifest.txt` 逐行按 committed blob 重算 sha256/bytes；`src-diff.patch` 与真实 `git diff` 的规范化文本比较。
- 门禁：读 `.normify/runs/2026-10-31-render-sdf-host-unlock-redo/*.json` 与证据 `logs/gate-*.txt` 对比（provenance），unit 数取 runs 记录。
- 叠加态：`git diff --name-only fa604008..HEAD -- src`、`git grep reject_sdf_geometry HEAD -- '*.rs'`、`git grep 'float a = dot3_ba' HEAD -- src/render/kernel/cuda`。
- **零编译**：PTX / aa_resolve PTX 未独立复算（见 O3）；source 身份已独立重算。
- 原始输出：`materials/selfcheck.txt`（生成脚本 `materials/selfcheck.ps1`，UTF-8 BOM —— Windows PowerShell 5.1 读取中文脚本所需；其余文本证据 LF 无 BOM）。

## 2. 结果（9 项 check）

| # | check | 结论 | 关键实测 |
| --- | --- | --- | --- |
| 1 | `scope.pin_commit` | ✓ | `013d8d4a` 共 **18** 文件 = `megakernel.rs` 1 + 卡/模块 md/runs 6 + 证据 11；`src` 只此一文件；`git diff -- src/render/kernel/cuda src/render/bvh` 空；`tests/fixtures/scripts/Cargo*/criterion-v2` 0 改动 |
| 2 | `src.line_counts` | ✓ | `megakernel.rs` **440 → 405**，numstat **0 / 35**（纯删除）|
| 3 | `guard.removed` | ✓（另 P1） | `.rs` 里 `reject_sdf_geometry` **0**、`sdf_geometry_guard` **0**、旧错误串 `S7b3 不含 SDF 求交` **0**；其余 4 处 `Unsupported` 保留（precomputed_rays/splat/light_mode/enabled_geom_ids）；`.cu` 注释仍有 **2** 行提到已删守卫（`render_common.cu:391/397`）|
| 4 | `kernel.identity_unchanged` | ✓ | 6 段拼接 **88,865 B / `778216b2…`**（= U1 收盘值）；`AA_RESOLVE_SOURCE` **2,364 B / `0bf8af5a…`**；PTX `34f91147…`/268,589 与 `b28e155c…`/27,536 与证据一致（未独立复算）|
| 5 | `evidence.manifest_and_patch` | ✓ | 上游 manifest **10/10** 对 committed blob 逐字节相符；`src-diff.patch` 与真实 diff 逐字相同 |
| 6 | `gates` | ✓（另 O1） | unit runs **1111 / 0 / 397**、exit 0；fmt/clippy/clippy-cuda passed；`no-run` 含 Finished + EXIT=0；`validate --fingerprints` 复跑 0 错 / 5 警告 / 无 drift |
| 7 | `stacked_state` | ✓ | `fa604008..HEAD -- src` 空；HEAD `megakernel.rs` 405 行、`.rs` 守卫 0；`render_math.cu:275` `a = dot3_ba` 且落在 6 段拼接内 ⇒ attempt4 前置同时在场 |
| 8 | `claims` | ✓（另 P1/P2） | README 440→405 ✓、零 `.cu` ✓；未提 `.cu` 注释残留（P1）；「同形态」未写 405/406 差 1 行（P2）；attempt4 材料 v2 的 prepare checklist 已含 host 解锁与 `10070226…/7782aeb1…/268,589` |
| 9 | `validate_rerun` | ✓ | HEAD 上 `normify validate --fingerprints` = **0 错误 / 5 警告 / 0 drift** |

## 3. 需修 2（证据措辞，轻微；不改代码、不重跑）

- **P1（README/receipt 缺注释残留说明）**：上游 README §2「零 `.cu` 改动」成立，但 `src/render/kernel/cuda/render_common.cu:391` 与 `:397` 仍各有一行注释引用**已删除**的 `reject_sdf_geometry`，且语义已被本片反转（现写「host 仍拒绝 8 / 本片改动今天不可达」）。证据未提这 2 行，读者会以为源码里已无相关文本（口径与上次 U1b 复核 P1 同类）。
  修法：README §2 补「`src` 可执行引用 0；`.cu` 注释仍有 2 行（因零 `.cu` 改动/内核身份必须不变而保留），语义待下一次内核重基线更新」，receipt `change.unchanged` 同步。
- **P2（「与 af96c048 同形态」不精确）**：`af96c048` 在删调用点处**新增 1 行解锁说明注释**（`// U1b：SDF（gtype==8）host 解锁——设备侧与 MESH 同路径（U1），数值验证在 U4 窗口。`），故 440→**406**、+1/−35；本次重做未带该注释，440→**405**、−35。守卫与 2 条单测的删除同形，语义等价。
  修法：README §2 写清「同形指删守卫与单测；未带 af96c048 的 1 行解锁注释（405 vs 406）」，或恢复该注释（属 host 源码改动，需另开片/重跑门禁，不建议在本卡做）。

## 4. 观察 7（记录级接受）

- **O1 门禁快照与 `runs/` 非同一次**：证据 `logs/gate-*.txt` 时间 03:11:49–03:13:30，`.normify/runs` 为 03:14:01–03:14:34；clippy/unit 的 `log_sha256` 不同，cli wall 14512 vs 456 ms、unit 85371 vs 32511 ms。判定数值一致（全 passed、unit 1111/0/397）。README 未声称快照来自 runs，建议注明或从 runs 重生成。
- **O2 validate 快照含 drift**：`materials/validate.txt` 是 0 错 / **6** 警告（含 `evidence/fingerprint-drift mjwarp.render.kernel`），该 drift 由同笔提交里的模块指纹刷新消除；我在 HEAD 复跑 = 0 错 / 5 警告 / 0 drift。历史快照，不矛盾。
- **O3 PTX 未独立复算**：按「不跑构建」纪律，未跑 NVRTC；PTX/aa_resolve PTX 取证据值，与既有 U1/窗口记录三方一致。source 身份是我独立重算的强证据。
- **O4 `no-run.txt` 为裁剪转写**（Finished + Executable lines + EXIT=0），弱于原始日志。
- **O5 下游材料已承接 checklist**：attempt4 材料 v2（`9f12002d`）prepare checklist 已写「host 解锁状态/ criterion 版本/ 清单哈希/ kernel 身份 `10070226…/7782aeb1…/268,589`」（我不复核该材料本身）。
- **O6 U4 窗口卡 note 仍过时**：`.normify/changes/2026-10-31-render-sdf-validate.json` 末段仍写「U1b 重做暂缓（SDF 能力维持不可达）」，与 `013d8d4a`/`fa604008` 现状不符；属窗口卡材料更新范围（我没动、也不建议我改）。
- **O7 叠加态成立**：HEAD 上 `git diff fa604008..HEAD -- src` 为空，host 已放行 `gtype==8`、设备源 `a = dot3_ba` 在 6 段拼接内 —— attempt4 的两项前置（host 放行 + a-order 形态）同时在场。

## 5. 边界与不宣称

- 只读：零 GPU、零窗口、零编译、零冻结；不改 `src/`、不改他人证据、不改判据/参考。
- 不宣称 SDF 通过、不宣称渲染数值验收通过、不宣称上游等价、不宣称精度达标、不宣称 attempt4 会通过。
- 本卡只证明「host 不再拒绝 + 设备源未变 + 与 refix 叠加态成立」，设备能力与数值结论只能由 attempt4 窗口给出。

## 6. 限制

- PTX 与 GPU 用例未重跑（零构建纪律）；source 身份独立重算，PTX 仅跨记录一致。
- `.cu` 注释语义修正需要改内核身份，本片范围内不可做（见 P1/O1）。
