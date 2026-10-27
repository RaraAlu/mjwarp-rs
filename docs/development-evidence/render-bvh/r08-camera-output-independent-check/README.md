# R08-S1 独立复核（QA）：逐相机输出视图

- 任务：`2026-10-28-render-bvh-r08-camera-output-check`（复核对象 `2026-10-28-render-bvh-r08-camera-output`，提交 `a6425a5d`，父 `db9abb09`）
- 结论：**pass / 阻断 0 / 观察 8（8 建议）**；只读复核，未改代码/判据/参考/容差，未开 GPU、未重渲染、未重捕获
- 时间：2026-10-28（空闲时段）

## 1. 总体判定

S1 在既有整缓冲 + `OutputLayout` 上新增的宿主侧逐相机视图，**语义与冻结上游 `render_util.py`（`71da24d`）
逐条对得上**；参考对账的每一项数值都由我用 **python/numpy 独立实现**重算、再用独立 Rust harness 与产品
输出**逐位比较**，五类计数全部相等（packed 120000 / unpacked 360000 / depth raw 120000 / depth scaled 360000 /
seg 120000）；错误面 9 类 + 组合、空/单相机、clamp 边界、两相机异构分辨率隔离全部按预期返回、无 panic；
`#[path]` 接线方式经核对无重复定义、`context.rs` 未动、模块源已登记、架构指纹校验 0 错误；四门禁在
**源码内容等价的独立 worktree** 复跑通过（unit 1010/0/376）。8 条观察均为建议级，不阻断。

## 2. 语义逐位核对（对照 pinned 上游）

| 上游 | 位置 | 语义 | 产品 | 结论 |
| --- | --- | --- | --- | --- |
| `unpack_rgb_kernel` | `render_util.py` L170–189 | `b=f32(val&0xFF)*(1/255)`、`g=(val>>8)`、`r=(val>>16)`；输出 `vec3(r,g,b)` | `unpack_rgb` L478–484 | 位序/掩码/常量一致 |
| `extract_depth_kernel` | L193–209 | `wp.clamp(val / depth_scale, 0.0, 1.0)`（**除法在前**） | `depth()` L344–347、`clamp_unit` L486–495 | 先除后 clamp 一致 |
| `_extract_seg_kernel` | L247–261 | `seg_data[world, seg_adr[camera_index]+pixel]` 原样 `(id,type)` | `segmentation_raw()` | 原样切片一致 |
| `get_rgb/get_depth/get_segmentation` | L212/L228/L264 | 以 `camera_index` 取 `*_adr[camera_index]` 起 `pixels` 个 | `region()` L418–453 | 一致 |
| 像素顺序 | 夹具 `encoding.pixel_order` | row-major（y 外、x 最快） | `CameraImage` 行主序、`get(x,y)=y*width+x` | 一致 |

- `INV_255 = f32::from_bits(0x3B80_8081)`（`src/render/texture.rs:25`）；我独立算 `float32(1.0/255.0)` 位型
  = **0x3B808081**，与 `(1.0f64/255.0) as f32` 一致（`python_expected.json.inv_255_bits`）。
- `camera_index` = **激活顺序位置**：`CameraPlan.cameras` 是 `cam_active` 选中顺序（`camera.rs:238`、
  `resolve_cameras` L285–287），`OutputLayout::resolve` 按 `plan.resolutions` 枚举写 `adr[position]`
  （`context/output.rs:48–87`）；上游亦按 `cam_active` 顺序建 adr 表。哨兵 `cameras=[1,0]` 实测
  `camera_id(0)=1`、`camera_id(1)=0`。
- clamp 边界语义：warp 1.15 `builtins.py`（`clamp` = `min(max(low,x), high)`，`max` 为 C `fmax` 语义、
  NaN 忽略）⇒ `clamp(NaN)→low=0`、`clamp(±inf)` 收敛、`<0→0`、`>1→1`；我的 harness 用 8 类边界值
  （0/-0/0.5/1/1.5/-2/±inf/NaN）在 3 个 scale 下逐位验证产品实现与文档语义一致。

## 3. 参考反解对账（独立实现，不重渲染）

方法：从 S7b 冻结参考（`rgb_fast_math_false.json`、`depth_seg_fast_math_false.json`，authoring 期 pinned
上游 GPU 采集）按 adr 顺序重建整缓冲 → 用 **python/numpy float32 独立实现** 生成期望值 →
独立 Rust harness（`harness/qa_r08_s1_cross_check.rs`）调用产品公共 API 逐位比较。

