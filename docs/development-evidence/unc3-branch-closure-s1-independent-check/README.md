# UNC-3 分支收口 S1 独立复核（可达性台账 / pinned 依据 / fixture 与窗口草案）

**任务**：`2026-10-27-unc3-branch-closure-s1-independent-check`（owner QA）。
**被复核提交**：`00fd72da`（父 `17d2708f`），复核时 HEAD = `00fd72da`（工作树对 `00fd72da` 无 tracked 改动）。
**结论**：**pass / 阻断 0 / 观察 11**（6 一般 + 5 建议）。未 commit、未 done。

本片是**只读复核**：不建 CUDA context、不跑 GPU、不捕获、不改任何产品/fixture 文件；
仅在 `docs/development-evidence/unc3-branch-closure-s1-independent-check/` 落证据。

## 0. 边界与不宣称

- 未重跑上游/设备对照：device 行为只按**源码语义**核（`src/physics/collision/cuda/*.cu`），
  没有 NVRTC 编译与新 window；`verify_s1.py` 只复算 pinned 公式数学，不声称设备执行结果。
- 不宣称 mesh 一般化已支持、不宣称任何分支「已覆盖」；U2/U7/U8 仍按 S1 口径记
  「不可达/防御」「仅限制存在」，**本复核不改变这三条定性**（只纠措辞与依据行号，见观察）。
- 无 pinned mujoco 运行环境（本机 `python -c "import mujoco"` 失败），U2 的「非法输入是否会被
  MJCF 编译器拒绝」未实测，见 O7。

## 1. 复核方法

1. 读被复核提交的 4 个文件 + 任务登记/收口记录（`.normify/changes/…-s1-reachability.json`）。
2. 逐条核 pinned 源码（`docs/evidence/d04-complex-physics/authoring-runs/run-5/installed-source/_src/`，
   rev `71da24d9…`）的行号与语义：`collision_primitive_core.py`、`collision_primitive.py`、
   `collision_core.py`、`collision_convex.py`、`collision_gjk.py`、`types.py`。
3. 逐条核设备实现：`primitive_plane_shapes.cu`、`primitive_plane_common.cu`、`multicontact.cu`、
   `multicontact_mesh.cu`、`multicontact_geom.cu`、`product.cu`、`support.cu` 的派发与上限。
4. **自写脚本独立复算**（不读交付数字）：`verify_s1.py` → `verify_s1.json`，内容见 §4。
5. 复核相邻既有证据（M0b/M3 bins 尺寸、M3 捕获窗 facts、B1 primitive 参考、G09 plane-box 输入、
   launcher 日志）作为交叉校验；这些是**已交付证据**，不是本轮新跑。
6. 证据自制：`manifest.json` 逐文件 sha256；`receipt.json` 记 task_id/判定；`claims-audit.json`
   记逐条 claim 的核对结果（行号 + 结论）。

## 2. 逐分支可达性判定（独立核）

