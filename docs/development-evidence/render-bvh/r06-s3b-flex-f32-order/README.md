# R06-S3b 1D flex 命中链 f32 次序位级对齐（f32-order fix）

任务：`2026-11-04-render-r06-s3b-flex-f32-order-fix-r2`（owner Backend，depends D8 诊断卡
`2026-11-04-render-r06-s3b-flex-cam2-depth-gap`，D8 提交 `a1cc22ae`）。原卡
`2026-11-04-render-r06-s3b-flex-f32-order-fix` 因 `schedule.files` 路径笔误（`src/render/kernel/cuda/scene.cu`
不存在）已 abandon（`59f62c73`），r2 由 Arch 登记（`9b961d8f`，真实路径
`src/render/bvh/cuda/scene.cu`）；本证据目录在 r2 下原样迁移、内容不重做。

本片把 D8 定位的 `cam2 depth` 缺口（`(321, 297)`，max 259 ULP）从**判据性红**推进到
**修复 + 验证**：根因是 1D flex 命中链中 `ray_map` 的 `M^T * vec` 三项点积在 NVRTC
内联上下文里被收缩成「舍入首项」，而冻结上游该处收缩成「舍入中间项」。修复后验证短跑：

| 指标 | 修复前（D8） | 修复后（本片验证短跑） |
| --- | --- | --- |
| cam2 depth `(mismatched, beyond4ULP)` | `(321, 297)` | **`(1, 0)`** |
| cam0 / cam1 depth | `(0, 0)` / `(0, 0)` | `(0, 0)` / `(0, 0)`（rigid 零漂移） |
| cam2 seg vs 参考 | 0 / 76,800 | **0 / 76,800** |
| 三相机 rgb | 全过（≤1 LSB） | 全过（≤1 LSB），两跑 sha 相等 |
| 修复后按 flex 细分 | — | flex1 **0/251**、flex0 **1/403**、全 flex **1/654** |

只剩 **1 个像素** `(x=134, y=128)`（flexid 0、ULP=3、beyond4ULP=0）位级不等，已按卡面
第 5 条**列像素 + 差值交 Chief 裁定**，本片**不自行放宽 4 ULP**、不宣称 verified。

## 1. U4 次序枚举表（上游表达式 → 产品表达式 → 是否可能差 ULP）

方法照 U4 sphere RCA：先把 1D flex 命中链逐式列出，再对每个「求和/收缩点」判定上游与
产品的实际编译形态。判定依据不是猜测，而是**冻结上游自己编译出的 PTX**（`s3b-flex-cache`
里 `render_util_0460c49.sm89.ptx` 与 megakernel 的 `.sm89.ptx`）+ 本片新做的
**warp GPU 回放 oracle**（见 §3）。

| 环节 | 上游表达式（冻结 71da24d / warp 1.15） | 产品表达式 | 是否可能差 ULP | 判定与依据 |
| --- | --- | --- | --- | --- |
| 射线归一化 | `wp.normalize(vec3(x,y,-znear))`，实测收缩 `mul(x*x); fma(y*y); fma(z*z)` | `build_rays.cu::normalize3` 显式 `fmaf(z,z,fmaf(y,y,x*x))` | 否 | 上游 PTX 直读 + S5 夹具 256 条 `dir_bits` 逐位匹配（另一种收缩序仅 247/256） |
| `warp_length3`（胶囊 vec/axis 长度） | `wp::length` 三项平方和 | 显式 `fmaf(z,z,fmaf(y,y,x*x))` | 否（本场景） | 本场景 4 条 flex 边的 norm/axis 长度两种收缩序取值 0 ULP 差；oracle 链复现参考亦证实 |
| `flex_quat_z2vec`（`atan2f`/`sinf`/`cosf`/half-angle） | `math.py::quat_z2vec` | 逐式翻译（位置化 (w,x,y,z) 约定） | 否 | 形态逐项一致；两套 NVRTC（12.8/12.9、compute_70/sm_89）对该链编译出的算术流一致 |
| `flex_quat_to_mat` | `math.py::quat_to_mat` | 位置化逐式公式 | 否 | 逐式一致（仅乘加，无收缩歧义） |
| `dot()`（ray_sphere / 胶囊四段二次式） | `wp::dot` 三项和 | `a.x*b.x + a.y*b.y + a.z*b.z` | **否**（实测已是中间项） | 216 变体枚举中产品侧 `dot3=舍入中间项` 与参考一致（§3） |
| `ray_map`：`lpnt = M^T (pnt-pos)` | `wp::mul(matT, dif)` | `mat_t_vec` 三项和 | 否（修复后） | 枚举：参考 lpnt=中间项；产品 pre-fix 亦为中间项 |
| `ray_map`：`lvec = M^T dir` | `wp::mul(matT, vec)` | `mat_t_vec` 三项和 | **是（唯一分叉点）** | 枚举：参考=中间项；产品 pre-fix 内联上下文=**舍入首项**（对产品 248/251） |
| `mat_vec`（法向/相机变换） | `wp::mul(mat, vec)` | `mat_vec` 三项和 | 否（修复后） | 同函数族，双击固定；rigid 零漂移作证 |
| `ray_quad` `det = b*b - a*c` | `mul,mul,sub`→实测 `fma(b,b,-(a*c))` | 同式 | 否 | 枚举：det=`fma` 形态两侧一致 |
| `ray_sphere` 早退 / 四段根式 / cap 选择 | warp 逐式 | 逐式翻译 | 否 | 枚举覆盖（根式两种写法在本场景等价） |
| `size = (radius, 0.5*length, 0)`、`pos = 0.5*(v0+v1)` | warp 逐式 | 逐式 | 否 | 逐式一致；oracle 链复现参考 |

