# perworld L v2 G-L2 验证窗执行（guards 107 冻结 + canonical 单次运行；**判定：未通过**）

- 任务：`2026-11-11-render-perworld-l-v2-validate-window-g-l2`（owner Backend；p70；P8-渲染与BVH；登记 `1f02532d`；attempt-1 处置记录 `43284a9e`）。
- 授权：S-3 双签 `ea91108e` + QA 复核 `dd9b9bc6` PASS（signed floor **9,956 MiB**；签名对象 README `437dc793…`，签名后 HEAD 版 `ddb15baa…`/21,718 B 属预期）；Chief G-L2 attempt 2 token 2026-11-11 03:52（**单次、失败即停、不重试、不放宽**）。
- 执行者：**Backend（唯一写者）**；整窗 repo-write-lock `owner=Backend`（pid 88116，收口提交后释放）。
- 产物：本证据目录 + `scripts/validate/render-bvh/guards-perworld-l2-light.json`（sha `fad2ed37…`/43,729 B，未跟踪→本卡提交）。
- **最终判定：执行完成、判定未通过（判据 A 失败）**：实体 `render_per_world_light` 在 world0 cam1 有 **2 个 rgb 像素 1 字节差**（`rgb_mismatched_pixels=2`、`rgb_max_byte_diff=1`）；depth/seg 位级 0 差；判据 C **PASS**、§2.4 对账 **Compared**；判据 B、两跑 sha、world 隔离因 A 断言中止而**未评估**。转 RCA（Chief 另派）。
- 非声明：本卡只出验证事实；不宣称 G-L2 通过、不宣称产品与冻结上游实现等价、不宣称灯字段验收通过、不宣称渲染数值等价；不放宽任何判据/阈值/参考，不重跑。

## 0. 判定摘要（一句话）

| 项 | 结果 |
| --- | --- |
| 12 个冻结 exe 单次运行 | 11/12 exit 0 全绿；实体 `render_per_world_light` **exit 101** |
| 实体判据 C（参考侧） | **PASS**：C1 38,285 / 40,644；C2 depth 35,828 / 33,883、seg 16,041 / 16,754；两套两对 ≥576；诊断 campersp depth/seg `[0,0]/[0,0]`、camhinge rgb `[50,363, 52,329]` |
| §2.4 对账 | **`capture.json C1 交叉核对 = Compared`** |
| 实体判据 A | **FAIL**：world0 cam0 全 0；world0 cam1 **rgb 2 像素 / 最大 1 字节**，depth 0 ULP、seg 0 |
| 实体判据 B / 两跑 sha / world 隔离 | **未评估**（断言在 A 处中止；源码顺序 A→B→两跑 sha→隔离） |
| 错误路径自证用例 `pilot_c1_crosscheck_matches_readout` | PASS（pilot 旧参考无 §2.4 字段 → 显式 `Skipped`，符合预期） |
| 运行前后 `live==guard==stash` | 12/12，运行前后均成立 |
| 冻结后编译 | 0（build=0、residual=0；guards 冻结为唯一编译阶段） |

## 1. 时间线与轮次

