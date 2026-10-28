# R10-S2 多 world 端到端验收（nworld>1 相机位姿接线 + world 数组形状 + 同配方参考）

任务：`2026-10-28-render-bvh-r10-s2-multi-world`（P8 渲染与 BVH；1 验证窗，无新捕获）
实施 HEAD：`40e9238f219daf980c48af124bfe63e585aa0f42`（in_progress 时的实施提交；本证据绑定该 HEAD）
runner：`target/r10-s2-window/run-window-attempt1.ps1` / 14 968 B / sha256 `4d636c39731145932a2057477c800e218090cb884542274e3a52e0fe28ce2438`（窗口前签名，窗口内零编辑）
guards：`target/r10-s2-window/guards.json` / 15 162 B / sha256 `0c849e67c857265e088e6c4a4ccf6de88053f3ba5776f6044a88dfe982cc86e7`
profile：`cuda-probe`；CUDA bin：`target/toolchains/cuda-12.8.1/bin`

## 1. 轮次记账（两轮分写；另有一次 precheck abort 非有效轮）

### attempt1（有效轮 1：8 实体跑完，新实体未过）
- 时间：`window_start 2026-10-28T03:46:10.2815332+08:00` → `window_end 03:46:56.7924446`；`body_wall_s=44.1`
- 8 实体串行：7 回归 exit 0；新实体 `render_r10_multi_world` **exit=101**（2 条用例失败）
  - `gpu_nworld_two_depth_seg_per_world_cameras` → `tests/render_r10_multi_world.rs:178` 断言失败，日志原文 `left: 19074` / `right: 0`（19074 = ULP≠0 的像素数，非超容差像素数）
  - `gpu_nworld_two_rgb_per_world_cameras` → `LauncherError::Capacity { slot: 0, required: 56, actual: 28 }`
- 根因（见 `attempt1/logs/render_r10_multi_world.log` 与 dev 诊断）：
  1. RGB：`megakernel` 把单 world 模型数组（`geom_type` 7×i32 = 28 B）按 `[nworld=2, ngeom=7]` 声明 → 需 56 B。**实现缺口，已修**（shape 去 world 维 + `world_arrays.rs`）
  2. depth：用例误用 fast-math 采集的 `s7b1::REFERENCE` 作对照，而 depth/seg 内核与**同配方**（`use_fast_math=False` 采集）参考位级一致；`mismatched` 是 ULP≠0 的像素数而非超容差数。**测试侧参考配方错配，已修正到同配方；断言 `mismatched==0`（位级）+ `max_ulp<=4` 兜底与容差零改动**
- 窗口事实：precheck `4201/11861/36·36·38`；baseline `4201,11861,38 / 4201,11861,38 / 4201,11861,41`；`baseline_free=11861`、**floor=8789**；采样 156、interval median 0.2818 s、peak 4492（seq 92）、free_min 11570、util_max 35、streak(≥80)=0；三口径 delta 291/270/295、sustained 291；`floor_breached=False`、`residual=0`、零 Rust 编译 True（deps 1821→1821）；守卫 46/46、窗后 8/8；runner 退出码 10（新实体失败，属预期记账路径）
- 本窗口未取得 RGB/depth 通过数值（如实记「未通过」）

### attempt2-exit9（precheck abort，**非有效轮**，无判据数据）
- 时间：`2026-10-28T12:05:31` 弹出（自采已满足时点）；runner 自身 precheck 采到 `open_util=40>=40`（2 点）→ `PRECHECK_ABORT` / exit 9
- 产物：`attempt2-exit9/facts.txt`（61 B，`attempt=2` + `precheck_abort open_util=40>=40 | open_util=40>=40`）与 `guard.log`（51 B），**原样留档、不删不改不补造**；无 baseline / 无实体 / 无采样
- 该次不计入有效轮

### attempt2-run（有效轮 2，最后一轮：8 实体全部通过）
- 时间：`window_start 2026-10-28T12:21:22.1237577+08:00` → `window_end 12:22:06.2418884`；entity `12:21:23.0165 → 12:22:04.7323`；`body_wall_s=41.7`
- 弹出前置：自采连续 **72/72 样本（5 s 间隔，约 6 分钟）** 满足 `util<40 且 free≥10240`（`target/r10-s2-window/gpu-monitor-3.csv`），precheck `open=3286/12776/18/31;3286/12776/9/31;3286/12776/23/31`
- **8 实体全部 exit 0**（`attempt2-run/entity-results.csv`）：
  - `render_r10_multi_world`（new）exit 0 / wall 2344 ms
  - `render_camera_rays` 0/1109、`render_context_lifecycle` 0/7910、`render_e2e` 0/7722、`render_kernel_depth_seg` 0/4219、`render_kernel_flags` 0/8234、`render_kernel_rgb` 0/2982、`render_r08_cameras` 0/7167
  - 新实体日志：`gpu_nworld_two_depth_seg_per_world_cameras ... ok`、`gpu_nworld_two_rgb_per_world_cameras ... ok`（`test result: ok. 2 passed; 0 failed`）
- **world0 数值**（同配方参考，断言口径未放宽）：depth `mismatched==0`（位级）+ `max_ulp<=4` 兜底；seg 全等；两相机逐相机断言通过。dev 诊断同配方数据：camera0/camera1 `mismatched=0 / beyond_ulp=0 / max_ulp=0`（76800/43200 像素）
- **world1 等价/隔离**（断言全过）：`world1 vs nworld=1 单独渲染同 state` 逐位 120000/120000；`world0 vs world1` 位姿差（per-world 相机生效）；隔离用例（world1 几何 +1000）world0 区段逐位不变、world1 全 miss/背景；两次运行 depth/seg SHA 相同
- 窗口事实：baseline `3286,12776,23 / 3282,12780,23 / 3286,12776,9`；`baseline_free=12780`、**floor=9708**；采样 148、interval median 0.2814 s、peak 3556（seq 87）、used_min 3283、free_min 12506、util_max 9、streak(≥80)=0；三口径 delta 270/90/273、sustained 270；`floor_breached=False`、`increment_exceeded=False`、`guard_aborted=False`、`residual=0`；零 Rust 编译 True（deps 1823→1823，max_write 04:23:44）；守卫 46/46、窗后 8 exe 复查 8/8；compute-apps post raw 29 / filtered 0

