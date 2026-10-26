# B1（plane 族 6 组合 device primitive）QA 独立复核

结论：**pass（阻断 0 / 观察 7）**。复核对象为交付提交 `249a4dbd`（实现）与证据收口 `f2c29d7e`/`b05190e5`，验收关闭 `4bdd89f4`；复核起点 HEAD `293c6c6a`，门禁绑定 HEAD `1cb66758`（见 §11 树状态说明）。本复核纯 CPU，未开 GPU 窗口，device 侧结果只取自交付的窗口原始日志（§5），并另做 pinned CPU 独立复算与源码语义逐分支核对（§4）；不把窗口日志当独立证据，也不把 CPU 复算当 device 验证。

## 1. 复核范围与不变量

- 交付范围：`src/physics/collision/cuda/` 新增 4 个 plane 段（common/convex/shapes/wrapper）、`product.cu` 6 组合 dispatch、`src/runtime/collision_gpu/{source,validate,primitive_tests,multicontact_tests}.rs`、`tests/g10_p5_primitive_plane_cuda.rs`、证据与 `.normify`。
- 独立核过 `git diff 0fa15b54..b05190e5` 逐文件清单：**没有** `abi.rs`/`case_evidence.rs`/`case_compare.rs`/`.normify/gates.toml`/`scripts/`/`fixtures/`/`src/physics/mod.rs` 的改动（diff 为空）；0 新 ABI 字段；0 新 PTX 入口（`product.cu` 只增 type-pair 分派，无新 `extern "C"`）。
- 未修改 `src/`、未改交付证据、未 commit、未把任务标 done。

## 2. 判据未放宽（逐条对代码）

读 `tests/g10_p5_primitive_plane_cuda.rs` 与 `case_*` 实现逐行核：

| 项 | 判据 | 是否放宽 |
| --- | --- | --- |
| count / 结构（worldid、type、dim、geom、geomcollisionid、includemargin、adhesion、friction、solref、solreffriction、solimp） | 零容差逐字段比较 | 否 |
| dist | 参考值 0 → 4 ULP@1.0；否则 ≤4 ULP@自身量级 | 否 |
| pos | 4·ULP（参考三轴最大绝对值） | 否 |
| frame | 4 ULP@1.0 | 否 |
| efc_address / overflow | 设备侧不变量（全 -1 / 全 0） | 不涉及参考 |
| 行序 | 按 `geomcollisionid` 集合相等后按 id 配对，行序不作判据 | 是（但为参考不可比项，交付已声明） |
| nacon | 按 GPU 参考（不用 CPU 预检凑） | 否 |

## 3. pinned 语义逐分支（对照 frozen 源）

对照 `docs/evidence/d04-complex-physics/authoring-runs/run-5/frozen-source/_src/collision_primitive.py`（L52–278、L281、L600、L686、L761、L838–914、L1000）与 `collision_primitive_core.py`（L50/L258/L311/L342/L465）：

- `plane_sphere`、`plane_capsule`（含 `normalize` 逐分量除法、`b_norm<0.5` 回退、`mat33(n,b,c)` 行序）、`plane_ellipsoid`、`plane_box`（`center_dist + dot(n, rot@corner)`，pos 左结合）、`plane_cylinder`（dist1/2/3 顺序、axis 先缩 half、`vec1=cross(vec,axis)`、`vec1` 归一化×radius×sqrt3×0.5）、`plane_convex`（穷举 + graph hillclimb + 4 点去重）与 pinned 对应；6 wrapper（写入门 id、capsule 共享帧、convex ≤4、mesh 用 `a.mesh_vert/a.mesh_graph`）与 pinned wrapper 对应。
- Arch 审查后修正的 4 处已核：① graph hillclimb **running best 严格 `>`**（best 初值 -1e6，`g5_plane_graph_hillclimb`）；② box `center_dist + dot(n,rot@corner)`；③ capsule/ellipsoid/cylinder `normalize` 逐分量除法；④ cylinder 接触 3/4 pos 结合顺序见观察 O4(a)。
- mesh 读取链：`product.cu::build_geom` 由 `a.mesh_vertadr/vertnum/graphadr[dataid]` 置 `d_geom.vertadr/vertnum/graphadr`、`mesh.vert=a.mesh_vert`；primitive 分支传 `{a.mesh_vert,a.mesh_graph}`；与 pinned 一致。
- 小偏差（均为观察，不影响本 6 case）：ellipsoid 非上游退化保护分支、convex hillclimb 迭代上限 `1<<20`、cylinder 退化分母 `1e-15`（与 `mjMinVal` 同量级）。

