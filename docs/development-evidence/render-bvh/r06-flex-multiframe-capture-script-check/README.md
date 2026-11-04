# R06 多帧采集脚本实现 QA 只读复核

任务：`2026-11-05-render-r06-flex-multiframe-capture-script-check`（owner QA）。复核对象：实现卡 `2026-11-05-render-r06-flex-multiframe-capture-script`（verified）→ 提交 **`5a99047f`**；冻结判据 spec `r06-flex-multiframe-criterion/spec.md`（sha256 `1d9fa5ce…` / 10,506 B）。日期：2026-11-05（空闲段）。
**边界：只读复核 + 只写本目录；零 GPU、不执行采集、不开窗；不改脚本/spec/实现卡证据/fixtures。**

## 1. 结论

**PASS-with-observations（0 阻断）。** 卡面 7 项（spec 对应性、单帧路径不变、阈值/帧数不可改、确定性自检、零采集、证据完整、非声明）全部成立；我独立复跑 CPU 用例（`--help` / `--dry-run` / 6 个参数错误 / 2 个 check-deltas 无数据 / 3 组合成数据）全部与实现证据一致，脚本 `RESULT: PASS`（0 fails）。
7 条低观察（O1–O7），其中 **O3 建议在采集窗开窗前确认/修**（采集路径非 pass-margin 时仍 exit 0 且未打印「不得落库」，与证据 README §8.5 表述不符）；其余为证据格式/文档笔误/参数边界，均不改变本卡「脚本实现完成」的结论。

## 2. 逐项核对

### 2.1 spec 对应性（PASS）
- **§2 帧集/Δz**：常量 `FRAMES=3`、`FRAME_DZ=0.10`、`ANCHOR_BODIES=('flex-anchor0','flex-anchor1','flex-anchor2')`；`_render_frame()` 对第 k 帧把三个锚点 `body_pos.z += k*0.10`（每帧重新从 XML 载入模型，帧间不累积），frame-000 不加偏移；**无物理步进**（全模块只有 `mujoco.mj_forward`/`mjw.forward`，无 `mj_step`/`mjw.step`）。
- **§8.3 产物布局**：`frame-{000,001,002}/{rgb,depth,seg,state}.npz` + 顶层 `capture.json`/`cam-poses.json`/`README.md`；输出根目录必须显式 `--out-dir`（脚本不写默认 fixtures 路径）。注：spec §8.3 的 fixtures 落点由调用方传入，见 O6。
- **§8.4 provenance**：`capture.json` 含 `spec`（task/sha/bytes）、`frames`/`frame_dz`/`frame_indices`、`anchor_bodies`/`anchor_body_ids`/`anchor_body_base_z`、`parameters`（含 `use_fast_math`）、`seed`/`repeat`、`files_sha256`/`per_frame_files_sha256`、`two_captures_bitwise_identical`、`environment`（versions + `upstream_revision`（pinned commit）+ GPU/driver，后者在 `want_cuda_device=True` 时）、`captured_at`/`command`。
- **§5 判定**：`DELTA_ACCEPT_MIN=384`、`DELTA_MARGIN_MIN=576` **硬编码**；`_verdict()` 三态正确；`_delta_report()` 用 `bits(depth)` 与 `seg_pair` 比较，cam2 形状 240×320 / (240,320,2) 与 base npz 键 `depth_cam2`/`seg_cam2` 一致。
- **冻结 spec**：脚本常量与证据一致（`1d9fa5ce…`/10,506 B），且 `verify_frozen_spec()` 运行时校验文件 sha/bytes；我实测 spec blob sha/bytes 与常量相同。

### 2.2 单帧路径不变（PASS，静态判定）
- `scripts/authoring/capture_render_flex.py` **+7/−0**（0 删除）：import、`add_frame_args(parser)`、main 内 4 行 dispatch；原参数默认值（`--frames` 默认 1、`--frame-dz` 默认 0.10）不变，不带帧循环参数时 `dispatch()` 返回 `None`、原流程/落盘/字段未触碰。
- 证据如实声明这是**静态判定**（无窗口、无动态重跑），动态「frame-000 与 base 同 sha」留给采集窗（spec §2 / G-2）——符合卡面「若声称逐字节不变须给出可比对依据」的要求（见 O7 记录）。

### 2.3 阈值与帧数不可改（PASS，含 O2）
- `capture_multiframe`/`dry_run_plan` 只接受帧数 3、Δz 0.10（2/0.2 均 exit 1，我独立复跑）；`--repeat` 强制 2；新增 CLI 仅 `--frames`/`--frame-dz`/`--check-deltas` 三个，**无任何改阈值入口**；`--frames 0`/负数边界见 O2。

### 2.4 确定性 / 自检（PASS，含 O3）
- 两次采集写 `.repeat2` 后逐帧逐文件 sha256 比对，不一致 `fail()` 非零退出；无跳过开关；`delta_constants` 打印。
- `--check-deltas` 三态（我独立用合成数据复跑）：pass 600/600 → exit 0；insufficient 600/500 → exit 3；non-discriminating 600/300 → exit 3；无数据/空目录 → exit 1（不伪造结果），非 pass-margin 时打印「不得落库」提示。
- 采集路径的同一自查用 `stream=False`（不打印警告、exit 0），见 **O3**。

