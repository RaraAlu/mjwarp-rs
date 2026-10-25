# G10@P5 multicontact 设备版 + primitive box-box · QA 独立复核

- 任务：`2026-10-25-g10-p5-multicontact-cuda-independent-check`（p52，QA）。
- 被复核片：`2026-10-25-g10-p5-multicontact-cuda`（Backend），产品提交 **`9a282074`**。
- 复核 HEAD：**`4c7be3c6`**（= `9a282074` + G6 map 的 docs/evidence/.normify 更新；窗口内硬守卫
  `src/tests/Cargo.toml/Cargo.lock/fixtures` 在 `9a282074..HEAD` 与工作树**均为空**）。
- 上游：pinned `71da24d956378a87a703b6e1442b13aec0c4ac29`；冻结源
  `docs/evidence/d04-complex-physics/authoring-runs/run-5/frozen-source/_src/`
  （`collision_gjk.py`、`collision_primitive_core.py`、`collision_primitive.py`、`collision_convex.py`）。
- 参考：QA 已提交的上游 f32 GPU 捕获
  `docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-upstream-gpu.json`
  （3 case 均 `two_captures_raw_bitwise_identical=true`；`nccd` 未捕获 = null）。

## 1. 结论

**pass（阻断 0 / 观察 6）**，`normify gate` + `normify check` 全绿。

- **3 case 逐字节**（QA 自建 dumper 落盘 + 自写比较器，未用交付 `bit_equal`）：
  `cylinder-box` **314/314**（8 接触，4/world）、`box-box-margin` **391/392**（唯一非位级 = pos 1 ULP，
  见 §5）、`mesh-mesh` **80/80**；分母 314/392/80 独立复算一致；`overflow` 全 0、`efc_address` 全 -1。
- **严格位级规则**（Arch 窗口条件 5）：除 box-box-margin 的 1 个 pos 1 ULP 外**无任何非位级项**；
  该 1 ULP 项 `abs_diff=7.4506e-9`、`scale=0.1170181334`、`bound=4·ULP(scale)=2.9802e-8`，
  **占比 0.25**，在 4·ULP(scale) 内 → 不构成阻断。
- **launcher 回归（21 段拼装无回归）**：`hfield-sphere` **314/314**、`sdf-box` **3122/3122**、
  `ellipsoid-box` **80/80**（对 G5 `out1` 逐字节）。
- **入口/拼装/PTX/ABI**：21 段拼装独立复现 **len=197428 / sha256=`682787c9…b9790`**；ctypes 直调
  NVRTC **3,556,510 B / 6 入口**；`abi.rs` 哈希与前两片相同（ABI 未改）。
- **内容级绑定**（Arch 窗口条件 2，Arch 2026-10-25 裁定 **accept-1**）：交付 `artifacts-sha256.txt`
  **21/21 与工作树一致**，其中 **20/21 与 `git show 9a282074:<path>` blob sha256 严格相同**；
  唯一差异 `window-output.txt` 为 **EOL-only**（工作树 CRLF 44 处由交付脚本 `WriteAllLines` 写出，
  git blob LF；`core.autocrlf=true` 的 clean 归一化后与提交 blob **逐字节相同**，
  `git hash-object` = `git rev-parse 9a282074:<path>` = `a77443a3…`）→ **内容级绑定成立、不重跑**，
  详见 §7。
- 边界：3 个冻结 case（8+10+2 接触）；mesh multicontact 未实现（需 mesh polygon ABI）；
  primitive 仍 10 组合未实现；未接入默认产品管线；未作精度达标/等价/放行类结论。

## 2. 复核方法（不复用交付比较器/汇总）

1. **自己的 dumper**：`dumper/`（独立 crate，只依赖 `mjwarp-rs` 公开 API + `cuda-probe`）用产品
   入口 `CollisionGpuProgram::new(0)` + `run(CollisionKernel::{ContactMulti,BoxBox,Contact}, ..)`
   跑 3 个 case，落 18 个原始数组 + `array-lengths.txt`；每 case **独立进程**（失败即停）。
