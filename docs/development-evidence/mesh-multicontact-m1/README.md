# G10@P5 M1：mesh multicontact dispatch 审计 + fixture + 上游 GPU 参考捕获

任务：`2026-10-26-mesh-multicontact-m1-capture`（in_progress，owner Backend）。
上游冻结修订：`71da24d956378a87a703b6e1442b13aec0c4ac29`。

**非目标**：不实现设备侧 mesh 多接触内核（M2）、不改判据/容差/参考/期望、不动既有 fixture 与
G5/B0/B3 旧参考、不宣称 mesh 已支持。本片只交付「dispatch 审计 + 新 fixture + CPU 预检 +
1 次上游 GPU 参考捕获」。

## 1. dispatch 审计（硬门槛，结论：走 multicontact）

详见 `dispatch-audit.md`（逐条行号）。摘要：

- `(BOX,MESH)`→CONVEX（`collision_driver.py` L79）、`(CYLINDER,MESH)`→CONVEX（L77）、
  `(MESH,MESH)`→CONVEX（L80）；目标组合不在 primitive 表（L48-70 只含 plane/sphere/capsule/box；L53 的 (PLANE,MESH) 属 plane-mesh、不在目标组合内）。
- `_narrowphase` 分流（L908-920）→ `convex_narrowphase`；`NATIVECCD disable` 只把 `(BOX,BOX)`
  覆盖为 PRIMITIVE（L910-912），**不影响** mesh-box / mesh-cylinder。
- `collision_convex.py`：`use_multiccd`（L1275）、mesh 组合扩大 `npolygonmax/nmeshdegmax`
  （L1283-1289）、`ccd_kernel_builder(use_multiccd)`（L756/761）、multicontact 静态门
  （L906-910）、mesh 需 `mesh_polyadr >= 0`（L911-919）、`multicontact(...)`（L922-944）、
  逐接触 `write_contact(id_=i)`（L982-995）⇒ 同一 pair 可写最多 4 行。
- `collision_gjk.py`：`multicontact`（L2216，vec4 dists）、`epa_phase` 返回 `multiccd_idx`
  （L2573）；`margin != 0` 或类型不在 {BOX,MESH,CYLINDER} → 单接触回退（L2669-2677）。
- **mesh-mesh 预留**：同 CONVEX/multicontact 分支，M1 不给 case。

## 2. fixture（新目录，旧目录零改动）

`fixtures/g10-p5-mesh-multicontact/`：`cube.obj`（8 顶点/12 三角凸立方）、`mesh-box.xml`、
`mesh-cylinder.xml`。两 XML 均含 `nativeccd="disable"`、`gravity=0`、确定性 `capture` keyframe；
**未禁 MULTICCD**、margin=0（目标路径成立条件）。sha 清单：`inputs-sha256.json`
（`cube.obj=90a8a990…`、`mesh-box.xml=88b8d7af…`、`mesh-cylinder.xml=d0b75f91…`）。

| case | 构图 | 位置（qpos） |
| --- | --- | --- |
| `mesh-box` | 静态 mesh 立方（半宽 0.1）+ free box 0.08×0.08×0.05 | box center z=0.14（底面 z=0.09 侵入 mesh 顶面 0.01） |
| `mesh-cylinder` | 静态 mesh 立方 + free cylinder r=0.06 h=0.1 | cylinder center z=0.19（底面 z=0.09 侵入 0.01） |

## 3. CPU 预检（只证 fixture 有效，不作判据）

脚本 `precheck_mesh_multicontact_cpu.py`（pinned venv，`wp.set_device("cpu")`，不建 CUDA
context），原始 JSON：`precheck-cpu.json`。**PASS（2/2）**：

| case | MULTICCD on | MULTICCD off | ids(on) | dist(on) | mesh polygon |
| --- | --- | --- | --- | --- | --- |
| `mesh-box` | **nacon=4** | nacon=1 | [0,1,2,3] | 全 -0.010000002 | polynum=6, polyadr=0, polyvertnum=4, polyvertadr=0, polymapnum=3, polymapadr=0 |
| `mesh-cylinder` | **nacon=4** | nacon=1 | [0,1,2,3] | 全 -0.010000005 | 同上 |

