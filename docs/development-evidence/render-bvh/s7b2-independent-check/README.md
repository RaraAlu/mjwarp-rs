# 渲染/BVH S7b2 独立复核（81 槽着色/RGB/纹理/灯位姿/同配方参考）

任务：`2026-10-25-render-bvh-s7b2-independent-check`（QA 独立复核，渲染/BVH S7b2）
复核对象：`f228099`（S7b2 主体，父 `14ee028`）+ 配套 `a96a158`（`TextureResources::handles()`）
证据基线 HEAD：`9495022`（本复核任务登记/启动提交）
结论：**验收面全部成立，0 阻断**；2 条证据/文档口径问题（建议 `normify fix`），若干观察项。

> 口径声明：本复核**未跑完整 G01**（团队规则：日常迭代定向 + fmt/clippy/clippy-cuda/unit）；
> 不宣称完整渲染/R01 通过；不宣称 `bvh_id` 与 Warp BVH 等价（仅签名对齐）。
> 本目录只含 QA 复核证据，未修改产品代码 / 交付夹具 / 原交付证据 / 容差 / 排除清单。

## 1. 方法（不读交付断言）

- **独立 harness**（`qa-harness-main.rs.txt`，构建于 `target/qa-s7b2-harness/`，产品 crate path
  依赖 + `cuda-probe`）：自带模型/场景/相机位姿/夹具解码，不 import 交付 `tests/support/**`；
  只调产品公开 API 渲染，把 RGB/depth/seg **原始位**落盘。
- **独立参考重采**（`capture_qa.py`，pinned venv：Python 3.12.13 / warp 1.15.0 / MuJoCo 3.15.0
  / mujoco_warp 3.15.0 / numpy 2.5.4；`PARAMS_28` 仅 `use_fast_math=False`）：QA 自己跑
  `put_model/put_data/forward/create_render_context/render`，直接读 `rc.rgb_data/depth_data/seg_data`
  与 `d.cam_xpos/cam_xmat/light_xpos/light_xdir` 位。仅复用交付 authoring 脚本的冻结 `PARAMS_28`
  作为参考配方约定，未复用其判定。
- **独立判定**：`judge_run.py`（packed ABGR / 解包 f32 / depth ULP / seg）、`judge_fixtures.py`
  （夹具解码与 provenance）、`judge_light.py`（灯位姿四路）、`judge_slots.py`（81 槽 ABI）、
  `judge_hashes.py`（运行哈希/源码/PTX 身份）、`judge_shininess.py`（材质高光指数语义）、
  `judge_evidence.py`（证据完整性/范围）。

## 2. 核心数字（QA 独立重算）

### 2.1 RGB 同配方 0 差异（硬指标）

| 项 | QA 结果 |
| --- | --- |
| 产品 vs 交付同配方夹具 `rgb_fast_math_false.json` | cam0 **76,800/76,800**、cam1 **43,200/43,200** 像素 packed u32 **逐位相同**；合计 **120,000/120,000** |
| byte 差直方图 `[0,1,2,3,>3]` | cam0 `[76800,0,0,0,0]`、cam1 `[43200,0,0,0,0]`；max byte diff **0** |
| 解包 f32（`byte * float32(1/255)`）每通道 | max LSB diff **0.0** |
| 产品 vs **QA 自己重采**的同配方参考 | 同样 **120,000/120,000 位等**（README/judge_run_base_qa.json） |
| 两次渲染确定性 | RGB/depth/seg 两次逐位相同 |

### 2.2 参考重采复现交付夹具（provenance 独立确认）

QA 用 pinned 3.15 自己重采（默认位姿/默认材质）→ 与交付夹具逐位比较：

- `rgb_fast_math_false.json`：cam0/cam1 `rgb_bits` **逐位相等**；
- `depth_seg_fast_math_false.json`：cam0/cam1 `depth_bits` **逐位相等**、`seg_pairs` **逐位相等**；
- `rgb.json`（S0b fast-math 派生）：QA 用 `reference/rgb.npz` 按 `rint(*255) → pack ABGR`
  独立重算，**逐像素相等**；两夹具全部像素 `a=255`、长度 = `w*h`、`unique/black` 计数一致；
- 重采 `cam_xpos/cam_xmat` 位与 S7b1 夹具 `camera_data_check` **逐位相等**（产品渲染使用的位姿
  即参考侧位姿）。

### 2.3 depth/seg 回归（S7b1 门）

