# R05-S3a hfield 参考采集 · 窗口执行记录（采集完成，未收口）

任务：`2026-10-28-render-bvh-r05-s3a-hfield-capture`（P8 渲染与 BVH；p58；owner Backend）
阶段：**离线 authoring + GPU 采集窗（attempt=1）已完成**；本片只采集、不比数：**零渲染数值结论**，数值验收全部留 `2026-10-28-render-bvh-r05-s3b-hfield-validate`。
窗口说明：attempt1–3 常规门槛（`util<40`）均未达标并停手；attempt4 为 **Chief 预批的 authoring-only 例外（`util<60`）**，达标后开窗采集（详见 §14）。**attempt4 不记为常规门槛通过，原始 `util<40` 失败记录保留。**

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

- 输出目录（新场景子树内，不写 R09；窗口已按此路径落盘，见 §15）：
  - `fixtures/render-bvh/scene-hfield/capture-hfield/hfield-only/`
  - `fixtures/render-bvh/scene-hfield/capture-hfield/hfield-primitive/`
- 每组文件数：**4**（`rgb.npz`、`depth.npz`、`seg.npz`、`capture.json`）。
- 实测体量：每组 npz 合计 **2 880 000 B**；capture.json **10 679 / 10 701 B**；8 文件合计 **5 784 264 B**。
- **A/B 两次逐字节一致判定口径**（脚本内置，`--repeat 2` 默认）：同进程渲染两次 → ①逐数组
  `np.array_equal`；②两份 npz 按确定性 zip（固定 1980-01-01 时间戳、`ZIP_STORED`、数组按名排序）
  写盘后 sha256 必须相同；③`capture.json` 内 `two_captures_bitwise_identical=true`；
  否则脚本非零退出。窗口实测两组均为 **true**。
- 格式沿用 R09-S3a：`rgb.npz` 存 `rgb_cam0..1`（f32 [H,W,3]）、`depth.npz` 存 `depth_cam0..1`
  （f32 [H,W]，0=无命中）、`seg.npz` 存 `seg_cam0..1`（i32 [H,W,2]，背景 (−1,−1)）；
  `capture.json` 保留 R09 字段语义并新增 `group`（采集组）与 `recipe`（见 §5）。

## 5. recipe 可打印且可见

- `capture.json` 根新增 `"recipe"`：`task_id/group/scene_xml{path,sha256,bytes}/script{path,sha256,bytes}/
  npz_files/use_fast_math/use_shadows/seed/repeat/python/versions/upstream_revision/capture_at_env/cameras[{slot,
  mujoco_cam_id,name,xpos,xquat,fovy,resolution}]`。
- `use_fast_math` 出现点（窗口产物实测，全部 false）：`recipe.use_fast_math`、`parameters.use_fast_math`、
  `context.parameters.use_fast_math`、`context.resolved.use_fast_math`，另加根 `use_fast_math` 与
  `context_repeat.parameters/resolved`（共 7 处，0 处 true）。
- `MJWARP_CAPTURE_AT` 固定值（两组共用、开窗前写入命令环境）：`2026-10-29T00:00:00+08:00`
  （窗口实测两组 `captured_at` 同值）。

## 6. 旧参考零读写守卫（采集前后各采一次）

- 范围：`fixtures/render-bvh/` 除 `scene-shadow/`、`scene-hfield/` 外。
- 基线（本轮现算，与 R09-S3b 窗口签名口径一致）：**38 文件 / 25 320 515 B / 聚合
  `b5cb50afcc608a67ca9a7b83222029402719f2770a3510ab423df13601e50d8b`**。
- **计算基准（口径限制，务必与 committed blob 区分）**：逐文件 sha/bytes 与上述聚合均按**工作树字节**
  计算；`core.autocrlf=true` 下文本文件在工作树为 CRLF（例：`fixtures/render-bvh/scene-e2e/camlight-model.json`
  工作树 **6 434 B** vs committed blob **6 020 B**）。外部若按 committed blob（LF）复算，逐文件 sha 与聚合
  都会 mismatch——这是记录口径差异，不是旧参考被动过。
- **聚合算法与排序依赖**：38 个文件按**仓库相对路径**排序（排序由 PowerShell `Sort-Object` 完成，可能受
  运行环境文化/排序规则影响）→ 各文件小写 sha256 hex 串**直接拼接** → sha256。因此该聚合值依赖 PowerShell
  路径排序口径，不能只按「blob 重算」复核。