| 时间（+08:00） | 事件 | 结果 |
| --- | --- | --- |
| 11-11 03:25 | attempt-1 token | 该 token 把构建+冻结排在 monitor 之前，与已签 S-3 纪律不符；attempt-1 monitor 不成立（354 samples、longest_clean 21<60、util 1–53% 峰值≥40、free 11,283–12,232 ≥ floor 9,956、compute_filtered=0、build=0）；**未运行任何 exe** → Chief 03:52 裁决作废；attempt-1 guards 产物移出仓保存 `target/render-bvh/perworld-l-v2-gl2/attempt1/guards-perworld-l2-light.json`（sha `7d755a73…`/43,729 B，不提交、不删除） |
| 03:49:56 | attempt-2 T0 冻结 | HEAD `43284a9e`、树 clean、锁 owner=Backend、0 构建、0 引用缺失；指纹全对（见 §2） |
| 03:50:01–04:15:08 | monitor（先行） | **exit 0、ready=true**：297 samples、clean 256、**longest_clean=60**、util 1–50%（clean 段 <40）、free 11,776–12,448 MiB、`compute_filtered_max=0`、`build_max=0`；验证窗**不适用** authoring 例外 |
| 04:15:17–04:15:26 | 构建 + guards 冻结（唯一编译阶段） | `cargo test --locked --no-run` exit 0（缓存热 0.43 s）；冻结器 exit 0/wall 5.04 s、**SELFCHECK PASS 18/18**；`GUARDS_FROZEN total=107 exes=12 inputs=93 lists=2 aggregate=c213c3cf… guards=fad2ed37…/43,729 B`；12/12 `live==guard==stash`；冻结后零编译 |
| 04:15:55 | 三点基线（沿用） | free 12,143×3、util 4/6/4%、temp 27℃ → `floor_effective=max(9,956, 9,071)=9,956 MiB`；04:17 Chief **GO** |
| 04:16:39 | run-0 | 直接启动冻结 exe 未注入 pinned CUDA bin → `NvrtcUnavailable` exit 101、0.209 s、未进测试体 → **INVALID_ROUND #1**（0 轮消耗） |
| 04:19:5x | launcher preflight abort | 我的运行脚本旧 suite 路径写错，preflight 中止、**0 exe 启动**（非 body 轮） |
| 04:20:16 / 04:20:51 | run-1 | 仅修 PATH、**漏 criterion v2.2 两个 outline env** → render_e2e 报 183 个冻结 outline 像素为 mismatch（**假红**：数值与 guards `outline_lists[0]` rows 183/first_pixel 28969/exact_max_ulp 73 完全一致）→ **INVALID_ROUND #2**（0 轮消耗；exe1 PASS 亦丢弃） |
| 04:19:44–04:19:54 | run-1 前基线重取 | free 12,148×3 → floor_effective 9,956；本记录保留 |
| 04:27:05–04:27:15 | run3 三点基线 | free 12,138/12,143/12,143 MiB、util 3/4/2%、temp 26/26/26℃、build=0、树 clean → `baseline_free=12,138`、`onspot=9,066`、**`floor_effective=max(9,956, 9,066)=9,956 MiB`**；运行启动距基线 5.3 s |
| 04:27:20 起（wall 55.429 s） | **run3 = canonical 有效轮** | 完整 env + pre/post outline 校验 + 12 exe 各一次、exit code 可靠捕获；结果见 §6（11 绿 + 实体 A FAIL） |
| 04:30 | Chief 裁决 | run3 记为有效 canonical 轮；G-L2 判定 = 判据 A FAIL；不通过、不放宽、不重跑；转 RCA |

## 2. T0 冻结（`window/t0-freeze.txt`）

HEAD `43284a9e874d172c1b0760d87f8c58db843283e8`（token 现场 `dd9b9bc6` + 卡登记 `1f02532d` + attempt-1 处置 `43284a9e`，均为卡 JSON 治理提交）；树 clean（仅 `?? guards-perworld-l2-light.json` 于冻结时产生）；锁 `owner=Backend` pid 88116；0 构建进程。指纹（实测吻合）：

| 项 | sha256 | bytes |
| --- | --- | --- |
| 参考 rgb.npz | `602574c2…493f` | 5,531,026 |
| 参考 depth.npz | `82eec6da…e92d` | 1,844,650 |
| 参考 seg.npz | `c3f09b16…42f0` | 3,687,826 |
| 参考 state.npz | `a1c5a321…6aca` | 2,486 |
| 参考 capture.json | `8b53e122…20a9` | 12,251 |
| 旧 guards JSON | `ca3ecd5c…4a15a` | 38,659 |
| 冻结器 | `14d82190…76c6b` | 20,802 |
| spec.md | `bb064fe0…a924d` | 39,379 |
| 实体最终提交 | `bab01507` | — |
| upstream | `71da24d956378a87a703b6e1442b13aec0c4ac29` clean | — |
| venv | 3.12.13 / 1.15.0 / 3.15.0 / 3.15.0 / 2.5.4 | — |

12 exe 顺序 = 旧 guards `entity_order`（11 回归）+ 新实体 `render_per_world_light`（追加末位）。

## 3. monitor（`window/monitor.csv`、`monitor-summary.json`、`monitor-exit-code.txt`）

`-Required 60 -MaxMinutes 30 -MaxUtil 40`，冻结后零编译：**exit 0、ready=true**；297 samples、clean 256、**longest_clean=60**、util 1–50 %（clean 样本均 <40）、free 11,776–12,448 MiB、`compute_filtered_max=0`、`build_max=0`、`free_below_signed_floor=false`。验证窗不适用 authoring 例外（util<60）——未使用。

