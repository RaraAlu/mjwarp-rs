# R05-S3a hfield 参考采集 · 离线 authoring 材料（采集待窗口，未开窗）

任务：`2026-10-28-render-bvh-r05-s3a-hfield-capture`（P8 渲染与 BVH；p58；owner Backend）
阶段：**离线 authoring 完成，采集未执行**（未开 GPU 窗、未产生任何渲染数值结论）。
本片只采集、不比数；数值验收全部留 `2026-10-28-render-bvh-r05-s3b-hfield-validate`。

> 说明：新脚本由 `scripts/capture_render_shadow.py`（R09-S3a，sha256
> `7c04809c6558eb8207d35c03c7b8777985b36d733eb711020910154bc6c18415`，477 行）**派生**；
> R09 脚本与 R09 参考（`scene-shadow/capture/`、`capture-exact/`）**零改动**，原脚本 sha 未变。

## 1. 捕获脚本

| 项 | 值 |
| --- | --- |
| 路径 | `scripts/capture_render_hfield.py` |
| sha256 | `cb1808f8c185df0e4695bb7107ec941c2e9e0aa8bb883fa1f630dc029d28a599` |
| 字节 | **20 314** |
| 行数 | **425**（LF 计数；新文件 500 行上限内；首版 566 行被 commit 钩子拒绝后压缩，逻辑不变、dry-run 复跑通过） |
| ASCII-only 自检 | **非 ASCII 字节 0**（本脚本非 `.ps1`，无需 ANSI/UTF-8 行数比对；仍做了 ASCII 专项自检） |
| 语法自检 | pinned python `ast.parse` → OK（`PY_SYNTAX_OK`） |
| dry-run | 两组各跑一次 `--dry-run`，exit 0（见 `materials/dryrun-*.json`） |

与 R09 脚本的派生差异（R09 脚本零改动）：`TASK_ID`/`SCENE_DIR`/`DEFAULT_CACHE` 改为 hfield 任务；
新增 `--group {hfield-only,hfield-primitive}`（选 `scene.xml`/`scene-mixed.xml`）与 `--use-shadows`
（默认 **true**，让 hfield 阴影路径被覆盖）；`--use-fast-math` **默认 false 不变**；`capture.json`
新增 `recipe` 块（npz 相对路径、scene.xml sha256、相机标识/位姿/分辨率、脚本自身 sha256、
fast-math/shadows 标志、pinned python/numpy 版本、upstream revision、`MJWARP_CAPTURE_AT`）；
`legacy_manifest()` 排除 `scene-shadow/` 与 `scene-hfield/`；out-dir 守卫收紧为「必须在
`fixtures/render-bvh/scene-hfield/` 子树内」。

## 2. 场景 fixture（`fixtures/render-bvh/scene-hfield/`）

| 文件 | sha256 | 字节 | 行数 | 组 |
| --- | --- | --- | --- | --- |
| `scene.xml` | `b3b9a6b4ee4844c92edec3f217d04b7e788a69f9ab1e864fe7dac765e9c6f53b` | 3 590 | 63 | hfield-only |
| `scene-mixed.xml` | `bf9bb966887466d1f526605875df3f9b8c6b244787e7f95be2eccc3cd3329bdf` | 3 180 | 60 | hfield-primitive |

几何/相机/光照口径（MuJoCo 3.15.0 CPU 解析核对，无 GPU）：

- **hfield**：`nrow=4, ncol=4`；`size="1.5 1.0 0.25 0.1"` ⇒ 水平范围 x∈[−1.5,1.5]、y∈[−1.0,1.0]，
  `size[2]=0.25 ≠ 0`；geom `type=hfield`、`pos="0 0 0"`、`quat="1 0 0 0"`、`group=1`、贴图 `mat-terrain`
  （`_texture_hfield` 着色路径同时被覆盖）。
