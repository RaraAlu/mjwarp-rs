# perworld L v2 采集脚本补写 spec §2.4 判别力计数（零 GPU；G-L1 前置）

- 任务：`2026-11-08-render-perworld-l-v2-capture-discriminability-counts`（owner Backend、p62、P8-渲染与BVH；登记 `b0556af1`）
- 依据：已签 spec v3 `bb064fe0a85ab38abba8043768666a0d9f73fc53d9f2f3dd72eace060c8a924d` / 39,379 B **§2.4**（`capture.json` 另须记录判别力计数）
- 来源：QA 复核实体 F1/O2 修正（`0c770bb2`）的观察项 ②「capture 计数交叉核对在真参考上目前会静默跳过——脚本未把 `c1_rgb_changed_px` 等写进 capture.json，而 spec §2.4 要求记录」。Chief 2026-11-08 派工：**G-L1 前必须补**，否则 G-L1 会产出不合规 capture.json。
- 本卡**纯 CPU**：零 GPU、不采集、不建 CUDA；只动 `scripts/capture_render_perworld.py`、`scripts/perworld_capture_meta.py` + 本证据目录。

## 1. spec §2.4 字段与结构

`capture.json` 新增（参考侧自算，两遍采集逐位一致校验之后计算）：

| 字段 | 结构 | 含义 |
| --- | --- | --- |
| `c1_rgb_changed_px` | `{"0->1": {"bitwise": N, "magnitude": M}, "1->2": {...}}` | cam-persp 相邻 world rgb 变化像素：`bitwise`=任一通道 f32 位型变化；`magnitude`=`max_channel\|Δrgb\| ≥ 1/255` |
| `c2_depth_changed_px` | `{"0->1": N, "1->2": N}` | cam-hinge depth f32 位型变化像素 |
| `c2_seg_changed_px` | `{"0->1": N, "1->2": N}` | cam-hinge seg `(id,type)` pair 变化像素 |
| `campersp_depth_seg_changed_px` | `{"0->1": {"depth": 0, "seg": 0}, "1->2": {...}}` | 诊断项：cam-persp depth/seg（预期 0/0，只记录不判） |
| `camhinge_rgb_changed_px` | `{"0->1": {"bitwise": N, "magnitude": M}, "1->2": {...}}` | 诊断项：cam-hinge rgb（只记录不判） |

相机归属写死（C1 = cam0/cam-persp、C2 = cam1/cam-hinge），不得互换；键名与 spec §2.4 对齐。

## 2. 实现（逐位复刻 G-L0 `readout_c1_c2.py`）

- 计数函数：`scripts/perworld_capture_meta.py::discriminability_counts`（+ `_rgb_counts` 助手）；主脚本 `scripts/capture_render_perworld.py` **恰 import 1 次、恰调用 1 次**，且在**两遍采集逐位一致校验之后**（`if first_hashes != second_hashes: fail(...)` 之后；静态检查见 `artifacts/script-static-checks.json`：identity 检查行 447、调用行 453）。
- 与 readout（sha `4a054aba…`/16,711 B）同式，无自创等价式：
  - `magnitude = count_nonzero(np.abs(x.astype(np.float64) - y.astype(np.float64)).max(axis=-1) >= 1.0/255.0)`
  - `bitwise = count_nonzero(np.any(x.view(np.int32) != y.view(np.int32), axis=-1))`
  - `c2_depth = count_nonzero(d0.view(np.int32) != d1.view(np.int32))`
  - `c2_seg = count_nonzero(np.any(s0 != s1, axis=-1))`
  - 诊断项同式（cam-persp depth/seg、cam-hinge rgb magnitude/bitwise）。
- 不把 rgb 量化回 8-bit 再比字节差（那会多算 254↔255 类相邻码步进，见实体 F1 修正）。
- **行数与身份**：

| 文件 | 行数（前 → 后） | bytes（前 → 后） | sha256（后） |
| --- | --- | --- | --- |
| `scripts/capture_render_perworld.py` | **487 → 490** | 23,483 → 23,637 | `f3af09ab992981f2dc5d7f2e3479aaf9eebb97df0d2a774b247c8e0772ad6f07` |
| `scripts/perworld_capture_meta.py` | **270 → 379** | 12,508 → 19,154 | `3fdb85953681a1a695a9611fc48ddf0862318acf0e0767e831029a820127b87e` |

