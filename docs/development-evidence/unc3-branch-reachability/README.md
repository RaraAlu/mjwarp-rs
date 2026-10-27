# UNC-3 分支收口 S1：逐分支可达性台账 + case/fixture 设计（无 GPU）

**任务**：`2026-10-27-unc3-branch-closure-s1-reachability`（owner Arch）。**本片只读分析与设计**：
不改代码/判据/容差/ABI，不改既有 fixtures，不跑 GPU、不捕获。方向③（补未覆盖分支）第一片。

**pinned 版本**：`71da24d956378a87a703b6e1442b13aec0c4ac29`（`docs/evidence/d04-complex-physics/authoring-runs/run-5/installed-source/_src/`）。
**设备实现参考**：`src/physics/collision/cuda/primitive_plane_shapes.cu`、`primitive_plane_common.cu`、
`multicontact_mesh.cu`、`support.cu`。

## 1. 可达性台账（逐条：可验证 / 不可达 / 仅限制存在）

| # | 分支 | pinned 依据 | 设备实现 | 判定 | 合法 MJCF 触发构造 |
| --- | --- | --- | --- | --- | --- |
| U1 | plane_cylinder 退化（`len_sqr<1e-12` → x 轴回退） | `collision_primitive_core.py` L496-517：`vec=axis*prjaxis-n`，退化时 `vec=(r,0,0)` | `primitive_plane_shapes.cu` L117-134（`len_sqr>=1e-12` 分支 + else 回退） | **可验证** | plane 法线 +z；cylinder `quat=identity`（MuJoCo cylinder 默认轴 = +z）、半高 h、中心 z = h−δ（δ=0.01 穿透）→ `prjaxis=−1`、`vec=(0,0,0)` 精确 0 → 4 条候选全走退化分支 |
| U2 | plane_ellipsoid「非上游退化保护」 | `collision_primitive_core.py` L312-339 **无**零长/非有限保护（直接 `normalize(cw_mul(rotᵀn,size))`） | `primitive_plane_shapes.cu` L93-100 新增 `if (!(length>0)||!isfinite(length))` 保护 | **不可达（非上游防御分支）** | 合法 MJCF 要求 ellipsoid `size` 各分量 >0 且有限 → `normalize` 输入恒非零；无法由合法输入触发。**不造假 MJCF**：以「静态 diff（pinned 无此保护）+ guard 直调 CPU 单测」证明其只是防御 |
| U3 | plane_box 8 角点覆盖（现有 case 只出现 3 个） | `collision_primitive_core.py` L366-390：8 角点逐一算 `dist/pos`；wrapper L800-831 对 8 条候选逐条 `write_contact`（margin/gap cutoff） | `primitive_plane_shapes.cu` plane_box 对应实现（8 候选 + cutoff） | **可验证** | 4 组合法 pose：①面朝下（4 角，现有 case）②绕 x 倾 45°（2 角）③绕 y 倾 45°（2 角）④单角朝下（倾 54.7°）→ 4 个 pose 逐步换 90°/镜像朝向，使 8 个角点各自至少一次进 cutoff |
| U4 | plane_capsule 帧基退化（`b_norm<0.5` → y/z 轴回退）+ 两端候选 | `collision_primitive_core.py` L283-306：`b=normalize(axis−n(n·axis))`，`b_norm<0.5` 时按 n[1] 选 y/z；两候选 = 两端端点 | `primitive_plane_shapes.cu` plane_capsule 对应实现 | **可验证** | capsule 直立（axis ∥ plane 法线，`quat=identity`）→ `b_norm≈0` → 退化帧基；横躺（axis ⟂ n）→ 正常帧基。两个 pose 各 2 候选 |
| U5 | plane_cylinder 非退化 4 候选 | `collision_primitive_core.py` L519-557：两端面 + 侧面三角两点共 4 候选 | `primitive_plane_shapes.cu` L117-134 之后 4 候选实现 | **可验证** | cylinder 倾斜 ~30–45°（axis 与 n 成角）→ `len_sqr>1e-12`、4 条候选分别进 cutoff |
| U6 | mesh `nmesh>1`（真多 mesh：不同 vertnum/polynum） | `collision_core.py` L118-141（两侧共享全局数组，逐 geom `dataid` 偏移）；`collision_convex.py` L911-919 | `product.cu` L155-163（按 dataid 解析 per-geom `vertadr/vertnum/graphadr`）；`multicontact.cu` L59-61 取 geom1 视图 | **可验证（进验收分母）** | 两套不同 asset：`cube.obj`（8v/6f）+ `tetra.obj`（4v/4f）→ `nmesh=2`、`nmeshvert=12`、`nmeshpoly=10`、poly slice 按 M0a 规则自洽 |
| U7 | mesh `degree>10` | `collision_gjk.py` 动态上界（`nmeshdegmax`） | `multicontact_mesh.cu` L11-13、L120：`G5_MC_MESH_DEG_MAX=10`，超界返回 0 → 单接触回退 | **仅限制存在（不进分母）** | 合成 obj：24 边锥（apex 邻接 24 个三角面 → degree=24）；期望设备回退、上游多接触 → **判据红如实记录** |
| U8 | mesh `polygon>16` | 同上（`npolygonmax`） | `multicontact_mesh.cu` L13-15：`G5_MC_MESH_FACE_MAX=16`，超界返回 0 → 单接触回退 | **仅限制存在（不进分母）** | 合成 20 边顶面棱柱（polyvertnum=20）→ 同上 |
| U9 | hfield-capsule 设备回归（M2 继承未覆盖） | — | 现有 `g10_p5_nvrtc_launcher`（已有 hfield-sphere） | **可验证（补 target，位级）** | 现有 G5 case bins：`docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases/hfield-capsule/`；参考：`fixtures/g10-p5/references/hfield-capsule.json`（5961 B，含 input/output/determinism）→ **无需新捕获窗** |

