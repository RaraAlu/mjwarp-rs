# 产品碰撞输入接线 S3 独立复核（QA）

- 任务：`2026-10-27-product-collision-inputs-s3-independent-check`（复核对象 `2026-10-27-product-collision-inputs-s3-entry-window`）
- 复核提交：`0c9497e4b21d8f5ca65583e6c37d846060348c0b`（25 文件；父 `41fa049c`）
- 复核时 HEAD：`152b16d8`（窗口任务登记）；工作区仅本复核任务与窗口任务的 `.normify` 记录被 start 修改
- 环境：Windows / 仓库 `D:\Rust\mjwarp-rs`；定向测试与门禁等价命令由 QA 侧独立重跑
- 结论：**pass / 阻断 0 / 观察 10（1 一般 + 9 建议）**；未改产品代码/证据/判据/参考/容差
- **窗口事实不在本次可核范围**：S3 窗口任务 `2026-10-27-product-collision-inputs-s3-window` 仅登记、尚未开窗，
  没有窗口日志/采样/冻结 exe/基线可核；本复核只核实现片与窗口候选材料（README §9.5）

## 1. 总体判定

S3 的核心主张我都独立核过并成立：产品入口是「先纯 CPU 校验、后设备探测」的显式设备入口，
三态回退不 panic、不重试；`run` 与 `CollisionArgs` 504 B 零改动；`nworld!=1` 在设备工作前被拒；
端到端 target 复用 M2/M3 判据支持文件（未改动，零放宽）并只读 M1/M3 冻结参考（sha 未变）；
新增 MJB fixtures 的 sha/字节/provenance 32×3 数组与冻结 bins 全部一致；门禁我独立复跑得到
**1006/0/376**，与归档一致。

两条需要你知道的事实：**① 有一次 GPU 开发自测发生在未签窗口之外（流程偏差）**——证据已如实标为
「开发自测、非验证窗、无窗口结论」（处理合规，但按 GPU 窗口纪律仍是偏差）；**② 我没有独立重跑
设备侧数值**（窗口外不自跑 GPU），12/12 bit_equal 只按开发自测日志与代码/判据链复核。

## 2. 入口与三态回退（代码 + 单测）

1. **顺序固定（代码结构）**：`run_product_collision` 先 `prepare_inputs`（`nworld=1` 检查 + S2
   `inputs_from_snapshot`/既有 `validate_collision_inputs`），失败即 `InvalidArgument`；
   Host 选择在此后**直接返回** `Host(Selected)`，不调用 `product_collision_availability()`、
   不取 `device_program()`、不触碰驱动/NVRTC；Device 选择才查可用性 → 取缓存程序 → 发射。
2. **三态**：① `cfg(not(feature="cuda"))` → `FeatureDisabled`（编译期分支）；② `narrowphase_backend_status()`
   的 `HostOnly(reason)` → `DeviceUnavailable(reason)`；③ 执行器错误 → `DeviceFailure(error.to_string())`；
   「探测通过但程序取不到」→ `DeviceFailure("设备程序不可用…")`。全程返回 `Ok(Host(..))`，无 panic、无重试。
3. **单测独立重跑**：
   - 默认 features（`cuda`→`cuda-probe`）：`cargo test --locked --lib product_entry` → **9 passed / 0 failed**，
     0.03 s；
   - `--no-default-features --features cuda-probe`：同样 **9 passed / 0 failed**，其中
     `production_availability_is_feature_disabled_without_cuda_feature` 覆盖 ① 生产分支。
   - 用例本身覆盖：Host 不碰设备、①②③ 各档原因、程序缺失、成功路径内核透传、`nworld=0/2` 拒绝、
     非法快照拒绝、cuda 构建可用性与缓存窄相状态一致；执行器为注入式 MockDevice。
4. **不 panic**：`product_entry.rs` 无 `unwrap/expect/panic!/todo!`（只有 `unwrap_or` 兜底）；
   `run` 零改动（`collision_gpu.rs` diff 仅 `+1 mod` + `+3 pub use` = 4 行新增）。

## 3. nworld 口径

`prepare_inputs` 首行拒绝 `meta.nworld != 1`（`InvalidArgument`，文本含「nworld=1」「另立」）。
因为它在 `run_product_collision` 的**第一步**、早于 `product_collision_availability()` 与
`device_program()`，所以拒绝发生在任何设备探测之前（代码顺序保证）。本片不支持多 world，
证据未宣称多 world（receipt `boundaries.multi_world_supported=false`）。

