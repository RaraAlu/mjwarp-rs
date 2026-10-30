# SDF 解锁 U1（设备 mesh 路径对齐）独立复核（QA，只读）

任务：`2026-10-30-render-sdf-mesh-path-check`
被核：`2026-10-30-render-sdf-mesh-path`（U1；主卡 verified）
被核 pin：**`ed508d77`**（代码+证据实际在此提交；`eff2de88` 只装卡记录，按内容核）；基线 **`02b9c629`**；核 start 时 `revision.before = b9bc2c1d`。
**结论：pass / 阻断 0 / 需修 0 / 观察 7。**
本卡只读：零 GPU、零窗口、零设备执行；允许并已自跑纯编译门（四门禁 + `--no-run`）与 NVRTC 纯编译门。

## 0. 一句话

`gtype==8` 在主命中（`render_common.cu`）与 DepthSeg（`render.cu`）两处都并入 mesh 路径、`hit.mesh_id` 一并填 8，上游「SDF 按底层 mesh 渲染」我逐条核到 pinned 源码；host `reject_sdf_geometry` **原样保留**，含 SDF 的模型今天仍到不了设备，所以改动**不可达**、能力未暴露。内核身份新旧三值我独立重算 + NVRTC 自跑全对，门禁 1108/0/392 复现。

## 1. 复核方法

- 全部数字由 `git cat-file` 读 committed blob 现算，或由本卡自跑命令产出；不采信被核证据的结论行。
- 重算脚本：`recompute.py`（16 项 check → `recompute.json`，problems 0）。
- 门禁自跑（本卡）：`fmt / clippy / clippy-cuda / unit / --no-run`；NVRTC 纯编译门（只编译、无设备）日志 `logs/kernel-identity.log`。

## 2. 结果（16 项 check，全过）

| # | check | 结论 | 关键实测 |
| --- | --- | --- | --- |
| 1 | `scope.pin_commit` | ✓ | `02b9c629..ed508d77` 共 **19** 路径 = `src` 2 + `.normify` 6 + 证据 11；`tests/ fixtures/ Cargo.*` 0 改动；提交 message 是「治理(排程): 关闭…卡记录」（观察 O1） |
| 2 | `src.line_counts` | ✓ | `render_common.cu` **414→415**、`render.cu` **499→496**；numstat 6/5 + 3/6 = **+9 / −11** |
| 3 | `kernel.source_sha256` | ✓ | 6 个 `.cu` 拼接：基线 **88,785 B / `be4ef01b…`** → pin **88,865 B / `778216b2…`** |
| 4 | `kernel.aa_resolve_unchanged` | ✓ | `aa_resolve.cu` 两提交点逐字节相同，2,364 B / `0bf8af5a…` |
| 5 | `kernel.ptx_identity_selfrun` | ✓ | 我自跑 NVRTC：megakernel **`778216b2…` / `34f91147…` / 268,589 B**；`aa_resolve` `0bf8af5a…`/`b28e155c…`/27,536 B；`1 passed; 0 failed` |
| 6 | `device.dispatch_semantics` | ✓ | ① `render_common.cu` mesh 分支条件含 `gtype == 8`；② `hit.mesh_id = (gtype == 7 \|\| gtype == 8) ? mesh_index : -1`；③ `render.cu` DepthSeg 的 `if (gtype == 7 \|\| gtype == 8)` 里 8 也调 `ray_mesh_leaf(scene.leaf_mesh[leaf], …)`；④ `cull_this` 已排除 8（latent fix）；⑤ `render_megakernel.cu` 顶点法线分支要求 `mesh_id >= 0`（所以 ② 必须补）；⑥ `cast.rs` 的 `leaf_mesh` 对 `7 \| 8` 填 `geom_dataid` |
| 7 | `host.rejection_retained` | ✓ | `megakernel.rs` 与基线**逐字节相同**（21,373 B / 440 行），`reject_sdf_geometry` 与错误串 `Unsupported("S7b3 不含 SDF 求交")` 原样；latent fix 的 2 条 host-only 单测仍在跑且 ok |
| 8 | `upstream.citations` | ✓ | pinned upstream `71da24d9`（clean）：`render.py` L738-761 `MESH\|SDF` 同走 `ray_mesh_with_bvh(_anyhit)` + `geom_dataid`；`render_test.py:1251` 原文「Mesh and SDF geoms render using the underlying mesh geometry.」；`bvh.py` L211-220 `MESH\|SDF` bounds 取 `mesh_bounds_size[did]` |
| 9 | `refs.inventory` | ✓ | pin `02b9c629` 口径：**28 行 / 14 文件**（docs 8 / fixtures 4 / `.normify` 2），清单 0 过期、0 缺失，`src/tests/scripts` 活体断言 **0** |
| 10 | `evidence.manifest` | ✓ | manifest **10 行** 与 committed blob **10/10** sha+bytes 全等、不自登记；证据 11 文件；2 个 UTF-16 日志（观察 O2） |
| 11 | `evidence.no_run_exit_code` | ✓ | `logs/no-run.txt`（29,068 B，UTF-16LE）：含 `Finished`，**末行 `EXIT=0`** |
| 12 | `evidence.gate_provenance_documented` | ✓ | 证据 `gate-*.txt` 与 `.normify/runs/…` 是**不同次** run（证据 10:01–10:03 vs runs 10:04–10:05，4/4 `scope_hash` 不同）；README §4 已明示「某次门禁转写快照、权威见 runs」，unit 均 1108/0/392 → 措辞与事实一致 |
| 13 | `gates.self_run` | ✓ | 我自跑：fmt/clippy/clippy-cuda/unit/no-run 全 **exit 0**；unit **136** result 行、**1108 / 0 / 392**；两条 latent guard 单测在跑 |
| 14 | `card.validate_and_checks` | ✓ | 我重跑 `normify validate --fingerprints` = **0 错误 / 5 警告**（`done` 已刷新 `mjwarp.render.kernel` 指纹）；`normify check <主卡>` 全部通过 |
| 15 | `commit.mapping_documented` | ✓ | `ed508d77` 装 19 路径含 2 个 `src`；`eff2de88`（message 写「功能(渲染)…」）只装卡 JSON → 内容自洽，记录为观察 O1 |
| 16 | `boundary.unchanged_paths` | ✓ | R08 / R02 证据目录、`megakernel.rs` 均 0 改动 |

