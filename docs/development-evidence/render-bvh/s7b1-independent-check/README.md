# 渲染/BVH S7b1 独立复核（depth/seg 0 ULP / 球面 fma 顺序独立反推 / W15 门禁 / 回归）

- 任务：`2026-10-25-render-bvh-s7b1-independent-check`（QA）
- 复核对象：`3e43278`（S7b1 功能提交，父提交 `ceb955a`），基线 HEAD `74928dd→04f5ceb→…`（见 receipt）
- 日期：2026-10-25
- 结论：**验收面全部成立（0 阻断）**。QA 自己解码两份夹具（不读交付断言），用独立 harness 复算：对**同配方参考**（`use_fast_math=False`）两相机 **120,000/120,000 像素 depth 逐位相同（max 0 ULP）**、seg 逐像素全等；dynamic≡precomputed、双 world 零跨、refit 两次位等、DisabledOutput、A′ 门禁过、S6b 回归 5/5、532/0。**并且独立完成了 fma 顺序的反推**：从参考 megakernel PTX 取球面求交的实际发射顺序，用自己的 f32 复现在 cam1 的 747 个球面像素上 **747/747 位等**；换一种看似等价的顺序只有 87/747 位等且 **187 像素 >4 ULP**；换一组新相机位姿后参考顺序 **813/813 位等**、另一种顺序 192 像素 >4 ULP → 根因是**运算顺序**，不是对着夹具调参。

## 1. 0 ULP 声明（独立解码 + 独立 harness，不读交付断言）

QA harness（`target/qa-s7b1-harness`，产品 crate path 依赖；复用交付 support 仅做夹具/上下文 plumbing，所有比较与 ULP 统计由 QA Python 自行重算）：

| 项 | QA 独立结果 |
| --- | --- |
| depth vs 同配方参考 | cam0 **76,800/76,800** 位等、cam1 **43,200/43,200** 位等，合计 **120,000/120,000**；ULP 直方图 `[76800,0,0,0,0,0]` / `[43200,0,0,0,0,0]`；max 0 ULP |
| seg vs 同配方参考 | 两相机逐像素 **0 mismatch**；深度零像素 16,995 / 27,498 与夹具一致 |
| dynamic vs precomputed | depth 逐位 **0 差异**、seg 0 差异 |
| 重复渲染 | 两次逐位一致（0 差异） |
| 双 world | world0 与单 world **逐位一致**；world1 **全 miss / 全背景** |
| refit（QA 自己移动网格顶点 + 显式 refit） | 两次渲染逐位一致；与非 refit 结果相比 changed_pixels=754（交付用例用自己的位移得到 1698，均证明变化非空且确定） |
| 禁用 depth/seg | 渲染成功但 `byte_len`/`read` 均 `DisabledOutput`；RGB 全 0 |
| 交付 gpu-run.log / receipt 数字 | 与 QA 实测一致（`depth_mismatch=0 max_ulp=0`、`seg_mismatch=0`、dynamic hash `03efa95416bbd008`、refit `changed_pixels=1698 deterministic=true`、双 world 两条断言） |

## 2. fma 对齐：独立从参考 PTX 反推 + f32 复现（核心新增复核）

### ① 从参考产物定位球面求交的实际发射顺序

参考 megakernel PTX：`target/render-bvh/s0b-cache/1.15.0/wp__build_megakernel__locals___render_megakernel_f77904cd_be6b38f/*.sm89.ptx`，**619,077 B、sha256 `9662200b14e1d5…`**（与 README 引用的参考产物一致）。QA 直接读取该 PTX（摘录见 `reference-ptx-excerpt.txt`）：

