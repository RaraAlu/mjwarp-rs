# M3 刚体运动学入口迁移证据

任务：`2026-10-20-layers-m3-rigid-kinematics`（M3刚体运动学入口迁移）。
基线 HEAD：`e9ae451`（超长文件拆分收口之后）。
本任务只迁移 `rigid_kinematics` 的三处底层发射，不改内核源码与浮点顺序。

## 实现范围

只迁移三处发射，其他入口保持原始发射：

| 路径 | 位置 | 迁移前 | 迁移后 |
| --- | --- | --- | --- |
| 普通探针 | `src/physics/kinematics.rs:274` | 裸 `kernel.launch(...)` | `RigidKernel::new(session, kernel).launch(...)` |
| 速度位置 | `src/physics/resident/plan_update_rigid.rs:52` | `self.rigid_kernel.launch(...)` | 同一调用改经类型包装 |
| update_rigid | `src/physics/resident/plan_update_rigid.rs:106` | `self.rigid_kernel.launch(...)` | 同一调用改经类型包装 |

- 探针编译点：`src/physics/kinematics.rs:273`；常驻编译点：`src/physics/resident/plan_subsets.rs:290`。内核编译发生在包装之外，`KINEMATICS_CUDA` 与 `FIELD_BATCH_CUDA` 源码逐字节未改。
- 速度位置同块的 `rigid_com_position` 发射只拆出 unsafe 块，未迁移。
- 新增包装层 `src/physics/rigid_kinematics/`：`mod.rs`（变体、ABI 槽位、计划与发射）、`metadata.rs`（元数据证明与精确跨度）、`tests.rs`（宿主规则测试）。
- 新增常驻适配 `src/physics/resident/rigid_upload.rs`（打包后类型化上传）与 `src/physics/resident/rigid_kinematics_tests.rs`（GPU 逐位对比）。
- `KinematicsPlan` 的 `rigid` 字段改为 `RigidModel`、`rigid_kernel` 改为 `RigidKernel`；`upload_kinematics_model` 返回 `RigidModel`。

ABI 与证明：

- 真实 ABI 保持四个指针与四个 u32，顺序 `meta, model, state, result, nq, nb, nj, worlds`。
- `meta/model/state` 声明 `Read`，`result` 声明 `ReadWrite`（内核回读父体位姿，G01 等价变体读 body0）。
- 变体证明区分普通（五段拓扑）、批量（另核字段描述符与 mocap 映射）与 G01 等价；父体前序、关节区间、关节类型与 qposadr 全部在范围内。
- 精确跨度逐槽核对：metadata 按变体元素数、model 参数或独立周期描述、state 含 qpos 与 mocap 位置姿态段、result 含守卫区；`nq=0` 保留真实单元素占位。
- 速度位置路径的外部 rigid 工作区要求同会话、精确长度，且不与计划内 metadata、parameters 与 state 重叠。
- 每线程仍只处理一个世界；网格 `div_ceil(256)`，工作量与全部乘加执行溢出检查。
- 任一拒绝先于内核写入；输出字节与 ready 状态保持；同一计划修复后可再次成功。

## 清单

扫描脚本：`build-inventory.ps1`（本目录）。清单：`unmigrated-launch-inventory.json`。

- 以 v5 清单 `docs/development-evidence/layers-m3-velocity-position-g01-correction/unmigrated-launch-inventory.json` 为基线，把全部行重定位到 61 片拆分后的布局，再记录本任务迁移。
- 旧文件到新组：拆分任务 scope 的根文件加产品子模块目录；`resident.rs` 与 `physics/mod.rs` 两个拆分前特例单独登记组。
- 行定位：组内按 `(receiver, method)` 顺序匹配发射行，编译点按当前产品前缀扫描重建；探针行因接收者文本改变单独定位。
- 计数结果：

| 字段 | 值 |
| --- | --- |
| unmigrated_low_level_call_sites | 68 |
| typed_low_level_call_sites | 31 |
| unmigrated_actual_entry_kinds | 55 |
| unresolved_dynamic_compile_sites | 0 |
| uncertain | `[]` |
| product_low_level_call_sites | 99 |
| wrapper_orchestration_call_sites | 30 |
| compile_call_sites | 89 |
| actual_cuda_entry_kinds | 88 |

