# NP5-01a C-2 复核（QA）：C-1 低项闭环 + C 批 force/energy 68 站点 source 级

- 复核对象：卡 `2026-11-11-np5-01a-c2-force-energy-source-level`；内容提交 **`215693d3`**（登记 `ac77dcd5`）。
- 性质：只读复核（零 GPU、零窗）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，0 一般 + 0 低 + 2 观察）**。

## 1. 第一节：C-1 的 1 低 + 2 观察闭环

- **① 英文否定声明 ✅**：`policy_scan.py` = **13,482 B / `9a63e720…`**；`--self-test` **28/28 PASS**。我构造实测（块内声明 + 远距 filler + 位级等价）：
  - **仍不豁免（hit=1）**：`never supersede`、`no withdrawal`、`without withdrawal`、`none superseded`、`must not supersede`、`not withdrawn`、`no 作废` —— 英文否定词与中文否定混排均生效；
  - **仍豁免（hit=0）**：`撤回声明`、`supersedes:`、`superseded by ...`、`作废声明` —— 肯定式声明未被误伤，词边界正确。
- **② `reviewed` 语义 ✅**：工具 docstring 写明「inventory 的 `reviewed` 只表示块内存在 `reviewed_by` 标记；`false` 不等于未复核（痕迹可在 evidence receipt 的 `quote_block_inventory`）」，C-2 README 同步。
- **③ `errata-0001` 记录 ✅**：`flex_hessian_validate` 的 field-table 描述滞后（表写 `ok` bool/`status` f32，产品现写 i32 `flags`）已记入 C-2 README、`evidence.json.o2_errata_pending`（id `errata-0001`、status recorded）与 receipt `errata_pending`，**field-table 历史未改**，待收尾 addenda 统一。

## 2. 第二节：C-2 force/energy 68 站点

- **站点集合 ✅**：68/68，与 `sites.json` 的 5 行并集**逐 id 相等**（fluid_force 27 / kinetic_energy 13 / applied_force 12 / smooth_force 9 / passive_force 7），无缺/无多/无重复。
- **直方图 ✅**：`{identical 37, n/a 31}` = 68；README 明示 `engine-semantic 0 / recipe-difference 0 / not-decidable 0 / pin-needed 0`；**累计 337**（269+68，交集为空）。
- **实现依赖表述 ✅**：流体盒分支 `powf(diam,3.0f)`/`powf(box*,4.0f)` 的位级随 NVRTC 版本，README §2 与 `fluid_force:rounding-note:3`、`output:2` 的 notes 均写明「实现依赖项、**不宣称其位级**」；`rounding-note:0` 说明普通 `* / + - sqrtf powf` 未写显式 `.rn`、由 JIT 决定，不上位级结论。
- **锚点 ✅**：我全量解析 **0 错误**（53 条 file:line + 15 条 `no-reference-expression`，后者均 n/a 带 reason）；抽样行内容正确：`fluid_force/cuda.rs:47/48` ↔ `passive.py:409`（`lin_com = lin_global - wp.cross(...)`）、`validate` 的 `cuda.rs:26/34`。
- **必填字段 ✅**：68/68；`policy_scan` 对 C-2 evidence/README/receipt **0 命中**；manifest **4/4**（539 B）；selfcheck 16 项全 PASS；门禁 1,406/0/402；UTF-8/LF 无 BOM/CR/NUL；无覆盖率/通过率表述。
- **证据哈希与派工值一致 ✅**：evidence `1d0aab16…`/98,071、README `619525b1…`/6,444、receipt `5a67d09b…`/4,903、manifest `391d1238…`/539、selfcheck `418826ad…`/1,092。

## 3. 发现

### 观察
- **O1 声明词表的英文覆盖**：声明词仅 `撤回`/`supersede`/`作废`；英文过去分词 `withdrawn:` 不属声明词，因此含它的块会**照常命中**（我实测 hit=1）。方向保守、非缺陷；建议补 `withdraw`/`withdrawn` 做对称，或在 docstring 明确声明词表（避免作者误以为会被豁免）。
- **O2 C-2 无 engine-semantic**：`powf` 等实现依赖以 notes/rounding-note 记录、README 已明示；`errata-0001` 待收尾 addenda 统一入 errata 清单，本卡确认记录完备。

## 4. 非声明

本复核对 C-1 低项闭环与 C-2 站点/分类/锚点/实现依赖表述/卫生负责；**不宣称** NP01 通过/精度完成/位级等价/上游等价/双平台，不改判据/参考/容差。
