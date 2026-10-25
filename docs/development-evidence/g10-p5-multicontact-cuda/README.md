# G10@P5 设备缺口片：multicontact（ccd_pair ≤4 接触）+ primitive box-box

- 任务：`2026-10-25-g10-p5-multicontact-cuda`（p50，Backend）。
- 状态：设备实现 + CPU 验证完成；GPU 窗口对照**待双签后执行**（本文件窗口小节将在跑完后回填）。
- 上游：pinned `71da24d956378a87a703b6e1442b13aec0c4ac29`，冻结源
  `docs/evidence/d04-complex-physics/authoring-runs/run-5/frozen-source/_src/`。

## 1. 范围（Arch/Chief 2026-10-25 拍板）

1. 设备 `eval_ccd_write_contact` 的 multicontact 分支（`use_multiccd=true` 静态特化，<=4 接触）：
   覆盖 `cylinder-box`（fixture `multiccd="enable"`）。
2. **`box-box-margin` 纳入本片**：其上游 dispatch 在 fixture `nativeccd="disable"` 下把
   `(BOX,BOX)` 从 CONVEX 改走 **primitive `box_box_wrapper`**（8 候选按 `detected` 过滤，
   5 接触/world）。因此一并移植 `box_box` 核心；**归属说明：`box_box` 属 primitive family，
   为覆盖该 case 一并移植，不属 multicontact 分支**。
3. `mesh-mesh`：fixture 未启用 multiccd → 上游单接触，按既有 `Contact` 路径对照；
   **mesh multicontact 分支不实现（需 mesh polygon ABI 输入），记未覆盖**。
4. 参考复用 QA 已提交 `qa-upstream-gpu.json`（不新捕获）；运行时重算 **23 项输入 sha256** 守卫。
5. 不改 `CollisionArgs` ABI；PTX 入口 4→6；`nvrtc_smoke.rs` 入口计数改实测（test-only，
   本文件声明、不加注）。

## 2. 实现对照表（上游函数 → 设备文件）

| 上游（`collision_gjk.py` / `collision_primitive_core.py`） | 设备实现 | 文件（行数） |
| --- | --- | --- |
| `multicontact` L2216-2451（BOX/CYLINDER 分支） | `g5_multicontact` + `g5_mc_result` | `multicontact.cu`（189） |
| `_polygon_clip` L2061 + `_area4`/`_polygon_quad`/`_plane_normal`/`_halfspace`/`_plane_intersect`/`_witness_on_face` | `g5_mc_polygon_clip` 等 | `multicontact_clip.cu`（257） |
| `_feature_dim`/`_aligned_faces`/`_aligned_face_edge`/`_box_normals`/`_box_edge_normals`/`_box_face`/`_cylinder_normals`/`_cylinder_edge_normals`/`_cylinder_face`/`_set_edge` | `g5_mc_*` helpers | `multicontact_geom.cu`（361） |
| `box_box` L595-1049 + `_compute_rotmore` L563 | `g5_box_box` / `g5_box_box_result` | `boxbox.cu`（474） |
| `box_box_wrapper`（`collision_primitive.py` L1250） | `g5_box_box_wrapper`（8 次 `g5_write_contact(id_=i)`） | `boxbox_wrapper.cu`（20） |
| `eval_ccd_write_contact` L769-1026（multicontact 分支与逐接触写入） | `g5_ccd_pair_write(..., use_multiccd, ...)` 新分支 | `contact.cu`（271，原 221） |
| 产品入口 | 新增 `mjwarp_collision_contact_multi`、`mjwarp_collision_boxbox` | `product.cu`（357，原 318） |

- 设备源拼装：`DEVICE_SOURCES` **21 段**（16→21），顺序
  common→geom→support→simplex→gjk_core→polytope→polytope_build→epa→ccd→sdf_octree→sdf_math→
  sdf_narrowphase→hfield→multicontact_geom→multicontact_clip→multicontact→contact→primitive→
  primitive_wrapper→boxbox→boxbox_wrapper，最后追加 `product.cu`。
- 「静态特化」映射：上游按 geom pair + `use_multiccd` 在 kernel 构建期特化；设备用**独立入口**
  表达（`contact_multi` 传 `use_multiccd=true`，既有 `contact` 传 false），因此 ABI 不变。
- multicontact 细节逐式对齐：`witness1[0]=x1/witness2[0]=x2`；`dists[i] if n>1 else dist`；
  位置 `0.5*(witness1[i]+witness2[i])+origin`；**frame 只算一次**（复用 EPA witness[0]）；
  `id_=i` 写 `geomcollisionid`；`n<1` 回退单接触。