2. **自己的比较器**：`qa_multicontact_check.py`（不 import 交付 `tests/support/*`）：
   - 排序键 **`(worldid, geomcollisionid)`**（同 world 内接触序号；`cylinder-box` 全部 dist 相同，
     不能用 dist 排序）；
   - 结构/计数/参数/整数零容差；`dist` 位级优先否则 ≤4 ULP（`ref==0` 用 4 ULP@1.0）；
     `pos` 按行内 `scale=max_i|ref_i|` 的 `4·ULP(scale)`；`frame` 按 scale=1.0 的 4 ULP；
   - **严格位级规则**：仅 box-box-margin 允许 1 个 pos 1 ULP，其它任何非位级项 → 失败；
   - `nccd`：参考未捕获（null）→ 只记录、不判失败（与 G5 同口径）；
   - 输入 23 项 bin 的 sha256/count 现场重算，与参考 `input_bins` 比对。
3. **拼装/PTX 独立复算**：`qa_assemble_check.py --rev 9a282074` 自己解析 `source.rs` 的
   `DEVICE_SOURCES`、读 21 个 `.cu`、删本地 `#include "*.cu"`、追加 `product.cu` 算 len/sha256；
   ctypes 直调 `nvrtc64_120_0.dll`（`--fmad=true --std=c++17 --gpu-architecture=sm_89`）得 PTX。
4. **反向判别力自检**：`qa_multicontact_check.py --self-test`（12 场景，合成 dump，不跑 GPU）。
5. **内容级绑定**：`qa_delivery_content_binding.py --rev 9a282074`（交付清单 vs git blob/worktree）。

## 3. 语义逐行对照（设备 vs pinned 上游，未发现偏差）

| 设备文件（行） | 上游 | 对照要点 |
| --- | --- | --- |
| `multicontact.cu`（189） | `collision_gjk.py::multicontact` L2216-2451 | `witness1[0]=x1/witness2[0]=x2`；`_feature_dim` 两次；box/cylinder face normals；`_aligned_faces`→`_aligned_face_edge` 两级匹配（geom1-edge 时 `nface1<3&&nface1<=nface2`，geom2-edge 时反向、dir 取反）；`i=res[0],j=res[1]`；`_set_edge(face[0],i,0/1)`；`idx1[j]/idx1[i]` 的 where 语义；三处 `_polygon_clip` 调用（geom1-edge 时 **交换 clipped1/clipped2**，设备用 `swapped` 还原）；`nclip<1` 回退；**mesh 分支整体省略 → 遇 MESH 回退 n=1**（交付已声明未覆盖） |
| `multicontact_clip.cu`（257） | `_area4`/`_polygon_quad`/`_plane_normal`/`_halfspace`/`_plane_intersect`/`_witness_on_face`/`_polygon_clip` L1487-2195 | `0.5*length(cross(a-c,b-d))`；quad 嵌套 while 提升 + `b==a` 修正（不更新 res）；`v1+n`、`cross(v2-v1, v3-v1)`、`dot(res,v1)`；`> -1e-10`；`FLOAT_MAX=1e30`、`|dot|<1e-10` 平行；`w1=v-dir*|dist|, w2=v`；逐边裁剪、`t∈(-3e-7,1+3e-7)`、clamp、两缓冲指针交换、负面裁剪、`nface2==2` 去重、`>4` 取 quad、witness 输出 |
| `multicontact_geom.cu`（361） | `_feature_dim`/`_aligned_*`/`_box_*`/`_cylinder_*`/`_set_edge` L1533-2026, L2199 | 逐行一致：`FACE_TOL=cos(0.0889)=0.996051f(0x3F7EFD33)`、`EDGE_TOL=sin(0.0889)=0.0887829438f(0x3DB5D3D5)`；dim1/2/3 的 index 编码与 `sgn==-1` 修正；`_box_normals2` 用 `transpose(mat)@n` 归一；6 面顶点顺序逐项；cylinder 16 边形 cos/sin 表（16+16 项 f32 位型另核）；`col0*cos-col1*sin+center`；第 3 列 `mat[:,2]` |
| `boxbox.cu`（474） | `collision_primitive_core.py::box_box` L595-1049 + `_compute_rotmore` L563 | 8 槽 `MJ_MAXVAL` 初值；pos21/pos12/rot21/rot12/abs/plen；面法线轴 `i+3*(p<0)+0/+6`；9 边叉积轴（`rot21abs[i,3-k-j]/cross_length`、`cle1/cle2` 位编码、`separation*(1-1e-12)`）；面-顶点路径（rotmore、clcorner、lp、cn1/cn2、`kk=dirs²` 交点、dirs==2 网格点、`points[2]>-lx…` 筛选、`depth=z; z*=0.5`、`rw=(box2?rot2:rot1)@rotmoreᵀ`、`normal=∓rw[:,2]`）；边-边路径（ax/pax 选择、`rotmore(cle1&(1<<pax2)?pax2:pax2+3)`、4 候选点、`rnorm[:,2]` 平行早退、`innorm`、投影、线段求交、quad 角点、`nl==0&&nf!=0`、`c1` 平方和、`depth`、`normal=(±1·rw)@rnorm`）；输出 `z+=hz; pos=rw@points+pw`。mat3×mat3 用 `__fmaf_rn` k 升序（同上游 `mat.h::mul`） |
| `boxbox_wrapper.cu`（20） | `collision_primitive.py::box_box_wrapper` L1250 | margin 透传、8 次 `write_contact(id_=i)`、每槽独立 `make_frame`、未填充槽 dist=MAXVAL 由 `detected` 门过滤 |
| `contact.cu` multicontact 分支（+72 行） | `collision_convex.py::eval_ccd_write_contact` L769-1026 | `want_multi = use_multiccd or (BOX,BOX)` + 双侧 {BOX,MESH,CYLINDER} 门；`multiccd_idx>-1` 才做多接触；`dists[i] if n>1 else dist`；**frame 只算一次**（用 witness[0]，`dist<=margin` 时 `w1-w2` 否则 `w2-w1`）；`0.5*(w1[i]+w2[i])+origin`；`id_=i`；n<1 回退（设备内联为 n=1 + witness[0]=x1/x2）；`ccd.cu` 的 `margin!=0 → multiccd_idx=-1` 与上游 L2669-2671 一致 |

