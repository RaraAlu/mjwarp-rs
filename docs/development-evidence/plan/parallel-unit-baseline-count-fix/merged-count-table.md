# 能力量化下限合并表（终稿草案，三口径脚注版）

- 对象：`docs/implementation-plan.md` §并行计算单元能力定义（提交 `efadf21c`）。
- 上游：`mujoco_warp` @ `71da24d956378a87a703b6e1442b13aec0c4ac29`；产品：工作区 HEAD `efadf21c`。
- 我方依据：`scan_upstream*.py`、`classify_kernels.py`、`recheck_bare_returns.py`、`verify_diff48.py`（本目录）；对照 `counting-rules-appendix-draft.md`、`number-corrections.md`。
- 对方依据：`target/render-bvh/gl2-ptx-diff/pu-baseline-review/reconcile/reconcile.md` = `c21af22e4b859afd…` / 38,966 B（v3，§5.1 差集逐行、§5.2 三口径、§6.4 裸 return 明细，22 条 manifest 自核）。
- 用法：文档改数时以"定稿值"列为准；"口径"列必须随数字一起落文，否则不可复算。
- 说明：本表把文档里出现的独立数字组并成 29 行；Backend `reconcile.md` 的"32 精确 / 8 约数"是把 `D.1`（3 个数）、`H.1`（2 个数）等拆细后的计数，两者不矛盾。

## 1. 合并表

| # | 项（能力编号） | 文档值 | 定稿值 | 口径 | 结论 |
| --- | --- | --- | --- | --- | --- |
| 1 | 上游内核数 | 361 | **361** | U-DEF 文本级（跨行装饰器计 1） | 精确 |
| 2 | 上游设备函数 | 411 | **411** | `@wp.func` 413 − `@wp.func_native` 2 | 精确 |
| 3 | `B.1` 分配 未初始化/清零/填值 | 195/73/13 | **195/73/13** | `wp.empty(` / `wp.zeros(` / `.fill_(` | 精确 |
| 4 | `B.4` 原地清零与填值 | 51/13 | **51/13** | `.zero_(` 51 / `.fill_(` 13 | 精确 |
| 5 | `B.5` 设备内拷贝 | 17 | **17** | `wp.copy(` | 精确 |
| 6 | `B.6` 范围回读 | 152 | **152** | `.numpy()` | 精确 |
| 7 | `B.6` 宿主上传 | 237 | **未定** | 候选 113（`wp.array(`）/179（`.numpy()` 含包根）/265（和） | 仍不成立（规则缺失） |
| 8 | `D.1` while/break/continue | 69/70/123 | **69/70/123** | U-BODY | 精确 |
| 9 | `D.1` 动态上界循环 | 698 | **726** | U-BODY `for`（疑漏"常量上界排除"） | 仍不成立（差 28） |
| 10 | `D.1` 提前返回 | 429 | **429** | `ast.Return(value=None)`：kernel 405 + func 24；`return 0`/`return x` 不计 | 精确（我方 428 差 1 根因：`sensor.py:747` 的 `return  # should not occur` 被误计为带值） |
| 11 | `D.2` 单函数最多参数 | 58 | **58** | `eval_ccd_write_contact` | 精确 |
| 12 | `G.1` 1/2/3 维索引 | 75/255/29 | **75/255/29** | `wp.tid()` 解包元数（单/双/三；和 359） | 精确 |
| 13 | `G.3` 参数总数 | ≥80 | **80** | `_render_megakernel` | 精确 |
| 14 | `G.3` 数组参数 | ≥76 | **73** | 80 = 73 `wp.array*` + 7 标量（含 2 个 `uint64` 句柄）；全上游数组参数最大值即 73 | **须改**（冲突） |
| 15 | `G.5` 同缓冲输入输出 | 24 | **24（待核）** | 两规则并存：`wp.launch_tiled(` 24（与 G.6 同数属巧合未证） | 待核 |
| 16 | `G.6` 按块发射 | 24 | **24** | `wp.launch_tiled(`（计数对象 = tiled 发射点，非块数） | 精确 |
| 17 | `G.7` 块宽取值 | 32/64/128/256 | **取值非计数** | 裸 `block=` 21 处，需逐 `wp.launch` 解析 | 口径修正（非计数项） |
| 18 | `H.1` 原子加 | i32 87；f32 66 | **87/66** | 第一实参注解类型解析（`atomic_add(` 文本 180 含复合原子） | 精确 |
| 19 | `H.3` 原子减 | i32 6；f32 6 | **6/6** | 同上（文本 16 含复合原子） | 精确 |
| 20 | `H.4`/`H.5` | 18 / 2,1 | **18 / 2,1** | `atomic_or(` / `atomic_min,max(` | 精确 |
| 21 | `I.3` 块内栅栏 | 2 处显式 | **2** | 上游 `wp.synchronize(`（`__syncthreads` 是产品 CUDA，不在上游 Python） | 精确 |
| 22 | `I.5` 块级算子 | 19 种 / 255 处 | **23 种 / 287 处** | `wp.tile_*` 调用点 | 仍不成立（差 32 处 / 4 种） |
| 23 | `I.6` 块内线代 | 26 | **26** | matmul 11 + chol_inplace 5 + lower 4 + upper 3 + chol_solve 3 | 精确 |
| 24 | `L.2` 设备事件 | 2 | **2** | `wp.Event` | 精确 |
| 25 | `L.3` 内核打印 | 32 | **32** | `wp.printf` 31 + `wp.print` 1（`collision_sdf.py:395`） | 精确 |
| 26 | `F.3` 工厂 | 87 | **87** | `@cache_kernel` | 精确 |
| 27 | `F.4` 独立编译单元缓存 | 91 | **未定** | `module="unique"` 105 / `@cache_kernel` 87 | 口径待定 |
| 28 | `C.7` 显式类型转换 | 9 种 / 1000 处 | **≈1040** | 约数（Backend 实测） | 约数（文档写"约"即可） |
| 29 | 手写 CUDA 内核 | 121 | **107 / 95 / 107 / 112**（源定义 / 产品命名 / 唯一名 / 编译入口；产品入口 100） | 见§2 脚注；121 = `__global__ void` 文本数（`.rs` 99 + `.cu` 22） | 口径修正（数字替换） |

