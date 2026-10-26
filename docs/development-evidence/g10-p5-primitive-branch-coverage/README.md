# G10@P5 B3：primitive 分支覆盖 + swap 几何顺序规范化

任务：`2026-10-26-g10-p5-primitive-branch-coverage`（in_progress，owner Backend）。
只报「本片 fixture 触发的分支在给定 case 上对照 B3 上游 GPU 参考的结果」，不宣称
primitive 全路径/全分支覆盖。判据/容差与 B1/B2 同口径，未改动。

## 1. 范围

- 新 fixture 目录 `fixtures/g10-p5-primitive-branches/`（6 个 MJCF + `pentagon_prism.obj`）；
  `fixtures/g10-p5-primitive/` 与既有参考一律只读。
- 6 条目标分支：plane-mesh **graph**、sphere-sphere **dist==0 fallback**、sphere-capsule
  **投影 clamp**、sphere-cylinder **端面 cap**、sphere-cylinder **角点 corner**、capsule-capsule
  **non-parallel det 分支**。
- swap 几何顺序：按 pinned `_add_geom_pair`（collision_driver.py L351-380）在 device 入口按
  类型序规范化 pair（type1 > type2 时交换 geom/type；相等类型保持调用方顺序）。

## 2. fixture 与分支命中证明（窗口前 CPU 预检）

预检脚本：`scripts/precheck_primitive_branches_cpu.py`（pinned venv python，`wp.set_device("cpu")`，
不建 CUDA context），结果原始 JSON：`precheck-branches-cpu.json`（同目录）。
每个 case 用 f32 复算 pinned 分支谓词（与 device 输入同一批 `geom_xpos/geom_xmat`），
并附 CPU 后端（同一 pinned 内核）接触数仅作 sanity，不作判据。

| case | 触发分支 | f32 复算证据（实测） | CPU 后端 nacon |
| --- | --- | --- | --- |
| `plane-mesh-graph` | `plane_convex` graph hillclimb（pinned L79 条件取反：`graphadr != -1 && vertnum >= 10`） | `graphadr=0`、`vertnum=10`、`graph[0]=10`、`max_support=0.05000000074505806 ≥ 0` | 4 |
| `sphere-sphere-concentric` | `sphere_sphere` `dist == 0` fallback（L81-84，`n=(1,0,0)`） | 两球心 f32 距离 `= 0.0` | 1 |
| `sphere-capsule-clamp` | `sphere_capsule` `clamp(dot(axis,offset), ±half_length)`（L117-121） | 投影 `= 0.15000000596046448 > 0.12`（clamp 生效） | 1 |
| `sphere-cylinder-cap` | `sphere_cylinder` 端面分支（L442-453） | `collide_side=false, collide_cap=true, x>0` | 1 |
| `sphere-cylinder-corner` | `sphere_cylinder` 角点分支（L454-462） | `collide_side=false, collide_cap=false` | 1 |
| `capsule-capsule-nonparallel` | `capsule_capsule` 非平行 det 分支（L177-205） | `det=2.0736003310918745e-4`，远大于 `MJ_MINVAL=1e-15`（两轴 90°，`axis_dot=0`） | 1 |

结论：6/6 `hit=true`（`precheck-branches-cpu.json` 的 `failures=[]`、`result=PASS`）。

## 3. swap 几何顺序规范化（先判后做 → 判定为小修，已实现）

- pinned 语义：`_add_geom_pair` 在 pair 写入时按类型序规范化（`type1 > type2` → `(geom2, geom1)`），
  因此上游 `contact.geom` 恒为（低类 geom, 高类 geom）。我们此前 device 入口用调用方顺序 →
  高类在前时 `contact.geom` 顺序与上游不一致（结构零容差判据会红）。
- 判定：**小修**。改动只在 `product.cu` 的 `mjwarp_collision_primitive` 入口内：比较
  `a.type1 > a.type2`、交换本地 `type1/type2/g1/g2`，并把 `params.geoms[0..1]` 覆写为规范化
  顺序；0 新 ABI 字段、0 新入口、不改 dispatch 的数值语义。既有调用方均为低类在前
  （B0/B1/B2 全部 case）→ 规范化是 no-op，13 组合回归要求位级不变。