## 4. 入口/拼装/PTX/ABI（CPU 独立复算）

- 入口真实：`new` 走 `cudarc::nvrtc::compile_ptx_with_opts` → `load_module` + **6** 个 `load_function`；
  `run` 真 launch/回读；`cuda-probe` 门控（默认构建无 CUDA 入口）。
- 拼装：**21 段 / len=197428 / sha256=`682787c964e216a55ef1be479024f799e4215ba9457519b0f5892de74bab9790`**；
  顺序 `common→…→hfield→multicontact_geom→multicontact_clip→multicontact→contact→primitive→
  primitive_wrapper→boxbox→boxbox_wrapper→product.cu`。
- PTX：**3,556,510 B / 6 入口**（contact/hfield/sdf/primitive/**contact_multi**/**boxbox**）。
- ABI：`abi.rs` sha256 `b27c8182…` 与 NVRTC/primitive 片相同 → **未改**。
- 交付 `artifacts-sha256.txt` **21/21 与工作树逐项一致**（含 5 个新 `.cu`、contact/product/source/
  collision_gpu/nvrtc_smoke/tests 等）。

## 5. 判据与窗口结果

窗口 HEAD `4c7be3c6`，只跑窗口外预构建的 QA dumper；每 case 独立进程 + 失败即停（本次 6/6 全过）。