## 4. CPU 独立复算（pinned venv，未 import warp/mujoco_warp）

方法：读 B0 GPU 参考（`docs/development-evidence/g10-p5-primitive-capture/upstream-primitive-gpu.json`，sha `3a6475ac…`）的 `geom_xpos/geom_xmat/dist/pos/geomcollisionid` 与 fixture `geom_size`，用 pinned venv（python 3.12.13 / mujoco 3.15.0 CPU）dump `model.mesh_vert`，按 pinned 源逐式 f32 左结合、非融合重算 6 case；write gate `dist<margin+gap`（fixture 默认 margin=0，与参考 count 自洽）。

| case | count 我/参考 | ids 我/参考 | dist ULP | dist 差/判据界 | pos 最大 ULP | pos 差/判据界 | 结果 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| plane-sphere | 1/1 | [0]/[0] | 0 | 0/0 | 0 | 0/0 | 位级一致 |
| plane-capsule | 1/1 | [1]/[1] | 0 | 0/0 | 0 | 0/0 | 位级一致 |
| plane-ellipsoid | 1/1 | [0]/[0] | 0 | 0/0 | 0 | 0/0 | 位级一致 |
| plane-cylinder | 1/1 | [0]/[0] | 2 | 0.5 | 14 | 0.4375 | 判据内 |
| plane-box | 3/3 | [0,1,3]/[0,1,3] | 0 | 0/0 | 2 | 0.5 | 判据内 |
| plane-mesh | 3/3 | [0,1,2]/[0,1,2] | 0 | 0/0 | 0 | 0/0 | 位级一致 |

（`pos` 界 = 4·ULP(参考接触三轴最大绝对值)；比值 = 实测差/界，均 <1。完整数据 `logs/recompute-plane.json`。）

结论：B0 参考与 pinned plane 语义自洽；plane-mesh 逐位复现成立（前提是 mesh 顶点用 `model.mesh_vert` ——attempt3 作废根因已被修正）。device 侧本轮无法重跑（GPU 窗口须 Arch 排期，本片声明纯 CPU），只能用交付窗口日志 + 我的 CPU 复算交叉判断，这点在 §10 明列。

## 5. 设备结果（取自交付窗口原始日志，未独立重跑）

- attempt4（唯一有效轮，20:48:30.910–20:48:59.043）：plane 6/6 nacon=1/1/1/1/3/3、failures=0（23.26 s）；primitive 回归 80/80×3 bit-equal max ULP 0；launcher 回归 314/314、sdf 3122/3122、ellipsoid-box 80/80 bit-equal max ULP 0；`samples.csv` 93 行、peak 3367 MiB、free_min 12695、util_max 41%、residual 0、基线/结束基线 3352/12710；窗口内零 Rust 编译。
- attempt1（20:44:09–20:44:38）：plane 测试 panic（host 校验缺项）；95 样本 used_peak 7071、free_min 8991（跌破 10240 地板）、util_max 98%，外部负载在窗口内返回，guard 未 abort → 作废轮，其回归数字虽有但**不作依据**。attempt3（20:46:55–20:47:23）：5/6，plane-mesh dev nacon=1 vs 参考 3（mesh 顶点用错）→ 作废。attempt2：runner 脚本 bug，零设备工作。
- 二进制重建三套 sha 与 addendum A 一致（attempt1/3/4 各自一套，全部窗口外重建、窗口内零编译），Chief 事后追认有效轮用重建后二进制（叙述无法独立核，仅核交付记录与窗口日志自洽）。

## 6. 拼接源 / 源指纹（CPU 独立复现）

- 在 `target/qa-plane/wt-b1`（worktree @ b05190e5）+ 独立 `CARGO_TARGET_DIR`：`--features cuda-probe --lib runtime::collision_gpu -- --include-ignored` = **25 passed / 0 failed**；`nvrtc_smoke_ok ptx_bytes=3713524 entries=6`，与交付一致（另一独立拼装复现 `source_bytes=228010 / sha 81af2f63…`，即全 CRLF 工作树字节）。
- 我复刻 `strip_local_includes` + 25 段 + `PRODUCT_SOURCE` 独立拼装：**旧 21 段 198815 / `52b225c2…`** 与交付完全一致；**新 25 段（git blob，LF）= 222369 / `ce08822a…`**（提交树可复现值）。
- 交付声称的 **222849 / `59631d83…` 依赖工作树 EOL**：只有「旧文件 LF + 4 个 plane 新文件 CRLF + product LF」的混合状态接近（我构造出 222851，差 2 字节），无法从提交树精确复现。全 CRLF 检出 = 228010 / `81af2f63…`。→ 观察 O2，建议记录 LF 归一化指纹或给 `.cu` 加 eol 属性。
- PTX 3713524、entries 6、25 passed、旧 21 段指纹均可独立复现；**没有**发现「源码/PTX 对不上」的证据。

