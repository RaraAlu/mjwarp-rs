# 设备 CCD 溢出位误报修正（EPA `FLOAT_MAX` 时不得置 `OverflowType.CCD`）

- 任务：`2026-10-25-g10-p5-ccd-overflow-bit-fix`（P5，G6 前必做；GPU 窗口 1 次）
- 状态：修正 + 验证完成，待提交（提交由 Arch 串行执行）
- 取证 HEAD：窗口起点 `c818a31e9fdc65db1796030d542554104de0e07e`；构建/探针源绑定 `a93c0c5e7c56af8b490e4e3899149198ea7a8ac5`（窗口期间 Arch 有两个治理提交，`src/physics/collision/cuda/` 未变；本片设备改动为工作树未提交内容）
- 不碰 G5 已冻结证据目录（只读其 case bins）；只新增本片证据
- 不声称「精度达标 / 上游完全等价 / 原生放行」

## 结论

1. 设备 `g5_ccd` 在 EPA 返回 `FLOAT_MAX` 时不再额外置 `OverflowType.CCD(1<<4)`，只透传 EPA 阶段的 `EPA_HORIZON(1<<8)`——与 pinned 上游 `collision_gjk.ccd()` 一致。
2. 强制 horizon 容量 = 2 的 3 例（`cylinder-box-h2`/`box-box-margin-h2`/`mesh-mesh-h2`）：
   设备 overflow **256**（= `EPA_HORIZON`）与同配方上游 f32 GPU **逐 world 位级相等（256，不含 16）**；
   `dist`/`x1`/`x2` 位级相等、`ncontact`/`multiccd_idx` 相等、两次运行位级一致。
   修正前设备源（HEAD 源）在触发 `FLOAT_MAX` 的 world 为 **272（=256|16）**——反向判别力成立。
3. G5 回归：交付 3 例（`hfield-sphere` 8 行、`hfield-capsule` 7 行、`sdf-box` 80 行）+ `ellipsoid-box`（2 行）
   接触输出按 G5 口径（`worldid`+全字段稳定排序）逐行位级一致，`nacon`/`nccd`/`overflow` 摘要计数一致；
   witness 三例（`cylinder-box`/`box-box-margin`/`mesh-mesh`）输出逐文件位级一致。合计 180 个 bin 文件、
   194 行接触记录、32 项摘要计数，**0 差异**。→ G5 已交付结论不受本片影响。
4. 上游全程只在**真实容量截断**时置 `CCD`；`ccd()` 自身从不因 `FLOAT_MAX` 置位。

## 改动（3 文件，+8/−5，全部在 `src/physics/collision/cuda/`）

| 文件 | 改动 |
| --- | --- |
| `ccd.cu:307-310` | 删除 `unsigned overflow = epa_phase_result.overflow; if (epa_phase_result.dist == G5_FLOAT_MAX) overflow \|= G5_OVERFLOW_CCD;`，改为 `out->overflow = epa_phase_result.overflow;` 并加注释（`CCD` 只由调用方容量门置） |
| `contact.cu:184-186` | 删除 `g5_ccd_pair_write` 中 `if (epa.dist == G5_FLOAT_MAX) atomicOr(overflow_out, G5_OVERFLOW_CCD);`；只保留 `if (epa.overflow != 0u) atomicOr(...)`。`g5_claim_ccd_slot`（`ccdid >= naccdmax` → `atomicOr(..., G5_OVERFLOW_CCD)`）不动 |
| `hfield.cu:262-263` | 注释纠正：上游只透传 `EPA_HORIZON`；`CCD` 只在 `nccd >= naccdmax` 截断时置（hfield 每棱柱 `g5_ccd` 的溢出汇总逻辑不变） |

`cuda/mod.rs` 的 include_str 引用不变；产品 ABI/结构体不变。

## 上游权威依据（pinned `71da24d956378a87a703b6e1442b13aec0c4ac29`）

