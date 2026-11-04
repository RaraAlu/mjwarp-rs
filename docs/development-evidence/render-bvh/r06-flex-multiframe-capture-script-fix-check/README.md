# R06 多帧采集脚本 O3 修复 QA 最小复核

任务：`2026-11-05-render-r06-flex-multiframe-capture-script-fix-check`（owner QA）。复核对象：修卡 `2026-11-05-render-r06-flex-multiframe-capture-script-fix`（verified）→ 提交 **`3e5fc677`**；O1/O4/O5 旧证据修正在 **`1678545e`**（`Normify-Fixes: 2026-11-05-render-r06-flex-multiframe-capture-script`）。QA 基线 = 我上一轮复核 `e1d93aca`；HEAD `1678545e`。日期：2026-11-05（空闲段）。
**边界：只读最小复核 + 只写本目录；零 GPU、不执行采集；不改脚本/spec/实现卡证据。**

## 1. 结论

**PASS（0 阻断、0 观察）。** 卡面 6 项全部成立：O3 采集路径已按 verdict 停手/非零退出且 verdict 入 `capture.json`；O2 `--frames` 白名单生效；O1/O4/O5 文案与证据修正到位；主脚本逐字节未改（498 行口径）、扩展模块 479 行、spec 未变；两笔提交 scope 干净、trailer 不混。仅 1 条范围说明（N1：真实采集的 exit3 无法在无 GPU 下动态复跑，按卡面允许以合成/函数级+AST 演示，采集窗内复验）。

## 2. 逐项核对

### 2.1 O3：采集路径按 verdict 停手 / 非零退出（PASS）
- **静态接线（AST 解析 `capture_multiframe`）**：`_delta_report(..., stream=False)` 在落 `capture.json` 前执行；capture 字典新增 **`delta_selfcheck = {thresholds, cam2_pixels, report, overall}`**；随后 `_print_delta_report(delta_lines)`；函数**最后一句**是 `return delta_exit_code(delta_lines)`；函数体内**不再有**无条件 `return 0`。
- **功能演示（同一对共享函数，合成 cam2 数据，CPU/零 GPU）**：
  - `pass-margin`（600/600、600/600）→ `delta_exit_code=0`、**不打印**警告；
  - `insufficient-margin`（600/500）→ **exit 3** + 打印「不得落库为验收材料；停手报用户」；
  - `non-discriminating`（600/300）→ **exit 3** + 同款警告。
- **`--check-deltas` 独立模式**：同三组数据仍 **0/3/3**（`pass-margin` 0，其余 3），打印明细与警告逻辑改为共用 `_print_delta_report`、退出码共用 `delta_exit_code`，语义未变。
- 无隐藏「跳过自检」入口：`capture_multiframe` 无参数分支可跳过 `_delta_report`/退出码；模块无 skip/force 开关（grep 0 命中）。

### 2.2 O2：`--frames` 白名单（PASS）
- `dispatch()` 首先校验 `args.frames not in (1, FRAMES)` → `fail()`；实测 `--frames 0` / `-1` / `2` 均 **exit 1** 且 reason 含「只接受 1（单帧原路径）或 3（帧循环）」。
- `--frames 1 --dry-run` 仍走**原单帧路径**（exit 0，输出为原单帧 plan，不含 multiframe 计划）；参数白名单无放宽（只接受 1/3）。

### 2.3 500 行棘轮与边界（PASS）
- **主脚本逐字节未改**：`git show HEAD:scripts/authoring/capture_render_flex.py` 与 QA 基线 `e1d93aca` **完全相同**，权威口径 **498 行**（`git show … | wc -l`）。
- 扩展模块 **446 → 479 行**；`scripts/authoring/README.md` **+4/−0**（新增 O3/O2 说明与用法）。
- 冻结 spec `1d9fa5ce…` / **10,506 B** 未变，且未被两笔提交触碰。
- `src/`、`tests/`、`fixtures/`、`Cargo*` 零 diff；无 `.npz` 采集产物；`fixtures/render-bvh/scene-flex/multiframe/` 不存在；零 GPU。

### 2.4 O1 / O4 / O5 旧证据修正（PASS，提交 `1678545e`）
- **O1**：`logs/dry-run-multiframe.json` 现为**纯 JSON**（2188 B，`json.load` 通过，`mode=multiframe-dry-run`、`gpu_used=false`），PowerShell 错误记录另存 `logs/dry-run-multiframe.stderr.txt`。
- **O4**：旧证据 README 的 artifact 行改为「**+7 行 / 0 删除**」；仅修订记录里保留旧「+5 行」的引用（1 处，属引用而非行数口径）。
- **O5**：实现 receipt `status=verified`，并记录收口提交 `5a99047f`。
- 旧证据 manifest **17/17**、修卡证据 manifest **18/18** 自核通过（我独立复算 sha256/bytes）。

### 2.5 提交 scope / trailer（PASS）
- `3e5fc677`：单 trailer `Normify-Task: …-capture-script-fix`，无 `Normify-Fixes`；只含 `scripts/authoring/`（模块+README）、修卡证据、卡/runs；**不含**旧证据目录。
- `1678545e`：单 trailer `Normify-Fixes: 2026-11-05-render-r06-flex-multiframe-capture-script`，无 `Normify-Task`；只含旧证据目录、卡/runs；**不含** `scripts/`。
- 两笔均无 `src/tests/fixtures/Cargo`、无 `.npz`。两张卡（修卡 + 实现卡）均 verified。

## 3. 说明（非问题）

- **N1**：真实采集路径的 exit3 无法在无 GPU 环境下动态复跑；本复核按卡面允许的方式（AST 接线 + 共享函数合成数据演示 + `--check-deltas` 端到端）确认行为，真实采集的 exit3/警告在采集窗内复验即可。
- 其余无观察；O6/O7 仍按既定记录（spec §8.3 落点 vs `--out-dir`、单帧不变为静态判定），不在本卡范围。

## 4. 复现与证据

- `scripts/verify_fix.py`：提交 scope/trailer、主脚本逐字节与行数（498/479）、spec、O3 AST 与共享函数演示、O2 白名单、O1/O4/O5、两张证据 manifest 自核、两张卡状态。
- `logs/verify.log`：`RESULT: PASS`（0 FAIL，1 条 OBS=N1）。
- `logs/qa-rerun.json`：机器可读结果（trailers、files、lines、delta_selfcheck 键、三态 exit、O2 用例、O1/O4/O5 判定）。
- 临时目录 `target/qa-multiframe-fix-rerun/`（gitignored）；`manifest.txt` 5 条，LF、无 BOM。