| # | S1 判定 | 我的复核 | 依据（我读到的行号） |
| --- | --- | --- | --- |
| U1 | 可验证 | **成立** | pinned `collision_primitive_core.py` L495-517：`prjaxis=dot(n,axis)`（L499）、`prjaxis>0` 翻转（L500-502）、`vec=axis*prjaxis-n`（L508）、`len_sqr<1e-12` 回退 `(r,0,0)`（L513-517）；设备 `primitive_plane_shapes.cu` L125-131 同式、L133-139 `len_sqr>=1e-12` else 回退；派发 `product.cu` L399 |
| U2 | 不可达（非上游防御分支） | **成立（措辞需限定）** | pinned L334 直接 `normalize(cw_mul(…))`，L312-339 无零长/非有限保护；设备 `primitive_plane_shapes.cu` L100-102 新增 `!(length>0)||!isfinite(length)` → `return 0`。措辞见 O7 |
| U3 | 可验证 | **成立（pose 方案需修）** | pinned L366-390 逐 8 角点算 dist/pos；wrapper `collision_primitive.py` L797-834 对 8 条逐条 `write_contact`；设备 `primitive_plane_shapes.cu` L19-42 同序（i&1/i&2/i&4）。8 角点**可达**（所有朝向都合法），但 4-pose 方案覆盖不到 8，见 O1 |
| U4 | 可验证 | **成立（fixture 命名需改）** | pinned L283-308：`b_norm<0.5` 回退（L289-293）、两候选 = ±segment 端点（L297-303）；设备 `primitive_plane_shapes.cu` L60-80 同式。见 O2 |
| U5 | 可验证 | **成立（增量口径需改）** | pinned L519-557：4 候选（两端面 L526-538 + 侧面两点 L540-557）；设备 L144-168 同式。见 O3 |
| U6 | 可验证（进分母） | **成立** | pinned `collision_core.py` L113-147 逐 geom 由 `geom_dataid` 取 `vertadr/vertnum/graphadr/polynum/polyadr`（两套 asset 各自偏移）；设备 `product.cu` L155-162 同式、L176 `geom.mesh_polyadr = poly->mesh_polyadr[dataid]`、`multicontact.cu` L89-99/L118-121/L171-187 两侧各传自己的 `vertadr/mesh_polyadr`，共享视图只持全局指针 → `nmesh=2` 在设备侧可行（我按 M0b bins 尺寸核算：cube 6 poly/8 vert/24 polymap + tetra 4/4/12 = 10 poly/12 vert/36 polymap，满足 M0a 规则） |
| U7 | 仅限制存在 | **成立** | 设备 `multicontact_geom.cu` L383 `#define G5_MC_MESH_DEG_MAX 10`；检查在 `multicontact_mesh.cu` L120（vertex 特征）与 L148（edge 特征）→ `return 0` → `multicontact.cu` L195-197 `nface==0` 回退单接触。pinned 依据文件错，见 O5 |
| U8 | 仅限制存在 | **成立** | `multicontact_geom.cu` L384 `#define G5_MC_MESH_FACE_MAX 16`；`multicontact_mesh.cu` L180 `nvert>16 → 0` → 同上回退。pinned 依据文件错，见 O5 |
| U9 | 可验证（补 target） | **成立（参考大小写错）** | `fixtures/g10-p5/references/hfield-capsule.json` 实测 9712 B、含 input/output/determinism、provenance 为上游 CUDA（4070 Ti SUPER / mujoco_warp 3.15 / warp 1.15）；G5 case bins 齐全（23 个输入 bin + out1/out2），launcher 现 `CASES=[hfield-sphere,sdf-box,ellipsoid-box]`。见 O4 |

补充（我核过的设备派发）：`product.cu` L395/L399/L401 分别调 plane_capsule/plane_cylinder/plane_box
wrapper；`g5_plane_cylinder` 的 `prjaxis>0` 翻转与 pinned 一致，回退值也是 `(r,0,0)`。

## 3. 红线核对（未改代码/判据/ABI/fixtures、未跑 GPU、未捕获）

- `git show --name-status 00fd72da` = 4 个文件：`.normify/changes/…-s1-reachability.json`（M）、
  `docs/development-evidence/changes/…-s1-reachability.json`（A）、
  `docs/development-evidence/unc3-branch-reachability/README.md`（A）、
  `…/reachability.json`（A）。**无 `src/`、`tests/`、`fixtures/`、`scripts/`、`Cargo.*` 改动**。
- 无捕获产物（无 `samples.csv`/`facts.txt`/新 bins/新 reference），无 GPU 运行痕迹；与 README §6
  「未跑 GPU、未捕获、未改既有 fixture」一致。
- 证据卫生：两文件均 UTF-8 无 BOM，README 64 行 / 7808 B，`reachability.json` 162 行 / 5308 B（≤500 行）；
  交付记录 49 行 / 1492 B。交付记录里的 blob 声明与实际 `git hash-object` **一致**
  （README `8a3cbb92…`、reachability `55bd4b53…`）。
