# SDF any-hit 背面剔除 latent fix 独立复核（QA，只读）

任务：`2026-10-30-render-sdf-anyhit-cull-latent-fix-check`（proposed → in_progress → verified）
被核：`2026-10-30-render-sdf-anyhit-cull-latent-fix`（主卡 verified）
被核 pin：**`f63834e0`**（工作+证据+卡）；基线 **`e6eb0c9a`**；核 start 时 `revision.before = 23277f36`（本复核卡提交）。
**结论：pass / 阻断 0 / 需修 3（均为证据口径，机械修，不重跑）/ 观察 8。**
本卡只读：不开 GPU 窗口、不跑设备执行、不改代码/他人证据；允许并已自跑纯编译门与 NVRTC 纯编译门。

## 0. 一句话

代码改动本身正确且不可达（host 仍硬拒绝 `gtype==8`），新的内核身份三值我独立重算与自跑 NVRTC 全部一致，门禁 1108/0/392 复现；问题全部集中在证据的可复现口径（`--no-run` 日志缺 exit 码、`gate-*.txt` 来源标注、旧引用「全仓」计数），不需要重跑窗口或测试。

## 1. 复核方法

- 全部数字由 `git cat-file` 读 **committed blob** 现算，或由本卡自跑命令产出；不采信被核证据的结论行。
- 重算脚本：`recompute.py`（15 项 check，输出 `recompute.json`，problems 3 项即下述需修）。
- 门禁自跑：`fmt / clippy / clippy-cuda / unit` 与 `cargo test --locked --no-run`，日志在 `logs/`；NVRTC 纯编译门日志 `logs/kernel-identity.log`（`PATH` 加 `target/toolchains/cuda-12.8.1/bin`，**只编译、无设备执行**）。

## 2. 结果（15 项 check，12 过 / 3 需修）

