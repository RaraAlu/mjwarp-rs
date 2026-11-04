# R06-S3b D8：cam2 depth (321,297) 缺口诊断 —— H1/H1b 分离（GPU dump 一次 + CPU 离线分析）

任务：`2026-11-04-render-r06-s3b-flex-cam2-depth-gap`（owner Backend，D7 修复后才可观测的 cam2 depth 红；先诊断、不改判据）。
依赖：D7 卡（`171ac222`，1D-only flex 上传回归修复）；上游判断：attempt4 = 设备侧失败、D7 修复后 cam2 depth 首次判据性红。
本片**零参考改动、零判据/容差改动、`src/` 零 diff**；GPU 只用一次 env-gated WIP dump（≤3 分钟，跑后还原，不提交 WIP）；其余全部 CPU 离线。

## 0. 结论（先给）

**H1 成立：cam2 depth 缺口是 1D flex 命中路径的 f32 算术未与上游位级对齐（codegen 级差异），不是候选/顺序选择缺陷（H1b），也不是 D5/D7/映射/深度语义残留。**

五条硬依据（详见 §4）：
1. **单边 flexid1（无关节、无同 id 不同 edge 的歧义）251/251 像素全部 bit 不等**——选择类缺陷无法解释这一条；flexid0（3 条边折线）只 70/403 不等。
2. **seg 与参考位级全等**（76,800 px 0 失配、hit/miss 0 差异）→ 命中几何/候选集一致。
3. 差异像素 **321/321 全在 flex 区内**、**300/321 距 flex 轮廓 ≤4 px**（中位 1.41 px）→ 近掠射/轮廓像素，正是 f32 系数 1 ULP 放大的位置。
4. 绝对深度差 **中位 2.38e-6 m、p90 1.14e-5 m、最大 3.70e-5 m**（<0.04 mm），ULP 长尾（5–1024 ULP），**无 >1024 ULP** → 同一命中面的数值尾差，不是换了命中面/root。
5. 与「同算法、不同 codegen」的 **CPU-warp 回放敏感集重叠 317/321 = 98.8%**（且 product 相等的像素在 CPU 回放里 ULP 中位数为 0）→ 差异像素就是 codegen 敏感像素。

判据面：**本片不放宽任何容差、不改参考、不宣称 pass/fail 结论**。「depth ≤4 ULP 在薄胶囊近掠射像素上近似等价于位级一致」按 Chief 指示作为 **B 类判据前提候选** 交 Chief/QA 按《机器判据裁定标准》处理。

## 1. 两段事实复核（attempt4 / D7 后）

| 段 | 事实 | 证据 |
| --- | --- | --- |
| attempt4（D7 前置） | 12:31 实体首步 `running 1 test` 后 panic `render-launch: transfer-wait CUDA 错误码 700`（illegal address）；rgb/depth/seg 未进入、两跑 sha 未产生 | `r06-s3b-flex-window/attempts/attempt-4-*`（`c7e8b1e9`）、D7 证据 `r06-s3b-flex-d7/`（`01082958`/`171ac222`） |
| D7 后短跑 | `running 1 test`、无 700；三相机 rgb 全过（≤1 LSB）、cam0/cam1 rgb+depth+seg 全过（depth `(0,0)` 位级）；随后 `tests/render_flex_validate.rs:347` 红：**cam2 depth `(mismatched, beyond4ULP) = (321, 297)`**；cam2 seg/两跑 sha 因断言顺序未评估 | D7 证据 `run1-fixed.log`/`run2-regression.log`（同判据复现 `(321,297)`） |

判据定义：`tests/render_flex_validate.rs:344` 计 `mismatched`/`beyond`，`:347` 断言两者为 0；`MAX_DEPTH_ULP = 4`（`src/render/kernel/output.rs:15`）。
参考侧 cam2：flex 654 px 深度 **1.88006771–2.22181678 m 全非零连续**；`depth=0` 的 47,725 px **全部非 flex**（背景/未命中）→ 不存在「参考侧 flex 深度为 0」的问题。