## 7. 行数与文件规模

worktree 实测（`Get-Content` 行数，文件含末尾换行时 +1）：`primitive_plane_common.cu` 44、`convex` 162、`shapes` 171、`wrapper` 113、`product.cu` 379（交付 §3 写 356 → 观察 O3；仍 <450 棘轮）、`tests/g10_p5_primitive_plane_cuda.rs` 493、`collision_gpu.rs` 464、`validate.rs` 45。全部 ≤500 上限。

## 8. host 组合校验与 T4

- `validate.rs`（从 503 行的 `collision_gpu.rs` 抽出，避免超 500 棘轮）：放行 {SPHERE,BOX}/{CAPSULE,BOX} 与 {PLANE, SPHERE/CAPSULE/ELLIPSOID/CYLINDER/BOX/MESH}（含互换），拒 plane-plane 等；`primitive_tests.rs` 有正/负例（含 (PLANE,PLANE)、(105,PLANE)）。
- `src/physics/mod.rs` 零 diff、未另立 repin、0 fixture 改动；`fixtures/precision/t4-current-kernels.json` 的 20 条 `product_sources` 不含新增 `.cu`/`source.rs`/`collision_gpu.rs` → T4 组合身份不漂移（交付说法我按文件清单核过）。

## 9. 窗口参数与守卫（按交付记录核）

- 守卫参数 = 增量 ≥3072 MiB 或 >50% 取更严、free<10240 MiB、新增非本任务计算进程即停；attempt4 采样 0.25 s、93 点、跑前跑后 `query-compute-apps` 均有原文、结束无本任务计算进程、无 residual CUDA pid、结束基线 3352/12710 与开窗基线相同。
- attempt1 的 95 样本（peak 7071 / free_min 8991 / util 98%）正好证明当时的离散点 guard 识别不了外部负载返回——这也是 addendum B 的留档，被我独立核过（`window-attempt1/samples.csv` 95 行、时间戳 20:44:09–20:44:38 区间）。

## 10. 未覆盖 / 边界（不夸大）

- `plane_convex` 的 **graph 分支**（`graphadr != -1 && vertnum >= 10`）无 case 覆盖，本片 plane-mesh 是 4 顶点穷举分支；实现按 pinned 转写但未实机验证。
- 只判「plane 族 6 组合在 B0 给出的 6 个 case 上、按上述判据」；不做 host 交叉、不做 release/Linux；不宣称 primitive 全组合、精度结论、跨环境全绿。
- device 侧数字非我独立重跑（GPU 窗口须报批）；我另做了 CPU 复算（§4）与源码语义核对（§3），但两者不能互相替代。
- 测试 mesh 常量与 MuJoCo `model.mesh_vert` 有 1 ULP 偏差（观察 O1），所以交付窗口里 plane-mesh 的「0 failures」是 4 ULP 判据内的通过，不是位级全等。

## 11. 门禁记录（主树，stdout 写 `target/qa-plane/gate-check/`）

- `normify gate 2026-10-26-g10-p5-primitive-plane-cuda-independent-check --repo D:\Rust\mjwarp-rs`：**exit 0**，fmt / clippy / clippy-cuda / unit 全过；unit 在复核期间观测为 **973→974 passed / 0 failed / 371 ignored**（B2 在途改动持续加测试，计数漂移；计数见收口 run `.normify/runs/2026-10-26-g10-p5-primitive-plane-cuda-independent-check/unit.json`；tree 指纹随本目录证据写入而更新，故不抄 at/scope_hash）。
- 该 run 绑定的是当时工作树：HEAD `1cb66758`（B2 守卫聚合提交）+ B2 在途未提交 `src/` 改动（`product.cu`、`collision_gpu/{validate,primitive_tests,multicontact_tests,source}.rs` 未跟踪/未暂存）→ unit 计数比 B1 基线 972/0/370 多出的 passed/ignored 均属 B2 在途测试（计数随其编辑漂移，见 run 文件），不计入 B1 基线。B1 自身基线由 worktree `b05190e5` 的 `cargo test --locked --no-fail-fast` 佐证：**970 passed / 2 failed / 370 ignored / 113 targets**，两个失败均为既有 EOL 问题（`sdf_rotation_fixture_is_well_formed` 与 `contract::frozen_three_d_package_contract_is_self_consistent`，worktree CRLF 检出导致脚本/包 sha 校验失败，与 B1 无关；总数 1342 与交付 972+370 一致）。
- `normify check`：**exit 0**（在最终 gate 之后运行；stdout 存 `target/qa-plane/gate-check/check.log`，不拷入本目录，避免改动范围使门禁记录过期；结果见 receipt.check）。