| # | check | 结论 | 关键实测 |
| --- | --- | --- | --- |
| 1 | `scope.changed_paths` | ✓ | `e6eb0c9a..f63834e0` 共 **19** 路径 = `src` 2 + `.normify` 6 + 证据 11；`tests/ fixtures/ Cargo.*` **0** 改动 |
| 2 | `src.line_counts` | ✓ | `render_common.cu` **410→414**、`megakernel.rs` **409→440**；numstat **8/4** 与 **35/4**，合计 **+43 / −8** |
| 3 | `kernel.source_sha256` | ✓ | 6 个 `.cu` 按 `mod.rs::MEGAKERNEL_SOURCE` 顺序拼接：基线 **88,190 B / `e68e3412…`**、pin **88,785 B / `be4ef01b1ae45188ac512c91cfef60244a1f8b6670307b245ae1719a34cf5f5e`** |
| 4 | `kernel.aa_resolve_unchanged` | ✓ | `aa_resolve.cu` 两提交点**逐字节相同**：2,364 B / `0bf8af5a…` |
| 5 | `kernel.ptx_identity_selfrun` | ✓ | 我自跑 NVRTC 纯编译：megakernel `be4ef01b…` / **`4dae03ff…`** / **268,305 B**；`aa_resolve` `0bf8af5a…` / `b28e155c…` / **27,536 B**；`1 passed; 0 failed` |
| 6 | `cull.semantics_latent` | ✓ | `cull_this = cull_backfaces && !(shadow_ray && (gtype == 7 \|\| gtype == 1 \|\| gtype == 8))`；`gtype==8` 分支**不进** `render_intersect_primitive`、`d` 初值 `-1.0f`、`n` 零向量 ⇒ cull 块要求 `d >= 0`，**当前不可达**；host `reject_sdf_geometry` 仍在发射前拒绝含 8 模型，错误串逐字未变 `Unsupported("S7b3 不含 SDF 求交")` |
| 7 | `refs.inventory` | ✗（P3） | 清单**自洽**：59 条 / 31 文件（docs 27 + fixtures 4）、**0 过期**、活体断言 0；但「全仓」口径实测 **79 行/41 文件**（未列 20 行/10 文件），见需修 P3 |
| 8 | `evidence.manifest` | ✓ | manifest **10 行**与 committed blob **10/10** sha+bytes 全等、**不自登记**；证据 11 文件；其中 **2 个 UTF-16LE** |
| 9 | `gates.self_run` | ✓ | 我自跑：fmt/clippy/clippy-cuda/unit 全 **exit 0**；unit **136** result 行、**1108 passed / 0 failed / 392 ignored**；两条新单测都在跑且 ok |
| 10 | `card.validate_and_checks` | ✓ | 我重跑 `normify validate --fingerprints` = **0 错误 / 5 警告**（指纹已由 `done` 刷新）；`normify check <主卡>` 全部通过 |
| 11 | `commit.mapping_documented` | ✓ | `d132336c` 19 路径（含 2 个 `src`、证据、runs）、message 却是「治理(排程)…」；`f63834e0` 只有卡文件、message 却是「修复(渲染)…」；内容自洽、parent = 基线（见观察 O1） |
| 12 | `evidence.gate_provenance` | ✗（P2） | 证据 `gate-*.txt` 与 committed `.normify/runs/*.json` **4/4 不是同一 run**（`at`/`scope_hash`/`input_hash`/`log_sha256` 全不同），两次均全过 |
| 13 | `evidence.no_run_exit_code` | ✗（P1） | 被核 `logs/no-run.txt`：29,090 B、UTF-16LE、有 `Finished`、**无 exit 码**（卡面 acceptance 要求「含 exit 码与 `Finished`」）；receipt 记 `exit=0` |
| 14 | `boundary.unchanged_paths` | ✓ | 卡面 `unchanged`（r08-rgb-hetero / -window-material / r02-skybox）**0 改动**；`r08-rgb-hetero-validate`、`r08-rgb-hetero-check`（未列入）同样 0 改动 |
| 15 | `qa.evidence_scope` | ✓ | 本卡新文件全部落在 `…sdf-anyhit-cull-latent-fix-check/`，仓库其它路径未动 |

逐项原始数字见 `recompute.json`；材料见 `materials/`（`diff-scope` 由 check 1–2 覆盖、`refs-inventory.txt`、`evidence-hygiene.txt`、`gate-provenance.txt`、`no-run-exit.txt`、`gates-summary.txt`）。

## 3. 需修 3（都是证据口径，机械修，不重跑）

1. **P1 `logs/no-run.txt` 无 exit 码**：卡面 acceptance 明文要求「`--no-run` 日志含 exit 码与 `Finished`」；实测有 `Finished`、无 exit（receipt 里 `no_run.exit=0` 不等于日志满足）。修法：重定向时追加 `; "EXIT=$LASTEXITCODE"` 写入日志（本卡 `logs/no-run.log` 已按此格式产出，可直接参照），或按实际口径改卡面。
2. **P2 `README §4` 的 gate 来源标注不成立**：证据 `logs/gate-*.txt` 与 `.normify/runs/2026-10-30-render-sdf-anyhit-cull-latent-fix/*.json` 来自**两次不同**门禁运行（证据 `04:25:2x`、runs `04:26:2x`；`scope_hash` 例如 fmt `be5b2477…` vs `4b2559…`）。内容结论一致（全过、unit 1108/0/392），修法：改措辞为「第一次运行快照，权威记录见 runs」，或重转写 runs 那次。
3. **P3 `README §3` 的「全仓检索 59 行/31 文件」口径不可复现**：我独立扫描（排除清单文件自身）= **79 行/41 文件**，未列入 20 行/10 文件 = 5 个 `.normify` 卡 + docs 下 3 个 `recompute.py`（8 行）+ 本片 `README`(4) + `receipt`(3)。清单本身 59/31 **无过期项**，「`src/tests/scripts` 活体断言 0」成立。修法：把「全仓」改成实际口径（docs+fixtures 且排除 .py 复算脚本与本片自述文件），或补全清单并重算。

