# G10@P5 M3：mesh-mesh multicontact 设备实现与验证（证据）

任务：`2026-10-26-mesh-multicontact-m3-mesh-mesh`（owner Backend，p41；依赖 M2 `2026-10-26-mesh-multicontact-m2-convex-vs-mesh`）。
上游冻结：`71da24d956378a87a703b6e1442b13aec0c4ac29`（pinned mujoco 3.15.0 / warp 1.15.0 / mujoco_warp 3.15.0）。
状态：**实现 + 捕获参考 + bins + CPU 校验/测试 + 设备验证窗 attempt1（有效轮，9/9 exit 0，mesh-mesh bit_equal 4/4）完成**；待 Arch/Chief 核对 → 收口 gate/done/commit → QA 复核。

本文件是证据层：只记录已跑出的事实与严格限定，不把「数值对照」提前写成结论。

---

## 1. 结论摘要

1. **分派链**：pinned `(MESH, MESH) → CONVEX`（`collision_driver.py:80`），走与 M2 完全相同的
   `convex_narrowphase → ccd_kernel_builder` MULTICCD multicontact 分支（`collision_convex.py:906-944`，
   mesh 需 `mesh_polyadr>=0`，`:911-919`），最终调用 `collision_gjk.py` 的 `multicontact()`。
2. **设备实现零新增**：M2 的 `g5_multicontact` 已按 pinned 通用语义实现两侧 MESH 分支
   （`multicontact.cu:88-102/117-149/169-191`）；pinned 的 9 个 mesh 数组是**模型全局数组**
   （`collision_core.py:120-141` 两侧赋同一全局数组），逐 geom 只差 `mesh_polyadr`/`vertadr`
   偏移，本实现两侧各自使用所属 geom 的偏移（`multicontact.cu:59-65` 取视图、每个 helper
   调用传对应 geom 的 `vertadr`/`mesh_polyadr`）。故 M3 **设备源 0 行改动**：
   `DEVICE_SOURCES` 28→28、入口 6→6、设备源指纹与 M2 相同。
   **该「0 行」结论已由验证窗 attempt1 的 bit_equal 4/4 证实**（见 §9）。
3. **新参考已捕获**：mesh-mesh 上游 pinned GPU 参考 `upstream-mesh-multicontact-mesh-mesh-gpu.json`
   （sha `96e7895d…`，count=4，两次进程内捕获逐位一致），旧 M1 参考零改动。
4. **32 数组 bins 已产出**：`cases/mesh-mesh/`（同 fixture，`xml_sha256`/`qpos_bits`/
   `geom_xpos`/`geom_xmat`/`collision_count` 与参考逐位一致；18 条 CPU 自检 PASS）。
5. **O7 值域校验已补**（仅 CPU 测试入口）：`polyvert`/`polymap` 元素值域 + 逐 mesh
   `vertadr/vertnum` 前提 + mesh geom 的 `dataid` 范围；`run()` 零改动。
6. **验证窗 attempt1 = 有效轮**：9/9 实体 exit 0（独立采集）、guard 无 abort（free_min 13557 >
   floor 10240）、零 Rust 编译；mesh-mesh `bit_equal 4/4`、dist/pos/frame ULP 全 0、
   `contact.geom=[0,1]`；13 组合 + B3 6 分支 + 4 swap + M2 两 case 位级/判据内无回退（§9）。

## 2. pinned 落点与分派链（行号按实测）

| 位置 | 内容 |
| --- | --- |
| `collision_driver.py:80` | `(GeomType.MESH, GeomType.MESH): CollisionType.CONVEX`（MJ_COLLISION_TABLE） |
| `collision_convex.py:906-944` | `convex_narrowphase` 的 ccd_kernel_builder MULTICCD 分支；`:911-919` mesh 需 `mesh_polyadr>=0` |
| `collision_gjk.py:2216` / `:2245-2262` | `multicontact()` 入口；mesh 数组视图选择（`if geomtype1==MESH … elif geomtype2==MESH`） |
| `collision_gjk.py:2254` | `dir`/`dir_neg` 与后续法线/边/面恢复的共享上下文 |
| `collision_gjk.py:2278-2310` | `_mesh_normals` 两侧调用（逐 geom `vertadr`/`mesh_polyadr`） |
| `collision_gjk.py:2326-2379` | `_mesh_edge_normals` 两侧调用（edge-face） |
| `collision_gjk.py:2400-2434` | `_mesh_face` 两侧调用（面恢复） |
| `collision_core.py:120-141` | Geom 的 mesh 字段赋值：`mesh_polynormal` 等**均为模型全局数组** |
| `collision_gjk.py:1592/1606/1621/1691/1927` | `_intersect1` / `_intersect2` / `_mesh_normals` / `_mesh_edge_normals` / `_mesh_face` |
| 本实现 | `contact.cu:148-150`（两侧类型 {BOX,MESH,CYLINDER}）、`contact.cu:203-241`（multiccd 分支）、`product.cu:155-163`（按 dataid 解析 per-geom `vertadr`/`mesh_polyadr`）、`multicontact_mesh.cu`（M2 的 mesh 助手，本片复用） |

