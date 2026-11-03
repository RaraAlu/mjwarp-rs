# NP01 逐字段类型表 B1 / B2a 批 —— QA 独立批次复核（只读）

- B1 批实现卡：`2026-11-03-precision-np01-field-tables`（实现提交 **`0d8ee05a`**，5 张表）
- B2a 批实现卡：`2026-11-03-precision-np01-field-tables-b2a`（实现提交 **`e6154d82`**，4 张表）
- 复核卡：`2026-11-04-precision-np01-field-tables-b1-b2a-check`（QA，p60）
- 复核时点：2026-11-04（本地空闲时段）。复核执行时 HEAD 已到后续批次（B2b/B2c），待批计数继续收紧属**预期**；
  本复核对 B1/B2a 的断言按各自提交内容核对。
- 边界：只读；未改产品代码/判据/容差/参考/fixtures/清单；零 GPU（未运行 nvidia-smi，未开窗）。

## 1. 结论

**PASS**（无阻断；表结构与逐字段事实成立、待批收紧自洽、口径与 `raw-kernels.json` 一致；观察 3 条，均属文档/清单同步层）。

## 2. 范围与边界

- `0d8ee05a`：`field-tables/{com_position,com_velocity,gravity_compensation,rne_acc,rne_bias}.json`（新增）、
  `field-tables/README.md`、`tests/precision_type_gate.rs`、卡与 runs；**无** `src/`、`fixtures/`、`include/` 改动。
- `e6154d82`：`field-tables/{fixed_tendon,spatial_tendon,tendon,tendon_velocity}.json`（新增）、
  `field-tables/README.md`、`tests/precision_type_gate.rs`、卡与 runs；**无** 产品路径改动。
- 九张表自各自批次提交后**未被后续批次修改**（`git diff <commit>..HEAD -- <table>` 均为空），零 GPU。

## 3. 表结构与逐字段事实

九张表均 `schema=1`、`row_id` 与 inventory 行 id 一致、`batch` 与批一致（B1 五张 / B2a 四张）；
`batch/layer/audit_level/status/source/upstream/evidence` 非空；`kernels[]` 非空且每 kernel 的
`params/outputs` 非空、`intermediates`/`accumulations` 为数组。`evidence[]` 中路径型条目逐条存在
（命令型条目保留原样）：com_position 3、com_velocity 2、gravity_compensation 3、rne_acc 2、rne_bias 2、
fixed_tendon 2、spatial_tendon 2、tendon 2、tendon_velocity 2，**缺失 0**。

## 4. 待批收紧与覆盖自洽

| 提交 | `PENDING_ROWS` 声明/实际 | inventory rows | 表数量 | 恒等式 |
| --- | --- | --- | --- | --- |
| `0d8ee05a`（B1） | 25 / 25，且五张新表均不在待批 | 30 | 5 | 30 = 5 + 25 |
| `e6154d82`（B2a） | 17 / 17，且四张新表均不在待批 | 30 | 13 | 30 = 13 + 17 |

门禁四类断言（`field_tables_cover_registered_rows_without_silent_gaps`）均真跑：行 id 必须对应表；
7 段 + kernel 段不得空；待批行不得已有表；「既无表又不在待批」为空（不允许静默缺表）。
独立执行 `normify gate <check> fmt clippy clippy-cuda unit` 全过（`unit` = `cargo test --locked`）。

## 5. 口径核对（fma / atomicAdd）

对照 `raw-kernels.json` 文本口径，九张表的相关声明一致：

- `com_position`（`src/physics/mod.rs`，该文件 fma=7 属 `rigid_kinematics` 的 `matrix_inertial` 定义，本表如实写
  「无显式 fma」）✓；`com_velocity`（fma 0 / atomicAdd 0）✓；`rne_bias`（0 / 0）✓。
- `gravity_compensation`（`src/physics/gravity_compensation/cuda.rs` fma 0 / atomicAdd **1**，表内登记
  `atomicAdd` 与 `__fadd_rn` 显式单次舍入）✓。
- `rne_acc`（`src/physics/rne.cu` fma **1** / atomicAdd **1**，表内登记 `fmaf` 序列与 `atomicAdd` 原子加）✓。
- `fixed_tendon` / `spatial_tendon` / `tendon` / `tendon_velocity`（均 0 / 0，表内明确「无显式 fma/无原子」）✓。
- `mass_multiply` 之外无上游升格；B1/B2a 表的上游置信级别沿用 inventory 原值（未升格）✓。

## 6. 观察（不阻断；建议 B-final 一并刷新）

1. **inventory 逐行漂移**：B1/B2a 已交付的表，其 inventory 行仍是旧状态（「部分/待补」、`evidence.missing` 仍列
   「逐字段类型表」、无 `field_table` 指针）。仅 B1b 四行在 B1b 批同步过。门禁不依赖该指针，不影响事实与门禁，
   但清单与 README 不一致；建议 B-final 统一刷新（含 `inventory.summary`，其 `updatedAt=2026-10-23`）。
2. **README 头部计数陈旧**：`field-tables/README.md` 标题仍是「B1 / B1b 批」、头部仍写「B1 + B1b 只完成 9/30」，
   但章节已含 B2a（以及后续 B2b/B2c）完成段。属文档同步问题，不构成通过声明；建议 B-final 更新头部为当前计数。
3. **门禁对 `evidence` 的校验强度 < 验收措辞**：验收写「`evidence[]` 指向的文件存在」，门禁实际只断言非空；
   QA 已独立逐条核过九张表的路径型条目（0 缺失），建议后续将路径存在性硬化成机判或在验收措辞改为「非空 + QA 抽核」。

## 7. 不声明

本复核只证明 B1/B2a 九张表的逐字段事实、待批收紧与门禁口径与各自提交一致。**不声明 NP01 已达验收、
不声明类型映射率达标、不声明与上游一致**；NP01 仍需 30+3 条全部有表且门禁全绿（复核时 23/30，余批次继续）。
