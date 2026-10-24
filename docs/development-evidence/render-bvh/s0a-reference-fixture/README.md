# 渲染/BVH S0a：3.12 与 3.15 双 MJB 参考夹具与模型 parity（按裁决口径 pass，含相机内参缺陷修正）

任务：`2026-10-24-render-bvh-s0a`（2026-10-24 修正记录见 §2）。日期：2026-10-24。执行：Backend。
依据：`docs/development-evidence/render-bvh-precheck/plan.json` 的 S0a 切片、README §6 与用户
批准的 D1 双 MJB parity 口径；判据改判见 §6（用户 2026-10-24 裁决 A）；夹具缺陷修正经用户
2026-10-24 批准。**CPU/离线 authoring，无 GPU。**

## 1. 结论（先给）

1. **S0a 按已批准判据成立（`parity=pass`），初始严格逐元素结论为 fail 并原样保留**：
   133 个字段中 126 个逐位一致、7 个存在版本差异（`geom_size`、`geom_aabb`、`geom_rbound`、
   `geom_quat`、`mesh_quat`、`mesh_polynormal`、`geom_sameframe`）。
   `parity.json` 同时保留：`conclusion.parity=pass`（批准判据）、
   `conclusion.strict_elementwise_parity=fail`、
   `version_difference_classification.strict_elementwise_parity=fail`（初始严格逐位结论）。
2. **7 个差异分两类，double 层数字全部保留**：
   - **6 个派生 float64 字段只差 double 尾数**：`geom_size/aabb/rbound/quat`、`mesh_quat`、
     `mesh_polynormal`；最大绝对差 **7.9e-14…2.6e-12**，`max_ulp_f64` 与 `f32_bit_differences`
     （全部 0）逐字段保留在 `version_difference_classification.version_differences`。
     按任务验收第 3 条「float f32_bits 优先」判等：**f32 位型完全相同（0 位差）**。
   - **1 个整型语义差异**：`geom_sameframe` 在 3.12 为 `BODY(1)`、3.15 为 `BODYROT(3)`
     （差 3 个 geom：base-sphere、tri-cyl、tri-ellip）。按裁决登记为
     `render_relevant=false` 排除：上游 MJWarp 渲染器 `_src` 全树无 sameframe 引用
     （Chief 2026-10-24 grep 核实）+ MuJoCo 3.15 枚举语义修正；该字段是碰撞帧复用内部标志。
3. `counts` 完全一致（26 个计数项）；两份 MJB 均能由对应版本加载；脚本与 MJB **确定性通过**
   （3.12/3.15 各编译两次字节一致）。其余 126 个字段（geom 类型/尺寸/位姿/rgba/材质/纹理像素/
   相机/光源/名称表/vis/stat/mesh 顶点/面/法线/纹理坐标等）**逐位相等**。
4. **原因假设**（未定位到 3.15 具体改动点）：3.15 的 mesh 资产帧（`mesh_quat`/`mesh_pos`）与
   派生包围量计算路径/精度与 3.12 不同，差异 ~1e-12 相对量级，沿 `mesh_quat → geom_quat →
   geom_size/aabb/rbound、mesh_polynormal` 传播；`geom_sameframe` 则是枚举语义修正
   （3.15 对「仅位置不同、旋转相同」的帧改标 `BODYROT`）。两条均为版本差异，
   不是同一 XML 在两版下解析出的数据漂移。
5. **不宣称**：两版模型逐位一致、3.12 与 3.15 等价模型、严格逐元素 parity 通过；
   只宣称**渲染相关模型数组在 f32 位型上等价**，范围限定见 §6。
   同样**不宣称初版 S0a 夹具无缺陷**：初版 cam-persp 内参缺失已在本轮修正（§2）。

## 2. 缺陷修正记录（用户 2026-10-24 批准；S0a 重钉）

### 2.1 缺陷与成因

