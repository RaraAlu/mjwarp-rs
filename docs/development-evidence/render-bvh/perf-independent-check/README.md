# 渲染/BVH perf 独立复核（独占 GPU 相对基准 / 原始序列派生 / 显存口径 / S8 修正）

任务：`2026-10-25-render-bvh-perf-independent-check`（QA 独立复核，渲染/BVH perf）
复核对象：`eadae87`（perf 主提交，任务 `2026-10-25-render-bvh-perf` verified）+ `18ae715`（S8 复核非阻断修正，
`Normify-Fixes: s8`）；复核基线 HEAD：`8ed13d2`（只含本复核任务登记）
结论：**验收面全部成立，0 阻断**；1 条证据字段问题 + 2 条建议（非阻断）。

> 口径：本复核未跑完整 G01；性能结论只作**单场景单机相对基准**，不宣称固定性能指标、不与上游/其它硬件比较。

## 1. 方法（不读交付断言）

- **原始序列独立重算**（`judge_raw_stats.py`）：`raw-timings.json` 两遍各 30 条 `frame_ms`/`launch_ms`，
  按声明的 nearest-rank（`index = round((n-1)*fraction)`，Rust `f64::round` 语义——Python 的
  `round()` 是银行家舍入，已显式替换）重算 min/median/p95，并与交付 `official_summary`、
  README/receipt 数字逐项比较。
- **独立测量**（`qa-harness-main.rs.txt`，`target/qa-perf-harness/`，debug 构建 = 与交付测试同
  profile）：QA 自己的 driver 构造产品 camlight 位姿/context/rays，复刻同一协议
  （warmup=5、迭代=30、`frame_ms` = render 调用 + RGB 读回同步），3 个独立进程（A/B/独占采样窗口）。
- **显存独立测量**：同进程跑 2 个完整 create→render×10→destroy 周期，记录 free 序列、峰值、
  周期间残差。
- **独占窗口自有采样**：100 ms `nvidia-smi` 采样 + 每 500 ms `--query-compute-apps` 进程列表，
  覆盖 QA 自己的测量窗口。
- **S8 修正核验**：重跑 `render_e2e`（5/5）、检查断言/fixture provenance/README 措辞/数字不回归。

## 2. 核心结论

### 2.1 `official_summary` 确由原始序列派生（逐项一致）

- 两遍 × 两模式 ×（launch/frame）×（min/median/p95）= **24 个统计量独立重算与交付数字一致**
  （舍入到 1e-3 ms 后 0 mismatch，实际差 ≤1e-9）；
- pass 级 `ratios` 由 summary median 精确导出：A `2.578 / 14.030 / 9.043`、B `2.531 / 14.389 / 9.106`；
- `official_summary` 与**同一次运行**日志 `s8perf[stat]` 行逐项一致（0 差）；
- `raw-timings.window` 与 `gpu-run.log`/`repeat-run.log` 的窗口时间戳一致；warmup=5、
  每模式 30 次、两遍独立窗口均核实。

### 2.2 QA 独立测量（与交付同口径、同量级）

| 测量 | 单帧 median | AA median | AA/单帧 | AA 每 120k 射线 |
| --- | --- | --- | --- | --- |
| QA run A | 13.701 ms | 33.393 ms | 2.437 | 8.348 ms |
| QA run B | 13.163 ms | 33.289 ms | 2.529 | 8.322 ms |
| QA 独占采样窗口 | 14.562 ms | 33.379 ms | 2.292 | 8.345 ms |
| 交付 A / B | 14.030 / 14.389 | 36.174 / 36.423 | 2.578 / 2.531 | 9.043 / 9.106 |
| 交付 perf 测试在本窗口重跑 | 13.727 | 33.424 | 2.435 | 8.356 |

- 单帧与交付差 **−6.2%~+3.8%**、AA 约 **−7.6%~−8.3%**、比值 2.29–2.53 vs 2.53–2.58 —— 同量级、
  相对关系一致；差异符合“窗口间桌面负载波动”的解释（交付明确只给相对基准，不宣称固定指标）。
- 交付 `render_perf` 套件本窗口重跑 **2/2**，与 QA 独立 harness 数值同带（13.727/33.424）。

### 2.3 显存口径（54 MiB 峰值 / 每帧无增长 / 52 MiB 差值）

- 复核 `raw-timings.memory`：create 2 MiB、compile 4 MiB、峰值 **56,623,104 B**、frame_free 10 次
  **全部恒定（增长 0）**、destroy 后 context 2 MiB 回池、相对进程起始残差 **54,525,952 B**；
- **QA 独立测量（2 个完整周期）**：cycle0 峰值 88,080,384 B（首帧含模块/驱动加载）、
  **cycle1 峰值仅 2,097,152 B（= context）**、两周期每帧 free 恒定（增长 0）、
  两次 destroy 后 free **完全相同**（`residual_between_cycles = 0`）。
- **判定：52 MiB 差值“驱动代码缓存/池一次性保留、非泄漏”的解释成立**（结构性证据：一次性下降、
  不随周期增长、每帧 free 恒定、context 可回池）；绝对幅度随进程状态/首帧加载时机不同，
  不影响“非泄漏”结论。

### 2.4 独占窗口

- 交付：`nvidia-smi-samples.txt` 329 点（100 ms），min/median/max = **4246/4278/4556 MiB**，
  首末 4278，与 receipt 全对；窗口起止 `--query-compute-apps` 无其它 CUDA 计算进程。
- **QA 自有采样**（覆盖 QA 测量窗口）：150 点，min/median/max = **4278/4278/4629 MiB**，
  首末 4279/4278（回桌面基线）；窗口内 app 列表除 QA 自己的 `qa-perf-harness.exe` 外
  **无 python/cargo/mjwarp 等其它计算进程**。
