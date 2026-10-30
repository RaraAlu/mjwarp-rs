# U4 SDF validation window — attempt1/attempt2 key artifacts (INVALID_ROUND, not a pass)

任务：`2026-10-31-render-sdf-validate`（U4 验证窗；本目录是其 attempt1/attempt2 的**关键产物快照**）
状态：**两次 attempt 均 `INVALID_ROUND exit10`，U4 未通过（红线）**；U1b 已回滚（`07252cd2`），U1 保留。
本目录只搬入不可从代码复算的窗口产物；完整归档（9 实体日志、facts、samples、sha-guards、monitor 原文件等）
仍在 `target/render-bvh/sdf-validate-window/attempts/{attempt1,attempt2}/`（工作树，不入库）。

## 1. 为什么入库

窗口结论依赖的产物（guards 冻结身份、两份 monitor 原始 CSV、INVALID_ROUND 判定、9 实体结果）此前只在
`target/`，不可通过代码复现也不随仓库分发。本目录把它们按字节复制进证据树，便于独立复核与长期留痕；
`target/` 原件保留、未改名、未覆盖。

## 2. 两次 attempt 的身份

| 项 | attempt1 | attempt2 |
| --- | --- | --- |
| guards 文件 | `attempts/attempt1/guards-sdf-validate.json` | `attempts/attempt2/guards-sdf-validate.json` |
| sha256 | `0639f0ff5064bd2a7a44c1b9f6a29298b623b4d4901c9f089e0648a97591910d` | `d921caae93df0d911c51a64501f2427dcb58cb86f64a4158fd1f45b06933d726` |
| bytes | 33,651 | 33,651 |
| aggregate_sha256 | `02b2021376e27140552000a7124e4e5cff527ee558670677b6d58dcbdccf826f` | `a50e99c951c1a9277d8821f8aca9fcad2506ec5878183ff0f7ead0c53837e67a` |
| counts | 9 exe + 83 inputs = 92 | 9 exe + 83 inputs = 92 |
| 冻结 | 首次冻结 | **全新冻结**（attempt1 冻结不复用） |
| body | 8/9 exit 0，`render_sdf_reference` exit 101 | 同（复现） |
| 判定 | `INVALID_ROUND exit10` | `INVALID_ROUND exit10` |

## 3. monitor 证据（两份都在，绝不写成常规通过）

- `attempts/attempt1/monitor-pre-window.csv` — **常规 `util<40` monitor 失败**（354 样本、longest 28/60、`util` **3–76**、`free_min` 11,545 MiB、exit 20）；**原样保留**。
- `attempts/attempt1/monitor-pre-window-exception.csv` — 已签条件式 validation-only `util<60` 例外 monitor（60/60、**69 样本**、exit 0）。
- `attempts/attempt2/monitor-pre-window-attempt2.csv` — 常规 monitor 失败（354 样本、longest 41/60、`util` 1–53、`free_min` 11,467 MiB、exit 20）。
- `attempts/attempt2/monitor-pre-window-exception-attempt2.csv` — 例外 monitor（60/60、83 样本、exit 0）。

两次 approach 都是「先常规、失败才启用已签例外」；例外 load-bearing 独立重算：attempt1 `ge40_clean=9` ⇒ true，
attempt2 `ge40_clean=8` ⇒ true；`ge60` 样本（attempt1 2 个、attempt2 1 个）均在 60 连清段之外（预热期、streak 0），
证据里照实记录，不写 `ge60=0`，不写「常规通过」。

### 3b. 已签条件式 validation-only 例外（五要素明文块）

三次 attempt 的例外路径都对应同一份用户签署（2026-10-31，via Chief）：

```text
mode=validation_only_exception_util_lt_60
approved_by=user via Chief
approved_at=2026-10-31T00:55+08:00    # Chief 即时转达；卡片 UI 精确时间戳未暴露，转达时点 ±5 分钟
reason=desktop 3D noise (dwm / DSH GUI / msedgewebview2): regular monitor failed at the signed
       original threshold while free never breached the floor and filtered compute-apps = 0
original_normal_threshold_attempts_retained=
  attempt1: attempts/attempt1/monitor-pre-window.csv (354 samples, longest 28/60, util 3-76, exit 20)
  attempt2: attempts/attempt2/monitor-pre-window-attempt2.csv (354 samples, longest 41/60, util 1-53, exit 20)
  attempt3: attempts/attempt3/monitor-pre-window-attempt3.csv (355 samples, longest 27/60, util 1-84, exit 20)
not_a_normal_threshold_pass=true
authoring_exception_used=false
```

