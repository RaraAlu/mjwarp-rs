# G2/G6 参考改用 GPU 后端重采（device=cuda，融合语义）

- 任务：`2026-10-25-g10-p5-g2-gpu-reference-capture`（Backend，p55）
- 状态：**GPU 窗口 attempt-3 已完成（1 次窗口、两轮独立捕获 raw sha 相同）**，新参考已产出；
  **待 Arch 审阅 → gate/commit → QA 独立复核**；本片未 gate、未 commit。
- HEAD：`209b10397e522992f94e3eb220333a954a85096c`（采集时；采集期间主树 `src/` 干净，
  fix 片改动已由 Arch 导出为 `target/fix-fma-helpers.patch` 并移出工作树）
- 范围：本证据目录 + 新参考 `docs/development-evidence/g10-p5-gjk-epa/upstream-mjwarp-f32-gpu.json`
- 性质：authoring 采集（不进产品路径、产品测试不跑 Python）；不改产品代码、不改 `case_compare.rs`、
  不改判据/容差/`gates.toml`/G01；旧 CPU 参考与其脚本**不删不改**（归档保留）。

## 1. 不变量（本片红线）

1. 旧证据逐字节不变：`upstream-mjwarp-f32.json`（sha256 `a00c722892df5f190096f2c89e39455cd43df3be8e3fe46869b09c1f6d20f04f`）
   与 `capture_mjwarp_f32.py`（sha256 `1edd7c8e5d2c7ec7ba7cb9eb218e4533e4f7fdb77c435de1107ad4086c624312`）；
   跑前/跑后 sha 由窗口脚本记录，`git status -- docs/development-evidence/g10-p5-gjk-epa fixtures/g10-p5` 保持干净。
2. 同上游、同输入：`upstream_revision=71da24d956378a87a703b6e1442b13aec0c4ac29`；5 case 的 qpos/qpos_bits/
   xml_sha256 与旧参考逐位相同（见 §3）。
3. 新参考只新增：`kind=g2-upstream-mjwarp-f32-gpu-reference`、`provenance.device=cuda` + GPU/驱动/算力信息。
4. 不宣称精度达标/跨环境全绿/上游完全等价；本片只交付「GPU 参考已重采且有 provenance」这一事实；
   「旧参考作废」要等 fix 片切换判据后才生效。

## 2. 派生脚本与 diff 说明

- 旧脚本：`docs/development-evidence/g10-p5-gjk-epa/capture_mjwarp_f32.py`（sha256 `1edd7c8e…6c624312`）
- 新脚本：本目录 `capture_mjwarp_f32_gpu.py`（sha256 `91d62fd8dc1b99aba2e2e4bbf5356c8bfc2dbbfb415894f6a9187999b6f100a1`）
- 完整 diff：本目录 `script-diff.txt`（`git diff --no-index` 原文，5956 B）。
- 允许的差异（全部逐条对应 acceptance 白名单）：
  1. `wp.set_device("cpu")` → `wp.set_device("cuda")`；
  2. 输出文件名：`upstream-mjwarp-f32.json` → `upstream-mjwarp-f32-gpu.json`；输出路径改为
     `g10-p5-gjk-epa/upstream-mjwarp-f32-gpu.json`（CLI 第 2 参可覆盖，便于两轮各自落盘）；
  3. `task_id`/`kind`/`purpose` 文案；
  4. provenance 增加 GPU 设备信息（`nvidia-smi` 查询 name/memory.total/driver_version/compute_cap）、
     `captured_at`（环境变量固定、两轮同值）、`kernel_cache_root`；
  5. kernel cache 目录：`warp-cache-cpu` → `warp-cache-gpu/<round>`（CLI 第 1 参，两轮目录独立）。
- **必须逐字不变**（由 `verify_inputs.py` 用文本块比对，实测全部一致）：
  `CASES` 块、`FLOAT_FIELDS`、`INT_FIELDS`、`capture()` 函数体（两次捕获 + 17 字段 + `geom_xpos/geom_xmat`）、
  `load_qpos()`。
- 其他辅助新增：`import os/subprocess/sys`、`sys.stdout` UTF-8 重配置、`gpu_info()` 小函数（仅 provenance 用）。
  这些属上述白名单项的实现手段，不改变采集/输出语义。

## 3. 同输入契约（`inputs-vs-old.json`；`verify_inputs.py` 实测 exit 0）

