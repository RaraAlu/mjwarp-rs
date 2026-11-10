# G-L2 判据 A FAIL — 只读 RCA 证据（perworld L v2 灯 / world0·world1 cam1 共 5 px）

- 任务：`2026-11-11-render-perworld-l-v2-gl2-criterion-a-rca`（owner Arch；本卡只记录 RCA，零 GPU、零重跑、零修复）
- 结论：**判据 A 的 5 个 1 字节失配来自 mesh 法线链「多归一化一次」的源码级差异（H2）**，站点唯一：我方 `src/render/kernel/cuda/render_math.cu:239`（`normalize3_mesh(min_normal)`）与 `src/render/kernel/cuda/render_common.cu:200`（`normalize3_mesh(mat_vec(mat, n_local))`）；上游 pinned `mujoco_warp 71da24d9` 只在 `mat @ cross` 之后归一化一次（`mujoco_warp/_src/ray.py:727-729`）。
- 本卡**不宣称**判据通过、不宣称与上游等价、不作发布依据；修复另卡。

## 1. 归属纠正（此前的两个误读）

1. `seg` 是 `(objid, objtype)`。`seg = [3, 5]` 的第二个分量是 MuJoCo **object type = `mjOBJ_GEOM`(5)**，**不是** geom 类型 5（cylinder）。诊断 dump 的 `attribution.geom_type` 直接取了 `seg[1]`，命名误导（`tests/support/render_bvh/light/diag.rs:207` 起）；后续把该字段改名 `objtype` 已列入 errata。
2. 真实命中是 **geom_id 3 = model `geom_type=7`（MESH）**，材质 `geom_matid[3]=0` = `mat-red`（`grid` checker 纹理、`emission=0.05`、`specular=0.4`、`shininess=0.6`）；dump 的 `mat_rgba=[0.9,0.1,0.1,1]`、`mat_texid_row=[-1,0,...]` 一致。**不是 cylinder。**

## 2. 输入与方法（只读 f32 重放）

输入（全部只读）：

| 文件 | 身份 |
| --- | --- |
| 诊断 dump | `target/render-bvh/perworld-l-v2-gl2/diag/pixel-dump.json` sha256 `68b3f78ed9586a83c47cfad8ab722407d27c8571cd82c92a5f9b4c6050326089` / **51,388 B** |
| 参考 | `fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040/{state,rgb,depth,seg}.npz`（G-L1 冻结参考） |
| 模型 dump | `tests/support/render_bvh/light/model-315.json`（pinned 3.15） |
| 上游源码 | `target/mjwarp-parallel/upstream`（pinned `71da24d956378a87a703b6e1442b13aec0c4ac29`，clean） |
| warp 头 | `include/warp-headers/intersect.h`（`intersect_ray_tri_woop`）、`vec.h`/`mat.h`（dot/normalize 语义） |

方法（脚本 `artifacts/replay_mesh_normal.py`，输出 `artifacts/replay-output.txt`）：

1. 用 dump 的 5 条射线（`direction_bits`，`offset_bits=0`）、相机 `xpos/xmat` 位型、`state.npz` 的 geom 3 `geom_xpos/geom_xmat` 位型、模型 mesh 顶点/面/顶点法线，做 **f32 重放**（`FMA` 以 float64 精确模拟、`diff_product`、woop `intersect_ray_tri_woop` 逐式）。
2. 命中校验：`depth = t × (−dir_local.z)`（我方 kernel 写法 `render_megakernel.cu:244-246`）与 dump 的 depth bits 比对；并用 float64 独立复核命中面/uv/t。
3. 分别按**上游公式序**（`normalize(mat @ cross)`）与**我方公式序**（`normalize(mat @ normalize(cross))`）重算 pre-normal / final-normal，比较位型与 ULP。
4. 无任何写入、无 GPU、无重跑；重放只读上述文件。

## 3. 唯一站点与两侧公式对照

| 环节 | 上游（参考，pinned `71da24d9`） | 我方（产品） |
| --- | --- | --- |
| 三角法线 | warp `intersect_ray_tri_woop` 返回 **未归一化** `cross(ab, ac)`（`include/warp-headers/intersect.h`，woop 尾部 `*normal = cross(ab, ac)`） | `src/render/kernel/cuda/render_math.cu:239`：`*out_normal = normalize3_mesh(min_normal)` —— **先归一化** |
| 变换到世界 + 归一化 | `mujoco_warp/_src/ray.py:727-729`：`normal = mat @ n`，`normal = wp.normalize(normal)` —— **整链只归一化一次** | `src/render/kernel/cuda/render_common.cu:200`：`*normal = normalize3_mesh(mat_vec(mat, n_local))` —— **第二次归一化** |

即：上游 = `normalize(mat @ cross)`；我方 = `normalize(mat @ normalize(cross))`。差异行就是 `render_math.cu:239` 与 `render_common.cu:200`（去掉前者对法线的归一化、保留后者，即与上游同构）。同一路径也服务 hfield（gtype 1）/SDF（gtype 8）；本场景无这两类。

## 4. 为什么正好是这 5 个像素

重放结果（`artifacts/replay-output.txt`）：

| # | world/cam | 像素 (x,y) | 命中 | 产品字节 → 参考字节 | 差异 |
| --- | --- | --- | --- | --- | --- |
| 1 | 0/1 | 13268 (148,41) | face 2 | [255,28,**29**] → [255,28,**28**] | B +1 |
| 2 | 0/1 | 31154 (114,97) | face 2 | [255,18,**19**] → [255,18,**18**] | B +1 |
| 3 | 1/1 | 27360 (160,85) | face 2 | [126,46,**92**] → [126,46,**93**] | B −1 |
| 4 | 1/1 | 28614 (134,89) | face 2 | [255,**48**,48] → [255,**47**,48] | G +1 |
| 5 | 1/1 | 39492 (132,123) | face 2 | [120,**42**,87] → [120,**41**,87] | G +1 |

