# R06 多帧 refit 修复 r2 —— QA 只读复核

- 任务：`2026-11-05-render-r06-flex-multiframe-refit-fix-r2-check`（owner QA；**QA 自登记**，Chief 2026-11-05 夜指派；登记提交 `24a165a8`）
- 复核对象：修复卡 `2026-11-05-render-r06-flex-multiframe-refit-fix-r2`（**verified**）→ 实现 `ff8135e5`、文档引用修正 `43295e49`（`Normify-Fixes`）、根因诊断 `77aeb6da`；**复核目标 = 修正后 HEAD `43295e49`**
- 日期：2026-11-05（18:00–24:00 空闲段）；边界：**只读复核 + 只写本目录**；零 GPU、不采集、不改 r2 证据/脚本/spec/fixtures
- 复核方法：不采信执行方结论，独立 AST/静态检查 + 真实 base 数据跑三条硬自查 + CLI 子进程行为 + 上游 pinned 行号内容核对；**QA 自检 54/54 PASS**（`scripts/verify_refit_fix_r2.py`），并**复跑执行方 CPU 自检 28/28 PASS**（复跑后工作树仍 clean → 证据可复现）

## 1. 结论

**PASS-with-observations（0 阻断）。** 8 项口径全部通过：调用点唯一无分支、计次断言 3×2=6 与反例真走判定、三条硬自查在采集路径且失败 exit 3、provenance 双 sha、冻结面零改动、行数 498/427/296、文档引用（第⑦条）已修正且上游行号内容核对通过、零 GPU 自证与四门禁齐全。4 条低观察（O1–O4）均不阻断，见 §3。

## 2. 逐项核对（8 条）

### 2.1 调用点唯一无分支（PASS）
- AST：`_render_frame` 内 **恰好 1 次** `mjw.refit_bvh(m, d, rc)`（源码行 161），位于 `mjw.forward`（行 158）之后、`api.capture_once`（行 164）之前；**不在任何 If/Try/For/While/With 守卫下**（祖先控制节点为空）。
- `record_refit(frame)` 恰好 1 次、紧跟 refit（行 162）；`_render_frame` 无 refit 形参；无 `if refit`、无 skip 分支；模块源码仅 1 处 refit_bvh 调用，自查模块 0 处。
- `--refit-bvh` 已删除：`--help` 无该参数、传参 argparse **exit 2**；`_capture_pass` 在 `capture_multiframe` 内恰好调用 2 次（`--repeat 2` 两遍采集都走同一帧循环），`reset_refit_log` 首遍前 1 次。
- 采集路径 `run_capture_checks` 调用 1 次，以 `checks["ok"]` 决定 `return 0` / `return 3`，失败分支打印「不得落库」。

### 2.2 计次断言 = frames × passes = 6（PASS）
- 6 次且顺序 `[0,1,2,0,1,2]` → PASS；**反例**：只记一遍（3 次）→ **FAIL**、乱序 `[0,2,1,0,1,2]` → **FAIL**、多记（9 次）→ **FAIL**。
- 反例调用的是采集路径同一判定函数：`run_capture_checks` 内联 `"refit": refit_summary(frames, passes)`（已确认接线）。Arch 旁证指出 B2/B3 测的是判定函数本身；结合三条硬自查门禁（不 refit 时 delta 必然非 pass-margin → exit 3），构成双保险——与 `--repeat 2` 第二遍漏 refit 的失败模式互为镜像。

### 2.3 三条硬自查在采集路径且失败 exit 3（PASS）
| # | 检查 | 独立复核结果 |
| --- | --- | --- |
| ① | base 幂等（frame-000 vs `reference/base/` 数组级位级） | 真实 base 数据 **15/15 数组位级全等 → PASS**；qpos 改 1 bit → **FAIL**（命中 `frame000_vs_base`） |
| ② | 位移 = k×0.100 m（±1e-6，读采集侧 state） | 真实模型 `flex_vertbodyid` + 采集侧 `flexvert_xpos`：k=1 均值 0.100000012 / k=2 0.2 → PASS；+2e-6 → **FAIL**；+5e-7 边界 → PASS；函数体不引用 CPU `mjd` 变量 |
| ③ | 两段变化 depth+seg 均 ≥576 | 600/600 → PASS；500 → **FAIL**（insufficient）；300 → **FAIL**（non-discriminating） |
- `run_capture_checks` 汇总：全过 → `ok`；分别注入 refit/base/位移/余量四类失败 → `failures` 各命中对应项（`refit`/`frame000_vs_base`/`displacement`/`delta_margins`）；采集路径据此 **exit 3**。

### 2.4 provenance（PASS）
- `spec_sha256` = `1d9fa5cef1ebc829747d63f648d3bab1ec935d8df8e3403bc52c2ac2aa7c40b6` / 10,506 B，与实际 spec 文件一致；`authoring_script_sha256` 含三脚本且与实际文件 sha 一致；`refit_bvh=true`、`refit_policy=every-frame`。
- `capture.json` 写 `provenance`、`refit_policy`、`refit_calls = FRAMES * args.repeat = 6`。

