# UNC-3 分支收口 S1：逐分支可达性台账 + case/fixture 设计（无 GPU）

**任务**：`2026-10-27-unc3-branch-closure-s1-reachability`（owner Arch）；QA 复核 `3aec54a1`（pass / 0 阻断 / 观察 11）；
本版为观察收口勘误版（任务 `2026-10-27-unc3-branch-closure-s1-observation-closure`）。**只读分析与设计**：
不改代码/判据/容差/ABI，不改既有 fixtures，未跑 GPU、未捕获。

**pinned 版本**：`71da24d956378a87a703b6e1442b13aec0c4ac29`（`docs/evidence/d04-complex-physics/authoring-runs/run-5/installed-source/_src/`）。
**设备实现优先级**：`src/physics/collision/cuda/primitive_plane_shapes.cu` → QA 核过的行号为准（见每行）。

## 1. 可达性台账（逐条：可验证 / 不可达 / 仅限制存在）

| # | 分支 | pinned 依据 | 设备实现 | 判定 | 合法 MJCF 触发构造（已按 QA 复算修正） |
| --- | --- | --- | --- | --- | --- |
| U1 | plane_cylinder 退化（`len_sqr<1e-12` → x 轴回退） | `collision_primitive_core.py` L496-517 | `primitive_plane_shapes.cu` L117-139（QA 核：else 回退 L137-139） | **可验证** | plane 法线 +z；cylinder `quat=identity`（MuJoCo default axis=+z）、半高 h、中心 z=h−0.01 → `vec=(0,0,0)`、`len_sqr=0.0` 精确、回退 `(r,0,0)`；QA 逐式 f32 重放：dist1=−0.009999997913837433、dist2=0.19、无 NaN |
| U2 | plane_ellipsoid「非上游退化保护」 | `collision_primitive_core.py` L312-339 **无**该保护（L334 直接 normalize） | `primitive_plane_shapes.cu` L93-102（QA 核：guard 在 L100-102） | **不可达（非上游防御分支）** | 常规合法 MJCF 不可达：ellipsoid `size` 各分量 >0 且有限，`normalize` 输入恒非零。**f32 溢出注记**：若 `size` 分量 ≥ √FLT_MAX=1.8446744e19，平方溢出 → 设备 `length=inf` 会触发 guard（上游 normalize 得 NaN），而 `abi.rs` 不校验 size 值域；MJCF 编译器是否拒绝该极端输入**未实测**。证据方式 = 静态 diff（产品树无 plane_ellipsoid CPU 实现，若加测试须标注「镜像」，非设备代码） |
| U3 | plane_box 8 角点覆盖 | `collision_primitive_core.py` L366-390；wrapper L800-834 逐条 `write_contact` cutoff（QA 核行号） | `primitive_plane_shapes.cu` plane_box | **可验证** | 现有 B1 设备 case 只出现角点 **{0,1,3}**（QA 用真实 qpos/尺寸复算，dist 三值与 reference 一致）。**5 pose 覆盖 8 角**（QA 验算并集 8/8）：①面朝下 {0,1,2,3}；②**Rx(180) 面朝下 {4,5,6,7}**；③绕 x 45° {0,1}；④绕 y 45° {1,3}；⑤单角朝下 {7}。原 4-pose 方案并集只有 5/8，已改 |
| U4 | plane_capsule 帧基退化（`b_norm<0.5`）+ 两端候选 | `collision_primitive_core.py` L283-306 | `primitive_plane_shapes.cu` plane_capsule | **可验证** | **现有 `plane-capsule.xml` 是 identity（轴∥法线）→ `b_norm=0`，退化帧基已被覆盖**（reference count=1、dist=−0.0100000016）。缺的是**横躺（axis ⟂ n，b_norm=1）非退化帧基 + 2 端点** → 新 fixture `plane-capsule-lying.xml`（原写反，已改） |
| U5 | plane_cylinder 非退化分支的「4 候选全写入」 | `collision_primitive_core.py` L519-557 | `primitive_plane_shapes.cu` | **可验证** | **非退化分支已被现有 plane-cylinder case 覆盖**（倾 10°，`len_sqr=0.030154≫1e-12`，count=1）。真增量 = 让 4 条候选全进 cutoff：r=0.06/h=0.1 时需 center z < −0.0881(10°)/−0.0566(30°)，即**深穿透 ≈0.17**；fixture `plane-cylinder-deep-tilt.xml`（原写「30–45°」不够，已改） |
| U6 | mesh `nmesh>1`（不同 vertnum/polynum） | `collision_core.py` L118-141；`collision_convex.py` L911-919 | `product.cu` L155-162/L176（QA 核行号）；`multicontact.cu` L89-99/L118-121/L171-187 两侧各传自己的视图 | **可验证（进验收分母）** | `cube.obj`（8 顶点 / **12 三角面，MuJoCo 编译合并为 6 polygon**）+ `tetra.obj`（4v/4f）→ `nmesh=2`、`nmeshvert=12`、`nmeshpoly=10`、polyvertnum 合计 36、polymap 合计 36；数组算术按 M0b bins 尺寸核算（QA 独立核过） |
| U7 | mesh `degree>10` | 动态容量：`collision_convex.py` L1262-1289（`max(m.nmeshdegmax,3)`）+ `types.py` L1661-1662（**不在 collision_gjk.py**，QA 纠） | `multicontact_geom.cu` L383-384 `#define G5_MC_MESH_DEG_MAX 10`；检查 `multicontact_mesh.cu` L120 **与 L148**，`return 0` → `multicontact.cu` L195-197 单接触回退 | **仅限制存在（不进分母）** | 合成 obj：24 边锥（apex 邻接 24 面 → degree=24）；设备回退、上游多接触 → **判据红如实记录** |
| U8 | mesh `polygon>16` | 同上（`npolygonmax`） | `multicontact_geom.cu` L383-384 `G5_MC_MESH_FACE_MAX 16`；`multicontact_mesh.cu` L120/L148/L180 检查 | **仅限制存在（不进分母）** | 合成 20 边顶面棱柱（polyvertnum=20）→ 同上，只记录 |
| U9 | hfield-capsule 设备回归 | G5 case bins | `g10_p5_nvrtc_launcher`（hfield-sphere 家族） | **可验证（补 target，位级优先）** | 参考 `fixtures/g10-p5/references/hfield-capsule.json` 实测 **9712 B**（QA 纠：S1 误写 5961 B）；判据 = **位级优先 + 4ULP@scale 兜底**（hfield-sphere 先例 bit_equal=314/314）；bins 在 `g10-p5-gpu-kernels/gpu-window/cases/hfield-capsule/`；**无需新捕获窗** |

