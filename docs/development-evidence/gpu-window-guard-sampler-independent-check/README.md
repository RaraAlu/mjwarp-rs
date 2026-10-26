# GPU 窗口守卫采样器修复 · QA 独立复核

- 任务：`2026-10-26-gpu-window-guard-sampler-independent-check`（p39，owner QA）
- 复核对象：`2026-10-26-gpu-window-guard-sampler` 交付，提交 `c6531982`（父 `627b989d`）
- 复核 HEAD：`c6531982c43af1445979ce014493c7cfe8b2c214`
- 结论：**pass（阻断 0 / 观察 8）**。纯 CPU 复核：只用只读 `nvidia-smi`，**未开 GPU 窗口、未建 CUDA context、未做 GPU 计算**；未改产品代码/判据/容差/`gates.toml`；未 commit、未 done。
- 本目录只写 QA 自己的证据；除注明「读交付」外，所有数字都是我自己重跑/自算。

## 0. 结论速览

1. **带单位解析成立**：`4165 MiB, 11875 MiB, 38 %` 与 `4165,11875,38` 都返回 4165/11875/38；缺列 / `N/A` / 非数字 / 小数 / 负数一律明确 `GUARD_ABORT`，**没有 Int32 转换异常**；旧实现（父提交）在同一行上会抛 `Cannot convert value "4194 MiB" to type "System.Int32"`，已复现（正是 capture attempt-1 的中止原因）。
2. **阈值未放宽**：`free<10240`、`increment>=min(3072,total/2)`、`util 连续 3 点 >=80% 且无本任务 GPU 步` 与 v2 完全一致；diff 里**没有任何阈值/中止规则/日志格式行被删改**，默认值实测 10240/3072/80/3。
3. **guard 真生效**（自构造注入序列，不依赖交付脚本）：正常 30/40/30 不 abort；util 80/90/88 连续 3 点无 own-step → `GUARD_ABORT`（日志 `reason=sustained_util_without_own_step`）；同样 util 加 `-OwnGpuStepActive` → 不 abort；free 10239 → abort（10240 不 abort）；增量 3072 → abort（3071 不 abort；`total=4000` 时更严的半值 2000 生效）；外来 GPU 上下文 → abort、桌面上下文只观察。
4. **采样落盘成立（核心）**：dot-source 模板起采样器跑 ≥3 s，**pwsh 7.6.7 与 Windows PowerShell 5.1 各 13 个样本**；`samples.csv` 行数=返回样本数、seq=1..n、时间戳单调、字段可解析；峰值/最低/最高与 CSV 独立重算一致；Stop 后进程退出、无采样循环残留、stop-file 已删、CSV 可独占打开（无泄漏句柄）；**Stop 前 1.4 s 已有 4–5 行落盘**（逐样本立即 flush 的直接证据）。
5. **范围干净**：`627b989d..c6531982` 只改 3 个 `scripts/` 文件 + 11 个证据文件 + `.normify`（任务 JSON + 4 个门禁 runs）；没有产品/测试/判据/容差/`gates.toml`/fixtures 改动；交付证据 11 文件、0 BOM、0 超 500 行、0 二进制；没有窗口日志或 CUDA API 引用（改动脚本对 `cudarc/CudaContext/nvrtc` 0 引用，仅注释里提到 Start-Job/示例 exe 名）。
6. **触发依据成立**：交付 README 引用了 stage-2 attempt4 `samples=0` 的窗口文档路径；我核了本机原始产物（`facts.txt` 有 `samples=0`、`samples.csv` 仅 26 字节表头）；capture 阶段「`[int]` 转换中止 + runner 局部覆盖」记录在 G2 capture README §4.1，内容与我的复现一致。
7. **未发现抓凑绿**：guard 规则代码未被改，dry-run 只是新增验证脚本；我的独立注入测试证明阈值真的会 abort。

## 1. 方法与隔离

- 自写脚本 `scripts/qa-guard-check.ps1`：dot-source `scripts/gpu-window-guard.ps1`，覆盖解析、阈值边界、注入判定、真实采样、resolver；分别在 **pwsh 7.6.7**（`target/toolchains/powershell-7.6.7/bin/pwsh.exe`）与 **Windows PowerShell 5.1** 下各跑一次，两次都是 **0 FAIL**。
- 自写脚本 `scripts/trigger-repro.ps1`：复现旧解析缺陷、Start-Job 采样器 0 行症状、复核 stage-2 原始产物与 capture 记录。
- 环境：Windows 11 `10.0.26200`；active Rust 工具链与本片无关（脚本改动，未跑 cargo）；GPU 窗口 0。
- 所有采样 CSV、注入日志、复现日志都写在 `target/qa-guard/`，**不写交付证据目录**。

## 2. 带单位解析（含旧缺陷复现）