## 4. 构建与 guards 冻结（`window/build-result.txt`、`cargo-artifacts.jsonl`、`freeze-*`、`guards-integrity.txt`）

- 构建（冻结阶段唯一编译点）：`cargo test --locked --no-run` exit 0、wall 0.43 s；12 个 perworld exe 全部解析；产物清单 `window/cargo-artifacts.jsonl`；构建 stderr `window/cargo-build-stderr.log`。
- 冻结：`freeze-guards-perworld-l2-light.ps1 -Execute -AllowOldStashDrift -WorkDir target\render-bvh\perworld-l2-window`（理由=旧 stash 已知漂移、仅历史参照、旧 guards JSON 不改）exit 0 / wall 5.04 s / **SELFCHECK PASS 18/18**。
- `GUARDS_FROZEN suite=render-perworld-l2-light total=107 exes=12 inputs=93 lists=2 aggregate_sha256=c213c3cf4eb3bbe1afe24b960e95e0c20a54b6b28626a072edfc71fb7e5590e2 guards_sha256=fad2ed37ea3d1e17a5f0e3b13158eb3e66d185b4dd7f09682d9e143cbeea9e2d bytes=43729`。
- 独立复核：guards sha/bytes、aggregate 复算、12/12 `live==guard==stash`、107=12+93+2、旧 suite `ca3ecd5c…` 只读未变、旧 stash 未改。
- 已知漂移（只报告不修复）：旧 stash `render_shadow_reference` recorded 101,102,921 B/`ce513922…091fa` vs actual 100,859,152 B/`4500bbe3…a2ae`（last_write 2026-11-03T23:09:54）。
- 冻结后零编译（build=0、无 residual 进程），此后再未编译。

## 5. 三点基线（`window/baseline-run3-three-points.txt`；裁决用本轮）

| 点 | 时间 | used | free | util | temp |
| --- | --- | --- | --- | --- | --- |
| 1 | 04:27:05.173 | 3,924 MiB | **12,138 MiB** | 3 % | 26 ℃ |
| 2 | 04:27:10.276 | 3,919 MiB | 12,143 MiB | 4 % | 26 ℃ |
| 3 | 04:27:15.350 | 3,919 MiB | 12,143 MiB | 2 % | 26 ℃ |

`baseline_free=12,138`、`onspot_floor=9,066`、**`floor_effective=max(signed 9,956, 9,066)=9,956 MiB`**；三点 free≥floor、util<40、build=0、树 clean。canonical 运行启动距基线 **5.3 s**（<5 分钟）。run-1 前另有一次重取基线记录于 `window/baseline-run2-three-points.txt`（无效轮，仅留档）。

## 6. canonical body 运行（run3；`window/run/`）

### 6.1 环境（逐行复刻 `scripts/validate/render-bvh/run-window.ps1`）

`run-window.ps1` 全部 `$env:` 赋值共 7 处：291-292（outline 成对导出）、307-308（PATH 前置）、346/367-368（body 后恢复/清空）。本运行在 body 前实际导出（`window/body-env.txt`、`window/canonical-env-checklist.txt`）：

```text
$env:PATH = "<repo>/target/toolchains/cuda-12.8.1/bin;" + $env:PATH
$env:MJWARP_RENDER_OUTLINE_LIST_DIR = "<repo>/docs/development-evidence/render-bvh/sdf-validate-window/attempts/attempt5/outline"
$env:MJWARP_REQUIRE_OUTLINE_CRITERION = "1"
```

env 自检 dry-run（先于启动）：`ENV_SELFCHECK_PASS failures=0`；12 exe 逐项环境需求列出（6 个 scene-a 回归需 outline 判据环境，其余仅需 PATH）；该 12 个 exe 不读取其它 MJWARP_* 环境变量（grep `tests/render_*.rs` + `tests/support/render_bvh`，仅 `outline.rs` 有 env 读取；hfield/shadow 的 `MJWARP_CAPTURE_AT` 仅为注释）。

### 6.2 pre/post outline 列表校验（`run-window.ps1:280-298` 语义）

| arm | sha256 | bytes | rows | first_pixel | exact_max_ulp | sidecar | pre | post |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| recipe | `cfd1038a…724ee` | 10,099 | 183 | 28,969 | 73 | `8a80fb0a…cf7948`/5,027 B | OK | OK |
| aa2 | `14893fc6…c8eab` | 9,509 | 172 | 29,448 | 91 | `f23ca697…0fc303`/4,734 B | OK | OK |