- **elevation 设计（转置/翻转可变，Arch 复核要求）**：XML 行主序网格 =
  `0.20*r + 0.35*c`（**行、列系数不同**）+ 非对角扰动 `(r=2,c=1)` 抬高到 1.05（+0.30）。
  这样「行/列约定错误」会在有效网格上表现为**不同的地形**（而非对称掩盖）；4×4、`size[2]≠0` 不变，
  扰动邻接的 4 格非共面、其余 5 格共面（合并且不合并两种形态仍在）。
- **elevation 存储约定（实测，纠正旧稿）**：MuJoCo 加载 MJCF elevation 时 ①归一化
  `(v−min)/(max−min)` **全程 f32 算术**；②行序**上下翻转**（image-origin 约定）；③按行主序连续存放。
  上游 `bvh.py::build_hfield_bvh` 用 `reshape((nr, nc))` 行主序读 flat ⇒
  **effective 网格 = flipud(归一化 XML 行主序网格)**（不是转置；旧稿「列主序⇒转置」已否证）。
  位级候选比对见 `materials/reverse-calc.txt` [2]：`norm_f32_math+flipud` **0 处不匹配**，
  `norm_f64_then_f32+flipud` 不匹配 2 处，其余 12–16 处。
- **cam0（俯视）**：`pos="0 0 3.4"`、`xyaxes="1 0 0 0 1 0"`（光轴 = −z）、`fovy=45`、
  `resolution="320 240"`；覆盖校验（CPU 投影，z=0 与 z=0.25 两个高度）：最坏 y 比 **0.710/0.766**、
  最坏 x 比 **0.799/0.862**（<1 ⇒ 全网在视野内）。
- **cam1（侧视）**：`pos="3.4 -2.6 1.8"`、`xyaxes="0.6069 0.7936 0 -0.2857 0.2185 0.9330"`、
  `fovy=40`、`resolution="240 180"`；朝向场地中心（前向 ≈(−0.741,0.567,−0.360)）。
- **primitive 遮挡（仅 mixed）**：静态 `body pos="0.62 -0.47 0.45"` + `box size="0.14 0.14 0.14"`
  （`group=2`）；cam1→场地中心视线到盒心最近距离 **0.0194 < 0.14** ⇒ 遮挡成立；盒子 z 底 0.31 > 地形
  最高点 0.25 ⇒ 不穿插。
- **光照**：`sun`（directional，castshadow）+ `lamp`（point，castshadow），`light_active` 全真；
  两组 `use_shadows=true` 覆盖 hfield 阴影透传路径（S2 设备侧阴影不剔背逻辑）。

## 3. pinned 环境（`--dry-run` 实测，CPU import，未创建 GPU device）

| 项 | 值 |
| --- | --- |
| python | `target/mjwarp-parallel/venv/Scripts/python.exe`，sha256 `feeec36120097a03201a96ae17a0e2c1fba70dd6241cb4bfdd9e7c2a00bada77`，**3.12.13** |
| warp / mujoco / mujoco_warp / numpy | **1.15.0 / 3.15.0 / 3.15.0 / 2.5.4** |
| upstream | `71da24d956378a87a703b6e1442b13aec0c4ac29`，`git status --porcelain` 空（clean） |
| GPU / driver / arch | RTX 4070 Ti SUPER / **596.36** / compute_cap **8.9**（arch 89） |
| platform | Windows-11-10.0.26200-SP0 |

## 4. 两组输出与判定口径

- 输出目录（新场景子树内，不写 R09）：
  - `fixtures/render-bvh/scene-hfield/capture-hfield/hfield-only/`
  - `fixtures/render-bvh/scene-hfield/capture-hfield/hfield-primitive/`
- 每组文件数：**4**（`rgb.npz`、`depth.npz`、`seg.npz`、`capture.json`）。
- 预体量（npz 为 `ZIP_STORED` 不压缩；按形状精确算）：
  rgb cam0 320×240×3×f32=921 600 B、cam1 240×180×3×f32=518 400 B；depth 307 200 + 172 800 B；
  seg 614 400 + 345 600 B ⇒ **npz 合计 2 880 000 B/组** + npy/zip 头（≈1 KB）+ capture.json（≈5–20 KB）；
  **两组预计 ≈ 5.8 MB**。
