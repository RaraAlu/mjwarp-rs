# M1（mesh multicontact dispatch 审计 + fixture + 上游 GPU 参考捕获）QA 独立复核

结论：**pass（阻断 0 / 观察 5）**。复核对象：交付 `35192320`（证据 `docs/development-evidence/mesh-multicontact-m1/`）。本复核纯 CPU：未开 GPU 窗口；独立逐条核 pinned dispatch 行号与语义、重跑 CPU 预检（输出重定向到 target/）、核参考 A/B 逐字节/结构/provenance、从原始 CSV 重算窗口聚合、核旧参考/旧 fixture/G5/B3 证据零改动、脚本归档与门禁。

## 1. 范围与红线

- 交付 diff（`35192320~1..35192320`）全部是：`docs/development-evidence/mesh-multicontact-m1/**`、`fixtures/g10-p5-mesh-multicontact/**`、`.normify/...m1-capture.json`；**`src/`、`tests/` 零改动**；旧 fixture 目录（`g10-p5-primitive/`、`g10-p5-primitive-branches/`、`mjwarp-complex-physics/`、`fixtures/precision/`）零 diff；G5/B0/B3 证据目录零 diff；不宣称 mesh 已支持。
- 参考是 authoring 用途，非产品精度结论；device 侧对照留 M2。

## 2. dispatch 审计（独立逐条核 pinned 源码）

我在 pinned `installed-source/_src/`（rev `71da24d9…`）逐行核审计引用，结论**成立**：

| 审计点 | pinned 实际 | 判定 |
| --- | --- | --- |
| `(BOX,MESH)` / `(CYLINDER,MESH)` / `(MESH,MESH)` → CONVEX | `collision_driver.py` L79 / L77 / L80 逐字存在 | ✓ |
| NATIVECCD disable 只覆盖 `(BOX,BOX)` | L908-920 `_narrowphase`；L910-912 仅 `collision_table[(BOX,BOX)] = PRIMITIVE`；L919 convex、L920 primitive | ✓ |
| convex 窄相 → 带 multiccd 的 CCD kernel | `collision_convex.py` L1236 `convex_narrowphase`；L1266-1270 计数；L1275 `use_multiccd = (disableflags & MULTICCD)==0`；L1283-1289 按 cyl/mesh 组合扩 `npolygonmax/nmeshdegmax`；L1432 传 `use_multiccd`；L756 builder、L761 参数 | ✓ |
| multicontact 静态门 | L906-910：`(use_multiccd or BOX,BOX) and 两侧 ∈ {BOX,MESH,CYLINDER}`；L911-919 mesh 需 `mesh_polyadr>=0` 否则 `multiccd_idx=-1`；L922-944 `multicontact(...)`；L946-951 0 接触回退单 EPA；L982-995 按 `ncollision` 逐条 `write_contact(id_=i)` | ✓ |
| gjk multicontact / EPA 条件 | `collision_gjk.py` L2216 `multicontact`（L2241 `dists=wp.vec4()`）；L2573 `epa_phase` 返回 `(dist,ncontact,x1,x2,multiccd_idx)`（L2592-2593）；L2669-2677 `margin!=0` 或类型不在 {BOX,MESH,CYLINDER} → `idx=-1` | ✓ |

- fixture 满足全部条件：`nativeccd="disable"`、未禁 MULTICCD（默认启用）、margin=0、mesh polygon 数据存在（预检 `polynum=6, polyadr=0, polyvertnum=4, polymapnum=3`）。
- **发现一处措辞错误（不影响结论）**：`dispatch-audit.md` L26 写「primitive 表（L48-70）只含 plane/sphere/capsule/box 组合，无 mesh」，但 `collision_driver.py` L53 存在 `(PLANE, MESH): PRIMITIVE`（plane-mesh 走 primitive）。目标组合 `(BOX,MESH)/(CYLINDER,MESH)` 确实**不在** primitive 表、在 L79/L77 为 CONVEX，所以「mesh-box/mesh-cylinder 走 CONVEX multicontact」结论正确，仅该句为过度概括 → 观察 O1。

