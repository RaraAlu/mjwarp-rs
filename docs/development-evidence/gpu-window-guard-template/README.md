# GPU 窗口守卫模板（`scripts/gpu-window-guard.ps1`）

- 任务：`2026-10-25-gpu-window-guard-template`
- 背景：2026-10-25 QA 的 faithful 影响面窗口（attempt-1）被**执行者自己的过严守卫**误中止——守卫把
  另一执行者的**纯 CPU 测试二进制**（`mjwarp_rs-bbf617…`，不出现在 `nvidia-smi --query-compute-apps`）
  当成「外部计算进程」而停窗。该轮 57 s 内完成 2/20（launcher 314/3122/80、primitive 80/80×3，全 0 ULP），
  守卫原因与采样点已入 `faithful-impact-measurement/logs/window-attempt-1-aborted.txt`。
- 结论：守卫的对象应是 **GPU 资源占用**（排他 CUDA 计算 + 显存），不是「进程存在」；纯 CPU 进程不占 GPU。

## 1. 新守卫口径（本模板实现）

**中止条件（任一成立即停窗）**
1. `nvidia-smi --query-compute-apps` 中**新增**且进程名不匹配本任务允许模式（默认 `g10_p5_*`、
   `render_*`、`mjwarp_rs-*`；基线 pid 恒允许）的 CUDA 计算进程；
2. free VRAM < **10240 MiB**；
3. used 相对基线增量 ≥ **3072 MiB** 或 > 总显存/2（**取更严**，本机即 3072 MiB）。

**只记录、不中止**：纯 CPU 进程（cargo/rustc/测试 exe 且无 CUDA 上下文）——它们不出现在
`query-compute-apps`，不占 GPU；窗口日志仍记录其存在以便回溯。

**其余纪律不变**：0.25 s 采样（used/free）、跑前跑后 `query-compute-apps` 原文、结束基线与残留 CUDA pid、
开窗前报 Arch 基线、同一时间只一个 GPU 满载任务、窗口内不编译/不改代码/不新捕获。

## 2. 用法（dot-source）

```powershell
. scripts/gpu-window-guard.ps1
$state = New-GuardState -Label 'my-window' -TotalMib 16376 `
    -AllowedExePatterns @('g10_p5_*','render_*','mjwarp_rs-*') `
    -FreeFloorMib 10240 -IncrementLimitMib 3072
Assert-Guard -State $state -Phase 'preconditions'
# ... run one measured step ...
Assert-Guard -State $state -Phase 'after-step-1'
# ... more steps ...
Complete-Guard -State $state
```

日志行格式：`guard_init` / `guard_baseline` / `guard_sample(phase,n,used,free,delta,cuda_pids)` /
`guard_stop(reason,...)` / `guard_end(peak,samples,residual,aborted)`；脚本不写仓库，只写调用方指定的
日志文件（UTF-8 无 BOM）。

## 3. 中止轮的处置口径

- **中止轮结果作废、不拼接**：如 attempt-1 的 2/20 只作「守卫误触记录」，**不进影响矩阵**；
- 重开窗口**整轮重跑**（本例 20 项 ~7 min，仍在上限 10 min 内），保持同一口径一致性；
- 中止轮与重跑轮的事实分别入窗口日志，摘要里注明「第一次 aborted（原因/时长/已跑项）」。

## 4. 边界

- 本模板只定义「何时该停窗」，不改任何验收口径/判据/容差；
- 既有各片 `run_window.ps1` 保留其历史证据语义，不回溯修改；后续新窗口默认引用本模板；
- 脚本纯 ASCII、PS 5.1 语法解析 0 错（本次验证），≤500 行（147 行）；
- 未改产品代码、未改 `gates.toml`/G01 脚本；未宣称任何验收通过。
