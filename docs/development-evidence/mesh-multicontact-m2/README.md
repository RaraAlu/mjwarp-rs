# G10@P5 M2：convex-vs-mesh multicontact 设备实现（+ 验证窗 attempt 记录）

任务：`2026-10-26-mesh-multicontact-m2-convex-vs-mesh`（in_progress，owner Backend）。
上游冻结修订：`71da24d956378a87a703b6e1442b13aec0c4ac29`。
**非目标**：不改判据/容差/参考/期望；不动旧 G5 bins/参考；不宣称 mesh 已支持；
mesh-mesh 延后 M3。窗口口径：**attempt4 = 数值全绿 + guard 因 free floor 环境边界 abort，
经 Chief 裁定接受为有效轮；不宣称通过结论。**

## 1. pinned 移植映射（`collision_gjk.py`，rev `71da24d`）

> 任务书写作 `collision_convex.py`，但这些函数实际在 **`collision_gjk.py`**（行号与任务书一致）。
> 设备源文件头已注明来源。

| pinned 函数/位置 | 设备实现 | 说明 |
| --- | --- | --- |
| `_intersect1` L1591-1602 | `multicontact_mesh.cu::g5_mc_intersect1` | 两整数区间最多 2 个公共值 |
| `_intersect2` L1605-1616 | `g5_mc_intersect2` | 与第三个区间求交 |
| `_mesh_normals` L1620-1686 | `g5_mc_mesh_normals` | dim=3/2/1；`polymap` 交集 + `mat @ polynormal[polyadr+idx]` |
| `_mesh_edge_normals` L1690-1734 | `g5_mc_mesh_edge_normals` | dim=2 单边；dim=1 逐面取前驱顶点成边 |
| `_mesh_face` L1926-1948 | `g5_mc_mesh_face` | 逆序写 face 顶点（逐字保留） |
| mesh 数据视图 `multicontact` L2245-2262 | `multicontact_geom.cu::g5_mc_mesh_view_of` + `product.cu::build_geom` | 按 `dataid` 解析 `mesh_polyadr[dataid]` 与 sidecar 9 指针 |
| normals/edge-normals/face 恢复 L2278-2310 / L2326-2379 / L2393-2436 | `multicontact.cu` 对应分支 | box/cylinder 路径逐字未动 |

## 2. 改动/新增文件（全部 ≤500 行）

| 文件 | 行数 | 改动 |
| --- | --- | --- |
| `src/physics/collision/cuda/multicontact_mesh.cu` | 191 | **新增**：5 个 mesh 助手 + 容量 clamp |
| `src/physics/collision/cuda/multicontact.cu` | 234 | mesh 视图 + normals/edge-normals/face 分派；`nface==0` 防御 |
| `src/physics/collision/cuda/multicontact_geom.cu` | 410 | + mesh 视图结构/构造函数 + clamp 常量 |
| `src/physics/collision/cuda/geom.cu` | 69 | `d_mesh` 扩 9 个 sidecar 指针；`d_geom` +`dataid`/`mesh_polyadr`；mesh 助手 |
| `src/physics/collision/cuda/product.cu` | 446 | `build_geom` 按 `dataid` 接 sidecar（缺失→空指针→单接触回退） |
| `src/physics/collision/cuda/hfield.cu` | 356 | 手工 `d_geom` 初始化补新字段 |
| `src/runtime/collision_gpu/source.rs` | 140 | `DEVICE_SOURCES` **27→28**（`multicontact_mesh.cu` 在 geom 后、clip 前）；本地 `include_str!`（不动 `src/physics/mod.rs`，保 T4 身份） |
| `src/runtime/collision_gpu.rs` | 458 | `#[doc(hidden)] pub fn validate_collision_inputs`（仅测试入口，转调 `abi::validate_inputs`） |
| `src/runtime/collision_gpu/multicontact_tests.rs` | 118 | 段数 28 + 顺序 + 3 个 mesh 符号存在性 |
| `src/runtime/collision_gpu/primitive_tests.rs` | 174 | 段数 28 |
| `tests/g10_p5_mesh_multicontact_m2_cuda.rs` | 89 | **新增**：M2 GPU 对照测试（2 case，ignored） |
| `tests/support/mesh_multicontact_compare_support.rs` | 480 | **新增**：M1 参考口径比较器（`[count][width]`、结构键配对、判据） |
| `tests/g10_p5_mesh_poly_case_assembly.rs` | 435 | +常驻真实校验测试（Chief 追加要求 1） |