## 2. cam2 depth 输入链（定位）

| 段 | 位置 |
| --- | --- |
| 相机 data 位姿 provenance | `reference/base/cam-poses.json`（D5 卡 `ab8da091`） |
| 用例取位姿 | `tests/render_flex_validate.rs:245` `camera_poses()`（f32 位型）→ `:485` 传入 |
| host 打包 | `world_camera_pose_arrays` / `world_active_camera_arrays` → 上传 `cam_xpos/cam_xmat`（`megakernel_uploads.rs:307-308`） |
| kernel 世界变换 | `render_megakernel.cu:149-155`：`ray_dir_world = cam_xmat @ ray_dir_local_cam`，`ray_origin_world = cam_xpos + cam_xmat @ ray_offset` |
| 射线 | `RayBuilder`（S5；相机空间、与相机位姿无关）→ `ray`/`ray_offset` |
| 场景 BVH | host 场景叶 + 1D flex 叶（`one_dimensional_scene_leaves`，叶编码 `leaf_geom = -1 - flex_local`）→ `SceneDevice` |
| flex 命中 | `flex_leaf_hit`（`render_flex.cu:117`）→ `ray_capsule`（`render/bvh/cuda/scene.cu:131`）→ `hit.dist` |
| 深度写 | `depth = hit.dist * -ray_dir_local_cam.z`（`render_megakernel.cu:243-245`），first sample 才写，地址 `depth_adr[camid] + rayid_local` |
| 判据 | `compare()`（`tests/render_flex_validate.rs:314`）按相机顺序比 depth/seg；rgb 循环在前 |

## 3. GPU 微窗口与 dump 记账（一次成功、无重试）

- 前置：compute_filtered=**0**（其余为 C+G 图形上下文）、build 进程 0、util 2–4%<40、free **14,314 / 14,314 / 14,314 MiB**、temp 29 °C；树仅本卡 WIP（`tests/render_flex_validate.rs` env-gated dump，未提交）。
- 记账：committed test exe `49923e3e…` bytes 107,660,771；WIP test exe `8a38fc6c…` bytes 107,982,020；D7 后 guards 仍 `attempt=1 frozen`、16/16、exes=2、inputs=14、guards sha `70d4cbc6…`，其 frozen 测试 exe 条目 `d9686515…` bytes 107,672,581 → **live ≠ frozen**（attempt5 必须重新真冻结）。
- 运行：wall **2.02 s**（≤3 分钟）；GPU 前 `14318 MiB / 2 % / 29 °C` → 后 `14318 MiB / 8 % / 29 °C`（进程退出即释放）；**单次、无重试**。
- 复现：`tests/render_flex_validate.rs:347` **cam2 depth `(321, 297)`**，与 D7 短跑完全一致；两跑 hash 相同（rgb `a4f29b09…`、depth `fa4cceaa…`、seg `c4be9853…`）。
- dump（`target/render-bvh/r06-s3b-flex-window/d8/`）：`depth_all.f32le` **787,200 B** sha256 `fa4cceaaf7eeca70f3307ead99ad24f095727fdbb904d7de1c33e9eb9ffef20e`；`seg_all.i32le` **1,574,400 B** sha256 `c4be985324b883abcbaa63fa44888b016fd28d2d266858aaa493877b49a64185`；`meta.json` sha256 `a347b644…`（`depth_adr=[0,76800,120000]`、`seg_adr` 同、`cam_res`、两跑 hash）。
- 还原：跑后 `git checkout -- tests/render_flex_validate.rs`，树只剩本卡 JSON；WIP 未提交。原始 dump 保留在 target（不入库），cam2 部分压缩入库 `artifacts/cam2-depth-seg.npz`（34,405 B）。

## 4. H1 vs H1b 分离（dump 逐像素分析）

