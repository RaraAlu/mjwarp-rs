# 离线 authoring 工具（非产品链）

本目录只包含**离线冻结参考生成/校验**工具，用于验收证据制作：

- 本目录的任何脚本都不被产品构建、产品测试、cargo 门禁或 `scripts/` 下的产品脚本引用；
- 生成产物只写入 `docs/development-evidence/`（gitignored 运行日志除外），**不进入产品
  fixtures**；若后续要将参考升格进 fixtures，需另立任务并单独评审；
- 产品构建与测试只运行 cargo，从不启动 Python；本目录的 Python 仅在人工/证据流程中运行；
- 运行前由包装脚本自动复核 pinned 环境（Python 与包版本、上游 HEAD 与 clean、warp.dll
  wheel RECORD、authoring 缓存哈希、GPU/驱动），任一不一致直接拒绝运行，不允许降级放行；
- 边界依据：用户 2026-10-22 批准「离线 authoring 例外」；AGENTS 的「不引入 Python 构建或
  运行链 / 产品测试不得启动 Python」约束保持不变。

## 一次性生成上游矩阵参考

```powershell
pwsh -NoProfile -File scripts/authoring/author-g03-upstream-matrix.ps1
```

默认输出到 `docs/development-evidence/g03-upstream-matrix/`，可用 `-OutputDir` 覆盖。

## 天然校验（MathDx potrf 与冻结上游 factor 逐位比对）

```powershell
pwsh -NoProfile -File scripts/authoring/validate-g03-upstream-matrix.ps1
```

需要 `target/mathdx11/out/factor-cdd46856c1496194/potrf_11_11_1_89_32_1_1_5_x_x_0.cubin`
（第一阶段离线链接产物；复现方式见 `docs/development-evidence/g03-mathdx-alignment/README.md`）。

## R06 多帧采集（帧循环模式，仅离线 authoring）

`scripts/authoring/capture_render_flex.py` 在原有单帧采集基础上增加帧循环模式与变化像素自检
（判据规格 `2026-11-05-render-r06-flex-multiframe-criterion-spec` 冻结：3 帧、Δz=0.10 m、
判据相机 cam2；阈值常量 384/576 硬编码）：

```powershell
# CPU-only 计划检查（不渲染、不占 GPU；输出根目录必须显式给出）
target/mjwarp-parallel/venv/Scripts/python.exe -B scripts/authoring/capture_render_flex.py `
  --frames 3 --frame-dz 0.1 --out-dir <输出根目录> --dry-run

# 帧循环采集（采集窗内执行；默认 --repeat 2 做两次采集逐位一致校验）
target/mjwarp-parallel/venv/Scripts/python.exe -B scripts/authoring/capture_render_flex.py `
  --frames 3 --frame-dz 0.1 --out-dir <输出根目录>

# 采集结果自检（读 <输出根目录>，打印 cam2 相邻帧 depth/seg 变化像素与判定）
target/mjwarp-parallel/venv/Scripts/python.exe -B scripts/authoring/capture_render_flex.py `
  --check-deltas --out-dir <输出根目录>
```

- 帧数 3 / Δz 0.10 / 相机选择是**冻结值**：脚本拒绝其它值（改这些需新 spec + 用户重签）；
- 单帧路径（不带 `--frames`/`--check-deltas`）保持原行为不变（`dispatch` 返回 None，原逻辑照跑）；
- 输出根目录由调用方显式给出，本目录脚本不写默认 fixtures 路径；本仓库当前不执行采集（采集窗另批）。
