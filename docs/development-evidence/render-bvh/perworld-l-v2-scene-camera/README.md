# perworld L v2 场景/相机：`scene-a-light` 插入 `cam-hinge` + `.gitattributes` LF 规则（零 GPU）

- 任务：`2026-11-08-render-perworld-l-v2-scene-camera`（owner Backend、p60、P8-渲染与BVH）
- 依据：S-1 已签并生效（签名留证 `023c69fa`；spec v3 sha256 `bb064fe0a85ab38abba8043768666a0d9f73fc53d9f2f3dd72eace060c8a924d` / 39,379 B）§2.1 写死相机与插入位置。
- 边界：只改 `fixtures/render-bvh/scene-a-light/scene.xml` + `.gitattributes` + 本卡证据；不碰 `scene-a/`、`src/`、`tests/`、`scripts/`；零 GPU、不采集、不开窗、不建 CUDA、不落参考。

## 1. 改动（7 项记录）

| 项 | 改前 | 改后 |
| --- | --- | --- |
| `scene.xml` sha256 | `4db66e2c6c32f3e928baf475537127ef19904deeda90eb697f2f4e1587e7df0a` | **`4b354054116ddb81c6209ed10c021d133ea30d6fd82c1a6f866bafff9f625716`** |
| `scene.xml` bytes | 4,987 | **5,134**（+147） |
| `scene.xml` 行数 | 84 | **86**（+2） |
| 预测核对 | Chief 写死 `4b354054…`/5,134 B/86 行 | **实算逐项一致（predicted_match=true）** |
| `.gitattributes` | 无该子树规则 | 新增一行 `fixtures/render-bvh/scene-a-light/** text eol=lf`（置于既有 `scene-flex` LF 规则之后、`*.npz -text -diff` 之前，不改其他行） |
| 改动 diff 面 | — | `scene.xml` 仅 **+2 行**（`cam-hinge` 插在 `cam-persp` 之后、`cam-track` 之前）；`.gitattributes` 仅 **+1 行** |
| EOL 结论 | `git ls-files --eol` 改前：`i/lf w/lf attr/`（无规则） | 改后：`i/lf w/lf **attr/text eol=lf**`；工作树 LF；`git check-attr text eol` → `set` / `lf` |

插入文本与 spec v3 §2.1 逐字一致：
```xml
    <camera name="cam-hinge" pos="-0.0925 0.4981 0.8296" quat="-0.211088 -0.149763 0.558924 0.787792"
            fovy="30" resolution="320 240"/>
```

## 2. MuJoCo CPU 解析自检（pinned venv，零 GPU）

`artifacts/mjcf-parse.json`（改前模型从 `git show HEAD:scene.xml` 载入对照）：

| 项 | 值 |
| --- | --- |
| `ncam` | **3** |
| `cam_mode` | **`[0, 0, 3]`**（cam-persp / cam-hinge / cam-track） |
| `cam_fovy` | **`[73.73979462, 30, 40]`** |
| `cam_resolution` | **`[[320,240],[320,240],[240,180]]`** |
| `cam-hinge` `cam_sensorsize` | **`[0, 0]`**（走 fovy 路径，不声称有 intrinsics） |
| `cam-hinge` `cam_intrinsic` | **`[0.01, 0.01, 0, 0]`**（模型默认值） |
| `cam-hinge` `cam_pos` / `cam_quat` | `[-0.0925, 0.4981, 0.8296]` / `[-0.211088, -0.149763, 0.558924, 0.787792]`（float32 容差 1e-6） |
| 非相机**物理**数组 | **逐位不变**（`non_camera_changed=[]`） |
| 预期变化的记录面数组 | `name_camadr`/`name_lightadr`/`name_matadr`/`name_meshadr`/`name_texadr`/`names_map`（新增相机名导致名字表偏移，属装配簿记） |
| 预期变化的标量 | `ncam 2→3`、`nbuffer 108272→108656`、`nnames 193→203`、`nnames_map 48→50`、`signature`（模型签名随新增相机变化） |