**入口数不变（6）**；`src/physics/mod.rs` 零 diff。

## 3. CPU 冒烟（NVRTC，窗口外）

```text
$env:PATH="...\target\toolchains\cuda-12.8.1\bin;$env:PATH"
cargo test --locked --features cuda-probe --lib runtime::collision_gpu::nvrtc_smoke -- --include-ignored --nocapture
```

```text
nvrtc_smoke_ok ptx_bytes=3883740 entries=6 source_bytes=254485 source_sha256=00810ddb5355564eed422e9d992a6843baf1f5e3541714230463df45a16ee253
```

- 对比 M0a：`238969 / ec2e7acd… / 3786520 / 6`；变化来自新增 mesh 片段，**entries 仍 6**。

## 4. 判据与 M2 对照测试（不放宽）

`tests/g10_p5_mesh_multicontact_m2_cuda.rs`（默认 ignored，需 GPU + NVRTC）：
- 参考 `upstream-mesh-multicontact-gpu.json`，**sha 守卫 `c83baffe…`**；两 case 的
  `two_captures_bitwise_identical` 必须为 true；
- 输入：M0b 32 数组 bins（`docs/development-evidence/mesh-multicontact-m0b/cases/`），
  9 个 mesh_poly slice 长度为 **`(1,1,18,6,6,24,8,8,24)`**（第 3 位 `mesh_polynormal` 是
  **18 个 f32 = nmeshpoly 6 × 3**，不是 6；`nmesh=1 / nmeshpoly=6 / nmeshvert=8 /
  nmeshpolyvert=24 / nmeshpolymap=24`），meta 断言 `g1=1(箱/柱), g2=0(mesh), type2=7`；
- 入口 `CollisionKernel::ContactMulti`（上游 `use_multiccd=true` 特化）；
- 判据（`tests/support/mesh_multicontact_compare_support.rs`）：
  `nacon` 与参考 `count` 零容差；行按 `(worldid, geomcollisionid)` 配对；
  位级优先，非位级 `dist` 4 ULP@量级（ref=0→4 ULP@1.0）、`pos` 4·ULP(scale)、
  `frame` 4 ULP@1.0；`friction/solref/solreffriction/solimp/includemargin/adhesion/
  worldid/type/dim/geom/geomcollisionid` 零容差；`overflow` 全 0；
  `efc_address` 参考未记录 → 设备全 `-1` 不变量。
- **`contact.geom` 顺序**：M2 测试显式断言 `contact.geom=[1,0]` 并打印（见 §5.2）。

## 5. 边界与债务（Arch/Chief 已裁定）

### 5.1 scratch 容量 vs pinned 动态上界（接受为未覆盖边界）

mesh 助手固定容量：单顶点相邻 polygon 数（degree）**≤10**、单 polygon 顶点数 **≤16**、
clip **16/32**（与 `contact.cu` scratch 一致）。pinned 对 cyl-mesh 取
`npolygonmax=max(m.npolygonmax,16)`、`nmeshdegmax≥max(m.nmeshdegmax,3)`。
超出本实现上界时 mesh 助手返回 0 → **单接触回退**；当前 cube fixture（degree=3、polygon=4）
远低于上界、未触发。**不宣称一般 mesh 覆盖**，不为它改判据、不造假 case 凑覆盖。

### 5.2 `contact.geom` 顺序（接受现状 + 记债务）

- ① M2 测试**显式断言** `contact.geom=[1,0]`：这是**调用方按 pinned 类型序传入**
  （box 6 < mesh 7，`_add_geom_pair` 归一化在 driver 侧），**不是 device 归一化**。
- ② **债务（M3/后续评估，不混进 M2）**：`mjwarp_collision_contact_multi` 入口尚未做类型序
  归一化——非规范顺序的调用方会得到不同 `contact.geom`；B3 的 swap 只在 primitive 入口做了。

### 5.3 其它

- **mesh-mesh 延后 M3**（本片只做 convex-vs-mesh；M1 未给 mesh-mesh case）。
- **UNC-6 类深穿透**：mesh face 若出现单字段超 4·ULP@scale → 记未覆盖、不修、不降声明面判据
  （窗口实测未触发，见 §8.2）。
- sidecar 缺失（9 数组全空 / `mesh_polyadr<0`）→ 9 指针空 → 单接触回退，与 M0a 语义一致。