逐项原始数字见 `recompute.json`；门禁日志见 `logs/`，门禁退出码见 `materials/gates-summary.txt`。

## 3. 观察 7（记录级接受）

- **O1 提交映射异常（第三次同类）**：代码+证据落在 `ed508d77`（message「治理(排程): 关闭 SDF 解锁 U1 卡记录」），而 message 写「功能(渲染): …U1：3 处 dispatch」的 `eff2de88` 只装卡 JSON（+7/−1）。内容与 diff 名集合自洽、卡检查全过，按既定口径按内容核、不改写历史；建议后续用 `Fixes`/`Refs` trailer 或单独工作提交，避免复核时再解释一次。
- **O2 两个证据日志为 UTF-16LE + CRLF**：`logs/kernel-identity.txt`（2,138 B）、`logs/no-run.txt`（29,068 B），带 PowerShell stderr 包装噪声；仓库历史同类很多（`docs/development-evidence` 下 106 个 UTF-16 blob），属既存捕获模式，建议今后统一 UTF-8/LF（本卡 `logs/` 即 UTF-8/LF）。
- **O3 证据快照与 `.normify/runs` 不是同一次 run**：证据 `10:01–10:03Z`、runs `10:04–10:05Z`，4/4 `scope_hash` 不同；unit 计数两边均 1108/0/392。README 已按上次复核建议改成「某次…权威见 runs」，措辞准确 → 不算需修，仅记录。
- **O4 validate 时点差**：证据 `materials/validate.txt` 是门禁时点 **0 错误 / 6 警告**（含预期 `evidence/fingerprint-drift mjwarp.render.kernel`），我现跑是 **0/5**——`done` 已按 `kernel.md` 新指纹刷新，与 README 声明一致。
- **O5 行数棘轮**：precheck 要求 `render.cu` 净零或负（500 行棘轮）、`render_common.cu` 约 ≤417；实测 **496（净 −3，比预估的 ~498 更保守）**、**415**，均满足；`megakernel.rs` 440、`cast.rs` 481 未动。
- **O6 本片仍是不可达分片**：host 硬拒绝 `geom_type==8`，含 SDF 的模型到不了设备 ⇒ 设备侧新 dispatch 今天无法被任何设备执行验证；U1b（host 解锁）与 U4（验证窗）都在后面，且 U4 需新冻结 guards（U1 改了 `.cu`，旧 exe 哈希全部失效）。
- **O7 旧引用清单含自引用**：pin `02b9c629` 的 28 行/14 文件里包含我上一张复核（`sdf-anyhit-cull-latent-fix-check`）证据对旧三值的引用；清单按 pin 固定、0 过期 0 缺失，不影响结论。

## 4. 边界与不宣称

- 本卡只做只读复核：不改 `src/`、不改他人证据、不开窗、不跑设备执行；未对任何 GPU 渲染数值下结论。
- 确认主卡范围表述：不改 81 槽 ABI / 判据 / 容差 / 参考，`megakernel.rs` 零改动，host 拒绝未解除、SDF 求交未实现、能力未暴露。
- 不宣称 SDF 渲染已覆盖、不宣称渲染数值验收通过、不宣称与上游逐位相同；本卡只核「设备 dispatch 对齐 + 不可达前提仍被 host 守卫钉住」。

## 5. 限制

- NVRTC/PTX 身份只在本机 pinned 工具链（CUDA 12.8.1）与编译阶段复现，换工具链不保证同值。
- 未做设备执行 ⇒ 无法证明解锁后 SDF 与 mesh 路径在 GPU 上逐位一致；该风险归 U4。
- `ed508d77` 的 message 与内容错位只能按内容核对（已做）。
