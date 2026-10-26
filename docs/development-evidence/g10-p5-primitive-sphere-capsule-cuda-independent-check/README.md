# B2（sphere/capsule 族 4 组合 device primitive）QA 独立复核

结论：**pass（阻断 0 / 观察 7）**。复核对象：交付 `6ab5cb22`（实现+证据）、验收关闭 `f578b474`、复核 start `580bd7e5`；本轮基于复核期间最新 HEAD `ac262ee9`（含 B1 补记 `190dc37d`、B1 观察收口 `686dc61d`、B2 证据收口 `07062ef7`/`3ac2a0d3`/`e6cb57de`）。本复核纯 CPU：未开 GPU 窗口，device 侧只核交付窗口原始日志；另做 pinned CPU 独立复算、源码逐分支核对、launcher 行序归一化 diff 复核与纯 CPU 证明重跑。

## 1. 复核范围与不变量

- 交付 diff（`6ab5cb22~1..6ab5cb22`）：新增 `primitive_sphere_capsule_shapes.cu`(223 行)/`primitive_sphere_capsule_wrapper.cu`(71)、改 `product.cu`(394)、`source.rs`(133)、`validate.rs`(56)、`primitive_tests.rs`(174)、`multicontact_tests.rs`(109)、`tests/g10_p5_primitive_sphere_capsule_cuda.rs`(465)、`tests/g10_p5_nvrtc_launcher.rs`(424)+`tests/support/launcher_compare_support.rs`(429)、证据；共 38 文件 +2287 −318。
- 独立核 0 新 ABI 字段（`src/physics/collision/abi.rs`、`CollisionArgs` 无 diff）；0 新 PTX 入口（`entries=6` 由我重跑冒烟确认）；`src/physics/mod.rs` 零 diff、`fixtures/` 零改动、未另立 repin；判据/容差/期望值/`case_evidence.rs`/`case_compare.rs`/`.normify/gates.toml` 均未改。
- `DEVICE_SOURCES` 25→27，两段追加在 `primitive_plane_wrapper` 之后、`product.cu` 之前（`source.rs` diff 核过），B1 的 25 段相对顺序未变。

## 2. pinned 语义逐分支

来源：`docs/evidence/d04-complex-physics/authoring-runs/run-5/frozen-source/_src/`（revision `71da24d9…`）。

| 组合 | pinned 核心 | 设备实现 | 核对点 |
| --- | --- | --- | --- |
| sphere-sphere | `collision_primitive_core.py::sphere_sphere` L58-87 | `g5_sphere_sphere` | `dir/dist` 逐分量除法、零距离 `n=(1,0,0)` fallback、`dist-(r1+r2)`、`pos1+n*(r1+0.5*dist)` 左结合 |
| sphere-capsule | L90-125 | `g5_sphere_capsule` | 球心投影到轴并 clamp 到 ±half_length、再调 `sphere_sphere` |
| capsule-capsule | L128-255 | `g5_capsule_capsule_core` | `det=ma*mc-mb*mb`、`|det|>=MJ_MINVAL(1e-15)` 非平行 1 接触（x1/x2 clamp、`(v-mb)/mc` 等）、平行轴 4 端点、`dist<=margin` 过滤、无效槽 `dist=inf`、**第 4 端点写入不递增 count（与 pinned 逐字一致）** |
| sphere-cylinder | L393-462 | `g5_sphere_cylinder` | 侧面/端面/角点三分支、`dist_cap<dist_radius` 判据、端面调 `plane_sphere` 后法线取反、角点 `safe_div`+`sign(x)` |

