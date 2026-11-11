# NP5-01a D-2 复核（QA）：D-1 L1 处置 + ccd_host 37 站点与 2 附录单元 source 级

- 复核对象：卡 `2026-11-11-np5-01a-d2-ccd-host-source-level`；内容提交 **`06811c89`**（登记 `70e790fd`）。
- 性质：只读复核（零 GPU、零窗）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，0 一般 + 0 低 + 2 观察）**。

## 1. 第一节：D-1 的 L1（缺 kind/label）

- **88 行字段补齐 ✅**：`np5-01a-d1/rows-fields-v1.json`（18,429 B、`row_count=88`）按 evidence 顺序补齐 `id/kind/label/classification`；我逐条与 `sites.json` 的 gjk_host/epa_host 两行并集比对：**88/88 kind/label 逐字一致、0 问题**；本批无命中政策禁词的条目，**无需 redaction**。
- **原字节/manifest ✅**：D-1 `evidence.json`/`README.md`/`receipt.json`/`logs/selfcheck.txt` **字节未改**（evidence 仍 `2d509468…`/115,577，与 `source_evidence` 一致）；D-1 manifest 重算 **6/6**（733 B）全对；`addenda.md` 说明补齐方式与原字节不变。
- **D-2 自身 ✅**：D-2 的 37 条条目**原生带 `kind`/`label` 37/37**（selfcheck 与我的核对均与 sites.json 逐字一致），无需再补页。

## 2. 第二节：D-2 ccd_host 37 站点 + 2 附录单元

- **站点集合 ✅**：37/37，与 `sites.json` 的 `ccd_host` 行**逐 id 相等**；`kind_counts` 13/6/14/4；直方图 `{identical 16, n/a 21}`。
- **双口径/总数 ✅**：`np5_01a_total_sites=608`、`np5_01a_unique_expressions=598`、`prior=571`（A40+B36+…+D1 88 清单在位）；`dual_count_note` 明确「608 = 行×站点（A179+B78+C226+D125）；598 = 608−10（rigid_com_position 在 kinematics/com_position 两行重复登记 10 个 tail）」——D-2 evidence 与 receipt 均在位。
- **附录单元 ✅**：`appendix_unit_count=2`，两个附录单元（`kind: appendix-unit`）均带 product/reference 锚点、type_mapping、premise、evidence_pointer：
  - `src/math/fma.rs` → `n/a`：内部数学助手（IEEE-754 单次舍入 FMA 的 host 实现），无上游源码表达式对应，host 无 PTX；`why_not_applicable` 齐全；reference 锚点为**已声明的语义指针**（warp `fma.rn` 语义，无源码函数）。
  - `render_light_dir_normalize` → `identical`：产品显式复刻 warp normalize 的 `fma(z,z,fma(y,y,x*x))` + 逐分量除法，参照 `smooth.py:951 wp.normalize`；带 `why_identical`（f32 精确 FMA、无升精度）、**明写不声明位级**。
- **host 口径 ✅**：`host_path_note` 明确 37 站点全部无 PTX、device 级整体记 `not-decidable` 原因 `host-path-no-ptx`；本片只做 source 级。
- **锚点/证据 ✅**：37 条站点锚点 + 2 附录锚点全解析、**0 错误**；policy_scan **0 命中**；manifest **4/4**（556 B）；selfcheck 31 项全 PASS；门禁 1,406/0/402；哈希：README `146723a5…`/5,967、evidence `f7fc0c6c…`/57,149、receipt `b42053a5…`/4,191、manifest `f629c8d3…`/556、selfcheck `94835a34…`/1,636；UTF-8/LF 无 BOM/CR/NUL；无覆盖率。

## 3. 发现

### 观察
- **O1 字段补页一致性（正向）**：A-3 与 D-1 均用 `rows-fields-v1.json` + addenda 补 kind/label（原字节不改），D-2 条目原生带字段——三种落法口径一致，后续收尾无需再补。
- **O2 附录单元登记方式合规**：`src/math/fma.rs` 的 reference_anchor 是**声明过的语义指针**（无上游源码函数），`render_light_dir_normalize` 的 `identical` 仅 source 级且明确不声明位级；两个附录单元都不计入 608 站点（`appendix_unit_count=2` 单列），与「608 = 场表行×站点」口径一致。

## 4. 非声明

本复核对 D-1 字段补页与 D-2 站点/附录单元/双口径/卫生负责；**不宣称** NP01 通过/精度完成/位级等价/上游等价/双平台，不改判据/参考/容差。