M3 与 M2 的唯一差别是两侧同时为 MESH；pinned 在该情形下取「geom1 的数组视图」是安全的，
因为数组全局、偏移逐 geom（若两侧不同 mesh 资产，`mesh_polyadr`/`vertadr` 不同但数组相同）。

## 3. Fixture（新）

`fixtures/g10-p5-mesh-multicontact/mesh-mesh.xml`（sha256
`bb9ffb2f979129bbac410e9959088528f35180d14f5572c9a52631fa2ed86d8d`，617 B）

- mesh A：`cube.obj`（8 顶点 / 6 polygon），静态 geom `mesh_a_geom`（dataid 0）；
- mesh B：同文件 `scale="0.9 0.9 0.9"` 的**独立 mesh asset**（dataid 1），挂在带 freejoint 的
  `mesh_b_body`（z=0.18）上，geom `mesh_b_geom`；
- `nativeccd="disable"`、**不禁** MULTICCD、`gravity=0`、确定性 `<key name="capture">`；
- 几何自洽：cube 半长 0.1、B 半长 0.09、中心距 0.18 → 穿透 0.01、面-面 4 接触
  （参考 dist 全 `-0.009999997913837433`）；
- 覆盖「全局 sidecar 数组 + 逐 geom 偏移」：`nmesh=2 / nmeshvert=16 / nmeshpoly=12 /
  nmeshpolyvert=48 / nmeshpolymap=48`，`dataid=[0,1]`、`vertadr=[0,8]`、`polyadr=[0,6]`。

### 3.1 CPU 预检（authoring-only，非参考/非结论）

`precheck-cpu.json`（本目录根下，不在 `scripts/`；pinned venv、`wp.set_device("cpu")`、无 CUDA context）：

- `(MESH, MESH)` 两侧均带 polygon data；`nmesh=2`、`nmeshpoly=12`、`nmeshvert=16`；
- MULTICCD **on** → `nacon=4`（`geomcollisionid=[0,1,2,3]`、`dist` 全 `-0.0099999979`）；
  MULTICCD **off** → `nacon=1` → 证明该 fixture 确实触发 multicontact 路径；
- 结果 `PASS`（`precheck_mesh_mesh_cpu.py`，sha 见 §12）。

## 4. 追加捕获窗 attempt1（首个有效轮）

窗口纪律：`free≥10240` open 门、`floor=min(10240, baseline_free−3072)`、0.25 s guard v2 采样、
RUN A/B 独立进程 + 独立 warp cache、raw 逐字节一致才复制为参考、pre/post/结束基线/无残留、
**零 Rust 编译**、机器负载前置（无 `cargo/rustc/cl/nvcc`、无 clean 重建/全量测试并行）。

- 窗口：`2026-10-27T00:48:22.577+08:00` → `00:48:49.334`（wall **26.8 s**）
- RUN A exit 0 / wall **16.2 s**；RUN B exit 0 / wall **6.1 s**；compare exit 0
- raw A/B sha 均 `96e7895d13c3ba23475a013fcaa586485e93f95cced5351e2874973917e149a1`（3676 B）
  → **RAW BYTE-IDENTICAL: PASS**，A 复制为参考 `upstream-mesh-multicontact-mesh-mesh-gpu.json`（同 sha）
- 参考内容：count=4、`geom` 全 `[0,1]`、`dist` 全 `-0.009999997913837433`、
  `geomcollisionid=[0,1,2,3]`、`qpos_bits=[0,0,1043878380,1065353216,0,0,0]`、
  `two_captures_bitwise_identical=true`、flags `nativeccd_disabled=true / multiccd=true`、
  `geom_dataid=[0,1]`、`captured_at=2026-10-27T00:48:22.577+08:00`
