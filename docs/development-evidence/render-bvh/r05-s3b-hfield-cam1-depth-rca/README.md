# R05-S3b hfield cam1 depth 判据红只读定位（RCA）

任务：`2026-10-28-render-bvh-r05-s3b-hfield-cam1-depth-rca`（只读 RCA；不改判据/容差/参考/fixture，不开窗）。
全部数字来自**本地调试跑**，不是窗口验收结论。

## 1. 结论（四选一）

**② 射线方向构造 / 相机本地帧层**：`nmesh=0` 发射修复后暴露的 cam1 depth 差异，根因是
**固定相机的世界位姿（`cam_xmat`）派生在我们 host 侧与 pinned 上游采集时使用的位姿相差 1–2 ULP**。
不是 ① 深度合成、不是 ③ hfield/mesh 相交算术、不是 ④ 参考语义。

- 上游采集输入：`mujoco_warp` 的 `d.cam_xmat`（GPU 上由 `smooth.py` 相机 kernel 计算，内部调用
  `mujoco_warp/_src/math.py::quat_to_mat`，编译选项 `fast_math=false` + `fuse_fp=true` → NVRTC `--fmad=true`）。
- 本产品输入：S3b 目标用 `CameraPose::from_fields`（host Rust）派生同一 `cam_xmat`，公式为
  `src/math/mod.rs::quat_to_mat`（代数展开式，与 `mujoco_warp math.quat_to_mat` 源码逐项相同），
  但 **host Rust f32 不自动收缩（无 FMA）**，而 GPU 编译产物会收缩 → 同一公式、不同舍入。
- cam0：`cam-top` 四元数恰为 `(1,0,0,0)`，两种算术都得精确单位矩阵 → **零差异**；
  cam1：`cam-side` 由 `xyaxes` 派生的非平凡四元数 → 9 个矩阵元素中 **5 个差 1–2 ULP**（近零元素残差也不同），
  world 射线方向逐像素 1–2 ULP → 命中 `t` 差 1–7 ULP → depth 判据红。

## 2. 机器证据（逐层排除）

先复现基线（本地调试，非窗口；命令见 §8）：
`render_hfield_reference` 两用例：RGB/seg 位级全等，**cam0 depth mismatch=0 / max_ulp=0**，
**cam1 depth mismatch=4423（hfield-only）/4389（hfield-primitive）、max_ulp=7、beyond_ulp=20、首差异 cam1 pixel=16913 (x=113,y=70)**。

| 层 | 检查 | 结果 |
| --- | --- | --- |
| hfield 几何 | 我们 `build_hfield_mesh` 顶点/索引/`half` vs 上游 `build_hfield_bvh` 同函数产物 | **逐位全等**：顶点 bit diffs=0（13 顶点）、indices equal=true、half equal=true（12 三角形）；输入 16 个高度位型及 size bits 也全等 |
| 射线（相机本地） | 我们的 `RayBuilder` dir/offset vs 上游 `create_render_context` 的 `rc.ray`/`rc.ray_offset` | **逐位全等**：120000 条 dir bits 全等、offset bits 全等 |
| 输出布局 | `rgb_adr/depth_adr/seg_adr/cam_res/cam_id_map/total_rays` | 全等（[0,76800]、120000 像素、两相机分辨率一致） |
| 参考语义 | pinned 上游同 env 重渲 `depth` 与夹具 `depth.npz` 逐位比较 | **0 差异**（cam0 76800、cam1 43200，max_abs_diff=0）→ 参考就是 `dist * -ray_dir_local_cam.z` 口径，自洽可复现 |
| 深度合成源码 | 上游 `render.py:1285-1294` vs 我们 `render_megakernel.cu:201-204` | 同式 `hit.dist * -ray_dir_local_cam.z`，背景 0 一致 |
| 相交公式（Arch 候选） | 上游 `intersect.h` 的 `U/V/W==0` 双精度兜底分支命中率：host 端用同一 Woop 公式逐像素 f32 模拟（cam0 47024 / cam1 6150 命中像素） | **0 次命中（cam0 0/47024、cam1 0/6150）** → 排除该候选；`det` 与 depth ULP 差无相关（cam1 `corr(log|det|,ulp)=0.094`、`corr(1/|det|,ulp)=-0.065`，det 四分位不匹配率 0.71–0.74 平坦） |
| 三角形内位置 | cam1 mismatch 像素 vs matched 命中像素的 CPU f64 反投射 bary 分布 | 几乎相同（`min bary` p50 0.0967 vs 0.0964；边缘<1e-3 占比 0.70% vs 0.81%）→ 不是「不同三角形/tie-break」 |
| **相机世界位姿** | 我们 `CameraPose::from_fields` 的 `cam_xmat` vs 上游 `d.cam_xmat` | **cam0 逐位相同（9/9）**；**cam1 5/9 元素差 1–2（整数位差 [+1,−1,+2,−2040873,+1]），近零元素残差 −1.19e-8 vs −1.48e-8**；`cam_xpos` 两边全等 |
| 位姿差 → 射线差 → depth 差 | 用两边 `cam_xmat` 各自把同一批本地射线转 world（f32），逐像素比 | cam0：world dir 位差 **0/76800**；cam1：世界方向位差 **42449/43200 像素**，命中像素中 **6110/6150** 有方向差；cam1 depth 不匹配 4423 像素中 71.8% 落在「方向不同」集合（cam0 全等） |

