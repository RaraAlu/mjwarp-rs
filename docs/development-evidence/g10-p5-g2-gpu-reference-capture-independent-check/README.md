# G2/G6 GPU 参考重采 · QA 独立复核（旧证据未动 / 输入一致 / 两轮一致 / provenance / 差异量级）

- 复核任务：`2026-10-25-g10-p5-g2-gpu-reference-capture-independent-check`（QA，p54）
- 被复核任务：`2026-10-25-g10-p5-g2-gpu-reference-capture`，交付提交 **`a69d1f81`**
- 复核 HEAD：`a69d1f81`；性质：**纯 CPU 只读复核**（git/JSON/日志 + 自写脚本），**未开 GPU 窗口**
- **结论：pass（阻断 0 / 发现 0 / 观察 5）**。旧 CPU 参考与旧脚本逐字节未动、新文件为新增；
  输入与旧参考逐位一致；两轮独立进程 + 独立 kernel cache，整文件 sha 相同（我自算）；provenance
  与实测 `nvidia-smi`/已装库/仓库脚本一致；新旧参考无结构差异、浮点差在 FMA 量级；
  脚本 diff 只含白名单变化；窗口记录与 Arch 批准一致、无残留。新参考**尚不是判据**（fix 片切换后才生效）。

## 1. 旧证据未动（硬红线，成立）

`qa-old-untouched.json`（git 对象 + worktree 字节，均自算）：

- 交付提交 `a69d1f81` 对旧参考/旧脚本的 `git diff` = **空**；两者 parent/head blob **相同**，
  worktree 字节 = `HEAD` 内容；
  - 旧参考 `upstream-mjwarp-f32.json` sha256 = `a00c722892df5f190096f2c89e39455cd43df3be8e3fe46869b09c1f6d20f04f`；
  - 旧脚本 `capture_mjwarp_f32.py` sha256 = `1edd7c8e5d2c7ec7ba7cb9eb218e4533e4f7fdb77c435de1107ad4086c624312`。
- 新参考 `upstream-mjwarp-f32-gpu.json` 在该提交为 **A（新增）**：parent 不存在、head 存在、
  worktree sha256 `d86c73cb04fef7f777afabf7d1291a8abaf0d744c41bdad277bd42350c8c87aa`（14330 B）。
- 提交文件分类：无 `src/`、`tests/`、`fixtures/`、`scripts/`、`case_compare.rs` 改动（仅证据目录 + `.normify`）。
- `git status -- docs/development-evidence/g10-p5-gjk-epa fixtures/g10-p5` = 干净。
- `case_compare.rs` 仍读**旧 CPU** 文件（`upstream-mjwarp-f32.json`），`src/` 自 `209b1039` 无 diff
  → **新参考尚未被产品判据使用**。

## 2. 输入一致（成立）

`qa-new-old-diff.json`（自写；不读交付 `inputs-vs-old.json`）：

- `upstream_revision` 新旧相同 = `71da24d956378a87a703b6e1442b13aec0c4ac29`。
- 5 case 的 `xml` / `xml_sha256` / `qpos` / `qpos_bits` **逐值/逐位与旧参考相同**；
  provenance 版本（python/mujoco/warp/mujoco_warp/numpy）在新旧参考中完全一致。
- 新旧 `count` 均 = **8 / 2 / 2 / 2 / 2**。

## 3. 两轮一致（成立；独立性来自两次独立 CMD + 两个 cache）

`qa-two-rounds.json`（sha 由我自算，不采信 meta 声明）：

- `round1-raw.json` = `round2-raw.json` = 新参考文件，整文件 sha256
  **`d86c73cb04fef7f777afabf7d1291a8abaf0d744c41bdad277bd42350c8c87aa`**；
  canonical 两轮同为 `10956ae0c113bd86541c96fdf7dedfcdd3408fb8a70d3bec5b45e8fb29846702`；
  `capture-meta.json` 的 4 个 sha 全部与我的重算一致。
- 新参考每 case `two_captures_bitwise_identical = true`（5/5）。
- 独立进程证据：两轮日志各含独立 `CMD:`（python 进程两次启动）、`kernel_cache_dir` 分别为
  `.../warp-cache-gpu/round1` 与 `.../round2`（互不相同）、`WALL_MS=28083` vs `4469`、日志 mtime 顺序递增；
  两个 cache 目录各自存在且非空（各 21 文件、约 8.03 MB，各含 `.cu/.meta/.ptx`；round2 也重新 JIT）。