- 禁词扫描：S1 任务 `checks` 里 forbid_claim 的 6 个词（词表见任务 JSON）在 S1 三个证据文件中 **0 命中**
   （本报告自身也不复述该词表，避免自伤）。
- S1 任务自检项（任务 JSON `checks`）：json pointer `changes/<task>.json:/task_id` 成立；
  其余为 forbid_claim，0 命中。本轮未重跑 S1 的 gate（避免对已 verified 任务做状态变更），
  只按 checks 手工核。

## 4. 独立复算（`verify_s1.py` / `verify_s1.json`）

脚本只复算 pinned 公式 + 数学生成，输入取自仓库内的参考/证据（B1 `upstream-primitive-gpu.json`、
G09 `fixtures/plane-box/cases.json`、M0b bins 尺寸）；不读交付结论数字。

**(a) 现有 B1 plane-box case 只出现 3 个角点 —— 成立。**
用 reference 记录的真实 qpos（`0 0 0.06, w=0.9971866…, x=0.0435780…, y=0.0609905…`）、
盒 `0.1/0.08/0.07`、`margin=gap=0`（`write_contact` L255-256 `detected = dist < margin+gap`）复算：

| 角点 i | dist | 进 cutoff |
| --- | --- | --- |
| 0 | −0.004002423 | ✓ |
| 1 | −0.028330005 | ✓ |
| 2 | +0.009903290 | ✗ |
| 3 | −0.014424292 | ✓ |
| 4..7 | +0.1101…+0.1483 | ✗ |

我算出的三个负 dist 与 reference `dist = [−0.0040024184, −0.0283300020, −0.0144242905]` 一致
（double 复算 vs f32 记录差 ~1e-9），count 3 = reference count 3。→ 角点集合 `{0,1,3}`。

**(b) G09 plane-box 世界（CPU native 既有证据，作口径对照）。**
`flat4` = 4 角 `{0,1,2,3}`、`edge2` = 2 角 `{0,1}`、`corner1` = 1 角 `{1}` → 并集 `{0,1,2,3}`。
→ 全仓既有 plane-box 案例的并集也只到 `{0,1,2,3}`（底面 4 角），`4..7` 未出现过。

**(c) U3 的 4-pose 方案：并集只有 5/8（不成立）。**
盒 `0.1/0.08/0.07`、`δ=0.01`（最低角点 −0.01）、`margin=gap=0`：

| pose | 中心 z | 进 cutoff 的角点 |
| --- | --- | --- |
| ① 面朝下（R=I） | 0.060000000 | {0,1,2,3} |
| ② 绕 x 倾 45° | 0.096066017 | {0,1} |
| ③ 绕 y 倾 45° | 0.110208153 | {1,3} |
| ④ 单角朝下（对角 −z，54.7°） | 0.134337567 | {7} |

并集 = `{0,1,2,3,7}` = **5 个**，不是 8 个。②③④ 的角点都是①底面 4 角的子集（④ 换成别的对角
符号最多再新增 1 个，仍 ≤5）。原因：姿势都在「底面朝下」一侧，`z=+0.07` 的 4 个角点离平面 ≥0.088，
不可能进 cutoff。最小修补：加一个 180° 翻转 pose（`R=Rx(180)`）→ 该 pose 给 `{4,5,6,7}`，
并集 = `{0..7}` = **8/8**（脚本已验）。所以「4 个 xml 覆盖 8 角点」需改成
「①+①翻转（或 4 个不同面朝下的 pose）」；只做 4 个面朝下同类 pose（含 yaw 90°）不增加任何角点。

**(d) U5 的「4 条候选分别进 cutoff」需要深穿透。**
按 pinned L496-557 复算（r=0.06、h=0.1、plane n=+z，center z 线性平移）：