- **旁证（R09 行拼接口径）**：同 38 文件按 `sha256 bytes path` 行拼接再 sha256 =
  `983617f16b534f9cda743ff2b84bfed08e0154e88c369419b885bacb841cc260`（与 R09-S3a
  `facts.txt` / `legacy-manifest-after.txt` 记录值一致，交叉印证）；字节直接拼接 = `b21b8b02…`（仅旁证，非判据）。
  逐文件清单：`materials/legacy-baseline.txt`。
- 脚本内置：采集前 `legacy_before = legacy_manifest()`、采集后再次采集并比对，
  `capture.json.legacy_reference_untouched` 必须为 true，否则非零退出。窗口实测两组均为 **true**；
  采集窗口后复算旧参考仍为 **38 / 25 320 515 / `b5cb50af…`**（见 §16）。

## 7. 只读不动承诺

`fixtures/render-bvh/scene-shadow/`（含 `capture/`、`capture-exact/`）与既有夹具、R09 脚本
`scripts/capture_render_shadow.py` 均只读；本片新增文件只落在 `scene-hfield/` 子树与
`docs/development-evidence/render-bvh/r05-s3a-hfield-capture/`。窗口后复核：`scene-shadow` 16 个采集
产物（11 561 363 B）逐文件 sha 未变、`git status --short -- scripts fixtures/render-bvh/scene-shadow
fixtures/render-bvh/scene-a fixtures/render-bvh/scene-e2e` 为 0 行。

## 8. GPU 基线 / floor / wall / attempt（规划值，历史记录）

现采 3 点（`nvidia-smi`，只读；**规划期**数字，见 `materials/gpu-baseline.txt`）：

| 采样 | free MiB | total MiB | util % |
| --- | --- | --- | --- |
| 23:25:17.679 | 13 290 | 16 376 | 35 |
| 23:25:19.731 | 13 290 | 16 376 | 36 |
| 23:25:21.776 | 13 290 | 16 376 | 39 |

- **floor = min(10240, baseline_free − 3072) = 10 218 MiB**（规划时）；实际开窗瞬间重算值为 **10 196 MiB**（§14 gate4）。
- compute-apps 原始 32 条（规划期）/ 30 条（窗口期，全部桌面图形上下文）；按窗口过滤 pattern
  （`python|g10_p5_|product_collision|stage2_equivalence|cargo|rustc|nvcc|cl\.exe|render_`）**过滤后 = 0**
- 构建进程（cargo/rustc/cl/nvcc/link）**= 0**
- **wall 预估 = 240 s 合计；窗口实测 14 716 ms**（9 952 + 4 764）
- **attempt = 1**（采集轮）；监测轮为 attempt1–4（§14）
- 注意：以上只是规划数字；真正开窗前的干净监测与签名记录见 §14 / `logs/window-facts.txt`。

## 9. 捕获命令（窗口内逐字执行；pinned python；两组分开跑、同一窗口）

```powershell
# 组 1：hfield-only
$env:MJWARP_CAPTURE_AT = "2026-10-29T00:00:00+08:00"
target\mjwarp-parallel\venv\Scripts\python.exe -B scripts/capture_render_hfield.py --group hfield-only --use-fast-math false --use-shadows true --out-dir fixtures/render-bvh/scene-hfield/capture-hfield/hfield-only

# 组 2：hfield-primitive
$env:MJWARP_CAPTURE_AT = "2026-10-29T00:00:00+08:00"
target\mjwarp-parallel\venv\Scripts\python.exe -B scripts/capture_render_hfield.py --group hfield-primitive --use-fast-math false --use-shadows true --out-dir fixtures/render-bvh/scene-hfield/capture-hfield/hfield-primitive
```

窗口内两条命令均 **exit 0**（日志：`logs/capture-hfield-only.log`、`logs/capture-hfield-primitive.log`）。

## 10. 窗口协议（脚本内置 + 窗口侧）

- 脚本内置：采集前后 legacy 快照比对、A/B 两次逐字节一致、非零退出即无效；
- 窗口侧（沿用 R09-S3b 协议）：开窗前最后构建 → 冻结脚本/场景（记 sha256 进 `materials/signature-record.md`）
  → 监测 ≥60 个连续干净样本（5 s 采样）→ 签名后任一实体变动作废重签 →
  窗内 A/B 重采一致 → 窗口产物与 exit 码全部留档；**exit 9 等无效轮按协议留档、不当作有效轮**。

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

