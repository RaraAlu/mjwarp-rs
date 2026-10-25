# faithful FMA 口径影响面实测（只读）

- 任务：`2026-10-25-faithful-impact-measurement`（QA，只读实测）
- 钉点：`5e000cb9`，隔离 worktree `target/worktrees/faithful-5e000cb9`（主树未动）；复核期间主树 HEAD `3a738c29`
- 执行口径：**实测在 worktree 钉点 `5e000cb9` 完成；gate/check 在主树执行**（绑定主树 HEAD `5278847c`，
  不用钉点 worktree 跑门禁，避免绑定旧树）
- 口径：① **仓库默认 GNU**（`.cargo/config.toml` 的 `-C link-dead-code=yes`）vs ② **faithful**：GNU + `RUSTFLAGS=-C target-feature=+fma`
  （必要时 ③ `+fma + link-dead-code=yes` 作隔离对照）；CPU-only 部分三口径，GPU 部分两口径串行
- 结论：**影响面窄 —— 20 个定向项里只有 `case_evidence::upstream_f32_within_4ulp`（CPU frame 判据）翻转**；
  **GPU 端 launcher / primitive / multicontact 的位级对照与 render S7b1/S7b2/S7b3/S8 的 same-recipe 对照在两口径下逐项相同**
  （20/20 运行 exit 0，关键计数、ULP、RGB 直方图与输出哈希全等）。
- 本片只报「该口径下哪些变/不变」，不宣称修复/验收通过；无新参考捕获、未改产品/判据/容差/门禁。

## 1. 隔离、EOL 与 clean 判据

- `git worktree add` 到 `target/worktrees/faithful-5e000cb9`（`target/` 为 gitignore），**钉 `5e000cb9`**；
  主树一律不动。钉点相对主树避开的在途改动：`src/math/mod.rs`、`src/physics/collision/relmat.rs`、
  `src/render/bvh/aabb.rs`（已逐字节核验 worktree 内这 3 个文件 = pinned blob，未混入 relmat 修复）。
- **EOL 方法学修正**：新 worktree 在 `core.autocrlf` 下把 `scripts/authoring/g10-p5/capture_sdf_rotation_references.py`
  检出为 CRLF，导致 fixture 脚本 sha 断言假失败（两口径同）。按既有做法把主树中「同 blob、仅 EOL 不同」的
  tracked 文件字节同步进 worktree（4 个主树已修改的 tracked 文件排除；57 个超长路径证据文件跳过，测试不引用），
  修正后 capture script sha 命中记录的 `f7135f31…`，CPU 测试全通过（`logs/eol-sync.txt`）。
- **clean 判据**（每口径独立删空 `CARGO_TARGET_DIR`；清空 `CARGO*/RUST*`，清空前计数 0）：
  rustc `1.99.0 (b940084d7)` host `x86_64-pc-windows-gnu`、cargo 1.99.0；target 目录
  `target/faithful/{gnu-default,gnu-fma,gnu-fma-ldc}`（CPU，deps 各 71）与
  `target/faithful/{gnu-default,gnu-fma}-cuda`（cuda-probe，deps 各 91）。
- **`RUSTFLAGS` 与 `.cargo/config.toml` 的交互（实测，证据 `logs/rustc-command-lines.txt`）**：
  | 口径 | `-C link-dead-code=yes` | `-C target-feature=+fma` |
  | --- | --- | --- |
  | 默认 GNU | **有**（来自 config） | 无 |
  | `RUSTFLAGS=-C target-feature=+fma` | **无（被 RUSTFLAGS 覆盖）** | 有 |
  | `RUSTFLAGS='-C target-feature=+fma -C link-dead-code=yes'` | 有（显式补回） | 有 |
  → 设 `RUSTFLAGS` 会**替换** config 的 gnu rustflags；faithful 口径默认不带 link-dead-code。CPU 侧 ③ 证明
  「补回 link-dead-code」不改变任何结论（`case_evidence` 仍 6.351978e-7 失败），即差异只来自 FMA 语义。

## 2. 纯 CPU 结果（三口径）

| 项 | 默认 GNU | GNU `+fma` | `+fma+ldc` | 翻转 |
| --- | --- | --- | --- | --- |
| G5 `rel_mat`（aᵀb 位级，10 world，8 world 可区分 bᵀa） | **10/10** | **10/10** | **10/10** | 否 |
| SDF 旋转 40/40（5 场景，每场景 2 world × 40 = 80 接触） | **80/80 ×5，dist/pos/frame max_abs 0e0** | 同左 | 同左 | 否 |
| `g10_p5_sdf_host`（G3 参考夹具结构/哈希） | 1 passed | 1 passed | 1 passed | 否 |
| `case_evidence::upstream_f32_within_4ulp`（σ frame ≤4.8e-7） | **pass，frame 3.838112e-7** | **FAIL，frame 6.351978e-7** | **FAIL，frame 6.351978e-7** | **是** |
| `case_evidence` 打印项（其余数值） | dist 位等 2/2、pos 1/2 max 2.00 ULP、frame 13/18 位等 | 同左（仅 frame 变） | 同左 | 仅 frame |