- 签名实体复核（开窗前 runner 打印 `SHA_OK`）：fixture `bb9ffb2f…`、capture `7e7c45f1…`；
  compare 零改动 `27ecb54a…`；runner `c5400cb5…`（不入库）

### 4.1 采样摘要（guard v2，87 samples）

| 项 | 值 |
| --- | --- |
| `used_peak` | 2278 MiB |
| `free_min` | 13784 MiB（floor 10240，未触发） |
| `util_max` | 80%（**streak=2**，见注 1） |
| `delta_peak` | 238 MiB（见注 2） |
| `residual` / `process_exited` | 0 / True |
| post guard | used 2007 / free 14055 / delta −33 / util 24，无 abort |

注 1：util 80% 出现在 warp JIT 编译的**连续 2 个**采样点，低于 guard 的 util abort 门
（80% × 连续 3 点），未 abort；属编译瞬态、非持续负载。
注 2：delta 三口径（同一 `used_peak=2278`）：
- vs `guard_baseline used=2040` → **238**（guard 聚合口径）
- vs 第一基线点 `used=2044` → **234**（3 点基线口径）
- vs 采样最小 `used=2032` → **246**（采样最小值口径）
三者均远低于增量门 3072。

### 4.2 窗口事实

- 基线 3 点：`2044,14018,36,32,596.36` / `2025,14037,37,32,596.36` / `2025,14037,37,32,596.36`
- post：`2007,14055,20,32,596.36`；结束基线 3 点：`2007,14055,20` / `2007,14055,19` / `2007,14055,23`（+32,596.36）
- post `compute-apps`：全桌面进程，按 `python|g10_p5|cargo|mjwarp` 过滤 = **0**；无 python/cargo 残留
- 机器负载：`cargo/rustc/cl/nvcc` 均无；CPU 负载采样 `3,9,2`；无 clean 重建、无全量测试并行
- 零 Rust 编译：窗内只跑 pinned python（capture A/B + compare）+ `nvidia-smi` + guard 采样
- 旧 M1 参考**未读写**：`c83baffe6de695fdefda6852c574d5d0d7312ebb591c1b487dfc59ec126c206a`（5703 B，跑前跑后一致）

## 5. mesh-mesh 32 数组 bins（同源）

生产脚本：`scripts/prepare_mesh_mesh_case.py`（CPU，`wp.set_device("cpu")`；参照 M0b 套路，
只针对新 fixture + 新参考；M0b 脚本零改动）。产物：`cases/mesh-mesh/`（32 个 `.bin` + `meta.txt`），
清单 `mesh-case-manifest.json`。

- 9 个 sidecar slice 长度（`nmesh=2 / nmeshvert=16 / nmeshpoly=12` 推导）：
  `[2, 2, 36, 12, 12, 48, 16, 16, 48]`
  （polynum/polyadr 逐 mesh、polynormal 36 f32=12 行 ×3、polyvert\* 逐 polygon、polymap\* 逐 vertex）
- 同源逐位对照（manifest `cross_check` 全 true）：`xml_sha256` == 现场重算 == 参考；
  `qpos_bits` == 参考；`geom_xpos`/`geom_xmat` 位型 == 参考；CPU collision `count=4` == 参考
- CPU 自检 18/18 PASS：M0a 12 条（9 数组齐全/长度三段/sum 恒等/区间/非负）+
  M3 4 条 `adr == cumsum(num)`（vertadr/polyadr/polyvertadr/polymapadr）+
  M3 2 条值域（`polyvert ∈ [0, vertnum[owner])`、`polymap ∈ [0, polynum[owner])`）
- `meta.txt` 由参考首行参数生成（g1=0/g2=1、type1=type2=7、condim/friction/solref/solimp 等）

## 6. O7 值域校验（M3 折入）

位置：`src/runtime/collision_gpu/abi.rs::validate_mesh_poly`（CPU 测试入口；`run()` 与
`validate_collision_inputs` 的调用关系不变）。规则（在 M0a 长度/区间/sum 规则之后）：

