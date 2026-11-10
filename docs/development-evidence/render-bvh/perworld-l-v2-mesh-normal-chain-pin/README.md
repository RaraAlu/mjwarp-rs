# P10 mesh 法线链修复（局部归一化恢复 + mat 展开 x 项先舍入；双内核身份新基线；零 GPU）

- 任务：`2026-11-11-render-perworld-l-v2-mesh-normal-chain-pin`（owner Backend；p97；P8-渲染与BVH）。依据 Chief 2026-11-11 07:31 派工与 Arch 离线重放结论（4/4 复现参考 normal）。
- 性质：**实现修复**（对齐冻结上游的 mesh 法线链），不改判据/参考/容差；零 GPU、不跑诊断轮、不宣称修复生效（需诊断轮 #8）。
- 结论口径：G-L2 判定维持**未通过（判据 A FAIL）**；本卡只把「已定位的首个分岔站点」按上游事实修正。

## 1. 改动清单（3 个 `.cu`；无新增 `.cu`，精度源面 42 文件不变）

| 文件 | 改动 |
| --- | --- |
| `src/render/kernel/cuda/render_math.cu` | ① `mesh_query_ray` 尾部恢复 **局部归一化** `*out_normal = normalize3_mesh(min_normal);`（并改写 H2 注释为上游事实）；② 新增 `mat_vec_xpref`（上游 `mat.h:3180 mul(mat,vec)` 的 **x 项先单独舍入**） |
| `src/render/kernel/cuda/render_common.cu` | 第 225 行 mesh 法线消费点：`normalize3_mesh(mat_vec(mat, n_local))` → `normalize3_mesh(mat_vec_xpref(mat, n_local))` |
| `src/render/kernel/cuda/render.cu` | 第 231 行同一消费点（depth_seg 拼接）：`mat_vec` → `mat_vec_xpref` |

old(HEAD `61dd32c6`) → new(工作树)：

| 文件 | old sha256 / bytes / lines | new sha256 / bytes / lines |
| --- | --- | --- |
| `src/render/kernel/cuda/render_math.cu` | `4f103495f4cc7c9a5da53f4b18e612224a1302a77092a0704bdf4fabca8780a0` / 12566 / 310 | `c26e116e1d6dd0518a20fd78ed55a992085b6ed385b2cdc9ef7aa3bf6152d5cc` / 13685 / 326 |
| `src/render/kernel/cuda/render_common.cu` | `9116ce26ae60542ee8f89de9067528cda47a28b693d27a67923135827e4fd434` / 22348 / 498 | `65310e906e90c084a88312302e03603cba9f805cfb036ff124a83838b06e15df` / 22354 / 498 |
| `src/render/kernel/cuda/render.cu` | `91359916c300ca19b0bb45201f1904aafa1a7292b017c06b43241b709f4b1f68` / 22027 / 496 | `8b390bbb2ebbd32febcaa8c38ccf073e8330a4a89b9560726e7f26fe42c0d791` / 22033 / 496 |

**不动**：`src/render/bvh/cuda/scene.cu`（`mat_vec`/`mat_t_vec` 保持 S5 已验路径）、`render_megakernel.cu` 顶点法线链（`mat_vec` + `normalize3_midpref`）、`normalize3_mesh` 本体、`render_shadow.cu`；`fixtures/`、`scripts/`、`tests/` 零改动。

## 2. 根因与 H2 前提作废

- 上游 `wp.mesh_query_ray`（warp/native/mesh.h:1880）返回前对**局部法线**做一次归一化；消费端随后 `mat @ n` 与再一次归一化 ⇒ 上游 mesh 法线链 = **两次 normalize**（局部 normalize → mat@n → 再 normalize）。S3 只读副本 `src/render/bvh/cuda/mesh.cu:290` 即为 `*out_normal = normalize3(min_normal);`。
- `f728b2d7` 基于错误的 H2 前提（「cross 未归一化 → 消费端 mat@n → normalize 一次」）把 `render_math.cu` 里这次局部归一化删掉了，使产品链少一次归一化；同时 `mat @ n` 用了 `scene.cu::mat_vec`（独立舍入项是 **y**：`fmaf(m[2],a.z, fmaf(m[0],a.x, m[1]*a.y))`），而上游 `mat.h:3180 mul(mat,vec)` 的独立舍入项是 **x**。
- 本卡恢复局部归一化并新增 x 项先舍入的专用 `mat_vec_xpref`，与 Arch 离线重放 4/4 的参考链一致；**H2 说明作废**（`render_math.cu` 尾部注释已改写）。

## 3. 双内核身份（`render_math.cu` 同时进入两个拼接 ⇒ 两个内核都换新基线）

| artifact | old（HEAD `61dd32c6`） | new（工作树） |
| --- | --- | --- |
| megakernel stitched source | 109,384 B / `624ab298e82e7c94dc66b101ca5e2abd141556beaf2d53485f0b3b8c4f6bb3e2` | **110,509 B / `6b1ade37958facfd48f485a7433f93bcf7256a6b08a167c1ece078e0744de278`** |
| megakernel PTX | `1fd531314875e52a08ffda3fd63e93184e46ebda207378d5378c25b878c430b8` / 347,070 B | **`48f8e7f820f7a0a931a82e07efbebef07b831b3895bb7c6423dcd4975c31fa02` / 348,399 B** |
| depth_seg stitched source | 52,427 B / `b546ef08bbb64f756b2de7fa9809da7d9918dc6a3bb16e07945de0714114d931` | **53,552 B / `1e3f273c88a77235640c44b2e3d52e39984cf659da87e6ff1dd237d8c871e1b9`** |
| depth_seg PTX | `e8d84963285a2a6982d99a51c95786c6a9898883c6a6031e639983ead174e5c7` / 138,540 B | **`31912377f59305e85b164da3d8b487ad127b6c262aa3792b99ecdb1b17c1cf50` / 138,857 B** |
| aa_resolve | `0bf8af5a…` / `b28e155c…` / 27,536 B | 未变（不在拼接中） |