## 13. 窗口前监测与开窗（事实摘要，全文见 `logs/window-facts.txt`）

| 监测轮 | 数据 | 最长连续干净 | util 范围 | free_min | 结论 |
| --- | --- | --- | --- | --- | --- |
| attempt1 | 177 样本 / 23:37:32→23:52:28 | 24/60 | 2–43 | 13 134 MiB | 停手，未开窗 |
| attempt2 | 177 样本 / 00:02:37→00:17:32 | 34/60 | 2–43 | 13 188 MiB | 停手，未开窗 |
| attempt3 | 354 样本 / 00:23:15→00:53:11 | 39/60 | 2–49 | 13 109 MiB | 停手，未开窗 |
| attempt4 | 60 样本 / 00:54:15→00:59:15 | **60/60** | 4–48 | 13 166 MiB | 例外轮达标，开窗 |

- attempt1–3 均执行 gate1（3 签字物重哈希，`signature_match=True`）+ gate2（3 点基线 + floor 重算 +
  compute filter/build procs 快照），未放宽任何阈值。
- **attempt4 为 Chief 预批的 authoring-only 例外（`util<60`）**：原 `util<40` 阈值失败记录保留在
  `logs/window-facts.txt` 的 attempt1–3 段；本片不把它记作常规门槛通过。理由（`materials/load-attribution.txt`）：
  窗口期 30 条 compute-apps 全部是 WDDM 桌面图形上下文（浏览器/WebView/Overlay/远程桌面/DSH GUI 等），
  无真 compute 作业；authoring 采集是上游确定性渲染、脚本自带 A/B 位等校验、本片零数值结论。
- 例外轮监测脚本：`logs/monitor-until-clean-attempt4.ps1`，sha256
  `689fd7eb9cf733a60d40b9bdccafe4119ee23e2e8e15a0df68638716a9c1871b`，ASCII-only 三项自检
  （非 ASCII 字节 0 / ANSI 行数 34 == LF 行数 34 / 无 `$` 代码行并入注释）。
- 开窗瞬间 gate4 复验：3 签字物 `signature_match=True`；3 点基线 free_min **13 268 MiB** ⇒
  **floor = min(10240, 13268−3072) = 10 196 MiB**；compute 过滤 **0**、构建进程 **0**。
  开窗 `2026-10-29T00:59:35.527+08:00`，窗口关闭 `2026-10-29T01:00:20.539+08:00`。

## 14. 采集执行事实（attempt=1，窗口内两条命令）

- `capture_at_env = 2026-10-29T00:00:00+08:00`（两条命令同值；产物 `captured_at` 两组同值）
- hfield-only：exit **0**，wall **9 952 ms**；hfield-primitive：exit **0**，wall **4 764 ms**
- 合计 **14 716 ms**（预算 ≤240 s）；窗口内零 cargo/rustc/cargo test；窗口结束立即释放 GPU
  （capture 进程退出，无 `render_` 残留、build procs 0；窗后 free=13 302 MiB / util=40 % 为桌面基线）
- 日志：`logs/capture-hfield-only.log`、`logs/capture-hfield-primitive.log`

## 15. 采集产物（8 文件，总 5 784 264 B）

| 组 | 文件 | 字节 | sha256 |
| --- | --- | --- | --- |
| hfield-only | `rgb.npz` | 1 440 478 | `f3e82e24e2a1ee0001a4ba5512709f7e2203845d7778a8f574085d4a9404fdd8` |
| hfield-only | `depth.npz` | 480 486 | `ea887010e8edcd8e602303342e4602480cbcf8e6b1c15cad0f8a79fc64602921` |
| hfield-only | `seg.npz` | 960 478 | `a532fe61759caf541ddb854edb13accaafecfe57089f1c2991e4703eca72c5e3` |
| hfield-only | `capture.json` | 10 679 | `9f0e3c88e9ee4a7c516ccfd4f10a32baf400211fb3e1bf3ab6bd2c380a3191bb` |
| hfield-primitive | `rgb.npz` | 1 440 478 | `f3473ce63b0c71b1a123fc47f32ecbe5925715fedba87e5a2a462af3b1537795` |
| hfield-primitive | `depth.npz` | 480 486 | `fc9ae386bd6822ef5814193b7450e47896f3713b15a5e99f8b9b9a73b1fc49f7` |
| hfield-primitive | `seg.npz` | 960 478 | `aa2b04cb4e0197fa81eab678c0084548d4ec4d8d9e9e6cb26d958aa84a4286d3` |
| hfield-primitive | `capture.json` | 10 701 | `7a5fffdcba6ac156fe921ee3ccdf0f2c694ab23910ec9f3a7ae41db32f579b13` |