- 初版 `scene.xml` 的透视相机 `cam-persp` 只写了 `sensorsize="0.036 0.024"`，没有焦距/主点。
  MuJoCo 对该相机保留 `cam_intrinsic=(0,0,0,0)` 并把 `cam_fovy` 留在 **180 度哨兵**
  （`180.00000500895632`）；MJWarp `render_util.compute_ray` 只要 `sensorsize[1] != 0` 就走
  内参分支，`fx=0` → `znear/fx` 除零 → NaN → normalize 得零向量，**cam-persp 整幅渲染为背景**
  （S0b 实测：`rc.ray` 前 76,800 条全零、rgb 全 0、depth 命中 0/76,800、seg 全 `(-1,-1)`）。
- **成因 = S0a 只做模型数组 parity、没有渲染**：3.12 与 3.15 对同一 XML 一致地产生坏值，
  所以 parity 照常 pass；缺陷在 S0b 首次真正渲染时暴露。S0b/S5/S7b 依赖透视相机的内参分支，
  必须修夹具而不是绕开。
- 初版 `scene.xml` 记录哈希与提交后文件不一致（工作区文件含 1 处 CRLF，提交时 git 归一化为 LF）：
  `8e723f9d…`（4,234 B，初版证据记录值）≠ 提交后 blob `255f4d8c…`（4,233 B）。本轮一并重钉：
  `scene.xml` 哈希改为按 **LF 归一化字节**（CRLF→LF）计算，S0a `model_parity.py` 与 S0b
  `capture_reference.py` 使用同一算法，任何行尾配置的检出都会得到同一哈希，避免再次出现
  「记录值取决于本机检出形式」的问题。

### 2.2 修改

1. `scene.xml`：`cam-persp` 增加 `focalpixel="160 160"`（并加注释说明原因）。320px 传感器上
   fx=160px → `cam_intrinsic=(0.018, 0.016, 0, 0)`、`cam_fovy=73.73979461915735`，
   `cam-track`（fovy 分支）不受影响。
2. `camera_sanity.py`（新）：parity 之后断言两版每个相机 `cam_intrinsic` 各项 finite、
   `fx>0 且 fy>0`、`cam_fovy ∈ (0,180)` 且不是 180 哨兵；任一不满足即失败退出。
   `run_s0a.ps1` 顺序执行 `model_parity.py` → `camera_sanity.py`，结果写
   `camera-sanity.json`，同类夹具缺陷以后由 S0a 自己抓到。
3. 哈希口径：`scene.xml` 按 LF 归一化字节（CRLF→LF）计算 sha256，S0a parity 与 S0b 参考捕获
   共用该算法；MJB/JSON/NPZ 仍按原始字节哈希。这样不同 `core.autocrlf` 检出不会改变记录的
   XML 哈希。

### 2.3 旧 → 新哈希对照

| 产物 | 初版 | 修正后 |
| --- | --- | --- |
| `scene.xml` | `8e723f9d48f2fb926522fb149c406ea8f59d8560effe6fa81d021f938726c406`（4,234 B；提交后 blob 为 `255f4d8c…`/4,233 B） | `a9f1f0a35464f4df17f191e1d68431bda0f17648273578e58fa08f84b0449289`（4,639 B；LF 归一化哈希 = 提交 blob） |
| `scene-312.mjb`（101,328 B） | `a73d4b793f60800414ee8484c1efad11416b5284444c9f864557851ddb84ebd3` | `c6f5f9f17118067ad39fb2ea6be4f1c274e4d932b553bbe9a4e59b464c56aad1` |
| `scene-315.mjb`（101,320 B） | `60d0a8564c6a99fba64e9ce0d229bb96a235db2bc6415c0441d1ad73dfd6b02f` | `8953988f832c05b0561f04dd8b8e797a51c136fdead0ffda2a0b190bfd92e0d4` |
| `model-312.json` | `968bb0513631e70756fd2e41fd04b94502d8bd9bb8dfd2010a0289f9657619ab`（312,613 B） | `cdd490f0209dd16a57b2685f3139548af8a2b73195cdb0e01a74ab094ef1df00`（312,635 B） |
| `model-315.json` | `59407e636b820fbcba15719e257d9be5ac180d8218ed872d3861c0b8bbd596c5`（311,089 B） | `6b055ca2b14e79076ef86a0eab7b3665e7785080885990d273e6278057fb5240`（311,121 B） |
| `parity.json` | `69c540b63c4954f1f7510589d2105dcb55c3577f593be6dd02a16b4a7a3074bf`（57,622 B） | `136ff14268cbf6889be22331c44b0c3ac262ea5d4bf2219bf688cea0cae3d8d2`（57,763 B） |

