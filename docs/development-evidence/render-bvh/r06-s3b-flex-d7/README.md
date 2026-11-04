# R06-S3b③ D7：attempt4 render 发射 CUDA 700 的根因定位与修复

任务：`2026-11-04-render-r06-s3b-flex-d7-transfer-wait`（owner Backend）。
基线：诊断窗口 HEAD `39d787c2`；修复落地前 HEAD `71f07b35`。
环境：Windows/MSVC、RTX 4070 Ti SUPER（驱动 596.36）、NVRTC CUDA 12.8.1（`target/toolchains/cuda-12.8.1`）。

## 1. 结论摘要

attempt4 实体在 `assembly.render_with(...)` 处报 `CUDA失败：transfer-wait；错误码700`
（`CUDA_LAUNCH_BLOCKING=1` 下改为 `render-launch；错误码700`），不是数值判据红，而是**设备侧非法访问**。
根因不在 harness 输入，也不在相机位姿：是 **R06-S3c 引入的产品回归**——1D-only 路径
（`flex_mesh == None`）下 flex 通道上传实参退化成 1 元素占位，而场景 BVH 仍含 flex 叶；
首个真实 flex 命中时设备侧读 `flex.geom_flexid[flex_local]` 越界。修复两处后，同命令两次短跑
**不再出现 700**，且 `mesh=None` 路径渲染出 `(flexid,9)` seg 像素。

## 2. 根因（代码级）

- 触发条件：相机位姿修复（D5）后 cam-flex 首次真正命中 flex 胶囊；attempt3 相机朝向错误、从未命中，故未暴露。
- 回归提交：`181eaa7a`（R06-S3c 2D/3D flex 设备接线）把上传实参从 `flex` 改为
  `flex: adjusted_flex.as_ref()`；`adjusted_flex = flex_bvh_ids.as_ref().zip(flex)`，
  而 `flex_bvh_ids` 仅在 `mesh_assembly` 为 `Some` 时才有值 ⇒ **1D-only 路径 `adjusted_flex = None`**。
- 设备侧假设（`src/render/kernel/cuda/render_megakernel.cu` L164-176 注释）：
  「`bvh_nflexgeom == 0` ⇒ 场景叶不会有负 `leaf_geom`」。该假设在 1D-only 路径被打破：
  场景 BVH 已由真实 flex 构造了 4 个叶（编码 `-1 - flex_local`），但 9 个 flex 设备槽是 1 元素零占位。
- 越界点：`src/render/kernel/cuda/render_common.cu` L398-403（同 `render_flex.cu::flex_leaf_hit`）
  `flex.geom_flexid[flex_local]` / `flex.rgba[4 * flex_id + 3]`。
- 本机无 compute-sanitizer（CUDA 工具链仅 nvcc/NVRTC/ptxas）；定位靠阻塞复现 + 分相机 mask + 反证实验。

## 3. 诊断实验（授权微窗口，1 次；日志见 `logs/`）

窗口：会话 backend，任务同上，取锁 12:38:19（pid 79128），HEAD `39d787c2`，树 clean；
GPU 前 1764 MiB / 14298 free / 2% / 28°C；全程零重编产品、零覆盖冻结 exe；诊断补丁仅测试侧、跑完删除。

1. **阻塞复现**（`CUDA_LAUNCH_BLOCKING=1`，冻结 exe `render_flex_validate-30bd8d889931ab2f.exe`，sha `d9686515…`）：
   `running 1 test` → panic `tests/render_flex_validate.rs:327` = `assembly.render_with(...)`；
   `Driver("render-launch: 传输后端错误：CUDA失败：render-launch；错误码700")`，1.54s，EXIT=101
   （`logs/blocking-baseline.log`）。→ 错误在 kernel 执行本身，不是上传/分配/清理阶段。
2. **分相机 mask 三跑**（测试侧 `ContextParams::with_active(CameraActive::Mask)`，`D7_CAM_MASK=0|1|2`）：
   cam0 EXIT=0（seg `[[0,5]]`）、cam1 EXIT=0（seg `[[0,5],[1,5],[2,5],[3,5]]`）、
   cam2 EXIT=101 同一 700 → **命中路径 = flex（cam-flex）**，与网格/纹理无关
   （`logs/mask-cam0.log`、`mask-cam1.log`、`mask-cam2.log`）。
3. **排除项（全部仍崩）**：`D7_SHADOWS=0`、`D7_SMOOTH=0`、两者同关、`D7_PRECOMPUTED=0`（动态射线）、
   `D7_FLAGS=seg|depth|rgb` 单通道、`D7_ALPHA0=0,1`（flex 全透明）→ 崩在主射线 cast_ray 的 flex 叶分派，
   不在着色/阴影/贴图/AA/ray_capsule/颜色（`logs/cfg-*.log`、`logs/flags-*.log`、`logs/alpha0-*.log`）。
4. **因果确认**：`D7_FLEX_FAR=1`（flex 顶点 +1e6，不可命中）→ EXIT=0 正常出图、无 flex 像素；
   `D7_SWAP_POSE=1`（cam0 用 cam2 的 data 位姿）+ mask=0 → EXIT=101
   → 崩需要 flex 叶被真实命中，与相机序号/分辨率无关（`logs/mask2-flexfar.log`、`logs/mask0-swappose.log`）。