- 观察：runner 未记录 PID，独立性不是 PID 级证据（见 §8-2）。

## 4. provenance 真实（成立）

`qa-provenance.json`（对实测 `nvidia-smi`、已装包、仓库脚本字节）：

- `device = cuda`；`gpu` = `NVIDIA GeForce RTX 4070 Ti SUPER / 16376 MiB / driver 596.36 / compute_cap 8.9`
  与评审时只读 `nvidia-smi` **逐项一致**（非手填模板）。
- 版本与已装包一致：warp **1.15.0**、mujoco_warp **3.15.0**、mujoco **3.15.0**、numpy 2.5.4、python 3.12.13。
- `authoring_script_sha256 = 91d62fd8dc1b99aba2e2e4bbf5356c8bfc2dbbfb415894f6a9187999b6f100a1`
  = 仓库内 `capture_mjwarp_f32_gpu.py` 实测 sha；旧脚本 sha 未变。
- `upstream_revision` 正确；`captured_at = 2026-10-26T01:29:57Z`（与 `capture-meta.json` 一致，
  落在窗口 `20261026-012956` 内）；`kernel_cache_root = target/g10-p5-g2/warp-cache-gpu` 存在；
  `authoring_only=true`、`reads_product_tree=false`、`product_python_dependency=false`。

## 5. 差异量级（成立：无结构差异，浮点差 FMA 量级）

`qa-new-old-diff.json`（我自算；行按 `INT_FIELDS` 键配对，原始行序差异见 §8-1）：

| case | count 旧/新 | 整型字段（多重集） | frame 位等 / max_abs | dist max_abs | pos max_abs |
| --- | --- | --- | --- | --- | --- |
| cylinder-box | 8/8 | 全等 | 72/72，0 | 0 | 1.490e-8 |
| ellipsoid-box | 2/2 | 全等 | 8/18，**8.192876e-7** | 1.863e-9 | 7.451e-9 |
| mesh-mesh | 2/2 | 全等 | 2/18，5.364e-7 | 9.313e-9 | 7.451e-9 |
| convex-mesh | 2/2 | 全等 | 2/18，2.980e-7 | 1.490e-8 | 1.118e-8 |
| degenerate-mesh | 2/2 | 全等 | 16/18，2.651e-11 | 1.048e-8 | 1.863e-9 |

- 全部 17 字段合计 **395/464 分量位级相同**，所有浮点最大绝对差 = **8.192876e-7**（frame），
  dist/pos ≤ 1.5e-8；`geom_xpos` 仅 degenerate-mesh 2/12 分量差（≤7.45e-9）、`geom_xmat` 仅
  mesh-mesh 2/36 与 degenerate-mesh 6/36（≤2.98e-8）——均为 FMA 量级。
- **无 count / 整型字段结构差异**；与交付 `compare-cpu-vs-gpu.json` 逐 field 的 `bit_equal/max_abs`
  **完全一致**（395/464、各 max_abs 相同），但我的配对方法独立实现。
- frame 最大差 8.192e-7 与既有上层 CPU/GPU 自身差量级（G5 证据 8.19e-7）一致 → 与「GPU 融合语义差」一致，
  未触发 B 前提的停车条件。

## 6. 脚本差异受控（成立）

`qa-script-diff.json`：

- 必须逐字不变的块：`CASES`、`FLOAT_FIELDS`、`INT_FIELDS`、`capture()`、`load_qpos()`、`bits32()`
  → **全部字节相同**（我按标记块独立提取比对）；17 字段齐全；两次捕获逻辑在。
- 全文 5 个 hunk / 62 行变更，分类：docstring 17、provenance helper（`gpu_info()`）13、路径/device 8、
  其余白名单 24，**unmatched = 0**；交付 `script-diff.txt` 的 hunk 数与我的 diff 一致。
- 结论：只出现声明的 device/输出名/kind/task_id/purpose/provenance/缓存目录变化，无采集语义变化。

## 7. 窗口纪律（与批准记录一致）

`qa-window.json`（独立解析窗口日志）：

- 采样 **122 点**，实际间隔中位 0.282 s（0.25 s 目标 + `nvidia-smi` 调用开销）；used 峰值 **4417**
  （基线 4078，增量 **+339**）、free 最低 **11645**、util 最高 **80%**、温度最高 **30 °C**
  —— 与 Arch 批准记录逐项一致；util≥80% 仅连续 2 点（<3，不触发守卫）。