- 两 case 均 `polygon_data_present=true`、`nativeccd_disabled=true`、`multiccd_enabled_by_default=true`；
- on/off 对照证明 fixture 实际走 multicontact 分支（off=1 为单接触回退）；
- **预检只证 fixture 有效**；GPU 参考仍以捕获为准（CPU/GPU 后端计数可能不同，以 GPU 为准）。

## 4. 捕获窗 attempt1（已双签执行，结果 passed）

### 4.1 开窗前基线（runner 原文，见 `baseline.txt`）

```
4659, 11403, 32, 32, 596.36
4659, 11403, 40, 32, 596.36
4659, 11403, 32, 32, 596.36
```

格式 `used_mib, free_mib, util_percent, temperature_c, driver_version`；free 11403 ≥ 10240。
开窗前另做人工复核采样（`4659, 11403, 36` / `…37` / `…33`，used/free/driver 相同）。
`compute-apps` 原文（`baseline.txt` 末行，实测 31 条）：全部为桌面图形进程
（explorer / Chrome / Feishu / VS Code / NVIDIA Overlay 等，`used_gpu_memory` 均 `[N/A]`）；
按 `g10_p5|capture_mesh|mjwarp-parallel` 过滤命中 **0** 条 → 无本任务 CUDA 进程，允许开窗。

### 4.2 脚本签名与产物

- capture：`capture_mesh_multicontact_gpu.py`，sha256
  `cbd71dca712bd9e94ce06e87d1ee8ae63f3f85ca5536b38c338b240bb3ecad87`（签名一致）
- compare：`compare_primitive_capture.py`，sha256
  `27ecb54a199798f5ea00c4accb6db65e65734cd023daa9102bc151557f3ccced`（复用、未改动，签名一致）
- 产物（**新文件**）：`upstream-mesh-multicontact-gpu.json`，5703 B，sha256
  `c83baffe6de695fdefda6852c574d5d0d7312ebb591c1b487dfc59ec126c206a`
- 中间 raw / 日志留档：`target/m1-mesh-multicontact/attempt1/`
  （`baseline.txt`、`samples.csv`、`guard.log`、`run-a.log`、`run-b.log`、`compare.log`、
  `facts.txt`、`upstream-mesh-multicontact-{A,B}.json`、`warp-cache-a/`、`warp-cache-b/`）

### 4.3 窗口时间与两次 RUN

- window（RUN A 开始前 → compare 结束）：`2026-10-26T23:04:40.540+08:00` →
  `2026-10-26T23:05:19.866+08:00`，wall ≈ 39.3 s
- RUN A：exit 0，wall 27.7 s；RUN B：exit 0，wall 6.9 s；compare：exit 0
- RUN A/B 为**独立进程 + 独立 warp cache**（`warp-cache-a` / `warp-cache-b`），
  `device=cuda:0`（日志 `device: cuda:0`，RTX 4070 Ti SUPER sm_89，CUDA Toolkit 12.9 / Driver 13.2）
- 每 case 进程内两次捕获：`mesh-box: count=4 two_captures=True multiccd=True`、
  `mesh-cylinder: count=4 two_captures=True multiccd=True`
- `captured_at=2026-10-26T23:04:40.540+08:00`（runner 取真实窗口开始时间并 export，A/B 同值；
  与参考 `provenance.captured_at` 交叉核验一致）
- **零 Rust 编译**：窗口内只调用 pinned python（capture A、capture B、compare 共 3 次）+
  `nvidia-smi` + guard 采样，未调用 cargo/rustc。RUN A 的 27.7 s 是 warp/NVRTC 自身 JIT
  （日志 `ccd_kernel_... compiled 10804.21 ms` / `10618.56 ms`），属 pinned python 捕获固有行为，
  不是 Rust 编译；RUN B 因 warp 库缓存命中同源 kernel 而明显更快（`633.69 ms` / `489.71 ms`）。

### 4.4 采样 / guard v2 聚合（0.25 s 采样）

- samples **129**（seq 1–129，`23:04:41.806` → `23:05:17.116`，连续无缺口，实测间隔 ≈ 0.276 s）
- `used_peak=5082 MiB`、`free_min=10980 MiB`、`util_max=94%`（`util_streak_max=2`，未触发
  80%×3 中止）、`delta_peak=418 MiB`（`increment_limit_mib=3072`）、`baseline_used=4664`