补充：把内核编译换到 **NVRTC 12.9.1**（与 pinned 上游 Warp 同版本）重跑，cam1 结果不变（4423/4389、max_ulp=7、beyond=20；
`source_sha256` 不变、`ptx_sha256` 变化 `97467d83…`→`812b4fe7…`）→ 排除「我们的 NVRTC 版本/内核代码生成」这一层。

## 3. 为什么此前没有暴露

- `scene-a` 等既有深度参考实体**直接用参考 `camera_data` 位姿**（`tests/support/render_bvh/s7b1.rs::camera_data_poses`，
  测试里断言我们的 pose bits == `reference.camera_data[*].xmat_bits`；`tests/render_kernel_depth_seg.rs` 只对
  `CameraPose::from_fields` 的 quat 路径做 **≤1.0e-6 容差**检查，从未要求位等）。
- R05-S3b hfield 目标实体是**第一个**用 `CameraPose::from_fields` host 派生位姿 + **位级 depth 判据**的实体；
  该 1–2 ULP 位姿差在这个判据下暴露。
- 与本次零 mesh 发射修复无关：占位数组在零 mesh 场景不被解引用；RGB/seg/cam0 位级全等也印证发射路径正确。

## 4. 最小修复方案预核（**待 Arch/Chief 立项批准，本片未改任何代码**）

- **方案 A（推荐，host 侧对齐 GPU 收缩序）**：在 `src/render/kernel/cast.rs` 内新增一个 **warp 编译产物对齐**的
  `quat_to_matrix`（与 `mujoco_warp math.quat_to_mat` 同公式，按上游 GPU 实际 fma 收缩序用显式
  `f32::mul_add` 固定；不从 `src/math` 改全局函数），`CameraPose::from_fields` 改用它。
  - 依据：本片用 host 逐式模拟（`materials/bruteforce-summary.txt`）在 18 个矩阵元素中已能复现上游 **16 个位型**，
    仅 2 个对角项仍差 1 ULP → 缺口就是「收缩序未逐指令对齐」；精确序列需从仓库内 pinned Warp 缓存
    `target/render-bvh/r05-s3a-hfield-cache/1.15.0/wp_mujoco_warp._src.smooth_*/…sm89.ptx` 反读（只读）。
  - 影响面：`src/render/kernel/cast.rs` 当前 411 行（+~30–40 行，≤500）；不改 `src/math/mod.rs`（物理路径共用，避免大影响面）；
    不改 `.cu`、81 槽 ABI、`DEVICE_SOURCES`、`CollisionArgs`、判据/容差/参考/fixture。
  - 验证链：改后重跑 `render_hfield_reference`（期望 cam1 depth mismatch=0）→ 8 回归 exit 0 → fmt/clippy/clippy-cuda/unit/
    `--no-run`/内核门/`normify validate` → 重冻结 guards + stash + Chief 重签 → attempt=2 窗口。