模型数组差异（逐值 diff 初版→修正后）**只有两处**：`cam_fovy[0]` 由 `180.00000500895632`
变为 `73.73979461915735`；`cam_intrinsic[0..1]` 由 `0,0` 变为 `0.0179999992, 0.0160000008`。
其余 131 个字段/数组取值完全不变；parity 结论与 7 个差异字段清单**不变**。

## 3. 交付物

| 文件 | 说明 |
| --- | --- |
| `fixtures/render-bvh/scene-a/scene.xml` | 同一参考场景：7 geom（plane/box/sphere/mesh/capsule/cylinder/ellipsoid）、2 相机（内参分支 + fovy 分支）、3 光源（directional/point/spot）、3 材质、3 纹理（2d checker/2d gradient/skybox gradient）、1 inline mesh（四面体）、显式 vis.headlight/vis.map.znear,zfar/statistic |
| `fixtures/render-bvh/scene-a/scene-312.mjb` | MuJoCo 3.12.0 C API 编译（`compile_scene_312.cpp`） |
| `fixtures/render-bvh/scene-a/scene-315.mjb` | MuJoCo 3.15.0 pinned venv Python 编译（`model_parity.py`） |
| `fixtures/render-bvh/scene-a/model-312.json` | 3.12 侧渲染相关模型数组 dump（133 字段 + 26 计数） |
| `fixtures/render-bvh/scene-a/model-315.json` | 3.15 侧同一字段集 dump |
| `scripts/authoring/render-bvh/compile_scene_312.cpp` | 3.12 C API：load XML → 存 MJB → 重载 MJB → JSON dump |
| `scripts/authoring/render-bvh/model_parity.py` | 3.15 Python：编译/存 MJB/重载/dump + 逐字段 parity + provenance |
| `scripts/authoring/render-bvh/model_parity_judge.py` | 用户 2026-10-24 裁决 A 的改判模块（严格结论保留 + 批准判据/排除/不宣称） |
| `scripts/authoring/render-bvh/camera_sanity.py` | 相机内参健全性检查（本轮新增；§2.2） |
| `scripts/authoring/render-bvh/run_s0a.ps1` | MSVC 环境 + 编译 3.12 工具 + 顺序执行 parity 与 camera-sanity（单命令复现） |
| `parity.json` | 含 `task_id`；环境/命令/哈希/逐字段结果/差异分类/确定性 |
| `camera-sanity.json` | 含 `task_id`；两版每相机 intrinsic/fovy 解析值与判定 |
| `receipt.json` | 回执与未满足项 |
| `run.log` | 本次运行日志 |

场景覆盖口径：geom_*（全部 3.12 版字段）、cam_*、light_*、mat_*、tex_*（含纹理字节）、
mesh_*（顶点/法线/纹理坐标/面/多边形/包围体）、vis.headlight、vis.map.znear/zfar、
stat.extent/center、名称表（`name_*adr` + `names` 字节）；字段表与 `compile_scene_312.cpp`
逐项一致，由 `model_parity.py` 的 `FIELDS` 清单校验（名称/类型/长度不符即脚本失败）。

## 4. 环境与 provenance