## 3. fixture 与 CPU 预检（独立复算）

- 3 个 fixture 实测 sha 与 `inputs-sha256.json` 一致：`cube.obj` `90a8a990…`(570B)、`mesh-box.xml` `88b8d7af…`(538B)、`mesh-cylinder.xml` `d0b75f91…`(552B)；两 XML 均 `nativeccd="disable"`、gravity=0、确定性 `capture` keyframe、无 margin、未禁 MULTICCD；`cube.obj` 8 顶点/12 三角。
- 我用归档的预检脚本副本（仅把输出路径改到 `target/qa-m1/`，逻辑未改）在 pinned venv（`wp.set_device("cpu")`，不建 CUDA context）重跑：**PASS 2/2**，`mesh-box`/`mesh-cylinder` 均 `MULTICCD on → nacon=4, ids=[0,1,2,3]`、`off → nacon=1`、`polygon_data_present=true`、`multicontact_path_hit=true`；与 `precheck-cpu.json` **数值逐项相同**（含 on/off dist、xml sha、qpos_bits）。
- 证据明确写了「预检只证 fixture 有效，不作判据；GPU 参考为准」：README §3、预检 JSON `note`、§5.4「辅助，不作判据」均存在 → 属实。

## 4. 参考可信性

- `upstream-mesh-multicontact-gpu.json` 实测 **5703 B / sha `c83baffe…`**；原始 `upstream-mesh-multicontact-A.json` 与 `-B.json` 各 5703 B，**sha 均为 `c83baffe…`，且 A/B/提交参考三者逐字节相等**（不是只看 compare 日志）。
- 两 case：`count=4`、`geomcollisionid=[[0],[1],[2],[3]]`、`two_captures_bitwise_identical=true`；`dist` mesh-box 全 `-0.010000001639127731`、mesh-cylinder 全 `-0.01000000536441803`；`pos` z≈0.0949999988；`geom=[[1,0]]`（pinned 类型序：box 6 < mesh 7，低类在前）、`type=1`、`dim=3`、`worldid=0`、`frame` 4×9、参数零容差字段齐全。
- provenance：pinned 版本（py3.12.13/mujoco 3.15/warp 1.15/mjwarp 3.15/numpy 2.5.4）、GPU RTX 4070 Ti SUPER/16376 MiB/driver 596.36/sm 8.9（与实测 `nvidia-smi` 一致）、脚本 sha `cbd71dca…`（= 归档副本 sha）、`captured_at=2026-10-26T23:04:40.540+08:00` = facts `window_start` 且在 `window_end=23:05:19.866` 内、`kernel_cache_parent` 存在。
- **旧参考零改动**：`3a6475ac…`（B0）与 `a8407a90…`（B3）实测 sha 不变，提交内零 diff。

## 5. 窗口事实（原始 CSV 独立重算）

- 129 样本（seq 1–129，`23:04:41.806`–`23:05:17.116`，时间戳单调、间隔≈0.276 s）；`used_peak=5082`、`free_min=10980`（>10240 地板）、`util_max=94`、`util_streak=2`（未达 80%×3，未 abort）、`delta_peak=418`（guard baseline_used=4664；`5082-4664=418`）、`residual=0`、`process_exited=true`；基线 `4659/11403` → 结束基线 `4677/11385`（3 点），post `4677/11385 delta 13 util 31`；post compute-apps 原文无本任务进程。
- 我另算「相对 CSV 首样本 delta=199」：首样本已在 RUN A 启动后（used 4883），守卫用的是窗前后快照 4664，二者不矛盾；证据写的 418 与 guard 聚合一致。
- **窗口内零 Rust 编译**：run-a/b 日志无 `cargo`/`rustc`/`Compiling` 标记；RUN A 27.7 s 对应 warp/NVRTC JIT（`ccd_kernel_... compiled 10804.21 ms` / `10618.56 ms`），RUN B 同源 kernel 缓存命中（`633.69/489.71 ms`）→ 属实。RUN A/B 独立进程 + 独立 cache（`warp-cache-a/b`）、`device: cuda:0`。

## 6. 三项缺口裁定（如实、无越界）

