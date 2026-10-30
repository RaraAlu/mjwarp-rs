# 球面求交 `a = dot(vec,vec)` FMA 收缩次序对齐（U4 sphere 轮廓 depth RCA 修复片）

任务：`2026-10-31-render-sdf-sphere-dot-a-order-fix`（Arch；P8；离线，零 GPU、零窗口）
触发：U4 验证窗 attempt1/attempt2 均 `INVALID_ROUND`，`render_sdf_reference` 的 sphere 轮廓 depth 超判据
（cam0 215/76,800、max |ULP| 191；cam1 4/43,200、max |ULP| 25；首差 (230,69)、参考 seg=(2,5)）。
RCA：只读、离线（`target/sphere-depth-rca.md`，摘要见 `materials/rca-summary.txt`）。

## 1. 根因（RCA 定论）

球面解析求交的 `a = dot(vec, vec)` 的 **f32 FMA 收缩次序**与上游 warp 1.15 GPU 生成码不同：

| 项 | 参考（warp GPU） | 产品（修复前 `dot3_ch0`） |
| --- | --- | --- |
| `a` | mul(y·y) → fma(x·x) → fma(z·z) | mul(x·x) → fma(y·y) → fma(z·z) |
| `b` / `dd` | mul(1 号分量) → fma(0) → fma(2) | 同左 |
| `det` | `fma(b,b,−(a·c))` 融合 | 同左 |

证据强度：44 个诊断像素上，产品链自身 **44/44** 位级复现产品 depth；固定该链后枚举
3 个点积次序 × 6 排列 × det 融合/非融合 = **432 变体**，唯一能位级复现参考的是
`a=(1,0,2) b=(1,0,2) det=fused`（**44/44**），产品原变体 `a=(0,1,2)` 对参考 **0/44**。
机制：`a` 只依赖射线方向，多数方向两种次序舍入相同；球轮廓处 `det≈0`，`a` 的 1 ULP 经 `sqrt(det)`
放大成 7–191 ULP depth 差。3.12 dump 与 3.15 capture 的 cam/geom 值逐位相同 ⇒ **不是 parity 问题**。
S7b1/S8 当时 747 像素 probe 全等是「样本无反例」，文件旧注释据此写反，本片一并更正。

## 2. 改动（只一个文件、只一处语义）

`src/render/kernel/cuda/render_math.cu`（+12/−13 行）：

- `render_sphere_hit`：`float a = dot3_ba(vec.x, vec.y, vec.z, vec.x, vec.y, vec.z);`（原 `dot3_ch0`）
- 删除不再使用的 `dot3_ch0`
- 注释按 RCA 更正（含 432 变体/44-44 复现结论、S7b1/S8 样本无反例的说明）

`b`、`dd`、融合 `det`、`1/a`、根选择、`depth = dist * −local_dir_z`、相机处理、判据/容差/参考/fixtures
**零改动**；完整 diff 见 `materials/src-diff.patch`。

## 3. 内核身份（新旧三值）

| 探针 | before | after |
| --- | --- | --- |
| megakernel source | `778216b2…` | `eb0e717f…` |
| megakernel PTX | `34f91147…` | `7782aeb1…` |
| megakernel PTX bytes | 268,589 | 268,589（同长，内容不同） |
| aa_resolve | 不变（`0bf8af5a…`/`b28e155c…`/27,536） | 不变 |

探针命令与原始输出见 `logs/kernel-identity.txt`。
**结论：内核身份变了 ⇒ U4 attempt3 必须全新冻结，attempt1/attempt2 的 92 条冻结一律不得复用。**

## 4. 门禁与构建

| 门禁 | 结果 |
| --- | --- |
| fmt | 通过 |
| clippy | 通过（`dot3_ch0` 删除后无未使用告警） |
| clippy-cuda | 通过 |
| unit | 通过，**1111 过 / 0 败 / 397 ignored** |
| `cargo test --locked --no-run` | **EXIT=0** 且含 `Finished`（`logs/no-run.txt`） |
| `normify validate --fingerprints` | 错误 0（`materials/validate.txt`） |

`logs/gate-*.txt` 是 `.normify/runs/<task>/<gate>.json` 的转写快照；权威结果以当时 `runs/` 记录为准。

## 5. 边界（没做什么）

不跑 GPU 窗口、不冻结、不跑 body；不改判据/容差/参考/fixtures；不改写任何既有证据目录
（R02/R05/R08/R09 与 sdf 系列历史证据零改动，见卡面 `unchanged`）；不宣称任何渲染数值通过。
U4 attempt3 按同一条已签条件式协议、全新冻结、判据零放宽，等 Chief 开窗令；A1（mesh 16→20）排在其后。
