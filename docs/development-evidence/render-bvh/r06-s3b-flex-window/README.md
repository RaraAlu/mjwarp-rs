# R06-S3b④ flex 渲染验证窗（attempt 1：INVALID_ROUND，未产出任何数值结论）

任务：`2026-11-04-render-r06-s3b-flex-validate`（owner Arch；材料 `…-window-material-r2`、设备用例 `…-s3b-flex-device-harness`、冻结器 `…-freeze-guards`）。
窗口开启：Chief 2026-11-04 07:32 GO（群内留痕已发），唯一执行会话 = arch-chat。
本片证据只记录**这一轮窗口的实施与中止原因**；**不宣称 flex 渲染通过、也不宣称不通过**——数值判据从未被评估。

## 1. 开窗前状态（全部满足）

| 项 | 值 |
| --- | --- |
| HEAD | `184f8084`（S3b①–⑤ 全 verified） |
| worktree | 0 dirty（EOL 规范化后确认；规范化脚本本身把 1 个历史 stale 文件归一回 HEAD，零 blob 变化） |
| EOL | `normalize-evidence-eol.ps1`：13,253 tracked 扫描，stale 1 → normalized 1，real-diff 0 |
| BOM | `check-evidence-bom.ps1`：**128 non-.ps1 WARN + 4 manifest 违规**，与卡片基线一致，无新增 |
| build 进程 | 0（cargo/rustc/cl/nvcc/link 均无） |
| 工具链 | `target/toolchains/cuda-12.8.1/bin`（含 nvrtc64_120_0.dll 等 14 项）；`target/toolchains/mujoco-3.12.0/package/bin`（mujoco.dll 等 10 项） |
| 环境中非我方 GPU 进程 | 1 条 `C+G`：`codex-computer-use-swift.exe` pid 51380（桌面辅助；util 2%；全卡 used 1372 MiB）。Chief 2026-11-04 裁 **判为通过**（环境分类问题、非计算任务）；其 pid/用途/util/显存已记于此 |

## 2. 三点基线与 floor

07:33:32 采样（间隔 2.5 s）：

| 点 | free MiB | util | temp |
| --- | --- | --- | --- |
| 1 | 14754 | 2% | 28°C |
| 2 | 14754 | 2% | 28°C |
| 3 | 14758 | 2% | 28°C |

- `floor = max(min_free − 3072, 已签更严值)` = max(14754−3072, 9795) = **11682 MiB**。
- 说明：群里留痕的 floor 是 11618（基于更早一次 14690 的采样）；按「取严者生效」，本窗口实际采用 **11682**（更严 64 MiB），**没有放松**。

## 3. monitor（通过）

`monitor-until-clean.ps1 -Required 60 -MaxMinutes 30`：
`samples=60 longest_clean=60 ready=True util_min=2 util_max=4 free_min=14824`
→ 60/60 连续干净、free_min 14824 ≥ floor 11682（人工逐点核）、util 全程 ≤4% < 40。原始 CSV：`attempts/attempt-1-monitor-pre-window.csv`。

## 4. 冻结（通过，含一处工具缺陷修复）

`freeze-guards-r06-s3b-flex.ps1 -Attempt 1`：

- **首次调用失败（缺陷 D1）**：PS 5.1 把 `cargo` 的 stderr 当 error record，脚本全局 `$ErrorActionPreference='Stop'` → `Resolve-Exe` 在 `cargo build` 处直接终止。修法（**不改任何阈值/判据**）：仅在 cargo resolve 调用段临时 `$ErrorActionPreference='Continue'`，退出码检查仍是唯一失败闸。
- 修复后：`GUARDS_OK=15/15 suite=render-r06-s3b-flex exes=2 inputs=13 stash=target/render-bvh/r06-s3b-flex-window/frozen-exes aggregate_sha256=2da63012… guards_sha256=d841b638… bytes=7146`。
- 冻结前先预热 `cargo build --bin mjwarp-rs` 与 `cargo test --no-run --test render_flex_validate`（缓存命中 0.4 s，**窗口内零编译**）。

