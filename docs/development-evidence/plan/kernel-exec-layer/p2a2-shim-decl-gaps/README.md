# P2-a2 书写面声明缺口增补（5 条，定义片）

- 状态：L1 定义片证据（owner=Arch）；只增补**声明面与判据**，不实现 emitter、不改内核源、0 窗 0 GPU。
- 与 P2-a 的关系：P2-a 的 **25 条保持独立可审计**；本卡是 5 条声明面增补（不合并成 30 条、不改 P2-a 判据/编号）。
- 条目：`items.json` 共 **5 条** = `G1`(D.1 控制流) / `G2`(D.5 整数运算) / `G3`(D.2 设备函数互调) /
  `G4`(D.8/D.9 下标与指针偏移) / `G6`(I.3 块内屏障)，每条含 `capability_id` / `gap_id` / `behavior` /
  `judgement` / `upstream_anchor` / `frozen_delta` / `evidence`（六要素 + 缺口 ID）。
- 依据：`p2b-pilot-coverage-gap.md`（`964e42b5…`，12 缺口，其中 5 条必修）；`build_rays.cu`、`rne.cu`、
  `mixed/solve/cuda.rs` 的实际构造锚点已逐条写进 `upstream_anchor`。
- 关键写死项：D.5 溢出**补码回绕**（Rust `wrapping_*`，debug 不 panic）、移位按位宽掩码、除零不 panic/陷阱；
  I.3 **仅块内**屏障（网格级显式拒绝）；D.2 不支持递归（P2-c 再评）；D.8/D.9 只暴露 opaque 视图句柄。
- 边界：G2 之外仍不覆盖模板泛型（G7）、switch 参数化（G8）、inline PTX（G9）——按裁定延后 P2-c，
  P2-b 的排除项清单见 P2-b 卡。
- 复核方式：`items.json` 可解析校验 `items_count=5` 与 `counts`；`judgement.cases` 是 P2-b 的断言清单。
