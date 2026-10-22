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