| case | 内核 | nacon | 每 world 接触 | nccd（参考 null） | 逐字节 | 独立分母 | max dist/pos/frame ULP |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `cylinder-box` | ContactMulti | 8 | 4（gcid 0..3） | 2（只记录） | **314/314** | 314 | 0 / 0 / 0 |
| `box-box-margin` | BoxBox | 10 | 5（gcid 0..4） | 0（只记录） | **391/392** | 392 | 0 / **1** / 0 |
| `mesh-mesh` | Contact | 2 | 1 | 2（只记录） | **80/80** | 80 | 0 / 0 / 0 |
| `hfield-sphere`（回归） | Hfield | 8 | — | 0 | **314/314** | 314 | 0 / 0 / 0 |
| `sdf-box`（回归） | Sdf | 80 | — | 0 | **3122/3122** | 3122 | 0 / 0 / 0 |
| `ellipsoid-box`（回归） | Contact | 2 | — | 2（参考 null） | **80/80** | 80 | 0 / 0 / 0 |

- **唯一非位级项（box-box-margin）**：`pos` 行 7 分量 0，device `0.09958209842443466` vs
  ref `0.09958210587501526`，`ulp=1`，`abs_diff=7.450580596923828e-9`，行内
  `scale=0.11701813340187073`，`bound=4·ULP(scale)=2.9802322387695312e-8`，**abs_diff/bound=0.25**
  → 在 4·ULP(scale) 内。除此之外 3 case 与 3 回归 case **零非位级项**（严格规则通过）。
- **结构特征**：`cylinder-box` 每 world 4 接触、dist 全 -0.06、**frame 每 world 只有 1 种**（共享帧）；
  `box-box-margin` 每 world 5 接触、gcid 0..4、dist 各异、**frame 每 world 1 种**（box_box 单法线）；
  `mesh-mesh` 每 world 1 接触。与上游 multicontact/box_box 语义一致（共享 frame、`id_=i` 进
  `geomcollisionid`）。
- 输入守卫：3 case × 23 项 bin 的 sha256/count 与参考 `input_bins` **69/69** 相同、与交付 tracked
  `gpu-window/cases` **69/69** 相同。

## 6. GPU 窗口事实

- 窗口：2026-10-25 23:50:15.149 → 23:52:11.675（**elapsed 115.7 s**），0.25 s 采样 **407 点**。
- HEAD **`4c7be3c65c1e456ca87b0b1f64fc85664e8cb959`**；`product_head=9a282074`；
  `git diff 9a282074..HEAD` = 8 个 docs/evidence/.normify 文件（G6 map 更新），
  **产品路径 diff 为空**（`product_diff_empty=true`）、产品工作树改动 0。
- 基线：used **3641 MiB / 16376**、free 12421、util 2%、30°C、driver 596.36；`query-compute-apps`
  **33 条全桌面图形上下文**、本任务相关 **0**。
- 峰值：采样峰值 **5069 MiB = +1428 MiB**（分布：3637×5、3641×398、3825×1、4691×1、5047×1、
  5069×1）；**本次 0.25 s 采样捕到了 CUDA context 创建瞬时**（与 NVRTC 片 run1 的 +1325 同量级）；
  软阈值 +1536 **未超**（1428 < 1536）、硬守卫 3072/8188 未触发；**free 最低 10993 MiB**（>10240
  停跑阈值，未触发）；util max 6%、temp max 31°C；结束 used **3641**；跑后
  `query-compute-apps` 33 条（全桌面）、本任务相关 **0**、dumper 残留 **0**。
- 峰值口径：**含 context 瞬时**；0.25 s 采样仍可能漏瞬时（本次捕到不代表每次都能捕到），
  峰值只作资源观测，不作设备占用结论；功能/数值结论不受影响。
- 工具链：`PATH` 前置 `target/toolchains/cuda-12.8.1/bin`；nvcc `Build cuda_12.8.r12.8/
  compiler.35583870_0`；NVRTC DLL sha 同前（`02d2d7ef…`/`5464ea01…`/`a7afa2a4…`）。
- dumper（窗口外预构建，窗口内未编译）：`target\qa-multicontact\dumper-target\debug\
  qa-multicontact-dumper.exe`，sha256 `28287daf847cfeaf716cb3e5252e50570eaf7455a0d833aef8dfe125d77a5cae`，
  构建 23:46:22。
- 逐 case wall：19.41 / 19.35 / 19.46 s（multicontact）、18.29 / 19.63 / 19.59 s（回归）；
  NVRTC 编译在 CPU 侧（21 段），GPU 满载极短。