- **A/B 两次逐字节一致判定口径**（脚本内置，`--repeat 2` 默认）：同进程渲染两次 → ①逐数组
  `np.array_equal`；②两份 npz 按确定性 zip（固定 1980-01-01 时间戳、`ZIP_STORED`、数组按名排序）
  写盘后 sha256 必须相同；③`capture.json` 内 `two_captures_bitwise_identical=true`；
  否则脚本非零退出。
- 格式沿用 R09-S3a：`rgb.npz` 存 `rgb_cam0..1`（f32 [H,W,3]）、`depth.npz` 存 `depth_cam0..1`
  （f32 [H,W]，0=无命中）、`seg.npz` 存 `seg_cam0..1`（i32 [H,W,2]，背景 (−1,−1)）；
  `capture.json` 保留 R09 字段语义并新增 `group`（采集组）与 `recipe`（见 §5）。

## 5. recipe 可打印且可见

- `capture.json` 根新增 `"recipe"`：`task_id/group/scene_xml{path,sha256,bytes}/script{path,sha256,bytes}/
  npz_files/use_fast_math/use_shadows/seed/repeat/python/versions/upstream_revision/capture_at_env/cameras[{slot,
  mujoco_cam_id,name,xpos,xquat,fovy,resolution}]`。
- `use_fast_math` 出现点：`recipe.use_fast_math`、`parameters.use_fast_math`、`context.parameters.use_fast_math`、
  `context.resolved.use_fast_math`（4 处一致，**默认 false**）。
- `MJWARP_CAPTURE_AT` 固定值（两组共用、开窗前写入命令环境）：`2026-10-29T00:00:00+08:00`
  （若窗口实际日期变化，只在开窗前改一次并两组同用；`recipe.capture_at_env` 与 `captured_at` 记录）。

## 6. 旧参考零读写守卫（采集前后各采一次）

- 范围：`fixtures/render-bvh/` 除 `scene-shadow/`、`scene-hfield/` 外。
- 基线（本轮现算，与 R09-S3b 窗口签名口径一致）：**38 文件 / 25 320 515 B / 聚合
  `b5cb50afcc608a67ca9a7b83222029402719f2770a3510ab423df13601e50d8b`**。
- 聚合算法（R09 窗口口径）：38 个文件 sha256 十六进制串**按路径排序后直接拼接**再 sha256
  （字节直接拼接 = `b21b8b02…` 仅作旁证）。逐文件清单：`materials/legacy-baseline.txt`。
- 脚本内置：采集前 `legacy_before = legacy_manifest()`、采集后再次采集并比对，
  `capture.json.legacy_reference_untouched` 必须为 true，否则非零退出。

## 7. 只读不动承诺

`fixtures/render-bvh/scene-shadow/`（含 `capture/`、`capture-exact/`）与既有夹具、R09 脚本
`scripts/capture_render_shadow.py` 均只读；本片新增文件只落在 `scene-hfield/` 子树与
`docs/development-evidence/render-bvh/r05-s3a-hfield-capture/`。

## 8. GPU 基线 / floor / wall / attempt

现采 3 点（`nvidia-smi`，只读；仅规划用，非开窗证据；见 `materials/gpu-baseline.txt`）：

| 采样 | free MiB | total MiB | util % |
| --- | --- | --- | --- |
| 23:25:17.679 | 13 290 | 16 376 | 35 |
| 23:25:19.731 | 13 290 | 16 376 | 36 |
| 23:25:21.776 | 13 290 | 16 376 | 39 |

- **floor = min(10240, baseline_free − 3072) = min(10240, 13290 − 3072 = 10218) = 10218 MiB**
- compute-apps 原始 32 条（全部 GUI/显示进程）；按窗口过滤 pattern
  （`python|g10_p5_|product_collision|stage2_equivalence|cargo|rustc|nvcc|cl\.exe|render_`）**过滤后 = 0**