### 5.4 QA 观察带入（M0a 修正片复核 O1/O5/O6）

- **O1（长度写法）**：真实 cube 的 9 个 slice 长度统一写
  `(1,1,18,6,6,24,8,8,24)`，并注明「`nmeshpoly=6`，`mesh_polynormal=18 f32（=6×3）`」；
  不要把第 3 位写成 6（多项式个数与 f32 长度是两回事）。
- **O5（强一致 sum 的依据）**：`sum(polyvertnum)==len(mesh_polyvert)`、
  `sum(polymapnum)==len(mesh_polymap)` 的成立依据是 **mjModel 的累加布局**（`types.py` 的 dim
  未显式要求该恒等式）；M0b 生产脚本按**同一 pinned 模型累加布局**生成这两组数组，
  **长度关系与 M1 参考同源**——证据不写成「types.py 强制」。
- **O6（未覆盖边界，M2 不扩范围）**：M0a 的 CPU 校验**未覆盖** `polyvert/polymap` 的
  **元素值域**（只校验区间/长度/求和），也**未与既有 23 项输入做交叉核对**。M2 沿用该边界，
  不扩 scope、不宣称「输入全面校验」。

## 6. 常驻真实校验（Chief 2026-10-26 追加要求 1）

- `src/runtime/collision_gpu.rs::validate_collision_inputs`：`#[doc(hidden)]`，直接转调
  `abi::validate_inputs`（与 `CollisionGpuProgram::run()` 首次分配前同一函数）；
  **仅测试入口、`run()` 行为零改动**。
- 常驻非 ignored 测试 `real_cpu_validation_accepts_m0b_and_old_cases`：
  M0b 两个 mesh case（9 项 present）→ `Ok(())`；旧 G5 **全部 10 个** case 目录（空 sidecar）
  → `Ok(())`。`g10_p5_mesh_poly_case_assembly` → **5 passed / 0 failed**。
- 不再使用临时探针（M0b 期的临时探针方案已由本常驻测试取代）。

## 7. 窗口冻结二进制（8 个；窗口内零 Rust 编译）

**冻结纪律**：以下 8 个实体已冻结（Arch 实算并报 Chief 双签）；**从冻结起不再跑任何 `cargo`
构建/测试/check（会重建 test 二进制）**，窗口内只跑这批实体；任何重建 → 作废重签。
全部为 `cargo test`（默认特性 = `cuda` → `cuda-probe`）构建产物，`--list` 验证共 **14 个 GPU 测试**
非空（1/1/5/1/2/1/1/2）。

| target | 绝对路径 | 大小 (B) | sha256 |
| --- | --- | --- | --- |
| `g10_p5_mesh_multicontact_m2_cuda` | `D:\Rust\mjwarp-rs\target\debug\deps\g10_p5_mesh_multicontact_m2_cuda-e11ae6eb82e10bd5.exe` | 41736192 | `250297c3e137c25c5e360bea68b509d633a215d7968a3af9701459f88b43bc3b` |
| `g10_p5_multicontact_cuda` | `D:\Rust\mjwarp-rs\target\debug\deps\g10_p5_multicontact_cuda-3c769ef530070ffe.exe` | 41681408 | `5154cd186bb3c2f8758ed18f17007c680b77ff389b6b0ee599620be491c127ae` |
| `stage2_equivalence_window` | `D:\Rust\mjwarp-rs\target\debug\deps\stage2_equivalence_window-b7e65259db9435d8.exe` | 42260992 | `fe0cf46a58c910f4e7f93422dccec628d4c95f18e817eed97d8af2d49c86b03c` |
| `g10_p5_primitive_cuda` | `D:\Rust\mjwarp-rs\target\debug\deps\g10_p5_primitive_cuda-0908b0f61ffbc599.exe` | 41684992 | `afc3047a08be71bfea96bd76e4297026c44b10b0e363019a4a278af0394734a8` |
| `g10_p5_nvrtc_launcher` | `D:\Rust\mjwarp-rs\target\debug\deps\g10_p5_nvrtc_launcher-719917fb0657e1fe.exe` | 41502720 | `5fe476ca36ac3d3b706eb8ea6f6cdf4ac6333b56d9b940fedae07a18e733418b` |
| `g10_p5_primitive_plane_cuda` | `D:\Rust\mjwarp-rs\target\debug\deps\g10_p5_primitive_plane_cuda-b1e8897a38248f9f.exe` | 41556480 | `36bb2d52753c83daba49d8d1966a83f90e87fed34123e8edcf76955990b4b515` |
| `g10_p5_primitive_sphere_capsule_cuda` | `D:\Rust\mjwarp-rs\target\debug\deps\g10_p5_primitive_sphere_capsule_cuda-effe95bf0fed2063.exe` | 41545728 | `50fd81ae82ae22ae4c3a7a17b8e700da528a7427bcf7cc51c0140e5cf699ff7f` |
| `g10_p5_primitive_branches_cuda` | `D:\Rust\mjwarp-rs\target\debug\deps\g10_p5_primitive_branches_cuda-555175a2d3995ff7.exe` | 41620992 | `93083de7c34425a229ae28364a09ff0c50952d88fb7ad53125a124b17f264265` |