产品 cam2 depth vs 参考（32 位逐位）：

| 指标 | 值 |
| --- | --- |
| mismatched / beyond4ULP / maxULP | **321 / 297 / 259** |
| ULP 直方图（0、1、2–4、5–16、17–64、65–1024、>1024） | 76,484 / 7 / 19 / **172 / 109 / 33** / 0 |
| 绝对差（m） | 中位 **2.384e-6**、p90 **1.144e-5**、最大 **3.695e-5** |
| 非 flex 像素 mismatch | **0** |
| seg vs 参考（位级） | **0/76,800 失配**（flexid 集合、hit/miss 全一致） |
| 差异像素属地 | 全在 flex 区（321/321） |
| 差异像素距 flex 轮廓 | 中位 **1.41 px**；≤2 px **222/321**；≤4 px **300/321** |
| flexid0（3 边折线，含关节） | 403 px：mismatch 70、beyond4 64、maxULP 259 |
| **flexid1（单边胶囊，无关节/无 edge 歧义）** | **251 px：mismatch 251、beyond4 233、maxULP 155** |
| CPU-warp 回放敏感集重叠 | **317/321 = 98.8%**；CPU 回放 ULP 在 product-mismatch 像素中位 **19**、在 product-equal 像素中位 **0** |

判定：
- **H1b（flex 候选顺序/近平局 tie、同 id 不同 edge）被否**：flexid1 只有一条边、无关节重叠，却 251/251 全不等；flex0 的差异也没有聚集在关节（距 v1/v2 投影 ≤4 px 仅 17/321，中位 49.9 px），seg 位级全等。
- **H1（1D 命中算术未位级对齐）确认**：差异是 ULP 长尾（最大 259 ULP、绝对值 <0.04 mm）、只出现在近掠射轮廓像素、且与「同算法不同 codegen」的 CPU 回放敏感集 98.8% 重叠。机理与既有 U4 sphere 轮廓 RCA 同型：`render_math.cu:247-262` 已记录「quadratic 系数 1 ULP → 轮廓 depth 7–191 ULP」，当时用显式 `__fmul_rn`/`__fmaf_rn`/`__fsub_rn` 钉死点积次序与 `det` 融合后才对齐；**flex 1D 胶囊路径尚未做同类对齐**。
- 为什么 rigid 路径没有这个问题：D7 短跑中 cam0/cam1 depth 位级全过（`(0,0)`）→ 产品 rigid 路径已与上游位级一致；缺口是 flex 专属。

## 5. 假设排除表

| 假设 | 结论 | 依据 |
| --- | --- | --- |
| H1 flex 命中算术未位级对齐 | **成立（主因）** | §4：单边 flexid1 全不等、ULP 长尾、轮廓聚集、CPU 回放敏感集 98.8% |
| H1b flex 候选顺序/近平局 tie | **否** | seg 位级全等；单边 flexid1 无歧义仍全不等；差异不聚关节（17/321≤4 px，中位 49.9 px） |
| H2 D5 残留（相机位姿） | 否 | 三相机 rgb 全过；cam0/cam1 depth 位级过；D5 CPU 断言位级相等（D8 run log 打印） |
| H3 D7 残留（flex 上传/几何） | 否（几何） | seg 位级全等；D7 witness；CPU 自检 |
| H4 depth 语义/公式 | 否 | 上游 `render.py:1285-1294` 与 `render_megakernel.cu:243-245` 逐式一致；cam0/1 位级过 |
| H5 参考/期望口径 | 否 | 参考 flex 深度全非零连续；`depth=0` 47,725 px 全非 flex |
| H6 BVH leaf AABB（宿主 f32 vs 上游 GPU） | 否（至少在 hit 层面） | seg 位级全等；非 flex 0 mismatch |
| H7 depth 槽/相机映射 | 否 | `depth_adr=[0,76800,120000]` 与参考形状一致；cam0/1 位级过；cam2 rgb 过 |