- 桌面 top5（工作集）：vmware-vmx 17726 MiB、DeepSeek Harness 1708 MiB、Code 1117 MiB、
  MsMpEng 779 MiB、explorer 668 MiB。

## 7. 内容级绑定核（Arch 2026-10-25 裁定：accept-1）

- 交付清单 21 项：**21/21 与当前工作树一致**；**20/21 与 `git show 9a282074:<path>` 的 blob
  sha256 严格相同**（含全部源码/测试/清单/任务记录/窗口脚本）。
- 唯一差异 `docs/development-evidence/g10-p5-multicontact-cuda/window-output.txt` 的证据链：
  | 证据 | 值 |
  | --- | --- |
  | `git check-attr -a -- <path>` | **空**（该路径无 `.gitattributes` 规则，文本判定回落 `core.autocrlf`；仓库 `.gitattributes` 只有 fixtures/evidence 等特定 `eol=lf` 规则） |
  | `git config --get core.autocrlf` | `true`（提交时 CRLF→LF 归一化） |
  | 工作树 / blob 行尾 | CRLF **44** 处 / LF **0** 处 |
  | 内容 sha256 | 工作树 `9dc451727d0a…`（CRLF）/ blob `5f3b34669203…`（LF） |
  | `CRLF→LF` 归一化后 | 与 blob **逐字节相同**（`worktree_normalized_equals_blob=true`） |
  | `git hash-object --no-filters <path>` | `b6586c71…`（原始 CRLF 字节） |
  | `git hash-object <path>`（clean 归一化） | **`a77443a340f65952a4b4718f8222e163d0cf9ecd`** |
  | `git rev-parse 9a282074:<path>` | **`a77443a340f65952a4b4718f8222e163d0cf9ecd`（相同）** |
- 结论：该差异是**仓库 EOL 归一化行为**，不是内容差异；提交该文件后 blob 内容 = 当前工作树
  归一化结果（hash-object 与 rev-parse 相同），**入库不改内容**。
- Arch 裁定：**accept-1（内容级绑定成立、不重跑）**；依据：源码/测试/清单/任务记录 20 项严格
  blob 相同 + `artifacts-sha256.txt` 21/21 与工作树一致（清单口径自洽）；该文件属窗口输出日志，
  非源码/测试/清单。证据落到 `qa-delivery-content-binding.json/log` 的 `eol_analysis` 与
  `arch_ruling` 字段。

## 8. 严格位级规则与反向判别力

- **严格规则**（Arch 条件 5）：仅 box-box-margin 允许 1 个 pos 1 ULP，其它任何非位级项即失败。
  窗口实测：该允许项恰好 1 个、ulp=1；其余全部位级 → 通过。
- **合成 dump 自检（12 场景，不跑 GPU，非真实设备数据）**：基线 392/392；`pos+1 ULP` → 391/392
  且严格规则放行（唯一允许）；`dist+1 ULP` → 严格规则**阻断**；`pos+1e-3`、`frame+1e-6` → 失败；
  **`single_contact_fallback`（只留 gcid=0 两行）→ 29 failures**；`drop_one_contact` → 失败；
  `swap/permute rows` → 排序后 392/392；`nccd=999` → **记录不判失败**；overflow/efc 异常 → 失败。
- 产品侧 CPU 单测：`cargo test --locked --features cuda-probe --lib runtime::collision_gpu --
  --include-ignored` → **20 passed / 0 failed**（12 + primitive 4 + multicontact 3 + NVRTC 冒烟 1）。

## 9. 覆盖与归属口径

- **`box_box` 归属**：属 primitive family（`_PRIMITIVE_COLLISIONS` 的 `(BOX,BOX)`；本项目 fixture
  全 `nativeccd=disable` → `_narrowphase` L909-912 把它覆盖为 PRIMITIVE），为覆盖 `box-box-margin`
  一并移植，**不属 multicontact 分支**——交付表述正确，且与 primitive 片发现 1 的口径衔接。