| case | count | qpos_len | xml_sha256（与文件一致） | qpos_bits 与夹具一致 |
| --- | --- | --- | --- | --- |
| cylinder-box | 8 | 14 | `6784fe2a…1b1b7c` | true |
| ellipsoid-box | 2 | 14 | `a6ec91d3…ce62d5c` | true |
| mesh-mesh | 2 | 14 | `d38a9cab…90a0d797` | true |
| convex-mesh | 2 | 14 | `b48f6b78…7fbe7e30` | true |
| degenerate-mesh | 2 | 14 | `44ebc732…3b40cf82` | true |

`upstream_revision` = `71da24d956378a87a703b6e1442b13aec0c4ac29`；旧参考每 case
`two_captures_bitwise_identical=true`。脚本块比对（CASES/FLOAT_FIELDS/INT_FIELDS/capture/load_qpos）
逐字一致（`verify_inputs.py` 输出见 `logs/`）。

## 4. GPU 窗口计划（窗口数 = 1）

预窗口基线（`logs/window-20261026-012557-pre.txt`，nvidia-smi 只读；已报 Arch 批准）：
RTX 4070 Ti SUPER 16376 MiB；used **4148** / free **11914** MiB、util 39%、29 °C、driver 596.36、sm 8.9；
`query-compute-apps` 35 条全部为桌面/系统上下文（无 CUDA 计算进程）。
实际窗口 attempt-3 的 pre 快照：used **4078** / free **11984** MiB、util 41%、29 °C（同一桌面基线）。

步骤（`run_capture_gpu.ps1 -Execute`）：

1. 预条件快照 + 守卫 baseline（`nvidia-smi` 只读）；
2. 0.25 s 采样开始（used/free/util/temp；守卫 v2：free<10240 MiB、used 增量≥3072 MiB、
   新增外来 GPU 上下文判定；本任务 own exe = `python.exe`；桌面上下文只记录）；
3. preflight（python：版本 + `warp.get_cuda_devices()`），`-OwnGpuStepActive` 下断言守卫；
4. round1：独立进程 + `target/g10-p5-g2/warp-cache-gpu/round1` 缓存 → `rounds/round1-raw.json`；
5. round2：独立进程 + `.../warp-cache-gpu/round2` 缓存 → `rounds/round2-raw.json`；
6. 停采样、post 快照（含 `query-compute-apps` 原文）、守卫收尾（峰值/残留 pid）；
7. CPU 侧后处理：两轮 sha256、行序规范化（`canonicalize.py`）、输入契约复验、旧 CPU vs 新 GPU 对照表
   （`compare_references.py`），最后写 `capture-meta.json`。

预计：preflight ≈5 s；每轮含 warp CUDA kernel 的 CPU 侧 JIT（NVRTC）≈20–40 s / 10–30 s，采集本身 <1 s；
两轮合计 **≈60–120 s**；GPU 计算占用 <3 s（kernel 启动），显存增量估计 **<800 MiB**
（CUDA context + 小数组）；守卫下限 free≥10240 MiB（当前余量约 1674 MiB）、增量<3072 MiB。
同一窗口内串行两轮，窗口内不编译其它任务、不改代码。

### 4.1 窗口尝试记录（attempt-1 / attempt-2 作废，不拼接）

- **attempt-1**（2026-10-26T01:27:50Z）：在 **guard preconditions** 即中止。事实：共享守卫模板
  `scripts/gpu-window-guard.ps1` 的 `Get-GpuSnapshot` 用带单位 CSV（本机 `4165 MiB, 11897 MiB, 38 %`）
  后直接 `[int]$parts[0]`，类型转换失败抛错 → 守卫中止。**零 GPU 工作**（无 context/preflight/采集），
  详见 `logs/window-attempt-1-aborted.txt`。
  修复方式：runner 在 dot-source 后**局部覆盖** `Get-GpuSnapshot`，实现为
  `nvidia-smi --query-gpu=memory.used,memory.free,utilization.gpu --format=csv,noheader,nounits`
  的整数解析；**guard v2 的阈值/中止规则/日志格式未被放宽**（free<10240、增量≥3072、
  外来 GPU 上下文、util 连续 3 次≥80% 全部保留）；共享模板文件本身**未修改**（模板 bug 由 Arch 另立治理片）。
  另加 300 s wall 看门狗（超时 abort）与 abort 元数据/exit 8；abort 路径也会落 post 快照与
  `query-compute-apps`。