- 说明（如实）：Windows 下 `--query-compute-apps` 的 `used_memory` 多为 N/A 且列表长期含桌面
  图形应用，所以“无其它 CUDA 计算进程”的证据强度 = 进程列表 + 显存曲线回基线 + 峰值带与本任务
  一致；亚 100 ms 峰值可能漏采（交付已如实写入未满足项）。

### 2.5 相对口径与未测项（诚实性）

- 只给单场景（scene-a）单机（RTX 4070 Ti SUPER）相对基准；扫描“固定性能指标/跨硬件/上游对比”类词表**无正向命中**（仅否定句）。
  比较/上游比较”等词表**无正向命中**（仅否定句）。
- 未满足项 6 条如实覆盖：单场景单机不可跨硬件、不与上游比较、未测 release、显存亚 100 ms
  峰值可能漏采、未测 nworld>1/更高 spp/未实现路径、无固定 CI 基准。

### 2.6 `18ae715`（S8 修正）三点 + 数字不回归

1. **反证①判别力断言**：`tests/render_e2e.rs` 新增 `tamper_diff = compare_pose_bits(&product.poses,
   &tampered.camera_data)` 并断言 origin/xmat 至少一项非零 → 可判定（QA 上轮独立结果 6 origin +
   17 xmat，与断言口径一致）。
2. **provenance 无悬空**：`camlight-model.json` 的 `camera_data_reference.sources` 仅保留实际存在的
   S7b1 同配方夹具（全部存在）；fixture 415 行 / 6,434 B / sha `470adbef…` 与 S8 receipt 一致。
3. **README 不命中禁用词**：S8 README 对任务登记的 3 个机器禁用词命中 **0**；“上游完全等价”仅出现在「未声明」否定句中出现 1 次（语义为否定，建议后续统一措辞）。
- **S8 数字不回归**（QA 本窗口重跑 `render_e2e` 5/5）：camlight 两相机 0 位差、端到端 0 差异、
  反证 115,972 / 扰动 18,859/19,182/9,464、运行哈希 `03efa95416bbd008` / `ed50f750936c0275` 保持。

## 3. 不回归 / 门禁 / 证据范围

- 本窗口重跑：perf **2/2**、S8 `render_e2e` **5/5**、S7b1 **6/6**、S7b2 **3/3**、S7b3 **6/6**
  （PTX 197,583 / 27,536 B 不变）；`cargo test --locked` **101 二进制 532/0**；fmt、clippy 通过。
- 交付 perf 证据 7 文件：README/receipt 与 git blob **逐字节相同**；raw-timings/gpu-run/repeat-run/
  regression/nvidia-smi-samples 与 README 声明 sha256 **全部一致**（与 blob 仅 CRLF 差异，
  EOL 归一后相同）。
- `git diff eadae87^..eadae87`：**仅 `tests/render_perf.rs` + perf 证据 + `.normify`，`src/` 变更 0**
  （无产品语义改动）；`18ae715`：仅 `tests/render_e2e.rs` + fixture + authoring 脚本 + S8 证据，
  `src/` 变更 0；`render_perf.rs` 340 行 / sha `96078c10…` 与 receipt 一致；`launch.rs` 未改。

## 4. 问题清单

### 建议（不影响数据）

1. **S8 receipt 的 authoring 脚本行数陈旧**：记 198 行，实际文件 **197 行**（sha256 `d6866011…`
   正确）。建议 `normify fix` 修正行数字段。
2. **README §2 对 `official_summary` 位置描述不准确**：原文写“measurement_a/measurement_b 各含
   `modes[]...`＋`official_summary`”，实际 `official_summary` 在 measurement 级（与 `modes` 平级）。
   数据本身元整派生正确，建议措辞修正以免误读。
3. **观察（非缺陷）**：本窗口 QA 独立测量 AA 比交付低约 8%、单帧低 0~6%，AA/单帧比 2.29–2.53
   vs 2.53–2.58；交付只作相对基准，属窗口间桌面负载波动。建议后续如需横向对比，固定同窗口同口径。

### 限制（如实）

4. 本轮未跑 release 构建、未与上游比较、未测 nworld>1/更高 spp；未重新插桩亚 100 ms 峰值采样
   （与交付未满足项一致）；性能数字只对本窗口/本机有效，不可作固定指标或跨硬件依据。

## 5. GPU 窗口

- 交接基线 4,278 MiB（Arch），QA 首测 **4,352 MiB**；结束 **4,352 MiB**，无本任务 CUDA 计算进程。
- QA 实际使用：3 次 perf harness（每次 2 配置 × (5 warmup + 30 迭代)）、1 次显存双周期测量、
  1 次独占采样窗口、5 套交付用例重跑（7.50+7.82+4.07+3.54+8.30 s），合计约 2 分钟量级，
  30 分钟窗口内、无冲突，未超窗口占用。

## 6. 证据文件

- `receipt.json`、`qa-artifact-manifest.json`、`artifact-check.json`、`gpu-window.txt`
- `judge_raw_stats.json`（原始序列重算 + 交付数字逐项比较）、`judge_perf_evidence.json`
  （证据/范围/口径/S8 修正/QA 独立测量/显存/独占采样）
- `qa_perf_a.json`、`qa_perf_b.json`、`qa_perf_exclusive.json`、`qa_mem.json`、
  `qa-window-mem.txt`、`qa-window-apps.txt`（QA 原始结果）
- `qa-gpu-runs.log`（perf/S8/S7b1/S7b2/S7b3 重跑）、`qa-unit-tests.log`（101 二进制 532/0）
- 脚本：`judge_raw_stats.py`、`judge_perf_evidence.py`、`qa-harness-main.rs.txt`
