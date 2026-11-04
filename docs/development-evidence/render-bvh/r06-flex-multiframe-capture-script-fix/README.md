# R06 多帧采集脚本 O3 小修（采集路径按 verdict 停手 / 非零退出）

- 任务：`2026-11-05-render-r06-flex-multiframe-capture-script-fix`（owner Backend、p60、P8-渲染与BVH）
- 依赖：实现卡 `2026-11-05-render-r06-flex-multiframe-capture-script`（提交 `5a99047f`，verified）+
  QA 只读复核卡 `2026-11-05-render-r06-flex-multiframe-capture-script-check`（提交 `e1d93aca`，PASS-with-observations）
- 冻结前提（未变）：判据 spec `2026-11-05-render-r06-flex-multiframe-criterion-spec`，
  sha256 `1d9fa5cef1ebc829747d63f648d3bab1ec935d8df8e3403bc52c2ac2aa7c40b6` / 10506 B（`verify_frozen_spec()` 运行时校验）
- 边界：**零 GPU、未执行采集、未开窗口、未创建 `fixtures/` 产物**；只改 `scripts/authoring/`（扩展模块 + 用法说明）
  与证据目录；未改 `src/`、`tests/`、`fixtures/`、`Cargo.toml`；未改判据/阈值/帧数/Δz/相机。实现完成不产生验收结论。

## 1) 结论

| 项 | 状态 | 落地位置 |
| --- | --- | --- |
| **O3**（必修，真缺口）采集路径按 verdict 停手/非零退出 + verdict 入 `capture.json` | **已修** | 本卡提交（`scripts/authoring/capture_render_flex_multiframe.py`） |
| **O2** `--frames 0`/负数显式 exit 1 | **已修** | 本卡提交（同模块 `dispatch()`） |
| **O1** `dry-run-multiframe.json` 重导纯 JSON | **计划**：旧证据层，由紧随本卡的独立 `Normify-Fixes: 2026-11-05-render-r06-flex-multiframe-capture-script` 提交落地 | 旧证据目录 `r06-flex-multiframe-capture-script/logs/` |
| **O4** 证据 README「+5 行」→「+7 行」 | **计划**：同上（`scripts/authoring/README.md` 无「+5 行」字样，实际文本在旧证据 README §6） | 旧证据目录 `r06-flex-multiframe-capture-script/README.md` |
| **O5** 实现卡 receipt status | **计划**：同上（改成 `verified` 并注明收口提交） | 旧证据目录 `r06-flex-multiframe-capture-script/receipt.json` |

**为何 O1/O4/O5 不混进本卡提交**：仓库纪律「新任务收尾（`Normify-Task`）与已关闭任务修正记录（`Normify-Fixes`）
不得写进同一提交」。O1/O4/O5 的目标文件属已 verified 的实现卡证据目录，只能以 `normify fix` + `Normify-Fixes:`
提交修正；本卡提交只含 `scripts/authoring/` + 本目录。两笔提交按此顺序相邻落地，QA 最小复核时均可核。

## 2) O3：修复前后行为对照

**修复前（实现提交 `5a99047f` 的 blob）**

- `capture_multiframe()`：`delta_lines = _delta_report(out_dir, api, stream=False)`（不打印）；
- 采集末尾固定 `return 0` → 即使 `overall` 是 `insufficient-margin` / `non-discriminating` 也 **exit 0、无警告**，
  与实现证据 README §2/§8.5 不符（QA O3）。

**修复后（本卡）**

| 环节 | 实现 |
| --- | --- |
| 采集末尾强制自检 | `capture_multiframe()` 在三帧落盘后仍用**同一** `_delta_report()` 计算 cam2 相邻帧变化像素（与 `--check-deltas` 同一实现，阈值 384/576 硬编码） |
| verdict 入 capture.json | 新增 `delta_selfcheck` 块：`{thresholds, cam2_pixels, report[{pair,depth_changed,seg_changed,verdict}], overall}` |
| 警告 | 新增共享 `_print_delta_report()`：非 `pass-margin` 时打印「spec §5：未达采集前置余量（>=576），不得落库为验收材料；停手报用户，不得改阈值。」 |
| 返回码 | 新增共享 `delta_exit_code(result)`：`pass-margin → 0`，否则 **`→ 3`**；采集路径 `return delta_exit_code(delta_lines)`，不再有无条件 `return 0` |
| 独立自检 | `--check-deltas` 复用 `_print_delta_report()` + `delta_exit_code()`，语义保持 **0/3/3**（无回归） |