均 ≤500（git-blob 换行口径）；未新建模块文件。

## 3. CPU 交叉核对（pilot attempt 3 保留数据；npz 在 `target/`、不入库）

`materials/crosscheck_pilot_counts.py` 用生产助手在 `target/render-bvh/perworld-l-v2-gl0/pilot-out/nw3-light-hinge02040/`
上重算，并逐项对照 **`target/render-bvh/perworld-l-v2-gl0/window/readout.json`**（修复后 readout；sha
`7138f238afe3dbce5af6d35a02329db0f280bb7cbac01b597757f9ce0d83c35a`）：

| 项 | 0→1 | 1→2 | 与 readout |
| --- | --- | --- | --- |
| C1 magnitude | 38,285 | 40,644 | ✅ |
| C1 bitwise | 43,604 | 41,815 | ✅ |
| C2 depth | 35,828 | 33,883 | ✅ |
| C2 seg | 16,041 | 16,754 | ✅ |
| 诊断 cam-persp depth / seg | 0 / 0 | 0 / 0 | ✅ |
| 诊断 cam-hinge rgb magnitude | 50,363 | 52,329 | ✅ |
| 诊断 cam-hinge rgb bitwise | 54,380 | 54,250 | ✅ |

结果 `SUMMARY PASS`、`matches_readout_json: true`、`unexpected=0`；原始输出 `logs/crosscheck-pilot-counts.log`，
产物 `artifacts/crosscheck-pilot-counts.json`（含 pilot npz sha256、readout sha256）。pilot 数据**非入库**，本核对只证口径一致，不替代 G-L1 真渲染。

## 4. 默认行为不变（HEAD vs 工作树逐叶对照）

`materials/compare_dryrun.py` 用 HEAD（旧脚本，sha `55b91f23…`/23,483 B、`296c2b12…`/12,508 B）与工作树（新脚本）跑同一组命令：

- 默认 `--dry-run`：**added=[]、removed=[]、changed 仅 `/recipe/script/{sha256,bytes}`、unexpected=0**；
- 显式 `--scene fixtures/render-bvh/scene-a-light/scene.xml --dry-run`：同上，`cameras` 3 条、`cam_active_fixed_only=[true,true,false]`、`legacy_manifest_entries=112`、unexpected=0；
- `--worlds 1 --dry-run`：两侧 exit 1、stderr 逐字相同、不创建 out_dir（行为不变）。
原始输出 `logs/dryrun-compare.log`、产物 `artifacts/dryrun-compare.json`。

## 5. 模块 CPU 自检

`python -B scripts/perworld_capture_meta.py` → **SELFCHECK PASS 16/16**（原 14 + 新增 2：
`discriminability-positive` 两对 world/两台相机/1/255 计入/254↔255 饱和步进只计 bitwise；
`discriminability-negative-missing-array` 缺数组显式拒绝）；日志 `logs/selfcheck.log`。

## 6. 边界与非声明

- 只改两个脚本 + 本卡证据；`unchanged fixtures/`、`unchanged src/`、`unchanged tests/`、spec 目录不变；零 GPU、不采集、不建 CUDA。
- 四门禁 fmt/clippy/clippy-cuda/unit 全过；证据 manifest 自核、UTF-8/LF/无 BOM。
- **非声明**：本卡只补 `capture.json` 的计数记录与 CPU 自证；不宣称采集有效性、不宣称判别力恢复、不宣称任何渲染数值结论；真实计数内容以 G-L1 真渲染参考为准。

## 7. 已知后续项（如实记录，不在本卡范围）

- 实体卡 `tests/support/render_bvh/light.rs::check_capture_c1` 目前只在 capture 记录值为**数组**（`[c1_0, c1_1]`）时做「实体 C1 == capture 记录」断言；本卡按 spec §2.4 写出的是 `{"0->1": {...}, "1->2": {...}}` 结构，因此该可选断言在 G-L2 会打印「交叉核对执行 = false」（显式跳过，不静默通过）。若要在 G-L2 落这条断言，需要一次实体卡 fix 适配新结构（本卡不动 `tests/`）。