| 项 | 我的计数 | 结果 |
| --- | --- | --- |
| RGB packed 区段 | 76,800 + 43,200 = **120,000** | **120000/120000 逐位相等** |
| RGB 解包每通道（r/g/b） | 120,000 × 3 = **360,000** | **360000/360000 逐位相等** |
| depth 原始位型 | **120,000** | **120000/120000 逐位相等** |
| depth 缩放（scale 1.0 / 0.1 / 100.0） | 120,000 × 3 = **360,000** | **360000/360000 逐位相等** |
| seg 对 `(id,type)` | **120,000** | **120000/120000 全等** |

交叉不变量（我独立重算，与 Backend 数字一致）：
背景 `(-1,-1)` = depth 零像素 = **16,995 / 27,498**（无 `-0` 位型）；scale=1.0 时 clamp 到 1.0 的像素
**12,890 / 15,702**；原始深度最大 **1.6825796365737915 / 3.703162670135498 m**；分辨率
`[[320,240],[240,180]]`、`rgb/depth/seg_adr=[0,76800]`、总像素 120,000。

## 4. 错误面与隔离（独立 harness，98 项检查 0 失败）

- 9 类错误逐项触发且不 panic：`CameraIndex`（2/9/`usize::MAX`）、`NworldUnsupported`（0/2/3）、
  `ChannelDisabled`（rgb/depth/seg 各自，且同 plan 的相机 1 区段仍正确）、`BufferMissing`（三通道）、
  `BufferLength`（短 1 / 长 1）、`BufferRange`（`adr[1]=100000 → end=143200 > 120000`）、`DepthScale`
  （0/-0/-1/±inf/NaN；次正规与 `f32::MAX` 接受）、`CameraCount`/`PlanFlags`/`LayoutPixels`。
- 空相机列表：`camera_count=0`、`camera_id(0)`/`rgb(0)` → `CameraIndex{0,0}`；单相机正常。
- clamp 边界：见 §2。
- 隔离哨兵：2 相机 **4×3 + 2×2**、`cameras=[1,0]`、`adr=[0,12]`；A 视图恰 `[0,12)`、B 恰 `[12,16)`，
  行主序映射与边界 `None` 正确，depth/seg 同样按相机区段化，无跨相机污染。

## 5. 接线方式（本次特有点）

- `src/render/mod.rs` 仅 +4 行：`#[path = "context/camera_output.rs"] pub mod camera_output;` + 一行再导出；
  `src/render/context.rs`（426 行）**未出现在提交 diff**；`context.rs` 的 `mod` 列表不含 `camera_output`
  （L18/20/23/24/26/27），故 **无重复定义**，模块路径为 `mjwarp_rs::render::camera_output`（顶层再导出）。
- `.normify/modules/mjwarp/render.md`：`[[source]]` 共 **26 项**且含 `src/render/context/camera_output.rs`；
  `normify validate --fingerprints` = **错误 0**（5 条为既有告警）；`normify check`（S1 任务）= 通过。

## 6. 不变量

`src/render/context/camera_output.rs` **495** 行、`tests/render_camera_output.rs` **493** 行、
`src/render/mod.rs` **374** 行、`src/render/context.rs` **426** 行（未动）、`src/model/render.rs` **500** 行（未动）；
提交 18 文件、0 个 `fixtures/ / src/render/kernel / native/ / include/ / src/runtime/ / Cargo* / build.rs`
改动；`DEVICE_SOURCES=28`、六入口、`CollisionArgs` 504 B 断言随 unit 通过
（`abi_collision_args_matches_device_layout_504 ... ok`、`all_six_entries_take_args_by_value ... ok`）。

## 7. 门禁独立复跑

主工作区在我复跑期间出现 Backend 的 S2 在编文件（`tests/render_r08_cameras.rs` 等未跟踪），
`cargo test --locked` 会编译到它们，因此我在 **detached worktree**（`target/qa-r08-s1-wt`，指向 `a6425a5d`，
工作树干净）用独立 `CARGO_TARGET_DIR` 复跑同一组命令：

| 门禁 | 命令 | 结果 |
| --- | --- | --- |
| fmt | `cargo fmt --check` | exit 0（日志 0 字节） |
| clippy | `cargo clippy --all-targets --locked -- -D warnings` | exit 0 |
| clippy-cuda | `cargo clippy --all-targets --locked --features cuda -- -D warnings` | exit 0 |
| unit | `cargo test --locked` | exit 0：**1010 passed / 0 failed / 376 ignored**，124 个 result 行 |
| 定向 | `cargo test --locked --test render_camera_output` | **4 passed / 0 failed**，0.09 s |

