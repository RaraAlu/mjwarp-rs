# QA 复核：P10 mesh 法线链修复（`b0e372be`）

- 任务：`2026-11-11-render-perworld-l-v2-mesh-normal-chain-pin-check`（owner QA；零 GPU、只读复核）。
- 对象：修复卡 `2026-11-11-render-perworld-l-v2-mesh-normal-chain-pin`，提交 **`b0e372be`**（27 files / +82−8；代码仅 3 个 `.cu`）。复核 HEAD `b0e372be`。
- **结论：PASS（0 阻断，2 条信息级观察）→ 可发诊断轮 #8 token。** 改动面恰为 3 个 `.cu` 的 3 处 + 注释；`mat_vec_xpref` 确为 x 项先单独舍入且行主序索引与 `mat_vec` 一致；**双内核身份 4 个 source/PTX 值我全部独立复算逐位一致**，PTX 算术增量逐 opcode 与声明完全一致；`fixtures/`/`scripts/`/`tests/` 零 diff、无新增 `.cu`、guards/冻结件未动；冻结回归测试我重跑 **13 passed/6 ignored**（depth_seg）与 **12 passed/6 ignored**（flags）；证据 manifest 17/17、编码干净；口径正确（不宣称修复生效、H2 作废只报告不改历史、G-L2 维持未通过）。

## 1. ① 改动面 ✅

- `b0e372be` 的源码改动**只有** 3 个 `.cu`：`render_math.cu`（+22/−3：恢复 `mesh_query_ray` 尾部 `*out_normal = normalize3_mesh(min_normal);`、新增 `mat_vec_xpref`、H2 注释改写）、`render_common.cu`（:225 `mat_vec`→`mat_vec_xpref`）、`render.cu`（:231 同）。
- **零 diff**：`src/render/bvh/`（含 `scene.cu::mat_vec`/`mat_t_vec`）、`render_megakernel.cu`（顶点法线链 `mat_vec`+`normalize3_midpref`）、`render_shade.cu`、`render_shadow.cu`、`aa_resolve.cu`、`render_flex.cu`、`fixtures/`、`scripts/`、`tests/`、guards JSON；无新增 `.cu`（精度源面 42 文件不变）。
- `normalize3_mesh` **函数体逐字节未变**（我抽取 old/new 函数体比对：identical）；只在其前新增 helper。

## 2. ② `mat_vec_xpref` 正确性 ✅

- `scene.cu::mat_vec`（未改）：`fmaf(m[2],a.z, fmaf(m[0],a.x, m[1]*a.y))` ⇒ 独立舍入项是 **y**（`m[1]*a.y` 为 plain mul）。
- 新 `mat_vec_xpref`：`fmaf(m[2],a.z, fmaf(m[1],a.y, m[0]*a.x))` ⇒ 独立舍入项是 **x**（`m[0]*a.x` 为 plain mul），随后 fma y、fma z；行主序索引 `m[0..2]/[3..5]/[6..8]` 与 `mat_vec` 完全相同 ✅。
- 与上游 `include/warp-headers/mat.h:3180` `mul(mat,vec)`（`r = get_col(0)*b[0]; for i≥1 r += col_i*b_i` → 首项为单独乘积）一致 ✅；`mesh.h:1880` 确有 `normal = normalize(min_normal);` ✅。
- 消费点仅两处（`render_common.cu:225`、`render.cu:231`），改后全仓已无 `normalize3_mesh(mat_vec(` 形式 ✅。

## 3. ③ 双内核身份 + 算术增量（我独立复算，逐位一致）✅

| 内核 | old source | new source | old PTX | new PTX |
| --- | --- | --- | --- | --- |
| megakernel | 109,384 / `624ab298…` | **110,509 / `6b1ade37…`** | 347,070 / `1fd53131…` | **348,399 / `48f8e7f8…`** |
| depth_seg | 52,427 / `b546ef08…` | **53,552 / `1e3f273c…`** | 138,540 / `e8d84963…` | **138,857 / `31912377…`** |