- **attempt-2**（2026-10-26T01:29:11Z）：在 **preflight 步骤**中止（guard preconditions 已通过）。
  事实：preflight 以 `python -c` 传内嵌双引号脚本文本，PowerShell 5.1 向原生 exe 传参时破坏引号，
  Python 收到破损脚本 → `File "<string>", line 1`；**零 GPU 工作**（import warp 之前即失败，
  guard baseline=peak=4078 MiB、delta 0、rounds 空、无 final、无残留）。详见
  `logs/window-attempt-2-aborted.txt`。修复方式：preflight 改为独立文件 `preflight_gpu.py`（文件参数），
  `Run-Python` 异常也写日志；采集脚本与采集逻辑未变。
- 两次作废轮均**不拼接、不进任何统计**；attempt-3 为重开的有效窗口轮（窗口数仍为 1）。

### 4.2 窗口执行结果（attempt-3，`window-20261026-012956`，有效轮）

- 执行：`run_capture_gpu.ps1 -Execute`（2026-10-26T01:29:56Z 起；`captured_at`=2026-10-26T01:29:57Z）。
- 耗时（logs 实测）：preflight **1.24 s**、round1 **28.08 s**（含 warp CUDA kernel 首次 JIT）、
  round2 **4.47 s**（独立进程 + 独立空 cache）、guard 收尾至 post 快照约 35 s 总 GPU 段。
- 采样（0.25 s × 122 点）：used 峰值 **4417 MiB**（基线 4078 → 增量 **+339 MiB**）、
  free 最低 **11645 MiB**（≥10240 下限）、util 最高 **80%**、温度最高 **30 °C**。
- guard v2：基线 used/free = 4078/11984 MiB、guard_samples=4、residual_context_pids 空、
  未触发任何 abort 规则（窗口内 own step 标记下的 util 尖峰不触发；无外来 GPU 上下文）。
- post 快照：`NVIDIA GeForce RTX 4070 Ti SUPER, 16376 MiB, 4071 MiB, 11991 MiB, 18 %, 30, 596.36, 8.9`；
  `query-compute-apps` 仍 **35 条**（与前快照同集合，无本任务残留 CUDA 进程）。
- **两轮独立进程 + 两个独立 kernel cache 的 raw 文件整文件 sha256 相同**：
  `d86c73cb04fef7f777afabf7d1291a8abaf0d744c41bdad277bd42350c8c87aa`（round1 = round2）；
  规范化后 canonical sha 亦相同（`10956ae0…346702`）。
- 每 case `two_captures_bitwise_identical = true`（5/5：cylinder-box、ellipsoid-box、mesh-mesh、
  convex-mesh、degenerate-mesh）；count = 8/2/2/2/2。
- 后处理说明：runner 在 guard 收尾后因 PowerShell 5.1 把原生 stderr 当终止错误而中止（**采集已完成**），
  故用**纯 CPU** 的 `finalize_capture.ps1` 从已保存的两轮文件完成 sha/规范化/输入复验/对照/最终产物落盘
  （`capture-meta.json` 经 `kv_to_json.py` 生成，规避 PS 5.1 ConvertTo-Json 的嵌套字典缺陷）；
  该轮 GPU 只使用一次，未重开窗口。

### 4.3 旧 CPU 参考 vs 新 GPU 参考对照（只描述，不宣称达标）

旧参考 = `upstream-mjwarp-f32.json`（warp CPU，两次舍入）；新参考 = `upstream-mjwarp-f32-gpu.json`
（warp CUDA，融合语义）。`compare-cpu-vs-gpu.json` / `logs/window-20261026-012956-compare.txt`：

| case | count | 整型行 | 浮点位等分量 | frame max_abs | dist max_abs | pos max_abs | geom_xpos 位等 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| cylinder-box | 8/8 | 8/8 | 227/232 | **0** | 0 | 1.490e-8 | 12/12 |
| ellipsoid-box | 2/2 | 2/2 | 42/58 | **8.193e-7** | 1.863e-9 | 7.451e-9 | 12/12 |
| mesh-mesh | 2/2 | 2/2 | 39/58 | 5.364e-7 | 9.313e-9 | 7.451e-9 | 12/12 |
| convex-mesh | 2/2 | 2/2 | 35/58 | 2.980e-7 | 1.490e-8 | 1.118e-8 | 12/12 |
| degenerate-mesh | 2/2 | 2/2 | 52/58 | 2.651e-11 | 1.048e-8 | 1.863e-9 | 10/12 |

- `count`/`dim`/`geom`/`worldid`/`type`/`geomcollisionid` 等整型字段**全部一致**（结构一致、
  `structural_fail=false`、`input_fail=false`），未出现结构差异。
- 浮点差仅低 ULP 级：frame 最大 **8.193e-7**（= 上游 CPU/GPU 自身差量级，G5 证据已记录 8.19e-7），
  dist/pos ≤ 1.9e-8；`geom_xpos` 仅 degenerate-mesh 有 2/12 分量差（同一 FMA 量级）。
