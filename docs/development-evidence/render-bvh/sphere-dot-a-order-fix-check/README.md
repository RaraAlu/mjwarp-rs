# 球面 `a = dot(vec,vec)` FMA 收缩次序 fix — 独立复核

任务：`2026-10-31-render-sdf-sphere-dot-a-order-fix-check`（QA；只读）
被核：`2026-10-31-render-sdf-sphere-dot-a-order-fix`（Arch；P8）
pin：`8126868c`（收口提交：`render_math.cu` + 证据 + 门禁 runs + 模块指纹；该提交 message 误写成「治理…去掉 in_progress 依赖」，
按 `git show` **内容**核，addendum `1e814b92` 已记录映射事实）
start 时 HEAD（`revision.before`）：`786a9342372a9e18a8ba25bb64c1434237d42840`
复核时间：2026-10-31 02:54–02:59（+08:00），空闲段；**零 GPU、未重跑窗口、未冻结、未编译**（U4 attempt3 body 在跑，遵「body 开始后零编译」纪律）

## 1. 结论

**pass / 阻断 0 / 需修 3 / 观察 6**

- 改动范围干净：只 `render_math.cu` **+12/−13**（309→308 行），语义只有一行
  `float a = dot3_ba(vec.x, vec.y, vec.z, vec.x, vec.y, vec.z);`；删除未再调用的 `dot3_ch0`；注释按 RCA 更正。
  `b`/`dd`（`dot3_ba`）、融合 `det`、`__frcp_rn`、根选择、`*dist = sol`、`depth = dist * -local_dir_z`、`normal` 全为原样；其它 `src`、`tests/`、`fixtures/`、`scripts/`、`Cargo*` 零改动。
- 内核身份：我用 `mod.rs::MEGAKERNEL_SOURCE` 的 6 个 `include_str!` 拼接**独立重算** source：
  before `778216b2…`/88,865 B → after `eb0e717f…`/89,063 B，与证据一致；PTX `34f91147…`→`7782aeb1…`/268,589 B（未独立复算，见观察 O4）；
  `aa_resolve` 三值不变（`aa_resolve.cu` 未改，结构性保证）⇒ **身份已变，attempt3 必须全新冻结、attempt1/2 的 92 条不得复用**。
- RCA 自洽（我用自带枚举驱动复跑）：**432 变体**（a/b/dd 各 6 种次序 × det 融合/非融合 = 6³×2），
  `a=(1,0,2) b=(1,0,2) det=fused` 家族 **44/44** 命中参考、0/44 命中产品；产品原变体 `a=(0,1,2)` **44/44 命中产品、0/44 命中参考**；
  次优 37/44。3.12 vs 3.15：两台相机的 `xpos/xquat/fovy` f64 **逐位相同**（我自核）⇒ parity 不是原因。
- 门禁：unit **1111 / 0 / 397**（run json exit 0，时间 18:53:40Z 早于提交 18:53:56Z）、fmt/clippy/clippy-cuda passed、`--no-run` EXIT=0 含 `Finished`。
- 历史证据零改写：`8126868c` 只碰 `render_math.cu` + 新证据目录 + 该任务 runs/card + `mjwarp.render.kernel` 模块文档；R02/R05/R08/R09 与 sdf 系列证据目录零改动。
- 证据卫生：manifest 11 行/12 文件重算 **0 不符**；`materials/src-diff.patch` 与真实 diff 一致（index `33efcb27..7a43e575`、2 个 hunk）。

需修 3 条（都是文档/证据措辞级，不影响源码与门禁结论）：
- **P1** `rca-summary.txt` 的变体算式写「3 个点积次序 × 6 排列 × det 融合/非融合 = 432」，但 3×6×2=36；实际是 **a/b/dd 各 6 种次序（6³）× 2 = 432**（脚本 `itertools.product(ORDERS,ORDERS,ORDERS,(fused,nonfused))`，我已复跑证实）。
- **P2** README §4 称 `logs/gate-*.txt` 是 `runs/<task>/<gate>.json` 的转写快照，但四份快照与 runs 均**不是同一次**：
  fmt 1610 vs 1712 ms；clippy 5629 vs 455 ms 且 `log_sha` 不同；clippy-cuda `log_sha` 不同；unit 95034 vs 12225 ms 且 `log_sha` 不同。
  数值结论一致（passed、1111/0/397），只是来源措辞要改（同 R08-S3 那次的修法）。
- **P3** rca-summary「唯一能位级复现参考的是 `a=(1,0,2) b=(1,0,2) det=fused`」应按家族表述：**dd 次序在此无关**，
  实际有 2 个并列 44/44（dd=(0,1,2) 与 dd=(1,0,2)）；固定代码的 dd 是 `(1,0,2)`。

## 2. 观察（记录级）

- O1 `render_shade.cu:10/:14` 注释仍列 `dot3_ch0`（已删）——纯注释；现在改会再次改内核身份（attempt3 在冻结/运行），
  建议留到下一次内核重基线一并改（同 U1b 的 `.cu` 注释先例）。
- O2 RCA 的「geom 值逐位相同」：相机侧我已逐位核；geom 侧仓库只有 3.12 dump，3.15 侧只能由同一 `scene.xml`（sha `376eb81c…`）间接支撑，建议标注为间接证据。
- O3 `validate.txt` 快照 0 err/6 warn（含 fingerprint-drift）早于同提交的模块指纹刷新；现跑 0 err/5 warn，属历史快照。
- O4 PTX 三值未独立复算（需 NVRTC 编译，窗口在跑禁编译）；source 三值我已独立重算。
- O5 `logs/no-run.txt` 是截断转写（`Finished` + 末两条 + `EXIT=0`），建议后续附原样日志。
- O6 提交映射异常按内容核通过：message 描述的是 `d3bca361` 的动作，内容是本 fix 全量；trailer 正确；addendum `1e814b92` 记录；fix 任务 verified。

## 3. 边界 / 不宣称

只读：不跑 GPU、不开窗、不冻结、不编译；不改源码与任何证据。
不宣称渲染数值验收通过 / 上游等价 / 精度达标；不宣称 U4 attempt3 会通过——数值结论只能由 attempt3 窗口给出。

## 4. 复现命令（只读）

```
git show 8126868c -- src/render/kernel/cuda/render_math.cu       # 单文件 +12/-13
git diff --numstat 8126868c^..8126868c -- tests/ fixtures/ scripts/ Cargo.toml Cargo.lock   # 空
# source 身份（6 个 include_str! 按 mod.rs 顺序拼接后 sha256）
#   8126868c^ => 778216b2…/88,865 B；8126868c => eb0e717f…/89,063 B
target\mjwarp-parallel\venv\Scripts\python.exe materials\recompute-rca-variants.py   # 432 变体 / 2 winners / 44-44
```

`materials/selfcheck-*.txt` 是本次重算的原始输出（diff / identity / rca / gates / parity / mapping）。