产品 vs 同配方参考：两相机 **120,000/120,000** depth 位等、max **0 ULP**、seg **0 mismatch**。

### 2.4 运行哈希（独立复算）

| 项 | QA 从自己 dump 复算 | 交付 `gpu-run.log` |
| --- | --- | --- |
| depth+seg FNV-1a 64 | `03efa95416bbd008` | `03efa95416bbd008` |
| RGB（测试用 `fold(0u64)` 种子） | `ed50f750936c0275` | `ed50f750936c0275` |

### 2.5 跨配方偏差（只作已知偏差记录，不作通过依据）

产品 vs S0b fast-math 夹具：cam0 `[76798,2,0,0,0]`（max 1 byte）、cam1 `[43200,0,0,0,0]`；
与交付夹具 `cross_recipe` 记录及测试打印**逐项一致**，QA 未把它当通过依据、也未写成产品缺陷。

### 2.6 泛化：新相机位姿（非夹具位姿）

两相机平移 `(+0.05,-0.03,+0.02)`（改 `mjm.cam_pos` → pinned forward 重采参考，位姿位由
`d.cam_xpos/d.cam_xmat` 位传给产品 harness）：

- RGB 对重采参考 **120,000/120,000 位等（0 差异）**；
- depth **0 ULP**、seg 全等；两次渲染逐位相同。

### 2.7 `mat_shin_exp` 语义修复（非对夹具调参）

**参考侧语义（读 pinned 采集生成的 megakernel CUDA 源，`reference-megakernel-headlight-excerpt.txt`）**：

- `var_340 = 128.0`；`var_341 = mat_shininess[mat_id] * var_340`；
- `var_353 = where(mat_id>=0, var_341, DEFAULT_MAT_SHININESS_EXPONENT)`；
- headlight 的 `compute_lighting` 调用（L9432）末段实参是 `var_352`（mat_spec）/`var_353`
  （mat_shin_exp）——即 headlight 高光指数取**材质**值，不是固定 64。

**产品侧代码**：`render_megakernel.cu` headlight 分支把 `mat_spec`/`mat_shin_exp` 传入
`compute_lighting`；`render_shade.cu` 用 `mat_shin_exp` 作 `powf(ndoth, …)` 指数；无
`geom_id`/像素/坐标特判。

**独立实验（换材质值，两档都验）**：

| 实验 | 参考侧变化（cam1） | 产品 vs 修改后参考 |
| --- | --- | --- |
| `mat_shininess[1]=0.05`（指数 6.4） | 1,046 像素变化、799 像素 >1 LSB、max 10 byte；cam0 0 变化 | **120,000/120,000 位等** |
| `mat_shininess[1]=0.3`（指数 38.4） | 814 像素变化、542 像素 >1 LSB、max 15 byte；cam0 0 变化 | **120,000/120,000 位等** |

cam0 不变化符合材质/几何分布（mat1 为球面 + 圆柱，仅 cam1 可见）。若实现用固定 64，这两档
不可能同时逐位等于参考；结合参考源码语义 ⇒ **修复是语义修复，不是对着夹具调参**。

**未独立复现**：交付记录的「修复前 332/43,200 超 1 LSB、cam0 26,649 像素差 1 byte」属修复前
状态，需要产品内核变体才能复现（QA 不改产品代码），仅登记为交付历史记录。

## 3. 其余复核面

### 3.1 灯位姿（四路一致）

QA 按上游 `smooth.py` else 分支自行 f32 复算（世界 body `xquat` 由 `.mjb` 验证为恒等：
`[1,0,0,0]`；三灯 `light_mode=0`、`light_bodyid=0`）：

- 3 灯的 `xpos` 6 组位型：**QA 复算 = pinned forward = 产品 `light_data_poses` = 夹具
  `light_data_check`**（全 true）；
- 3 灯的 `xdir`：上述四路**逐位一致**；三种 normalize 假设（fma 链/plain dot/乘倒数）在该
  数据上均命中同一位型（不构成对运算顺序的区分，仅用于排除误算）。

### 3.2 81 槽签名 / 逐槽 / 占位 / `bvh_id`

- 内核签名 **81 个形参 = 1 bounds + 73 数组 + 7 标量**，与 S7a 冻结 `MEGAKERNEL_SLOTS`
  逐槽（含元素类型）**0 mismatch**；`args.push` **81** 次；README 逐槽表覆盖槽 0..80 且
  名称与内核形参逐槽一致。
