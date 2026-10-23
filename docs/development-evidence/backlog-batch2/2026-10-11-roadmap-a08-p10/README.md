# A08@P10 零拷贝切片：借用式视图、所有权与跨流完成事件

任务：`2026-10-11-roadmap-a08-p10`。日期：2026-10-23。执行：Backend（批次 2）。
矩阵：A08 零拷贝优化（`上下文、所有权与跨流事件` / P10）。

## 1. 条款语义与细化口径

A08 验收为「上下文、所有权与跨流事件」。零拷贝本身属优化验收项，必须先有
安全接入路径（矩阵 §运行时与数据接口：若未达零拷贝，不得宣称已支持）。按当前
代码布局，本片交付**宿主侧零拷贝契约**（`src/runtime/external/view.rs`）：

- **上下文**：`MirrorBuffer` 携带 `name/device_ordinal/context/allocation/stream/
  generation`，表示设备缓冲的宿主镜像与分配身份。
- **所有权**：`ZeroCopyView<'a>` 只借用 `MirrorBuffer`，不持有/复制数据；
  `field()`/`elements()` 返回指向原缓冲的切片，`copy_field` 是显式复制对照。
- **跨流事件**：`StreamFence { stream, generation, completed }` 复刻
  `runtime::completion` 的提交/完成契约；视图仅在**同流、已完成、同代数**时构造，
  否则分别返回 `StreamMismatch`/`EventPending`/`GenerationMismatch`。

## 2. 实现

| 文件 | 内容 |
| --- | --- |
| `src/runtime/external/view.rs` | `MirrorBuffer`（所有者）、`StreamFence`（事件）、`FieldLayout`、`ZeroCopyView`（借用视图）、`copy_field` 对照与 `ZeroCopyError` |
| `src/runtime/external/view/tests.rs` | 指针相等（零拷贝证明）、跨步视图、事件/流/代数拒绝、写入与代数、越界/空缓冲 |
| `src/runtime/external.rs` | 新增 `pub mod view;`（316→318 行，未超 500 行上限） |

## 3. 验证（本次实测）

| 判定 | 结果 |
| --- | --- |
| 零拷贝证明 | `view.field()` 切片指针 == 原缓冲指针 + offset×4 字节；`copy_field` 与视图值一致 |
| 跨步视图 | `stride=3, width=2, count=2` 逐元素切片指针等于原缓冲偏移（`1`、`4` 元素） |
| 所有权 | 视图仅借用所有者（`owner()` 指针等于 `&MirrorBuffer`）；接口无 `to_vec`/复制 API |
| 跨流事件 | 未完成 → `EventPending`；异流 → `StreamMismatch`；代数过期 → `GenerationMismatch`；重复完成 → `AlreadyCompleted` |
| 写入与代数 | `write` 递增 generation；旧事件失效，新事件可建视图并读到新值 |
| 边界拒绝 | 空缓冲 → `EmptyBuffer`；越界字段/写入 → `OutOfBounds` |

复跑：`cargo test --locked --lib external::view` → **5 过 0 败**；`cargo fmt --check`
与 `cargo clippy --all-targets -- -D warnings` 通过。

## 4. 验收指标映射（NP01–NP04）

- **NP01**：数据路径全 f32（视图不引入精度变换）。
- **NP02**：记录宿主侧契约、流/代数/完成位语义与指针相等证明。
- **NP03**：不适用——无物理数值输出，仅有契约与所有权判定。
- **NP04**：不适用——不做 GPU 性能测量，不宣称零拷贝性能。

## 5. 未验收边界

- 未实现真实 CUDA 事件与设备指针（`runtime::completion` 的 submit/wait 实机路径）。
- 未实现设备内存分配、空闲池化与跨流并发（A03/A04）。
- 未验收零拷贝 GPU 数据路径的性能与 A07 设备数据视图的复制/同步契约。
- 不宣称完整 A08 或零拷贝支持结论；不改变任何既有运行时路径。

## 6. 复现

```powershell
cargo test --locked --lib external::view -- --nocapture
```