- wrapper（`collision_primitive.py` L357-429 / L432-507 / L510-597 / L915-1000）：轴取 `rot` 第三列（`rot[2],rot[5],rot[8]`）、`write_contact(id_=0)` 或 capsule 的 0/1 两次、`make_frame(normal)`；设备 4 wrapper 逐项对应。
- `make_frame`（`math.py` L203-213 `orthogonals`）：normalize → selector(y 或 z) → `b=normalize(selector-a*dot)` → 零长度 b=0 → `c=cross`；设备 `common.cu::make_frame` 一致（B1 已核，本轮复核未变）。
- `g5_primitive_safe_div` = `x / (y!=0 ? y : 1e-15)`，与 pinned `safe_div` 一致；`G5_PRIMITIVE_MINVAL=1e-15` 与 `mjMINVAL` 一致。
- **dispatch 顺序**：pinned `_add_geom_pair` L373-376 按类型序规范化 pair（`type1>type2` 时交换），`_PRIMITIVE_COLLISIONS` L1335-1349 的 key 即（低类型,高类型），kernel L1474-1480 逐 key 精确匹配；设备 `product.cu` 对 (CAPSULE,SPHERE)/(CYLINDER,SPHERE) 交换 wrapper 入参，与上游一致。

## 3. 判据与 host 校验（未放宽）

- `tests/g10_p5_primitive_sphere_capsule_cuda.rs`：count 与结构（worldid/type/dim/geom/geomcollisionid/includemargin/adhesion/friction/solref/solreffriction/solimp）**零容差**；`dist` 位级优先，否则 ≤4 ULP@自身量级（ref=0 → 4 ULP@1.0）；`pos` 4·ULP(参考三轴最大绝对值)；`frame` 4 ULP@1.0；`efc_address` 全 -1、`overflow` 全 0 设备侧不变量；行序按 `geomcollisionid` 排序配对（行序非判据）。与 B1 同口径，无一处放宽。
- `validate.rs`：放行 {SPHERE,SPHERE}/{SPHERE,CAPSULE}/{SPHERE,CYLINDER}/{CAPSULE,CAPSULE}（无序对）+ 既有组合；`primitive_tests.rs` 增正/负例（如 ELLIPSOID/SPHERE、CAPSULE/CYLINDER 仍拒）。
- `capsule-capsule` 判据参考是 B0 **GPU** 参考（nacon=1），未按 CPU 预检 nacon=2 凑数（§5 给机制）。

## 4. CPU 独立复算（pinned venv，无 warp/mujoco_warp import）

方法：读 B0 GPU 参考（`upstream-primitive-gpu.json`，sha `3a6475ac…`）的 `geom_xpos/geom_xmat`、fixture 的 `geom_size`，按 pinned 源逐式 f32 重算；写入闸 `dist<margin+gap`（margin=gap=0）。CUDA 默认 `fmad=true`，`det` 按单次舍入 FMA 模拟（见 §5 机制）。

| case | count 我/参考 | ids | dist ULP | pos ULP | frame ULP | 结果 |
| --- | --- | --- | --- | --- | --- | --- |
| sphere-sphere | 1/1 | [0]/[0] | 0 | 0 | 0 | 位级一致 |
| sphere-capsule | 1/1 | [0]/[0] | 0 | 0 | 0 | 位级一致 |
| sphere-cylinder | 1/1 | [0]/[0] | 0 | 0 | 0 | 位级一致（侧面分支） |
| capsule-capsule | 1/1 | [0]/[0] | 0 | 0 | 0 | 位级一致（非平行分支） |

→ B0 GPU 参考与 pinned 语义自洽；device 窗口 4/4 与本表一致。完整数据 `logs/recompute-sphere-capsule.json`。

## 5. capsule-capsule 的 FMA 收缩机制（后端差异的根因）

该 fixture 两胶囊轴均 +z、rigid，理论 `det=ma*mc-mb*mb=0`（平行分支，2 接触）。但 f32 下：

- 非融合两次舍入：`det=0.0` → 平行分支 → 端点对 → **2 接触**（CPU 预检 nacon=2）；
- NVCC 默认收缩成 `fma(ma,mc,-(mb*mb))`：`det≈2.625e-12 ≥ 1e-15` → 非平行分支 → x1=x2=0 → 1 接触、dist=-0.020000003（B0 GPU 参考 nacon=1）。

我的复算两种都跑（`scripts/recompute_sphere_capsule.py`，`--unfused` 切换）：融合口径 4/4 位级一致；非融合口径 capsule-capsule 变 2 接触（`logs/recompute-unfused.log`）。这解释了 Arch 说的「GPU 参考 1 vs CPU 预检 2」，也说明该 case 的 count 依赖编译期 FMA 收缩（观察 O3）。