## 4. 端到端 target 与判据

- **判据零放宽是结构性的**：`tests/support/mesh_multicontact_compare_support.rs` **不在 S3 提交内**
  （`git show --name-only 0c9497e4` 无该路径），target 直接调用其中既有的
  `compare_case_with_geom_order`（该文件 275 行），判据仍是「位级优先 + dist 4 ULP@量级
  （ref=0→4 ULP@1.0）+ pos 4·ULP(scale) + frame 4 ULP@1.0 + 结构/参数零容差 + nacon 与参考 count
  零容差 + contact.geom 按 pinned 类型序 + overflow 全 0 + efc 全 -1」。没有新增容差或包装。
- **冻结参考零改动**：M1 `c83baffe…`、M3 `96e7895d…` 两文件 sha 与 target 内写死的守卫一致；
  `docs/development-evidence/mesh-multicontact-m1/m2/m3/` 三个目录在 `41fa049c..0c9497e4` diff 为空。
- **输入侧（宿主 CPU）我独立重跑了**：只跑 e2e target 里无需 GPU 的
  `s1_snapshot_of_pinned_mjb_matches_frozen_case_bins`（过滤掉设备用例，未开 GPU）→ **1 passed / 0 failed**，
  `mesh-box`/`mesh-cylinder`/`mesh-mesh` 各 `s1_fields=32/32 bins_match=true mismatches=0`；
  真实 pinned 3.12 DLL + 新 MJB → S1 快照 → S2 adapter 与冻结 M0b/M3 bins 逐字节一致。
- **设备侧（只按证据复核，未重跑）**：`logs/s3-targeted-native.log` 记录开发自测 2 passed、41.77 s：
  3 case 各 `nacon=4 = ref_count`、`bit_equal=12/12`、`max_dist_ulp=0 / max_pos_ulp=0 / max_frame_ulp=0`、
  `contact.geom=[1,0]/[1,0]/[0,1]`、`failures=0`。该数值未经 QA 独立重跑（避免窗口外 GPU 使用），
  窗口结论必须来自签核窗口对冻结 exe 的重跑。

## 5. 新 MJB fixtures 与 provenance

- 三份 MJB 在**证据目录** `docs/development-evidence/product-collision-inputs-wiring/fixtures/mjb-312/`
  （不在 repo `fixtures/`）：`mesh-box-312.mjb` 8800 B `70b2fdc3…`、`mesh-cylinder-312.mjb` 8815 B
  `ceef075c…`、`mesh-mesh-312.mjb` 12032 B `298b8089…`，与 receipt/README/target 内写死守卫三方一致。
- `provenance.json`（897 行，机器可读）记录 `mujoco=3.12.0 / python=3.12.13 / numpy=2.5.4`、
  `authoring_only=true`、3 case 的 XML/MJB/参考 sha、qpos（与冻结参考 qpos 精确相等，`qpos_bits` 为 f32 位型）、
  32 数组 × 3 = 96 项的 dtype/count/sha256/bin_sha256/bin_match/round_trip_match，`result=PASS`。
  我自写脚本复算 **36 项检查 0 问题**：MJB/XML/参考 sha+字节、qpos 与参考相等、96 项数组 sha/count
  与冻结 bins 逐项相符。
- 生成脚本 `scripts/generate_collision_inputs_mjb.py`（245 行）`PINNED_MUJOCO = "3.12.0"`，版本不符打印
  FAIL 并退出；生成日志 `array_checks=96 array_mismatches=0 RESULT=PASS`。测试只读 MJB、不启动 Python。

## 6. 门禁、范围、不变式、行数

- **门禁等价命令独立复跑**：fmt / clippy / clippy-cuda / clippy(`--features native-model-probe`) 全部 exit 0、
  0 warning；`cargo test --locked` **1006 passed / 0 failed / 376 ignored**，与
  `.normify/runs/2026-10-27-product-collision-inputs-s3-entry-window/unit.json` 一致（基线 997 + 9）。
  未重跑 `normify gate`（避免覆盖已 verified 任务的 runs 产物）。