`-MaxUtil 60` 只用于例外 monitor（`monitor-pre-window-exception*.csv`，PowerShell 5.1）；常规 monitor 从未加
`-MaxUtil 60`、也从未加 `-OpenUtilLimit`。例外是否 load-bearing 由连清段内的 `ge40_clean` 独立重算证明
（attempt1 9、attempt2 8、attempt3 6），不把 `ge60` 预热样本算作通过条件。

>> 注：`attempts/attempt2/invalid-round-record.txt` 的 `failed_tests` 行是从 attempt1 抄来的旧值（3 个含 shadow），
>> 实际 attempt2 为 **3 passed / 2 failed**（`sdf_mesh`、`sdf_occluder`；shadow 在 `82dfa0c3` 的 u8 修后已绿）。
>> 该归档记录按「原样保留」不改字节，更正见 `materials/errata.txt`。

## 3c. attempt3（INVALID_ROUND；两个独立结论）

attempt3 在 `07252cd2` 回滚之后、球面 a-order fix（`8126868c`）之后执行，`INVALID_ROUND exit10`：

- **① 流程无效**：`U1b` 仍处回滚态 ⇒ host 继续拒绝 `gtype==8`，`render_sdf_reference` 的 5 个 GPU 用例
  全部在发射前 panic `Unsupported("S7b3 不含 SDF 求交")`。**本 attempt 的 SDF 侧不作为任何判据依据**；
  7/9 实体红（该实体 5 个用例 0 passed）。窗口准备 checklist 缺失项：开窗前未核对「host 解锁状态与 target 预期一致」。
- **② 真实参考冲突**：另 6 个 scene-a 回归实体（`render_e2e`、`render_kernel_depth_seg`、`render_kernel_flags`、
  `render_kernel_rgb`、`render_r10_multi_world`、`render_skybox_reference`）全部红在**同一批球面轮廓像素**
  （首差 pixel 28969 / (169,120)），depth `max_ulp=73`（>4）；这些像素在 fix 前位级 0 差异 ⇒
  scene-a 的 fast_math=false 参考要求 `a=(0,1,2)`，而 scene-sdf 的 fast_math=false 参考在 fix 片 RCA 中
  要求 `a=(1,0,2)`。**两份参考互不兼容，成因待只读 RCA**（本记录不写成因定论）。
- 窗口本身：常规 monitor 失败（355 样本、longest 27/60、util 1–84、free_min 11,429、exit 20，**原样保留**）；
  已签例外 monitor **60/60**（60 样本、util 15–43、free_min 11,548、`ge40_clean=6`、`ge60=0`、exit 0）；
  全新冻结 `guards sha 8b34570e…`/33,651 B/aggregate `013babf4…`（9 exe 全变、83 inputs 不变）；`-GuardsOnly` 92/92；
  body 2/9 exit 0（`render_context_lifecycle`、`render_mesh_bvh`）。
- **attempt3 冻结作废、不复用**；U4 仍**未通过**，判据/参考零改动。

## 4. `-GuardsOnly` 92/92 的来源（如实标注）

- attempt1：窗口内 `window/guard.log` 已含 `GUARDS_OK=92/92`（body 之前）；另有一次 `guards-only-run/`
  （`window_start=2026-10-31T01:52:41+08:00`，**晚于 body 结束**）——这是**窗口后补跑的复核**，
  `-GuardsOnly` 只做文件 sha 校验（CPU-only，不碰 GPU、不改变窗口判定）。两处时间在本目录以外，
  此处只作说明；该补跑记录未入库（`target/` 保留）。
- attempt2：`GUARDS_OK=92/92` 只来自窗口内 `window/guard.log`（body 前），**没有**窗口后补跑。

## 5. 偏差方向（attempt2 只读诊断）

`render_sdf_reference`：`cam0` 215/76,800、`cam1` 4/43,200；全量 `max |ULP|` = 191（cam0）/ 25（cam1）；
首差像素 (230,69)，参考 `seg=(2,5)` = sphere 轮廓。诊断是**双向偏差**：cam0 前 40 例 `product>ref` 16 例、
`product<ref` 24 例、ULP 7–71。RGB `mismatched=0`/`max_byte_diff=0`/`beyond_1_lsb=0`、seg `mismatched=0`。

## 6. 未做的事

不改判据/容差/参考、不缩相机、不跳球、不抬 `MAX_DEPTH_ULP`、不把「诊断已定位」当通过；本目录不含任何
「渲染数值验收通过 / 上游等价 / 精度达标」的声明。后续：sphere 轮廓 depth 只读 RCA → 独立修复片 →
U4 attempt=3（全新冻结）。