- 与「GPU 融合语义差」一致；无 count/整型/结构差异 → 不触发停车条件。

## 5. 两轮一致性与停车条件（实测口径）

- 判定顺序：① 两轮 **raw 文件 sha256 相同** → `status=ok`（最终产物 = round1 raw）；
- 若 raw 不同但**行序规范化后 canonical sha256 相同** → `status=stop_raw_sha_differs_canonical_equal_needs_decision`
  （exit 7）：**不写最终产物**，两轮 raw/canonical 与对照表全部保留，报 Arch 决策
  （warp GPU 接触数组用 atomic 计数，raw 行序可能随进程/调度变化；下游 `sort_contacts` 本来就重排，
  是否接受 canonical 由 Arch/Chief 定；若接受只需 CPU 侧一次复制，不必重开窗口）；
- 若 canonical 仍不同 → `status=stop_two_rounds_differ`（exit 3），不写最终产物；
- 若任一 case `two_captures_bitwise_identical=false` → `status=stop_two_captures_false`（exit 6）；
- 若输入契约不符（exit 4）或与旧 CPU 参考出现 count/整型字段结构差异（exit 5）→ 停手报 Arch
  （说明 GPU recipe 不是简单融合，B 前提要重估）。
- 浮点字段差异只记录不判失败：`compare-cpu-vs-gpu.json` 给每字段位等分量数、max_abs、max ULP。

## 6. 产物清单（attempt-3 已填 sha）

- 新参考：`docs/development-evidence/g10-p5-gjk-epa/upstream-mjwarp-f32-gpu.json`
  sha256 `d86c73cb04fef7f777afabf7d1291a8abaf0d744c41bdad277bd42350c8c87aa`（14330 B，来自 round1-raw）
- 两轮原始：`rounds/round1-raw.json`、`rounds/round2-raw.json`（两者同 sha256
  `d86c73cb…c87aa`）
- 规范化：`rounds/round1-canonical.json`、`rounds/round2-canonical.json`（同 sha256
  `10956ae0c113bd86541c96fdf7dedfcdd3408fb8a70d3bec5b45e8fb29846702`）
- 窗口记录：`capture-meta.json`、`logs/window-20261026-012956-{pre,post}.txt`、`...-samples.txt`、
  `...-guard.log`、`...-preflight.txt`、`...-round1.txt`、`...-round2.txt`、`...-git.txt`、
  `...-verify-inputs.txt`、`...-compare.txt`、`...-canonicalize{1,2}.txt`、`...-meta-json.txt`、
  `logs/window-attempt-{1,2}-aborted.txt`
- 对照表：`compare-cpu-vs-gpu.json`；输入契约：`inputs-vs-old.json`；脚本 diff：`script-diff.txt`
- 采集/后处理脚本：`capture_mjwarp_f32_gpu.py`、`preflight_gpu.py`、`run_capture_gpu.ps1`、
  `finalize_capture.ps1`、`canonicalize.py`、`compare_references.py`、`verify_inputs.py`、`kv_to_json.py`
- 旧证据未变（跑前/跑后同 sha）：`upstream-mjwarp-f32.json` = `a00c7228…20f04f`、
  `capture_mjwarp_f32.py` = `1edd7c8e…6c624312`；`git status -- docs/development-evidence/g10-p5-gjk-epa` 仅
  新增 GPU 参考文件，旧文件无改动。

## 7. CPU 侧复现（已跑通）

```powershell
$py = 'target\mjwarp-parallel\venv\Scripts\python.exe'
& $py -B docs/development-evidence/g10-p5-g2-gpu-reference-capture/verify_inputs.py
& docs/development-evidence/g10-p5-g2-gpu-reference-capture/run_capture_gpu.ps1   # 计划模式，不建 GPU 上下文
```

窗口执行（已执行，attempt-3）：`& docs/development-evidence/g10-p5-g2-gpu-reference-capture/run_capture_gpu.ps1 -Execute`

窗口后 CPU 侧定稿（本例因 runner 后处理中止而单独执行；**不重开窗口**）：
`& docs/development-evidence/g10-p5-g2-gpu-reference-capture/finalize_capture.ps1 -Stamp 20261026-012956`

## 8. 口径红线

真实 GPU 只在本窗口使用一次；不宣称精度达标/跨环境全绿/上游完全等价；本片不动 `src/`、
不改固定判据与容差、不改旧证据语义；「旧参考作废」由 fix 片切换判据后才生效。
