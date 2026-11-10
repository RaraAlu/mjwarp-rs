# perworld L v2 判据 A 像素级诊断 dump（诊断轮执行证据；**独立 diagnostic round，不计 G-L2 轮次**）

- 任务：`2026-11-11-render-perworld-l-v2-entity-diagnostic-dump`（owner Backend；p60；P8-渲染与BVH）。
- 授权：Chief 2026-11-11 诊断卡（phase-1 只写代码）+ **诊断轮 token 2026-11-11 04:50**（独立 diagnostic round：不计 G-L2 轮次、不进 G-L2 证据链、不改任何冻结件；单次、不重试）。
- 结论：诊断轮 **attempt 2 成功**——dump 已生成（`68b3f78e…`/51,388 B），实体仍按预期在判据 A 断言失败（exit 101）；attempt 1 因启动环境遗漏失败（无 dump、未消耗轮次）。
- 非声明：本证据不改变 G-L2 判定（G-L2 = 执行完成、判定未通过，判据 A FAIL 维持）；不宣称根因已定位；不放宽/不改判据、阈值、参考、相机、recipe；dump 未入库（留 `target/` 供 Arch 只读）；诊断 exe 不进 guards。

## 1. 诊断方法（代码已提交 `6dc5e5f6`）

- opt-in：`MJWARP_DIAG_PIXEL_DUMP=<输出 JSON 路径>`；未设置/为空时测试零副作用（无打印、无文件、无额外 GPU 工作）。
- 代码：`tests/support/render_bvh/light/diag.rs`（493 行；字段按 Chief 2026-11-11 修订：去掉参考侧 `|ref_f32*255−round|` 边界距离，改两侧字节+packed u32+0/255 饱和标记、3×3 邻域、seg geom/mat 归属与材质标量、depth 位型+ULP、ray/灯/相机位型；near-miss 改为同字节像素饱和区比例 + 最接近失配 5 像素分 geom 归属）；`tests/render_per_world_light.rs` 478→491 行（仅断言前加 opt-in hook，断言零改动）；`light.rs` 499 行未动。
- 触发位置：判据 A 断言之前（先算 `criterion_a_stats` → dump → 原断言循环），因此断言失败前 dump 必已写出。

## 2. attempt 1（失败，环境遗漏，无 dump、未消耗轮次）

| 项 | 记录 |
| --- | --- |
| 时间 | 2026-11-11 04:46 前后（`window/preflight-attempt1.txt`） |
| 命令 | `cargo test --features cuda-probe --test render_per_world_light -- --ignored --test-threads=1` |
| env | 只导出 `MJWARP_DIAG_PIXEL_DUMP`，**漏 pinned CUDA bin PATH** |
| 结果 | exit 101、wall 0.537 s；测试体前崩于 `tests/support/render_bvh/light.rs:187 build_rays 编译: Backend(NvrtcUnavailable)`；`pixel-dump.json` 未生成 |
| 处置 | 按 token「dump 未生成 → 停手报 Chief、不重试」→ 申请修正授权；记录 `window/failure-attempt1.txt`、`test-attempt1.*.log` |

## 3. attempt 2（成功，单次）

### 3.1 env 与自检（`window/body-env.txt`、`window/preflight2.txt`）

```text
$env:PATH = "D:\Rust\mjwarp-rs\target\toolchains\cuda-12.8.1\bin;" + $env:PATH
$env:MJWARP_DIAG_PIXEL_DUMP = "D:\Rust\mjwarp-rs\target\render-bvh\perworld-l-v2-gl2\diag\pixel-dump.json"
# 实体路径不触发 criterion v2.2 outline 比较，故未设两个 outline env
```

自检 PASS：PATH 首段 = pinned CUDA bin；`nvrtc64_120_0.dll` 86,728,192 B、`cudart64_12.dll` 573,952 B 均在；`where nvrtc64_120_0.dll` 命中 pinned bin；`cargo test --locked --features cuda-probe --test render_per_world_light --no-run` exit 0。诊断 exe（**新 build，非冻结件**）：`target/debug/deps/render_per_world_light-8f8e60a38d5a26c5.exe`，sha256 `b6a373d2c8a52b6b3fcd2db6d4678024944a2b973b00fc588248469eb78e184d`，92,353,025 B，mtime 2026-11-11T04:44:22。

### 3.2 单次运行（`window/test2.*.log`）

- 命令：`cargo test --locked --features cuda-probe --test render_per_world_light -- --ignored --test-threads=1`
- 结果：**exit 101（预期：判据 A 断言失败）**、wall **2.157 s**（cargo 起测）、test body 1.95 s
- `test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 16 filtered out`；panic `tests/render_per_world_light.rs:376 world0 cam1 rgb 不匹配像素 left 2 right 0`
- 另一用例 `pilot_c1_crosscheck_matches_readout`：ok
- **dump 在断言前写出**：`target/render-bvh/perworld-l-v2-gl2/diag/pixel-dump.json`，sha256 `68b3f78ed9586a83c47cfad8ab722407d27c8571cd82c92a5f9b4c6050326089`，**51,388 B**（dump 本体不入库，见 `window/pixel-dump-identity.txt`）

### 3.3 6 个 world×cam 摘要（dump `frames`）