### 2.5 冻结面未动（PASS）
- `77aeb6da..43295e49` 对 `src/`、`tests/`、`fixtures/`、`Cargo.toml`、`Cargo.lock`、判据 spec 目录**零 diff**。
- spec sha/bytes 未变（`1d9fa5ce…`/10,506 B）；帧数 3、Δz 0.10、锚点、阈值 384/576 仍硬编码、无改值入口；`--repeat 2` 冻结（`--repeat 1` → exit 1）；`--frames 0/4`、`--frame-dz 0.2` → exit 1；`--check-deltas` 三态 **0/3/3**。

### 2.6 行数与文件（PASS）
- `capture_render_flex.py` **498 行且逐字节不变**（与 `77aeb6da` blob 相同，sha `f50e150e…`）；主模块 **427**、自查模块 **296**（均 ≤500）；authoring README 99。
- `fixtures/render-bvh/scene-flex/multiframe/` 不存在；`git ls-files fixtures` 无 multiframe；范围内提交无 `.npz`。

### 2.7 文档引用（第⑦条，PASS）
- `mujoco_warp/record.py:110-111`（pinned `71da24d9`，包根；无 `_src/record.py`）与 `contrib/render.py:452-453` 均存在；实测行内容分别是 `mjw.refit_bvh(m, d, rc)` / `mjw.render(m, d, rc)`（两处都是 refit→render）。
- 证据与脚本中 **无 `mujoco_warp/_src/record.py`、无 `contrib/render.py:110`**；`scripts/authoring/README.md` 与 `capture_render_flex_multiframe_checks.py` docstring 已改为正确路径（`43295e49`）。

### 2.8 零 GPU 自证与门禁（PASS）
- 执行方 CPU 自检 `RESULT: PASS (28/28)`，QA **复跑同为 28/28**，复跑后 `git status --porcelain` 仍为空（证据可复现、未污染）。
- 四门禁记录：fmt/clippy/clippy-cuda 全过，unit **1321 过 / 0 败**（ignored 399）；r2 证据 manifest **11/11** 自核通过；工作树除本复核证据/卡外 clean。

## 3. 观察清单（按严重度，均不阻断）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | 低（证据卫生） | 4 个 CPU 自检日志含 U+FFFD 替换符：`check-deltas-synth-insufficient.log`/`-non-discriminating.log` 各 48、`-pass.log` 2、`dry-run.log` 25（执行方 `verify_refit_fix.py` 对子进程 cp936 stdout 用 `decode("utf-8","replace")`）；数值/exit 码与判定仍可读，manifest 自核通过 | 自检脚本对子进程输出做 GBK 回退解码，或设 `PYTHONIOENCODING=utf-8`；不影响修复正确性 |
| O2 | 低（证据范围） | `artifacts/refit-fix.diff` 现为**文档修正提交** `43295e49` 的 diff（1,863 B），r2 实现 diff（57 KB 级、含新增 296 行自查模块）仅以提交号 `ff8135e5` 引用（证据 README §10 已明说） | 实现 diff 可用 `git show ff8135e5 -- scripts/authoring/` 取；后续同类证据建议保留实现 diff 副本，少一次跳转 |
| O3 | 建议（引用口径） | `capture_render_flex_multiframe.py:160` 注释写「上游 `record.py:110-111`」，缺 `mujoco_warp/` 前缀（另两处已带正确前缀） | 下次触碰该文件时补全为 `mujoco_warp/record.py:110-111`，便于 grep 统一口径 |
| O4 | 事实（设计） | 三条硬自查在**帧产物已落盘后**判定：exit 3 时 `--out-dir` 已有 npz/capture.json 残留；r2 证据 README §12 已写明 G-2 窗口须立即删除并在证据留痕（G-1 先例） | 非缺陷；G-2 卡面并入该操作要求即可 |

## 4. 复现与证据

- `scripts/verify_refit_fix_r2.py`：QA 独立复核脚本（A 静态/AST、B 计次+反例、C 三条硬自查+真实 base、D provenance、E CLI、F 边界/文档），只读、零 GPU、不写仓库（`sys.dont_write_bytecode` 防 `__pycache__`）。
- `logs/verify-r2.log` / `logs/verify-r2.json`：**RESULT: PASS (54/54)**，逐项结果。
- `logs/repro-backend-selfcheck.log`：复跑执行方 `verify_refit_fix.py` → **PASS (28/28)**，复跑后工作树 clean。
- `facts.json`：机器可读事实（HEAD/提交、行数、sha、spec、上游行内容、门禁、证据 manifest）。
- `manifest.txt`：本目录 sha256 + bytes + 证据目录内相对路径（base dir 见表头，不自列）。

## 5. 非声明

- 本复核只声明「修复脚本 + CPU 自检的只读复核通过」；**不声明 G-2 采集已通过**、不声明多帧判据通过 / 动画一致性已覆盖 / 与上游等价 / 对外发布依据。
- 真实采集有效性（三条硬自查在 GPU 帧循环中通过、verdict pass-margin）由 **G-2 采集窗口**判定；本卡为其前置 QA 门。