机制链：

1. **重放校验**：5 个像素全部命中 **face 2**；`depth` 位型 **2/5 与 dump 完全一致、3/5 差 1 ULP**（重放脚本自身的收缩序与编译产物不完全相同，属重放精度；产品与参考的 depth 位型本身全等，见 dump `depth.ulp=0`）⇒ 射线/几何/求交/bary 重建可信，**求交侧排除**。
2. **顶点法线取舍**：`vertex_normal(n_i, face)` 判据 `dot < 0.8` 时回退到面法线。face 2 的三个顶点法线 dot = **0.4664 / 1.0000 / 0.5726** ⇒ **2 个被拒（回退面法线）、1 个保留**，因此最终着色法线主要由 `face = transpose(mat) @ normal` 决定。
3. **差异放大**：我方多归一化一次使 pre-normal 差 **1 ULP**（`[0,1,1]`）；经 `transpose(mat) @ normal` 后 face 各分量 ULP 差可达 `[1,0,43]` / `[1,9,46]`（小分量 ULP 数大但绝对量 ~1e-7）；最终着色法线差 **1 ULP**（`[0,0,1]` / `[1,0,0]`）。
4. **翻字节**：法线 1 ULP 差经光照（`mat_shininess=0.6 → pow(ndoth, 76.8)` 高光放大）后，在 **rgb 量化边界**附近翻 1 字节。这同时解释：
   - **只 mesh**：其它 geom 走各自解析法线，只归一化一次；
   - **只 cam1**：cam0 视野内只有 floor+背景（无 mesh）；
   - **G 通道也翻**（#4/#5）：`grid` checker 的 G 恒为 0.15 ⇒ G 只依赖光照 ⇒ 法线差异即可解释（无需假设纹理差异）；
   - **depth/seg 全等**：法线不参与求交/深度；
   - **legacy 全绿**：既有口径为 rgb ≤1 LSB；spec v3 判据 A 写死 **0 字节**（§13 风险 #10 已预警）。
5. 量级：face-2 像素 16,515/world 的最终法线均差 ~1 ULP，按边界概率 × 高光放大，期望每 world 约 1–4 次翻转，实测 2 / 3 / 0，量级吻合。

## 5. H1 否证（不是编译链 12.8.1 vs 12.9）

- 差异可**逐行指出**（我方多一次 `normalize3_mesh`），属源码结构不同，而非"同式不同收缩"。
- 我方 `dot3_ba`/`mat_vec`/`mat_t_vec`/`normalize3_rays` 的收缩序来自既有实证明（U4 sphere depth RCA、S5）；重放用同一套 helper 能把 product↔reference 的 depth 复现到 ≤1 ULP，说明这些 helper 不是站点。
- 若为纯编译链差异，无法解释"差异只出现在 mesh 这一条链、且法线位型在 5/5 像素上都出现 1 ULP 差"。

## 6. 修复方向与影响面（修复另卡，本卡不做）

- 一行级：让 mesh 路径与上游同构——`render_math.cu:239` 的 `*out_normal` 改回**未归一化**的三角面 `cross`，保留 `render_common.cu:200` 在 `mat @ cross` 之后的一次归一化。culling 只用 `dot` 符号，不受影响；shadow 的 `hit + normal*1e-4` 会随之更贴近上游。
- 影响面：改 `.cu` ⇒ `MEGAKERNEL_SOURCE`/PTX 变 ⇒ **内核身份变** ⇒ 需重冻 12 个 exe + 新 PTX 基线 + 全窗重跑（1 片 + 1 窗）；**判据/参考/spec/签名材料不变，不需要用户签名**。
- 建议顺序：先诊断轮（打补丁重建实体 exe、非 G-L2 证据、不重冻）确认 5 px 归零，再正式重冻 + attempt3。
- 若修复后仍余个位数 1 字节差：按同一方法（depth 位等校验 + 法线位差定位）继续收敛其它归一化/收缩站点。

## 7. 潜在风险登记

- 我方 woop 求交**未实现**上游 `U==0 || V==0 || W==0` 时的 **double 精度重算 U/V/W** 分支（`render_math.cu:100-108` 注释已登记；`intersect.h` 中 `intersect_ray_tri_woop` 有该分支）。本次被"产品与参考 depth 位型逐像素全等"排除（该分支会改 `t`），但仍属**潜在对齐缺口**，未来若出现"深度一致但法线/uv 差"以外的形态需优先复查。
- 本卡只覆盖判据 A 的 5 个 rgb 失配；判据 B / 两跑 sha / world 隔离在本轮因断言中止**未评估**，本卡不作任何相关结论。

## 8. 非声明

- 不宣称 G-L2 通过、不宣称判据 A/B/C 满足、不宣称与上游实现等价、不宣称任何渲染数值结论；本卡只是只读 RCA 记录。
- 不改变任何判据、阈值、参考、spec、冻结件与窗口状态；证据内的"修复方向"仅为登记，不构成实施。
- 上游其余相关头文件行为（`wp.normalize`/`dot` 的具体展开）以本仓库 `include/warp-headers` 与 pinned 源码为准；本卡不复制上游生成代码（warp 生成 `.cu` 缓存已在 `target/` 瘦身中丢失，未在本卡重建）。