- **方案 B（设备侧）**：相机位姿改由设备 kernel 计算（与上游同编译配方）；改动更大（新增/改 `.cu`、`DEVICE_SOURCES`），
  建议后续片评估，不作本片修复。
- **方案 C（测试侧，不可行/仅备选）**：S3a `capture.json` 的 `context` 块**不含** `camera_data` 位型，无法像 scene-a 那样直接取参考位姿；
  且用参考位姿会绕过产品派生路径，窗口就测不到该缺口（只可作「已知限制」记录，不做修复）。

## 5. 本片未做/局限

- 只读定位；**未改** `src/`、`tests/`、`fixtures/`、`scripts/`（临时探针已还原，`git status -- src tests` 为空）。
- 未从 PTX 逐指令反推完整收缩序（已定位到公式/收缩层，给出 16/18 的复现率）；方案 A 落地前需按 PTX 补齐。
- 未开窗、未重冻结、未改任何冻结物；`target/render-bvh/r05-s3b-window/attempt1/` 原样保留。
- 本片**零渲染数值验收结论**。

## 6. 复现与临时探针（全部在 `target/`，不入库）

- Rust 临时 dump 探针（已还原）：在 `tests/render_hfield_reference.rs` 临时加 `rca_tmp_dump_cam1_depth_inputs`，
  写出 `our-dump.json`（网格/射线/位姿/两位参考 depth/两次渲染 depth bits）后 `git checkout --` 还原。
- pinned python（warp 1.15.0 / mujoco_warp 3.15.0 / numpy 2.5.4）：
  `target/render-bvh/r05-s3b-window/rca-cam1-depth/upstream_dump.py` 导出上游 mesh/射线/`d.cam_xmat`，
  并重渲 depth 与夹具逐位比较（0 差异）。
- 对比/模拟脚本：`analyze.py`、`stats2.py`、`campose_link.py`、`bruteforce_mjw_quatmat.py`（结果在
  `analysis-summary.json`、`step1-det-summary.json`、`campose-link.json`、`upstream-camworld.json`）。
- NVRTC 12.9.1 A/B：`target/toolchains/cuda-12.9.1-nvrtc/bin` 前置 PATH 跑 `render_hfield_reference --ignored`，
  结果与 12.8.1 相同（日志 `nvrtc129-run.log`）。

## 7. 40 像素「dir 位级相等却 depth 不等」闭环（内核插桩，临时代码已还原）

首次复核用 host f32 模拟 world dir 时，有 40 个命中像素被判「dir 位等」（其中 34 个 depth 不等）。用**内核插桩**
取实际值后（2 个临时 pass：cam1 的 `ray_dir_world`/`ray_origin_world`/`hit.face`/`hit.dist` 位型；跑完 `git checkout --`
还原，`src/` 零 diff）结论是：

- 我们内核实际 world dir vs 上游 warp kernel 实际值：**43022/43200** 像素不同（命中像素 **6146/6150**）；
  `ray_origin_world`（= `cam_xpos`，offset=0）**43200/43200 位等**。
- 首次复核用的 host 模拟本身与内核实际值在 **32111/43200** 像素上不一致（对上游 kernel 32055/43200）→
  **host 模拟不能判定 dir 相等，属对照口径 artifact**。
- 那 34 个像素：**实际 dir 34/34 都不同**、origin 34/34 位等；`our_t=depth/(-dirz)` vs `ref_t` 位差 1–3 ULP。
- 一句话：**「dir 位级相等」是 host 模拟 artifact；用内核实际值后，cam1 命中像素 99.9% 的 world dir 都因
  相机位姿 1–2 ULP 差而不同，所有 depth 不匹配像素（含这 34 个）都由同一位姿根因解释，不存在第二个现象。**
- 机器数据：`materials/kernel-vs-upstream-dir.json`（含 34 行逐像素表）、`materials/forty-pixels.json`、
  `materials/forty-tie-analysis.json`（tie-break 假设排除：这些像素 CPU 反投射只有一个候选三角形，无共享边并列）。