1. `mesh_vertadr.len() == mesh_vertnum.len() == nmesh`；逐 mesh `vertadr/vertnum` 非负且
   `vertadr+vertnum <= nmeshvert`；
2. `polyvert` 元素（逐 polygon，按所属 mesh）∈ `[0, mesh_vertnum[owner])`；
3. `polymap` 元素（逐 vertex，按所属 mesh）∈ `[0, mesh_polynum[owner])`；
4. sidecar 存在时，mesh geom（type 7）的 `geom_dataid` ∈ `[0, nmesh)`。

边界声明：sidecar 唯一生产者是 M0b/M3 同源 bins（脚本侧 §5 自检同样校验值域）；
`M0a` 未覆盖的 `polyvert/polymap` **元素值域**由此补齐；仍未覆盖的项：
未与既有 23 项输入交叉核对元素值域之外的语义、未覆盖非本生产侧来源的数据（见 §8）。

负例与正例（单测：M3 收口时按 500 行棘轮拆分到
`src/runtime/collision_gpu/validate_mesh_poly_tests.rs`，模块路径
`runtime::collision_gpu::validate_tests::mesh_poly_tests`；测试名不变，详见 §13）：
`rejects_mesh_poly_value_domain_violations` 覆盖 6 条负例（vertadr 长度/区间/负值、
polyvert 越界、polymap 越界、dataid 越界）+ 1 条真实 cube 正例；同模块另含
`accepts_real_cube_mesh_poly_shapes` / `rejects_inconsistent_mesh_poly_sidecar` /
`rejects_inconsistent_mesh_poly_sums`。`validate` 相关单测共 8 个，8/8 passed
（4 个在 `validate_tests` 父模块、4 个在 `mesh_poly_tests` 子模块）。

真实 bins 正例（常驻非 ignored CPU 测试；M3 拆分为
`tests/g10_p5_mesh_m3_case_assembly.rs::real_cpu_validation_accepts_m3_mesh_mesh_bins`，
旧路径 `tests/g10_p5_mesh_poly_case_assembly.rs::…` 见 §13）：
（M3 bins sidecar 9 项 present、长度 `[2,2,36,12,12,48,16,16,48]`、`geom_dataid=[0,1]`、
`validate_collision_inputs == Ok(())`）；同文件另加 `m3_same_source_matches_m3_reference`
（bins ↔ manifest ↔ 参考逐位同源）。拆分后两个 CPU 装配目标合计 5 + 2 = 7 条测试，全 passed
（测试名不变）。

## 7. M3 设备对照测试

`tests/g10_p5_mesh_multicontact_m3_cuda.rs`（ignored，需 GPU 窗口）：

- 读 M3 bins 与 M3 参考，先锁参考 sha256 `96e7895d…`（参考不可被改动）；
- 断言 `(MESH, MESH)` 调用方按 geom 序传 `g1=0/g2=1`、`type1=type2=7`，
  `contact.geom=[0,1]`（两侧同类型、pinned 类型序不交换；device 不做归一化）；
- 判据与 M2 完全一致（`tests/support/mesh_multicontact_compare_support.rs`，零放宽）：
  `nacon` 与参考 count 零容差、行按 `(worldid, geomcollisionid)` 结构键配对、位级优先、
  `dist` 4 ULP@自身量级（ref==0 → 4 ULP@1.0）、`pos` 4·ULP(参考三轴最大绝对值)、
  `frame` 4 ULP@1.0、结构/参数零容差（includemargin/adhesion 位级；friction/solref/
  solreffriction/solimp/worldid/type/dim/geom/geomcollisionid 零容差）、`overflow` 全 0、
  `efc_address` 参考未记录 → 设备侧全 `-1` 不变量。

### 7.1 验证窗 attempt1 结果（有效轮，详见 §9）

- `case=mesh-mesh nacon=4 ref_count=4 bit_equal=4/4 max_dist_ulp=0 max_pos_ulp=0 max_frame_ulp=0
  contact.geom=[0,1] failures=0`（测试 1 passed，22.31 s）
- 结论口径：**判据内零失败（failures=0）**，mesh-mesh 位级一致；不宣称一般 mesh 覆盖（§8）。

## 8. 边界与未覆盖（如实声明，不宣称）