- `Assert-Guard -Phase post`：used 4677 / free 11385 / delta 13 / util 31，无 abort；
  `Assert-GuardAggregates` 无 `GUARD_ABORT`（`guard.log` 原文留档）
（近阈值：94% 与 streak 2 接近 80%×3 守卫线，本片未 abort，规则正确。）
### 4.5 raw A/B 逐字节一致

```
run_a=...\upstream-mesh-multicontact-A.json sha256=c83baffe6de695fdefda6852c574d5d0d7312ebb591c1b487dfc59ec126c206a
run_b=...\upstream-mesh-multicontact-B.json sha256=c83baffe6de695fdefda6852c574d5d0d7312ebb591c1b487dfc59ec126c206a
RAW BYTE-IDENTICAL: PASS
```

compare exit 0 → 参考自动复制到 `upstream-mesh-multicontact-gpu.json`（sha 同 `c83baffe…`）。
本片 attempt1 为**首次捕获尝试且 passed**，无作废轮、无拼接。

### 4.6 结束基线 / post compute-apps / 残留

- 结束基线 3 点：`4670, 11392, 17, 32, 596.36` / `4677, 11385, 20, 32, 596.36` /
  `4677, 11385, 27, 32, 596.36`
- post `compute-apps`：仍全部为桌面图形进程，无 python/capture/mjwarp 相关进程 → 无残留
- guard 采样器：`process_exited=True`、`residual=0`

## 5. 上游 GPU 参考内容核验（2/2 case）

参考顶层：`schema=1`、`kind=m1-mesh-multicontact-upstream-mjwarp-f32-gpu-reference`、
`task_id` 与一致、`upstream_revision` 与冻结修订一致、`dispatch` 字段写入 multicontact 判定依据、
`purpose` 明确「authoring only，not a product-precision verdict」。

### 5.1 计数与字段（两 case 均 `count=4`）

| case | count | geomcollisionid | dist | pos（z / xy） | two_captures |
| --- | --- | --- | --- | --- | --- |
| `mesh-box` | 4 | [[0],[1],[2],[3]] | 全 `-0.010000001639127731` | z=`0.0949999988`，4 个角点 | true |
| `mesh-cylinder` | 4 | [[0],[1],[2],[3]] | 全 `-0.01000000536441803` | z=`0.0949999988`（f32 实测），4 个 rim 接触点 | true |

- 结构字段齐全且可用于零容差比较：`type=1`、`dim=3`、`geom=[1,0]`、`worldid=0`、
  `includemargin=0`、`adhesion=0`、`friction=[1,1,0.005,1e-4,1e-4]`、`solref=[0.02,1]`、
  `solreffriction=[0,0]`、`solimp=[0.9,0.95,0.001,0.5,2]`、`frame` 4×9
- 输入留痕：`qpos` + `qpos_bits`（逐位）、`geom_xpos`/`geom_xmat`（`[nworld][ngeom][3/9]`）、
  `xml_sha256`（与 `inputs-sha256.json` 一致）、flags `nativeccd_disabled=true /
  multiccd_enabled=true`
- `fields.elem` / `fields.vert` 为空数组：参考**不携带 mesh 多边形输入数组**（见 §6 裁定 2）
- 参考**未记录 `efc_address`**（与 B0/B3 同约定，见 §6 裁定 1）

### 5.2 provenance

`captured_at=2026-10-26T23:04:40.540+08:00`、`device=cuda:0`、
GPU `NVIDIA GeForce RTX 4070 Ti SUPER` 16376 MiB / driver 596.36 / sm_89、
python 3.12.13 / mujoco 3.15.0 / warp 1.15.0 / mujoco_warp 3.15.0 / numpy 2.5.4、
`authoring_script_sha256=cbd71dca…`（与签名一致）、`authoring_only=true`、
`product_python_dependency=false`、`reads_product_tree=false`、`kernel_cache_parent=target/m1-mesh-multicontact/attempt1`。

### 5.3 旧参考未动（跑完实测 sha）

| 文件 | sha256 |
| --- | --- |
| `docs/development-evidence/g10-p5-primitive-capture/upstream-primitive-gpu.json` | `3a6475acf362795ab98e8ee39309b09102c2dc76f7a57c6c6dbc95a069a98734` |
| `docs/development-evidence/g10-p5-primitive-branch-coverage/upstream-primitive-branches-gpu.json` | `a8407a90ba7ab491aac7e73c94108e5abd07ce3a3d4218382477b109dc62fca0` |

