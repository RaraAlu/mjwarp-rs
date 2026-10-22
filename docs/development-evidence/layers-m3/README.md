# M3 安全包装同步迁移证据

## 目标与验收

把 `src/physics`、`src/runtime` 中剩余未迁移的底层同步内核发射全部改走统一类型化包装，
直到清单四项归零：

- `counts.unmigrated_low_level_call_sites = 0`
- `counts.unmigrated_actual_entry_kinds = 0`
- `counts.unresolved_dynamic_compile_sites = 0`
- `uncertain = []`

## 清单链

- 基线：`docs/development-evidence/layers-m3-rigid-kinematics/unmigrated-launch-inventory.json`（v6）。
  - 低层调用 99、未迁移 68、入口 88、未迁移入口 55。
- 逐批版本：`unmigrated-launch-inventory-v7.json` 至 `unmigrated-launch-inventory-v30.json`。
- 终版（canonical）：`unmigrated-launch-inventory.json`（v30）。
- 生成脚本：`refresh-inventory.ps1`；用法：

```text
target/toolchains/powershell-7.6.7/bin/pwsh.exe -NoProfile `
  -File docs/development-evidence/layers-m3/refresh-inventory.ps1 `
  -Prior <上一版> -Patch <work-XX-*.json> -Out <新版>
```

脚本以上一版为基线应用单批补丁（移除/新增底层行、替换编译点、入口状态、家族），
重算派生计数并追加手工交叉核对记录；输出文件必须不存在，旧版全部保留。

## 终版计数（v30）

| 计数 | 值 |
| --- | --- |
| compile_call_sites | 89 |
| actual_cuda_entry_kinds | 88 |
| typed_actual_entry_kinds | 88 |
| unmigrated_actual_entry_kinds | 0 |
| kernel_abi_families | 85 |
| product_low_level_call_sites | 77 |
| typed_low_level_call_sites | 77 |
| unmigrated_low_level_call_sites | 0 |
| wrapper_orchestration_call_sites | 98 |
| excluded_non_kernel_helpers | 4 |
| dynamic_compile_sites | 1 |
| unresolved_dynamic_compile_sites | 0 |

## 批次记录

每批一个 Work 提交，均为“包装 + 元数据证明 + 精确跨度 + 会话/别名 + 故障测试 + 清单刷新”。

| 批次 | 入口 | 提交 |
| --- | --- | --- |
| work-01 | tendon_velocity_validate/compute | 6c7e6d6 |
| work-02 | mass_multiply_validate/mass_multiply | 0bea70d |
| work-03 | borrowed_mass_validate/multiply | 70fc1e9 |
| work-04 | resident_mass_factor/substitute | d156ea1 |
| work-05 | resident_crb_init/accumulate/matrix | de3723c |
| work-06 | rigid_com_position 探针 | 0a24297 |
| work-07 | attached_kinematics 探针 | 14e437d |
| work-08 | com_velocity 探针与常驻 | 0dd1f42 |
| work-09 | resident_sparse_validate/gather | 9716848 |
| work-10 | gravity_compensation clear/validate/compute/sum | dd07665 |
| work-11 | 常驻质心位置 FIELD_BATCH | 18b653f |
| work-12 | 常驻附着与初始化附着 | c9b1689 |
| work-13 | 常驻固定肌腱与相机光源 | 6c998ff |
| work-14 | 常驻空间肌腱三项 | 8f903c5 |
| work-15 | 常驻肌腱合并/索引/唤醒 | 1551c59 |
| work-16 | 常驻柔体失效/位置 | 77e8b55 |
| work-17 | 常驻柔体边/面 | 068fd14 |
| work-18 | 常驻柔体 Hessian 缓存/发布 | 94696c9 |
| work-19 | tendon_passive validate/clear/compute | 4376672 |
| work-20 | flex_edge_passive validate/clear/compute | 0dc31dd |
| work-21 | joint_passive validate/compute/sum | 7c4c946 |
| work-22 | passive_force clear/sum/clear_gravity/add_fluid + 重力合并跨度修复 | 327d255 |
| work-23 | fluid_force validate/body/project/clear | 1cef741 |
| work-24 | sparse_solve validate/fused | 38a4960 |

每批补丁：`work-01-*.json` 起至 `work-24-sparse-solve.json`。

## 验证

- 每批双配置 `cargo check`（默认与 `cuda-probe`）、`cargo fmt`、
  双配置 `cargo clippy --all-targets -- -D warnings`、`cargo test --locked --lib`（256 过 0 败）。
- 每批新增宿主故障测试 3–5 项（长度/维度、元数据坏值、容量/缺证明/别名/错会话）。
- GPU 定向测试日志：`runs/work-01-gpu-tests.log` 至 `runs/work-24-gpu-tests.log`；
  每次 NVRTC 使用全新空捕获目录，规避 `MJWARP_PHYSICS_PTX_DIR` 双语义。
  设备：RTX 4070 Ti SUPER，驱动 596.36，CUDA 12.8.1。
- 关键覆盖：刚体运动学、常驻全链、柔性位置/边/面/Hessian、固定与空间肌腱、
  相机光源、质心与空间速度、重力补偿、质量矩阵/分解/稀疏、肌腱/关节/柔体边/共享/流体被动力、
  常驻稀疏求解。

## 已知回归的定位与修复

work-10 迁移重力补偿后，共享被动力链两条 GPU 测试出现
`gravity_compensation_abi InvalidDimension`（work-19/20/21 日志中的红项）。

- 根因：冻结内核 `gravity_compensation_sum` 读取分片缓冲的两个字段
  （`parts[4+tid]` 与 `parts[4+worlds*nv+tid]`），原包装把分片槽当单字段，
  且要求输出恰好为单字段；共享链传入的是四字段平滑力缓冲。
- 插桩证据：`Capacity { slot: 1, required: 164, actual: 296 }`。
- 修复（work-22）：分片槽按 `8+2*worlds*nv` 校对，输出按最小 `8+worlds*nv` 校对；
  宿主测试同步校正跨度断言并新增宽输出/窄分片用例。
- 复测：`passive_force` 共享链 4 项 GPU 测试全绿（`runs/work-22-gpu-tests.log`）。

## 未验收边界与债务

- G01 全量门禁属既有债务，不在本任务内：重力位证书 2 bit / 4 ulp 差异由 f32 输入舍入导致
  （力臂 32/7 的 f32 表示偏 -2.7e-7），证书用有理精确值；基线 `360718f` 同样失败，
  max_abs 1.19e-7 ≤ 2e-6。审计结论见 `docs/development-evidence/g01-numerics-audit/`。
- PTX 捕获目录双语义属既有债务，定向测试用独立空目录规避。
- 开放项：锚点分层、可见性、fixtures 契约。
- `excluded_non_kernel_helpers` 4 项为非内核辅助（不计入内核调用口径）。
- 1 个动态编译站点已解析（`unresolved_dynamic_compile_sites = 0`）。
- 本任务只覆盖 M3 的安全包装同步迁移；不宣称完整上游等价、性能、Linux 或 release 验收。