## 4. 观察 8（记录级接受，建议后续改进）

- **O1 提交映射异常**（Chief 指定记录、不阻断）：`d132336c` message「治理(排程): 归档…卡记录」实际装了代码+证据+runs+指纹更新+卡；`f63834e0` message「修复(渲染): …」实际只装卡 addendum（+7/−1）。内容与 diff 名集合自洽、卡检查全过，按既定口径不改写历史。
- **O2 主卡 acceptance 数字陈旧**：卡面写「历史引用 50 行/18 文件，全部在 docs/**」，证据实测 59 行/31 文件（含 4 条 fixtures）。证据已报实测值，建议卡面回填，避免以后复核以卡面数字对不上为缺口。
- **O3 时序**：主卡 `closed_at=04:25:43Z` 早于第二次门禁 unit 完成 `04:26:44Z`（addendum `04:26:27Z` 声明「最终化后复跑门禁」）。两次都过、结论不变；建议以后先门禁后 done。
- **O4 两个证据 blob 为 UTF-16LE/CRLF**：`logs/kernel-identity.txt`（2,158 B）、`logs/no-run.txt`（29,090 B），均带 PowerShell stderr 包装噪声。仓库历史同类 **106 个 UTF-16 blob**（docs 证据共 9,489 blob）与 613 个 CRLF blob，属既存捕获模式，故只记观察；建议今后统一 UTF-8/LF 捕获（本卡 `logs/` 已 UTF-8/LF）。
- **O5 引用清单文件 6,909,577 B（6.9 MB）**：因 4 条 fixtures 命中整行内嵌（`fixtures/g01/reference.json` 单行数 MB）。建议清单对长行截断（如 ≤200 字符）或只记 `path:line` + 命中片段，否则该证据永久占用仓库体积。
- **O6 `unchanged` 检查面偏窄**：只钉了 3 个路径，未含 `r08-rgb-hetero-validate`（R08-S3 窗口证据）与 `r08-rgb-hetero-check`；我按 diff 文件集合确认两者 0 改动，建议后续把 R08 三处都列入。
- **O7 validate 时点差**：证据 `materials/validate.txt` 是门禁时点 **0 错误 / 6 警告**（含预期 `fingerprint-drift mjwarp.render.kernel`），现在重跑是 **0/5**——`done` 已刷新指纹，与 README 声明一致，不算缺口。
- **O8 指纹刷新**：`.normify/modules/mjwarp/render/kernel.md` 指纹 `b8aeeafe…` → `0c5de3b7…`（随内核源改动），我重跑 validate 无 `fingerprint-drift`。

## 5. 边界与不宣称

- 本卡只做复核：不改 `src/`、不改任何他人证据、不开窗口、不跑设备执行；未对任何 GPU 数值或渲染结果下结论。
- 对主卡的范围表述予以确认：host 拒绝未解除、SDF 求交未实现、不改 81 槽 ABI / 判据 / 容差 / 参考；`git diff` 只有两个源码文件 + 注释/契约 + host-only 单测。
- 不宣称 SDF 渲染已覆盖、不宣称渲染数值验收通过、不宣称与上游逐位相同；本卡只核「latent 判据对齐 + 不可达前提被单测钉住」这一片。

## 6. 限制

- NVRTC 纯编译门只在**本机**（DARKSEA，CUDA 12.8.1 工具链）验证；PTX 身份与 NVRTC 版本相关，换工具链不保证同值（与既有口径一致）。
- 我未做设备执行，因此无法给出「解锁后 cull 行为在设备上正确」的任何证据；该风险留给解锁片。
- `d132336c` 提交映射异常只能按内容核对（已做），历史 message 不改写。