1. **scratch 容量上界**（M2 继承）：mesh 助手固定 degree≤10、单 polygon 顶点≤16、clip 16/32；
   pinned 对 mesh 动态上界更大。超出本实现上界时 mesh 助手返回 0 → 单接触回退；
   当前 cube fixture（degree=3、polygon=4）远低于上界，未触发；不对一般 mesh 作覆盖声明。
2. **`contact.geom` 顺序**（M2 继承）：本片 mesh-mesh 两侧同类型，调用方按 geom 序传 `[0,1]`；
   device 不归一化。债务保留：multicontact 入口未做类型序归一化（B3 只在 primitive 入口做过）。
3. **UNC-6 类深穿透**：本片 fixture 穿透 0.01（**未触发**深穿透分支），设备深穿透行为未覆盖。
4. **hfield-capsule**（M2 继承）：ABI 504 后无设备回归证据，记未覆盖（补偿见 M2 证据 §7.3）。
5. **`efc_address` 缺口**（M2 继承）：上游捕获未记录 `efc_address`；设备侧断言全 `-1`，
   不构成对 efc 视图的等价声明。
6. **不宣称 mesh 全类型/全分支**：本片只覆盖 `(MESH, MESH)` 单 fixture 的 convex multicontact 路径。
7. **M3 设备「0 行改动」**：实现推断 + 设计依据已由验证窗 `bit_equal 4/4` 证实（§7.1/§9.1）；
   该证实仅限本 fixture/本 case，不延伸到一般 mesh。
8. **窗口留档的 runner 缺陷（记录，不改原始记录）**：验证窗 `facts.txt` 的
   `guard_aggregate=System.Collections.Specialized.OrderedDictionary` 一行是字典对象字符串化
   的无效行；采样事实以 `guard.log` + `samples.csv` 为准（§9.2 已重算）。本轮无需重跑；
   若该 runner 再用于后续 attempt，必须先修正该行（修 runner 会破签名 → 需重新报签）。

## 9. 验证窗 attempt1（首个有效轮）

签名集合（Arch 实测、Chief 双签）= 9 个冻结二进制 sha + runner sha
`81475a0580a13ddaad03e83a5189b2baf8136de27c3e2dc703c3d8e786ea7f89`（8397 B）；
冻结后未再跑任何 `cargo`（至本文件写成时的状态）。

### 9.1 逐实体 exit code 与结果（独立采集）

| target | exit | 结果（判据内） |
| --- | --- | --- |
| `g10_p5_mesh_multicontact_m3_cuda` | **0** | mesh-mesh：nacon=4、**bit_equal 4/4**、max dist/pos/frame ULP=0、`contact.geom=[0,1]`、failures=0 |
| `g10_p5_mesh_multicontact_m2_cuda` | **0** | mesh-box / mesh-cylinder：各 nacon=4、bit_equal 4/4、ULP=0、`[1,0]`、failures=0 |
| `g10_p5_multicontact_cuda` | **0** | cylinder-box 314/314 bit；box-box-margin 391/392（max_pos_ulp=1）；G5 mesh-mesh case（Contact 单接触路径）80/80；failures=0 |
| `stage2_equivalence_window` | **0** | 9 个 synth case dump 全完成；窗外 CPU 比较按「记录 C 只记录不判 pass/fail」，17 条 `failures=` 含已知 recipe 分岔（非 M3 回归） |
| `g10_p5_primitive_cuda` | **0** | sphere-box-shallow / -deep / capsule-box 各 80/80；failures=0 |
| `g10_p5_nvrtc_launcher` | **0** | hfield-sphere 314/314；sdf-box 3122/3122；ellipsoid-box 80/80；failures=0 |
| `g10_p5_primitive_plane_cuda` | **0** | 6 case：plane-sphere/capsule/ellipsoid/cylinder nacon=1，plane-box/mesh nacon=3；failures 全 0 |
| `g10_p5_primitive_sphere_capsule_cuda` | **0** | sphere-sphere / sphere-capsule / sphere-cylinder / capsule-capsule 各 nacon=1；failures 全 0 |
| `g10_p5_primitive_branches_cuda` | **0** | 6 分支 + 4 swap（含 plane-mesh-graph swap）；failures 全 0 |

9/9 exit 0；汇总 `failed_binaries=0`。

### 9.2 采样摘要（guard v2，0.25 s，732 samples）

