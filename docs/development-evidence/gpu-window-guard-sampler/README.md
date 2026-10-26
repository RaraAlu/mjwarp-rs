# GPU 窗口守卫采样器修复（2026-10-26）

任务：`2026-10-26-gpu-window-guard-sampler`（p40，owner Backend；纯 CPU，**GPU 窗口 0**，不开窗）。

## 1. 修的两个缺陷（stage-2 attempt4 暴露）

1. **采样器未落盘 → guard 未生效**：stage-2 窗口中 runner 用 `Start-Job` 起 0.25 s 采样器，结果
   `samples=0`（`samples.csv` 仅表头），guard v2 实际未消费任何样本，只剩 pre/post 快照与逐步骤 wall。
   事实见 `docs/development-evidence/g10-p5-cuda-default-pipeline-stage2/window-attempt4-and-b-classification.md`
   §2 偏差 ①。本片用**真实子进程 + 逐样本立即 close/flush + Stop join（超时保护）**替代 `Start-Job`。
2. **带单位 CSV 解析缺陷**：`Get-GpuSnapshot` 对 `nvidia-smi` 默认输出 `4165 MiB, 11875 MiB, 38 %`
   过滤后直接 `[int]`，会抛「无法将值 4165 MiB 转换为 Int32」；capture 阶段被迫在 runner 内局部覆盖才跑通（记录：docs/development-evidence/g10-p5-g2-gpu-reference-capture/README.md §4.1）。
   本片改为 `ConvertFrom-GpuCsvLine`：剥单位/空格，两种输入（带单位与纯数字）都返回正确整数；
   缺列/非数字给明确 `GUARD_ABORT`（非 `Int32` 转换异常）。

## 2. 改动（`scripts/`，不改阈值/中止规则/日志格式）

- `scripts/gpu-window-guard.ps1`
  - 新增 `ConvertFrom-GpuCsvLine`（两种输入、明确 abort）；`Get-GpuSnapshot` 复用它。
  - 新增可复用 `Start-GuardSampler` / `Stop-GuardSampler`（0.25 s 采 used/free/util/温度，逐样本
    `[IO.File]::AppendAllText` 立即 flush 写 `samples.csv`，列 `seq,timestamp,used_mib,free_mib,util_percent,temperature_c`；
    Stop 写 stop-file 后 `WaitForExit(timeout)`，超时 kill，再统计；返回 `samples/used_peak_mib/free_min_mib/
    util_max_percent/first_ts/last_ts/process_exited/residual_processes`）。
  - 新增 `Resolve-GuardPwsh`（`pwsh` 不在 PATH 时回退 `target/toolchains/powershell-*/bin/pwsh.exe`）。
  - `Assert-Guard` 增加**注入 seam** `-Snapshot` / `-Apps`（省略时行为与原来完全一致，仍走 live nvidia-smi），
    供纯 CPU 注入序列验证 guard 规则；阈值、中止规则、日志格式未动。
- `scripts/gpu-guard-sampler-loop.ps1`（新）：子进程采样循环，每样本立即 flush，stop-file 退出。
- `scripts/gpu-window-guard-dryrun.ps1`（新）：纯 CPU dry-run（见 §3）。

## 3. dry-run（纯 CPU；只用只读 `nvidia-smi`，不建 CUDA context、不做 GPU 计算）

命令：`pwsh -NoProfile -File scripts/gpu-window-guard-dryrun.ps1`（本机 pwsh 7 路径见
`target/toolchains/powershell-7.6.7/bin/pwsh.exe`）。结果：**DRYRUN RESULT: PASS**，19 项断言全 PASS（脚本自报 ASSERTIONS total=19 pass=19）。

| 组 | 断言（原始日志） |
| --- | --- |
| 解析 | `4165 MiB, 11875 MiB, 38 %` 与 `4165, 11875, 38` 都→ 4165/11875/38；`N/A` 列 → `GUARD_ABORT`（`logs/parser-dryrun.log`） |
| 真实采样 ≥3 s | `samples.csv` 11 行 = 返回 11 样本（≥10）；seq=1..11；时间戳单调；字段可解析；峰值 4199 / free 最低 11863 / util 最高 40 与 CSV 独立重算一致（以 committed logs/samples.csv 独立重算：11 行）；进程 exited；无残留进程（`logs/sampler-dryrun.log`、`logs/samples.csv`） |
| guard 真生效（注入序列） | 正常序列 30/40/30 → 不 abort；util 85/90/88 连续 ≥3 点 ≥80% 且无本任务 GPU 步 → `GUARD_ABORT`；同样 util 但 `-OwnGpuStepActive` → 不 abort；free=10000<10240 → abort；used 4165→7400（Δ3235≥3072）→ abort（`logs/injection-dryrun.log` 与 `logs/inject-*.log`） |

- dry-run 默认输出目录为 	arget/gpu-window-guard-sampler-dryrun/<timestamp>（不默认覆盖已入库证据）；本证据中的 logs/ 来自修复轮显式 -OutDir target/gpu-window-guard-sampler-dryrun/fix1 后**显式复制**建档。\n- Resolve-GuardPwsh 按 $PSScriptRoot 推导 repo 根后查找 	arget/toolchains/powershell-*/bin/pwsh.exe；已在 **PS 5.1** 下实测返回该路径（找不到才回退 powershell.exe 并写 guard_pwsh_fallback 警告）。\n\n## 4. 边界

- 只读 `nvidia-smi` 查询；不建 CUDA context、不跑 GPU 计算、不占窗口。
- 阈值/中止规则/日志格式**未放宽或改动**（free<10240、增量≥3072 且取 3072 与总显存一半的更严者、
  util 连续 3 点 ≥80% 且无本任务 GPU 步）；未动产品代码/判据/容差/`gates.toml` 的 GPU 门禁定义。
- 本片不宣称跨环境全绿；guard 的「GPU 步」判定仍由 runner 的 `-OwnGpuStepActive` 显式传入（与 v2 相同）。