1. 参考未记录 `efc_address` → 不补采；M2 用设备侧全 -1 不变量 + 注记（与 B1/B2 同口径）✓。
2. 参考不含 mesh polygon 输入数组（`fields.elem/vert` 空）→ M2 的 32 数组 bins 由 M0b 从同一 pinned 模型/同一 fixture 生成，要求 xml_sha/qpos_bits 与本参考一致；M0b 任务存在且 depends_on M1 ✓。
3. `contact.geom=[1,0]` 为 pinned 类型序规范化（box 6 < mesh 7），与 B3 swap 同语义、M2 零结构容差比较 ✓；我按 `_add_geom_pair` 逻辑推得同为 [1,0]。
- README §8/receipt `uncovered` 明确未宣称 mesh 已支持、无设备对照、mesh-mesh 无 case。

## 7. 脚本归档

- 归档副本 `capture_mesh_multicontact_gpu.py` **7161 B / `cbd71dca…`**、`precheck_mesh_multicontact_cpu.py` **6636 B / `4e8ee1d2…`**，与 `scripts/sha256.json`、`inputs-sha256.json`、参考 provenance 的 `authoring_script_sha256` 一致；仓库 `scripts/` 下执行副本已删除（`git status` 无残留）；`compare_primitive_capture.py` 未改（实测 `27ecb54a…`）。
- 捕获脚本与 B3 脚本 diff 属受控变化（fixture 目录/输出/CASES/task 元数据/增加 flags 记录），**数值捕获逻辑未变**（同 recipe：A/B 独立进程+独立 cache、每 case 进程内两次捕获、captured_at 由 runner 注入）。
- 说明：执行副本删除后「byte_identical_after_copy」无法再独立对比，但参考 provenance 自哈希 = 归档副本 sha，等价链条成立（O3 措辞）。

## 8. 门禁与红线

- 交付未改 `src/`、`tests/` → unit 应保持 980/0/372；我在主树跑 `normify gate`：**exit 0**，fmt/clippy/clippy-cuda/unit 全过（unit 计数见 `.normify/runs/<task>/unit.json`），`normify check` exit 0。
- 未宣称 mesh 支持/全路径/跨环境；本片无 device 数值结论。

## 9. 未覆盖 / 边界

- 未开 GPU 窗口，未重跑捕获；device 对照、null/非 null sidecar 发射级、12 个 G5 case 全量重验均留 M2。
- mesh-mesh 仅有 dispatch 预留结论、无 case；mesh-ellipsoid 等其它组合不在本片。
- precheck 结果只作 fixture 有效性，不作判据。

## 10. 复现命令

```powershell
# 预检重跑（输出改到 target/，不覆盖证据）
python target/qa-m1/precheck_rerun.py
# 独立核对日志（预检等价、窗口聚合、raw/reference、旧参考、脚本 sha）
python target/qa-m1/make_logs.py
normify gate 2026-10-26-mesh-multicontact-m1-independent-check --repo D:\Rust\mjwarp-rs
normify check 2026-10-26-mesh-multicontact-m1-independent-check --repo D:\Rust\mjwarp-rs
```

## 11. 观察清单（详见 `qa-observations.json`）

| id | 级别 | 摘要 |
| --- | --- | --- |
| O1 | 一般 | dispatch 审计「primitive 表无 mesh」措辞错误：L53 `(PLANE,MESH): PRIMITIVE` 存在；目标组合不在 primitive 表，结论不受影响 |
| O2 | 一般 | baseline compute-apps 条目数写 34，原始 baseline 行实测 31 条（facts post 也 31）；无任务进程、窗口结论不受影响 |
| O3 | 建议 | 「byte_identical_after_copy」在执行副本删除后无法独立对比，建议改为「参考 provenance 自哈希=归档 sha」表述 |
| O4 | 建议 | README §5.1 mesh-cylinder pos z 写 0.095，f32 实际 0.0949999988（四舍五入） |
| O5 | 建议 | util 峰值 94%、streak 2 接近 80%×3 守卫线；本片未 abort 正确，建议证据保留「近阈值」提示 |