- `collision_gjk.py:2663-2667`：`_epa(... idx == -1)` → `return FLOAT_MAX, 0, wp.vec3(), wp.vec3(), -1`（不置任何额外位）。
- `collision_gjk.py:1423-1430` / `1446-1453`：`_add_edge` 返回 -1（horizon 容量不足）时，仅 `wp.atomic_or(overflow_out, worldid, OverflowType.EPA_HORIZON)`。
- `collision_gjk.py:2712-2729`：`ccd()` 直接返回 `epa_phase(...)` 的结果，没有 `FLOAT_MAX → CCD` 逻辑。
- `collision_convex.py:860-870`（`eval_ccd_write_contact`）：`ccdid = wp.atomic_add(nccd_in, geomgeomid, 1)`；**只有 `ccdid >= naccdmax_in`** 才 `wp.atomic_or(overflow_out, worldid, OverflowType.CCD)` 并 `return`（这是 `CCD` 位的唯一来源）。`collision_convex.py:290-300` 同口径。
- `types.py:166`：`CCD = 1<<4`、`EPA_HORIZON = 1<<8`。

## 强制 horizon=2 的构造（不改上游源码）

上游 `_add_edge` 的容量判定是 `if n == pt.horizon.shape[0]: return -1`（`collision_gjk.py:961-964`），即
**`ccd()` 形参 `horizon` 是 `wp.array[int]`，其数组长度 = `Polytope.max_horizon`**。因此：
- 上游：调用 `collision_gjk.ccd(...)` 时传 `horizon = wp.empty(2, dtype=int)`（长度 2，产品默认 `MJ_MAX_EPAHORIZON=24`），
  确定性进入 `_add_edge` 的 -1 分支 → `EPA_HORIZON`、`dist=FLOAT_MAX`；
- 设备：`overflow_probe.cu` 中 `init_polytope(&local, max_horizon)`，`max_horizon=2`（数组上界仍是 24 的本地缓冲），
  其余调用口径与 `eval_ccd_write_contact` 的 ccd 段逐项一致（margin 覆盖、原点平移、`cutoff=gap`、逐 world 单次 launch）；
- 这是**等价容量**构造（同一段上游/设备代码 + 更小的 horizon 容量），不是改算法；两个方向都使用同一组 G5 冻结 case bins 输入。

## 强制 horizon 位型对照（3 例 × 2 world）

| case | world | 修正前设备（HEAD 源） | 修正后设备 | 上游 CUDA f32 | `dist` bits | ncontact | 判定 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `cylinder-box` | 0 | 272 | **256** | 256 | 1900671690 (`0x7149F2CA`=1e30) | 0 | 位级相等 |
| `cylinder-box` | 1 | 272 | **256** | 256 | 同 | 0 | 位级相等 |
| `box-box-margin` | 0 | 272 | **256** | 256 | 同 | 0 | 位级相等 |
| `box-box-margin` | 1 | 272 | **256** | 256 | 同 | 0 | 位级相等 |
| `mesh-mesh` | 0 | 0（EPA 正常收敛） | 0 | 0 | 3169614733 | 1 | 位级相等 |
| `mesh-mesh` | 1 | 272 | **256** | 256 | 1900671690 | 0 | 位级相等 |

- 修正前只在真正触发 `FLOAT_MAX` 的 world 多置 16（`mesh-mesh` world0 未触发，保持 0）；修正后与上游逐 world 完全一致。
- `x1/x2` 位级相等（`mesh-mesh` world0 的 `x1` bits `1015580809/1015580808/1015580809`、其余 world 全 0），
  `multiccd_idx` 相等（`mesh-mesh` world0 为 1，其余 -1）。
- 设备两次运行位级一致（`two_runs_bitwise_identical=true`）；上游两次独立捕获 JSON **字节级相同**
  （sha256 `9879145bfe652036834b112e66915a16ddf33f1e8f6e1e6cc0612a96f6afd960`，两次捕获共用同一 JIT 缓存）。