## 8. 方案 A 影响面清单（立项用；本片未改代码）

1. **文件/行数**：只改 `src/render/kernel/cast.rs`（新增 warp 收缩序对齐的 `quat_to_matrix` + `CameraPose::from_fields` 改调用）：
   当前 411 行，预估 **+30~40 行（≤451）**；`src/math/mod.rs::quat_to_mat` **不动**（物理路径共用）。
2. **变 sha 的目标**：所有链接 `mjwarp-rs` lib 的测试 exe 都会重编；其中窗口冻结的 **9 个 render 实体的 exe sha 必变**
   （需重冻结 guards + Chief 重签）。无「非 render 产品目标」引用该 host 路径（`CameraPose::from_fields` 调用方只在
   render 内核层与 tests/hfield target；`src/` 内无其它调用者）。
3. **既有参考影响**：scene-a / scene-shadow 的深度/位等实体**直传参考 `camera_data` 位姿**
   （`tests/support/render_bvh/s7b1.rs::camera_data_poses` 从 `reference.camera_data[*].xmat_bits` 解码；
   `tests/render_kernel_depth_seg.rs` 断言 data 位姿位等）→ **渲染输入零变化、参考不受影响**。
   `camera_data_poses` 位等断言不变；`check_camera_data`/e2e 的 quat 辅助口径断言是容差型
   （`fixed_max <= 1e-6`，e2e `max_ulp <= 1`）→ 修复后差异只会更小，断言仍成立（`tests/render_e2e.rs:161-175`）。
4. **必须重跑**：
   - 零影响论证：R08/R09 的 GPU 深度/像素实体用参考 data 位姿 → 期望输出位级不变（参考 sha 不变）。
   - 必须重跑清单：9 个窗口实体（1 新 + 8 回归）、`render_kernel_depth_seg`/`render_e2e`/`render_r08_cameras` 的 host 用例
     （含 quat 辅助口径断言）、`cargo test --locked` 全量 + fmt/clippy/clippy-cuda/`--no-run`/内核门/`normify validate`。
5. **红线**：`.cu`、`DEVICE_SOURCES=28`、81 槽、`CollisionArgs`、判据/容差/参考/fixture、`hfield_scene.rs` 全部零改动
   （host Rust 单点改动；内核编译门 source/ptx sha 期望不变）。
6. **语义定性**：这是**补齐上游 GPU FMA 收缩语义的 host 派生位姿舍入口径**，不是放宽判据、不是绕过产品路径
   （参考 data 位姿直传路径保持不动；本片也不采用让测试直接喂上游 `cam_xmat` 的绕行方案）。

## 9. 追加：Arch 版本假设验证 + 旧场景重放（2026-10-29）

- **版本测试（否定假设）**：pinned `mujoco 3.15.0` 加载同一 `scene-hfield/scene.xml`，`cam_quat` **8/8**、`cam_mat0` **18/18** 个 f64 位型与我们的 3.12 dump **全等**（cam0/cam1 均无差异）→ 「xyaxes→quat 版本级舍入差」不成立；改夹具/重采参考不能修复。
- **旧场景重放（反证）**：scene-a cam-persp（mode 0，`quat="0.9 0.1 -0.2 0.3"`，已验证）用 host 公式重放 vs 已验证参考 `camera_data_check[0].xmat_bits`：**9 元素中 2 个差 1 ULP（index 0、7）** → 旧位等实体从未让产品公式进入权威比较（直传参考 `camera_data` 位姿；quat 路径只做容差检查，e2e ≤1 ULP / depth_seg ≤1e-6）；「R08/R09 位等已证明公式没问题」不成立。
- **候选 fused 公式**：scene-a 仍差 1/9、hfield cam1 仍差 2/9 → 精确 GPU fma 序必须从 pinned warp 缓存 PTX（`target/render-bvh/r05-s3a-hfield-cache/1.15.0/wp_mujoco_warp._src.smooth_*`，只读）反读，不能靠公式模拟推断。
- 机器数据：`materials/arch-version-and-replay.json`、`materials/arch-version-replay.txt`；脚本在 `target/`（不入库，命令见 txt）。