## 3. dry-run 前后对照（零 GPU）

`capture_render_perworld.py --dry-run --scene fixtures/render-bvh/scene-a-light/scene.xml --out-root fixtures/render-bvh/scene-a-light/perworld --out-dir …/nw3-light-hinge02040 --worlds 3 --hinge-deg 0,20,40`：

- `artifacts/dryrun-diff.json`：**unexpected = 0**；仅 7 条预期变化——`/recipe/scene_xml/{sha256,bytes}`、
  `/environment/scene_xml/{sha256,bytes}`、`/environment/camera_modes` `[0,3]→[0,0,3]`、
  `/cam_active_fixed_only` `[true,false]→[true,true,false]`、`/model_static/cameras` 3 条；
- `legacy_manifest_entries`：改前/改后均 **112（不变）**；默认 dry-run 仍 **107**（该文件条目哈希变化属预期）；
- before/after 原始输出：`logs/before-dryrun.json` / `logs/after-dryrun.json`。

## 4. 历史旧 sha 记录（不回改）

- `4db66e2c…` 在 HEAD 的历史证据中 **8 处**（`artifacts/old-sha-references.txt` 逐条列出；另有本卡登记 JSON 1 处引用）；
  这些记录描述的是**改前形态**，本卡**一律不回改**；新 sha 为 `4b354054…`。

## 5. 门禁 / 边界 / 非声明

- 四门禁 fmt/clippy/clippy-cuda/unit 全过；`unchanged src/`、`unchanged tests/`、`unchanged scripts/`；
  未改 `fixtures/render-bvh/scene-a/`；不采集、不开窗、零 GPU、不建 CUDA、不落参考。
- **非声明**：本卡只让 fixture 与已签 spec v3 §2.1 对齐；**不宣称**判别力恢复、采集是否有效、per-world 灯验收、
  与上游实现等价、任何渲染数值结论；pilot/采集窗/验证窗另卡。

## 6. 证据清单

| 路径 | 说明 |
| --- | --- |
| `README.md` | 本文件 |
| `receipt.json` | 机器可读回执（`/task_id` 供 normify `json` 检查） |
| `artifacts/scene-identity.json` | 改前/改后 sha/bytes/行数、预测核对、EOL、旧 sha 计数、mjcf 摘要 |
| `artifacts/mjcf-parse.json` | MuJoCo 解析对照（相机字段/非相机数组/标量变化） |
| `artifacts/dryrun-diff.json` | dry-run 前后逐字段 diff（unexpected=0） |
| `artifacts/scene-camera.diff` | scene.xml + .gitattributes 改动 diff |
| `artifacts/old-sha-references.txt` | 历史旧 sha 记录 8 处（不回改） |
| `logs/before-dryrun.json` / `logs/after-dryrun.json` | 显式场景 dry-run 前后原始输出 |
| `logs/after-default-dryrun.json` | 默认 dry-run（legacy 107） |
| `logs/eol.txt` | `git ls-files --eol` + `git check-attr` 前后结论 |
| `manifest.txt` | 本目录 sha256 + bytes（首行 `#`，不自列） |

## 7. 复现

```powershell
target/mjwarp-parallel/venv/Scripts/python.exe -c "import mujoco; m=mujoco.MjModel.from_xml_path('fixtures/render-bvh/scene-a-light/scene.xml'); print(m.ncam, m.cam_mode.tolist(), m.cam_fovy.tolist())"
target/mjwarp-parallel/venv/Scripts/python.exe -B scripts/capture_render_perworld.py --dry-run --scene fixtures/render-bvh/scene-a-light/scene.xml --out-root fixtures/render-bvh/scene-a-light/perworld --out-dir fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040 --worlds 3 --hinge-deg 0,20,40
git ls-files --eol -- fixtures/render-bvh/scene-a-light/scene.xml
```