- `box_box` 细节：8 槽初值 `MJ_MAXVAL`；四处早退逐行保留；未填充槽由 `g5_write_contact`
  的 `detected = dist < margin + gap` 过滤；mat3×mat3 用显式 `__fmaf_rn` 升序累加
  （匹配上游 `mat.h::mul` 的 fmaf）。

## 3. 参考与判据

- 参考：`docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-upstream-gpu.json`
  的 `cases.{cylinder-box, box-box-margin, mesh-mesh}`（QA 已提交；`two_captures_raw_bitwise_identical=true`）。
- 输入守卫：G5 冻结 case 的 23 项输入 bin 在测试运行时重算 sha256/count，与 QA 参考
  `input_bins` 全同才比较（`tests/support/multicontact_qa_support.rs`）。
- 判据（G5 既有口径，未改）：按 `(worldid, geomcollisionid)` 稳定排序后逐字段；位级优先；
  `dist` 自身量级 ≤4 ULP（`ref==0` 用 4 ULP@1.0）；`pos` 按 `scale=max_i|ref_i|` 的
  `4·ULP(scale)`；`frame` 按 scale=1.0 的 4 ULP；参数/整数/`nacon`/`overflow` 零容差；
  `nccd` 只记录（参考为 null，与 G5 同口径）；`efc_address` 逐项期望 -1。
- 内核选择：`cylinder-box`→`ContactMulti`；`box-box-margin`→`BoxBox`；`mesh-mesh`→`Contact`。

## 4. CPU 侧验证（窗口外，已完成）

- `cargo test --locked --features cuda-probe --lib runtime::collision_gpu -- --include-ignored`
  → **20 passed / 0 failed**（新增 `multicontact_tests` 3 条：21 段顺序、新入口唯一性、
  `boxbox` 类型校验）。
- NVRTC 冒烟（CPU，无 CUDA 上下文）：拼装源 **197428 B**、sha256
  **`682787c964e216a55ef1be479024f799e4215ba9457519b0f5892de74bab9790`**；PTX **3,556,510 B /
  6 入口**。
- `cargo fmt --check`、`cargo clippy --features cuda-probe --all-targets -- -D warnings` 绿；
  `normify gate`：fmt/clippy/clippy-cuda 通过、unit **595 过 / 0 败**。
- 放行前 CPU 复算（authoring，warp CPU；`authoring/cpu_*.py`）：
  `multicontact`(cylinder-box) 4 接触/dist 0 ULP 位级/frame 位级；`box_box`(box-box-margin)
  10/10、5+5/world、dist ≤1 ULP、frame 位级（详见 `plan.md` §7）。

## 5. GPU 窗口（已执行 2026-10-25 23:40:07–23:41:14，elapsed 69.2 s）

- HEAD **`9d337bfe77b951a6550fd9c13f9a95a2b86ce84e`**（`head_match=true`）；窗口只跑窗口外预构建的
  三个 test binary（lib `cf1a59c9…` / launcher `a96d059e…` / multicontact `092cb74c…`），窗口内不编译。
- **基线（Arch 裁定按当前基线走；取开窗前 3 点均值）**：used **4061 / 4070 / 4030** → 均值
  **4054 MiB**；free 12001；util 30%（桌面瞬时）；31°C；本任务相关计算进程 **0**。
  **漂移事实**：相对批准时 3646 MiB 漂 **+384~+424 MiB**（窗口前预采序列曾达 +263~+477），来源为
  桌面占用（非本任务计算进程）；Arch/Chief 裁定按**功能等价口径接受**（本次不是性能测量），
  **不得解读为「窗口无干扰」**。守卫按增量判（基准 = 3 点均值）。
- **A CPU NVRTC 冒烟**（不创建 CUDA 上下文）`exit 0`，1 passed（19.92 s）：
  拼装源 **197428 B / sha256 `682787c9…`**、PTX **3,556,510 B / 6 入口**。
- **B NVRTC launcher 回归**（21 段拼装后防回归）`exit 0`，1 passed（25.82 s）：
  `hfield-sphere` **314/314**、`sdf-box` **3122/3122**、`ellipsoid-box` **80/80**，ULP 全 0。
- **C multicontact 3 case** `exit 0`，1 passed（19.57 s），逐 case（按 worldid+geomcollisionid 排序后）：