`logs/o3-before-after.txt` 记录了修复前 blob 的关键行（`:278` 自检不打印、`:296` 无条件 `return 0`）与修复后的
`delta_selfcheck` / `_print_delta_report(delta_lines)` / `return delta_exit_code(delta_lines)` 行号。

## 3) O2：参数边界

`dispatch()` 在任何模式分发前先校验 `--frames in (1, 3)`：

| 输入 | 修复前 | 修复后 |
| --- | --- | --- |
| `--frames 0` / `--frames -1` | 静默走单帧路径、exit 0 | **exit 1**，打印「只接受 1 或 3，收到 0/-1；其它值（含 0/负数）一律拒绝」 |
| `--frames 2` | 进帧循环后 exit 1 | **exit 1**（更早、同一原因） |
| `--frames 1`（或不带） | 单帧原路径 | 不变（dispatch 返回 None） |
| `--frames 3 --frame-dz 0.1` | 帧循环 | 不变 |

日志：`logs/o2-frames-0.*`、`logs/o2-frames-minus-1.*`、`logs/o2-frames-2.*`（stdout/stderr 分离，CRLF 已归一化为 LF）。

## 4) CPU 自证（本卡实际执行；零 GPU、零采集）

`scripts/verify_fix.py`（可复跑）输出 **`RESULT: PASS (22/22 pass, 0 fail)`**，结果在 `logs/verify-fix.json`，
分组如下：

| 组 | 内容 | 证据 |
| --- | --- | --- |
| A（动态） | `delta_exit_code` 三态映射 `{pass:0, insufficient:3, non-discriminating:3}`；`_verdict` 384/576 七个边界；`_print_delta_report` 仅非 pass 时输出「不得落库」 | `logs/verify-fix.json`（A1–A5） |
| B（接线，AST） | `capture_multiframe` 调用 `_delta_report` / `_print_delta_report`；`capture.json` 字典含 `delta_selfcheck`；末尾 `return delta_exit_code(delta_lines)`；无无条件 `return 0` 收尾 | `logs/o3-before-after.txt`（B1–B6） |
| C（O2） | `--frames 0` / `-1` / `2` 均 exit 1 且含拒绝原因 | `logs/o2-*.stderr.txt` |
| D（回归） | `--check-deltas` 合成数据三态 **0 / 3 / 3**；`--dry-run` exit 0 且 stdout 为**纯 JSON**（`json.load` 成功） | `logs/delta-regression.json`、`logs/dry-run-multiframe.stdout.txt` |
| E（行数） | 主脚本 **498 行（git blob 口径）且未改**；扩展模块 **479 行 ≤ 500** | `logs/line-counts.txt` |
| F（冻结 spec） | sha256/bytes 与常量一致（`1d9fa5ce…` / 10506 B），未被修改 | `logs/verify-fix.json`（F1） |
| G（单帧路径） | `capture_render_flex.py` 工作树 sha 与 HEAD blob **逐字节相同**（`f50e150e…`） | `logs/script-identity-after.json` |

**O3 动态限制（如实登记）**：采集路径的「真实采集 → 非零退出」需要 GPU 窗口，本卡不能执行；因此用
①纯函数 `delta_exit_code` 的动态三态证明 + ②AST 级接线证明（采集函数确实调用它、且无无条件 `return 0`）+
③`--check-deltas` 同一实现的三态回归 三者组合；真实采集的端到端非零退出留采集窗内首次执行时复核。

## 5) 行数口径（500 行棘轮）