两份 `capture.json` 关键字段（与产物逐项自洽）：

- `task_id=2026-10-28-render-bvh-r05-s3a-hfield-capture`；`group` 分别为 `hfield-only` / `hfield-primitive`
- `recipe.scene_xml.sha256` = 各组签字场景 sha（`b3b9a6b4…` / `bf9bb966…`）；`recipe.script.sha256`
  两组均 `cb1808f8…`
- `use_fast_math=false`（7 处，0 处 true）；`use_shadows=true`（7 处，0 处 false）
- `two_captures_bitwise_identical=true`；`legacy_reference_untouched=true`、`legacy_reference_files=38`
- `captured_at=2026-10-29T00:00:00+08:00`（两组同值）；`recipe.cameras=0:cam-top:320x240,1:cam-side:240x180`
- `files_sha256` 与 §15 表逐项一致

## 16. 窗口后旧参考零读写复核

- 旧参考复算：**38 文件 / 25 320 515 B / 聚合 `b5cb50afcc608a67ca9a7b83222029402719f2770a3510ab423df13601e50d8b`**
  与窗口前一致；复核沿用 §6 的「工作树字节 + PowerShell 路径排序」口径（**不是** committed blob 复算）。
- `scene-shadow/`：目录 **17 文件 / 11 566 120 B**（含 `scene.xml` 4 757 B），其中 **16 个采集产物
  / 11 561 363 B**；逐文件 sha 与窗口前快照比对 **mismatch = 0**。
- `git status --short -- scripts fixtures/render-bvh/scene-shadow fixtures/render-bvh/scene-a
  fixtures/render-bvh/scene-e2e` = **0 行**（窗口期间未改这些路径）。

## 17. 措辞与边界

- 本片**只采集不比数**：不写任何 rgb/depth/seg 差异或正确性数字，不宣称任何数值结论。
- attempt4 例外全程留档（批准人/时间/理由/原失败记录/脚本 sha/ASCII 自检），不表述为常规门槛通过。
- 旧参考与 R09 采集路径零改动（§16）。
- **局限（QA O-3）**：本片「窗内零编译」证据只到「构建进程（cargo/rustc/cl/nvcc/link）=0」，
  **没有** `target/debug/deps` 前后计数快照（R09-S3b 窗口口径有该快照）；采集日志里的 Warp JIT
  `(compiled)` 属 pinned Python/Warp 侧内核编译，**不是** Rust/cargo 编译，不能当作零 Rust 编译证据。

## 18. normify 前置状态

- 脚本 + 两个场景 XML + 8 个采集产物 + 证据（README/receipt/manifest/logs/materials）已落地；
- 本窗口产物按「产物」「证据」两段提交，另有 verified 收口提交（`normify done` 后）；
- **manifest 路径基准（两种，头部亦已注明）**：8 个捕获产物行使用**仓库根相对路径**
  （`fixtures/render-bvh/scene-hfield/capture-hfield/...`）；证据目录内文件行使用**本证据目录相对路径**
  （如 `README.md`、`logs/...`、`materials/...`）；manifest 不自登记，可按头部基准一键校验；
- **证据编码修正（QA O-1）**：4 个证据文件原为 PowerShell 重定向产生的 UTF-16LE 副本
  （`logs/capture-hfield-{only,primitive}.log`、`materials/dryrun-hfield-{only,primitive}.json`），
  已转为 UTF-8（LF、无 BOM），内容不变；manifest 按 committed blob 的 LF 口径重算并自校验。
  这些文件**不在** `guards.json` 输入内，不影响验证窗；
- 数值验收（RGB/depth/seg 判据）全部在 `2026-10-28-render-bvh-r05-s3b-hfield-validate` 验证窗执行。

## 遗留（带入 S3b，来自 S2 O5）

`SceneLeaf.size` 必须与三角化产出的 `half` **同源**（不得 host 侧重推）：S3b 目标实体构造
SceneState 时应先 `build_hfield_mesh(...)` 再取 `half()` 填 `SceneLeaf.size`；fixture 侧保证。