| 输入 | 结果 |
| --- | --- |
| `4165 MiB, 11875 MiB, 38 %` | 4165 / 11875 / 38 |
| `4165,11875,38` | 4165 / 11875 / 38 |
| `4165 MiB, 11875 MiB, 38 %, 33`（多温度列） | 4165 / 11875 / 38（取前 3 列） |
| `4165 MiB, 11875 MiB`（缺列） | `GUARD_ABORT: cannot parse nvidia-smi line (need >=3 columns)` |
| `4165 MiB, N/A MiB, 38 %` | `GUARD_ABORT: ... column 2 ...` |
| `abc, def, ghi` / `4165.5 MiB,...` / `-1 MiB,...` | `GUARD_ABORT`（列 1/2，非 Int32 异常） |
| 真实 `nvidia-smi` 行（`4194 MiB, 11868 MiB, 38 %`） | 解析为 4194 / 11868 / 38；`Get-GpuSnapshot` 实测 used=4194 free=11868 |

- **旧实现复现**：在活行上跑父提交的解析逻辑 → `Cannot convert value "4194 MiB" to type "System.Int32"`，与 `g10-p5-g2-gpu-reference-capture` README §4.1 attempt-1 记录一致。
- 边界（观察 O6）：空字符串传入会触发 `Mandatory` 参数绑定错误而不是 `GUARD_ABORT`；这不在 acceptance 列举的「缺列/非数字」范围内，且本机未观察到 `nvidia-smi` 返回空行。

## 3. 阈值未放宽（diff + 边界实测）

- diff 删除行只有旧 `Get-GpuSnapshot` 的 3 行解析体 + `[switch]$OwnGpuStepActive` 参数行 + 2 行 live 取值（被 seam 取代）；**与阈值/中止规则/日志格式相关的删除行 = 0，新增阈值行 = 0**。
- 默认值实测：`free_floor_mib=10240`、`increment_limit_mib=3072`、`util_abort_percent=80`、`util_abort_consecutive=3`、`total=16376`。

| 注入场景 | 结果 |
| --- | --- |
| free 恰好 10240 | 不 abort |
| free 10239 | abort，日志 `reason=free_below_floor` |
| 增量 3071 | 不 abort |
| 增量 3072 | abort，日志 `reason=increment_over_limit` |
| `total=4000` 时增量 2000 | abort（`>= 2000`，更严的半值规则生效） |
| util 79 ×3 | 不 abort |
| util 85,85,30,85,85 | 不 abort（streak 在低点复位） |
| util 80/90/88（无 own-step） | 第 3 点 abort，日志 `reason=sustained_util_without_own_step` |
| util 95 ×3 + `-OwnGpuStepActive` | 不 abort |
| 注入新桌面上下文（chrome，白名单路径） | 不 abort（observation only） |
| 注入新外来上下文（`D:\Tools\foreign.exe`） | abort，`foreign GPU context 999999:foreign.exe:...` |

- **交付 dry-run 的一个测试缺陷（观察 O1）**：其增量用例注入 `used=7400` 时隐含 `free=8640 < 10240`，实际中止原因是 `free_below_floor`（committed `logs/inject-delta.log` 原文如此），`Expect-Abort` 只断言「发生了 GUARD_ABORT」，**所以交付 dry-run 并没有真正验证增量规则**。规则本身没问题——我用 free≥10240 的独立用例验证了 3071/3072/半值三档。

## 4. 采样落盘（核心）

| 检查项 | pwsh 7.6.7 | Windows PowerShell 5.1 |
| --- | --- | --- |
| 采样时长 / 样本数 | ~3.6 s / **13** | ~3.6 s / **13** |
| `samples.csv` 行数 = stats.samples | 13 = 13 | 13 = 13 |
| ≥10、seq=1..n、时间戳单调、字段可解析 | 全 PASS | 全 PASS |
| 峰值 / free 最低 / util 最高 与 CSV 重算一致 | 4199 / 11863 / 40 一致 | 4201 / 11861 / 40 一致 |
| Stop 前 1.4 s 已落盘行数 | 4 | 5 |
| 进程 exited / 无残留循环进程 / stop-file 删除 | 全 PASS | 全 PASS |
| stop 后 CSV 可独占打开（无泄漏句柄） | PASS | PASS |
| CSV 表头 | `seq,timestamp,used_mib,free_mib,util_percent,temperature_c` | 同 |

- 采样器是**真实子进程**（`gpu-guard-sampler-loop.ps1`），模板里**已无 `Start-Job` 调用**（只剩解释性注释）；子进程解释器路径在两壳下分别为 toolchain pwsh 与 `System32\WindowsPowerShell\v1.0\powershell.exe`（后者见观察 O4）。
- 只用只读 `nvidia-smi --query-gpu`；没有 CUDA API 调用、没有 context 创建。

## 5. 触发依据与「修前为何未生效」

- stage-2 attempt4：本机原始产物 `facts.txt` 含 `samples=0`，`samples.csv` 26 字节仅表头 `time,used,free,util,temp`；交付 README §1.1 用路径引用了 stage-2 窗口文档 §2 偏差①。
- capture 阶段：`g10-p5-g2-gpu-reference-capture/README.md` §4.1 attempt-1 记录了共享模板 `Get-GpuSnapshot` 对带单位 CSV 直接 `[int]` 抛错、guard preconditions 中止、runner 局部覆盖；我已用活行**精确复现同一异常**。交付 README §1.2 叙述了该事实但**没有给路径/行号**（观察 O7）。
- 「为什么 guard 未生效」：采样器 0 行 → guard 没有连续 used/util 样本可消费，只剩 pre/post 快照与逐步骤 wall，与 stage-2 窗口文档描述一致；我用 Start-Job 复刻采样器在 2 s 观测窗内也得到 **0 数据行**（同症状；精确机制无法钉死，因为 stage-2 的窗口 runner 未入库），修后采样器同一会话 **13 行/3.2 s**。