注意事项（见观察 O8）：fresh checkout 因 `core.autocrlf=true` 会把缺少 `eol=lf` 属性的
`fixtures/g10-p5-primitive-branches/*.xml` 检成 CRLF，导致既有 `fixtures_match_s3_signature` 失败；
我把私有 worktree 的**已跟踪文件字节**对齐主工作区（仅行尾差异，5042 个文件；S1 三个源码文件内容
`git diff a6425a5d` 为空）后四门禁复跑通过。该问题**非 S1 引入**，是 checkout 可复现性缺口。

本复核任务本身无门禁项（checks 只有 `json` + `forbid_claim`）：`normify gate` 返回
「任务没有引用任何门禁」**exit 1 属预期**，receipt 已记录 exit 与原文。

## 8. 证据与复现

- `recompute_reference.py`（44 项检查 0 问题）→ `python_expected.json`（我的期望值 dump 1,080,000 个数）；
- `harness/qa_r08_s1_cross_check.rs`（临时测试源，复制到 `tests/` 后 `cargo test --locked --test qa_r08_s1_cross_check -- --nocapture`，跑完删除）→ `harness_cross_check.json`（45 项检查、五类计数）、`harness_edges.json`（98 项检查）；
- `logs/`：`recompute-python.log`、`qa-cross-check.log`、`directed-render-camera-output.log`、`fmt-check.log`、
  `clippy.log`、`clippy-cuda.log`、`unit.log`、`module-wiring.txt`、`gates-summary.txt`、`evidence-selfcheck.txt`；
- `verify_evidence.py` → `manifest.json`（本目录文件 sha256/bytes/lf_lines）。
- 机器可读例外：`python_expected.json`、`harness_*.json`、`manifest.json`、`logs/unit.log`（>500 行）。

## 9. 观察（8 条，均建议级，给修/接受建议）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | 建议 | `camera_id()` 用 `plan.cameras[i] as i32` 直接窄化（L294），无 checked 转换 | 接受（id 域受模型 ncam 限制）；下次改动时改 `i32::try_from` 更稳 |
| O2 | 建议 | `depth()` 先取区段/缓冲再校验 `depth_scale`，故「未启用/缺失/长度不符」优先于 `DepthScale`（harness 实测 disabled+scale=0 → `ChannelDisabled`） | 修（先校验 scale，或文档写明错误优先级）；成功路径无影响 |
| O3 | 建议 | README/回执把「`-0.0 → +0.0`」写成上游 `wp.clamp` 语义；warp 文档只保证 clamp=min(max)、max 为 fmax；±0 并列结果属 `fmaxf` 实现细节，夹具无 `-0` 样本、未设备侧验证 | 改措辞（标为「按 CUDA fmaxf：+0」，登记未覆盖项）；不影响已验数值 |
| O4 | 建议 | `new()` 只校验相机数/开关表/像素总数/缓冲长度，不校验 adr 表是否为前缀和：等总长的置换 adr 会通过并静默产生错切片 | 接受（README 已写「须来自同一次解析」前置）；如需硬化可加一次廉价前缀和自检 |
| O5 | 建议 | 文件在 `src/render/context/` 但模块声明在 `src/render/mod.rs`（`#[path]`），模块路径 `render::camera_output` 与目录树不一致 | 记录+后续片：scope 放宽时把声明移回 `context.rs`（或移动文件），消除结构债 |
| O6 | 建议 | `PixelOverflow` 在 64 位目标实际不可达（u32 宽高乘积 < `usize::MAX`），无测试覆盖 | 接受（保留 32 位/未来防御）；不必补测 |
| O7 | 建议 | 产品自带测试的 `expected_depth` 用 `f32::clamp`，未覆盖 NaN/`-0` 语义（夹具值域无此样本） | 修（把 2–3 个边界值加进产品测试）；或登记未覆盖项 |
| O8 | 建议 | fresh checkout 在 `core.autocrlf=true` 下把 `fixtures/g10-p5-primitive-branches/*.xml` 检成 CRLF，既有 `g10_p5_branch_closure_cuda::fixtures_match_s3_signature` 因而失败（**非 S1 引入**） | 修（`.gitattributes` 给该目录加 `text eol=lf` 或 `-text`，独立小任务）；本次门禁改用字节对齐的私有 worktree 复跑 |

## 10. 边界与不宣称

只读复核：未改任何代码/判据/参考/容差，未开 GPU、未重渲染、未重捕获、未重跑 S7b 采集。本片只核
「宿主侧视图语义、参考反解、错误面、隔离、接线与不变量」，不宣称渲染级/产品级验收、覆盖、精度或
等价结论，也不替代 S2 的端到端窗口。
