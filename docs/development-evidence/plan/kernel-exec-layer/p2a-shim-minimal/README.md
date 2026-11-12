# P2-a 书写面最小 API 语义冻结（25 条，定义片）

- 状态：L1 定义片证据（owner=Arch）；**只冻结声明面 / 语义 / 判据**，不实现 emitter、不改内核源、0 窗 0 GPU。
- 条目：`items.json` 共 **25 条** = `H.7`(1) + `H.8`(1) + `G.8`(1) + `E.3`(10) + `E.4`(5) + `E.5`(2) + `E.6`(2) + `E.7`(3)。
- 六要素：每条含 `capability_id` / `behavior` / `judgement` / `upstream_anchor` / `frozen_delta` / `evidence`。
- 判据摘要：`H.7`/`H.8` 各含生成源文本断言（原子形态命中 / 不命中）；`G.8` 两种索引模式各 1 条文本断言 + 发射期校验；
  `E.3–E.7` 22 个函数逐条给出 ≥3 个边界（含 `normalize(0)`→0 向量、奇异 `determinant` 不特判、
  非归一化 quat 不自动归一、spatial 分量序 `(ang,lin)`、`where` 不求值未选分支）。
- 上游锚点：`docs/implementation-plan.md` §并行计算单元能力定义 + 上游 `71da24d`；`E.*` 引 A1 头树
  （`include/warp-headers/{vec,mat,quat,spatial,builtin}.h`，行号以 A1 头树为准、P2-b 复核）。
- 边界：证据只覆盖本片冻结的语义与判据；emitter 生成、PTX 形态、值级对照在 **P2-b**。
  本目录不建 manifest（L1）；不改判据/容差/参考；不宣称与上游逐位相同。
- 复核方式：`items.json` 可直接解析校验 `items_count=25` 与八组计数；逐条 `judgement.cases` 是 P2-b 的断言清单。