| 项 | 值 |
| --- | --- |
| `used_peak` | 2505 MiB |
| `free_min` | **13557 MiB**（floor 10240，未触发） |
| `util_max` | 60%（80%×3 abort 门未接近；`util_streak=0`） |
| `residual` / `process_exited` | 0 / True |
| guard abort | 无 |

**floor 实际值**：`floor = min(10240, baseline_free − 3072)`，基线 free 13840 → **floor 10240**。
**delta 三口径**（同一 `used_peak=2505`）：vs guard baseline used 2227 → **278**；
vs 第一基线点 used 2222 → **283**；vs 采样最小 used 2222 → **283**；均远低于增量门 3072。

**权威数字来源**：窗口采样事实以 `logs/validation-attempt1-guard.log`（`guard_baseline` /
`guard_aggregate` 两行）与 `logs/validation-attempt1-samples.csv`（逐样本 used/free/util）为准；
`logs/validation-attempt1-facts.txt` 中 `guard_aggregate=System.Collections.Specialized.OrderedDictionary`
一行是 runner 把字典对象字符串化后的**无效行**（窗口原始记录，不改写），不作为数字来源；
§9.2/§9.3 的数值即由 guard.log + samples.csv 重算得出。

### 9.3 窗口事实

- 窗口：`2026-10-27T01:13:27.970+08:00` → `01:16:50.025+08:00`（body wall **202.1 s**）
- 基线 3 点：`2222,13840,3,31,596.36` / `2222,13840,2,31` / `2222,13840,2,31`；free 13840 ≥ 10240
- post：`2361,13701,1,31,596.36`；结束基线 3 点：`2361,13701,1` / `2351,13711,28` / `2552,13510,28`（+31,596.36）
- post `compute-apps`：全桌面进程（其中几条为 `[Insufficient Permissions]` 的桌面/系统进程，
  基线与 post 均存在、属同口径噪声），按 `python|g10_p5|cargo|mjwarp` 过滤 = **0**；无 python/cargo 残留
- **零 Rust 编译**：窗内只跑 9 个冻结 exe + `nvidia-smi` + guard 采样；冻结后未跑 cargo
- 机器负载 precheck：`cargo/rustc/cl/nvcc` 均无；CPU 负载采样 `1,3,3`；无 clean 重建/全量测试并行
- 参考未动：旧 M1 参考 `c83baffe…`（5703 B）与 M3 参考 `96e7895d…`（3676 B）窗口前后一致

### 9.4 判定

**attempt1 = 首个有效轮**（≤2 有效轮内；无作废/污染轮）；窗口数值按「判据内零失败」记录，
mesh-mesh 位级一致；不据此宣称一般 mesh 或全分支覆盖（§8）。

## 10. 与 M2 的不变量核对（M3 期间未动）

| 不变量 | 状态 |
| --- | --- |
| ABI 504 B / sidecar 72 B / 入口 6 | 未改（`DEVICE_SOURCES` 28→28、入口 6→6） |
| 设备源（`src/physics/collision/cuda/*.cu`） | **0 行改动** |
| 设备源冒烟 | `254485 / 00810ddb5355564eed422e9d992a6843baf1f5e3541714230463df45a16ee253 / 3883740 / 6`（与 M2 相同） |
| 旧 M1 参考 / 旧 G5 bins / 参考 | 未读写、未改（验证窗前后 sha 复核一致） |
| M3 参考（新） | 验证窗前后 `96e7895d…` 一致，未被测试改动 |
| 判据/容差/期望 | 零改动（compare 复用 + 新增 geom 顺序参数，M2 行为不变） |

## 11. 运行与复现

- 预检：`target/mjwarp-parallel/venv/Scripts/python.exe -B docs/development-evidence/mesh-multicontact-m3/scripts/precheck_mesh_mesh_cpu.py`
- 捕获（窗口内，需签名）：`.../scripts/capture_mesh_multicontact_mesh_mesh_gpu.py <cache-dir> <out.json>`
  （RUN A/B 独立进程 + 独立 cache；compare 用仓库 `scripts/compare_primitive_capture.py`）
- bins：`.../scripts/prepare_mesh_mesh_case.py`
- CPU 测试：`cargo test --locked --test g10_p5_mesh_poly_case_assembly --test g10_p5_mesh_m3_case_assembly`
  （validate 相关单测：`cargo test --locked --lib runtime::collision_gpu::validate`）