- 构建进程（cargo/rustc/cl/nvcc/link）**= 0**；存在 4 个 python 进程但均不在 compute-apps 列表 ⇒ 非 GPU 作业
- **wall 预估 = 240 s 合计（每组 ≤120 s：模型加载 + 2 次重复 × 2 相机，内核缓存已预热）**
- **attempt = 1**（本材料第 1 次采集窗）
- 注意：以上只是规划数字；真正开窗仍须先做 ≥5 分钟干净监测（free≥10240、util<40、
  filtered compute-apps 0、构建进程 0）并完成 Arch→Chief 双签。

## 9. 捕获命令（逐字；pinned python；两组分开跑、同一窗口）

```powershell
# 组 1：hfield-only
$env:MJWARP_CAPTURE_AT = "2026-10-29T00:00:00+08:00"
target\mjwarp-parallel\venv\Scripts\python.exe -B scripts/capture_render_hfield.py `
  --group hfield-only `
  --out-dir fixtures/render-bvh/scene-hfield/capture-hfield/hfield-only

# 组 2：hfield-primitive
$env:MJWARP_CAPTURE_AT = "2026-10-29T00:00:00+08:00"
target\mjwarp-parallel\venv\Scripts\python.exe -B scripts/capture_render_hfield.py `
  --group hfield-primitive `
  --out-dir fixtures/render-bvh/scene-hfield/capture-hfield/hfield-primitive
```

（`--use-fast-math` 默认 false、`--use-shadows` 默认 true，无需显式传；如需显式可加
`--use-fast-math false --use-shadows true`，recipe 会记录实际值。）

## 10. 窗口协议（脚本内置 + 窗口侧）

- 脚本内置：采集前后 legacy 快照比对、A/B 两次逐字节一致、非零退出即无效；
- 窗口侧（沿用 R09-S3b 协议）：开窗前最后构建 → 冻结脚本/场景/runner/exe（记 sha256）→
  guard precheck 为唯一硬门 → 监测 ≥5 分钟干净 → 签名后任一实体变动作废重签 →
  窗内 A/B 重采一致 → 窗口产物与 exit 码、`WINDOW_RUN_DONE` 全部留档；
  **exit 9 等无效轮按协议留档、不当作有效轮**。

## 11. 只采集不比数

本片**不产生任何渲染数值结论/判据**：不写 RGB/depth/seg 差异数字，不宣称任何正确性；
数值验收（RGB ≤1 LSB 位级优先 / depth ≤4 ULP 位级优先 / seg exact / 两次 sha 相同）
全部在 S3b 验证窗按既有判据执行，不新增、不放宽。

## 12. 有效网格反算 + 转置敏感性 + O1 边界（f32 口径）

**数据来源**：pinned python（mujoco 3.15.0 + numpy 2.5.4）加载 `scene.xml` 得到的 `mjm.hfield_data`；
**先定约定**再用上游 `reshape((nr,nc))` 得到 effective 网格（详见 `materials/reverse-calc.txt`）。

- 原始 flat（f32 位型，内存顺序）：
  `['0x3eba2e8c','0x3f1364d9','0x3f49b26c','0x3f800000','0x3e783e10','0x3f22e8ba','0x3f2aaaab','0x3f60f83f','0x3df83e10','0x3eaaaaab','0x3f0ba2e9','0x3f41f07c','0x0','0x3e59364e','0x3ed9364e','0x3f22e8ba']`
- 布局候选位级比对：**`norm_f32_math+flipud` 0 处不匹配**（唯一匹配）；`norm_f64_then_f32+flipud` 不匹配 2 处；
  identity/fliplr/rot180/transpose 系列 12–16 处 ⇒ 约定 = **f32 归一化 + 行序上下翻转 + 行主序存放**。