- pre/post `query-compute-apps` 均 **35 条**、名称与 PID 集合相同、无 python 进程；
  guard `aborted=False`、`residual_context_pids` 空。
- 守卫模板 `scripts/gpu-window-guard.ps1` 自 `209b1039` **未改**；attempt-1/2 为 **零 GPU 工作**
  （日志无 CUDA device 初始化、无 module 编译、无 round 产物），不拼接；有效窗口仅 attempt-3。
- 窗口内 git 快照显示 `src/` 干净（fix 补丁已移出工作树）；`finalize_capture.ps1` 为 CPU 后处理
  （无 `-Execute`/GPU 采集调用，仅路径字符串含 `warp-cache-gpu`）。

## 8. 观察（非阻断，5）

1. **cylinder-box 接触行原始顺序**在旧 CPU 与新 GPU 参考间不同（GPU atomic 写序：
   `worldid`/`geomcollisionid` 交错），但按 `INT_FIELDS` 键的**多重集完全一致**、逐字段多重集一致；
   产品侧 `case_compare.rs::sort_contacts` 会对 expected/actual 都排序，故 B 版 fix 只要沿用排序比较
   即不受影响。交付 `compare` 的 `integer_fields_bit_equal=true` 是「按键配对」口径，未声称原始行序相同。
2. **两轮未记录 PID**：runner 只落 `CMD`/cache/wall/日志时序，独立性证据为「两次独立 python 调用 +
   两个独立 kernel cache + 不同 wall + 顺序日志」，不是 PID 级证据；建议后续 capture runner 落 PID。
3. 采样实际间隔中位 0.282 s（非严格 0.25 s），属 `nvidia-smi` 调用开销；守卫 4 个离散快照的
   `contexts` 恒 35（own-step 期间未采到 python 的 CUDA context），own GPU 工作由 sampler 的
   used 峰值与 round 日志（module 编译/计数）证明。
4. 窗口内 warp 自身做 CUDA kernel JIT（round1/round2 各 7 个 module），属采集本身；
   窗口内**无其它任务**编译（git 快照 `src/` 干净、无 cargo/rustc 进程）。
5. 新参考**尚不是判据**：`case_compare.rs` 仍指向旧 CPU 参考，`src/` 自 `209b1039` 无改动；
   「旧参考作废/归档」要等 fix 片切换判据后才生效（交付 README 亦如此声明）。

## 9. 口径与边界

- 本复核纯 CPU：只读 git/JSON/日志 + 只读 `nvidia-smi` 查询，**未开 GPU 窗口、未建 CUDA 上下文**；
  未重跑 warp 采集、未修改交付证据/产品树/判据；未 commit、未 done。
- 不宣称精度结论、跨环境全绿、上游等价；只核「GPU 参考已重采且 provenance/输入/两轮/差异量级成立」。
- 真实无 FMA 机器未实机验证（与 fix 片无关）；Windows-only。

## 10. 复现与产物

```powershell
$py = 'target\mjwarp-parallel\venv\Scripts\python.exe'
& $py docs\development-evidence\g10-p5-g2-gpu-reference-capture-independent-check\qa_old_untouched.py
& $py docs\development-evidence\g10-p5-g2-gpu-reference-capture-independent-check\qa_new_old_diff.py
& $py docs\development-evidence\g10-p5-g2-gpu-reference-capture-independent-check\qa_two_rounds.py
& $py docs\development-evidence\g10-p5-g2-gpu-reference-capture-independent-check\qa_provenance.py
& $py docs\development-evidence\g10-p5-g2-gpu-reference-capture-independent-check\qa_script_diff.py
& $py docs\development-evidence\g10-p5-g2-gpu-reference-capture-independent-check\qa_window.py
# 证据冻结后 gate/check（stdout 写 target/，避免 gate-scope 自失效）
& docs\development-evidence\g10-p5-g2-gpu-reference-capture-independent-check\run_gate_check.ps1
```

证据目录 `docs/development-evidence/g10-p5-g2-gpu-reference-capture-independent-check/`：
`README.md`、`receipt.json`、`artifacts-sha256.json`、`qa-old-untouched.json`、`qa-new-old-diff.json`、
`qa-two-rounds.json`、`qa-provenance.json`、`qa-script-diff.json`、`qa-window.json`、
`logs/`（`commands.txt`、`nvidia-smi-live.txt`、`summary.txt`）、脚本 7 个
（`qa_*.py` ×6、`run_gate_check.ps1`、`qa_artifacts_sha256.py`）。不入库：`target/qa-capture-check/**`。