| 量 | 参考 PTX 实际序列（行号） |
| --- | --- |
| `a = dot(vec,vec)` | **L530 `mul.ftz.f32 %f4132, %f7, %f7` → L531 `fma.rn.ftz.f32 %f4133, %f8, %f8, %f4132` → L532 `fma.rn.ftz.f32 %f14, %f9, %f9, %f4133`**（0 号分量先 mul，1/2 号分量依次 fma） |
| `b = dot(vec,dif)` | **L940 `mul.ftz.f32 %f4241, %f8, %f4239` → L941 `fma.rn.ftz.f32 %f4242, %f7, %f4238, %f4241` → L942 fma comp2**（**1 号分量先 mul**，0/2 号分量 fma） |
| `dd = dot(dif,dif)` | **L943 `mul.ftz.f32 %f4243, %f4239, %f4239` → L944 fma comp0 → L945 fma comp2**（1 号分量先 mul） |
| `det = b*b - a*c` | L946-949 该内联实例为 `mul,mul,sub`（未融合）；**但启用融合的实例才是与夹具匹配的那一个**——由下方 f32 复现判定：融合 det 才 747/747 位等，不融合只有 87/747 |

补充：参考 megakernel 中球面代码被内联多次，L940-949 只是一处实例（其 det 为未融合）；QA 不依赖「读某一处」的结论，而是用**数值复现**判定 pass-relevant 实例的融合方式（下节），并用 `a` 顺序（L530-532）与 `b/dd` 顺序（L940-945）作为 PTX 侧证据。

### ② QA 自己的 f32 复现（独立实现）

复现链严格按产品/上游公式（`compute_ray` fovy 分支、`mat_vec` 世界变换、球面 world 空间求交、`depth = dist * -dir_local.z`）：

- f32 语义：`fma(a,b,c)` = 精确 f64 乘加后单次 f32 舍入；`sqrt/div/rcp` 用正确舍入；
- 相机变换先在**floor 平面像素**（10,720 个，解析平面深度）上标定：`mv` 顺序 = `fma(c0,d0, mul(c1,d1))` 再 `fma(c2,d2)` → **10,720/10,720 位等**；
- 球面像素（cam1，747 个，seg=(2,5)）：
  - **参考顺序**（a=comp0-first，b/dd=comp1-first，det 融合，den=rcp）→ **747/747 位等、max 0 ULP**；
  - **另一种看似等价的顺序**（所有 dot 都 comp1-first、det 不融合）→ 仅 **87/747** 位等，直方图 `[87,153,141,120,59,187]`，**187 像素 >4 ULP**。
- 结论：**顺序不同就会偏出几十 ULP**，参考顺序可以 100% 复现夹具 → 这是「运算顺序对齐」而非「对着夹具调参」。

### ③ 新射线位姿泛化（不是只对这份夹具成立）

QA harness 用同一产品 kernel、把两台相机位姿平移 `(+0.05,-0.03,+0.02)` 后重新渲染（新射线集），QA 再用同一 f32 复现比对：

- 新位姿下命中球面的像素 **813 个**：参考顺序复现 **813/813 位等（max 0 ULP）**；
- 另一种顺序：**192 像素 >4 ULP**。
- 说明球面顺序对齐在新射线下同样成立，**不是只对 fixture 特判/调参**。

### ④ 代码不是按像素/几何特判

- `render_math.cu::render_sphere_hit` 是纯算术：`r2=mul(radius,radius)`、`dif=pnt-pos`、`a=dot3_ch0(...)`、`b/dd=dot3_ba(...)`、`c=dd-r2`、`det=fma(b,b,-mul(a,c))`、`sqrtf`、`den=__frcp_rn(a)`、`x0/x1` 选择；**无 geom_id 白名单、无像素/坐标分支、无 fixture 常量**；
- `render.cu` 只把 `geom_type == 2` 的分支改调 `render_intersect_primitive`（球面走新函数），其余 primitive/mesh 路径未变；
- 未引入 fast-math（配方仍 `fmad=true`、`fast_math=false`）、未放宽容差（门禁与 4 ULP 预算均未动）。

## 3. A′ 门禁（成立）