- 验证：新测试对 4 个类型不同 case 额外以 `(g1=1,g2=0,type1=高类,type2=低类)` 跑 swap 变体，
  与同一 GPU 参考逐字段一致（含 `contact.geom` 顺序）；attempt2 实测 4/4 `failures=0`（§7.4）。

## 4. 文件与行数

| 文件 | 行数 | 说明 |
| --- | --- | --- |
| `src/physics/collision/cuda/product.cu` | 406 | 改：primitive 入口 pair 规范化（+ 既有分派；<450 未拆） |
| `tests/g10_p5_primitive_branches_cuda.rs` | 366 | 新增：6 branch case + swap 变体单 test + 非 ignored CPU 支持层自检 |
| `tests/support/primitive_branches_case.rs` | 251 | 新增：`#[path]` 引用的 case 定义 / mesh 数据 / 分支谓词复算 |
| `tests/support/primitive_compare.rs` | 246 | 新增：`#[path]` 引用的逐字段对照支持（非独立测试目标） |
| `scripts/precheck_primitive_branches_cpu.py` | 245 | 新增：分支命中 CPU 预检（authoring，产品不依赖） |
| `scripts/capture_primitive_branches_gpu.py` | 152 | 新增：B3 捕获脚本（同 B0 配方，旧脚本零改动） |
| `fixtures/g10-p5-primitive-branches/*` | 6 XML + 1 OBJ | 新增；sha 见 `inputs-sha256.json` |

所有新增/修改文件 ≤500 行；`DEVICE_SOURCES` 段数保持 27（本片不新增 `.cu` 片段）；
`src/physics/mod.rs` 零 diff。

## 5. CPU NVRTC 冒烟（窗口外，不建 context）

```text
$env:PATH="D:\Rust\mjwarp-rs\target\toolchains\cuda-12.8.1\bin;$env:PATH"
cargo test --locked --features cuda-probe --lib runtime::collision_gpu::nvrtc_smoke -- --include-ignored --nocapture
```

结果（1 passed / 0 failed）：

```text
nvrtc_smoke_ok ptx_bytes=3786510 entries=6 source_bytes=237922 source_sha256=92ee236e519dc04b26b2d24400fa57f382ed9cea1b5b28596c238ab6cfa4cec8
```

- 入口数 **6 → 6**（0 新入口）；拼装源 237212（B2 收口态 `054aaa40…`）→ 237922（本片
  只改 `product.cu` 的入口文本，变化为预期）。
- `make_frame` 仍单一定义；27 段顺序断言不变。

## 6. 测试与门禁（窗口外）

| 命令 | 结果 |
| --- | --- |
| `cargo test --locked`（默认特性） | **975 passed / 0 failed / 372 ignored**（基线 974/0/371：+1 ignored 窗口测试、+1 新增非 ignored CPU 支持层自检） |
| `cargo test --locked --features cuda-probe --lib runtime::collision_gpu -- --include-ignored` | 26 passed / 0 failed |
| `cargo test --locked --features cuda-probe --test g10_p5_primitive_branches_cuda --no-run` | 编译通过（窗口内不编译） |
| `cargo fmt --all -- --check` / `clippy --all-targets --features cuda-probe -D warnings` | exit 0 / exit 0 |

### 6.1 CPU 支持层自证（窗口前必须过；不建 CUDA context）

`reference_support_parses_without_gpu`（non-ignored）读取已捕获参考 `a8407a90…`，对 6 case 断言：

1. 参考 sha256 锁定 `a8407a90ba7ab491aac7e73c94108e5abd07ce3a3d4218382477b109dc62fca0`；
2. `geom_xpos` 摊平长度 = `nworld*ngeom*3`、`geom_xmat` = `nworld*ngeom*9`（三层数组）；
3. `fields` 各字段行数 = count、行宽 1/3/9/5/2 匹配；`reference_rows` 逐行解析（含
   `geomcollisionid` 与 `geom` 两列）；
4. 6 case `branch_hit_failures` 为空。

实测输出：

```text
plane-mesh-graph: count=4 support_parse_ok
sphere-sphere-concentric: count=1 support_parse_ok
sphere-capsule-clamp: count=1 support_parse_ok
sphere-cylinder-cap: count=1 support_parse_ok
sphere-cylinder-corner: count=1 support_parse_ok
capsule-capsule-nonparallel: count=1 support_parse_ok
support parse OK (sha/3d-flatten/fields-widths/reference_rows/branch predicates); device numerics not run here
```