## 哪些输入会走到 `g5_ccd` 的 `FLOAT_MAX` 分支（判别）

`g5_epa` 只在 `idx == -1` 时返回 `face_idx=-1`（`epa.cu:113-117` / `130-135` / `145-148` / `157`），
对应上游 `_epa` 的两类退化路径：

1. **horizon 容量不足**（本片主路径）：`g5_add_edge` 在 `n == pt->max_horizon` 时返回 -1 → `EPA_HORIZON`。
   产品默认 `max_horizon = MJ_MAX_EPAHORIZON = 24`，需要「单次 EPA 迭代内 horizon 边数 > 24」才会触发，
   典型是复杂 mesh/退化 polytope 的深穿透；`naccdmax`/`naconmax` 只是 EPA 槽位与接触行容量，
   **不控制 horizon 容量**，调它们不会进入该分支（QA 的 `naccdmax=1` 例只触发容量门 `CCD`，不触发本分支）。
2. **退化附面**：`g5_attach_face` 返回 `dist2 == 0` 时 `idx = -1`（`epa.cu:145-148`），此时 `overflow` 保持 **0**。
   上游同路径同样不置位；修正前设备会错加 `CCD(16)`，修正后为 0（与上游一致）。

G5 交付的 3 个 case（`hfield-sphere`/`hfield-capsule`/`sdf-box`）与 `ellipsoid-box`、witness 三例在
G5 窗口内的输出 `overflow` 全 0（本片回归重跑逐字节/排序行一致），**不触发上述两条路径**，故 G5 结论不变。

## G5 回归（修正后设备源重跑，对比 G5 冻结窗口输出）

| 类别 | case | 对比方式 | 结果 |
| --- | --- | --- | --- |
| 接触数组 | `hfield-sphere` | `out1`/`out2`：行按 `worldid`+全字段稳定排序后位级；`overflow`/`nccd`/`nacon`/`efc_width` 摘要直接比 | 8 行 ×2 组，0 差异 |
| 接触数组 | `hfield-capsule` | 同上 | 7 行 ×2 组，0 差异 |
| 接触数组 | `sdf-box` | 同上 | 80 行 ×2 组，0 差异 |
| 接触数组 | `ellipsoid-box` | 同上 | 2 行 ×2 组，0 差异 |
| witness | `cylinder-box`/`box-box-margin`/`mesh-mesh` | `witness1`/`witness2` 逐文件位级 | 6 文件 ×2 ×3 例，0 差异 |

- 合计 180 个 bin 文件、194 行接触记录、32 项摘要计数，全部一致（`regression-result.json`）。
- 说明：G5 `probe.cu` 同一次运行两遍的原始行序会因 `nacon` 原子计数而不稳定（窗口日志里
  `hfield-sphere`/`ellipsoid-box` 的 `raw_equal=false` 即此原因，G5 冻结 `out1/out2` 之间行序也不同）。
  G5 判据本来就用「`worldid` 分组 + 稳定排序后比较」，本片回归沿用同口径；**行序不作判据**。

## GPU 窗口事实（单次，≈54 s）

| 项 | 值 |
| --- | --- |
| 窗口 | 19:41:15 → 19:42:09（`elapsed_seconds=53.9`），仅本任务；批准：Arch + Chief 双签 |
| 基线（跑前） | used **4016 MiB** / total 16376 MiB / free 12046 MiB / util 1% / 29°C / driver 596.36 |
| 峰值 | used **5381 MiB**，增量 **+1365 MiB**（0.5 s 采样，101 个样本；守卫 3072 MiB / 8188 MiB 未触发） |
| 结束（跑后） | used **4008 MiB**（低于基线）/ free 12054 MiB / 5% / 30°C |
| 计算进程 | 跑前/跑后 `--query-compute-apps` 均只有 WDDM 图形上下文；按 python/probe/nvcc/cargo/warp 过滤 = **0**，无本任务进程残留 |
| 内容 | 修正前/后设备探针（3 例）、上游 CUDA 强制 horizon 捕获 ×2、G5 探针回归（4+3 例 ×2） |
| 备注 | 构建（nvcc，不占 GPU）在窗口内一次完成，产物 sha256 见 `build-report.txt` |