`exact_max_ulp ≤ 128` 均成立；两次校验均 OK，无 drift。canary（`window/run/exe-logs/render_e2e.stdout.log`）：

`outline[recipe] cam1: list_rows=183 outside_mismatch=0 in_list_differing=183 in_list_max_ulp=73 violations=0 exact_max_ulp=73`

### 6.3 12 个冻结 exe（`--ignored --test-threads=1 --nocapture`，直接冻结路径，零编译）

| # | entity | exit | wall (s) | test result |
| --- | --- | --- | --- | --- |
| 1 | render_per_world | 0 | 1.730 | ok 1 passed |
| 2 | render_e2e | 0 | 9.224 | ok 5 passed |
| 3 | render_kernel_rgb | 0 | 3.886 | ok 3 passed |
| 4 | render_kernel_depth_seg | 0 | 4.580 | ok 6 passed |
| 5 | render_kernel_flags | 0 | 10.213 | ok 6 passed |
| 6 | render_r10_multi_world | 0 | 2.547 | ok 2 passed |
| 7 | render_skybox_reference | 0 | 1.624 | ok 1 passed |
| 8 | render_context_lifecycle | 0 | 7.829 | ok 5 passed |
| 9 | render_mesh_bvh | 0 | 0.808 | ok 6 passed |
| 10 | render_hfield_reference | 0 | 2.658 | ok 2 passed |
| 11 | render_shadow_reference | 0 | 4.676 | ok 3 passed |
| 12 | **render_per_world_light** | **101** | 1.726 | **FAILED 1 passed / 1 failed** |

exit code 捕获可靠性：dotnet `ProcessStartInfo` + 异步读 stdout/stderr + `WaitForExit` + `ExitCode`；`window/launcher-exitcode-selftest.txt` 用无害命令自测（`cmd /c exit 0/7/101`，两机制均正确捕获）；本运行 11×0 + 1×101 与 test result 行逐条一致。

### 6.4 实体 `render_per_world_light` 明细（`window/run/exe-logs/render_per_world_light.*`）

- 判据 C（参考侧，产品数据前）：**PASS** — `c1_rgb_changed=[38285, 40644]`（magnitude）、`c2_depth_changed=[35828, 33883]`、`c2_seg_changed=[16041, 16754]`、`verdict=PassMargin`（两套两对 ≥576）；诊断 `campersp depth=[0,0] seg=[0,0]`、`camhinge rgb=[50363, 52329]`（仅记录、不参与判定）。
- §2.4 对账：**`capture.json C1 交叉核对 = Compared`**（非 Skipped）。
- 判据 A：**FAIL** — world0 cam0：rgb 0/0、depth 0/0 ULP、seg 0；**world0 cam1：`rgb_mismatched_pixels=2`、`rgb_max_byte_diff=1`**，depth 0 ULP、seg 0；panic `tests/render_per_world_light.rs:363`（`left: 2 right: 0`）。
- 判据 B、两跑 sha、world 隔离：**未评估**（测试在 A 首个失败断言中止；源码顺序 A(359-384)→B(386-393)→两跑 sha(395-411)→隔离(413+)）。
- 另一用例 `pilot_c1_crosscheck_matches_readout`：PASS（pilot 参考无 §2.4 字段 → 显式 `Skipped`，符合预期）。
- 结论：真实判据 A 失败（非环境/启动问题：同 env 下 11 回归全绿、canary 证明 outline 判据生效、§2.4 正常 Compared），按 fail-stop 停手，未重试、未放宽，转 RCA。

### 6.5 运行后完整性

12/12 `live==guard==stash`；guards sha `fad2ed37…`/43,729 B 未变；aggregate `c213c3cf…`；旧 suite `ca3ecd5c…` 只读未变；outline 列表 post 校验 OK；build_processes=0、residual `render_*`=0；树仅 `?? scripts/validate/render-bvh/guards-perworld-l2-light.json`；GPU 1–2 %/free 12,140–12,143 MiB/26–27 ℃。

## 7. 无效轮次与假红（`window/invalid-rounds/`）