## 2. mesh 三档口径（Chief 已批）

- `nmesh>1`（U6）：**进验收分母**，判据沿用（位级优先 / 4·ULP@scale），对照新上游 GPU 参考。
- `degree>10`（U7）/ `polygon>16`（U8）：**不进分母**，只验「限制存在」；设备回退行为与上游差异如实记录，不宣称等价。
- mesh 分支层未逐条标注：由 **S2** 静态映射表补（device 函数↔pinned 行号+触发条件+已绑定 case）。

## 3. S3 捕获窗草案（报 Chief 签后才开）

- **新 fixtures**（S3 落地）：`fixtures/g10-p5-primitive-branches/` 增 `plane-cylinder-degenerate.xml`、
  `plane-box-corner-{face-down,face-up,tilt-x,tilt-y,corner}.xml`、`plane-capsule-lying.xml`、
  `plane-cylinder-deep-tilt.xml`；`fixtures/g10-p5-mesh-multicontact/` 增 `mesh-mesh-two-assets.xml`、
  `fan-cone-24.obj`、`prism-20.obj`。
- **新参考文件**（旧参考一律不动）：`docs/development-evidence/g10-p5-branch-closure-capture/upstream-branch-closure-gpu.json`
  （U1/U3/U4/U5/U6 正常判据组）+ `.../upstream-mesh-limits-gpu.json`（U7/U8 限制组，只记录）。
- **纪律**：离线 authoring、pinned venv + `wp.set_device("cuda")`；两次独立进程 + 独立 cache，raw 逐字节一致才复制 A；
  `captured_at` 取窗口开始；窗口内零 Rust 编译；guard `floor=min(10240, baseline_free−3072)`。
- **预算（QA O6 修正）**：显存增量按 **M3 捕获窗实测 `delta_peak=238 MiB`**（baseline used 2040 → 2278）报签，
  不再用「~15 MiB」；预计 wall **≤90 s**（case 数比 M3 多，仍按 M1/M3 同型估算）；1 个捕获窗。
- **S3 files**：`fixtures/g10-p5-primitive-branches/`、`fixtures/g10-p5-mesh-multicontact/`、
  `docs/development-evidence/g10-p5-branch-closure-capture/`、`scripts/authoring/g10-p5/`（捕获脚本）、
  `docs/development-evidence/changes/<S3-task>.json`（交付记录；QA 复核任务预登记）。