## 6. 改动范围与门禁

- `git diff 627b989d..c6531982`：19 文件 = 3 scripts（guard +104/−6、sampler-loop +23、dryrun +128）+ 11 证据 + `.normify`（任务 JSON + 4 runs）；**product/tests/判据/容差/`gates.toml`/fixtures 0 改动**。
- 交付证据 11 文件：0 BOM、0 超 500 行、0 二进制；最大文件 `logs/samples.csv` 656 B。
- 交付门禁 runs：fmt/clippy/clippy-cuda/unit 全 pass；`unit` 11254 ms 真跑 971/0，clippy 两项 403/454 ms 为缓存命中（本片只改 `.ps1`，Rust 未变，不影响门禁语义）。
- 我在**scratch 证据目录**重跑交付 dry-run：exit 0、**18 PASS / 0 FAIL**（交付 README/receipt 写 20 项，观察 O3）；重跑不会污染交付证据。

## 7. 观察（8 条，均不阻断）

1. **O1** 交付 dry-run 的增量用例实际因 `free_below_floor` 中止（free 8640），未验证增量≥3072 规则；建议注入时保持 free≥10240 并断言中止原因。
2. **O2** README/receipt 的采样统计 4199/11863/41 与 committed `samples.csv`/`sampler-dryrun.log` 的 4200/11862/39 不一致；dry-run 默认写交付证据 logs，重跑会静默覆盖原始日志。
3. **O3** 「20 项断言」与实际 18 项（3 解析 + 10 采样 + 5 注入）不符。
4. **O4** `Resolve-GuardPwsh` 的 toolchain 回退声明不成立：PS 5.1 下即使 `target/toolchains/.../pwsh.exe` 存在也返回 `powershell.exe`；pwsh7 下只有调用者位于 `<repo>\target\<dir>`（祖父目录=repo 根）才碰巧命中，`scripts/` 下的调用者会用同样的双重 `Split-Path` 得到 repo 的父目录。功能上无影响（loop 兼容 5.1、两壳都 13 样本），但文档应改或实现应改成从模板路径解析 repo 根。
5. **O5** 注入空 Apps（`-Apps @()`）时日志 `contexts=1`：空数组分支变 `$null`，`$null | ForEach-Object` 会执行一次产生伪 pid。abort 判定不受影响，仅日志可读性。
6. **O6** 空行输入触发 `Mandatory` 参数绑定错误而非 `GUARD_ABORT`（acceptance 未列的边界）。
7. **O7** capture 阶段局部覆盖记录未带路径/行号引用（记录在 G2 capture README §4.1）。
8. **O8** 实现片 receipt 的 `head=5bcd78f4`（早两个提交）且 `status=in_progress`，与 verified/`c6531982` 不一致。

## 8. 未覆盖 / 边界

- 未实机开 GPU 窗口；guard 在真实窗口中的在线行为未复测（本片是纯 CPU 验证）。
- 未复现 stage-2 attempt4 的精确 runner（未入库），只复现了 `samples=0` 可观测症状与新增采样器修复对比。
- 不宣称跨环境/跨壳全绿；只记录本次两壳实测。
- 未跑全量 G01；门禁仅本任务 checks 与交付 runs 核对。

## 9. 复现命令（QA 脚本在 `scripts/`）

```powershell
# 独立检查（两壳各跑一次；必须用 scratch 输出目录，避免写交付证据）
& 'D:\Rust\mjwarp-rs\target\toolchains\powershell-7.6.7\bin\pwsh.exe' -NoProfile -File scripts\qa-guard-check.ps1 -OutDir target\qa-guard\out-ps7
powershell -NoProfile -File scripts\qa-guard-check.ps1 -OutDir target\qa-guard\out-ps51

# 触发复现（旧解析缺陷 / Start-Job 0 行症状 / stage-2 原始产物 / capture 记录）
powershell -NoProfile -File scripts\trigger-repro.ps1
```

## 10. 证据清单

- `README.md`（本文件）、`receipt.json`、`artifacts-sha256.json`
- `qa-parser.json`、`qa-thresholds-guard.json`、`qa-sampler.json`、`qa-trigger.json`、`qa-range-evidence.json`、`qa-observations.json`
- `logs/`：pwsh7 与 5.1 两次独立检查输出、交付 dry-run 复跑输出、触发复现、静态检查（scope/diff/threshold/CUDA 标记）
- `scripts/`：`qa-guard-check.ps1`、`trigger-repro.ps1`、`static-check.ps1`、`startjob-probe.ps1`（Start-Job 0 行症状探针）
- 实现片本目录外的证据：`docs/development-evidence/gpu-window-guard-sampler/`（复核对象，未改动）
