# A2-3 验证窗「开窗就绪包」（只准备；不排窗、不开窗）

- 任务：`W2/A2-3`（未登记执行卡；等 QA 复核 A2-2 PASS 后由 Chief 开窗）。
- 依据：A2-2 收口 `f9d76798`（verified）、Chief 2026-11-12 裁决（见 §7）、S4 窗口纪律（`g10-p5-branch-closure-validation`）。
- 本文件为**就绪材料**，不是窗口记录、不含任何通过声明。

## 1. 10 exe 冻结清单（A2 身份；sha256 全值）

| # | target | 文件 | bytes | sha256 |
| --- | --- | --- | --- | --- |
| 1 | g10_p5_branch_closure_cuda | `g10_p5_branch_closure_cuda-1b98cdc445ce5abc.exe` | 41,631,232 | `9fe3192bd35f7a68cf5c1d2943e95a69fab8f6abd91ad6d00ccfed4b418cac7a` |
| 2 | g10_p5_nvrtc_launcher | `g10_p5_nvrtc_launcher-719917fb0657e1fe.exe` | 41,714,176 | `6c306f58ef1eaf4a08659b4350ce5b2dc7d36e814c8358cf9b33a89a27310a99` |
| 3 | g10_p5_multicontact_cuda | `g10_p5_multicontact_cuda-3c769ef530070ffe.exe` | 41,698,816 | `4c035bfac51d0f269b9b684a74ae05060bba953156d03a1313b2da37cddc9ec3` |
| 4 | g10_p5_primitive_cuda | `g10_p5_primitive_cuda-0908b0f61ffbc599.exe` | 41,701,376 | `6f949c44ddb3991256f164889af95ad1f1a2eb5f5a377b798b06324613ae1944` |
| 5 | g10_p5_primitive_plane_cuda | `g10_p5_primitive_plane_cuda-b1e8897a38248f9f.exe` | 41,573,376 | `83a6864fa9206a0cb27224bdbc237abf342833083502b08b575ae31fb270b0d4` |
| 6 | g10_p5_primitive_sphere_capsule_cuda | `g10_p5_primitive_sphere_capsule_cuda-effe95bf0fed2063.exe` | 41,562,112 | `b4eaeaa4922c368d7c653e1e40ddc1ab075de70f7f16f5accdd3c025b02a8f15` |
| 7 | g10_p5_primitive_branches_cuda | `g10_p5_primitive_branches_cuda-555175a2d3995ff7.exe` | 41,638,400 | `3d2fb1e36105c80fc58a81efe2a81f41f2751c027a8c8c4ea58593434c061e74` |
| 8 | g10_p5_mesh_multicontact_m2_cuda | `g10_p5_mesh_multicontact_m2_cuda-e11ae6eb82e10bd5.exe` | 41,753,088 | `2e2afa00b78b47c7b03ac8a0176fd2662b4a3c2f03646eaf53e826d84efd47a8` |
| 9 | g10_p5_mesh_multicontact_m3_cuda | `g10_p5_mesh_multicontact_m3_cuda-31f06b6f3d1d660f.exe` | 41,756,160 | `9efcfe18c75e783712b0ce7ef70411bc06782f3a633c7c23a4559e63ccf97fca` |
| 10 | stage2_equivalence_window | `stage2_equivalence_window-b7e65259db9435d8.exe` | 42,277,888 | `013617a81ed55243b45cd5eccd9a71443aac625d8f19e8e559916400c2364c75` |

存放：`target/a2-freeze/`（10/10 已在盘、与上表逐字一致；A2-2 `refreeze-10-exe.json` copy_match 全 true）。
源码树 digest `266bbf94…`（29 个 `.cu`）；工具链 `+1.99.0-x86_64-pc-windows-msvc`；`MJWARP_MUJOCO_ROOT=target/toolchains/mujoco-3.12.0/package`。

## 2. 15 fixture 冻结清单（sha256 全值）

