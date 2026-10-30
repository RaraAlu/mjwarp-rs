# SDF 渲染参考采集 authoring（U2；离线 dry-run，不跑 GPU、不写参考）

任务：`2026-10-30-render-sdf-capture-authoring`（SDF 解锁片 U2；预核 `target/sdf-unlock-precheck.md` §8）
前置：`2026-10-30-render-sdf-mesh-path`（U1，设备 mesh 路径对齐，verified）
后续：U3 采集窗（GPU，**需用户双签**）→ U1b host 解锁 → U4 验证窗（GPU，需双签）

## 0. 一句话

上游渲染 SDF = 渲染其底层 mesh（`render_test.py:1251`、`render.py:738-761`），因此参考采集**不需要任何 SDF 距离场代码**：
本片只做新场景 fixture + 派生采集脚本 + 窗口材料，dry-run 已验证（零 CUDA device、零盘写）。

## 1. 交付物与身份

| 项 | 值 |
| --- | --- |
| 场景 `fixtures/render-bvh/scene-sdf/scene.xml`（group `sdf-mesh`） | **3,388 B / 59 行 / sha256 `376eb81ce200b29373af3965c7abefd90b8f00188a688db583080836dfa868e8`** |
| 场景 `fixtures/render-bvh/scene-sdf/scene-mixed.xml`（group `sdf-occluder`） | **3,062 B / 55 行 / sha256 `7869cfae5e16795ebf86b93c931ef36a8e94464452762e28f1357e0e68a1584e`** |
| 采集脚本 `scripts/capture_render_sdf.py` | **20,380 B / 427 行（≤500）/ 非 ASCII 0 / sha256 `9acd2130f58cfc2cc495bc73bae4e12cb1e9fb3c68c4d245819e809e524fde67`** |
| 派生来源 | `scripts/capture_render_hfield.py`（R05-S3a 骨架）：只换场景子树/组名/task id/消息；环境 pin、确定性 npz、两跑字节相等、legacy sha 清单全部沿用；派生器 `target/sdf-u2/derive_sdf_capture.py`（不入库），缺失片段会直接失败以防静默漂移 |
| pinned 环境 | python `feeec361…`（3.12.13）、warp 1.15.0 / mujoco 3.15.0 / mujoco_warp 3.15.0 / numpy 2.5.4、upstream `71da24d9…`（clean） |

## 2. 场景设计（为什么这样写）

- SDF geom：`<geom name="sdf-cube" type="sdf" mesh="cube" material="mat-sdf" contype="0" conaffinity="0"/>` —— **volume SDF、无 plugin**（上游 `_SDF_VOLUME` 同写法），只用于渲染；`contype/conaffinity=0` 保证没有任何碰撞对进入 SDF narrowphase（`collision_sdf` 不参与）。
- 底层 mesh `cube`：8 顶点 / 12 三角形、外向绕序（`face` 显式列出），半尺寸 0.35；带 checker 纹理材质 ⇒ 纹理与顶点法线分支也可观测。
- 另含 plane floor + 一个非 SDF sphere（遮挡/合并/seg 场景）；**2 个相机**（`cam-front` 320×240 透视、`cam-top` 240×180 俯视）+ **2 个 castshadow 灯**（sun 方向光 + lamp 点光）⇒ 阴影 any-hit 路径也会打到 SDF。
- `scene-mixed.xml` 额外加一个静态 box occluder 在 `cam-front` 视线内（首命中排序/depth/seg）。
- **静态校验（CPU-only，pinned MuJoCo）**：`logs/scene-check.json` —— ngeom 3/4、nlight 2、ncam 2、nq 0、nmesh 1（8 顶点/12 面，bounds ±0.35）；SDF geom `type=8`、`dataid=0`（指向 cube mesh）、`contype=conaffinity=0`；两个场景均通过断言（恰好 1 个 SDF geom、dataid 合法、非碰撞、mesh 存在）。
- 注意：MuJoCo 对 SDF geom 会把 `geom_size` 填成底层 mesh 半尺寸（本 fixture 实测 `[0.35,0.35,0.35]`），与渲染 bounds 一致；上游 `render_test.py` 里 tetra 的 `[1,1,1]` 是其 mesh 归一化的结果，不能当通用假设。

## 3. dry-run（离线；两组合各一次）

```text
target\mjwarp-parallel\venv\Scripts\python.exe -B scripts/capture_render_sdf.py --dry-run --group sdf-mesh     --out-dir fixtures/render-bvh/scene-sdf/capture-sdf/sdf-mesh
target\mjwarp-parallel\venv\Scripts\python.exe -B scripts/capture_render_sdf.py --dry-run --group sdf-occluder --out-dir fixtures/render-bvh/scene-sdf/capture-sdf/sdf-occluder
```

两次均 **exit 0**；日志 `logs/dryrun-sdf-{mesh,occluder}.json`。要点：
- `dry_run=true`、`repeat=2`、`seed=20261030`、`use_fast_math=false`、`use_shadows=true`、`samples_per_pixel=1`
- **未创建 CUDA device**（dry-run 分支 `want_cuda_device=False`），**未写 --out-dir**（实测 `fixtures/render-bvh/scene-sdf/capture-sdf/` 不存在，目录下只有两个 xml）
- `legacy_manifest_entries=69`（本脚本 legacy 清单 = `fixtures/render-bvh` 排除新 `scene-sdf/` 子树；**包含** shadow/hfield 的既有参考，它们冻结不动，任何改动都会让采集失败——比只排除 shadow/hfield 更严）
- 环境/版本/upstream clean/python sha 与 receipt 一致

## 4. 预期产物（U3 采集窗将写入；本片不写）

`fixtures/render-bvh/scene-sdf/capture-sdf/<group>/`：
- `rgb.npz`（`rgb_cam0` `[240,320,3]` f32、`rgb_cam1` `[180,240,3]` f32）
- `depth.npz`（`depth_cam0` `[240,320]`、`depth_cam1` `[180,240]` f32，raw planar，0=无命中）
- `seg.npz`（`seg_cam0` `[240,320,2]`、`seg_cam1` `[180,240,2]` int32，背景 (-1,-1)）
- `capture.json`：`task_id`、`group`、`kind=render-sdf-capture`、`two_captures_bitwise_identical`、`files_sha256`、`captured_at`、`environment`、`recipe`（含场景 sha）、`parameters`、`legacy_reference_untouched`、`legacy_reference_files`、`context`/`context_repeat`
（尺寸以脚本实测为准；`rgb` 总量估算 ≈ 320×240×3×4 + 240×180×3×4 ≈ 1.4 MB/组）

## 5. 边界与不宣称

- 本片**零 GPU、零窗口、零设备执行、零参考写入**；不改 `src/`、`tests/`、既有采集脚本/参考，不改判据/容差。
- 不宣称任何渲染数值结论；SDF 数值验证只在 U4 验证窗（需用户双签）之后成立。
- U3 采集窗命令逐字与基线 SOP 见 `docs/development-evidence/render-bvh/sdf-capture-window-material/`。
