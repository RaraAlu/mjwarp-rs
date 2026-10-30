# SDF 渲染参考采集窗（U3；authoring-only util<60 例外；2 groups）

任务：`2026-10-30-render-sdf-capture`（SDF 解锁片 U3）
前置：`2026-10-30-render-sdf-capture-authoring`（U2，verified）；脚本 `scripts/capture_render_sdf.py` sha `80645968…`
签字：常规窗口计划 user via Chief **2026-10-30T23:10:52+08:00**；**authoring-only `util<60` 例外 user via Chief 2026-10-31T00:00+08:00**（同一份签字覆盖本 SDF 采集窗与 per-world 采集窗）
性质：**采集，不是验收**——本窗不产生任何渲染数值判定；数值验证在 U4（U1b host 解锁后）。

## 1. 窗口事实

| 项 | 值 |
| --- | --- |
| shell | **Windows PowerShell 5.1.26100.7920**（`aggregate_sha256` 的 `Sort-Object` 文化排序约束；本窗不产 aggregate，仍统一用 5.1） |
| 常规 monitor（失败，原样保留） | `monitor/monitor-pre-window-regular-failure.csv`：354 samples / **longest_clean=21/60** / util 35–50（84 个 ≥40）/ free_min 13,388 / floor 10,445 未破 / filtered=0 / build=0 / exit 20，`23:12:30–23:42:26` |
| 例外基线（重取） | `monitor/pre-window-snapshot-exception.txt`：min free **13,201** ⇒ `floor = 13,201 − 3,072 = 10,129`；filtered=0、build=0、树干净 |
| 例外 monitor（生效） | `monitor/monitor-pre-window-attempt2-exception.csv`：**60/60**、util 30–42、free_min **13,171**、ge60=0、filtered=0、build=0、exit 0，`00:00:50–00:05:51` |
| 采集命令 | 两条，各一次（`--use-fast-math false --use-shadows true --repeat 2`）：`sdf-mesh` → 00:06:08、`sdf-occluder` → 00:06:22，均 exit 0 |
| 零编译 | 窗口内未跑 cargo/rustc（只有 monitor + 两条 pinned python 采集） |
| 失败留档 | 常规失败 CSV + 快照 + `monitor/monitor-failure-regular.txt` 原样保留；**未被覆盖** |

## 2. 例外生效证据（机器级，独立重算）

`monitor/exception-monitor-recompute.json`（`target/recompute-exception-monitor.py`，pinned python）：

- `samples=60`、`longest_clean=60`、`ready=true`、`exit=0`
- **`ge40_samples=17`、`ge40_clean_samples=17`** ⇒ 17 个 util≥40 的样本被记为 `clean=True` 且 streak 继续到 60 ⇒ **生效门槛是 60 而非 40**（`exception_load_bearing=true`）
- `ge60_samples=0`、`util_max=42 < 60`、`free_min_mib=13,171 ≥ floor 10,129`、`compute_filtered_max=0`、`build_max=0`
- 口径：`not_a_normal_threshold_pass=true`、`authoring_exception_used=true`、`signature_unchanged=true`；常规失败记录原样保留，不写成"常规通过"

## 3. 采集产物（两 group，各 5 文件 → 实为 4 文件/组）

见 `materials/products.txt`（path/bytes/sha256/mtime）。摘要：

| group | rgb.npz | depth.npz | seg.npz | capture.json |
| --- | --- | --- | --- | --- |
| `sdf-mesh` | 1,440,478 B / `7ca2abd6…` | 480,486 B / `3415b796…` | 960,478 B / `afd5eacd…` | 10,547 B / `d45c9ef9…` |
| `sdf-occluder` | 1,440,478 B / `598044fe…` | 480,486 B / `5515305d…` | 960,478 B / `99ed0398…` | 10,567 B / `08e897c8…` |

`capture.json` 关键字段（两组一致/按组变化）：`task_id=2026-10-30-render-sdf-capture`、`kind=render-sdf-capture`、`group`、**`two_captures_bitwise_identical=true`**、**`legacy_reference_untouched=true`**、**`legacy_reference_files=69`**、`use_fast_math=false`、`use_shadows=true`、`repeat=2`、`recipe.scene_xml.sha256`（`376eb81c…` / `7869cfae…`）、`context.resolved.nrender=2`、`recipe.cameras=2`；`captured_at` 为空串（未设 `MJWARP_CAPTURE_AT`，脚本允许）。数组结构：`rgb_cam0 [240,320,3] f32`、`rgb_cam1 [180,240,3] f32`、`depth_cam0 [240,320]`、`depth_cam1 [180,240]`、`seg_cam0 [240,320,2] i32`、`seg_cam1 [180,240,2] i32`。

## 4. 边界与不宣称

- 只写 `fixtures/render-bvh/scene-sdf/capture-sdf/` 与本证据；未改 `src/**`、`tests/**`、场景 fixture、采集脚本（`unchanged` 检查钉住）；判据/容差/参考定义未改。
- 不宣称任何渲染数值结论、不宣称 SDF 渲染能力已覆盖；本窗产物是 U4 的输入参考。
- 后续：**U1b host 解锁**（U3 verified 后、U4 前；U4 未过则回滚 U1b、保留 U1）→ **U4 验证窗**（新 target + 8 回归，新冻结 guards，旧 exe 哈希全部失效不复用）。