## 6. 拼接源 / 指纹（独立复现）

- 我在主树 `cargo test --features cuda-probe --lib runtime::collision_gpu -- --include-ignored` 重跑：**26 passed / 0 failed**，`nvrtc_smoke_ok ptx_bytes=3786439 entries=6 source_bytes=237212 source_sha256=054aaa40…`（`logs/lib-collision-gpu-summary.log`）。
- 自写 27 段拼装（复刻 strip 本地 include + 追加 product.cu）：工作树字节 = **237212 / `054aaa40…`**（与冒烟逐字节一致）；git blob（LF）= **236732 / `9807a539…`**（B2 证据 §5.1 的 LF 口径，我复算一致）；旧 21 段 = **199869 / `67fb8445…`**（工作树与 blob 相同，因旧 `.cu` 均 LF；`product.cu` 已含 B2 新增行）。工作树与 LF 差 480 字节来自 B1 的 4 个 plane `.cu` 在工作树为 CRLF（逐文件 delta 44/162/169/113）。
- 结论：B2 冒烟指纹**可从当前工作树精确复现**；LF 口径也可由提交树复现。B1 交付的 `222849`（混合 EOL）问题在 B2 未重演，因为 B2 两段新 `.cu` 在工作树为 LF。

## 7. launcher 行序归一化（B2 附带修，只改 tests/）

- 旧测法：`compare_case` 按原始行序逐字段硬比（`chunks_exact` + `compare_exact!` 宏）；多 world 接触写入依赖共享 `atomicAdd`，跨 world 行序无保证 → attempt1 `ellipsoid-box` 报 14 条（dev worldid [0,1] vs ref [1,0]）。
- 新测法：`g5_contact_order` 按 `worldid` 分组后组内 `dist → pos[0..2] → geomcollisionid`（`total_cmp`）稳定排序，逐字段仍走原检查器；与 `tests/support/stage2_equivalence_compare.rs::deterministic` 的键一致（我核过）。
- **diff 复核（纯搬移）**：我按「旧文件 vs 新主文件+support」做 token 级比较——`ulp_at/vector_scale/vector_bound/ordered/ulp_distance/dist_ok/check_exact/check_dist/check_vectors` **函数体逐 token 相同**（`check_exact_f32` 仅签名换行格式与尾逗号差异，语义相同）；唯一实质变化是 `compare_exact!` 宏被删除、`compare_case` 改为显式传入两侧 permuted 切片，字段集/阈值/期望来源不变（`nacon/nccd/overflow/efc 长度`仍先比）。为过 clippy 的清理只有两处死代码（main 未再使用的 `FOUR_ULP_AT_ONE`、support 未使用的 `cases_root` import），不改变比较逻辑。
- **纯 CPU 证明重跑**（我执行，未用窗口）：`cargo test --test g10_p5_nvrtc_launcher world_order_normalization` → **raw_order_failures=14 sorted_failures=0**，旧口径指纹 `max_dist_ulp=76 / max_pos_ulp=2676526 / max_frame_ulp=28621803`，与 attempt1 launcher 日志逐字一致（`logs/launcher-normalization-test.log`）。按 Chief 要求又**直接执行提交源二进制**（`target/debug/deps/g10_p5_nvrtc_launcher-cb470647dafbe31c.exe --nocapture`，sha256 `5c191046…`）：1 passed / 1 ignored、proof 行同上、exit 0（`logs/direct-exe.log`）。
- **23ae4d82 → 5c191046 的等价性**：window 实际 launcher 为 `23ae4d82…`（拆分前），提交源构建为 `5c191046…`（我实测）；作废链在 `3ac2a0d3` 留痕、`e6cb57de` 补入 `35ba212d…`（拆分后、clippy 清理前中间态）作废记录。`ac262ee9` 已补 §11.1 如实边界：pre-split 830 行文件从未入库（当时被 500 行 size gate 拒），**没有 git 级原文 diff**，证据不写「已验证 git diff 等价」；等价性依据 = proof 输出逐字相同 + 固定拆分脚本入证据并记 sha256 + 逻辑文本核对 + 死代码删除说明，并明确 **窗口验证的是拆分前二进制 23ae4d82，提交源 5c191046 仅经 CPU proof + 门禁验证、未再上窗**（receipt `committed_source_gpu_verified=false`）。我的独立核对与这一口径一致；未发现任何比较逻辑变化。
- 归一化是否掩盖真实错误：世界分组先排序后 `worldid` 本身仍零容差比较；字段按行重排后仍逐位/4 ULP 比 —— 我判断不掩盖（见 `qa-launcher-normalization.json`）。