- 占位槽「内核不读」：按表 19 个 placeholder 中 **18 个在内核体零引用**；唯一例外
  **slot 62 `mesh_texcoord`** 被传入 `sample_texture`，仅在 `mesh_texcoordadr >= 0` 的
  mesh-texcoord 分支解引用（scene-a 为 `-1` → 永不取用）。详见 §5 观察 3。
- `bvh_id`（slot 54）为**自有打包 blob**：host 写 header 对（字节偏移、字节长度）于
  `header[2i]/[2i+1]`，设备按 `header[0,2,4,6,8]` 取五段指针；各段起止 = `40 + 16k` 字节；
  `SceneView` 结构与 S7b1 `render.cu` **逐 token 相同**。**仅签名对齐，不宣称 Warp BVH 等价**。

### 3.3 A′ 渲染门禁 / PTX

- 产品唯一编译路径：`RenderKernel::compile → compile_render_artifact`（`WARP_1_15_RENDER` +
  `RecipePurpose::Render` + `check_render_ptx`）；无旁路。
- QA harness 现场编译：PTX **195,594 B**、`source_sha256=0fcdb02a…`（= 五段内核源拼接
  74,714 B 的 sha256，QA 独立重算一致）、`ptx_sha256=75a6ad35…`；entry 恰
  `scene_bvh_ray` + `render_megakernel`。
- PTX 全部 `.f64` 行：2 条 W15 白名单全局常量（符号+字面量逐字节）+ 1 组 sin/cos 归约三元组
  （`.reg .f64`、`cvt.rn.f64.s64`、`mul.f64 …0d3BF921FB54442D19`、`cvt.rn.f32.f64`）；无其它 f64。
- `scene.cu` sha256 `8dad4d61…` **未变**（与 S2b 证据一致）；`launch.rs` **恰 500 行**且未改；
  `render_math.cu` 在 `14ee028..f228099` **零改动**。

### 3.4 写前拒绝面（QA 独立判定，非 catch_unwind 兜底）

4 类配置均**成功建 context 后由 renderer 返回** `LauncherError::Unsupported`（无 panic）：

- `use_shadows=true` → `Unsupported("S7b2 不含阴影")`
- `use_precomputed_rays=false` → `Unsupported("S7b2 只支持 use_precomputed_rays=true（动态射线归 S7b3）")`
- `samples_per_pixel=2` → `Unsupported("S7b2 只支持 samples_per_pixel=1（AA 属 S7b3）")`
- `render_skybox=true` → `Unsupported("S7b2 不含天空盒")`

### 3.5 门禁 / 回归（QA 重跑）

- `cargo test --locked`：**98 个测试二进制、532 passed / 0 failed**（`qa-unit-tests.log`）；
- `cargo fmt --all -- --check`：通过；`cargo clippy --all-targets --locked --features cuda-probe
  -- -D warnings`：通过；
- 交付 GPU 用例原样重跑：S7b2 **3/3**、S7b1 **6/6**、S6b **5/5**（`qa-gpu-runs.log`）。

### 3.6 证据完整性 / 提交范围

- 交付证据 7 个文件与 `f228099` 的 git blob **逐字节相同**；receipt 的 `evidence_files` 列表一致。
- PTX/源码/夹具 sha、`kernel_source_bytes=74,714`/`lines=1,804`、`launch.rs=500`、capture
  脚本 sha、`capture.log` 的 `div_rn=206/div_approx=0` 与真实 kernel cache 文件（637,336 B、
  `ad21b93f…`）**全部一致**。
- `git diff 14ee028..f228099`：32 个文件，**全部在本片 + 配套任务范围内**（out_of_scope 空）；
  S2–S6 证据/夹具、`src/render/bvh/**`、`render_math.cu`、`launch.rs` 均未改。

## 4. GPU 窗口

- 设备：RTX 4070 Ti SUPER / 驱动 596.36 / CUDA 13.2 / 16 GiB；开始前无 CUDA 计算进程。
- 窗口起止显存：开始 **3336 MiB**（桌面基线）→ 结束 **3558 MiB**（桌面应用波动区间
  3.3–3.6 GiB；与开始同一波动区间）；**结束时无本任务 CUDA 计算进程，GPU 已释放**。
- 实际 GPU 计算：三套交付用例 2.65 s + 4.14 s + 8.37 s，QA harness 5 次渲染，参考重采 4 次，
  合计约 1–2 分钟量级，30 分钟窗口内、无冲突、未超窗口占用。

## 5. 问题清单（按严重级别）