## 2. mesh 三档口径（Chief 已批）

- `nmesh>1`（U6）：**进验收分母**，判据沿用（位级优先 / 4·ULP@scale），对照新上游 GPU 参考。
- `degree>10`（U7）/ `polygon>16`（U8）：**不进分母**，只验「限制存在」——设备按设计回退（返回 0 → 单接触），
  与上游参考**如实记录差异**，不宣称等价；这两条在 S5 汇总里单列「仅限制存在」。
- mesh 分支层未逐条标注：由 **S2** 用静态映射表补（device 函数↔pinned 行号+触发条件+已绑定 case），不逐 case 硬凑。

## 3. S3 捕获窗草案（草案，报 Chief 签后才开）

- **新 fixture（S3 落地，不在本片改）**：`fixtures/g10-p5-primitive-branches/` 增
  `plane-cylinder-degenerate.xml`、`plane-box-corner-{a,b,c,d}.xml`、`plane-capsule-upright.xml`、
  `plane-cylinder-tilted.xml`；`fixtures/g10-p5-mesh-multicontact/` 增 `mesh-mesh-two-assets.xml`、
  `fan-cone-24.obj`（degree 24）、`prism-20.obj`（polygon 20）。
- **新参考文件（新文件，旧参考一律不动）**：
  `docs/development-evidence/g10-p5-branch-closure-capture/upstream-branch-closure-gpu.json`（含 U1/U3/U4/U5/U6
  正常判据组）与 `.../upstream-mesh-limits-gpu.json`（U7/U8 限制组，只记录不判 pass）。
- **捕获纪律**：离线 authoring，pinned venv + `wp.set_device("cuda")`；**两次独立进程 + 独立 cache，raw 逐字节一致**
  才复制 A 为参考；`captured_at` 取真实窗口开始；预计 wall ≤90 s、显存增量 ~15 MiB 量级（同 M3 捕获窗）。
- **S3 files**：`fixtures/g10-p5-primitive-branches/`、`fixtures/g10-p5-mesh-multicontact/`、
  `docs/development-evidence/g10-p5-branch-closure-capture/`、`scripts/authoring/`（新增捕获脚本）。

## 4. S4 验证窗草案

- 实体：新增 `g10_p5_branch_closure_cuda`（U1/U3/U4/U5/U6）+ `g10_p5_nvrtc_launcher` 加 hfield-capsule（U9）+
  既有回归（M2/M3/M2 mesh、G5 multicontact 3、primitive 3、launcher、plane、sphere_capsule、branches、stage2 synth）。
- 限制组（U7/U8）在同一 target 内以「只记录」方式输出，**不参与 pass/fail**，红/回退行为写入证据。
- **S4 files**：`tests/`、`tests/support/`、`src/runtime/collision_gpu/`（若需注册新 target）、
  `docs/development-evidence/g10-p5-branch-closure-validation/`。
- 窗口纪律：预构建冻结二进制 sha + 3 点基线 + compute-apps + 负载快照报签；窗内零 Rust 编译；
  9/12 实体独立采集 exit；≤2 有效轮；他人重构建作废。

## 5. 顺序与预估

S1（本片，无 GPU，~1 天）→ S2（老片债务 + mesh 分支层静态映射，无 GPU，~0.5 天）→ S3（fixtures + 1 捕获窗，~0.5 天）
→ S4（设备测试 + 1 验证窗，~1–1.5 天）→ S5（台账四类分开 + 整片汇总，~0.5 天）。合计 ~3.5–4 天 + 2 窗。

## 6. 边界（不宣称）

- U2 记「不可达/防御分支」，U7/U8 记「仅限制存在」；三者都**不写「覆盖了」**。
- 本片未跑 GPU、未捕获、未改任何 fixture；所有「可验证」结论待 S3/S4 窗口实证。