## 8. 窗口事实（取自交付原始日志，独立重算聚合）

| 项 | attempt1（有效非收口） | attempt2（收口） |
| --- | --- | --- |
| 窗口 | 21:07:51–21:08:31（body 35.1 s） | 21:13:40–21:14:10（body 25.5 s） |
| 采样 | 134 | 100 |
| used_peak / free_min / util_max | 4186 / 11876 / 59% | 4784 / 11278 / 41% |
| delta_peak（vs 首样本）/ util 连续 ≥80% | 774 / 0 | 1370 / 0 |
| 基线 → 结束基线 | 3407/12655 → 3408/12654 | 3408/12654 → 3408/12654 |
| residual CUDA / 窗口内编译 | 0 / 0 | 0 / 0 |
| 结果 | B2 4/4、B1 6/6、primitive 3/3 bit-exact、launcher 14 条（行序） | 4/4 + 6/6 + 3/3 + 3/3 bit-exact |

- 我用 CSV 独立重算峰值/最低/最高/连续段/时间戳单调，与 `facts.txt`/`guard.log` 完全一致（`logs/window-samples-recompute.log`）；`post_compute_apps` 无本任务进程（facts 原文 34 条桌面上下文）。
- 签名二进制：B2 `8359f17c…`、plane `43a6f32b…`、primitive `3ac9d52c…`、launcher（attempt2）`23ae4d82…`；attempt1 launcher 为 `d08491c6…`（拆分前旧口径）。B2/plane/primitive 三者在 `target/debug/deps` 现存且 sha 与 facts 一致 → 窗口二进制可由提交源复现。
- launcher 提交源构建：`target/debug/deps/g10_p5_nvrtc_launcher-cb470647dafbe31c.exe` sha256 = **`5c191046…`**（我实测，并与 Arch 消息一致）；证据 §11 写的 `35ba212d…` 是过期值（观察 O1）。

## 9. B1 观察收口的交叉复核

- B1 补记 `190dc37d`：README §5.1/§5.2、receipt `window_binary_rebuilds`/`sampler_aggregates_per_attempt`/`attempt1_external_load`/`fix_regate_note`、`window-attempt1/samples.csv` 均已提交；README 两条内容与 addendum A/B 一致（attempt1 95 样本/peak 7071/free_min 8991/util 98%，guard 未 abort）。
- O2 双口径已在 B1/B2 证据落地，我复算 B1 blob LF `222369/ce08822a`、B2 LF `236732/9807a539`、B1 旧 21 段 `198815/52b225c2`、B2 旧 21 段 `199869/67fb8445`，全部一致。
- 一处残留：B1 receipt `fix_regate_note` 仍写「unit 计数 973」，而 `190dc37d` 的 unit run 记录是 **974/0/371**（commit message 也写 974）→ 观察 O6。
- launcher sha 链后续由 `e6cb57de` 补入 `35ba212d` 作废中间态（我复核时该提交刚落地，链内容与 receipt `invalidated_launcher_shas` 一致；见 §7）。

## 10. 未覆盖 / 边界

- 本 case 覆盖的分支：capsule-capsule 走**非平行分支**（见 §5），sphere-cylinder 走**侧面分支**；capsule_capsule 平行轴分支、sphere_cylinder 端面/角点分支**未被这 4 个 fixture 覆盖**（我的复算判定分支），不宣称已验证。
- 交换几何顺序（调用方传入 (capsule,sphere) 等）的 dispatch 路径有代码但**无 case 覆盖**；device `params.geoms` 用调用方顺序，pinned 由 `_add_geom_pair` 按类型序规范化（观察 O5）。
- 未做 host 交叉、未跑 release/Linux；不宣称 primitive 全组合覆盖、精度结论、跨环境等价。
- device 侧非我重跑（GPU 窗口须报批）；我以 CPU 复算+窗口日志交叉，二者不能互相替代。