| 倾角 | len_sqr | 4 候选距离（center=0） | 4 条全 <0 所需 center z |
| --- | --- | --- | --- |
| 10°（现有 case） | 0.030154 | −0.108900 / +0.088062 / −0.093271 / −0.093271 | < −0.088062 |
| 20° | 0.116978 | −0.114490 / +0.073448 / −0.083709 / −0.083709 | < −0.073448 |
| 30° | 0.250000 | −0.116603 / +0.056603 / −0.071603 / −0.071603 | < −0.056603 |
| 45° | 0.500000 | −0.113137 / +0.028284 / −0.049497 / −0.049497 | < −0.028284 |

即：倾斜本身只让 `len_sqr ≥ 1e-12`（非退化分支），要让 4 条候选**全部** `dist<0` 必须把圆柱中心
压到平面下方（30° 时 z<−0.0566，最低点穿透 ~0.17）。现有 B1 plane-cylinder case（倾 10°、
center z=0.1）reference count=1 → 只写入 1 条候选；见 O3。

**(e) U1 的精确性（f32 逐步）。** 平面 n=+z、柱 quat=identity（轴 +z）：
`prjaxis = +1 → 翻转 −1`，`vec = (0,0,−1)*(−1) − (0,0,1) = (0,0,0)`，`len_sqr = 0.0` **精确 0**
（全部是 1.0/2 的幂运算），走回退 `vec=(0.059999999,…0,0)`；`dist1=−0.009999998`、`dist2=0.19`，
不产生 NaN。设备 L125-139 与 pinned L496-517 逐式同序（我按行比对），回退值同为 `(r,0,0)`。→ U1 构造成立。

**(f) U2 guard 的 f32 边界（静态推理）。** 设备 `length = sqrtf(dot(scaled,scaled))`（f32）：
`geom_size` 分量 ≥ `√FLT_MAX = 1.8446744e19` 时平方溢出 → `length = inf` → `!isfinite` → guard 触发
（复算：1.84e19 → dot 3.400e38 有限；1.85e19 → inf）。`abi.rs` 只校验长度/索引（L198-216、
L329-461），**不校验 `geom_size` 值域**。见 O7。

## 5. S3 / S4 草案核对

- 与 M3 既有纪律一致的部分（✓）：离线 authoring + pinned venv + `set_device("cuda")`；
  **两次独立进程 + 独立 cache、raw 逐字节一致才复制 A 为参考**（M3 实测 A/B/参考同 sha）；
  `captured_at` 取真实窗口开始（M3 = 00:48:22.577 = window start）；旧参考一律不动、新参考用新文件；
  窗口 3 点基线 + compute-apps + 负载快照报签、窗内零 Rust 编译、≤2 有效轮、他人重构建作废。
- hfield-capsule「无新捕获窗」✓：reference 9712 B（含 input/output/determinism）+ G5 bins 齐全
  （其 `out1/out2`、`meta.txt`、`probe-report.txt` 都在）；launcher 现 3 个 CASE，加第 4 个是小改。
- S3 files 范围：`fixtures/g10-p5-primitive-branches/`、`fixtures/g10-p5-mesh-multicontact/`
  （两目录均存在）、新 evidence 目录、`scripts/authoring/`（递归覆盖 `scripts/authoring/g10-p5/`）。
  bins/捕获脚本按 M1/M3 先例放各自 evidence 目录，范围够；**但**：缺交付记录
  `docs/development-evidence/changes/<S3 task>.json` 与 QA 预登记（见 O10）。
- S4 files 范围：`tests/`、`tests/support/`、`src/runtime/collision_gpu/`（条件项）、evidence 目录；
  **缺父文件 `src/runtime/collision_gpu.rs`**（模块父文件，注册新 target 时会改它；M2 已因漏父文件
  补过一次治理片）与交付记录/QA 预登记（见 O10）。