| # | fixture | sha256 |
| --- | --- | --- |
| 1 | fixtures/g10-p5-primitive-branches/plane-cylinder-degenerate.xml | `393ff1af9811b9badf6e9c13e707fa9a843e2b1d6f3face4f98b2cc7c6157583` |
| 2 | …/plane-box-corner-face-down.xml | `b158ce87c6fd68b79306823cd23f0e778482c425b5f617fcfbbd67fba9b22cab` |
| 3 | …/plane-box-corner-face-up.xml | `0223d2a925fde62ca114b57b3baf797b40654de25d74f943e9d9f716ed7c70b7` |
| 4 | …/plane-box-corner-tilt-x.xml | `2d9d360c6c87b53fd7d7a03d8889bdceb947c15982d2062052218efe4dbe3e8f` |
| 5 | …/plane-box-corner-tilt-y.xml | `159df60fc97aa53beb2df325fdebcc9327d22bbdef354f4d01bece8e460be8c5` |
| 6 | …/plane-box-corner-single.xml | `fc9f710c9accb679f65b98d6484caeaab91d2e05c5f7c3f59f03bd04691ecc1c` |
| 7 | …/plane-capsule-lying.xml | `9d43b44e8cd53a214bf43aa4958980e764ad8f87c08c74cd7d788879b58cf5c2` |
| 8 | …/plane-cylinder-deep-tilt.xml | `9801a8869fbb67eb2ca401956b66e1620d3e1b92f849ac6754262905fcfc6d84` |
| 9 | fixtures/g10-p5-mesh-multicontact/mesh-mesh-two-assets.xml | `1d71151364f4ba910308c80a42817f7a9b7fe843e42afb8396a3b5277f1b3bfb` |
| 10 | …/fan-cone-24.obj | `4bc34a51a60d76469a68c64e6132cfd1f5713072488ea8deefd44a4ca517f73b` |
| 11 | …/prism-20.obj | `a4477d3b080479e7356ac14bcd67c33272b2771eadd40b14fe3f40360e92cee4` |
| 12 | …/mesh-limits-cone-box.xml | `6ce3552f2cab48fdbbb66a6d397790d18748dcb0a5a87b91186b5742e297f731` |
| 13 | …/mesh-limits-prism-box.xml | `1bee35cfa3378d96d18317ff75bbf4af0561f7bd71915574e2ea596bc8d4f3b5` |
| 14 | …/fan-cone-33.obj（A2 新增） | `589080586fac53cca91ff6d1355b4e92c95b4a83bd512caf4ced33546444ee0b` |
| 15 | …/prism-33.obj（A2 新增） | `bb1b05c276b3f78a0bf680ff64682a4abad72233b880f1ed19c14f060677c401` |

参考（2）：
- `docs/development-evidence/g10-p5-branch-closure-capture/upstream-branch-closure-gpu.json` — `24d89e92…`/16,932 B
- `docs/development-evidence/g10-p5-branch-closure-capture/upstream-mesh-limits-gpu.json` — `b6abb8fa5820c103f6d880936d6ae95fa60035e5b9ce49a0364b15de1def19ad`/4,581 B

## 3. fresh-freeze 步骤（A2-2 已完成，本窗只复用，不重建）

1. 环境：`+1.99.0-x86_64-pc-windows-msvc`；`MJWARP_MUJOCO_ROOT=target/toolchains/mujoco-3.12.0/package`。
2. 逐 target：`cargo test --locked --all-features --no-run --test <target>`（10 个）。
3. 解析 `Executable tests\<t>.rs (target\debug\deps\<t>-<hash>.exe)` → 复制到 `target/a2-freeze/`。
4. 复制后**重哈希**，与冻结表逐项对（10/10 copy_match=true；已做）。
5. 冻结后到开窗前：`git diff <freeze_commit> -- src/` 必须为空；exe mtime 晚于构建开始、冻结后未重建。
6. 冻结产物不提交（target/），只提交 sha 清单（已在 A2-2 证据）。

## 4. runner 命令（窗口内逐实体）

- 9 个设备 exe（顺序固定，单进程单实体）：
  `target\a2-freeze\<file>.exe --ignored --nocapture --test-threads=1`
  环境：`PATH` 前置 `target\toolchains\cuda-12.8.1\bin`（NVRTC DLL，S4 同法）。
- stage2（record-C，只跑+落盘，不比对）：
  `target\a2-freeze\stage2_equivalence_window-b7e65259db9435d8.exe --ignored --nocapture --test-threads=1 window_product_equivalence_dump`
- runner 脚本不入库不重写历史：新 runner 可放 `target/a2-window/`，逐实体 exit code/stdout/stderr 落盘 + `SHA_OK` 行写 `sha-guards.txt`/`facts.txt`。
- guards：10 exe + 15 fixture + 2 参考逐项 sha（本文件 §1/§2）；不一致 → **abort exit 9**。