结论：唯一的分叉是 `mat_t_vec` 的 `lvec` 三项和收缩序，其余环节逐式/逐位枚举均已排除。

## 2. 逐像素定位（D8 dump + 新产品 dump）

- D8 dump（`a1cc22ae`）已给出 cam2 321 个位级失配像素、全部落在 flex 区、seg 位级全等；
  flexid1（单边胶囊 251 px）**251/251 全部 bit 不等**、flexid0 70/403。
- 本片进一步用 **warp GPU instrument 回放**（§3）在 251 个 flexid1 像素上逐位对拍：
  - 回放 `d * -ray_dir_local_z` vs **参考 depth：0/251 失配**（逐位全等）→ 参考链可被同一公式复现；
  - vs **产品 depth：251/251 失配**，ULP 中位 8、max 155 → 差异在产品编译产物，不在输入/参考。
- 再用回放 dump 的精确中间量做 **216 组收缩序变体枚举**（`scripts/contract_enum2.py`）：
  - 参考侧唯一能 251/251 复现的组合：`dot3`/`mat_vec`/`dot2` 全部**舍入中间项** + `det=fma(b,b,-(a*c))`；
  - 产品侧最贴近组合：`lvec` 改为舍入首项 → 248/251（另 3 px 残差与本片修复后的 1 px 同源量级）；
  - **首次分歧运算点 = `scene.cu::mat_t_vec` 的 `lvec` 分量**。

## 3. warp GPU instrument 回放 oracle（诊断物，不是参考）

- 归因：`scripts/upstream_flex_chain.py` 用 pinned warp 1.15.0（CUDA 12.9 / sm89）把冻结上游
  1D flex 命中链（`render.py` flex 分支 + `ray.py::ray_capsule` + `math.py`）逐式复刻成诊断内核；
  输入 = 产品侧 dump 的 cam2 相机空间 ray（`f32-order/cam2-ray.f32le`）+ 参考相机位姿
  （`cam-poses.json`）+ `state.npz` flex 顶点/半径；输出 = `pos/vec/length/quat/size/dirw/lpnt/lvec/d`。
- 产物 `artifacts/flex1-oracle.npz`（251 行）与 `artifacts/flex0-oracle.npz`（403 行）为诊断支撑，
  **不是参考、不是判据输入**；两个 npz 的 product 列来源不同：**flex1 的 `prod_depth` 是修复前 D8
  产品 dump（`a1cc22ae`）**，flex0 的 `prod_depth` 是**修复后**产品 dump（第 2 次微跑 `residual/cam2-depth.f32le`）；
  `ref_depth` 两列都是参考位序。
  本片未改 `fixtures/`、未重新生成任何参考。
- 记账：`artifacts/warp-replay-kernel-sha256.txt`（warp 生成的内核 `.cu/.meta/.sm89.ptx` 指纹）。

## 4. 修复（只动 `scene.cu` 两条表达式）

文件：`src/render/bvh/cuda/scene.cu`（r2 卡 `files` 已用真实路径；旧卡 `files` 误写成
`src/render/kernel/cuda/scene.cu` 已 abandon）。r2 `files` 同时列了
`src/render/kernel/cuda/render_flex.cu`，但该文件**本次未修改**（保留在 scope 内仅为对齐登记意图）。

- `mat_t_vec`（`M^T * a`）：
  - comp0 `fmaf(m[6],a.z, fmaf(m[0],a.x, m[3]*a.y))`
  - comp1 `fmaf(m[7],a.z, fmaf(m[1],a.x, m[4]*a.y))`
  - comp2 `fmaf(m[8],a.z, fmaf(m[2],a.x, m[5]*a.y))`
- `mat_vec`（`M * a`）同构：
  - comp0 `fmaf(m[2],a.z, fmaf(m[0],a.x, m[1]*a.y))`
  - comp1 `fmaf(m[5],a.z, fmaf(m[3],a.x, m[4]*a.y))`
  - comp2 `fmaf(m[8],a.z, fmaf(m[6],a.x, m[7]*a.y))`