- 实体数口径：草案写「9/12 实体独立采集 exit」，M3 验证窗为 9 实体；S4 加「新 target + launcher
  第 4 例」后约 10，措辞需明确（见 O10）。

## 6. 观察（按级别）

### 一般（6）

- **O1 一般｜U3 的 4-pose 覆盖 8 角点不成立。** 复算并集 = `{0,1,2,3,7}` = 5/8（§4c）；
  ②③④ 的角点都被①底面覆盖，`z=+0.07` 侧 4 角离平面 ≥0.088 进不了 cutoff。补 180° 翻转 pose 后 8/8。
  建议：把 `plane-box-corner-{a,b,c,d}.xml` 改为「面朝下 + 翻转面朝下 + 两个斜面/单角」，
  或在 S3 落地时按脚本逐角验证后再定稿。
- **O2 一般｜U4 的现有 case 事实颠倒。** 现有 `fixtures/g10-p5-primitive/plane-capsule.xml` 是
  `qpos=identity` → 轴 ∥ 平面法线 → `b_norm = 0`（**已经是退化帧基**，reference count=1、
  dist=−0.0100000016）。S1 写「新增 `plane-capsule-upright.xml`（+ 现有 lying case）」正好相反：
  现有是 upright，缺的是**横躺**（`b_norm=1`、两端点 2 接触）。且 U4 的「退化分支」已被现有 case
  覆盖，增量只是「正常帧基 + 2 端点」。
- **O3 一般｜U5 的分支已被现有 case 覆盖，增量表述不准。** 现有 plane-cylinder 倾 10° →
  `len_sqr = 0.030154 ≫ 1e-12`，非退化分支已走（reference count=1）；U5 实际增量是
  「4 条候选全部写入」，需深穿透（§4d：30° 时 center z<−0.0566）。只写「倾斜 30–45°」不足以达成。
- **O4 一般｜U9 参考大小写错 + 「位级」表述偏强。** `fixtures/g10-p5/references/hfield-capsule.json`
  实测 **9712 B**（S1 写 5961 B）；且 launcher 的比对对象是冻结 probe `out1/`，不是该 JSON，
  「位级」是**期望**（hfield-sphere 先例 bit_equal=314/314、M3 验证窗），实际断言是
  「位级优先 + 4ULP@scale 兜底」。建议写「位级优先（hfield-sphere 先例）/ULP 兜底」。
- **O5 一般｜U7/U8 的 pinned 依据文件写错。** `nmeshdegmax`/`npolygonmax` 在 pinned
  **`collision_gjk.py` 里根本不存在**（我全文件 grep 无命中）；动态容量计算在
  **`collision_convex.py` L1262-1289**（`npolygonmax=max(m.npolygonmax,16)`、`nmeshdegmax=max(m.nmeshdegmax,3)`），
  Model 字段定义 `types.py` L1661-1662（说明 L1126-1127）。结论「仅限制存在」不变，仅引用需改。
- **O6 一般｜S3 显存预算与 M3 事实不符。** S1 写「显存增量 ~15 MiB 量级（同 M3 捕获窗）」；
  M3 捕获窗 facts 实测 `delta_peak_mib = 238`（baseline used 2040 → used_peak 2278、free_min 13784）、
  wall = runA 16.2 s + runB 6.1 s + compare。15 MiB 低估 ~16×（除非它指「数据增量」而非显存）。
  窗口签核用的是这个数字，建议改为实测口径或注明口径差异。

### 建议（5）

- **O7 建议｜U2「不可达」需限定为「常规合法输入」。** 设备 guard 是 `!(length>0)||!isfinite(length)`；
  f32 下 `geom_size` 分量 ≥ 1.8446744e19 时平方溢出 → `length=inf` → guard 真会触发（返回 0 接触），
  而上游 `normalize` 会 NaN；`abi.rs` 不校验 size 值域。该极端输入是否被 MJCF/MuJoCo 编译器拒绝
  我**未实测**（无 pinned mujoco）。建议 S1/S5 措辞写「常规合法输入不可达；f32 溢出极端 size 属防御
  触发（上游 NaN），不设 fixture」。