口径：`git show <commit>:<path> | wc -l`（= blob 换行数；`Get-Content` / `Measure-Object -Line` 在 cp936 下会少算）。

| 文件 | 修改前 | 修改后 | 说明 |
| --- | --- | --- | --- |
| `scripts/authoring/capture_render_flex.py` | 498 | **498（未改，+0/−0）** | 剩余 2 行余量未使用；本卡零改动 |
| `scripts/authoring/capture_render_flex_multiframe.py` | 446 | **479（+47/−14）** | 全部改动集中于此（自检打印/退出码/verdict/O2 校验） |
| `scripts/authoring/README.md` | 53 | **57（+4/−0）** | 用法节补 O3/O2 语义 |

## 6) 与冻结 spec 的一致性

- 帧数 `FRAMES=3`、Δz `FRAME_DZ=0.10`、锚点 body、cam2（320×240 = 76,800）、阈值 `384/576` 均未改动；
- 沙箱内 `verify_frozen_spec()` 仍逐次校验 spec sha256/bytes；`--dry-run` 输出含该块（`logs/dry-run-multiframe.stdout.txt`）；
- 未增加任何改阈值/帧数/Δz/相机的 CLI 入口。

## 7) 边界与非声明

- 未执行采集、未开 GPU 窗口、未跑上游渲染；未创建 `fixtures/render-bvh/scene-flex/multiframe/`；
- 未改 `src/`、`tests/`、`fixtures/`、`Cargo.toml`；产品构建/测试不引用本模块（离线 authoring 工具）；
- 本卡不产生验收结论，不做「已覆盖/已验收/上游对齐」声明；采集窗（G-1）仍等 **S-1 用户签名 + Chief GO**；
- O6/O7（采集窗落点须传 spec 路径、frame-000 与 base 同 sha 动态对照）按 Chief 口径记已知项，本卡不动。

## 8) 证据清单

| 路径 | 说明 |
| --- | --- |
| `README.md` | 本文件 |
| `receipt.json` | 机器可读回执（`/task_id` 供 normify `json` 检查） |
| `artifacts/o3-fix.diff` | `git diff HEAD -- scripts/authoring/`（本卡全部改动） |
| `scripts/verify_fix.py` | CPU-only 可复跑验证（A–G 组，22 项） |
| `logs/verify-fix.json` | 22 项逐条结果与失败清单（`RESULT: PASS`） |
| `logs/o3-before-after.txt` | O3 修复前 blob 行 + 修复后接线行 |
| `logs/o2-frames-0.*` / `o2-frames-minus-1.*` / `o2-frames-2.*` | O2 参数边界 stdout/stderr（exit 1） |
| `logs/o2-dryrun-cases.json` | O2 + dry-run 逐例 argv/exit 机器可读汇总 |
| `logs/dry-run-multiframe.stdout.txt` / `.stderr.txt` | dry-run 纯 JSON stdout（`json.load` 可解析）+ 警告 stderr |
| `logs/delta-regression.json` | `--check-deltas` 三态回归 exit 0/3/3 |
| `logs/line-counts.txt` | git blob 口径行数（498 / 479） |
| `logs/script-identity-after.json` | 两脚本 before/after sha256、行数、主脚本 changed=false |
| `manifest.txt` | 本目录 sha256 + bytes + 证据目录内相对路径（base dir 见表头，不自列） |

## 9) 复现

```powershell
# 22 项 CPU 自证（零 GPU；从仓库根执行）
target/mjwarp-parallel/venv/Scripts/python.exe -B docs/development-evidence/render-bvh/r06-flex-multiframe-capture-script-fix/scripts/verify_fix.py

# 单点复核
target/mjwarp-parallel/venv/Scripts/python.exe -B scripts/authoring/capture_render_flex.py --frames 0 --out-dir target/multiframe-work/never --dry-run   # 期望 exit 1
target/mjwarp-parallel/venv/Scripts/python.exe -B scripts/authoring/capture_render_flex.py --check-deltas --out-dir target/multiframe-work/synth-pass              # 期望 exit 0
```