## 6. 判据面（交 Chief/QA 裁定，本片不改）

- 冻结判据 `depth ≤4 ULP` 对 rigid 路径是可满足的（产品已位级），对 1D flex 薄胶囊（r=0.02/0.04、多数像素近掠射）**实际要求位级一致**：CPU-warp 同算法回放也造成 597/654 flex 像素 >4 ULP。
- 本片只把它记为 **B 类（参考-前提型）判据前提候选**：若后续按 A 类修 flex 位级对齐，则判据可保持；若判据面改为「flex depth 允许更大容差/单独声明」，属签名级裁定，由 Chief/QA 处理。**本片未放宽、未改参考、未宣称 pass/fail。**

## 7. 修法方向（不实施；另立 fix 卡）

候选差异面（flex 1D 命中链）：
1. `render_flex.cu`：`warp_length3`（`sqrtf(fma(z,z,fma(y,y,x*x)))` 的次序是否与 warp `vec.h length` 的 NVRTC 收缩一致）、`flex_quat_z2vec`（`length`/`div`/`atan2f`/`sinf`/`cosf`）、`flex_quat_to_mat`（与 `quat_to_mat` 逐位置一致但 NVRTC 收缩未钉）、`size[1]=0.5*length`、`pos=0.5*(v0+v1)`。
2. `render/bvh/cuda/scene.cu`：`ray_map`（`mat_t_vec` 行点积次序）、`dot`、`ray_quad`（`det=b*b-a*c` 融合次序）、cap 四段 `b/c` 点积次序。
3. 方法照 U4 sphere RCA（`render_math.cu:247-262`）：枚举「点积次序 × det 融合 × 归一化写法」变体，用本 dump 的 cam2 depth bits 与参考 bits 逐像素对拍，找唯一能位级复现的变体（当时 432 变体只有 1 个能复现 44/44 像素）。
- 该 RCA 需要产品侧变体运行或把「每像素 ray 输入」一并 dump；**建议下一次微窗口 dump 增加 `ray_local`/flex 输入字段**，避免多轮试错。需 Chief token。

## 8. 复现

```
# GPU 一次（WIP env-gated dump；跑后立即 git checkout -- tests/render_flex_validate.rs）
$env:PATH="D:\Rust\mjwarp-rs\target\toolchains\cuda-12.8.1\bin;$env:PATH"
$env:MJWARP_S3B_D8_DUMP="1"
$env:MJWARP_S3B_D8_DIR="D:\Rust\mjwarp-rs\target\render-bvh\r06-s3b-flex-window\d8"
cargo test --locked --features cuda-probe --test render_flex_validate -- --ignored --test-threads=1

# CPU 离线
target/mjwarp-parallel/venv/Scripts/python.exe -B docs/development-evidence/render-bvh/r06-s3b-flex-cam2-depth-gap/scripts/analyze_dump.py
target/mjwarp-parallel/venv/Scripts/python.exe -B docs/development-evidence/render-bvh/r06-s3b-flex-cam2-depth-gap/scripts/build_evidence.py
```
环境：python 3.12.13 / numpy 2.5.4 / warp 1.15.0 / mujoco_warp 3.15.0（pinned `71da24d…`）/ RTX 4070 Ti SUPER driver 596.36。

## 9. 非声明与边界

- 不宣称 flex 渲染通过/不通过；不宣称 attempt4/5 转绿；不改判据/容差/参考/guards；不重冻；`src/` 零 diff。
- 本片证据只覆盖「cam2 depth (321,297) 的 H1/H1b 分离」；修复与 attempt5 另行立项，attempt5 仍需：修法卡 verified + QA pass + 重新真冻结 16/16 + Chief token。
- 产物清单见 `logs/dump-sha256.txt` 与 `manifest.txt`（`artifacts/cam2-depth-seg.npz` 为 cam2 产品深度/seg 压缩副本，供后续 RCA；原始三相机 dump 在 target，未入库）。