## 5. fail-stop 条件（任一命中即停；不重试、不放宽）

1. 任一 exe exit code ≠ 0 / 崩溃 / 超时（单实体 >300 s）。
2. 任一 sha 守卫不符；guard 表缺失；冻结后重建过 exe。
3. 窗内检测到 `cargo`/`rustc`/`cl`/`nvcc` 进程或新编译产物（零 Rust 编译被破坏）。
4. monitor 出现污染样本（util/free 越界、外部 compute-app、温度异常）；最长 clean 断链。
5. free 低于 floor；三点基线不一致（pre/post 基线偏差超阈）。
6. 判据失败：M2/M3/S4 branch 任一 failures≠0；U7/U8 pass_rule 任一条件不满足
   （nacon 零容差/结构键无法配对/字段超容差/overflow≠0/efc_address≠-1/geom 序不对）。
7. 任何结论需要放宽容差/改参考才能过 → 停，报 Chief，不自行处置。
8. 出现无法归因的数值差异 → 停、留档、报 Chief。

## 6. 预计时长 / 资源

- 10 实体顺序执行：S4 实测 202.1 s（9 实体）→ 10 实体约 **220–300 s**；A2 只加 U7/U8 判据与少量断言，
  预估 **240–360 s**（含 JIT/launch）；加 runner 基线采样/守卫/收尾约 **≤10 min** 整窗。
- 显存：单进程峰值 ~1.5 GiB；基线 free ≈12 GiB → 预计 free_min ≥11 GiB > floor（开窗按实测重算）。
- 前置构建（10 exe）**已在 A2-2 完成**，窗口内零构建。

## 7. 开窗前四前提逐项勾表（Chief 执行/留痕）

| # | 前提 | 判据 | 状态（待填） |
| --- | --- | --- | --- |
| 1 | fresh freeze | 10/10 exe sha 与 §1 一致；`src/` 自冻结后零 diff；无重建 | ☐ |
| 2 | monitor 60/60 | 100 ms 采样、连续 60 干净样本、compute-apps 过滤后 0、util/free 阈值取严 | ☐ |
| 3 | 三点基线 | pre ×3（util/used/free/temp/driver）记录并贴群 | ☐ |
| 4 | floor 取严 | `free≥10240 MiB` 且 `floor=min(10240, baseline_free−3072)`；取更严 | ☐ |
| 附 | QA 复核 A2-2 PASS | QA 复核结论落卡 | ☐ |
| 附 | HEAD/材料留痕 | 窗口 HEAD、guards 表、命令、fail-stop 已贴「开发团队」群 | ☐ |

## 8. 窗口内零 Rust 编译保证

- 窗内只执行 `target/a2-freeze/` 已冻结 exe；不调用 cargo/rustc/cl/nvcc，不重建任何 Rust 产物。
- runner 开跑前做进程 precheck（上述 4 个进程存在即 abort exit 9）；窗口期间 0.25 s 采样同样监控。
- NVRTC 在 launcher 内的 PTX 编译是**产品既有运行路径**（S4 同款），不是 Rust 编译；其产物不落仓。
- 冻结后到开窗前若源码/依赖任何变化 → 冻结作废，必须重走 fresh freeze（本包失效）。

## 9. Chief 2026-11-12 裁决（待落 A2-2 证据/计划 append-only）

1. **over-32 设备 case 不进 A2-3 窗口**：host 确定性容量错误已由 A2-1 边界单测覆盖；device over-32 为 record-only、
   不进 pass/fail；不补 XML/MeshData/case，窗口保持最小。若 QA 要求 record-only 证据 → 另开 0 窗片、不占窗。
2. **NP02 inventory 全量重算不做**：保持 identity-only swap；scanner 基线（100 路径/33 缺口）与仓内 patched 清单
   （102/32）的差异如实挂 **NP5-03 面**；**不得宣称 inventory 全量一致**。
3. **新 exe 身份已变**：后续任何重建 Rust exe 的窗口（含渲染）都要重冻；写进 A2-3 计划与证据。

## 10. 非声明

- 本包不构成窗口记录/通过声明；不宣称 U7/U8 通过、mesh 容量缺口已清、与上游等价（over-32）、G10@P5 整族通过。
- 窗口由 Chief 按四前提开；本包不排窗、不开窗。