## 5. guards 断言（缺陷 D2 + 等价独立核验）

`freeze-guards-r06-s3b-flex.ps1 -GuardsOnly` 只适用于**冻结前**（它断言每条 exe `pending_freeze==true`），真冻结后再跑该开关会抛 `exe entry not marked pending`；`run-window.ps1 -GuardsOnly` 则硬编码 R05 suite（73 条目），不适用本 suite。
因此改用**等价独立核验**（逐项手算）：
- live exe sha == guards sha：`mjwarp-rs` ✅、`render_flex_validate` ✅（2/2）；
- stash 副本 sha == guards sha：2/2；
- 输入条目真 sha/bytes：13/13。
→ 守卫绑定成立。**D2 属工具可用性缺陷，已记录待修，不影响本轮守卫有效性。**

## 6. 实体体（中止：INVALID_ROUND）

命令（PATH 前置 `target/toolchains/cuda-12.8.1/bin`）：
`cargo test --locked --features cuda-probe --test render_flex_validate -- --ignored --test-threads=1 --nocapture`

| 次 | wall | 结果 | 原因 |
| --- | --- | --- | --- |
| 1 | 0.4 s | FAILED（未进入渲染） | `context 创建: Owner(Transfer(Backend(NvrtcUnavailable)))` —— 未把 CUDA bin 放进子进程 PATH（**缺陷 D3a**，非数值问题） |
| 2 | 2.0 s | FAILED（未进入渲染） | `panicked at tests/render_flex_validate.rs:56`，调用链 `:294 ints(model,"flex_edge")` —— `model-312.json` 的 17 个 flex 数组里**没有 `flex_edge`**（只有 `flex_elem`/`flex_shell` 等），`values()` 取不到 `arrays.flex_edge.values`（**缺陷 D3b**，用例作者缺陷） |

**结论：INVALID_ROUND。** 三台相机（`cam-flex`/`cam-persp`/`cam-track`）与 rgb/depth/seg 判据**一次都没有被评估**，`render_flex_validate` 在模型输入构造阶段即中止。因此本记录**不能**用于任何方向的验收结论（既非 pass 也非 fail）。

## 7. 中止后处置

- 依窗口纪律「失败即停、不重试、不重冻到过」：**未做第二次实体尝试**，GPU 已空闲（util 2%），窗口未产出参考数据。
- 本轮 artifacts 原文留档：`attempts/attempt-1-{monitor-pre-window.csv,entity-flex.log,guards-frozen.json}`。
- 修复需求（**不在本窗口内做**）：
  - **D3b（阻断，owner Backend）**：`tests/render_flex_validate.rs` 在 `--no-run` 阶段从未真正执行；需按 `model-312.json` 的实际 schema 取 flex 边（`flex_edge` 不在 dump 内，应改从 `flex_elem` 派生或从 `model_315`/provenance 取），并**在 CPU 上先自证取值**；同时核查 `flexedge_j_rownnz: vec![1; len/2]` 这类硬编码。
  - **D3a（环境，owner 窗口卡）**：执行器必须把 `-CudaBin` 前置进 PATH（run-window.ps1 已有先例）。
  - **D1/D2（工具，owner 本卡）**：freeze 脚本 EAP 修复已随本轮提交；`-GuardsOnly` 语义需区分「冻结前结构自检」与「冻结后校验」，建议冻结后校验交给开窗卡并复用独立核验步骤。
- 修复 + 纳入后需**重新真冻结**（exe 会因 rebuild 变哈希）再开新 attempt；按 Chief 2026-11-04 指令，若 08:50 前无法收干净则顺延到 12:00 空闲块。

## 8. 非声明

不宣称 flex 渲染通过/不通过；不放宽任何判据/容差；不改参考、不改产品代码；`nbuffer` 版本差异（B 类，`b94698e5`）不参与本窗口判据。