## 3. GPU 窗口结果（两口径串行，20/20 exit 0）

窗口：`18:06:38Z→18:09:04Z`（146 s），守卫未触发；**两口径逐项结果完全相同**（下表数字两侧一致）。

| 项 | 默认 GNU | GNU `+fma` | 翻转 |
| --- | --- | --- | --- |
| launcher（hfield-sphere / sdf-box / ellipsoid-box） | **314/314、3122/3122、80/80**；max ULP 0/0/0；failures 0 | 同左 | 否 |
| primitive（shallow / deep / capsule-box） | **80/80 ×3**；max ULP 0 | 同左 | 否 |
| multicontact（cylinder-box / box-box-margin / mesh-mesh） | **314/314、391/392（max_pos_ulp=1）、80/80** | 同左（391/392、max_pos_ulp=1） | 否 |
| **S7b1 depth/seg**（同配方参考，120,000 像素） | cam0 76800/76800 mismatch 0 max_ulp 0；cam1 43200 同左；seg_mismatch 0；rays=120000 recipe_max_ulp=0 | 同左（逐值相同） | 否 |
| **S7b2 RGB**（同配方 120,000 像素） | cam0 76800 pixels mismatched **0** max_byte_diff **0**；cam1 43200 同左；depth/seg 回归 0 | 同左 | 否 |
| S7b3 flag variants（5 变体） | 5/5 matched；各变体两相机 mismatched 0 | 同左 | 否 |
| S7b3 AA(spp=2) | 两相机 mismatched 0、max_byte_diff 0；重复运行位等 | 同左 | 否 |
| S7b3 baseline 不回归 | RGB/depth/seg 全 0 | 同左 | 否 |
| **S8 e2e same-recipe**（120,000 像素） | depth mismatch 0 max_ulp 0；RGB mismatched 0；输出哈希 `depth_seg=03efa95416bbd008`、`rgb=ed50f750936c0275` | 同左（哈希相同） | 否 |
| S8 camlight pose bits | origin/xmat mismatch [0,0]/[0,0]、max_abs 0.0（cam0 vs host quat_to_mat 辅助口径 max_ulp=1、2/9 分量，两口径相同） | 同左 | 否 |

补充（两口径完全一致、均属 recorded 而非判定）：S7b1 cross-recipe(S0b fast-math) 记录 cam0 depth_mismatch=19074/max_ulp=2、
cam1 4064/max_ulp=4；S7b2/S8 cross-recipe 记录 max_byte_diff=1（2 像素）——这些跨配方已知偏差**不随 FMA 口径变化**。

## 4. 窗口事实（有效轮 attempt 3）

- 基线：used **3398** / free 12664 MiB、util 35%、29°C；跑后 used **3386** / free 12676、util 14%、30°C。
- 0.25 s 采样 **416 点**：used 3376–**4738** MiB（起始 3398 → 峰值 +1340）、free 最低 **11324**、util 最高 63%、30°C。
- 守卫参数：新增 CUDA 计算 pid（非本任务 `g10_p5_*`/`render_*`）中止；free<10240 中止；used 增量≥3072 中止；
  纯 CPU 进程只记录不中止（`logs/cpu-process-observations-summary.txt`，窗口内有其它 agent 的 cargo/rustc/clippy/fmt
  与 `mjwarp_rs-<hash>` CPU 测试进程，均未中止）。
- 跑前/跑后 `query-compute-apps`：33/33 条桌面图形上下文，本任务相关 0；**无残留 CUDA pid、无残留测试进程**。
- **窗口内不编译**：20 份运行日志均无 `Compiling `，逐口径解析到 `target/faithful/<mode>-cuda/...`。
- 三次尝试：attempt 1 被 v1 保守守卫误中止（作废）、attempt 2 因驱动脚本 env 作用域 bug 无效（作废，
  见 §5 观察 1/2）、attempt 3 有效；累计 GPU 占用约 358 s，在批准的 10 分钟窗口预算内。

## 5. 观察（均非阻断）