- effective 网格（= `reshape((nr,nc))` 行主序读 flat）：
  ```
  [[0.36363637 0.57575756 0.78787875 1.        ]
   [0.24242425 0.6363636  0.6666667  0.87878793]
   [0.12121212 0.33333334 0.54545456 0.75757575]
   [0.         0.21212122 0.42424244 0.6363636 ]]
  ```
- 阈值：`f32(1e-5)` 位型 **`0x3727C5AC`**；逐格 `diff=(z00+z11)−(z01+z10)`（全 f32）：

| 格 (r,c) | diff | 位型 | 共面？ |
| --- | --- | --- | --- |
| (0,0) | +0.181818187 | `0x3E3A2E8C` | 否 |
| (0,1) | −0.181818128 | `0x3E3A2E88` | 否 |
| (0,2) | 0 | `0x00000000` | 是 |
| (1,0) | −0.181818128 | `0x3E3A2E88` | 否 |
| (1,1) | +0.181818247 | `0x3E3A2E90` | 否 |
| (1,2) | 0 | `0x00000000` | 是 |
| (2,0) | 0 | `0x00000000` | 是 |
| (2,1) | 0 | `0x00000000` | 是 |
| (2,2) | 0 | `0x00000000` | 是 |

- **没有任何格 |diff| 等于阈值**（`any_exact_threshold=False`）；`min ||diff|−threshold| = 9.999999747378752e-06`
  （来自 |diff|=0 的共面格）。`fits_plane` 预测/斜率残差（9 条）：max **5.96046448e-08**、**0 处命中阈值**。
- 三角化形态（O1 修复后 f32 口径）：4 个非共面单格 + 5 个共面格合并 ⇒ **13 顶点 / 36 索引 / 12 三角形**；
  顶点坐标 checksum `f3e162dd04949f5d5948b89350e51f70bc3f4e0de1a2f20fab104576d0c8853a`。
- **转置敏感性证据**（证明夹具可区分行/列约定）：对 effective 网格**转置**后跑同一套三角化 →
  顶点/索引/三角数仍为 13/36/12，但**坐标 checksum = `b83e93a680d46d7d8be4dc2fe1bc86efad18715b377daa267a15dec209e50fb0` ≠ 原值**
  ⇒ `transpose_distinguishable=True`。行/列被调换会得到不同地形→不同像素，S3b 不会因对称而放过。
- 复现：`target\mjwarp-parallel\venv\Scripts\python.exe -B materials\reverse-calc.py`（输出全文 `materials/reverse-calc.txt`）。
- 取数顺序对拍（python `mjm.hfield_data` ↔ 原生探针 JSON ↔ our host 路径）见 `materials/order-crosscheck.txt`：
  ①约定实测如上；②原生探针字段是 `m->hfield_data` 原样拷贝（`native/g01_fields.inc:78`、
  `native/model_probe_collision_inputs.cpp:97`），host `hfield_data()` 直传、`triangulate_hfield` 行主序读
  ⇒ 与上游同构；③对称旧夹具位级对拍一致（只证数值/归一化、不证顺序）；④非对称运行时对拍未做
  （需构建原生探针，离线 CPU），已列为 **S3b 停手条件**：若出现镜像/转置形差异立即停手报 Arch。
## 13. normify 前置状态

- 脚本 + 两个场景 XML 已落地；证据骨架（README/receipt/materials/manifest）就绪；
- **未执行** `normify done`（采集与窗口材料未完成）；**未提交任何窗口产物**（尚未采集）；
- 本片后续：Arch 核材料 → 转 Chief 双签 → 开窗采集（2 组）→ 窗口材料入证据 → 两段提交/收口；
  S3b 验证窗另起材料单独签。

## 遗留（带入 S3b，来自 S2 O5）

`SceneLeaf.size` 必须与三角化产出的 `half` **同源**（不得 host 侧重推）：S3b 目标实体构造
SceneState 时应先 `build_hfield_mesh(...)` 再取 `half()` 填 `SceneLeaf.size`；fixture 侧保证。