- 依据：上游 flex 副本 PTX 形态（`mul` 中间项 + 两条 `fma`）+ 本片 oracle 251/251 复现。
- 未动：`dot()`、`det` 形态、`ray_quad`、`warp_length3`、`flex_quat_z2vec`、判据/容差/参考/`fixtures/`。
- 为什么 rigid 不受影响：rigid 路径此前位级全等说明其内联上下文本就按中间项收缩，显式化是等价变换；
  本次验证 cam0/cam1 depth `(0,0)` 位级全等即回归证据。

## 5. 验证（四门禁 + 预批 1 次短跑）

- 四门禁：fmt 通过 1762 ms、clippy 通过 5938 ms、clippy-cuda 通过 404 ms、unit **1316 过 0 败**（84284 ms）。
- 验证短跑（committed 用例、无 WIP）：wall **2.38 s**，单次；GPU 前 `free 14,307 MiB / util 5% / 29°C`
  → 后 `free 14,318 MiB / util 8% / 29°C`；EXIT=101（预期观察点）。
- 断言结果（`tests/render_flex_validate.rs`）：三相机 rgb 全过；cam0/cam1 depth `(0,0)` + seg 全等；
  **cam2 depth `(1, 0)`**（1 px 位级不同、beyond4ULP=0）；两跑 sha 相等（数值见 §7）。
- 内核/可执行身份（内核源变更，attempt5 必须重新真冻结；**窗口规则 v2**）：
  - **post-r2-gate canonical rebuild**（`cargo clean -p mjwarp-rs && cargo test --locked --features cuda-probe
    --no-run --test render_flex_validate`，19:27:07，**recorded after canonical rebuild**）：
    `2331BA260C6863F53555830E428FEBC28FC897A2625944A92CC828AB55E6324D` / **107,661,795 B**。
  - **debug test exe 的 sha 随构建实例变化**（同源同大小 107,661,795 B、sha 不同：`3A096B52…`、`3CF1EA02…`、
    `2331BA26…`；Arch 独立探针 `CARGO_INCREMENTAL=0` + clean 复现同现象、`/Brepro` 构建失败）。
  - **窗口规则 v2（Chief 2026-11-04 19:26 批准）**：attempt5 开窗先做**一次 canonical rebuild → 立即记录
    exe sha+bytes → 用该次产物 `freeze -Attempt 1`（16/16、exes=2、inputs=14）→ 之后零编译**跑实体；
    预录 sha 只作参考、不作冻结卡口（详见 `artifacts/fingerprint-canonical-rebuild.json`）。
  - 产品 PTX（离线 NVRTC 12.8.1、compute_70、fmad=true 预检）：`source_concat_sha256=545c8fbe…`、
    **345,514 B**、`ptx_sha256=aa51258b…`（旧 `d48fc74e…` / 343,993 B / `9118d1ea…`）；
  - 算术指令流：4188 → 4220 条（+32，46 个非等块；多内联点 + 寄存器分配变化，非单点 diff）。

## 6. 残留像素（1 px，已判定为候选顺序 tie，非 f32 收缩问题；交 Chief 裁定）

- 坐标 `(x=134, y=128)`（flat index 41094），cam2；参考 seg `flexid=0`（flex-0 三边折线），type=9；
- prod bits `0x400015ba` = 2.001326084136963 m；ref bits `0x400015b7` = 2.0013253688812256 m；
  **ULP=3**，绝对差 7.153e-7 m；`beyond4ULP=0`；cam2 seg 0/76,800 失配。

**flex-0 oracle 判定（第 3 次微跑，Chief 批准）**：把上游 instrument 回放扩到 flex-0 三条边
（verts 0-1 / 1-2 / 2-3，radius=0.02，取最近命中边），结果：

| 项目 | 值 |
| --- | --- |
| oracle vs **参考** depth（flex-0 全 403 px） | **0 / 403 失配**（中位 0 ULP、max 0） |
| oracle vs **产品** depth（flex-0 全 403 px） | **1 / 403 失配**（即本 px，3 ULP） |
| 该 px 参考侧胜出边 | **edge0**（verts 0-1），`d=2.0099208`，depth bits `0x400015b7` = 参考 |
| 该 px 产品侧胜出边 | **edge1**（verts 1-2），`d=2.0099216`，depth bits `0x400015ba` = 产品 |
| 两条边 d 差 | **7.1526e-7 m** |
| tie 容差 | `RAY_TOL_REL * max(dist,1) = 1e-6 * 2.0099208 = **2.0099e-6**` |
| tie 判定 | **|Δd| < tol → 两条边在容差内平局** |