| 项 | 值 |
| --- | --- |
| 3.12 产品版本 | MuJoCo **3.12.0**（`target/toolchains/mujoco-3.12.0/package`，C API `mujoco.lib`） |
| 3.15 参考版本 | MuJoCo **3.15.0**（`target/mjwarp-parallel/venv`，Python 3.12.13、NumPy 2.5.4） |
| 工具链 | MSVC（VsDevCmd x64）+ pinned venv Python；无 GPU |
| `scene.xml` sha256 | `a9f1f0a35464f4df17f191e1d68431bda0f17648273578e58fa08f84b0449289`（4,639 B，LF 归一化哈希；S0a/S0b 共用） |

哈希与版本（实测值以 `parity.json` 的 `sources`/`environment`/`hashes` 为准）：

| 产物 | 字节 | sha256 |
| --- | ---: | --- |
| `scene-312.mjb` | 101,328 | `c6f5f9f17118067ad39fb2ea6be4f1c274e4d932b553bbe9a4e59b464c56aad1` |
| `scene-315.mjb` | 101,320 | `8953988f832c05b0561f04dd8b8e797a51c136fdead0ffda2a0b190bfd92e0d4` |
| `model-312.json` | 312,635 | `cdd490f0209dd16a57b2685f3139548af8a2b73195cdb0e01a74ab094ef1df00` |
| `model-315.json` | 311,121 | `6b055ca2b14e79076ef86a0eab7b3665e7785080885990d273e6278057fb5240` |
| `parity.json` | 57,763 | `136ff14268cbf6889be22331c44b0c3ac262ea5d4bf2219bf688cea0cae3d8d2` |

- 3.12 MJB 由 3.12 C API `mj_loadModel` 重载成功后才 dump；3.15 MJB 由 3.15
  `MjModel.from_binary_path` 重载成功后才 dump（`loads_with_312/315=true`）。
- 确定性：两版 MJB 各独立编译两次，sha256 一致；数组 dump 一致（JSON 中输出路径字段除外）。

## 5. 逐字段 parity 结果（`parity.json`，修正后重跑，结论不变）

- 比较字段 **133** 个（float32 29、float64 32、int32 63、int64 1、byte 8）；计数项 26 个全部一致。
- 逐位一致 **126**；不一致 **7**：

| 字段 | 类型 | 位差 | 最大绝对差 | f32 位差 | 分类 |
| --- | --- | ---: | ---: | ---: | --- |
| `geom_size` | float64 | 3/21 | 9.42e-14 | 0 | double 尾数 / f32 一致 |
| `geom_aabb` | float64 | 6/42 | 1.27e-13 | 0 | 同上 |
| `geom_rbound` | float64 | 1/7 | 7.93e-14 | 0 | 同上 |
| `geom_quat` | float64 | 4/28 | 9.55e-13 | 0 | 同上（仅 mesh geom 继承 mesh 资产帧） |
| `mesh_quat` | float64 | 4/4 | 9.55e-13 | 0 | 同上（mesh 资产帧） |
| `mesh_polynormal` | float64 | 12/12 | 2.56e-12 | 0 | 同上（由 mesh 帧派生） |
| `geom_sameframe` | byte | 3/7 | 2（值 1→3） | — | 枚举语义版本差异（BODY→BODYROT，内部碰撞标志） |

最差字段：`mesh_polynormal`（2.56e-12）> `geom_quat`/`mesh_quat`（9.55e-13）> `geom_aabb`
（1.27e-13）> `geom_size`（9.42e-14）> `geom_rbound`（7.93e-14）；`geom_sameframe` 见上。

## 6. 判据与改判记录（用户 2026-10-24 裁决 A）

1. **初版结论（严格逐位）**：2026-10-24 首轮按 S0a 验收原文「逐元素相等」判
   **fail**：7/133 字段不一致；该结论与全部 double 层差异数字（`max_abs`、`max_ulp_f64`、
   `f32_bit_differences`、逐字段 mismatch 值）**原样保留**在
   `version_difference_classification` 与 `fields` 中，未删除、未改写。
