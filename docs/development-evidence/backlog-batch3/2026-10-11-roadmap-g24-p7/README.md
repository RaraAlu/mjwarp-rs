# G24@P7 历史与延迟切片：环形索引、插入、读取与局部重置

任务：`2026-10-11-roadmap-g24-p7`。日期：2026-10-23。执行：Backend（批次 3）。
矩阵：G24 历史与延迟（`控制与传感历史、时钟` / `环形索引与局部重置` / P7）。

## 1. 阶段语义与细化口径

旧计划 G24@P7 只有粗切片（控制与传感历史、时钟；验收「环形索引与局部重置」）。
按当前代码布局，本片交付 `physics::history` 的**宿主侧最小切片**（不接 forward 调度）：

- `HistoryBuffer`：与上游 `d.history` 一致的每世界区域
  `[user, cursor, times(n), values(n*channels)]`；物理索引映射
  `phys=(cursor+1+logical)%n` 与环形二分 `find_index`（`t<=最旧→0`、`t>最新→n`）。
- 插入：标量/向量两条路径，覆盖精确匹配替换、早于最旧替换最旧、新于最新
  推进 `cursor`、乱序左移插入；向量路径精确匹配时不重写时间戳（与上游一致）。
- 读取：端点外夹取到最旧/最新、精确匹配（`|t-times[i]|<1e-6`）直取、
  零阶保持（interp=0）与线性插值（interp=1）；`read_delayed` 按 `t-delay` 读取。
- 局部重置：执行器（`user=0`、`cursor=n-1`、`times[j]=-(n-j)*dt`、值清零、
  SO3+QUAT 每样本首通道置 1）与传感器（delay 分支 `user=-dt`；interval 分支
  `user=t0`、`times[j]=ceil((t0-(n-1-j)*period)/dt)*dt`）；支持按世界掩码局部重置。

## 2. 实现

| 文件 | 内容 |
| --- | --- |
| `src/physics/history/buffer.rs` | `DelayBufferLayout`、`HistoryInterp`、`HistoryBuffer`（映射/二分/插入/读取/延迟读取/掩码局部重置）与 `HistoryError` |
| `src/physics/history/tests.rs` | 冻结上游回放、重置逐位对照、原生 C 参考、索引/拒绝面 |
| `src/physics/history.rs` / `src/physics/mod.rs` | 子模块挂接与导出（既有 `HistoryLayout` 行为不变） |

## 3. 参考来源与生成方式

| 项 | 值 |
| --- | --- |
| 模型源 | `fixtures/history/model.xml`（sha256 `5f52046d…`；2 执行器历史 + 1 无历史；3 传感器历史；nhistory=38、nworld=2、timestep=0.01） |
| 用例 | 冻结上游自定义内核回放 4 个场景（real_actuator0 / real_sensor0 / scalar_n4 / vector_n4_dim3）、33 次读取；重置 4 个用例（initial / all_reset / local_mask_first / local_mask_second）+ 合成 SO3+QUAT |
| 上游参考 | MJWarp `71da24d956…`，Warp 1.15.0：`history._history_physical_index/_find_index/_insert_scalar/_insert_vector/_read_scalar/_read_vector` 与 `_reset_actuator_history_kernel`/`_reset_sensor_history_kernel` |
| 原生参考 | MuJoCo 3.15.0（pinned venv）f64 `MjData.history` / `mj_resetData` |
| 生成脚本 | `scripts/authoring/author-g24-history.ps1` + `.py`（离线；产品测试不启动 Python） |
| 产物 | `fixtures/history/cases.json`、本目录 `authoring.json` |

## 4. 对照结果（本次实测）

| 判定 | 结果 |
| --- | --- |
| 产品 vs 冻结上游缓冲字 | 4 个回放场景的扁平缓冲字**逐位一致**（位型不一致 0） |
| 产品 vs 冻结上游读取值 | 33 次读取共 51 个值，**逐位一致**（max abs **0.000e+00**） |
| 产品 vs 冻结上游重置 | 4 个重置用例（含掩码局部重置）与 SO3+QUAT 合成重置**逐位一致** |
| 产品 vs 原生 f64 一级判定 | `mj_resetData` 的 `d.history` max abs **2.682e-09** ≤ 1e-6（延用上游 history 测试同口径） |

读取的精确匹配、ZOH 与线性插值均与上游 f32 逐位相等（本次未出现 FMA 末位差）。
复跑：`cargo test --locked --lib history` → **6 过 0 败**；全量 `cargo test --locked`
**470 过 0 败**；`cargo fmt --check` 与 `cargo clippy --all-targets --locked -- -D warnings` 通过。

## 5. 验收指标映射（NP01–NP04）

- **NP01**：新增路径全 f32（时间戳、cursor 槽、值与插值算术，无 f64 中间量）。
- **NP02**：记录冻结上游提交、`history` 函数名、模型 sha256、缓冲布局
  （actuator nsample/interp/adr、sensor interval/delay）与环境。
- **NP03**：原生 C f64 先行（2.682e-09），再与 MJWarp f32 参考逐位对照（0 差异）。
- **NP04**：**不适用/未测**——不发射内核、无批量耗时与显存记录。

## 6. 未验收边界

- 未实现三次插值（interp=2 Catmull-Rom）；`HistoryInterp::from_raw(2)` 显式拒绝。
- 未实现 `init_ctrl_history`/`init_sensor_history` 公共初始化入口，以及
  interval 运行期门控（`_insert_sensor_history_stage` 的 period 推进）、
  `apply_sensor_delay`/`read_ctrl`/`read_sensor` 的 forward 调度封装与 sensor 全类型。
- 未实现 `d.history` 全量分配与 actuator/sensor registry 组装、GPU 内核与图执行；
  不宣称完整 G24 或 P7 验收通过；NP04 未测。

## 7. 复现

```powershell
scripts/authoring/author-g24-history.ps1
cargo test --locked --lib history -- --nocapture
```