tie 规则（产品与上游一致）：`closer = (d < dist - tol) || (|d - dist| <= tol && (hit.geom_id < 0 ||
candidate_id > hit.geom_id))`；flex 候选 `candidate_id = -2`，首次 flex 命中后 `hit.geom_id = -2 < 0`，
因此**容差内后访问的候选胜出**。参考侧最后访问 edge0、产品侧最后访问 edge1（产品场景 BVH 遍历顺序
与上游 warp BVH 查询顺序不同）→ 深度取到不同的边，差 3 ULP；seg 仍相同（都是 flexid 0）。

**结论**：这 1 px 的差异是**flex-0 两条边在 `RAY_TOL_REL` 容差内平局时的候选访问顺序**决定的，
**不是 f32 运算次序/FMA 收缩点**（oracle 精确复现参考 0/403 已证明链算术一致）。按 Chief 的
两种口径，它属「边界情形、本卡 scope（只调 f32 次序/FMA 形式）内不可修」：要消除需让产品的场景 BVH
构建/遍历顺序或 tie 语义与上游 warp BVH 完全一致，属算法/语义改动，另立卡并需 Chief 裁定；
本片**不自行放宽 4 ULP、不宣称 bit-first 通过**，证据封存交 Chief 决定后续（B 类前提报用户签，
或另立 tie-order 卡）。

## 7. 微窗口与记账

| 项目 | 值 |
| --- | --- |
| 第 1 次微窗口（ray dump，Chief GO） | 18:55:44 acquire（pid 61884）；跑前 `free 14,267 MiB / util 5% / 29°C`、无 CUDA 计算、build 0；run wall 0.48 s；跑后还原 WIP、release |
| `cam2-ray.f32le` | **1,843,200 B**、sha256 `6de834a5dfeebb0dcf7d40dada806afe2c6695986fdc975e535e688b4d881cbd` |
| `cam2-rays.f32le`（另一会话 18:53 独立产物） | 与上一条**逐字节相同**（同 sha），两次独立运行一致 |
| 第 2 次微窗口（残留定位，Chief GO） | 19:11:48→19:11:50，wall 1.67 s；`free 14,261 MiB / util 7% / 29°C`；跑后 `git checkout --` 还原 |
| `residual/cam2-depth.f32le` | 307,200 B、sha256 `d7a04b1aebeec74d6366eab63b5c6fae28515a04bbecaf97884078b4e8a99802` |
| `residual/cam2-seg.i32le` | 614,400 B、sha256 `86106de1036beb308ffc1288d34dabE602aa753b1bc4b3aed78be9210d505742` |
| 两跑 sha（修复后） | rgb `d84ecaed…`、depth `9829215335d2cb2fd22b02a4d4a8354cc1a5db80c806b94a8888ef632c717326`、seg `c4be9853…`（与 D8 相同，seg 未变） |
| 写锁 | backend-chat，19:07:33 acquire（pid 85668，TTL 90 min），本片收口时 release；WIP 一律未提交并已还原 |

## 8. 复现

```powershell
# 1) 产品 PTX 指纹（离线，不占 GPU）
target/mjwarp-parallel/venv/Scripts/python.exe -B docs/development-evidence/render-bvh/r06-s3c-flex-mesh-device/tools/nvrtc_precheck.py --repo . --cuda target/toolchains/cuda-12.8.1 --out target/s3b-rca/product-megakernel-fixed

# 2) 上游 oracle 回放（诊断；需要 GPU，按短窗口规则）
target/mjwarp-parallel/venv/Scripts/python.exe -B docs/development-evidence/render-bvh/r06-s3b-flex-f32-order/scripts/upstream_flex_chain.py

# 3) 收缩序枚举与残留像素复算（CPU）
target/mjwarp-parallel/venv/Scripts/python.exe -B docs/development-evidence/render-bvh/r06-s3b-flex-f32-order/scripts/contract_enum2.py
target/mjwarp-parallel/venv/Scripts/python.exe -B docs/development-evidence/render-bvh/r06-s3b-flex-f32-order/scripts/residual_pixel.py

# 4) 验证短跑（需要 GPU；单次 ≤3 分钟，跑前查三点基线）
cargo test --locked --features cuda-probe --test render_flex_validate -- --ignored --test-threads=1
```

## 9. 口径与边界

- 本片不改判据/容差/参考/`fixtures/`；「4 ULP 在薄胶囊近掠射像素上近似等价位级」只作为
  **B 类判据前提候选**（QA 复核、Chief 裁定），**不得替代冻结 bit-first 判据**；替代属签名级、需用户签。
- attempt5 前置不变（本卡 verified + QA pass + 重新真冻结 `16/16`、exes=2、inputs=14 + Chief GO）；
  本卡提交后必须重冻（test exe / 内核身份已变，见 §5）。
- D8 证据 errata（直方图互斥桶、replay 产物 sha/统计）按 Chief 指示随后续证据提交一起处置。