## 2. 修复轮（未进窗口，实施提交 40e9238f）
- `src/render/kernel/megakernel.rs`（491 行）：静态/data 数组 shape 去 nworld 维（30 处）；相机位姿 `cam_xpos/cam_xmat` 保留 `[nworld, ncam]`；相机静态参数单份化
- 新 `src/render/kernel/world_arrays.rs`（74 行，3 单测）：`shared_shape`/`shared_matrix_shape`/`camera_pose_shape`/`light_type_shape`（`light_type` 保持 `[1, nlight]` 供内核 `shape.dims[1]`）
- `tests/render_r10_multi_world.rs`（298 行）：depth 参考由 `s7b1::REFERENCE`（fast-math）改为 `recipe_depth_seg()`（同配方）；断言/容差零改动
- dev 自测（**非验证窗、仅诊断**）：`render_r10_multi_world` 单 target 2 条 ignored → 2 passed / exit 0 / wall 3.1 s

## 3. 冻结实体（8/8 相对 attempt1 签名表全变）
原因：`megakernel.rs` 属公共库代码，改动后 8 个 render test 二进制全部重建。

| 实体 | bytes | sha256 |
| --- | --- | --- |
| render_r10_multi_world | 94 030 207 | `891ee06b0b18401fb5a285ce0b10eb8781a685c321a560f7fc910cfb43eadb42` |
| render_camera_rays | 89 577 444 | `9db09b9463cae6216e6d2ea08edd4a0ff34c2e5c00a639f8e01bcd4e3f7fc8c2` |
| render_kernel_rgb | 93 911 155 | `615b4492727bb42649f610d6fa6f0846967c4ab95d98a7076735c2892c8c5235` |
| render_kernel_depth_seg | 90 485 465 | `9b74d0325fd688cf5b0bf09df696d06db882b37e1c3cca8e82014d22e387766d` |
| render_kernel_flags | 95 863 071 | `d577264fb19cf5a270cbdfa37a819ceee41945580fc1fd3b03d71301e0774d37` |
| render_e2e | 94 457 096 | `0394c5bcc59ee3337e7f4a79d539cbc9d99ba6d4b71498b135fa98f387bfc2ce` |
| render_context_lifecycle | 86 046 774 | `63e865312842a55b04ea2b356bb0522fd92b952ccc119568748702b886d7142a` |
| render_r08_cameras | 94 265 178 | `b7355ad31ec22c6cc8820f0f97863897e43ad1afc992c235909f047fdcb9f369` |

## 4. 口径与限制（必须随证据保留）
- **方案 A 边界**：world0 对**同配方** nworld=1 冻结参考做位级对照（depth `mismatched==0` + `max_ulp<=4` 兜底、seg 全等）；world1 = 同引擎等价（`nworld=1` 单独渲染同 state 逐位）+ 隔离（几何 +1000 → world0 逐位不变、world1 全 miss/背景）+ 两次 SHA 相同
- **上游 nworld>1 数值未覆盖**：无上游 nworld=2 数值参考，不对上游 nworld>1 做任何数值结论；不写「上游等价」
- **多 world 着色共用单份几何/灯位姿**：本片只覆盖「两 world 几何相同（world_dx=0）」情形；逐 world 几何/灯位姿数据缺口 → backlog `2026-10-28-render-per-world-geom-light-gap`（backlog 不 start）
- **`render.cu` 499 行贴线事实**（500 棘轮边界）；`camera_output.rs`(499)、`model/render.rs`(500)、kernel 三文件、设备 `.cu`、`DEVICE_SOURCES`(28)、入口(6)、`CollisionArgs`(504 B) 未改
- 判据/参考/夹具/容差零改动；未删或降级任何失败用例
- 不写「窗口通过/已覆盖/精度达标/上游等价/双平台等价」

## 5. 数据来源
- 窗口原始产物登记在 `target/r10-s2-window/`（attempt1 / attempt2 / attempt2-run），并已复制入本目录对应子目录（见 manifest.txt 的 sha/bytes）
- 最终门禁（`normify gate`）绑最终树，见 `.normify/runs/2026-10-28-render-bvh-r10-s2-multi-world/`

## 6. 守卫与检查产物
- **窗口内守卫**：`sha_guards = 46/46`（8 exe + 38 只读输入，mismatches=0）；窗后 8 exe 复查 **8/8**（attempt1 与 attempt2-run 均如此）
- **A 修复后 guards-only 自检**：`target/r10-s2-window/guards-check-r2/` = **46/46、0 mismatch**（用新 `guards.json 0c849e67…`）
- **旧目录仅历史**：`target/r10-s2-window/guards-check/`（attempt1 旧 sha `6a73c51a…` 记录）与中间产物 `guards-check-a2/`、`guards-check-a3/`、`guards-check-final/` **保留未删、仅作历史**，不作为本次自检依据
- **dev 自测**：`render_r10_multi_world` 单 target 2 条 ignored → 2 passed / exit 0 / wall 3.1 s，**标「非验证窗、仅诊断」**，不作窗口结论
- **attempt2-exit9 明确无 baseline / 无实体 / 无采样**（runner precheck 阶段 abort）
