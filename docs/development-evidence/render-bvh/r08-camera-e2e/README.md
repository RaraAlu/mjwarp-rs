# R08-S2 多相机选择矩阵 + 异构分辨率端到端（1 验证窗，无新捕获）

- 任务：`2026-10-28-render-bvh-r08-camera-e2e`（p89，P8-渲染与BVH，owner Backend）
- 上游冻结：`71da24d956378a87a703b6e1442b13aec0c4ac29`
- 参考来源：复用 S0b/S7b/S8 冻结件（本片**无新 authoring 捕获**、不启动 Python、不重渲染参考）
- 记录口径：本目录只写执行事实与数字，不用结论式褒扬措辞（不在此复写具体禁词字面）

## 1. 本片交付

| 类别 | 文件 | 行数 | 说明 |
| --- | --- | --- | --- |
| 测试 | `tests/render_r08_cameras.rs` | 320 | host 5 条 + GPU 6 条（默认 ignore，窗口内 `--ignored` 执行） |
| 测试辅助 | `tests/support/render_bvh/r08.rs` | 366 | `OutputValues` → 整缓冲 → `CameraOutputViews` 适配、选择矩阵参数、参考拼装、逐相机对账计数、证据 SHA-256 |
| 测试辅助 | `tests/support/render_bvh/r08_gpu.rs` | 321 | 窗口 GPU 用例：四形态矩阵对冻结参考、两次运行 SHA、隔离/乱序逐位、动态射线、异构 depth/seg、空选择 no-op |
| 产品 | `src/render/kernel/megakernel.rs` | 498（改前 498） | 空选择守卫 `Err(InvalidArray("render_cameras"))` → `Ok(())`，等行数替换 1 行 |
| 产品 | `src/render/kernel/renderer.rs` | 265（改前 265） | 同上，等行数替换 1 行 |

产品侧只有上述 2 行语义替换；判据、参考、容差、设备 `.cu`、`launch.rs`(500)、`src/model/render.rs`(500)、
`context.rs`(426)、`mod.rs`(374) 均未动。

提交（本片内）：`d62a007b`（CPU 侧测试与接线）、`5a6b6c33`（空选择 no-op + 定向用例）、
`f8a15ef1`（A2：测试侧断言顺序修正，`+14/−4`）、本证据提交。

## 2. 任务卡 acceptance 对应

| # | acceptance | 本片落点 |
| --- | --- | --- |
| ① | `cam_active` = None(All)/bool 掩码/名称列表/索引列表/空列表，五形态经产品链路可用；激活顺序 = `CameraPlan.cameras`；空列表可渲染（0 相机 0 射线，不 panic） | host 用例断言布局/adr/`camera_index` 语义；GPU 用例执行 All/Mask/Names/Indices 四形态与空选择 no-op |
| ② | 异构分辨率（320×240 / 240×180 同 context）+ depth/seg 通道异构 | GPU 用例逐相机对 S7b 同配方冻结参考（RGB ≤1 LSB、depth ≤4 ULP、seg 全等）；异构通道 `depth_adr=[0,-1]`、`seg_adr=[-1,0]` |
| ③ | 相机隔离：单相机子集渲染与组合渲染对应区段逐位相同；A 区段不被 B 改写 | `gpu_isolation_subset_and_reorder_bitwise`（subset_cam0 / subset_cam1 / reorder） |
| ④ | 射线路径：预计算射线 vs 内核动态重建（`rays=None`）对参考均在判据内；两次运行 SHA-256 相同 | `gpu_dynamic_rays_depth_seg_bitwise_matches_precomputed`、`gpu_two_runs_sha256_identical` |
| ⑤ | 窗口纪律：冻结 sha+字节、runner sha、3 点基线、compute-apps 过滤 0、open free≥10240、floor、0.25 s 采样、pre/post/结束基线、零 Rust 编译、attempt ≤2、日志 UTF-8 无 BOM | 见 §3（两轮分写）；原始文件在 `logs/attempt1/`、`logs/attempt2/`、`logs/gate-wait/` |
| ⑥ | 空闲时段与同窗闭合 | 两次 attempt 均在 2026-10-28 空闲块内（02:29 / 02:39 +08:00）执行；窗口本体 41.2 s / 40.9 s |
| ⑦ | 不变量：判据零放宽、参考/夹具/容差不动、`fixtures/precision` 与 T4 钉子不动、18 个 G01 pinned 源零改动、`DEVICE_SOURCES`=28、产品入口 6、`CollisionArgs` 504 B、`src/model/render.rs`(500) 不碰 | 见 §7 核对 |
| ⑧ | 证据：README + receipt（含 `/task_id`）+ 窗口日志/采样/守卫/manifest；禁词 0；新文件 ≤500 行 | 本目录；`receipt.json` 机器校验项 `/task_id` |