**签名补签**：会签表里 plane 曾为 63 位（少一个 `f`）；**开窗前哈希复核拦住该位、该轮未跑、
冻结未破**。plane 正确 64 位见上表（第 25 位 `f`），其余 7 个不变。
**8 个是 GPU 窗口二进制**；`g10_p5_ccd_overflow`（纯 CPU 读冻结 bin）**不计入窗口二进制**，
窗口实际只用 8 个、不存在「第 9 个窗口二进制」。

### 7.1 窗口内容映射（8 个二进制 → 覆盖）

| 二进制 | 窗口内运行的测试/内容 |
| --- | --- |
| `g10_p5_mesh_multicontact_m2_cuda` | M2 `mesh-box` / `mesh-cylinder` 对照 M1 参考 `c83baffe…` |
| `g10_p5_multicontact_cuda` | G5 设备 case：cylinder-box / box-box-margin / mesh-mesh |
| `g10_p5_nvrtc_launcher` | G5 设备 case：hfield-sphere / sdf-box / ellipsoid-box |
| `g10_p5_primitive_cuda` | G5 设备 case：sphere-box-shallow / sphere-box-deep / capsule-box |
| `g10_p5_primitive_plane_cuda` | B1 plane 6 |
| `g10_p5_primitive_sphere_capsule_cuda` | B2 sphere/capsule 4 |
| `g10_p5_primitive_branches_cuda` | B3 6 分支 + 4 swap 变体 |
| `stage2_equivalence_window` | 9 case 等价窗口 dump（GPU 侧；CPU 比较在窗外跑）；**额外覆盖**，含 convex-mesh / degenerate-mesh synth，非 G5 bins |

### 7.2 G5 case 覆盖（口径更正：12 → **10 个 bins case / 9 设备 + 1 未覆盖**）

`docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases/` 实际 **10 个** case：

| case | 设备回归 target | 覆盖 |
| --- | --- | --- |
| `cylinder-box`、`box-box-margin`、`mesh-mesh` | `g10_p5_multicontact_cuda` | ✅ 设备 |
| `hfield-sphere`、`sdf-box`、`ellipsoid-box` | `g10_p5_nvrtc_launcher` | ✅ 设备 |
| `sphere-box-shallow`、`sphere-box-deep`、`capsule-box` | `g10_p5_primitive_cuda` | ✅ 设备 |
| `hfield-capsule` | 无设备 target（仅 `g10_p5_ccd_overflow` CPU 口径） | ⚠️ **未覆盖** |
| （synth）`convex-mesh`、`degenerate-mesh` | `stage2_equivalence_window`（额外覆盖，非 bins case） | ✅ 设备 |

结论：**G5 10 个 bins case：9 个有设备回归 target；`hfield-capsule` 仅 CPU 覆盖 → 未覆盖**。

### 7.3 窗口内容更正（acceptance 原文「12 case」为笔误）与 `hfield-capsule` 未覆盖

- **执行口径**：**10 个 bins case / 9 个设备回归 + 1 个未覆盖**（`hfield-capsule` 仅 CPU）。
  acceptance 里「G5 老 12 个 case」为**笔误**（任务声明不改写，按证据层更正记录）。
- **`hfield-capsule` 在 ABI 504 后无设备回归证据（未覆盖）**：不新写 target，不宣称其设备路径回归；
  若用户要求补，则**单列小片**再评估。