5. **Witness 反证**：测试侧给一个 1D `FlexMeshChannel`（mesh_count=0）使 `adjusted_flex` 变 `Some`
   → EXIT=0，`seg_pairs=[[0,5],[0,9],[1,9]]`，正常出图（`logs/mask2-meshchannel-witness.log`）
   → 只要上传真实 flex 通道 700 即消失。

## 4. 修复（本卡，`src/render/kernel/megakernel.rs` 两处）

1. 上传实参兜底（原 L403，现 L410）：`flex: adjusted_flex.as_ref().or(flex),`（单行，无新增行）。
2. 不变量守卫（L323-328）：场景 BVH 含 flex 叶（`flex_leaves.is_some()`，等价于含负 `leaf_geom`）时，
   最终上传实参（`adjusted_flex.as_ref().or(flex)`）不得缺省，否则 host 侧返回
   `LauncherError::InvalidArray("flex_channel_placeholder")`，不再把问题留给设备。
3. 回归断言（`tests/render_flex_validate.rs:494`，现有 ignored GPU 用例内、`mesh=None` 路径）：
   `assert!(first.2.iter().any(|pair| pair[1] == 9), "D7 回归：mesh=None（1D-only）必须出现 (flexid,9) seg 像素")`。

行数：`megakernel.rs` 449 → 456（该文件内注释自述「450 死线」，实测已只剩 1 行余量；
Chief 提到的 379 行是旧计数）。**纯函数 + unit 单测放不下**，按 Chief 预批 fallback 采用内联守卫，
并记 backlog：「把上传实参选择与不变量抽成纯函数，交 unit 门禁覆盖」。其余 `src/` 零 diff。

## 5. 修复后授权短跑（2 次，诊断性复验，**不是判据验收**）

命令（唯一入口，两次相同）：`cargo test --locked --features cuda-probe --test render_flex_validate -- --ignored --nocapture`
（run2 追加 `--test-threads=1`）。日志：`logs/run1-fixed.log`、`logs/run2-regression.log`。

- run1：GPU 前 1756 MiB / 14306 free / 2% / 28°C → 后 1756 / 14306 / 1% / 29°C；
  `running 1 test`，**无 700**，1.74s，EXIT=101。
- run2：GPU 前 1756 / 14306 / 5% / 28°C → 后 1753 / 14309 / 1% / 29°C；
  同一失败点、同一断言，1.51s，EXIT=101（可重复、非偶发）。

失败点已变成**判据面**（不再是设备错误）：`tests/render_flex_validate.rs:347`
`cam2 depth 位级/超 4 ULP`，`left=(321, 297)` `right=(0, 0)`；
在此之前三台相机 RGB（≤1 LSB）、cam0/cam1 depth 与 seg 均通过 → 说明修复后 flex 路径已真实出像。
**该 depth 差异是本次修复后才第一次可观测到的新缺口，不属于 D7 范围，也不代表 attempt4/attempt5 转绿**；
attempt5 前需另卡分析（下一节）。

## 6. 证据解读口径（必须按此读）

- 回归断言位于 **`#[ignore]` GPU 用例**内，四项门禁（fmt/clippy/clippy-cuda/unit）**不会执行它**；
  `unit 1316/0` 不表示本回归已被覆盖。「修复有效」只由本页 §5 的两次授权短跑日志支撑
  （`running 1 test`、无 700、`(flexid,9)` 断言未触发）。
- 守卫位于 GPU 发射路径、普通 `cargo test` 不触发，属**结构性未覆盖**；
  纯函数化 + unit 覆盖记 backlog（见 §4）。
- 本卡是「诊断 + 修复」，不是数值判据验收：cam2 depth 判据仍红，`seg` 在该用例中因 depth 先红而未评估。

## 7. 边界与未动项

- 未动 `scripts/validate/render-bvh/guards-r06-s3b-flex.json`、`target/.../frozen-exes/` 冻结 stash、
  attempt3/4 证据、判据/容差/参考文件。
- 注意：修复重建了 live 测试 exe（`render_flex_validate-30bd8d…exe`，107,660,771 B，12:52:36），
  与 attempt4 冻结时的 `d9686515…` 不再相同；冻结 stash 未变。**attempt5 必须先重新真冻结 16/16**。
- 诊断补丁 `tests/d7_camera_mask_probe.rs` 跑完即删，未入库（本卡 files 不含它）。

## 8. 下一步（建议）

1. 新卡：flex 命中路径的 **depth 数值缺口**（cam2 `(321,297)` / >4 ULP 297 像素）——`depth = hit.dist * -ray_dir_local_cam.z`
   与参考的 flex 命中距离/深度语义需逐式核对；rgb 全过说明颜色路径基本对。
2. attempt5 前提重排：本卡 verified + QA pass + 上述 depth 缺口处理（或裁定 attempt5 仍按判据跑并接受红）+ 重新真冻结 + Chief token。
3. backlog：`megakernel.rs` 上传实参选择/不变量纯函数化，交 unit 门禁覆盖。