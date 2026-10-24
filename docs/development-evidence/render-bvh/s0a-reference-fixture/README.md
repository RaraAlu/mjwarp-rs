# 渲染/BVH S0a：3.12 与 3.15 双 MJB 参考夹具与模型 parity（parity 未成立，S0b 阻塞）

任务：`2026-10-24-render-bvh-s0a`。日期：2026-10-24。执行：Backend。
依据：`docs/development-evidence/render-bvh-precheck/plan.json` 的 S0a 切片、README §6 与用户
批准的 D1 双 MJB parity 口径。**CPU/离线 authoring，无 GPU。**

## 1. 结论（先给）

1. **S0a 硬门未成立**：按 S0a 验收「渲染相关模型数组逐元素相等」，133 个字段中 **126 个逐位一致、
   7 个不一致**（`geom_size`、`geom_aabb`、`geom_rbound`、`geom_quat`、`mesh_quat`、
   `mesh_polynormal`、`geom_sameframe`）→ `parity.json` 记录 `parity=fail`，
   **S0b（3.15 参考图）不得启动**，本任务不做放宽容差或调参凑。
2. **7 个差异分两类**（分类为诊断，不改变 fail 判定）：
   - **6 个派生 float64 字段只差 double 尾数**：`geom_size/aabb/rbound/quat`、`mesh_quat`、
     `mesh_polynormal`；最大绝对差 **7.9e-14…2.6e-12**（约 1e-12 相对量级），
     **转 f32 后位型完全相同（0 位差）**。这些量由 mesh 资产帧/包围体在编译期派生，
     产品渲染路径按 f32 上传模型，故在 f32 口径下无差异。
   - **1 个整型语义差异**：`geom_sameframe` 在 3.12 为 `BODY(1)`、3.15 为 `BODYROT(3)`
     （差 3 个 geom：base-sphere、tri-cyl、tri-ellip）。该字段是**碰撞帧复用的内部标志**，
     渲染器不读取；属 MuJoCo 3.12→3.15 的语义修正/变更，不是夹具误差。
3. `counts` 完全一致（26 个计数项）；两份 MJB 均能由对应版本加载；脚本与 MJB **确定性通过**
   （3.12/3.15 各编译两次字节一致）。其余 126 个字段（geom 类型/尺寸/位姿/rgba/材质/纹理像素/
   相机/光源/名称表/vis/stat/mesh 顶点/面/法线/纹理坐标等）**逐位相等**。
4. **原因假设**（未定位到 3.15 具体改动点）：3.15 的 mesh 资产帧（`mesh_quat`/`mesh_pos`）与
   派生包围量计算路径/精度与 3.12 不同，差异 ~1e-12 相对量级，沿 `mesh_quat → geom_quat →
   geom_size/aabb/rbound、mesh_polynormal` 传播；`geom_sameframe` 则是枚举语义修正
   （3.15 对「仅位置不同、旋转相同」的帧改标 `BODYROT`）。两条均为版本差异，
   不是同一 XML 在两版下解析出的数据漂移。

## 2. 交付物

| 文件 | 说明 |
| --- | --- |
| `fixtures/render-bvh/scene-a/scene.xml` | 同一参考场景：7 geom（plane/box/sphere/mesh/capsule/cylinder/ellipsoid）、2 相机、3 光源（directional/point/spot）、3 材质、3 纹理（2d checker/2d gradient/skybox gradient）、1 inline mesh（四面体）、显式 vis.headlight/vis.map.znear,zfar/statistic |
| `fixtures/render-bvh/scene-a/scene-312.mjb` | MuJoCo 3.12.0 C API 编译（`compile_scene_312.cpp`） |
| `fixtures/render-bvh/scene-a/scene-315.mjb` | MuJoCo 3.15.0 pinned venv Python 编译（`model_parity.py`） |
| `fixtures/render-bvh/scene-a/model-312.json` | 3.12 侧渲染相关模型数组 dump（133 字段 + 26 计数） |
| `fixtures/render-bvh/scene-a/model-315.json` | 3.15 侧同一字段集 dump |
| `scripts/authoring/render-bvh/compile_scene_312.cpp` | 3.12 C API：load XML → 存 MJB → 重载 MJB → JSON dump |
| `scripts/authoring/render-bvh/model_parity.py` | 3.15 Python：编译/存 MJB/重载/dump + 逐字段 parity + provenance |
| `scripts/authoring/render-bvh/run_s0a.ps1` | MSVC 环境 + 编译 3.12 工具 + 顺序执行 parity（单命令复现） |
| `parity.json` | 含 `task_id`；环境/命令/哈希/逐字段结果/差异分类/确定性 |
| `receipt.json` | 回执与未满足项 |
| `run.log` | 本次运行日志 |

场景覆盖口径：geom_*（全部 3.12 版字段）、cam_*、light_*、mat_*、tex_*（含纹理字节）、
mesh_*（顶点/法线/纹理坐标/面/多边形/包围体）、vis.headlight、vis.map.znear/zfar、
stat.extent/center、名称表（`name_*adr` + `names` 字节）；字段表与 `compile_scene_312.cpp`
逐项一致，由 `model_parity.py` 的 `FIELDS` 清单校验（名称/类型/长度不符即脚本失败）。