- **补偿性证据（不替代设备回归，如实标注）**：
  ① launcher 的 `hfield-sphere` 在同一 hfield 设备路径上做位级回归（`g10_p5_nvrtc_launcher`，
  窗内执行，覆盖 hfield kernel 的接触数组写出）；
  ② ABI 布局静态断言 `abi_collision_args_matches_device_layout_504` 覆盖 hfield 相关参数/指针
  （`hfield_size/nrow/ncol/adr/data`、`hfield_id`）的偏移，证明 M0a 504 变更未扰动 hfield 参数布局
  （CPU 单测，非 GPU 行为证据）。

## 8. 验证窗 attempt 记录（attempt4 经 Chief 裁定 (b) 接受为有效轮）

> 口径声明：**attempt4 = 数值全绿 + guard 因 free floor 环境边界 abort，经 Chief 裁定接受为有效轮**；
> **不宣称通过结论**。attempt1 = 作废（runner PATH 缺 CUDA DLL、无 device 工作）；
> attempt2/3 = 作废（guard free floor）；**attempt4 唯一有效轮**。

详细表见 `logs/window-attempts-summary.txt`；原始留档 `target/m2/attempt{1,2,3,4}/`。

### 8.1 attempt 表

| attempt | 窗口时间 | body | 结果 | guard 摘要 |
| --- | --- | --- | --- | --- |
| 1 | 00:06:29–00:06:30 | 0.9 s | **作废（env）** | runner PATH 缺 CUDA DLL（NVRTC 不可加载）→ 8 个加载期 panic，**无 device 工作** |
| 2 | 00:06:53–00:07:33 | 40.5 s | 作废（guard free floor） | 8 并行：153 samples、used_peak 7475（delta 2608）、free_min **8587**、util_max 44、residual 0 |
| 3 | 00:08:23–00:09:41 | 77.4 s | 作废（guard free floor） | 3/3/2 分批：287 samples、used_peak 6238（delta 1371）、free_min **9824**、util_max 46、residual 0 |
| 4 | 00:10:14–00:13:14 | 180.2 s | **唯一有效轮（Chief 裁定 (b)）** | 完全串行：658 samples、used_peak 6366（delta **1499**）、free_min **9696**、util_max 48、streak 0、residual 0；8/8 exit 0 |

（attempt2/3 的 runner 退出码采集有缺陷但 8 个日志均 `test result: ok`；attempt4 改用直接调用，
退出码全 0。）

### 8.2 数值结果（attempt4；attempt2/3 相同）

- ① `mesh-box` / `mesh-cylinder`：各 nacon=4/4、**bit_equal 4/4**、max dist/pos/frame ULP=0、
  `contact.geom=[1,0]`、failures=0。
- ② G5 9 个设备 case 全量位级：multicontact → cylinder-box 314/314、box-box-margin 391/392
  （max_pos_ulp=1）、mesh-mesh 80/80；launcher → hfield-sphere 314/314、sdf-box 3122/3122、
  ellipsoid-box 80/80；primitive → sphere-box-shallow/-deep/capsule-box 各 80/80；**failures 全 0**。
  `hfield-capsule` 无设备 target → **未覆盖**。
- ③ stage2 9 case（**synth**，非 G5 bins）：dump 全部完成、每 case 两次运行逐位一致（18 dump）；
  窗外 CPU 比较按「记录 C 只记录不判 pass/fail」口径（含 sdf-box 已知 recipe 分岔等 FAIL 行），
  非 M2 回归。
- ④ B1 plane 6 / B2 4 / B3 6 分支 + 4 swap / primitive 3 / launcher 3：failures 全 0。
- ⑤ `contact.geom` 按 pinned 类型序 **`[1,0]`**（M2 测试显式断言；调用方归一化，非 device）。
- ⑥ UNC-6 类深穿透：mesh 两 case 全字段位级/ULP 0 → **未触发**（记未覆盖类未被命中）。

### 8.3 窗口事实（attempt4）

开窗基线 3 点 `4862, 11200, 37/31/37, 32, 596.36`；post `4863, 11199, 11, 33, 596.36`；
结束基线 3 点 `4863,11199,11 / 4863,11199,10 / 4863,11199,10`；post `compute-apps` 无本任务进程；
residual=0；**零 Rust 编译**（只跑 8 个冻结 exe，冻结后未跑 cargo）。

### 8.4 guard 边界：Chief 裁定 (b)（环境边界；守卫未缩）

- 判定：**attempt4 = 数值全绿 + guard 因 free floor 环境边界 abort，经 Chief 裁定接受为有效轮**。
- 开窗基线 free **11200 ≥ 10240**（open 门满足）；abort 来自 8 个 program 的 own usage：
  attempt4 **完全串行**单进程峰值 delta ≈ **1499 MiB**，当前桌面基线下 free ≈ **9696 < 10240**
  → floor 在当前机器状态结构性不可达（非测量污染）。