| world | cam | rgb_mismatched_pixels | rgb_max_byte_diff | depth_mismatched_pixels | seg_mismatched_pixels |
| --- | --- | --- | --- | --- | --- |
| 0 | 0 (cam-persp) | 0 | 0 | 0 | 0 |
| 0 | 1 (cam-hinge) | **2** | 1 | 0 | 0 |
| 1 | 0 (cam-persp) | 0 | 0 | 0 | 0 |
| 1 | 1 (cam-hinge) | **3** | 1 | 0 | 0 |
| 2 | 0 (cam-persp) | 0 | 0 | 0 | 0 |
| 2 | 1 (cam-hinge) | 0 | 0 | 0 | 0 |

### 3.4 全部失配像素（5 个；dump 比 G-L2 断言可见的 2 个多 3 个）

| # | world/cam | pixel (x,y) | 产品 bytes (packed u32) | 参考 bytes (packed u32) | 差异通道 | geom/geom_type/mat |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | 0/1 | 13268 (148,41) | `[255,28,29]` `0xffff1c1d` | `[255,28,28]` `0xffff1c1c` | B 29→28 | 3 / 5 (cylinder) / 0 |
| 2 | 0/1 | 31154 (114,97) | `[255,18,19]` `0xffff1213` | `[255,18,18]` `0xffff1212` | B 19→18 | 3 / 5 / 0 |
| 3 | 1/1 | 27360 (160,85) | `[126,46,92]` `0xff7e2e5c` | `[126,46,93]` `0xff7e2e5d` | B 92→93 | 3 / 5 / 0 |
| 4 | 1/1 | 28614 (134,89) | `[255,48,48]` `0xffff3030` | `[255,47,48]` `0xffff2f30` | G 48→47 | 3 / 5 / 0 |
| 5 | 1/1 | 39492 (132,123) | `[120,42,87]` `0xff782a57` | `[120,41,87]` `0xff782957` | G 42→41 | 3 / 5 / 0 |

共同事实：全部为 **cam-hinge（cam1）**、`geom_id=3`、`geom_type=5`（cylinder）、`mat_id=0`（`mat_texid` 行 `[-1,0,-1,…]` → slot1=0 = grid 纹理）；材质标量 `mat_shininess=0.6`、`mat_specular=0.4`、`mat_emission=0.05`（`mat_rgba` 见 dump）；**depth 两侧位型全等（ULP=0）、seg 两侧相同**；ray offset `[0,0,0]`、dir 位型与灯/相机位型均在 dump 中；差异均为**单通道 ±1 字节**，其中 3 个像素含 R=255 饱和通道（饱和标记见 dump）。

### 3.5 near-miss 摘要（dump `near_miss.by_frame`）

| world/cam | equal_byte_pixels | saturated_any_ratio | 0/255 分通道计数 |
| --- | --- | --- | --- |
| 0/0 | 76,800 | 0.689 | zero `[16995,16995,16995]`；full `[28171,15013,35920]` |
| 0/1 | 76,798 | 0.536 | zero `[21778,21778,21778]`；full `[17196,3926,6666]` |
| 1/0 | 76,800 | 0.689 | zero `[16995,16995,16995]`；full `[31754,17990,35920]` |
| 1/1 | 76,797 | 0.586 | zero `[22192,22192,22192]`；full `[20166,5793,9300]` |
| 2/0 | 76,800 | 0.689 | zero `[16995,16995,16995]`；full `[35920,23415,35920]` |
| 2/1 | 76,800 | 0.658 | zero `[23274,23274,23274]`；full `[17889,4871,15288]` |

- 最接近失配的 5 个非失配像素（每帧）：距离均为 **1 px**，**全部 geom 3 / type 5**；`nearest_geom_histogram` 与逐像素坐标/最近失配引用见 dump。
- 判读：失配不孤立在纹理边界处（neighbor 同 geom），且 cam-persp 全 0，说明现象集中在 cam-hinge 视角下 cylinder+grid 材质的着色字节量化边界。

### 3.6 与 G-L2 断言只看到 2 px 的关系

G-L2 canonical run 的判据 A 断言在最外层 `(world, cam)` 循环中**首个失败处中止**（world0/cam1 第 1 个失配像素即 panic），因此当时只报告 world0 cam1 的 `rgb_mismatched_pixels=2`；本轮 dump 在断言前采集全量数据，显示**实际全部失配为 5 px**（world0 cam1 2 px + world1 cam1 3 px，world2 cam1 与全部 cam-persp 为 0）。该差异只是断言中止顺序造成的可见范围差，不改变 G-L2「判据 A FAIL」判定。

## 4. 边界与非声明

- 边界：零 GPU 编辑；诊断轮为独立 round，不计 G-L2 轮次；不改 tests/src/scripts/fixtures；guards JSON、冻结 stash、12 个冻结 exe 证据与 G-L2 证据零改动；dump 不入库（`target/` 留 Arch 只读），证据只录 sha/bytes 与表格；诊断 exe 不进 guards。
- GPU：attempt 2 前后 4%→1–2%、3,919 MiB used、12,143 MiB free、26–27℃；无其它团队 CUDA 任务。
- 非声明：本卡不改变 G-L2 判定（判据 A FAIL 维持）；不宣称根因已定位、不宣称灯字段/渲染数值等价；不改任何判据/阈值/参考/相机/recipe；未重跑、未挑选结果。