- 渲染源码仍只经 `compile_render_artifact` 编译；PTX **137,824 B**、`source_sha256=82ea0d3b…`、`ptx_sha256=01018ed9…`，与交付一致；
- PTX **同时含** `.visible .entry scene_bvh_ray` 与 `.visible .entry render_depth_seg`；
- 全 PTX f64 全局 **恰 2 条**，逐字节等于 W15 `_svd_config<double>` 白名单（符号+字面量 `0d3D719799812DEA11`）；
- `src/render/bvh/cuda/scene.cu` sha256 `8dad4d61…` **未变**（S2b 证据一致）；`src/render/kernel/launch.rs` **恰 500 行**（棘轮）。

## 4. 跨配方偏差仍如实记录（未当通过依据）

- 产品 vs **S0b fast-math** 参考：cam0 直方图 `[57726,17681,1393,0,0,0]` max 2 ULP；cam1 `[39136,3677,382,3,2,0]` max 4 ULP、>4 ULP=0——与夹具 `cross_recipe_vs_s0b` 记录**逐项相同**；
- 由于产品已与同配方参考逐位相同，这组数字等于「两份参考彼此之间的 fast-math 编译噪声」；receipt/README 明确写为**已知偏差（记录项）**，不作为通过依据、不放宽容差。

## 5. 回归 / 证据 / 差异范围（成立）

- 交付 GPU 用例原样重跑：**6 passed / 0 failed**（4.35 s；含 NVRTC A′ 门禁用例 + 5 GPU 用例），日志与 receipt 数字一致；
- S6b 回归原样重跑：**5 passed / 0 failed**（7.62 s）；
- `cargo test --locked`：**532 passed / 0 failed**；
- 两份夹具 sha/bytes 与 receipt `references` 一致（`depth_seg.json` 1,491,460 B `97fc6838…`；同配方 1,492,762 B `d6ab6dde…`）；
- **`git diff ceb955a..3e43278` 只动本片路径**（`src/render/kernel/**`、`src/render/context/{lifecycle,owner}.rs`、`tests/render_kernel_depth_seg.rs`、`tests/support/render_bvh/s7b1.rs`、`fixtures/render-bvh/scene-a/s7b/`、`scripts/authoring/render-bvh-s7b/`、本片证据、`.normify`），**无 S2–S6 证据/夹具改动**（QA Python 检查 out_of_scope=[]）；
- GPU：窗口起点 3,406–3,412 MiB，GPU 用例结束回 3,406 MiB，无本任务计算进程。

## 6. GPU 窗口说明（如实）

GPU 实际占用只有两次测试运行（约 12 s）。QA 的 CPU 侧工作（f32 顺序复现、相机变换标定、新位姿泛化、PTX 反推）在 30 分钟窗口后继续，期间 **GPU 空闲、无其他任务运行**；没有超窗口占用 GPU。若需严格把窗口限定为 GPU 占用，本片满足（GPU 总计约 12 s）。

## 7. 未满足项 / 限制

- 球面顺序对齐的 PTX 侧证据取自 S0b fast-math megakernel（`9662200b…`）；同配方 PTX 也存在（`ad21b93f…`，618,684 B 磁盘字节），但 megakernel 内联多处球面实例，QA 以数值复现（747/747 位等）判定 pass-relevant 实例的融合方式，未逐指令反汇编全部实例；
- 相机变换（`mat_vec`）的收缩顺序由 floor 平面 10,720 像素标定（100% 位等）；球面顺序在其上复现，未单独证明每个 inlined 实例的顺序；
- 未覆盖：RGB/着色（S7b2）、AA/flag 变体（S7b3）、正交相机（scene-a 无）、SDF/flex/hfield/skybox/splat、逐 world 相机数据（参考 nworld=1）、产品跟踪相机接线（S8）；
- 单 fixture（scene-a 两相机）+ 一个翻译后的新位姿；无 fuzz/随机射线；
- 未跑完整 G01（fast-loop 口径）。