- 设备测试（窗口内）：`cargo test --locked --features cuda-probe --test g10_p5_mesh_multicontact_m3_cuda -- --ignored --test-threads=1 --nocapture`
  （实际窗口跑的是预构建冻结二进制；构建口径见 §9.5）
- 窗口 runner（不入库）：捕获窗 `target/m3-mesh-mesh/window/run-capture-attempt1.ps1`（sha 见 §12）；
  验证窗 `target/m3-mesh-mesh/window/run-validation-attempt1.ps1`（sha `81475a05…`，见 §12）

### 11.1 验证窗冻结二进制（9 个；窗口内零编译）

构建口径：`cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --no-run --test …`
（与 M2 窗口同型；同名实体因 M3 源改动 sha 变化，属预期）。冻结副本目录 `target/m3-freeze/`
（不入库；复本逐字节复核一致），runner 开窗前逐实体 sha 复核。

| target | 文件 | 字节 | sha256 |
| --- | --- | --- | --- |
| mesh_m3 | `g10_p5_mesh_multicontact_m3_cuda-31f06b6f3d1d660f.exe` | 41743360 | `e502b8eb8678016718196bf50b532050884162647e268128467fc7a03224b69f` |
| mesh_m2 | `g10_p5_mesh_multicontact_m2_cuda-e11ae6eb82e10bd5.exe` | 41740800 | `a6b3548512004221c5be13c52893789d50231a4d7580b694df860c5a7e3f14db` |
| multicontact | `g10_p5_multicontact_cuda-3c769ef530070ffe.exe` | 41686016 | `0901879b47bea6d7d7ff60b5248fb8d270ee9ddf15dd5d4b43668191beb7c8d1` |
| stage2 | `stage2_equivalence_window-b7e65259db9435d8.exe` | 42265600 | `9e5f0b0968d7cca4a9e71093a466f5d39eb0b45e996dc49fe0e994dfb9009b70` |
| primitive | `g10_p5_primitive_cuda-0908b0f61ffbc599.exe` | 41689600 | `5511da57e8c8ebd09ef8472f6d2cdf05f52abf463158fd2d388196bcbf0e73da` |
| launcher | `g10_p5_nvrtc_launcher-719917fb0657e1fe.exe` | 41507840 | `6a2a324910074ea16035e122cbbd282d7c11c3f0f07541cbdb2626eea87f288d` |
| plane | `g10_p5_primitive_plane_cuda-b1e8897a38248f9f.exe` | 41561088 | `e7dccc078092357cf905bb47f6b947dc4c093ac8444ff149b5401176b62ef016` |
| sphere_capsule | `g10_p5_primitive_sphere_capsule_cuda-effe95bf0fed2063.exe` | 41549312 | `6cd4f7a6755ba5af0dcb384ea734a7f1c7428b5124b0f5273d0c6cd3627373fd` |
| branches | `g10_p5_primitive_branches_cuda-555175a2d3995ff7.exe` | 41625600 | `ff61a349d87ddd0cee8643271ec533ed2080be336860152b30dab47e54ba92ef` |

runner 内固化同一张表（不符即 abort）；`logs/frozen-binaries.txt` 为同一清单的原始留档。

## 12. 文件与 sha256