## GPU 逐位对比

设备：NVIDIA GeForce RTX 4070 Ti SUPER，驱动 596.36，显存 16376 MiB。资源锁 `gpu:windows-t4` 由任务启动时持有。

命令（每次 NVRTC 使用独立空捕获目录）：

```text
CUDA_PATH=target/toolchains/cuda-12.8.1
PATH=%CUDA_PATH%\bin;%PATH%
MJWARP_PHYSICS_PTX_DIR=target/m3-ptx/<run>
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features cuda-probe --lib -- --ignored rigid_kinematics_tests:: --test-threads=1 --nocapture
```

两轮独立运行均 4 项通过（`run-01` 2.62s、`run-02` 2.64s）：

| 测试 | 覆盖 |
| --- | --- |
| `plain_probe_path_is_bitwise_stable` | 普通探针类型路径与原始发射逐位一致；关节类型 Some(3)/Some(0)/None |
| `strict_plan_matches_raw_and_recovers_after_metadata_write` | 严格常驻计划逐位一致；坏元数据拒绝后输出与 ready 不变、修复后恢复；nq=7 与 nq=0 |
| `equivalent_plan_matches_raw_after_seeded_body_zero` | G01 等价计划逐位一致；513 世界，预置 body0 位姿 |
| `velocity_position_matches_rigid_and_rejects_workspaces` | 速度位置工作区逐位一致；长度不符、异会话、状态槽与结果槽别名拒绝；坏元数据拒绝不改写工作区；两类变体 |

PTX 逐字节等于冻结基线，捕获目录文件名即哈希：

| 变体 | 哈希 |
| --- | --- |
| 普通 | `788c870d787e06d58bce113eef24846c0b14aa43f524189a3508f58fc940f280` |
| mocap 批量 | `93318e85811ad7db5a50c3217eb08201ae0d51b649c6d69feec1f5b6355239a4` |
| mocap+G01 等价 | `58cb789359125491c79f2b208717b69fbcf18ef1f9172fbfad27a7fe9d02cdb7` |

日志：`runs/run-01-gpu-tests.log`。

## 门禁

本任务检查：gate fmt、gate clippy、gate clippy-cuda、gate unit、清单 json 五项、`unchanged`（fixtures、native、include、build.rs、docs/evidence、Cargo.toml、Cargo.lock、rust-toolchain.toml、src/runtime）、`forbid_claim`。

| 门禁 | 结果 |
| --- | --- |
| fmt | 通过（960ms） |
| clippy | 通过（2624ms） |
| clippy-cuda | 通过（6049ms） |
| unit | 通过，332 过 0 败（14259ms；新增 3 项宿主规则测试） |

`normify validate`：0 错误（1 条既有叶子过粗结构警告）。

## 精度映射

- 全部物理输入、中间量与输出保持 f32；不新增 f64 路径，不改 FMA、容差、fixtures、CUDA 内核字符串与编译选项。
- 迁移只改 Rust 调用包装与元数据证明，PTX 与冻结基线逐字节相同（见上表）。
- 本次不宣称 NP01 至 NP04、GPU 性能、Linux 或 release 验收，也不宣称完整 M3。

## 未验收项与债务边界

- G01 全量回归未通过属既有债务：gravity 冻结位证书 2 bit 差异在基线 worktree `360718f` 同样失败（归 `2026-10-22-g01-numerics-audit`），`MJWARP_PHYSICS_PTX_DIR` 的报告路径与 PTX 捕获目录存在双语义冲突。本任务的定向测试用独立空捕获目录规避后者，不承担修复。
- 门禁脚本历史过期已由 `2026-10-22-g01-gate-script-sync` 承接；本任务不跑 g01-full。
- 其余 `rigid_com_position`、`camlight_rigid`、`attached_kinematics` 等入口保持原始发射，属后续切片。

## 证据文件

- `build-inventory.ps1`：清单重建脚本（不覆盖 v5 旧清单）。
- `unmigrated-launch-inventory.json`：v6 清单。
- `runs/run-01-gpu-tests.log`：GPU 测试日志。
- 交付回执：`docs/development-evidence/changes/2026-10-20-layers-m3-rigid-kinematics.json`。