## 3. 窗口执行（两轮分写，原始文件见 `logs/`）

窗口 runner：`target/r08-s2-window/run-window-attempt1.ps1`，**14976 B / sha256
`c8786951674365cfabba62fa668e92254c58092c30214dba08b9a5d347f4e71a`**；签名后零编辑（两次 attempt 同一字节）。
runner 与 7 个冻结 exe 均在 `target/`（不入库），sha/字节登记见 `manifest.json` 与 `logs/attempt*/sha-guards.txt`。

| 项 | attempt1 | attempt2 |
| --- | --- | --- |
| attempt 编号 | 1 | 2（最后一个有效轮） |
| precheck（3 点 used/free/util/temp） | `4285/11777/35/31 ×3` | `4323/11739/35/31; 4323/11739/36/31; 4323/11739/36/31` |
| 基线（3 点） | `4285/11777/35/31 · 4285/11777/39/31 · 4290/11772/39/31` | `4323/11739/36/31 · 4323/11739/38/31 · 4323/11739/38/31` |
| baseline_free / floor（min(10240, free−3072)） | 11777 / **8705** | 11739 / **8667** |
| 窗口起止（+08:00） | 02:29:26.5586 → 02:30:10.5365 | 02:39:59.7093 → 02:40:42.8477 |
| entity 区间 / body | 02:29:27.6878 → 02:30:08.8851 / 41.2 s | 02:40:00.5172 → 02:40:41.4231 / 40.9 s |
| 采样数 / 间隔（median, min–max） | 146 / 0.2815 s（0.2781–0.3540） | 145 / 0.2816 s（0.2788–0.3052） |
| used peak / used_min / free_min | 4560 @seq21 / 4290 / 11502 | 4593 @seq22 / 4322 / 11469 |
| util_max / util≥80 streak | 43 / 0 | 36 / 0 |
| delta（vs 基线 / 首样本 / 样本最小）/ sustained | 275 / 260 / 270 / 275 | 270 / 248 / 271 / 270 |
| increment_exceeded / floor_breached / guard_aborted | false / false / false | false / false / false |
| 7 entity exit | r08_cameras **101**，其余 6 个 0 | **全部 0** |
| 新 target 结果 | 5 passed / 1 failed | **6 passed / 0 failed** |
| 零 Rust 编译（deps 快照） | true（1816→1816，max write 02:21:30） | true（1816→1816，max write 02:31:19） |
| 守卫 45/45（7 exe + 38 只读输入） | 45/45，0 mismatch | 45/45，0 mismatch |
| 窗后 7 exe 复查 / residual / 外部占用 | 7/7 / 0 / compute-apps raw 28 filtered 0 | 7/7 / 0 / raw 28 filtered 0 |
| 外部占用（窗外噪声，非本团队） | 窗内 util 35–43、cpu load 11%、无 cargo/rustc/nvcc | 窗内 util 35–36（post 12%）、cpu load 8%、无 cargo/rustc/nvcc |
| 产物 | 保留（`target/r08-s2-window/attempt1/`） | 保留（`target/r08-s2-window/attempt2/`） |

### 3.1 attempt1（有效轮，新 target 未过）

- `render_r08_cameras` exit **101**（5 passed / 1 failed）。失败断言是 `gpu_empty_selection_render_is_noop`
  的显存快照位置：`memory_before` 取在 `MegakernelRenderer::new()` **之前**，把 NVRTC 模块加载的
  **2 097 152 B（2 MiB）** 记为「空选择渲染分配」，失败值 left `(15809380352, 17170956288)`
  vs right `(15807283200, 17170956288)`。
- 该轮**产品 no-op 语义断言全部成立**：两个 renderer 均返回 `Ok`（各两次）、三通道缓冲未分配、
  无可读输出、不 panic。