## 4. S4 验证窗草案

- 实体组成（口径写死）：新增 target `g10_p5_branch_closure_cuda`（U1/U3/U4/U5/U6，1 个二进制）+ `g10_p5_nvrtc_launcher`
  扩 hfield-capsule（U9）+ **既有回归 8 个**（M2 mesh m2、M3 mesh m3、G5 multicontact、primitive、plane、
  sphere_capsule、branches、stage2）= **预计 10 个实体**（M3 窗口是 9，因 launcher 已含在回归内并新增 1 个 target）。
- U7/U8 限制组在同一新 target 内以「只记录」方式输出，**不参与 pass/fail**；红/回退行为写入证据。
- **S4 files**：`tests/`、`tests/support/`、`src/runtime/collision_gpu/`、**`src/runtime/collision_gpu.rs`（父文件单列）**、
  `docs/development-evidence/g10-p5-branch-closure-validation/`、`docs/development-evidence/changes/<S4-task>.json`；
  QA 复核任务预登记。
- 窗口纪律：预构建冻结二进制 sha + 3 点基线 + compute-apps + 负载快照报签；窗内零 Rust 编译；逐实体独立采集 exit；
  ≤2 有效轮；他人重构建作废。

## 5. 顺序与预估

S1（本片，无 GPU，~1 天）→ S2（老片债务 + mesh 分支层静态映射，无 GPU，~0.5 天）→ S3（fixtures + 1 捕获窗，~0.5 天）
→ S4（设备测试 + 1 验证窗，~1–1.5 天）→ S5（台账四类分开 + 整片汇总，~0.5 天）。合计 ~3.5–4 天 + 2 窗。

## 6. 边界（不宣称）

- U2 记「不可达/防御分支」（含 f32 溢出注记与「未实测」边界）；U7/U8 记「仅限制存在」；三者都不写「覆盖了」。
- S1 未跑 GPU、未捕获、未改 fixture；所有「可验证」结论待 S3/S4 实证。

## 7. QA 观察收口（O1–O11，2026-10-27）

| 观察 | 级别 | 处置 |
| --- | --- | --- |
| O1 U3 4-pose 并集仅 5/8 | 一般 | 改为 5 pose（含 Rx(180) 面朝下）覆盖 8/8；注明现有 B1 case 只有 {0,1,3} |
| O2 U4 现有 case 事实颠倒 | 一般 | 现有 identity case 已是退化帧基；改为新增**横躺**（非退化帧基 + 2 端点）case |
| O3 U5 分支已被现有 case 覆盖 | 一般 | 明确非退化分支已覆盖；真增量为 4 候选全写入，需深穿透 ≈0.17（给出 z 阈值） |
| O4 U9 参考大小/位级表述 | 一般 | 参考改为 **9712 B**；判据写「位级优先 + 4ULP@scale 兜底」 |
| O5 pinned 容量出处写错 | 一般 | 改为 `collision_convex.py` L1262-1289 + `types.py` L1661-1662（collision_gjk.py 无） |
| O6 S3 显存预算低估 16× | 一般 | 按 M3 捕获窗实测 **delta_peak 238 MiB** 报签（不再用 15 MiB） |
| O7 U2 需 f32 溢出注记 | 建议 | 补 `size≥1.8446744e19 → inf` 注记 + abi 不校验 size + MJCF 编译器未实测 |
| O8 U2「guard 直调单测」不可落地 | 建议 | 证据=静态 diff；若加测试须标注「镜像」（无 CPU 实现） |
| O9 行号/define 位置精修 | 建议 | 按 QA 行号修正（L137-139、L800-834、L120&L148、`multicontact_geom.cu` L383-384、`multicontact.cu` 为「参与 mesh 那侧视图」） |
| O10 S3/S4 files/口径缺项 | 建议 | 补交付记录、S4 父文件 `src/runtime/collision_gpu.rs`、QA 预登记、S4 实体口径 10；U3 行矛盾与 cube.obj 三角面说明已修 |
| O11 docs-only QA 片 gate 口径 | 建议 | 本 QA 片无 gate 项（receipt 已写「gate 不适用」）；排程清单新增口径：docs-only QA 可无 gate 但须显式记录，含窗口片 QA 登记 gate 四项 |

**结论不变**：9 条可达性判定方向不变（U1/U3/U4/U5/U6/U9 可验证、U2 不可达防御、U7/U8 仅限制存在）；
本版只勘误 pose/构造/数字/出处/行号，不改判据与不宣称清单。