2. **用户裁决（2026-10-24，选项 A）**：按项目已批准容差收口。判据为：
   - 范围限定 **渲染相关模型数组**（plan S0a 验收第 3 条）；
   - 整型/字节数组按 exact；float32 按 f32 位型；**float64 按任务验收「float f32_bits
     优先」= f32 位型判等**（6 个派生字段 f32 位差 0）；
   - `geom_sameframe` 登记 `render_relevant=false` 排除：上游 MJWarp 渲染器 `_src` 全树
     无 sameframe 引用（Chief 2026-10-24 grep 核实）+ MuJoCo 3.15 枚举语义修正
     （`BODY(1)→BODYROT(3)`，碰撞帧复用内部标志）。
   - **不改任何容差、不删任何 double 差异数字、不逐元素择优**。
3. **改判结果**：`conclusion.parity=pass`（批准判据），`judged_failures=[]`；
   `conclusion.strict_elementwise_parity=fail` 与
   `version_difference_classification.strict_elementwise_parity=fail` 作为初始严格结论保留；
   S0b 硬门按批准判据为 `met`。
4. **不宣称**（固定口径）：不宣称两版模型逐位一致，不宣称 MuJoCo 3.12/3.15 等价，
   不宣称严格逐元素 parity 通过；只宣称**渲染相关模型数组在 f32 位型上等价**
   （6 个 float64 字段 f32 位差 0 + 126 字段逐位一致 + geom_sameframe 按上述理由排除）。

## 7. 未满足项与边界

- 初始严格逐元素 parity 仍为 fail（7/133 字段），本轮为按用户批准判据的改判，
  不是两版模型逐位一致的证明。
- 未定位 MuJoCo 3.15 引起 mesh 资产帧计算差异的具体源码改动；当前为原因假设（§1.4）。
- 未做 3.15 native 产品化评估与 D1 备选（升级产品 ABI）影响面分析；未跑任何渲染/GPU 任务。
- `geom_sameframe` 排除依据：上游 MJWarp 渲染器 `_src` 无 sameframe 引用（Chief grep 核实）
  + 3.15 枚举语义修正；如需要可让 QA 复核。
- 初版相机内参缺陷（§2）已修，但“模型数组 parity 通过 ≠ 渲染路径可用”的教训
  由新增 `camera_sanity.py` 覆盖；其它渲染前置（BVH 内容、光源/材质可用性）不在 S0a 检查范围。
- 未改产品 `src/tests/Cargo.*`；产品测试不启动 Python；本切片全部 CPU/离线。

## 8. 复现

```powershell
target\toolchains\powershell-7.6.7\bin\pwsh.exe -NoProfile -ExecutionPolicy Bypass `
  -File scripts\authoring\render-bvh\run_s0a.ps1
```

单命令完成：MSVC 编译 `compile_scene_312.cpp` → 3.12 编译/重载/dump → 3.15 编译/重载/dump →
逐字段 parity → 相机内参健全性检查 → 写 `parity.json` 与 `camera-sanity.json`。
退出码 0 表示按批准判据 parity 成立且相机内参健全（严格逐位结果另存于
`version_difference_classification`）；退出码非 0 表示两者之一不成立。

## 9. 证据索引

- `parity.json`：机器可判定的 parity 结果（含 `task_id`、环境、命令、哈希、133 字段明细、
  批准判据 `conclusion.parity=pass`、初始严格结论 `strict_elementwise_parity=fail`、
  7 字段差异分类与全部 double 数字、确定性、硬门结论）。
- `camera-sanity.json`：两版每相机的 `intrinsic`/`fovy_deg` 解析值、检查项与 pass 判定。
- `receipt.json`：回执（改判结论/修正记录/产物摘要/未满足项）。
- `run.log`：本次运行输出。
- `fixtures/render-bvh/scene-a/`：XML、双 MJB、双 model JSON（逐字段原始值）。
- `scripts/authoring/render-bvh/`：C++ 3.12 工具、Python parity + 改判模块 + 相机健全性检查、单命令包装。