该测试只读 JSON，不构造 `CollisionGpuProgram`、不建 context；attempt1 的 support 层 panic
（见 §7.3）由它兜底。若窗口再红，日志分层可判「support 层 vs device 数值」。

## 7. 窗口记录

### 7.1 捕获窗口 attempt1（成功，1 有效轮）

- runner：`target/g10-p5-primitive-branches-capture/run-window-attempt1.ps1`；pinned python
  `target/mjwarp-parallel/venv/Scripts/python.exe`；`device=cuda:0`；窗口内零编译（只跑 python）。
- RUN A/B 独立进程 + 独立 warp cache；**raw 逐字节一致**：
  `a8407a90ba7ab491aac7e73c94108e5abd07ce3a3d4218382477b109dc62fca0`（A/B 同 sha）。
  每 case 进程内两次捕获 `two_captures_bitwise_identical=true`。
- `captured_at=2026-10-26T21:53:51.581+08:00` = `window_start`，落在
  `window_end=21:54:13.438` 内；与 baseline/run-a/run-b/samples/facts 的 mtime 同区间。
- 6 case count：`plane-mesh-graph=4`（与 CPU 预检 4 一致）、其余各 1；GPU/CPU 预检计数无差异。
- 采样 70 samples：`used_peak=3874`、`free_min=12188`、`util_max=36`、`residual=0`。
- 旧参考 `upstream-primitive-gpu.json` sha 实测仍为 `3a6475ac…`（未动）。
- 产品：本目录 `upstream-primitive-branches-gpu.json`（sha `a8407a90…`）。

### 7.2 验证窗口签名集合（attempt2 重签后）

| 目标 | sha256 | 状态 |
| --- | --- | --- |
| `g10_p5_primitive_branches_cuda` | `25410b3d46a62a66c46e08269aa74201e542d8be9864cd36e140f76bffc74bf5` | 修复后重建；Chief 重签，attempt2 使用并全绿 |
| `g10_p5_primitive_plane_cuda` | `115b5c9907aad60e73faec44819887c4b8950a24cde83c12d7a67cffd99a1207` | 与签发一致（未变） |
| `g10_p5_primitive_sphere_capsule_cuda` | `280f2e47944b5a01fc5bc7a834cb8a77f66d7a28c854fc359d4b540ca888f0e2` | 与签发一致（未变） |
| `g10_p5_primitive_cuda` | `ec56ff7448e91fef32c955cedc7c3cc868663c2d28ec3aa802ee105dcf81fd15` | 与签发一致（未变） |
| `g10_p5_nvrtc_launcher` | `84ee607479e44c7dbf4cc3dfb5b204252b83b0f8ccf244a83cd2239918d97836` | 与签发一致（未变） |

窗口内 5 二进制并行，判据同 §8；窗口纪律：开窗前重采基线（free ≥10240 MiB、无本任务
CUDA 计算进程）、0.25 s 采样 + guard v2 + 聚合断言、pre/post 快照 + 结束基线 +
`query-compute-apps` 无残留、零编译、作废轮如实记录不拼接。

### 7.3 验证窗口 attempt1（aborted，非数值回退）

- 事实：`window_start=21:55:42.076`、`window_end=21:56:23.829`，body wall 37.1 s；141 samples：
  `used_peak=4672`、`free_min=11390`、`util_max=68`、`residual=0`；签名集合逐字节核对一致；
  窗口内零编译。
- 结果：branches exit 101（panic），plane / sphere_capsule / primitive / launcher exit 0。
- 根因（测试 support 层，非数值）：曾把 `geom_xpos/geom_xmat`（`[nworld][ngeom][3/9]` 三层）
  喂给两层 `flatten_f32` → `.expect("f32")` panic；panic 在第一个 case 的 device 运行之前，
  没有任何 per-case 数值/分支结论。
- 同窗回归（有效但 attempt2 复跑）：B1 plane 6/6 `failures=0`；B2 4/4 `failures=0`；
  primitive 3/3 `max_*_ulp=0`；launcher 3/3 `max_*_ulp=0`。
- 修复（只碰 `tests/`）：新增三层 `flatten_f32_nested`、`reference_rows` 解析抽到
  `reference_expectation`、新增非 ignored CPU 自检（§6.1）；判据/期望/参考/fixtures/`src/`
  零改动。详细记录：`window-attempt1-invalid.md`；原始日志在
  `target/g10-p5-primitive-branches-verify/attempt1/`。