## 3. 环境与 provenance

| 项 | 值 |
| --- | --- |
| 3.12 产品版本 | MuJoCo **3.12.0**（`target/toolchains/mujoco-3.12.0/package`，C API `mujoco.lib`） |
| 3.15 参考版本 | MuJoCo **3.15.0**（`target/mjwarp-parallel/venv`，Python 3.12.13、NumPy 2.5.4） |
| 工具链 | MSVC（VsDevCmd x64）+ pinned venv Python；无 GPU |
| `scene.xml` sha256 | `8e723f9d48f2fb926522fb149c406ea8f59d8560effe6fa81d021f938726c406`（4,234 B） |

哈希与版本（实测值以 `parity.json` 的 `sources`/`environment`/`hashes` 为准）：

| 产物 | 字节 | sha256 |
| --- | ---: | --- |
| `scene-312.mjb` | 101,328 | `a73d4b793f60800414ee8484c1efad11416b5284444c9f864557851ddb84ebd3` |
| `scene-315.mjb` | 101,320 | `60d0a8564c6a99fba64e9ce0d229bb96a235db2bc6415c0441d1ad73dfd6b02f` |

- 3.12 MJB 由 3.12 C API `mj_loadModel` 重载成功后才 dump；3.15 MJB 由 3.15
  `MjModel.from_binary_path` 重载成功后才 dump（`loads_with_312/315=true`）。
- 确定性：两版 MJB 各独立编译两次，sha256 一致；数组 dump 一致（JSON 中输出路径字段除外）。

## 4. 逐字段 parity 结果（`parity.json`）

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

## 5. 判定与待裁决

- **按 S0a 验收原文（逐元素相等）**：`parity=fail`，**S0b 硬门未满足**，不启动参考图捕获；
  未放宽任何容差、未挑字段、未调参凑（`parity.json` 的 `conclusion` 与
  `version_difference_classification.s0b_hard_gate` 均如实记录）。
- **按 plan §4.3 的模型浮点口径**（`f32_bits` 优先，不可达时 ≤1 ULP）：6 个派生 float64 字段
  **f32 位型完全一致**，在 f32 口径下无差异；`geom_sameframe` 为整型语义版本差异，
  且渲染器不读取该字段。
- 需 Chief/用户裁决的口径（本任务不自行选择）：
  - **A（f32 语义口径）**：模型浮点按 §4.3 f32_bits 判等；`geom_sameframe` 作为
    「非渲染内部标志 + 3.15 语义修正」登记排除（仅此 1 个字段），S0b 放行。
  - **B（严格 double 口径）**：保持 `parity=fail`，S0b 阻塞；若需要严格相等，只能评估
    产品切 3.15 native（D1 备选，影响全部现有原生测试，工作量大）或放弃 mesh 派生字段覆盖
    （会削弱 S0b 的 mesh 渲染验收，不建议）。
  - **C（严格记录 + 渲染级验证）**：保留 fail 记录，但允许 S0b 以 §4.3 渲染容差
    （法向 1e-4 rad、距离 4 ULP/1e-5 等）验证差异影响；本轮模型差异 ~1e-12 量级，
    远低于渲染容差，可在 S0b 证据中显式量化。
- 无论哪种口径，**不得**把本轮 parity 说成严格逐元素通过；MJB/模型差异记录保持原样。

## 6. 未满足项与边界

- S0a 硬门未满足（7/133 字段不一致），S0b 未启动（GPU 窗口未使用）。
- 未定位 MuJoCo 3.15 引起 mesh 资产帧计算差异的具体源码改动；当前为原因假设（§1.4）。
- 未做 3.15 native 产品化评估与 D1 备选（升级产品 ABI）影响面分析；未跑任何渲染/GPU 任务。
- `geom_sameframe` 为碰撞内部标志，渲染链路不读取；该判断附依据（muJoCo 渲染器使用
  `mjvScene`/`mjr`，不消费该字段），如需要可让 QA 复核。
- 未改产品 `src/tests/Cargo.*`；产品测试不启动 Python；本切片全部 CPU/离线。

## 7. 复现

```powershell
target\toolchains\powershell-7.6.7\bin\pwsh.exe -NoProfile -ExecutionPolicy Bypass `
  -File scripts\authoring\render-bvh\run_s0a.ps1
```

单命令完成：MSVC 编译 `compile_scene_312.cpp` → 3.12 编译/重载/dump → 3.15 编译/重载/dump →
逐字段 parity → 写 `parity.json`。退出码非 0 表示 parity 未成立（本切片即此情况）。

## 8. 证据索引

- `parity.json`：机器可判定的 parity 结果（含 `task_id`、环境、命令、哈希、133 字段明细、
  7 字段差异分类、确定性、硬门结论）。
- `receipt.json`：回执（结论/产物摘要/未满足项）。
- `run.log`：本次运行输出。
- `fixtures/render-bvh/scene-a/`：XML、双 MJB、双 model JSON（逐字段原始值）。
- `scripts/authoring/render-bvh/`：C++ 3.12 工具、Python parity 脚本、单命令包装。