1. **attempt 2 驱动脚本 bug（已修正，数据作废）**：脚本在「每个 test 之后」清 `CARGO_TARGET_DIR`/`RUSTFLAGS`，
   导致每模式第 2–10 项回落到 worktree 内 `target/` 并在窗口内重编译、且 `+fma` 口径后 9 项实际退化为默认 GNU。
   已修（env 按模式设置、整模式结束才清理）并整轮重跑；attempt 2 数字不进矩阵（`logs/window-attempt-2-invalid.txt`）。
2. **attempt 1 守卫误触**：v1 按进程名匹配 `mjwarp` 命中其它 agent 的 CPU lib 测试二进制；GPU 记账全程平坦。
   已按批准细化守卫并与项目模板（`39ae6a1b`）口径对齐（`logs/window-attempt-1-aborted.txt`）。
3. **窗口内其它 agent 的 CPU 编译活动**：按批准规则只记录不中止；不影响数值结果，但会抬高 wall time（launcher 19–22 s）。
4. **日志中文二次编码**：窗口日志中 cargo 中文输出有控制台编码转换痕迹（数字/断言 ASCII 行完整），
   影响矩阵只取 ASCII 数字与哈希，不影响结论。
5. **render 不敏感的经验事实**：faithful 下 render 全部 0 差异/同哈希，但本片未做 kernel/PTX 级机制解释（超出只读测量范围）。
6. **既有 1 ULP/跨配方项与口径无关**：multicontact `box-box-margin` 391/392（max_pos_ulp=1）与 S7b2/S8 cross-recipe
   `max_byte_diff=1` 在两口径完全一致，说明它们不是 FMA 语义导致的。
7. **attempt-1（作废轮）数字仅作对照、不拼接**：其 launcher `314/314、3122/3122、80/80` 与 primitive `80/80×3`
   与有效轮**逐项一致**；但 attempt-1 的 wall 时间不可比（其脚本同样有 env 作用域问题、第 2 项含窗口内重编译，
   primitive 36.4 s vs 有效轮 19.4 s）。影响矩阵只使用有效轮 attempt 3 的 20 项结果，不含 attempt-1/2 任何数字。

## 6. 影响矩阵结论（供口径决策）

- **翻转（1 项）**：`case_evidence::upstream_f32_within_4ulp` —— GNU 默认 **pass（frame 3.838112e-7）**，
  faithful **fail（frame 6.351978e-7 > 4.8e-7）**；补回 link-dead-code 不改变该结论。
- **不变（19 项）**：G5 rel_mat、SDF 40/40、launcher 314/3122/80、primitive 80/80×3、multicontact 314/391/80、
  S7b1 depth/seg 120,000/120,000、S7b2 RGB 0 差异、S7b3 flags/AA/baseline、S8 e2e 120,000/120,000 与 camlight 位等。
- 「无参考不可判」项：**无**（所列项均有冻结参考）。
- 影响面：**窄**；若切 faithful 口径，已 verified 结果中只有 `case_evidence` 的 frame 判据会翻转，
  render 的「0 差异/位级」结论仍成立（本片实测，同一窗口、同 tree 钉点）。

## 7. 边界

只读；未改产品代码/判据/容差/期望值/gate/G01 脚本；未新捕获参考；未跑 render perf、Linux、其它 GPU；
未解释 render 不敏感的底层机制；不宣称修复或验收通过。

## 8. 复现与显式路径

```powershell
& target\toolchains\powershell-7.6.7\bin\pwsh.exe -NoProfile -File docs\development-evidence\faithful-impact-measurement\run_cpu_matrix.ps1      # CPU 三口径
& target\toolchains\powershell-7.6.7\bin\pwsh.exe -NoProfile -File docs\development-evidence\faithful-impact-measurement\run_gpu_prepare.ps1     # 窗口外构建
& target\toolchains\powershell-7.6.7\bin\pwsh.exe -NoProfile -File docs\development-evidence\faithful-impact-measurement\run_gpu_window.ps1 -Modes both  # 窗口内
& target\mjwarp-parallel\venv\Scripts\python.exe docs\development-evidence\faithful-impact-measurement\qa_impact_matrix.py
```

本目录：`README.md`、`receipt.json`、`artifacts-sha256.json`、`impact-matrix.json`、`cpu-matrix.json`、脚本
（`run_cpu_matrix.ps1`、`run_gpu_prepare.ps1`、`run_gpu_window.ps1`、`qa_eol_sync.py`、`qa_impact_matrix.py`、
`qa_trim_logs.py`、`qa_artifacts_sha256.py`）、`logs/`（CPU/GPU 原始日志、窗口 summary/samples/pre/post、
三次尝试说明、rustc 命令行、EOL 摘要、CPU 进程汇总）。
不入库：`target/worktrees/faithful-5e000cb9`、`target/faithful/**`（构建产物/原始大日志/attempt2 原始件）。