- 判据数值全 0：RGB `beyond_1_lsb=0`(120000)、depth `mismatch=0 / max_ulp=0`(120000)、
  seg `mismatch=0`(120000)；隔离/乱序逐位相同；两次运行 SHA 相同 `f9fe1b9e…be688`；
  动态射线=预计算 `b09e2eec…0c31`；异构 depth/seg 对账成立。
- 6 个回归 target 全 0 failed。
- 记账：**有效轮但新 target 未过**；不重跑、不改判据。

### 3.2 A2 修正（仅测试侧）

- 提交 `f8a15ef1`：`tests/support/render_bvh/r08_gpu.rs` **1 file changed, +14/−4**；快照移到两个
  renderer 构造完成之后，no-op 断言只比较「构造后 → 渲染后」，同时保留构造前/构造后两点采样并打印。
- **未动**：产品码（`src/render/kernel/*`、`camera_output.rs`）、判据/参考/容差、runner。
- 只重建 `render_r08_cameras` 一个 exe（同 profile/命令）：
  **94 029 705 B / sha256 `918b5851efaf7f2494201419ae3f90d7bb39e612414d68b7a833bedd01952775`**；
  其余 6 个 exe **逐字节未变**（对照见 `logs/attempt*/sha-guards.txt`）。
- 静态对盘 45/45、0 mismatch；`guards-check/` 自检产物已刷新到新 sha（02:33，非 02:22 旧值）。

### 3.3 attempt2（最终轮）

- precheck → 基线 → 窗口 → 结束基线完整；7 entity 串行，**exit 全 0**；
  `render_r08_cameras` 7558 ms、`camera_rays` 1158、`context_lifecycle` 8060、`render_e2e` 8151、
  `depth_seg` 4168、`flags` 8752、`rgb` 3037。
- 采样 145 点，peak 4593 MiB（基线 +270）、free_min 11469 ≥ floor 8667、util_max 36、streak 0、
  residual 0、零 Rust 编译、窗后 7/7、守卫 45/45。
- 开窗前条件等待（非 precheck 轮次）：02:33–02:39 自采 `ca_raw=28 / filtered=0 / build=0`、
  free 11739–11765；250 ms 级观测 160 点中 `util≥40` 仅 14 点（8.75%，尖峰宽 2–6 点）；
  连续 20 个 250 ms 样本满足 `util<40 且 free≥10240` 后**只弹一次** runner（02:39:57），
  原始采样见 `logs/gate-wait/gate-samples.csv`。

## 4. 判据数字（attempt2，`logs/attempt2/render_r08_cameras.log`）

选择矩阵四形态（同一冻结参考、同配方）：

| 形态 | cameras | rgb_adr | rgb_pixels | beyond_1_lsb | depth_pixels | mismatch | max_ulp | seg_pairs | seg_mismatch | 输出 sha256 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| All（None） | 2 | [0, 76800] | 120000 | 0 | 120000 | 0 | 0 | 120000 | 0 | `f9fe1b9ef198e42d4f37ef4d325bbb7b8dbfdf16b4c4d25b624f1e69684be688` |
| Names（all） | 2 | [0, 76800] | 120000 | 0 | 120000 | 0 | 0 | 120000 | 0 | 同上（与 All 同 sha） |
| Mask（first） | 1 | [0] | 76800 | 0 | 76800 | 0 | 0 | 76800 | 0 | `cfc1a4b6bea08b1723f086e7deef8c036d182eda84c5d4f74284e67315789d66` |
| Indices（reversed） | 2 | [0, 43200] | 120000 | 0 | 120000 | 0 | 0 | 120000 | 0 | `7389143567d715af7447674c22791723f838cb25315a741f09cb13fba8c998b3` |

逐相机（pos0 / pos1）：cam0 76800、cam1 43200；两位置均 `beyond_1_lsb=0`、`depth mismatch=0/max_ulp=0`、
`seg mismatch=0`。两次运行输出 sha 相同（均 `f9fe1b9e…be688`）。
动态射线 vs 预计算：depth/seg 逐位相同，sha256 `b09e2eeccd48c880ad44e257ef8ecc2da84973af09cd7866fe63d4a3975a0c31`。

隔离（与组合渲染逐位比较）：subset_cam0 → cam0_rgb=76800 / cam1_rgb=0；subset_cam1 → 0 / 43200；
reorder → 76800 / 43200：三者均与组合渲染对应区段**逐位相同**。