## 11. 门禁记录（主树，stdout 写 `target/qa-b2/gate-check/`）

- `normify gate 2026-10-26-g10-p5-primitive-sphere-capsule-cuda-independent-check --repo D:\Rust\mjwarp-rs`：**exit 0**，fmt/clippy/clippy-cuda/unit 全过；unit 计数见 `.normify/runs/<task>/unit.json`（该 run 绑定本证据目录内容）。
- `normify check`：**exit 0**（紧跟最终 gate 之后运行；stdout 存 `target/qa-b2/gate-check/check.log`，不拷入本目录以免范围变化使门禁记录过期）。
- 复跑默认 `cargo test --locked` 与 `--no-default-features --lib` 由上述 gate（默认特性）覆盖；`--no-default-features` 的 455/0/5 我本轮未独立重跑（记为未覆盖，风险低：B2 未改特性/target 配置）。

## 12. 复现命令

```text
# CPU 复算（两口径）
target/mjwarp-parallel/venv/Scripts/python.exe target/qa-b2/recompute_sphere_capsule.py
target/mjwarp-parallel/venv/Scripts/python.exe target/qa-b2/recompute_sphere_capsule.py --unfused

# 独立拼装 27 段（工作树 vs git blob）
python target/qa-b2/assemble_b2.py

# 冒烟（只编译设备源，不建 CUDA context）
$env:PATH="D:\Rust\mjwarp-rs\target\toolchains\cuda-12.8.1\bin;$env:PATH"
cargo test --locked --features cuda-probe --lib runtime::collision_gpu -- --include-ignored --nocapture

# launcher 行序归一化纯 CPU 证明
cargo test --locked --features cuda-probe --test g10_p5_nvrtc_launcher world_order_normalization -- --nocapture

# 门禁
normify gate 2026-10-26-g10-p5-primitive-sphere-capsule-cuda-independent-check --repo D:\Rust\mjwarp-rs
normify check 2026-10-26-g10-p5-primitive-sphere-capsule-cuda-independent-check --repo D:\Rust\mjwarp-rs
```

## 13. 观察清单（详见 `qa-observations.json`）

| id | 级别 | 摘要 |
| --- | --- | --- |
| O1 | 一般 | B2 README §11 旧句仍写「提交源构建的二进制为 35ba212d…」，与同 §11 链/receipt 的 `5c191046…` 矛盾（`e6cb57de` 已把 35ba212d 登记为作废中间态，但旧句未改）；同处行数 417/419 实际 424/429，且有制表符残留 |
| O2 | 一般 | B2 README §5 仍写「设备对照测试尚未运行」，与 §10 窗口 4/4 结果矛盾（收口后残留） |
| O3 | 一般 | capsule-capsule nacon=1 依赖 NVCC FMA 收缩（det 2.625e-12 vs 非融合 0.0 → 2 接触）；§6 只写「后端差异」未写机制 |
| O4 | 建议 | 未覆盖分支未逐项点名（capsule 平行轴、cylinder 端面/角点；本 case 分别走非平行/侧面） |
| O5 | 建议 | 交换几何顺序路径无 case 覆盖；device `contact.geom` 用调用方顺序，pinned 按类型序规范化 pair |
| O6 | 一般 | B1 receipt `fix_regate_note` 仍写 973，实际提交 run 为 974/0/371（跨片笔误） |
| O7 | 建议 | attempt2 窗口 launcher `23ae4d82…`（拆分前）未留档于 target，无法做二进制级复核；`ac262ee9` 已补 §11.1 如实边界 + 拆分脚本 sha256 + `committed_source_gpu_verified=false`，我的源码级核对与其一致 |

## 14. 证据清单

`receipt.json`、`qa-semantics.json`、`qa-criteria-scope.json`、`qa-device-reference-recompute.json`、`qa-source-fingerprint.json`、`qa-window-attempts.json`、`qa-launcher-normalization.json`、`qa-observations.json`、`logs/`、`scripts/`、`artifacts-sha256.json`。
