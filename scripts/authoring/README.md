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
- **采集路径采集完立即跑同一自检**：verdict 写入 `capture.json.delta_selfcheck`（`pass-margin` /
  `insufficient-margin` / `non-discriminating`）；非 `pass-margin`（余量 <576）时打印「不得落库」警告
  并以 **exit 3** 结束（与 `--check-deltas` 同义），不会静默成功；
- `--frames` 只接受 1（单帧原路径）或 3（帧循环）：`--frames 0` / 负数显式 exit 1，不静默走单帧；
- 单帧路径（不带 `--frames`/`--check-deltas`）保持原行为不变（`dispatch` 返回 None，原逻辑照跑）；
- 输出根目录由调用方显式给出，本目录脚本不写默认 fixtures 路径；本仓库当前不执行采集（采集窗另批）。

## 为什么多帧必须每帧 `refit_bvh`（G-1 负结果后的强制口径）

上游 `mujoco_warp` 的 `render()` 不会重建 BVH：`create_render_context` 在创建时刻用内部
`mjd = MjData(mjm) + mj_forward(...)` 把场景 BVH（含 1D flex 的 capsule bounds）固化进 render context，
之后 `render()` 全程不 refit。上游自己的多帧渲染路径每帧都是**先 refit 再 render**：
`mujoco_warp/record.py:110-111`（callback 内 `mjw.refit_bvh` → `mjw.render`）与
`upstream/contrib/render.py:452-453`（`contrib/` 在仓库根、不在 `mujoco_warp` 包内）。

本仓库 `capture_render_flex_multiframe.py` 的帧循环每步改 `body_pos.z` 注入位姿；若不 refit，
就是「状态变了、bounds 仍停在 base」。G-1 采集窗与 A/B 诊断的对照（同一帧集、只差 refit）：

| 帧循环 | cam2 flex 像素（frame-000/001/002） | 变化计数（000→001、001→002） | 自检 verdict |
| --- | --- | --- | --- |
| 不 refit（G-1 实际） | 654 / 246 / 69 | 697/616、244/183 | exit 3（non-discriminating） |
| 每帧 refit（A/B 诊断） | 654 / 718 / 798 | 1,167/1,074、1,262/1,126 | exit 0（pass-margin） |

因此本脚本**删除了 `--refit-bvh` 诊断开关**（默认关比没有更危险）：帧循环每帧 `mjw.forward`
之后、`render` 之前**无条件**调用 `mjw.refit_bvh(m, d, rc)`；**`--repeat 2` 的第二遍采集同样逐帧 refit**
（第二遍也是「新 state → 新 bounds」，漏掉会变成假通过）。CPU 侧用计次断言兜底：
`refit 次数必须 = 帧数 × 采集遍数 = 3 × 2 = 6`，且顺序为每遍 `frame0,frame1,frame2`，无分支可跳过。
`--dry-run` 输出 `"refit_policy": "every-frame"` 与 `"refit_calls": 6`，`--refit-bvh` 不再被接受。

## 采集路径三条硬自查（不达标 exit 3，停手）

逻辑拆在 `capture_render_flex_multiframe_checks.py`（守 500 行棘轮），采集路径内建：

1. **base 幂等**：`frame-000` 的 rgb/depth/seg/state 与 `fixtures/render-bvh/scene-flex/reference/base/`
   **数组级位级全等**（不等 → exit 3；同时补上 G-1 QA O7 的动态证据缺口）。
2. **位移断言**：读**采集侧 state**（设备回读落盘的 `state.npz.flexvert_xpos`，不是 CPU `mjd`）——
   附着到非 world body 的顶点相对 base 帧平均位移 = `k × 0.100 m`（`k = 0,1,2`，±1e-6）。
3. **变化像素余量**：`000→001` 与 `001→002` 的 depth 与 seg 变化像素均 ≥ **576**（= 1.5 × 384，
   pass-margin 口径；阈值仍硬编码，无改值入口）。

`capture.json.provenance` 记录 `spec_sha256`（`1d9fa5ce…`/10,506 B）、`authoring_script_sha256`
（修复后三脚本）与 refit 标记；`capture.json.capture_checks` 记录三条自查与 refit 计次的逐项结果。

**spec 漂移审计**：判据 spec §8.2 只写 `put_model → forward → 渲染`、未提 BVH；本修复属于补齐隐含前提
「渲染必须消费当前 state」，**不改变判据/阈值/帧设计/相机**，`spec.md` sha256 未变（`1d9fa5ce…`/10,506 B），
**不触发用户重签**，G-2 采集窗沿用 S-1 签名。

**非声明**：本脚本修复后，采集通过只能说明「采集有效 → 可进入验证窗 G-2」，不代表多帧判据通过、
动画一致性已覆盖或与上游一致。