统计（本表 29 行）：**精确 20 行**（#1–6、8、10–13、16、18–21、23–26）；**须改 1 行**（#14）；**仍不成立 3 行**（#7、#9、#22）；**待核/待定 2 行**（#15、#27）；**口径修正但非"错" 2 行**（#17、#28）；**产品口径定稿 1 行**（#29）。

## 2. 口径脚注（数字必须带其一）

### 2.1 上游（`71da24d`，`mujoco_warp/_src` 34 个非测试文件）

| 代号 | 定义 | 例 |
| --- | --- | --- |
| U-DEF | 定义级：`@wp.kernel`/`@wp.func` + `def` 计 1（跨行装饰器计 1；`@wp.func_native` 不计） | 361 内核 / 411 设备函数 / 87 工厂 |
| U-CALL | 调用点：`wp.xxx(` 出现次数（注释、docstring 内的不计） | `B.1`/`B.5`/`L.3`/`I.5` |
| U-BODY | 设备函数体范围内语句出现次数（按括号配平跳签名，再按 `def` 缩进截断） | `D.1` 控制流 |
| U-AST | Python AST 语义（`ast.Return(value=None)`；注释不算值） | `D.1` 裸 return |
| 类型解析 | 原子按第一实参数组的 dtype 注解解析；复合类型单列 | `H.1`/`H.3` |

范围警告：包内非测试 45 个文件（含包根）会改 3 个数：`wp.zeros` 73→76、`wp.copy` 17→23、`.numpy()` 152→179。

### 2.2 产品（`src/**/*.rs` 内嵌源 + `src/**/*.cu`）

| 代号 | 数值 | 定义 |
| --- | --- | --- |
| P-DEF | **107** | 源定义：`.rs` 90 + `.cu` 17（`.cu` 22 中 5 处为文件头 ABI 注释；`.rs` 91 中 1 处为宏模板） |
| P-PROD | **95** | 产品命名：107 − 12 个测试探针（8 路径可识别 + 4 文件内 `#[cfg(test)]`） |
| P-UNIQ | **107** | 唯一内核名（90 + 17，无重名）；产品命名范围 95 |
| P-ENTRY | **112**（产品 100） | 编译入口：源定义 107 + 宏 `small_prefix_factor_##N` 展开 5；产品 95 + 5 |
| 参考 | 121 / 125 / 114 / 91 / 108 | 文本数（`__global__ void` = 121；裸 `__global__` = 125）与 v1 中间数，仅交叉校验 |

一句话写法（文档直接用）：

> 现有手写 CUDA 内核 **95 个产品命名入口**（源定义 107，含 12 个测试探针；`.rs` 内嵌 90 + `.cu` 17；唯一名 107；宏展开后编译入口 112、产品 100）。

## 3. 文档侧待办（按优先级）

1. **必改**：`G.3` 数组 ≥76 → **73**（或改措辞为"78 = 73 数组 + 7 标量"式结构，见 `number-corrections.md` §1）。
2. **必改**：121 → **95 个产品命名入口**（脚注四口径）。
3. **建议**：`B.6` 237、`D.1` 698、`I.5` 19/255、`F.4` 91 四项补规则或降级为"约数"；`G.5` 24 与 `G.6` 24 拆开写并注明计数对象；`G.7` 标"取值"；`C.7` 标"约"。
4. **建议**：把 §2 口径脚注整段落进"依据与留白"，并给每个数字标口径代号。

## 4. 复核

```
cd D:\Rust\mjwarp-rs
target\mjwarp-parallel\venv\Scripts\python.exe target\impl-plan-review\scan_upstream.py      # 361/411/调用点
target\mjwarp-parallel\venv\Scripts\python.exe target\impl-plan-review\scan_upstream5.py     # U-BODY
target\mjwarp-parallel\venv\Scripts\python.exe target\impl-plan-review\classify_kernels.py    # P-DEF/P-PROD
target\mjwarp-parallel\venv\Scripts\python.exe target\impl-plan-review\verify_diff48.py       # 103/99/91 差集
target\mjwarp-parallel\venv\Scripts\python.exe target\impl-plan-review\recheck_bare_returns.py # D.1 裸 return 429
target\mjwarp-parallel\venv\Scripts\python.exe target\impl-plan-review\check_b4_f4.py         # B.4 51/13
target\mjwarp-parallel\venv\Scripts\python.exe target\impl-plan-review\check_f4.py            # F.3 87 / F.4 105
```
