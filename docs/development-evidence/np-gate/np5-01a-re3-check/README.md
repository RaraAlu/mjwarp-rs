# NP5-01a RE-3 复核（QA）：B 批全量 78 + C 批可映射子集 155/226 设备级行映射

- 复核对象：卡 `2026-11-11-np5-01a-re3-tendon-and-c-mappable-device-level`；内容提交 **`01d4c979`**（登记 `69732998`）。
- 性质：只读复核（零 GPU、零窗）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，0 一般 + 1 低 + 1 观察）**。

## 1. 产品侧 24 单元（我逐文件核对）

- **24 个单元**：22 个有 `__global__` entry；`tendon_wrap` 与 `field_batch` 各 **202 B / 0 entry**（两者空 PTX 同 sha `fc056e0f…`，与「源码无 `__global__`」一致；field_batch 为 host 打包、tendon_wrap 的 device 函数被 spatial 内核内联）。
- 我在 `target/.../RE-3-draft/ptx/` **逐文件核对 24/24 单元的 sha256/bytes 与 `evidence.json.product_units.units` 完全一致**；单元 entry 合计 54。
- 编译口径与 README 一致（产品真实 launch unit、pinned NVRTC 12.8.93 / compute_70 / `--fmad=true` / `-lineinfo`；合成尺寸实例只影响立即数、不参与分类，`synthetic_instance_note` 在位）。

## 2. 参考侧 40 映射对（去 hash 匹配）

- **40 个映射对**（24 行合计；distinct 参考 entry **28**）全部命中冻结 `smooth.ptx`(48 entries) + `module.ptx`(15 entries) 的 **entry 并集(63)**，按 `_<name>_[0-9a-f]{8}_cuda_kernel_forward` 去 hash 名精确匹配，**无错配、无缺失**。
- 冻结件身份：`smooth.ptx` = 993,601 B/`4179acd0…`/48 entries、`module.ptx` = 754,389 B/`d06154da…`/15 entries，与 evidence/receipt 记录一致。
- **lineinfo 前提 ✅**：我按 `^\.loc` 指令行核，两份冻结件真 `.loc` 数 **均为 0**（文中 6/4 处「loc」是 `.local` 内存段，非 lineinfo）——`no-lineinfo` 理由成立。

## 3. 行级分类（与 evidence 独立复算一致）

- 24 行 = B 7 + C 17；站点 304 = B 78 + C 226。
- 分类：`engine-semantic` **15 行/204 站点**、`not-decidable` **7 行/73 站点**、`n/a` **2 行/27 站点**，24 行 304 站点自洽。
- **not-decidable 缺因（实测）**：`no-reference-entry` 4 行 **47** 站点（tendon_velocity 7 / applied_force 12 / kinetic_energy 13 / sleep_wake 15）；`no-lineinfo`+`frozen-artifact-only` 3 行 **26** 站点（tendon_wrap 9 / flex_edge 9 / flex_face 8）；合计 73。
- **n/a 2 行**：`tendon`（整数寻址/values-indices 搬运，产品 key_ops 无浮点算术）、`field_batch`（host 打包、0-entry 单元），各有 `basis` 说明。
- **source 级不重判 ✅**：`totals.source_level_untouched`、receipt `source_level_untouched` 与 README §4 均明写 B-1/C1..C4 逐站点分类不变，device 行结论只属映射层（如 flex_edge/flex_face 行级 `not-decidable` 不推翻 source 级分类）。

## 4. 证据/门禁/边界

- PTX 实体不入仓（`ptx_landed=false` 两处），regen 脚本在 target；`evidence.json` 只记单元 sha/bytes/entry 名单。
- manifest **4/4**（568 B，我独立重算 sha+bytes，集合相等）；selfcheck **30 项 PASS**；门禁 fmt/clippy/clippy-cuda exit 0、unit **1,406/0/402**。
- 证据哈希与派工值一致：README `e7de0e55…`/8,585、evidence `f5988b7c…`/94,815、receipt `28f4bd21…`/4,782、selfcheck `d23a209d…`/1,769、manifest `fb651440…`/568。
- 边界：提交仅 `docs/development-evidence/np-gate/np5-01a-re3/**` + `.normify/**`；编码 UTF-8/LF 无 BOM/CR/NUL；`policy_scan` 0 命中；无覆盖率/通过率表述。

## 5. 发现

### 低
- **L1 行级 schema 缺分类附加字段**：`rows[]` 只有 `basis`（文字理由）+ `device_observation`，**没有** RE-2 先例的 `why_engine_semantic`、`bit_level_guarantee:false`、`why_not_applicable` 机器可读字段；`bit_level_guarantee=false` 只在 `method_rules` 文本里。派工复核要点④要求 engine-semantic 行带 `why`+`bit_level_guarantee:false`。建议后续片/addendum 给 24 行补这三个逐行字段（或给出 basis→字段的映射表），便于机器校验。

### 观察
- **O1 派工摘要的缺因拆分与 evidence 不一致**：摘要写「`no-reference-entry` 4 行 **40** 站点；`no-lineinfo`+`frozen-artifact-only` 3 行 **33** 站点」，evidence 实际为 **47 / 26**（总数 73 一致）。evidence 自洽、无需改数据；建议更正摘要措辞以免误传。

## 6. 非声明

本复核对 RE-3 行级映射层、产品单元身份、参考映射与卫生负责；**不宣称** NP01 通过/精度完成/位级等价/上游等价/双平台，不改判据/参考/容差。
