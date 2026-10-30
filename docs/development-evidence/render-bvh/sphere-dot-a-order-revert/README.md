# 回滚球面 `a` 收缩次序 fix（attempt3 证伪其适用性）

任务：`2026-10-31-render-sdf-sphere-dot-a-order-revert`（Arch；P8；离线，零 GPU 窗口、零冻结）
被回滚：`2026-10-31-render-sdf-sphere-dot-a-order-fix`（`8126868c`，`a = dot3_ba` 即 (1,0,2)）
上游判定来源：U4 验证窗 attempt3（`INVALID_ROUND`，`target/render-bvh/sdf-validate-window/attempt3/`）

## 1. 为什么回滚（事实，不改写历史）

attempt3 暴露**两个独立问题**：

1. **流程无效**：`U1b` 仍处于回滚态（`07252cd2`），host 继续拒绝 `gtype==8` ⇒ `render_sdf_reference`
   5 个 GPU 用例全部在发射前 panic `Unsupported("S7b3 不含 SDF 求交")`；本 attempt 的 SDF 侧**没有任何判据依据**。
2. **真实参考冲突**：fix 把 `a` 改成 (1,0,2) 后，6 个 scene-a 回归实体全部红在**同一批球面轮廓像素**
   （首差 pixel 28969 / (169,120)），depth `max_ulp = 73`（>4）；这些像素在 fix 前是**位级 0 差异**。

即：scene-a 的 `use_fast_math=false` 参考（S7b1-kernel 采集，`fixtures/render-bvh/scene-a/s7b/s7b*`）
与 scene-sdf 的 `use_fast_math=false` 参考（U3 采集）**对 `a` 的 f32 收缩次序要求不同**（前者要 (0,1,2)，
后者按 fix 片 RCA 在 scene-sdf 数据集内位级复现的是 (1,0,2)）。**同一产品实现无法同时位级匹配两份参考**，
73 ULP 也远超 ≤4 ULP，不能用容差桥接。成因（两份采集的生成 kernel 为何不同）另做**只读 RCA**（`target/`），
本片不把成因写成定论。

按 Chief 处置：回滚 fix，产品回到与全部历史参考位级一致的状态；**U4 球轮廓判据保持「未通过」**，
不改判据/容差/参考；`U1b` 重做暂缓（SDF 能力维持不可达）。

## 2. 回滚内容（逐字节）

| 项 | 值 |
| --- | --- |
| 文件 | `src/render/kernel/cuda/render_math.cu` |
| fix（gen2）blob | `7a43e575…`（`a = dot3_ba`，`dot3_ch0` 已删） |
| 回滚后（gen3）blob | **`33efcb27…` == gen1 pre-fix blob，逐字节相同** |
| 内容 | `a = dot3_ch0(vec, vec)`（先乘第 0 分量）、`dot3_ch0` helper 存在、注释回到 fix 前文本 |
| diff | 相对 fix 提交反向 25 行（`materials/src-diff.patch`） |

`git diff --stat` 仅该文件；零其它 `src`/`.cu`/`tests`/`fixtures`/判据/参考改动。

## 3. 内核身份（三代）

| 代 | 触发 | megakernel source | megakernel PTX | PTX bytes |
| --- | --- | --- | --- | --- |
| gen1 | 原始（U1b 回滚后） | `778216b2…` | `34f91147…` | 268,589 |
| gen2 | fix `8126868c` | `eb0e717f…` | `7782aeb1…` | 268,589 |
| **gen3** | **本回滚** | **`778216b2…`（== gen1）** | **`34f91147…`（== gen1）** | **268,589** |

`aa_resolve` 三代均不变（`0bf8af5a…`/`b28e155c…`/27,536）。
探针命令与原始输出：`logs/kernel-identity-three-generations.txt`。
注：用 CRLF checkout 的同一 blob 测出的 source 是 `86a55721…`（换行形态影响 source 哈希）；本片把工作树
规范成 LF 后测得 `778216b2…`，且 committed blob 与 gen1 完全一致。

## 4. 门禁与构建

| 项 | 结果 |
| --- | --- |
| fmt / clippy / clippy-cuda | 通过 |
| unit | 通过，**1111 过 / 0 败 / 397 ignored** |
| `cargo test --locked --no-run` | **EXIT=0** 含 `Finished`（`logs/no-run.txt`） |
| `normify validate --fingerprints` | 错误 0（done 刷新模块指纹） |

`logs/gate-*.txt` 是某次门禁的转写快照；权威结果见 `.normify/runs/<task>/`。

## 5. 边界

不改判据/容差/参考/fixtures；不改写 fix 片证据目录（历史不动，关系在本 README 与 receipt 记录）；
零 GPU 窗口、零冻结；attempt3 冻结作废、不复用；U1b 重做暂缓；不宣称任何渲染数值通过或 U4 通过。