### 一般（证据/文档口径，修复建议 `normify fix`，不影响验收结论）

1. **`rgb_hash` 记录少一位十六进制**：README §5.4 与 receipt `gpu.run_determinism.rgb_hash`
   记为 `ed50f750936c275`（15 位），实际（`gpu-run.log` 与 QA 独立复算）为
   `ed50f750936c0275`（16 位）。建议修正该字段。
2. **README §4 统计行与逐槽表自相矛盾**：统计行写「real 54、自有打包槽 3（54/71/73 位、75）、
   placeholder 24」，但按该表逐行分类应为 **real 58 / own 4 / placeholder 19**（81 槽齐全，
   0..80 无缺漏）；receipt `abi.real_slots/own_pack_slots/placeholder_slots` 沿用了错误统计。
   建议按表重算后修正（不影响 ABI 实际形状：签名/槽序已独立核对一致）。

### 建议（表述/测试严谨性，非缺陷）

3. 「placeholder = 1 元素占位（内核不读）」对 **slot 62 `mesh_texcoord`** 过强：该槽被传入
   `sample_texture`，仅在 `mesh_texcoordadr>=0` 分支解引用（scene-a 为 `-1`，不取用）；建议
   表述为「本场景不读取；网格带 texcoord 时为真实数据」。
4. `bvh_id` 段布局描述「段间 16 字节对齐」：各段起点实为 `40 + 16k` 字节（段**间距** 16 字节，
   绝对偏移 ≡ 8 mod 16）。功能无影响（所有访问 4 字节对齐），建议措辞改为「段间距 16 字节」。
5. 交付拒绝面用例对 context 创建用了 `catch_unwind(... continue)`：若创建期 panic 也会被算作
   「写前拒绝」。QA 独立判定 4 类均在建 context **成功之后**由 renderer 返回 `Unsupported`
   （无 panic），故结论成立；仍建议测试断言建 context 成功，避免把 panic 当拒绝。

### 未独立确认（如实登记，不计入通过依据）

6. 修复前症状数字（cam1 332/43,200 超 1 LSB、max 3；cam0 26,649 像素差 1 byte）与「两处疑似
   偏离同时回退后仍 0 差异」属修复前/变体实验，QA 未构建产品内核变体，**未复现**；另上游
   `render.py` L487 `wp.texture_sample` 默认 lod（-1.0）与产品 `0.0f` 是**源码级差异**，本场景
   1 mip 下未产生位级差异（QA 与参考 0 差异），但 QA 未做变体证明「无影响」。

## 6. 未覆盖 / 边界（不宣称）

- 未跑完整 G01；不宣称完整渲染/R01/双平台。
- 阴影、天空盒、splat、AA（spp>1）、flex/hfield/SDF、动态射线：本片写前拒绝，未验功能。
- 正交相机、带 texcoord 网格、大网格、光照 shadow 语义：未覆盖（scene-a 两相机均透视、
  网格无 texcoord）。
- `bvh_id` 为自有打包，**不宣称** Warp BVH ABI/等价；81 槽只保证签名与槽序一致。
- 相机 data 位姿由调用方显式传入，跟踪相机 TARGETBODY 的产品接线属 S8，不在本片。

## 7. 证据文件

- `receipt.json`：机器可判定回执；`qa-artifact-manifest.json`：QA 运行产物（target/）哈希绑定。
- `judge_fixtures.json` / `judge_run_base.json` / `judge_run_base_qa.json` / `judge_run_pose.json`
  / `judge_run_shin.json` / `judge_run_shin2.json` / `judge_light.json` / `judge_slots.json` /
  `judge_hashes.json` / `judge_shininess.json` / `judge_evidence.json`：各项独立判定结果。
- `harness_base/pose/shin/shin2.json`：QA harness 报告（含 PTX 事实、位姿位、灯位姿位）。
- `megakernel.ptx`：产品 A′ 门禁编译产物（195,594 B）。
- `reject.json`：4 类写前拒绝的独立判定。
- `qa-gpu-runs.log`：S7b2/S7b1/S6b 交付用例原样重跑输出；`qa-unit-tests.log`：532/0 全量单测。
- `reference-megakernel-headlight-excerpt.txt`：参考 megakernel 源中材质高光指数与 headlight
  调用实参摘录（loc 与 sha 见文件头）。
- 脚本：`capture_qa.py`、`judge_*.py`、`qa-harness-main.rs.txt`（QA 全部自写；评估口径见 §1）。