### 7.4 验证窗口 attempt2（全绿，本片最后有效轮）

- 事实：`window_start=2026-10-26T22:18:38.121+08:00`、`window_end=22:19:09.213`，
  body wall 26.5 s；103 samples：`used_peak=5242`、`free_min=10820`、`util_max=45`、
  `delta_peak=22 MiB`、`residual=0`；post/结束基线正常、无残留；5/5 exit 0。
- **6 分支 case 全通过**（failures=0，nacon 与 GPU 参考一致）：
  `plane-mesh-graph`(graph)=4、`sphere-sphere-concentric`(dist==0)=1、
  `sphere-capsule-clamp`(clamp)=1、`sphere-cylinder-cap`(cap)=1、
  `sphere-cylinder-corner`(corner)=1、`capsule-capsule-nonparallel`(非平行)=1。
- **4 swap 变体全通过**（failures=0）：plane-mesh (7,0)、sphere-capsule (3,2)、
  sphere-cylinder-cap (5,2)、sphere-cylinder-corner (5,2)；`contact.geom` 规范化顺序
  `[0,1]` 与数值同时通过，无「顺序不符/数值不符」消息。
- **13 组合位级回归无回退**：B1 plane 6/6 `failures=0`；B2 4/4 `failures=0`；
  primitive 3/3 `bit_equal=80/80, max_*_ulp=0`；launcher 3/3 `max_*_ulp=0`
  （bit_equal 314/314、3122/3122、80/80）。
- 详细记录：`window-attempt2-facts.md`；原始日志在
  `target/g10-p5-primitive-branches-verify/attempt2/`。

## 8. 判据（沿用 B1/B2，未放宽）

位级优先；非位级 `dist` ≤4 ULP@自身量级（ref=0 → 4 ULP@1.0）、`pos` 4·ULP(参考三轴最大绝对值)、
`frame` 4 ULP@1.0；`count`/结构（worldid/type/dim/geom/geomcollisionid/includemargin/adhesion/
friction/solref/solreffriction/solimp）零容差；`efc_address` 全 -1、`overflow` 全 0 设备侧不变量；
行序按 `geomcollisionid` 配对；nacon 按 GPU 参考。

## 9. ABI / T4 / 只读声明

- **0 个新 ABI 字段**（`CollisionArgs`/`CollisionContactOut`/`collision_gpu/abi.rs` 未改）；
  **0 个新入口**（PTX 入口 6 → 6）。
- **T4 `physics_core` 组合身份未变**：`src/physics/mod.rs` 零 diff、未另立 repin、`fixtures/`
  既有目录零改动；本片 fixture 只在新目录新增。
- 旧参考 `upstream-primitive-gpu.json`（sha `3a6475ac…`）与 B1/B2 证据零改动。

## 10. 未覆盖项 / 状态

- 本片设备对照结论以验证窗口 attempt2 为准：6 分支 + 4 swap + 13 组合回归全部通过（§7.4）；
  attempt1 aborted（support 层 bug）已单独留档、不拼接。
- plane-mesh 只验证 graph 分支的一条数据路径（10 顶点棱柱）；capsule-capsule 只覆盖一组
  90° 非平行轴向；sphere-cylinder 只覆盖 `x>0` 侧端面/角点。以上按「仅覆盖给定 case」口径，
  不外推为全分支覆盖。
- UNC-5 G5 逐秒峰值未合并进本片（单列后续）。
- 未宣称 primitive 全分支/全路径覆盖，未宣称精度或跨环境结论。

## 9. 脚本归档（入库副本）

- 窗口执行时脚本路径为仓库 scripts/capture_primitive_branches_gpu.py / scripts/precheck_primitive_branches_cpu.py（参考 JSON 的 uthoring_script 记录该路径）；
- 为满足提交范围与可复现性，本目录 scripts/ 存放入库副本，**与执行文件逐字节相同**：capture 9cff9b310705ec9387b8f69962a08ff76e7c0c7f0386c6af9242aeee610c82d8（6458 B）、precheck 6b0eba49e491c269d3bc12fdcb54a2c7af3278b5055c74b084865c1a4c50e16a（9224 B）；
- 副本 sha 见 scripts/sha256.json。窗口事实与参考 sha（8407a90…）不受归档动作影响。
