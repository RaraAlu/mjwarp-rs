# NP5-01a D-1 复核（QA）：A-3 L1/O1/O2 处置 + D 批 gjk/epa host 88 站点 source 级

- 复核对象：卡 `2026-11-11-np5-01a-d1-gjk-epa-host-source-level`；内容提交 **`32fd81b8`**（登记 `e24091fb`）。
- 性质：只读复核（零 GPU、零窗）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，0 一般 + 1 低 + 2 观察）**。

## 1. 第一节：A-3 的 L1/O1/O2

- **L1 kind/label 补齐 ✅**：`np5-01a-a3/rows-fields-v1.json`（8,169 B / `3cb3ba73…`、`row_count=40`）按 evidence 顺序补齐 `id/kind/label/classification`；我逐条与 `sites.json` 三行并集比对：**39 条 kind/label 逐字一致**；**1 条 policy-redacted**（`com_position:rigid_com_position:rounding-note:0` —— sites.json 原文 label 含历史正面措辞），其 `label` 记为 `policy-redacted` 并指回 `sites.json` 原文位置，**未把禁词复制进证据**，分类/kind 信息不丢 —— 处理方式合规（policy_scan 对新文件 0 命中）。
- **原字节/manifest ✅**：A-3 `evidence.json`/`README.md`/`receipt.json`/`logs/selfcheck.txt` **字节未改**（evidence 仍 `c9a9676f…`/53,532，与 `source_evidence` 对应）；A-3 manifest 重算 **6/6**（714 B）全对。
- **O1 双口径 ✅**：D-1 receipt `section1.o1_dual_count` = `field_table_sites: 608`、`unique_expressions: 598`、`duplicate_tails: 10`，附来源（`rigid_com_position` 在 kinematics 行 A-2 11 站点 / com_position 行 A-3 14 站点）与**去重方法**（按 (kernel,kind,index) 去行前缀；重复 tail：acc:0/1/2、int:0–3、out:0–2）；README 同步，并声明收尾 addenda 将并列两口径；本片不改任何计数。
- **O2 记录 ✅**：A-3 engine-semantic 措辞沿用 A-2 先例已记入 `np5-01a-a3/addenda.md` §2。

## 2. 第二节：D-1 gjk/epa host 88 站点

- **站点集合 ✅**：88/88，与 `sites.json` 2 行并集**逐 id 相等**（gjk_host 45 / epa_host 43），无缺/无多/无重复。
- **直方图 ✅**：`{identical 37, n/a 51}`；**累计 483+88=571**。
- **host 口径 ✅**：`device_level_note` 明确 **88 站点全部无 PTX**，device 级映射整体记 `not-decidable` 原因 **`host-path-no-ptx`**（不逐站点重复登记），本片只做 source 级；`host_recipe_note` + README 明确 **host 刻意非融合**（独立乘、独立加、两次舍入，不用 `mul_add`、不引 f64），而 device 侧 pinned ccd PTX 为 `fma.rn` 融合语义——**两套 recipe**，本片只做 source 级判定，不声明 device 等价或位级保证。
- **`identical` 语义 ✅（抽核）**：source 级「同表达式/类型/求值次序、f32 全链无升精度」——如 `math.rs:15 dot3`（左结合、两次舍入）↔ `collision_gjk.py:657`（warp CPU 后端非融合展开）、`simplex.rs:298 linear_combine`（i 升序）↔ `collision_gjk.py:292`；premise 均写明 host 无 PTX、不做设备级/位级结论。
- **`n/a` 51 条理由 ✅**：缓冲/整型索引/状态位/结构输出（`GjkResult`/`Polytope`/计数/溢出位）/迁移基准/rounding-note，均有 `why_not_applicable`。
- **锚点/证据 ✅**：88 条锚点全解析、**0 错误**；policy_scan **0 命中**；manifest **4/4**（557 B）；selfcheck 30 项全 PASS；门禁 1,406/0/402；哈希与派工值一致（README `10711306…`/5,769、evidence `2d509468…`/115,577、receipt `0588aed4…`/4,072、manifest `103a930b…`/557、selfcheck `b03513d7…`/1,608）；UTF-8/LF 无 BOM/CR/NUL；无覆盖率。

## 3. 发现

### 低
- **L1 D-1 条目也缺 `kind`/`label`**：D-1 的 88 条条目同样没有这两个字段（selfcheck 的「kind match」由 id 推导），与 A-3 修复前同型。建议收尾前给 D-1 也补一份 rows-fields（或 addendum），使各片字段一致、kind/label 机器可读。

### 观察
- **O1 A-3 policy redaction 处理合规（正向）**：redacted label 保留分类信息、指回 sites.json 原文位置、未复制禁词；禁词仅存在于 `target/` 工作材料 sites.json，不在仓内证据。
- **O2 host recipe 与 `host-path-no-ptx` 口径完整**：88 站点整体记 not-decidable `host-path-no-ptx`（不逐站点重复）、host/device 两套 recipe 分列、`identical` 仅 source 级语义；608/598/10 双口径记录与去重方法到位。

## 4. 非声明

本复核对 A-3 字段补页/policy 处理/双口径记录与 D-1 host 站点/recipe/锚点/卫生负责；**不宣称** NP01 通过/精度完成/位级等价/上游等价/双平台，不改判据/参考/容差。