- 复现条件：pinned CUDA 12.8.1 NVRTC + product 选项、按 `mod.rs` 拼接、**source name 取 identity 用例名**（`render_megakernel.cu` / `render_depth_seg.cu`；名字进入 PTX 身份，用其它名字会得到不同 sha——这是复现要点，非缺陷）。
- **PTX 算术增量逐 opcode**：megakernel `div +12 / fma +8 / mul +4 / sqrt +4`（其余 0，sqrt 62→66）；depth_seg `div +3 / fma +2 / mul +1 / sqrt +1`（其余 0，sqrt 28→29）——与证据完全一致。`+7 = 一次 normalize`（mul+fma+fma+sqrt+div×3），4/1 个内联实例。
- **探针链序**：old 16 ops = 3 行 `mul→fma→fma`（首乘 `[mat+4]×[n+4]`=m[1]·a.y）+ 一次 normalize；new 23 ops = 局部 normalize（mul/fma/fma/sqrt/div×3）→ 3 行 `mul→fma→fma`（首乘 `[mat+0]×n_local.x`=m[0]·n.x）→ 第二次 normalize ✅（我重数探针 PTX：16/23，序列一致）。
- **depth_seg 换新基线如实写明**（README §3/§5 + receipt）：因为 `render_math.cu` 同时进入两个拼接，换基线是必然；数值预期不依赖 normal 数值——`render.cu` 中 `normal` 仅初始化/赋值（:195/196/:231），命中判定只用 `dot(lvec, n_local) < 0` 的**符号**，正长度归一化不改变符号 ⇒ depth/seg 数字不变。冻结回归我重跑：**13 passed / 6 ignored / 0 failed**（depth_seg）、**12 passed / 6 ignored / 0 failed**（flags）✅。

## 4. ④⑤ 边界 / 门禁 / 证据 ✅

- `fixtures/`、`scripts/`、`tests/` 零 diff；无新增 `.cu`；guards JSON 与 12 冻结 exe 未动；H2 历史证据目录（RCA / mesh-normal-single-normalize）**未被改写**（作废只在代码注释与新证据中声明，符合「不改历史语义」）。
- 本卡 `normify gate` 复跑 fmt/clippy/clippy-cuda/unit 全过，unit **1,391/0/402**；identity 用例 1 passed ×2 且打印值与证据一致。
- 证据 manifest **17/17** 独立重算（bad=[] / extra=[]）、17 文件全 UTF-8/无 BOM/无 CRLF；3 个 `.cu` 的 old/new sha+bytes+行数与 README 表逐项一致（13,685/326、22,354/498、22,033/496）。

## 5. ⑥ 口径 ✅（2 条信息级观察）

- README §4/§7 与 receipt `non_claims`：**不宣称修复生效**（4 px 是否归零待诊断轮 #8）、不改判据/阈值/参考/容差/相机/recipe、G-L2 判定维持未通过（判据 A FAIL）、不重跑/不重冻/不采集 ✅。
- **O1（信息）**：证据的算术 op 绝对总数用 3,094/3,122（mega）与 1,201/1,208（depth）；我的 opcode 过滤口径得 3,097/3,125 与 1,202/1,209（固定 +3/+1 的过滤差异；**增量与 sqrt 计数完全一致**）。不影响任何结论。
- **O2（信息）**：复现 identity 时 source name 必须与 identity 用例一致（见 §3），否则 sha/bytes 会变——建议在证据复现命令里固定写明（不影响本轮签署）。

## 6. 非声明

本复核只核改动面/身份/算术增量/边界/证据与口径，**不判定修复效果**（4 px 是否归零由诊断轮 #8 实测），不改变 G-L2 判定，零 GPU，未重跑 GPU/未重冻/未改代码。