- **范围**：S3 提交 25 文件（20 个非 `.normify`）逐条对声明范围 → **0 越界**。
  （Arch 消息里的「scope 17/0」是另一计数口径；我的计数是 25/20，越界结论一致。）
- **26 个 unchanged 路径**：`git diff 41fa049c..0c9497e4 -- <26 路径>` 输出为空
  （18 pinned 源、`abi.rs`、设备 `.cu`、两个 fixtures 目录、M0b/M1/M2/M3 证据目录）。
- **ABI**：`CollisionArgs` 504 B / 入口 6 / `DEVICE_SOURCES` 28 的断言随 unit 跑过并通过；`abi.rs` 未改；
  `Cargo.toml` 不在提交内（无新 feature）。
- **行数（LF 口径）**：`product_entry.rs` 174、`product_entry_tests.rs` 366、`collision_gpu.rs` 466（+4）、
  `backend.rs` 184（+19）、e2e target 162、support 122、adapter support 320（+1）、生成脚本 245、
  README 374 —— 均 ≤500；`provenance.json` 897 行、`receipt.json` 613 行为机器可读 JSON（见 O3）。

## 7. 观察

### O1（一般）｜GPU 开发自测发生在未签窗口之外（流程偏差）

`logs/s3-targeted-native.log` = 2 passed / 41.77 s，使用本机 RTX 4070 Ti SUPER + CUDA 12.8.1 NVRTC，
跑的是未冻结的 target（无窗口 guard/基线/冻结 exe）。按团队 GPU 窗口纪律（开窗前材料报签、窗口内零编译、
冻结 exe、guard/基线/采样），这属于**窗口外 GPU 使用**，是流程偏差。
**处理的合规部分**：证据已如实标注「本地开发自测（非验证窗）」「无冻结/无 guard」「不写窗口通过」
「窗口结论以窗口任务冻结 exe 的独立重跑为准」（README §9.3、receipt `dev_self_test.note`、
`window.opened=false`），窗口任务 note 也记录「仅一次本地开发自测、非验证窗」——没有把自测当窗口结论。
**建议**：在窗口材料/编排台账登记这次自测（时间、GPU、命令、用途、时长），并明确「后续 dev 自测也按
窗口外使用需报备」的口径；本片结论不受影响（不采用该数值作为窗口证据）。

### O2（建议）｜「单测纯 CPU、无 GPU」在 CUDA 环境下不精确

README §9.1 称单测「纯 CPU、无 GPU」。其中 `production_availability_follows_cached_narrowphase_status`
（`cfg(feature="cuda")`）会调用 `narrowphase_backend_status()` → `probe_backend_with_cuda()`：
当驱动/设备/NVRTC 都可用时会执行 `compile_program()`（NVRTC PTX 编译，**不建 CUDA context、不发射内核**）。
本机 PATH 无 NVRTC → 该探针在 `nvrtc_library_present()` 处提前返回，9 个用例 0.03 s 跑完；
但若在「CUDA bin 在 PATH」的机器上跑默认 unit 门禁，会发生驱动查询 + NVRTC 编译（耗时数秒）。
建议把措辞改成「不建 CUDA context、不发射内核、不依赖设备结果」，或该一致性用例改为注入式（不走真实探测）。

### O3（建议）｜机器可读 JSON 行数例外未在记录中注明

`provenance.json` **897 行**、`receipt.json` **613 行**，都超过 500 行；交付记录里这两项的
`machine_readable_exception` 字段缺失（S1 曾对 oracle/receipt 显式标注该字段），receipt 的 s3 段也没有注明。
建议在交付记录或 `receipt.s3` 增补 `machine_readable_exception: true` 并在 README §9.6 表格注明。

### O4（建议）｜README §9.1 行数写错

§9.1 写 `product_entry_tests.rs`「346 行」，实际 **366 行**（§9.6 与交付记录均为 366）；同文档数字不一致。

### O5（建议）｜README §9.3 类型名笔误

写的是 `ConfigInputsModelInput`，产品类型实际是 `CollisionInputsModelInput`。

### O6（建议）｜三态分类文档与代码不一致

README §9.1 的三态表把「探测通过但程序取不到」归在 ②（设备/程序不可用），但代码与单测实际返回
`DeviceFailure`（③ 类）；单测文件头注释也把它写进 ②，断言却是 `DeviceFailure`。建议把该情形明确归到 ③，
或把 ② 的措辞改成「设备不可用（探测阶段失败）」。

