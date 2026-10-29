# R02-S1b：天空盒解锁后的 stale 负控断言更新（仅测试）

任务：`2026-10-29-render-bvh-r02-skybox-s1b-negative-controls`（P8-渲染与BVH，owner Backend，Arch 登记）
本片性质：**测试修正片**（窗口 attempt1 失败 A 的修复）。不改产品代码、不改判据/容差/参考/fixture；
不开 GPU 窗口、不跑设备数值（改动的断言全部是 `#[ignore]` GPU 用例）。

## 1. 背景（窗口 attempt1 的失败 A）

`target/render-bvh/r02-skybox-window/attempt1/invalid-round-note.txt` 记录：`render_e2e`、`render_kernel_flags`、
`render_kernel_rgb` 三个实体 exit 101，panic 在「`render_skybox` 必须在写前拒绝（实际 Ok(())）」——
R02-S1 已按设计移除天空盒前置拒绝，但这 3 份测试的 ignored GPU 负控仍按旧行为断言（S1 的 scope 缺口：
这 3 个文件不在 S1 卡 files 内，QA 也不跑 GPU）。

## 2. 改了什么（只三份测试）

| 文件 | 变更 | 行数（LF） |
| --- | --- | --- |
| `tests/render_e2e.rs` | 移除 stale 的「skybox 拒绝」断言；新增 skybox **接受面**断言（context 可创建、`resolved().skybox.is_some()`、`render` 返回 `Ok(())`、`outputs().read(ImageOutput::Rgb)` 成功）；动态射线负控保留（`Unsupported`） | 500 → **496**（净 −4，红线 ≤500 满足） |
| `tests/render_kernel_flags.rs` | 移除 stale 的 skybox 拒绝项；新增 skybox 接受/发射成功断言（同口径）；动态射线负控保留 | 475 → **495** |
| `tests/render_kernel_rgb.rs` | 同上；另新增 **splat 负控**（`SplatInputs` 一组参数 → `LauncherError::Unsupported`），保留动态射线负控 | 270 → **309** |

新断言验证的是**新行为**（接受 + 发射 + e2e 输出缓冲已写），不是「删断言/弱化为不 panic」；
仍未支持特性（动态射线、splat）保留写前拒绝。`flex` 目前**没有 host 前置拒绝**（R06 范围），
不作负控候选，已在全仓扫描材料中说明。

## 3. 仓库级 stale 扫描

`materials/render-skybox-scan.txt` 给出全仓（`tests/` + `src/`）`render_skybox` 命中逐条分类：
接受面（产品解锁代码/新断言）、解析面（`context/*`、`params` 的字段语义、上游戏义注释）、
负控面（本片已更新的三处）。**没有发现其他「预期天空盒被拒绝」的 stale 断言**；
`tests/render_context_params.rs` / `render_lighting_params.rs` 中的 `render_skybox` 是解析/解耦断言，非拒绝断言。

## 4. 门禁

见 `materials/gates.txt`：fmt 0 / clippy 0 / clippy-cuda 0 / unit 0（**136 result 行，passed 1104 /
failed 0 / ignored 390**）/ `--no-run` 0 / `validate --fingerprints` 错误 0（5 条既有警告）。
本片**未执行任何 GPU 用例**（改动的三个测试均 `#[ignore]`），无渲染数值结论。

## 5. 影响与后续

提交本片后**必须重新冻结** attempt=2（exe 字节已变，attempt1 的 77 条目 guards 属之前树），
再跑 `-GuardsOnly` 与 stage B attempt=2。判据不变：depth 位级优先 + max_ulp≤4/beyond=0 双关。

## 6. 非声明

本片不产任何窗口结论/数值结论；不改判据、不改容差、不改参考；不写窗口级或精度/等价类断言。