- **6 条补偿事实**：① 开窗基线 free 11200 ≥ 10240；② post `query-compute-apps` 无本任务进程 /
  residual=0 / process_exited=true；③ delta_peak 1499 MiB 与单进程 own usage 吻合；
  ④ util_max 48%（未触 80%×3）、util_streak=0；⑤ 数值全 bit-equal/判据内（mesh 4/4 位级、
  G5 9 设备 case failures=0、B1/B2/B3+swap 全 0）；⑥ **8/8 exit 0**。
- **唯一触发的 guard 规则 = free floor；新外部 CUDA 进程 / 增量 / util 三条均未触发。**
- attempt1–3 作废对照：attempt1 = runner PATH 缺 CUDA DLL（无 device 工作）；
  attempt2 = 8 并行 free floor；attempt3 = 3/3/2 分批 free floor；**attempt4 唯一有效轮**。

### 8.5 派生口径（M3 起用，不追溯改 M2 数据）

- **open 门保持 `free ≥ 10240` 不变**；
- **guard floor 改为 `floor = min(10240, baseline_free − 3072)`**（让增量守卫主导，free floor 只兜
  外部冲击）；
- 注明：**M2 按当时口径（固定 10240）执行**；**M3 起用新公式**，不追溯修改 M2 数据。

## 9. Chief 2026-10-26 追加要求（单列，必交项）

1. **真实校验必须常驻（不许留成镜像测试）**：已实现 §6（`#[doc(hidden)]` CPU 入口 +
   常驻非 ignored 测试，M0b 真实 bins / 旧 G5 空 sidecar 均走**真实** `validate_inputs`）。
2. **不得改既有 13 组合行为**：代码层面 box/cylinder 路径逐字未动（§2）；位级证明由验证窗
   G5 **10 bins case（9 设备 + hfield-capsule 未覆盖）** + B1/B2/B3/primitive/launcher 回归给出；
   **红了停手报 Arch，不改判据/容差/期望**。
3. **窗口计划**：实现 + CPU 冒烟完成 → 窗口计划 §7/§8，报 Arch 转 Chief 双签后开窗；
   实际 attempt 记录、guard 边界与 Chief 裁定 (b) 见 §8（**不宣称通过结论**）。

## 10. 门禁与验证数字（当前树）

| 项 | 命令 | 结果 |
| --- | --- | --- |
| fmt | `cargo fmt --all -- --check` | exit 0 |
| clippy | `cargo clippy --locked --all-targets -- -D warnings` | exit 0 |
| clippy-cuda | `cargo clippy --locked --all-targets --features cuda-probe -- -D warnings` | exit 0 |
| unit | `cargo test --locked --no-fail-fast` | **987 passed / 0 failed / 373 ignored**（117 二进制；M0b 后基线 986/0/372 → +1 常驻真实校验测试 +1 ignored 新窗口测试） |
| all-features | `cargo test --locked --all-features --no-fail-fast`（msvc + `MJWARP_MUJOCO_ROOT`） | **1036 passed / 0 failed / 431 ignored**，exit 0 |
| NVRTC 冒烟 | 见 §3 | `254485 / 00810ddb… / 3883740 / 6` |
| collision_gpu 子集 | `cargo test --locked --features cuda-probe --lib runtime::collision_gpu -- --include-ignored` | 33 passed / 0 failed |

## 11. 未覆盖 / 红线

- 验证窗已执行 attempt1-4：**attempt4 = 数值全绿 + guard 因 free floor 环境边界 abort，经 Chief
  裁定接受为有效轮（不宣称通过结论）**；attempt1-3 作废原因见 §8.1。
- `hfield-capsule` 在 ABI 504 后**无设备回归证据（未覆盖）**（仅 CPU `g10_p5_ccd_overflow` 口径 +
  §7.3 补偿性证据）；不新写 target，除非用户/Chief 点名。
- 不宣称 mesh 全类型/全分支支持；mesh-mesh 延后 M3；不宣称一般 mesh（degree>10 / polygon>16）覆盖；
  UNC-6 类未被命中但仍记未覆盖边界。
- 旧 G5 bins/参考、判据/容差/期望零改动；`src/physics/mod.rs` 零 diff；入口仍 6。