## 12. 复现命令

```
# 1) B1 worktree 冒烟（PTX/entries/25 passed）
cd target/qa-plane/wt-b1
$env:CARGO_TARGET_DIR='<abs>/target/qa-plane/t-b1'
$env:PATH='<abs>/target/toolchains/cuda-12.8.1/bin;'+$env:PATH
cargo test --locked --features cuda-probe --lib runtime::collision_gpu -- --include-ignored --nocapture

# 2) CPU 复算（pinned venv）
target/mjwarp-parallel/venv/Scripts/python.exe target/qa-plane/recompute_plane.py

# 3) 拼装/源指纹
python target/qa-plane/assemble_check.py
python target/qa-plane/assemble_check2.py
python target/qa-plane/assemble_mixed.py
python target/qa-plane/eol_delta.py

# 4) 门禁（主树）
normify gate 2026-10-26-g10-p5-primitive-plane-cuda-independent-check --repo D:\Rust\mjwarp-rs
normify check 2026-10-26-g10-p5-primitive-plane-cuda-independent-check --repo D:\Rust\mjwarp-rs
```

（脚本原件在 `scripts/`，日志在 `logs/`；`wt-b1` worktree 与 `t-b1` target 为临时目录，复核后可清理。）

## 13. 观察清单（详见 `qa-observations.json`）

| id | 级别 | 摘要 |
| --- | --- | --- |
| O1 | 一般 | 测试 `TETRA` 常量 2 处与 `model.mesh_vert` 差 1 ULP（`0xbb93fae9` vs `0xbb93faea`），docstring「dump 自 mujoco」不准确；plane-mesh 无法位级全等 |
| O2 | 一般 | 交付源指纹 222849/`59631d83…` 依赖工作树 EOL，不可从提交树复现；LF 拼装 = 222369/`ce08822a…` |
| O3 | 建议 | 证据 §3 写 `product.cu` 356 行，实际 379 行（<450 棘轮仍满足） |
| O4 | 建议 | cylinder 接触 3/4 pos 结合顺序与 pinned 字面仍不同（本 case 位型相同、id2/3 未 detected）；ellipsoid 有非上游退化保护；convex 有 `1<<20` 迭代上限 |
| O5 | 建议 | 6 个日志含 UTF-8 BOM，`window-attempt4/console.log` 为 UTF-16LE（git 视为二进制）；README §5 第 94–95 行有控制字符残留 |
| O6 | 建议 | attempt2 无任何原始产物留档（仅叙述 runner bug），attempt1 的 guard.log 未归档（只有 facts 的 guard_post + samples.csv） |
| O7 | 一般 | addendum 的 README §5.1/§5.2 + receipt 字段 + attempt1 samples.csv 目前 **staged 未提交**；staged 门禁 runs 是在含 B2 在途改动的工作树上重跑（973/371，18.3 s），正式依据待 B2 冻结后重跑提交；receipt 已加 `fix_regate_note` 说明，B1 自身基线用 worktree 972/0/370 佐证 |

## 14. 证据清单

- `receipt.json`：结论/范围/关键数字/门禁/边界。
- `qa-semantics.json`、`qa-criteria-scope.json`、`qa-device-reference-recompute.json`、`qa-source-fingerprint.json`、`qa-window-attempts.json`、`qa-addendum-waiver.json`、`qa-observations.json`。
- `logs/`：gate、smoke-b1、unit-b1-summary、recompute（json+log）、assemble×3、eol-delta、tetra-bits、tree-state。
- `scripts/`：复算/拼装/行尾/常量位型脚本原件。
- `artifacts-sha256.json`：本目录全部产物 sha256（自洽校验用）。