- 采集方式：identity 用例（NVRTC 纯编译，无 GPU 会话），old 在改动前从 HEAD 采集（`logs/old-identity-*.log`），new 改动后采集（`logs/new-identity-*.log`）；离线拼接复算与 identity 逐位一致（`logs/` 中 stitched 源 sha 相同，见 §4）。

## 4. 离线 NVRTC 证明（零 GPU）

**方法**：pinned CUDA 12.8.1 NVRTC，product 选项（`--fmad=true --include-path=include/warp-headers --std=c++17 --gpu-architecture=compute_70 --device-as-default-execution-space`）；old = HEAD blobs 拼接，new = 工作树拼接（拼接顺序同 `mod.rs`）；另用**隔离探针**（函数从真实文件逐字节抽取，见 `logs/offline-probe-*.cu`）在无控制流干扰下读取精确链序。

### 4.1 产品 PTX 的算术增量（`logs/ptx-arith-delta.txt`）

| 内核 | old 算术 op | new 算术 op | 增量（按 opcode） | 解释 |
| --- | --- | --- | --- | --- |
| megakernel | 3,094 | 3,122 | `div +12, fma +8, mul +4, sqrt +4`（其余 f32 opcode 全 0） | **+28 = 4 × 7**：4 个 inlined mesh-normal 实例各恢复一次 normalize（mul+fma+fma+sqrt+div×3） |
| depth_seg | 1,201 | 1,208 | `div +3, fma +2, mul +1, sqrt +1`（其余全 0） | **+7 = 1 × 7**：1 个实例 |

- `sqrt` 计数 62→66（mega）/ 28→29（depth）分别对应 4 / 1 个受影响实例；**除「恢复的一次 normalize」外没有任何算术 opcode 变化**——`mat` 展开的 y→x 独立项切换是同一组 mul/fma 的重排（opcode 多重集不变，见探针）。

### 4.2 探针链序（`logs/chain-sequence.md`，指令逐条引用）

- **OLD（16 f32 ops）**：3 行 `mul → fma → fma` 中**首乘用 `[mat+4]×[n+4]`（m[1]·a.y）**⇒ y 项先单独舍入；随后**一次** normalize（`mul,fma,fma,sqrt,div,div,div`）。
- **NEW（23 f32 ops = old + 7）**：先对三角面法线做局部 normalize（`mul,fma,fma,sqrt,div,div,div`）→ `mat` 展开 3 行 `mul → fma → fma` 中**首乘用 `[mat+0]×n_local.x`（m[0]·n_local.x）**⇒ **x 项先单独舍入**，再 fma y、fma z → **第二次** normalize。
- 结论：离线 PTX 逐指令确认新链 =「**局部 `mul→fma→fma→sqrt→div` 归一化 → `mat` x 项先舍入 → 再归一化**」，与上游 `mesh.h:1880` + `mat.h:3180` 链一致。

## 5. depth/seg 口径（预期与实测）

- **预期不变**：depth/seg 数值不依赖 normal 的数值，只依赖 `dot(lvec, n_local)` 的**符号**（背面剔除），而按正长度归一化不改变符号；seg 同样只用 geom/face/sign。
- **实测**：`cargo test --locked --features cuda-probe --test render_kernel_depth_seg`（冻结参考用例）**13 passed / 6 ignored / 0 failed**（`logs/depth-seg-frozen-regression.log`）；`render_kernel_flags` **12 passed / 6 ignored / 0 failed**（`logs/kernel-flags-regression.log`）。
- 注意：depth_seg 的 **source/PTX 身份已换新基线**（§3），这是 `.cu` 进入两个拼接的必然结果；数值预期与冻结参考一致已由测试确认。

## 6. 验证（零 GPU）

| 项 | 结果 |
| --- | --- |
| `cargo build --locked --features cuda-probe` | exit 0（`logs/build-cuda-probe.log`） |
| `cargo test --locked --features cuda-probe --no-run` | exit 0（`logs/no-run-cuda-probe.log`） |
| 四门禁 fmt / clippy / clippy-cuda / unit | 全过；unit **1,391 过 / 0 败 / 402 ignored** |
| identity 用例（NVRTC 纯编译） | megakernel/aa_resolve 1 passed；depth_seg 1 passed；0 failed |
| 冻结回归 | depth_seg 13 passed/6 ignored；kernel_flags 12 passed/6 ignored |

## 7. 边界与非声明

- 边界：只改 3 个 `.cu` + 新增本证据目录；`fixtures/`、`scripts/`、`tests/`、`src/render/bvh/`、`render_megakernel.cu`/`render_shade.cu`/`render_flex.cu`/`render_shadow.cu`/`aa_resolve.cu`、guards/冻结件、已签材料**零改动**；零 GPU、不采集、不跑诊断轮、不重冻；不新增 `.cu`（精度源面 42 文件不变）。
- 非声明：**不宣称修复生效**（4 px 是否归零需诊断轮 #8 实测）；不改判据/阈值/参考/容差/相机/recipe；G-L2 判定维持未通过（判据 A FAIL）；不改写历史提交；不宣称渲染数值等价。