| case | 内核 | nacon | nccd | 位级比较 | max_dist_ulp | max_pos_ulp | max_frame_ulp | failures |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `cylinder-box` | `ContactMulti` | 8 | 2 | **314/314** | 0 | 0 | 0 | 0 |
| `box-box-margin` | `BoxBox` | 10 | 0 | **391/392** | 0 | 1 | 0 | 0 |
| `mesh-mesh` | `Contact` | 2 | 2 | **80/80** | 0 | 0 | 0 | 0 |

  - `box-box-margin` 的 1 项非位级是 `pos` 的 1 ULP（判据为 4·ULP(scale)，无失败）。
  - 内核归属：`cylinder-box` = multicontact 分支；`box-box-margin` = primitive `box_box`
    （为覆盖该 case 一并移植，不属 multicontact 分支）；`mesh-mesh` = 既有单接触路径。
- **峰值/守卫/残留**：0.25 s 采样 **235 点**，峰值 used **4149 MiB（+95 vs 均值 4054）**，
  util max 77%、temp max 31°C；守卫 3072 MiB **未触发**；窗口内 free 最低 11937 MiB（>10 GiB
  停跑阈值，未触发）；结束 used **3961 MiB**；跑前跑后 `query-compute-apps` 本任务相关进程
  **0/0**，无残留。
- 证据：`window-log.txt`（含基线 3 点/漂移事实/每步守卫与 free 检查/计算进程检查）、
  `window-output.txt`（三步命令原文）、`window-memory-samples.txt`、`artifacts-sha256.txt`（21 项）。

## 6. 实现中记录的简化/边界

1. multicontact 的 `n<1` 回退内联到 `g5_multicontact`（上游由调用方回退）：失败/无多接触
   一律 `n=1 + witness=x1/x2`，故实际返回 1..4；`ncontacts==1` 时距离仍用 EPA 的 `dist`
   （与上游 `dists[i] if ncollision > 1 else dist` 一致）。
2. mesh 分支全部省略（`_mesh_normals`/`_mesh_edge_normals`/`_mesh_face` 与 mesh 分派）：
   mesh 对恒单接触；mesh-mesh fixture 未启用 multiccd，故本片该 case 应可对齐（窗口验证）。
3. `_polygon_clip` 的 32 顶点 scratch 上界与上游同源（`2*npolygonmax`），未加额外越界守卫
   （保持逐式等价；上游同样不设检查）。
4. `boxbox` 面路径未加 `n≤8` 截断（上游 `mat83f` 同样没有），退化 face-face 与上游一样
   存在越界假设。
5. `1.0f - 1e-12f` 在 f32 下恒为 1.0f（上游运行时同样），保留原式逐行对应。
6. `nvrtc_smoke.rs` 只改入口计数为实测（test-only；入口列表 4→6），不改产品语义。

## 7. 未覆盖

1. **primitive dispatch 13 组合里未实现的 10 个**（`nativeccd="disable"` 时
   `MJ_COLLISION_TABLE` 的 PRIMITIVE 12 组合 + `(BOX,BOX)`）：`plane-sphere`、`plane-capsule`、
   `plane-ellipsoid`、`plane-cylinder`、`plane-box`、`plane-mesh`、`sphere-sphere`、
   `sphere-capsule`、`sphere-cylinder`、`capsule-capsule`。本片已实现的 primitive 组合为
   `sphere-box`、`capsule-box`（primitive 片）+ `box-box`（本片）；其中 **`box_box` 属
   primitive family，为覆盖 `box-box-margin` 一并移植，不属 multicontact 分支**。
2. **mesh 的 multicontact 分支未实现**（需要 mesh polygon ABI 输入：`mesh_polynormal`/
   `mesh_polyvert*`/`mesh_polymap*`）；`mesh-mesh` 仅按单接触路径对照（其 fixture 未启用
   multiccd，与上游单接触 dispatch 一致）。
3. multicontact 的 `n<1` 回退已内联（见 §6.1）；本片未覆盖“mesh 参与的多接触”与
   “非 box/cylinder 类型的多接触”。
4. 未做性能测试（本片无性能判据）。
5. 不改判据/已 verified 数字、不改 `Cargo.*`/fixtures XML；不声明本片 3 case 之外的结论。

## 8. 设备文件 sha256（本轮实现，供窗口 artifacts 对照）

| 文件 | 行数 | sha256 |
| --- | --- | --- |
| `multicontact_geom.cu` | 361 | `f3087fe1…` |
| `multicontact_clip.cu` | 257 | `8710d609…` |
| `multicontact.cu` | 189 | `bf86115b…` |
| `boxbox.cu` | 474 | `050de787…` |
| `boxbox_wrapper.cu` | 20 | `f6885354…` |
