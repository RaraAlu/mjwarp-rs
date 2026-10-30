# 回滚 U1b host 解锁（U4 两次 INVALID_ROUND，红线触发）

任务：`2026-10-31-render-sdf-host-unlock-rollback`（Arch；P8；离线，零 GPU）
前置：`2026-10-31-render-sdf-host-unlock`（U1b，回滚对象）
保留：U1 `2026-10-30-render-sdf-mesh-path`（设备侧 `.cu` 改动不回滚）

## 1. 为什么回滚

U4 验证窗 attempt1 与 attempt2 都在新实体 `render_sdf_reference` 的 **sphere 轮廓 depth** 上
复现同一量级超判据：

- `cam0` 215 / 76,800 像素不符、**max_ulp = 191**；`cam1` 4 / 43,200、max_ulp = 25；
- 首差像素 (230,69)，参考 seg = (2,5) = sphere（轮廓边缘，邻行是 plane）；
- 同两组场景 RW：**RGB `mismatched=0`/`max_byte_diff=0`/`beyond_1_lsb=0`，seg `mismatched=0`**；
- attempt2 用**全新冻结 + 只读诊断**后完全复现（诊断：**双向偏差**——前 40 例 product>ref 16 例 /
  product<ref 24 例、ULP 7–71，全量最大绝对值 191 ULP；seg 恒 (2,5)）。

按 Chief/QA 红线：**U4 未通过** → 回滚 U1b、保留 U1；不缩相机、不跳球、不抬 `MAX_DEPTH_ULP`、
不把「诊断已定位」当通过。窗口 attempt1/attempt2 记录原样保留在
`target/render-bvh/sdf-validate-window/attempts/{attempt1,attempt2}/`（guards 字节副本、monitor CSV、
entity-results、9 实体日志、facts、invalid-round-record）。

## 2. 回滚内容

| 项 | 值 |
| --- | --- |
| 源码 diff | 只 `src/render/kernel/megakernel.rs`（U1b 的逆操作）：406 行/`9b013d90…` → **440 行/`fb9aae8f82eb9a3891f8c43879d2f79c894f9eeb4cd15da468004d746999e380`** |
| 恢复 | `reject_sdf_geometry`（line 55）、`render()` 调用点（line 121）、`#[cfg(test)] mod sdf_geometry_guard_tests`（line 422，2 条 host-only 单测） |
| 能力 | host 对 `geom_type==8` 重新返回 `Unsupported（S7b3 不含 SDF 求交）` ⇒ SDF 能力回到**不可达** |
| U1 保留 | `git diff --stat -- src/render/kernel/cuda` **为空**；内核身份仍为 U1 值 source `778216b2…`/PTX `34f91147…`/268,589 |
| 零越界 | 不动判据/容差/参考/`fixtures/**`/其它 `src/**`/`tests/**`/`scripts/**`；U4 实现片/材料证据不改 |

## 3. 门禁与计数对账

fmt / clippy / clippy-cuda / unit 全绿；`cargo test --locked --no-run` EXIT=0 含 `Finished`；
`validate --fingerprints` 错误 0。

**unit 1111 过 / 0 败 / 397 ignored**：1109（回滚前：1106 于 U1b 后 + U4 实现片 3 条 host 测试）
+ 2 条恢复的 host-only 单测 = **1111**。卡面 acceptance 里写的 1108 是笔误（按 1106 基数算漏了 U4 的 +3），
以实测 1111 为准并在 `materials/identity.txt` 留痕。

## 4. 后续

sphere 轮廓 depth 的 RCA（先读 pinned `ray.py::ray_sphere`/`_ray_quad` 与产品 `render_math.cu`/`render_common.cu`
的 f32 运算顺序、边/顶点 tie-break、`|dir|` 口径）→ 独立修复片（参考 R05 相机 FMA 片）→ fix 后
U4 attempt=3（再次全新冻结，92 条身份再换）。本片不跑 GPU、不改判据。

## 5. 证据清单

| 文件 | 内容 |
| --- | --- |
| `README.md` | 本文 |
| `receipt.json` | 任务 id、回滚前后身份、unit 对账、attempt 引用、墙线声明 |
| `manifest.txt` | 本目录除自身外全部文件 sha256/bytes |
| `logs/gate-*.txt` | 四门禁转写快照（权威 = `.normify/runs/`） |
| `logs/no-run.txt` | `--no-run` 原样日志（EXIT=0、Finished） |
| `materials/restore-diff.patch` | 回滚 diff（U1b 逆操作） |
| `materials/identity.txt` | before/after 身份、恢复要素、`.cu` 零改动、unit 对账 |
| `materials/attempt-refs.txt` | attempt1/attempt2 失败留档路径、复现数字、verdict 与后续 |