### 2.5 零采集 / 零窗口（PASS）
提交 25 文件 = 脚本 3 个 + 证据 16 个 + 卡/门禁 runs；**无** `src/`/`tests/`/`fixtures/`/`Cargo*` 改动、**无** `.npz` 入库、`fixtures/render-bvh/scene-flex/multiframe/` 不存在；我的复跑全部 CPU-only，`--dry-run` 确认不写产物（out-dir 未创建）且 `gpu_used=false`。

### 2.6 证据完整 / 非声明（PASS，含 O1/O4/O5）
- 实现证据 manifest **16/16** sha/bytes 自核通过（我独立复算）；脚本 sha256 与 receipt/script-identity 一致；`cpu-checks.json` 的 argv/exit/输出与我独立复跑逐条一致；README 含摘要、spec 逐条对应表、参数/自检日志、单帧不变性说明。
- 非声明齐全：实现完成不产生验收结论、未采集、未开窗、未改 fixtures；无「多帧已覆盖/已验收」类表述（grep 0 命中）。

## 3. 行数口径（回应 Arch 与 Backend 的 496/432 vs 498/446 差异）

**权威口径：`git show <commit>:<path> | wc -l` = git blob 换行数 = `[IO.File]::ReadAllLines().Count`**：

| 文件 | 权威行数 | Arch `Get-Content` | `Measure-Object -Line` |
| --- | --- | --- | --- |
| `capture_render_flex.py` | **498** | 496 | 431 |
| `capture_render_flex_multiframe.py` | **446** | 432 | 374 |

- 差异根因：本机 PowerShell 5.1 的 `Get-Content` 默认按系统 ANSI（cp936）解码 UTF-8 文件，个别多字节序列吞并相邻换行导致行数少计；`Get-Content -Encoding UTF8` 即回到 498/446。
- 补充：仓库 500 行扫描脚本 `split-program/scan-oversized.ps1` 的受检扩展名是 `rs/cu/c/h/cpp/sh/ps1`，**`.py` 不在扫描范围**；因此「≤500」是自加口径。498 距 500 仅 **2 行余量**（按 480 目标已超）——该文件后续如需加行应先拆分，避免以后变成棘轮争议。
- 建议：以后复核统一写「`git show … | wc -l`」，不要用 `Get-Content`/`Measure-Object -Line` 计行。

## 4. 观察清单（按严重度）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O3 | 低-中（建议采集窗前处理） | `capture_multiframe` 用 `_delta_report(stream=False)` 且固定 `return 0`：采集后 verdict 为 insufficient/non-discriminating 时**不打印「不得落库」、也不非零退出**；证据 README §2 §8.5 行称「非 pass-margin 时打印提示」与实现不符 | 采集路径打印警告并返回非零（或至少打印同款提示），同时修正 README 表述；采集窗落库前必须显式跑 `--check-deltas` |
| O1 | 低 | `logs/dry-run-multiframe.json` 前 12 行是 PowerShell ErrorRecord 前导（因 `2>&1` 捕获），文件本身不是合法 JSON（从第一个 `{` 起正文可解析） | 改名 `.txt` 或分离 stdout/stderr；正文结论我已独立复跑验证 |
| O2 | 低 | `--frames 0`/负数静默落入单帧路径、exit 0，未按「其它值 exit1」拒绝（无放宽判据/帧数入口，仅是参数校验边界） | 显式校验 `--frames in (1,3)`；`frames<=1` 语义写清 |
| O4 | 低 | 证据 README §6 写 `capture-script.diff`「（+5 行）」，实际 +7/−0（artifact 内容正确） | 改文档数字 |
| O5 | 低 | 实现 receipt `status` 仍为 `implemented_pending_commit`（卡已 verified/committed） | 状态字符串更新 |
| O6 | 低 | spec §8.3 落点 `fixtures/render-bvh/scene-flex/multiframe/` 与脚本 `--out-dir` 参数化的偏差（已在证据 §5 声明） | 采集窗须传 spec 路径，并在窗口材料里核对路径与体积估算（≈14.2 MB） |
| O7 | 低 | 「单帧逐字节不变」为静态判定（+7/−0），动态 frame-000 vs base 同 sha 留采集窗 | 已在证据如实声明；采集窗首帧必须做该对照（spec §2/G-2） |

## 5. 复现与证据

- `scripts/verify_capture_script.py`：脚本身份/行数/sha、+7/−0 diff、spec 常量与运行时校验一致性、提交范围/零采集边界、实现证据 manifest 16/16、证据编码、CPU 复跑（help / dry-run / 6 参数错误 / 2 无数据 / 3 合成数据）与合成像素计数、500 行扫描范围核对。
- `logs/verify.log`：`RESULT: PASS`（0 FAIL；8 条 OBS，对应上表）。
- `logs/qa-cpu-rerun.json`：本次独立复跑的机器可读结果（dry-run 字段、各用例 exit、合成计数、行数口径）。
- 复跑临时目录 `target/qa-multiframe-rerun/`（gitignored，不落库）；`manifest.txt` 5 条，LF、无 BOM。