## 复现

```powershell
# 1) 构建 4 个探针（不占 GPU；含 HEAD 源 device-before 抽取）
powershell -NoProfile -ExecutionPolicy Bypass -File `
  docs/development-evidence/g10-p5-ccd-overflow-bit-fix/build_probes.ps1

# 2) GPU 窗口（约 1 分钟；跑前自查 query-compute-apps）
powershell -NoProfile -ExecutionPolicy Bypass -File `
  docs/development-evidence/g10-p5-ccd-overflow-bit-fix/run_window.ps1

# 3) 判据（CPU 读已存盘产物；两个 ignored 测试）
cargo test --locked --test g10_p5_ccd_overflow -- --ignored --test-threads=1
```

`capture_forced_horizon.py` 也可单独重跑（`--device cpu` 为诊断，`--device cuda` 为正式判据参考）。

## 证据文件

| 文件 | 内容 |
| --- | --- |
| `overflow_probe.cu` | 本片设备探针（`g5_ccd` + 可配置 `max_horizon`，只读 device 源，不改设备逻辑） |
| `capture_forced_horizon.py` | 上游 f32 GPU 强制 horizon 捕获（`ccd()` + `horizon` 长度=2，不改上游源码） |
| `build_probes.ps1` / `build-report.txt` | 4 个探针构建命令、device-before/after 源 sha256、产物 sha256 |
| `run_window.ps1` / `window-log.txt` / `window-memory-samples.txt` | 窗口执行、基线/峰值/结束基线/无残留 |
| `upstream-forced-horizon-run{1,2}.json` | 上游 CUDA 两次独立捕获（字节级相同） |
| `device/<case>/run{1,2}/*.bin` | 修正后设备输出（`max_horizon=2`，每 case 两次运行） |
| `device-before/<case>/run{1,2}/*.bin` | 修正前设备输出（HEAD 源，反向判别 272） |
| `overflow-result.json` | 逐 world 位型对照结论（设备/修前/上游，含 `float_max_worlds_total`） |
| `regression-result.json` | G5 回归逐 case 结果（排序行/摘要/位级文件数） |
| `device-regression/<case>/...` | G5 回归重跑输出（`out1/out2` 或 `witness1/witness2`，与冻结输出对比用） |
| `artifacts-sha256.json` | 本目录全部证据文件的 sha256 清单 |
| `receipt.json` | 任务回执（scope、数值、证据 sha256、门禁、unmet） |

## 未覆盖 / unmet

- 本片按 Arch 指定**只改设备**。宿主同语义误置位（`gjk/ccd.rs:225-228`、`gjk/contact.rs:126-130`）
  已由 Arch 登记独立片 `2026-10-25-g10-p5-host-ccd-overflow-bit-fix`（纯 CPU，G6 前），本片未改宿主。
- 强制 horizon 用**等价容量 = 2** 确定性触发；产品默认 `MJ_MAX_EPAHORIZON=24` 下未构造真实复杂 mesh 的
  >24 边 horizon 用例（判据覆盖的是溢出位语义本身，不声称覆盖所有真实触发输入）。
- 未验证 `_attach_face` 退化（`dist2==0`，overflow=0）的独立用例；该路径上游/设备修正后都给 0，
  本片只按代码路径说明，未单独构造。
- 只 Windows + RTX 4070 Ti SUPER（sm_89、nvcc 12.8）实测；未跑 Linux/T4、性能基准；
  GPU 结果不外推其它硬件/驱动。
- 不声称「精度达标」「上游完全等价」「双平台通过」；本片判据范围仅上述 overflow 位与回归。