- **O8 建议｜U2 的「guard 直调 CPU 单测」目前不可落地。** 产品侧没有 plane_ellipsoid 的 CPU 实现
  （只有 `src/physics/collision/cuda/primitive_plane_shapes.cu` 设备版；`src/runtime/collision_gpu/validate.rs`
  只做输入校验），要「直调 CPU 单测」只能新写 Rust 镜像（复刻 guard 条件）——那就不是设备代码。
  建议 S1 明确：U2 证据 = 静态 diff + 极性说明，若加测试须标注「镜像」。
- **O9 建议｜行号精度小偏差（不改变判定）。** `primitive_plane_shapes.cu` 的 else 回退实际在
  L137-139（引用写 L117-134）；`plane_box_wrapper` 的 8 次 `write_contact` 是 L800-834（引用写 L800-831）；
  U7 的 degree 上限检查有两处（`multicontact_mesh.cu` L120 与 L148）；`#define` 在
  `multicontact_geom.cu` L383-384（`multicontact_mesh.cu` L11-15 是注释，引用写成了定义处）；
  `multicontact.cu` L59-61 是「取参与 mesh 的那一侧 geom 视图（geom1 或 geom2）」，
  不是「取 geom1 视图」（两侧 mesh 时取 geom1 的视图，但 vertadr/polyadr 各侧自传，语义没问题）。
- **O10 建议｜S3/S4 files 与实体口径需补齐。** （i）S4 缺父文件 `src/runtime/collision_gpu.rs`；
  （ii）S3/S4 都缺**交付记录** `docs/development-evidence/changes/<task>.json` 与「QA 预登记」写法；
  （iii）「9/12 实体」口径不明，建议直接列实体清单（M3 为 9，S4 预计 10）。
  另：U3 行内自相矛盾（branch 列「现有 case 只出现 3 个」vs trigger 列「①面朝下（4 角，现有 case）」），
  建议 3 个限定为「现有 B1 设备 case」并注明 G09 已覆盖底面 4 角；`cube.obj（8v/6f）」建议注明
  「文件为 8v/12 三角面，MuJoCo 编译合并为 6 polygon」（M0b 实测 `mesh_polynormal` 72 B = 18 float = 6）。
- **O11 建议｜docs-only QA 片的门禁登记不一致。** 本复核任务的 `.normify` 登记只有
  `json(receipt)` + `forbid_claim` 两个 check，没有 `kind=gate` 项 → `normify gate` 返回
  「任务没有引用任何门禁」（exit 1），只有 `normify check` 可跑（exit 0）。M3 的 QA 片当年登记了
  fmt/clippy/clippy-cuda/unit 四项 gate。若验收口径是「gate/check exit 0」，建议 QA 复核片统一登记
  四项 gate（docs-only 也能 cache-hit 跑），或在登记时显式写「gate 不适用」。本片我未擅自改任务
  checks（避免与 Arch 的 gate/done 流程冲突），只把实况记在 `receipt.json`。

## 7. 证据清单

| 文件 | 内容 |
| --- | --- |
| `README.md` | 本报告 |
| `claims-audit.json` | 逐条 claim ↔ pinned/设备行号 ↔ 我的核对结论 |
| `verify_s1.py` | 复算脚本（只读输入：B1 reference / G09 cases / M0b bins 尺寸） |
| `verify_s1.json` | 脚本输出（§4 全部数字的机器可读版） |
| `manifest.json` | 本目录逐文件 sha256 |
| `receipt.json` | 任务 id / 判定 / 计数 / 复核 HEAD / gate-check 指引 |

复现：`python docs/development-evidence/unc3-branch-closure-s1-independent-check/verify_s1.py`
（本机无 `python` 别名时用 DSH 自带解释器；无第三方依赖除 numpy）。