## 5. 异构通道口径与产品缺口

- 窗口口径 = **分辨率异构（320×240 / 240×180 同 context）+ depth/seg 通道异构**
  （`depth_adr=[0,-1]`、`seg_adr=[-1,0]`，未启用通道按错误面拒绝读取）。
- **异构 RGB 是产品缺口**：`megakernel` 的三通道开关是**全局**（非逐相机），产品不支持
  「cam0 有 RGB / cam1 无 RGB」。本片**不改 kernel**；修复需按 S7b3 先例拆片子文件。
  backlog 已由 Arch 登记：`2026-10-28-render-per-camera-rgb-gap`（**不 start、不修**）。

## 6. 空选择发射口径（已批行为变更）

- `render_cameras` 空选择从 `InvalidArray("render_cameras")` 改为 **no-op `Ok(())`**，对齐上游
  `wp.launch(dim=0)`（空网格不发射、不写输出）；非空路径行为不变。
- 依据：R08-S2 任务卡 acceptance ①「空列表可渲染」；全仓无已签口径把该拒绝当契约；判据/参考/容差不变。
- 定向用例 `gpu_empty_selection_render_is_noop`：两个 renderer 均 `Ok`、三通道缓冲未分配、
  显存无增量、无可读输出、不 panic（attempt1 起即成立，attempt2 复跑成立）。

## 7. 不变量核对

| 项 | 期望 | 实测 |
| --- | --- | --- |
| `megakernel.rs` 行数（改前/改后） | 等行数替换，不增长 | 498 / 498 |
| `renderer.rs` 行数 | 等行数替换 | 265 / 265 |
| `launch.rs` | 500，不动 | 500（未改） |
| `src/model/render.rs` | 500，不动 | 500（未改） |
| 新文件行数 | ≤500 | 320 / 366 / 321 |
| `DEVICE_SOURCES` | 28 | 28（`src/runtime/collision_gpu/source.rs` 未改） |
| `CollisionArgs` | 504 B | 504（`abi_tests` 锁定，未改） |
| 判据/参考/容差/`fixtures/precision`/T4 钉子 | 零改动 | 零改动（本片 diff 见 manifest `changed`） |
| 设备 `.cu`、`context.rs`、`mod.rs` | 不动 | 未动 |

## 8. 门禁与定向测试（本片最终树）

| 门禁 | 结果 | wall_ms | 日志 |
| --- | --- | --- | --- |
| `cargo fmt --check` | exit 0 | 1613 | `logs/fmt-check.log`（空输出） |
| `cargo clippy --all-targets --locked -- -D warnings` | exit 0 | 451 | `logs/clippy.log` |
| `cargo clippy --all-targets --locked --features cuda -- -D warnings` | exit 0 | 393 | `logs/clippy-cuda.log` |
| `cargo test --locked` | exit 0；**1015 passed / 0 failed / 382 ignored**（125 个 result 行） | 11938 | `logs/unit.log` |
| `cargo test --locked --test render_r08_cameras`（host） | 5 passed / 0 failed / 6 ignored | 313 | `logs/directed-render-r08-cameras.log` |

窗口内 GPU 结果见 `logs/attempt2/`（7 entity 日志 + 采样 + 守卫 + facts + guard_aggregate）。

## 9. 证据格式与自检

- 文本证据 UTF-8 无 BOM、LF/CR=0；窗口日志为 runner/测试原样输出。
- 机器可读例外：`logs/unit.log`（2132+ 行）、`logs/attempt*/samples.csv`、`logs/attempt*/sha-guards.txt`、
  `logs/attempt*/guard_aggregate.json`、`receipt.json`、`manifest.json`。
- 禁词自检：任务卡 `forbid_claim` 三项（本目录不复写字面）命中 0；四个措辞约束词命中 0。

## 10. 未完成 / 后续

- 独立复核片：`2026-10-28-render-bvh-r08-camera-e2e-check`（由 Arch 排）。
- 异构 RGB 逐相机开关：backlog `2026-10-28-render-per-camera-rgb-gap`。
- 多 world（nworld>1）逐相机视图：归 R10；仓库卫生任务 `2026-10-28-repo-fixture-eol-attributes`
  与本片无关（S2 收口后单独排）。
