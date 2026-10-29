# R08-RGB：逐相机 RGB 通道掩码（host 三通道逐相机填充 + latent 负偏移修复）

任务：`2026-10-29-render-r08-rgb-hetero`（Arch 2026-10-29 登记；Chief 批准 S1 离线实现）。
性质：只改 `megakernel.rs`/`megakernel_arrays.rs` 的 host 填充与测试；**不改 `.cu`、不改 81 槽 ABI、不改判据/参考/夹具**；本片不开 GPU 窗口、不做数值验收结论（设备侧验证归 S3 窗口 `2026-10-29-render-r08-rgb-hetero-validate`）。

## 1. 改动

- `src/render/kernel/megakernel_arrays.rs`：新增纯函数 `channel_flags(&[bool]) -> Vec<i32>`（+ 2 条单测：逐项映射、全 false/空）。
- `src/render/kernel/megakernel.rs`：`render_rgb/render_depth/render_seg` 由「全局布尔填满 nrender」改为 `channel_flags(&resolved.plan.render_*)`（逐激活相机）；删除不再使用的 `ImageOutput` import。
- 原因：81 槽签名的 51/52/53 槽（0-based）本就是长度 `nrender` 的 `Array(Bool)`；内核已按 `render_*.data[camid]` 读取并在 `do_rgb/do_depth/do_seg` 守卫内写入。host 用全局布尔时，关闭通道的相机 `*_adr=-1` 仍会进入写入分支（负偏移写，latent 越界）；逐相机填充后该类相机 `do_*=false`，不再写。
- 测试：`tests/support/render_bvh/r08_gpu.rs` 新增 `heterogeneous_rgb_channels`（GPU，`#[ignore]`）与既有 `heterogeneous_depth_seg_channels` 的 MegakernelRenderer 交叉掩码回归；`tests/render_r08_cameras.rs` 新增 `gpu_heterogeneous_rgb_channels` 入口。

## 2. 行数与范围（全部在卡面上界内）

| 文件 | before LF | after LF | 上界 |
| --- | --- | --- | --- |
| `src/render/kernel/megakernel.rs` | 410 | 409 | 420 |
| `src/render/kernel/megakernel_arrays.rs` | 332 | 357 | 365 |
| `tests/render_r08_cameras.rs` | 320 | 326 | 380 |
| `tests/support/render_bvh/r08_gpu.rs` | 321 | 403 | 420 |
| `tests/support/render_bvh/r08.rs` | 366 | 366（未改） | 380 |

零改动（逐文件 diff 为空）：`src/render/kernel/cuda/**`、`megakernel_uploads.rs`、`camera_output.rs`、`context.rs`、`context/params.rs`、`camera.rs`。

## 3. ABI 与内核身份（零设备源改动）

- `MEGAKERNEL_SLOTS.len() == 81` 不变；51/52/53 仍为 `Array(Bool)`（`args.rs:463-465`），未新增/重排槽。
- 6 个内核 `.cu` 拼接后 `source_sha256 = e68e3412c1e1809e71100d9e842d5b0f5b886ffdec88d8401bb01c76e812904c`（与基线一致，`git diff` 对 `src/render/kernel/cuda/**` 为空）。
- NVRTC 编译门（仅编译，不使用 GPU）：`source_sha256=e68e3412…`、`ptx_sha256=0b9474928ab8a7add27e5da7ac0869ee955a75405cd976cd04839f2312e88f3e`、`ptx_bytes=268275`，与基线三值一致；`aa_resolve` 亦不变（`0bf8af5a…`/27536 B）。日志：`logs/kernel-gate.txt`。

## 4. 测试覆盖（S1 只编译与单测；GPU 用例本片不执行）

- 单测（已跑）：`channel_flags_maps_each_camera_independent`、`channel_flags_handles_all_false_and_empty`；`cargo test --locked --lib render::kernel::megakernel_arrays` 6 passed / 0 failed。
- GPU `#[ignore]` 用例（窗口内执行）：
  - `heterogeneous_rgb_channels`：`[true,false]`/`[false,true]`/全 off 三态；启用相机 packed RGB 对 same-recipe S7b2 参考逐位相同、未启用相机 `rgb_packed` 读取拒绝；depth/seg 对 S7b1 参考位等/全等；每态两跑输出 SHA-256 相等。
  - `heterogeneous_depth_seg_channels`（既有，追加）：同一交叉掩码（cam0 仅 depth、cam1 仅 seg）再经 `MegakernelRenderer` 发射，逐相机对参考且关闭通道读取拒绝——覆盖 latent 负偏移写回归（修复前会写 `adr=-1`）。
- 窗口重放计划（S3）：9 实体（本 target 为 new + 8 回归）；无新参考；判据沿用现有位级零放宽。

## 5. 门禁

- `normify gate`：fmt / clippy / clippy-cuda / unit 全过；unit **1106 passed / 0 failed**（基线 1104，+2 为本次新增单测）。
- `cargo test --locked --no-run` 全目标编译通过（日志 `logs/no-run.txt`）。
- 内核编译门：见 §3。

## 6. 非声明与遗留

- 本片不做数值验收结论：GPU 用例代码已编译（`--no-run`），执行归 S3 窗口；不开窗、不预批例外。
- 不改冻结 ABI、不改 `.cu`、不改判据/容差/参考/夹具。
- 缺口卡 `2026-10-28-render-per-camera-rgb-gap` note 已指向本卡；S3 `2026-10-29-render-r08-rgb-hetero-validate` 已预登记、未 start；QA 复核卡 `2026-10-29-render-r08-rgb-hetero-check` 已预登记。

## 7. 证据文件

| 文件 | 内容 |
| --- | --- |
| `logs/line-counts.txt` | 5 文件 before/after LF |
| `logs/source-sha.txt` | 内核拼接 source sha 与 `.cu` diff 为空 |
| `logs/kernel-gate.txt` | NVRTC 编译门 source/ptx/bytes |
| `logs/no-run.txt` | `cargo test --no-run` 结果 |
| `logs/unit.txt` | 新单测与 unit 汇总 |
| `logs/scope-diff.txt` | 本片改动文件清单（只 host 4 文件 + 证据） |