### O7（建议）｜`nworld` 拒绝用例没有「执行器零调用」断言

`rejects_non_single_world_before_device_work` 走的是生产入口，没有注入点，因此无法断言执行器调用次数；
真正断言「零调用」的是 ①/② 注入式回退用例。若记录/说明里写「单测断言执行器零调用」，需限定到注入用例；
`nworld` 的「先于设备探测」由 `prepare_inputs` 在 `run_product_collision` 第一步的代码顺序保证。

### O8（建议）｜端到端 target 不在默认门禁编译范围

e2e target 有 `#![cfg(all(feature="cuda", feature="native-model-probe"))]`，默认 gate（默认 features 无
`native-model-probe`）不编译它；本片靠定向自测/窗口覆盖。建议把「native 侧 clippy + e2e 编译」登记为
定向门禁项（与前两片 native 覆盖问题同型）。

### O9（建议）｜两份 S3 日志为 CRLF

`logs/s3-fmt-clippy.log`、`logs/s3-mjb-generation.log` 为 CRLF（均无 BOM），其余三份 LF；
建议统一 LF，便于跨机 sha 比对。

### O10（建议）｜「gate 跑两次结果相同」只留最终一次归档

`s3-gate.log` 注明 gate 跑过两次且结果/计数相同，但 `.normify/runs/<task>/` 覆盖写，只保留最终一次
（与前两片同型）。建议保留两次 run 目录或在 receipt 注明只归档最终树。

## 8. 证据完整性

- 独立复算脚本（本目录，可复现）：`verify_s3_checks.py`（13 项：receipt s3/顶层 task_id、provenance 结构、
  26 unchanged、scope、禁词、参考 sha）；`verify_provenance.py`（36 项：fixtures/XML/参考/qpos/96 数组）。
- `verify_s3_checks.json` / `verify_provenance.json` 均为 PASS / 0 problem。
- receipt `/s3/task_id` 正确；顶层 `task_id` 仍为 S1、`s2.task_id` 仍为 S2（未被覆盖）。
- S3 新增 5 份证据日志 UTF-8 无 BOM（2 份 CRLF 见 O9）；本复核目录全部 UTF-8/LF/无 BOM。
- 禁词扫描：S3 改动文本文件（`.rs/.md/.json/.py/.log`）6 条规定词条 **0 命中**；README 中
  「窗口通过」只出现在否定句；`receipt.s3.window.opened=false`、`boundaries.window_passed_claimed=false`。

## 9. 复现命令（QA 侧）

```
# 产品入口单测（默认 features / 无 cuda 构建两套）
cargo test --locked --lib product_entry
cargo test --locked --no-default-features --features cuda-probe --lib product_entry
# e2e target 的宿主侧用例（只读 pinned DLL，不跑设备用例、不开 GPU）
MJWARP_MUJOCO_DLL=target/toolchains/mujoco-3.12.0/package/bin/mujoco.dll \
  cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features native-model-probe \
  --test g10_p5_product_collision_wiring_cuda -- --ignored --test-threads=1 \
  s1_snapshot_of_pinned_mjb_matches_frozen_case_bins --nocapture
# 独立复算
target/render-bvh-precheck/venv312/Scripts/python.exe verify_s3_checks.py
target/render-bvh-precheck/venv312/Scripts/python.exe verify_provenance.py
# 门禁等价命令（不写 normify run 文件）
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo clippy --all-targets --locked --features cuda -- -D warnings
cargo test --locked
```

设备侧 e2e 用例（`product_entry_device_output_matches_m1_and_m3_frozen_references`）**本次未重跑**：
按窗口纪律应由签核窗口对冻结 exe 执行；开发自测日志见
`docs/development-evidence/product-collision-inputs-wiring/logs/s3-targeted-native.log`。

## 10. 边界与不宣称

本复核只做宿主侧只读核对：未开 GPU、未开窗口、未跑设备内核、未改产品代码/证据/判据/参考/容差。
S3 实现片本身不做窗口结论；本复核同样不对窗口结果、host↔device 行为一致性、精度目标、性能、
跨平台或任何验收结论作判断。窗口事实（10 实体 exit、冻结 exe sha、基线/floor、采样聚合、零编译等）
不在本次可核范围——窗口任务尚未开窗，待其收口后另行复核。