| 轮次 | 时间 | 原因 | 消耗验证轮 | 处置 |
| --- | --- | --- | --- | --- |
| run-0 | 04:16:39 | 直接启动冻结 exe 未注入 pinned CUDA bin → `NvrtcUnavailable` exit 101、0.209 s、未进测试体 | 0 | **INVALID_ROUND #1**；记录 `window/window-incidents.txt` |
| launcher preflight abort | 04:19:5x | 我的本地 runner 脚本旧 suite 路径写错，preflight 中止，0 exe 启动 | 0 | 非 body 轮；`invalid-rounds/launch-attempt1-preflight-aborted.txt` |
| run-1 | 04:20:16/51 | 漏 criterion v2.2 两个 outline env → render_e2e 假红（183/73/28969 与冻结 outline 列表逐项一致） | 0 | **INVALID_ROUND #2**；`invalid-rounds/run1-rootcause-false-red-outline-env.txt`、`fail-stop-render_e2e.txt`（含 R4 更正）、run-summary-continuation（index 已按 R3 修正） |
| run3 | 04:27:20 | — | **有效** | canonical 轮；结果见 §6（判定未通过） |

附：`invalid-rounds/arch-rca-facts.txt`（exe 新旧身份对照与 S-1 后无重验记录，只读材料）与 `invalid-rounds/logs/`（run-1 的 render_per_world/render_e2e stdout/stderr 原文）。

## 8. R1–R5 落地物（Chief 2026-11-11 04:30 裁决）

| 项 | 落地物 | 结果 |
| --- | --- | --- |
| R1 退出码捕获 | `window/launcher-exitcode-selftest.txt` | `cmd /c exit 0/7/101` 两机制均正确捕获 PASS；本运行 11×0+1×101 与 test result 一致（自测为 run3 后补记，仅验机制） |
| R2 继承漂移记录 | `window/guards-input-drift.txt`、`window/actual-input-hashes.txt` | 93 条输入实测 **92 match / 1 drift / 0 missing**；唯一漂移 `fixtures/render-bvh/scene-e2e/camlight-model.json`：guards 记录 6,434 B/`470adbef…e26dc5`，当前 6,020 B/`e44786d6…e5ca9de`（旧 suite freeze 后由 `18ae7157` 修改，L2 冻结器原样继承旧 83 条输入；e2e 经 `include_str!` 编入当前字节）。**不重冻、不改旧 suite、不改冻结器**；本条为既有继承漂移，本轮不作输入绑定声明 |
| R3 index off-by-one | `window/invalid-rounds/run-summary-continuation.json` | 按 guards `entity_order` 真实序号修正（render_e2e 3→2）并留 `index_fix` 注记 |
| R4 fail-stop 文本更正 | `window/invalid-rounds/fail-stop-render_e2e.txt` | 追加：run-1 为 outline env 缺失假红、非产品回归、INVALID_ROUND #2、结果作废不复用 |
| R5 签名材料缺项 | `window/material-errata.txt` | S-3 `materials/command.txt` §5 body 命令缺 env 导出（PATH + 两 outline env + pre-body outline 校验）；G-L2 后 errata，本窗未改签名材料 |

## 9. 边界、门禁与证据自核

- `unchanged src/`、`unchanged tests/`、`unchanged fixtures/`（G-L1 参考 5 文件已在冻结前入库；本窗零改动）、`unchanged scripts/`（仅新增 `scripts/validate/render-bvh/guards-perworld-l2-light.json` 一个冻结级产物）；spec 目录、S-3 材料目录、G-L1 证据目录只读未变。
- 四门禁（`normify gate`）：fmt 通过 / clippy 通过 / clippy-cuda 通过 / **unit 1,387 过 0 败**。
- 证据卫生：UTF-8 无 BOM、LF；manifest 首行 `#` + `# base dir:`，行格式 `sha256  bytes  path`，不列自身；manifest 自核全部吻合。
- 环境/工具链：Windows x64 MSVC、rustc 1.99.0、RTX 4070 Ti SUPER、驱动 596.36、CUDA 12.8.1（NVRTC）、pinned upstream `71da24d9` clean。

## 10. 非声明

- 不宣称 G-L2 验证通过；本卡判定 = 未通过（实体判据 A 失败）。
- 不宣称产品与冻结上游实现等价；不宣称灯字段端到端验收通过；不宣称渲染数值等价；不宣称判别力恢复。
- 不据此改变任何判据、阈值、参考、相机或 recipe；不重跑、不挑选结果；实体 A 失败转 RCA（Chief 另派）。
