# NP5-02 复核（QA）：NP01 精度偏离审计（608 站点全路径聚合；零 GPU）

- 复核对象：卡 `2026-11-11-np5-02-np01-precision-deviation-audit`；内容提交 **`ac840ff7`**。
- 性质：只读复核（零 GPU、零窗）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，0 一般 + 2 低 + 1 观察）**。

## 1. 审计对象与直方图（独立复算）

- 站点口径：`np5_01a_sites=608`、`np5_01a_unique_expressions=598`。我从 11 片 evidence 逐片重算
  landing 直方图 = `{n/a 320, identical 262, engine-semantic 20, recipe-difference 2, pin-needed 1, not-decidable 3}`
  = **608**，与 README §1、receipt 一致。
- `type_class` 逐类重算：f32 510、`int-encoded-f32` 10、bool 15 + bool→f32 4 + bool/int 1 = 20、
  int 41、n/a 16、n/a-view 7、f32 编码 2、int→f32 编码 1、f64-to-f32 1 = **608**；
  分类里**无裸 f64/f16/tf32**，唯一 f64 相关是 `f64-to-f32` 编码 1 条 + `double_in_product` parity 说明。
- `double_in_product` **恰好 1 条**（`field_batch` wrap-pulley `1.0/wrap_prm`），带
  `double_precision_parity`（上游 numpy float64→f32 同路径，非产品单方面升精度）——与 NP5-01a C-4 复核一致。
- 33 张类型表（30 field + 3 host）与 receipt `type_tables` 一致；输入 evidence sha/bytes 我复算全对。

## 2. 类型偏离 0 项（不以数值反推）

- `type_deviation.count=0`，basis 4 项：608 站点 type_mapping、33 类型表、CUDA token 门禁
  （`cuda-surface.json` + ratchet）、host source 精度 token 白名单——**依据是类型表/门禁，不是数值反推**。
- `explicitly_not_inferred_from` 明写「数值完全相等 / ULP 计数」不得用于推类型偏离；README §2 有同义表述。
  这与本团队既有裁定（recipe 差可由融合产生，不能反推类型问题）一致。

## 3. recipe 差 / 引擎语义差 / 实现残差（分列）

- recipe 差：B1 12 条表达式 `--fmad=true` 全 `fma.rn`、`mul.rn/add.rn` 归零；`--fmad=false` 与 host
  两次舍入对应；B2 5 case×3 字段量化 `max_abs` 1e-9–8e-7，明确「用 max_abs 判读、不用 ULP（近零失真）」。
- 引擎语义差：source 级 **20 站点**（A-1a 1 / A-1b 3 / A-2 4 / A-3 3 / B-1 6 / C-1 2 / C-3 1）我按分片
  evidence 复算一致；RE-3 设备级 15 行 204 站点单列；A0 引用位——**不进类型结论**。
- 实现残差：B2 4 case（hfield 两例位等；sdf/ellipsoid 7.45e-9/3.80e-7）+ 1 条 known deviation
  （`witness/cylinder-box-sweep`，已登记 G6 后处理卡）——归实现/求和顺序差，不归 recipe/类型。

## 4. 未完成声明 / manifest / 门禁 / 边界

- **强制未完成句在位**：`precision_deviation_zero_items_verified=false`、`「NP01 精度偏离 0 项未完成」`
  同时出现在 evidence 与 README §5/§6；README §6 明写「不宣言 NP01 通过」；receipt 另声明
  `PASS` 仅指审计执行完成（selfcheck 有对应断言）。
- manifest 4/4（555 B，sha/bytes 全对）；selfcheck 全 PASS；门禁 fmt/clippy/clippy-cuda exit 0、
  unit **1,406 passed / 0 failed / 402 ignored**；policy_scan 对 README/evidence/receipt 3 文件 **0 命中**；
  编码 UTF-8/LF 无 BOM、CR、NUL；边界仅 `np5-02` 与 `.normify`（自检记录的 QA 复核目录属 QA 证据）；无覆盖率/通过率数字。
- 哈希：README `15a247c7…`/5,483、evidence `6526ca13…`/32,689、receipt `98b05ef7…`/10,211、
  manifest `8aaeedf8…`/555、selfcheck `0a71e377…`/1,356。

## 5. 发现

### 低
- **L1 站点枚举与「3 站点」不符（README §3）**：原文「最终口径 recipe-difference **3 站点**（A-1a `acc:3`、
  A-1b `acc:5`、RE-1 的 `acc:5` …）」——`A-1b acc:5` 与 `RE-1 acc:5` 是**同一站点**
  `mass_solve:rigid_mass_solve:accumulation:5` 的两层分类（source 级 not-decidable → device 级由 RE-1 定
  recipe-difference），而 A-1a 的 3 站点实为 `sparse_mass_diag:accumulation:2`、`small_prefix…:accumulation:1`
  （landing 2 条）+ `…:accumulation:3`（R2 改判）。建议分层写并列出 id：
  「A-1a source 级 3 站点（列 id）；A-1b `acc:5` 由 RE-1 在 device 级定 recipe-difference（同站点两层）」。
  evidence 内各站点数据本身正确，属文档表述问题。
- **L2 节号交叉引用错（README §1）**：landing 直方图后注「（最终口径另见 §4）」，但最终口径变化的
  recipe-difference/pin-needed 在 **§3**（recipe 差），§4 是引擎语义差与实现残差。建议改为「另见 §3」
  或直接在 §1 给出最终口径（并注明与 §3 的关系）。

### 观察
- **O1 `type_class` 分组表述不精确（正向记录）**：README §1 把 `n/a-view 7 + f32 编码 2 + int→f32 编码 1 +
  f64-to-f32 1`（=11）并入「其它 f32 派生」；其中 `n/a-view`、`f64-to-f32` 并非「f32 派生」。
  总数 608 正确、无统计错误；建议按 11 个 type_class 原样列出（或不合并），便于逐类对账。

## 6. 非声明

本复核对 NP5-02 的聚合数字/四类分列/未完成声明/门禁与卫生负责；**不宣称** NP01 通过、精度偏离 0 项完成、
E 指标完成、上游等价；不宣称位级等价；不宣称双平台等价；不改判据/参考/容差/相机/recipe。