- primitive 实现 3 组合（sphere-box/capsule-box/box-box），**13 组合中未实现 10 个**（README §7.1
  列表一致）。
- **mesh multicontact 未实现**（需 mesh polygon ABI 输入）：交付如实列为未覆盖，`mesh-mesh` 只按
  单接触路径（其 fixture 未启用 multiccd，与上游单接触 dispatch 一致），未写成通过。
- 未做性能测试（本片无性能判据）。

## 10. 观察项（均非阻断）

1. **内容级绑定 20/21 exact + 1 项 EOL-only**（见 §7）：**Arch 2026-10-25 裁定 accept-1
   （EOL-only，CRLF↔LF 44 行尾，clean/autocrlf 归一化后与提交 blob 逐字节相同）→ 接受、不重跑**。
2. **交付 README §1 第 4 行残留「GPU 窗口对照待双签后执行…跑完回填」**，与 §5 已执行矛盾
   （仅文案，**语义以 §5 为准**）。该文件属已 verified 交付任务 `2026-10-25-g10-p5-multicontact-cuda`
   的证据，重开任务代价大于收益 → **本复核未改动交付方已验收证据**，留待后续统一文档批次清理。
3. **流程改进（归档、不阻断）**：交付窗口 HEAD 绑的是产品父提交 `9d337bfe`，未绑最终产品提交；
   本次用内容级绑定（20/21 严格 + 1 EOL-only）闭合。**以后窗口日志须同时记「产品 diff 范围 +
   最终产品提交号」**，不再依赖事后内容级绑定。
4. **nccd 参考为 null**：`cylinder-box`/`mesh-mesh` 设备 nccd=2 只记录、不参与判定（上游 `nccd`
   未捕获；与 G5 同口径）。若后续要判 nccd，需要重新 authoring 捕获。
5. **峰值采样**：本次捕到 +1428 MiB context 瞬时（407 点/0.25 s），也说明上片交付 +197 的稀疏采样
   不代表无瞬时；该类瞬时属工具链/上下文开销，不作设备占用结论。
6. **`contact.cu` 的 `want_multi` 是运行时布尔组合**（`use_multiccd || (BOX,BOX)`），上游是
   kernel 构建期静态特化；两者在当前 dispatch 下等价（产品按入口选择），仅记录实现差异。

## 11. 边界与未覆盖

- 仅 3 个冻结 case（8+10+2 个接触点）+ 3 个 launcher 回归 case；不扩展结论到其它姿态/组合。
- 未覆盖：mesh multicontact（需 mesh polygon ABI）；primitive 其余 10 组合；无性能测试；
  产品入口仍经 `cuda-probe`、未接入默认产品管线；环境仅 Windows/RTX 4070 Ti SUPER 16 GiB/NVRTC 12.8。
- 未改判据/已 verified 数字/产品代码/`Cargo.*`/fixtures；交付证据未被本复核修改。
- 不宣称精度达标、上游完全等价、原生放行或双平台通过。

## 12. 可复现路径与命令

- dumper：`dumper/`（`cargo build --manifest-path dumper/Cargo.toml --offline`；
  `CARGO_TARGET_DIR=target/qa-multicontact/dumper-target`；窗口内不编译）。
- 比较器：`qa_multicontact_check.py`（3 case vs 上游参考，含 `--self-test`）、
  `qa_launcher_regression_check.py`（launcher 回归 vs G5 `out1`）、
  `qa_assemble_check.py --rev <commit>`（21 段拼装/PTX）、
  `qa_delivery_content_binding.py --rev <commit>`（交付清单 vs git blob）、
  `qa_artifacts_sha256.py`（本目录清单）。
- 窗口：`run_window.ps1`（HEAD/产品 diff 守卫 + 0.25 s 采样 + 3072/50% + free<10240/计算进程即停 +
  per-case 早停）。
- 原始输出：`target/qa-multicontact/prod-out/<case>/`（不入库，可由 dumper 重跑重建）；
  诊断中间物 `target/qa-multicontact/{assemble-check.json,selfcheck.json}`（不入库）。
