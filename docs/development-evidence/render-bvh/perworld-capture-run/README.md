# 逐 world 渲染参考采集跑片（attempt2 成功）

任务：`2026-10-30-render-perworld-capture`（Arch；P8；GPU 采集窗）
前置：`2026-10-30-render-perworld-capture-authoring`（脚本 + 窗口期修复）、`2026-10-30-render-perworld-capture-window-material`（窗口材料）
上游缺口：`2026-10-28-render-per-world-geom-light-gap`

本片只采集、不比较、不对设备侧做任何断言；数值判据留给 per-world 实现片（`2026-10-30-render-perworld-impl-s1`）的验证窗。

## 1. 窗口事实

| 项 | 值 |
| --- | --- |
| shell | **Windows PowerShell 5.1.26100.7920**（`aggregate_sha256` 文化排序约束；本窗不产 aggregate，仍统一用 5.1） |
| attempt1（失败，原样保留） | 例外 monitor 60/60（util 4–48、free_min 13,047、floor 10,216、exit 0）；采集命令 **exit 1**，停在脚本场景自检（`expected exactly one scalar hinge joint`），**零渲染、零写盘**；记录见 `attempt1/` |
| attempt2 基线 | `monitor/pre-window-snapshot-attempt2.txt`：三点 min free **13,749** ⇒ `floor = 13,749 − 3,072 = 10,677`；filtered=0、build=0、树干净 |
| attempt2 例外 monitor | `monitor/monitor-pre-window-attempt2.csv`：**60/60**、util **2–39**、free_min **12,907**（≥ floor）、ge60=0、filtered=0、build=0、exit 0，`00:21:11–00:26:11` |
| 例外路径 | 用户签 `authoring_only_exception_util_lt_60`（2026-10-31T00:00+08:00，via Chief），`-MaxUtil 60`、**不是常规门槛通过** |
| 采集命令 | 1 次：`--worlds 3 --hinge-deg "0,20,40" --use-fast-math false --use-shadows true --repeat 2`，**exit 0**（日志 `logs/capture.log`） |
| 零编译 | 窗口内未跑 cargo/rustc（只有 monitor + 一条 pinned python 采集） |
| 窗口后复查 | free 12,892 MiB（≥ floor）、build 进程 0、树干净 |

**例外生效口径（机器重算，`monitor/exception-monitor-recompute.json`）**：60 samples、longest_clean 60、util_min 2 / util_max 39、**ge40=0**、ge60=0、free_min 12,907、filtered/build 0 ⇒ `exception_load_bearing=false`、`exception_effectively_not_load_bearing=true`（60 个样本全部 <40，60 门槛未承担负载）。照实记录：不称为常规门槛通过，也不把例外当门槛证据。

## 2. 产物（`fixtures/render-bvh/scene-a/perworld/nw3-hinge02040/`）

| 文件 | 字节 | sha256 |
| --- | --- | --- |
| `rgb.npz` | 2,765,524 | `305f2bfdffc4e4efe406c4b8461a67ea115df73bf16d4303496fd3aa4eee56b7` |
| `depth.npz` | 922,336 | `cc065d0499f957d14d071fde295152c6e5cd0ca7989adb0cb7065226179903c5` |
| `seg.npz` | 1,843,924 | `b46b637097f9d53cf7d30a97d40c7511f5d266668c6dea45952826490e15a11a` |
| `state.npz` | 2,486 | `36c56b4c26ac4b36602d8fec31bb3a56b33031513b1e8660ded372c777b6e426` |
| `capture.json` | 8,836 | `b8f9e474b209dc2a9de8cae968961243cf989777580e6b58feeaab925ee9ec46` |

`capture.json` 关键字段（完整见 `materials/capture-fields.json`）：`nworld=3`、`hinge_deg=[0,20,40]`、`repeat=2`、`two_captures_bitwise_identical=true`、`legacy_reference_untouched=true`、`legacy_reference_files=79`、`state.qpos=(3,8)`、`geom_xpos=(3,7,3)`、`geom_xmat=(3,7,3,3)`、`nrender=1`、`samples_per_pixel=1`、`use_fast_math=false`、`use_shadows=true`、cameras `[cam-persp mode=0 320x240]`、`per_world_scope.covers=per-world geom poses`、`does_not_cover=per-world light poses`。

口径注：`capture.json.task_id` 是 authoring 卡 id（脚本 `TASK_ID` 按 QA O1 对齐到 authoring 卡）；本证据 `receipt.json` 的 `/task_id` 是本采集卡 id。

## 3. 校验

- **两跑确定性**：脚本 `--repeat 2` 逐数组 `np.array_equal` 且 npz 字节相等，否则非零退出、不宣称；本次 exit 0 且 `two_captures_bitwise_identical=true`。
- **legacy 对拍**：采集前后 manifest 均 **79** 条（69 既有渲染参考 + 10 个 SDF 采集参考文件），`legacy_reference_untouched=true`。
- **三处一致**：磁盘 sha（`materials/products.txt`）= `capture.json.files_sha256` = 脚本 stdout。
- **写盘边界**：`--out-dir` 在 `scene-a/perworld/` 子树内且采集前不存在；窗口内只写该目录。

## 4. 范围与不宣称

- 参考只覆盖 **per-world geom 位姿**；`light_xpos/light_xdir` 一并落盘仅为完整性，**不宣称** per-world 灯位姿。
- 本窗**不做**设备侧比较、**不产**数值验收结论、不冻结 guards、不改判据/容差。
- 未覆盖：AA/spp>1（spp=1）、per-world 灯位姿、多场景。

## 5. 证据清单

| 文件 | 内容 |
| --- | --- |
| `README.md` | 本文 |
| `receipt.json` | 任务 id、窗口事实、例外口径、产物 sha、脚本/场景身份 |
| `manifest.txt` | 本目录除自身外全部文件的 sha256/bytes |
| `monitor/monitor-pre-window-attempt2.csv` | 例外 monitor 原始 CSV（60 samples） |
| `monitor/pre-window-snapshot-attempt2.txt` | attempt2 三点基线快照 + 构建/树状态 |
| `monitor/exception-monitor-recompute.json` | 独立重算（ge40/ge60/free/filtered/build/load-bearing） |
| `attempt1/*` | attempt1 失败留档（monitor CSV、快照、重算、capture-failure.txt），原样保留 |
| `logs/capture.log` | 采集命令原样输出（含 stdout JSON） |
| `materials/products.txt` | 产物 sha256/bytes |
| `materials/capture-fields.json` | `capture.json` 关键字段摘录 |
| `materials/script-identity.txt` | 脚本/dry-run/环境/场景身份 |
| `materials/dryrun-plan-attempt2.json` | attempt2 开窗前 dry-run 计划（离线，含静态模型自检） |