| 文件 | 字节 | sha256 |
| --- | --- | --- |
| `fixtures/g10-p5-mesh-multicontact/mesh-mesh.xml` | 617 | `bb9ffb2f979129bbac410e9959088528f35180d14f5572c9a52631fa2ed86d8d` |
| `.../m3/upstream-mesh-multicontact-mesh-mesh-gpu.json` | 3676 | `96e7895d13c3ba23475a013fcaa586485e93f95cced5351e2874973917e149a1` |
| `.../m3/scripts/capture_mesh_multicontact_mesh_mesh_gpu.py` | 7552 | `7e7c45f18601d9518b4105eb1799a98f84c3ddf6fb498aa665b01e1bd545eda3` |
| `.../m3/scripts/precheck_mesh_mesh_cpu.py` | 9602 | `bc00d2dc99e969d9bb8c045d752911d443a6ed67cba566b3b60bf90e3924d08b` |
| `.../m3/scripts/prepare_mesh_mesh_case.py` | 20592 | `c44ff8782aceaab1ccd77d6d9674b402cf0260231963793b12ca05de6b2e6d3d` |
| `.../m3/scripts/sha256.json`（三脚本清单） | — | 见文件本体 |
| `.../m3/mesh-case-manifest.json` | 8626 | `abc44369c219b2284ff176f8c22d7ed82fbecfda503366e67256ee321a6f66fc` |
| `.../m3/precheck-cpu.json` | 4177 | `37e26764e49b75e4a9b15b5d49b46521f9f425a452295d7d25a77d97ef37aebe` |
| `scripts/compare_primitive_capture.py`（仓库既有，零改动） | 849 | `27ecb54a199798f5ea00c4accb6db65e65734cd023daa9102bc151557f3ccced` |
| 旧 M1 参考（零改动） | 5703 | `c83baffe6de695fdefda6852c574d5d0d7312ebb591c1b487dfc59ec126c206a` |
| 捕获窗 runner（不入库） | 6936 | `c5400cb5b35bb67b74e13108664222b601512559a309d7e2f6639cb973deafb7` |
| 验证窗 runner（不入库） | 8397 | `81475a0580a13ddaad03e83a5189b2baf8136de27c3e2dc703c3d8e786ea7f89` |
| 验证窗冻结二进制清单 `logs/frozen-binaries.txt` | 1520 | `8485931efcc7db04eefcc2caad6b1d5f12cde1fccfdc1089cd02afc3dfb729d5` |

窗口原始留档（逐字节复制）：
- 捕获窗：`logs/capture-attempt1-{baseline,facts,guard,run-a,run-b,compare,samples}`
- 验证窗：`logs/validation-attempt1-{baseline,facts,guard,samples,machine-load,binary-shas,frozen-binaries}` +
  9 个 `<target>.log` + `stage2-cpu-compare.log`（记录 C 只记录口径）
- 权威采样来源：`validation-attempt1-guard.log` + `validation-attempt1-samples.csv`（见 §9.2 说明）

## 13. 测试文件 500 行棘轮拆分（M3 收口修正）

M3 收口 commit 被 normify commit-msg 的 500 行棘轮拒绝（两测试文件超标），按 Chief 四条条件做
**纯搬移**拆分后重跑门禁：

| 文件 | 拆分前 | 拆分后 | 处理 |
| --- | --- | --- | --- |
| `src/runtime/collision_gpu/validate_tests.rs` | 506 | 237 | 4 条 mesh-poly 测试 + `CubeMeshPoly` 数据块原样搬到 `src/runtime/collision_gpu/validate_mesh_poly_tests.rs`（282 行），父文件用 `#[path = "validate_mesh_poly_tests.rs"] mod mesh_poly_tests;` 挂接 |
| `tests/g10_p5_mesh_poly_case_assembly.rs` | 601 | 418 | M3 两条测试 + M3 常量原样搬到 `tests/g10_p5_mesh_m3_case_assembly.rs`（183 行） |
| `tests/support/mesh_poly_case_support.rs` | 383 | 406 | 仅搬入 3 个 helper（`sha256_hex` / `f32_bits` / `json_f32_bits`；Arch 允许「装配 helper 挪到 tests/support/」） |

- **纯搬移验证**：脚本比对「新子文件正文 == 原文件对应行」逐字节 **True**（validate 子文件 body、
  M3 常量与两条测试均 verbatim）；`cargo fmt` 只消除了拼接处 1 处多余空行（assembly 419→418），
  搬移块内无格式/注释/断言改动。
- **测试数与名字不变**：`validate` 相关单测 8/8（4 条在 `validate_tests` 父模块 + 4 条在
  `validate_tests::mesh_poly_tests` 子模块；测试名不变，仅模块路径前缀变化）；
  assembly 7/7 = 旧文件 5 + 新文件 2（测试名不变，文件路径变化）。
- **拆分前后门禁数字与行为一致**：拆前 unit **990 过 / 0 败 / 374 ignored**；拆后
  `normify gate`（fmt/clippy/clippy-cuda/unit 重跑）**990 / 0 / 374**，exit 0；
  定向测试 `validate` 8/8、assembly 5+2 全过（`cargo test --locked --test ...`）。
- 拆分只动测试文件与 `tests/support/` helper：产品代码、判据、窗口冻结二进制与设备源均未改，
  窗口结果不受影响（无需重开窗）。
- `scope-selfcheck`（拆分后）：**75 路径 / 越界 0**。