G5 证据与既有 fixture 目录零改动。

### 5.4 与 CPU 预检的交叉一致（辅助，不作判据）

两 case GPU `count=4` 与 CPU 预检 `nacon=4` 相同；`dist` 数值与预检逐位相同
（mesh-box `-0.010000001639127731`、mesh-cylinder `-0.01000000536441803`）。
这只说明 fixture 与分支稳定，**M2 判据仍以 GPU 参考为准**。

## 6. 缺口裁定（Arch/Chief 已确认）

1. **参考未记录 `efc_address`** → 不补采。M2 采用「设备侧 `efc_address` 全 `-1` 不变量 +
   证据注记『参考未记录 efc』」的既有约定（B1/B2 同口径）。
2. **参考不含 mesh polygon 输入数组** → 不补采。M2 的 32 数组 bins 由 **M0b** 从同一 pinned
   模型 + 同一 fixture（`fixtures/g10-p5-mesh-multicontact/`，`xml_sha256`/`qpos_bits` 必须与
   本参考逐位一致）生成，不在 M1 扩 scope。
3. **`contact.geom=[1,0]`** 是 pinned 类型序规范化结果（box=6 < mesh=7，低类在前）；
   与 B3 swap 同语义，M2 直接按零结构容差比较该顺序，不另加顺序断言逻辑。

## 7. 脚本归档 / 复现

- 入库副本：`scripts/capture_mesh_multicontact_gpu.py`、`scripts/precheck_mesh_multicontact_cpu.py`
  （sha 见 `scripts/sha256.json`，归档时逐字节校验一致：7161 B / 6636 B）。
- **执行时**路径在仓库 `scripts/`；入库副本在本证据目录 `scripts/`。归档后已按 B3 做法删除仓库
  `scripts/` 下的两个未跟踪副本；如需复跑，先把副本复制回 `scripts/` 并核对 sha256。
- runner（窗口编排，未入库）：`target/m1-mesh-multicontact/window/run-window-attempt1.ps1`；
  比较脚本为仓库既有 `scripts/compare_primitive_capture.py`（复用，未改动）。
- 复现要点：pinned python `target/mjwarp-parallel/venv/Scripts/python.exe`；RUN A/B 独立进程 +
  独立 cache；raw 逐字节比较；每 case 进程内两次捕获；`MJWARP_CAPTURE_AT` 由 runner 从真实窗口
  开始时间 export（A/B 同值）。

## 8. 未覆盖 / 红线

- 本片不含设备实现与设备数值对照（M2）；参考是 **authoring 用途**，不构成产品精度结论。
- mesh-mesh 无 case（预留）；mesh-ellipsoid 等其它 mesh 组合不在本片范围。
- 参考未记录 `efc_address`、不携带 mesh 多边形输入数组（均见 §6 裁定）。
- 不改判据/容差/参考/期望；不动既有 fixture/G5/B0/B3 证据；**不宣称 mesh 已支持**。
- `src/`、`tests/` 本片零改动。

## 9. 证据文件清单

| 文件 | 用途 |
| --- | --- |
| `README.md` | 本文件：审计 + fixture + 预检 + 捕获窗事实 + 参考核验 |
| `receipt.json` | 机器可读回执（task_id / attempts / reference / 窗口事实 / 裁定） |
| `dispatch-audit.md` | dispatch 审计逐条行号证据 |
| `inputs-sha256.json` | fixture 输入 sha 清单 |
| `precheck-cpu.json` | CPU 预检原始 JSON（2 case，on/off） |
| `upstream-mesh-multicontact-gpu.json` | 上游 GPU f32 参考（2 case，5703 B） |
| `scripts/sha256.json` + `scripts/*.py` | 捕获/预检脚本入库副本与 sha |

### 脚本归档的复核链（QA M1 观察 O3）

- `byte_identical_after_copy` 在执行副本删除后**无法再独立复核**（这是如实边界，不宣称可复验）。
- 实际证据链：参考 JSON 的 `provenance.authoring_script_sha256` = `scripts/sha256.json` 记录的归档副本 sha
  （capture `cbd71dca…`、precheck `4e8ee1d2…`），即「同一文件的两个独立指纹互相印证」，而不是事后对执行副本再比一次。
