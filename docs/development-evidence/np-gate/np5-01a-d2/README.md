# NP5-01a D-2：ccd_host 37 站点 + 2 附录单元 host 路径 source 级对照（零 GPU）

- 任务：`2026-11-11-np5-01a-d2-ccd-host-source-level`（owner Backend）。
- 依据：`np5-01a-sites.json` 的 ccd_host 37 站点 + `rows_without_table` 的 2 个附录单元
  （`src/math/fma.rs`、`render_light_dir_normalize`）；词表 `np5-01a-vocab-addendum.md`
  （`453c5c91…`/6,771）。
- 边界：source-level、零 GPU、零窗；不改 `src/`/`tests/`/`fixtures/`/`scripts/`/判据/参考/容差。
- 累计：NP5-01a 场表站点 **571 + 37 = 608**（= A179 + B78 + C226 + D125）；
  **唯一表达式 598**（608 − 10 重复 tail，见 §4）。D 批 125 = D-1 88 + D-2 37 全收口。

## 1. D-1 复核 L1 处置（第一节）

- **L1（88 条补 `kind`/`label`，不抽样）**：`np-gate/np5-01a-d1/rows-fields-v1.json`
  （`row_count: 88`）按 evidence 顺序补齐 `id`/`kind`/`label`/`classification`，
  `kind`/`label` 与 `sites.json` 的 gjk_host/epa_host 两行并集**逐字一致**（构建时校验，
  无政策禁词条目、无需 redaction）；`addenda.md` 说明补齐方式；**D-1 原 evidence 字节不改**
  （`2d509468…`/115,577），D-1 manifest 重算为 6 文件。
- **D-2 自身条目原生带 `kind`/`label`**（不再靠后续补页），构建时与 sites.json 逐条校验。

## 2. 站点与分类计数（37 站点 + 2 附录单元）

| 组 | 站点 | 切分（intermediate / accumulation / output / rounding-note） |
| --- | --- | --- |
| `ccd_host.gjk_phase_phase.rs` | 10 | 4 / 3 / 3 / 0 |
| `ccd_host.ccd_internal_ccd.rs` | 5 | 2 / 0 / 3 / 0 |
| `ccd_host.epa_phase_with_horizon_ccd.rs` | 7 | 3 / 0 / 4 / 0 |
| `ccd_host.ccd_pair_contact.rs 装配` | 11 | 4 / 3 / 4 / 0 |
| `ccd_host.rounding-note` | 4 | 0 / 0 / 0 / 4 |

分类计数：**`identical` 16 / `n/a` 21**（无 engine-semantic/not-decidable 站点；
无覆盖率/通过率表述、不声明位级）。

- `identical` 16：`inflate` 的 `dist -= margin1+margin2` 先加后减、HFIELD 重心坐标两次三项乘加、
  `dist=-length3(sub3(x1,x2))`、phase/EPA 输出字段与判据（`needs_epa`、失败 `FLOAT_MAX`）、
  ccd 装配 `margin` 修正/中点 `0.5*(w1+w2)+origin`/`margin+gap` 严格小于判据/`dist`/`pos`/`frame` 输出 ——
  **source 级表达式/类型/求值次序对应，不是位级结论**。
- `n/a` 21：快照/中间量（`orig_margin`/`cutoff`/`inflate` 法线）、整型索引与状态位
  （`multiccd_idx`/计数/溢出位）、结构输出（`Polytope`/`GjkPhase`/`EpaPhase`/`CcdOutput`）、
  控制流布尔量（`is_discrete`）、平移基准/阈值选择（`origin`/`cutoff`）、4 条 rounding-note。
- **D 批 device 级**：host 路径无 PTX，37 站点整体记 not-decidable 原因 `host-path-no-ptx`
  （不逐站点重复）；本片只做 source 级对照。

### 2.1 附录单元（2 个，非站点；原生带 kind/label）

| 单元 | kind/label | 分类 | 说明 |
| --- | --- | --- | --- |
| `src/math/fma.rs` | `appendix-unit` / `fma_f32`（IEEE-754 f32 精确 FMA：硬件 FMA 主路 + 整数精确模拟兜底） | `n/a` | 产品内部数学助手，无上游源码表达式映射（上游对应编译器 `fma.rn` 语义）；语义由产品单元测试（1e6 用例 + 77 例边界集）覆盖；host 路径不可 PTX 对照 |
| `render_light_dir_normalize` | `appendix-unit` / 光方向归一化（warp normalize 的 `fma(z,z,fma(y,y,x*x))` 序列 + 逐分量除法） | `identical` | 产品 `light_data_poses` 显式用精确 f32 FMA 复刻 warp normalize 收缩序 + 逐分量除法（length>0，否则零向量），与 `smooth.py:951 wp.normalize(light_xdir)` 同序；source 级、不声明位级 |

## 3. 方法与判据

- 产品锚点（repo 行号）：`src/physics/collision/gjk/phase.rs`（`inflate`/`gjk_phase`）、
  `ccd.rs`（`epa_phase_with_horizon`/`ccd_internal`/`ccd`）、`contact.rs`（`ccd_pair` 装配）；
  附录单元 `src/math/fma.rs:75`、`src/render/kernel/megakernel_arrays.rs:228`。
- 参考锚点（`collision_gjk.py`，冻结 71da24d9）：`gjk_phase` 2502–2572、`_inflate` 2455–2501、
  `epa_phase` 2573–2682、`ccd` 2683–2729、`eval_ccd_write_contact` 769–1026、
  `multicontact` 2216–2454；附录单元参考 `smooth.py:951`。
- **host recipe 口径**：host 站点刻意非融合（独立乘、独立加、两次舍入；不用 `mul_add`、不引 f64）；
  device 侧 pinned ccd PTX 为 `fma.rn` 融合语义（另一 recipe）——只做 source 级判定。
- `identical` 的 source 级含义：产品与上游在同一表达式上写法/类型/求值次序可对应（f32 全链、无升精度）；
  `n/a`：非算术点（缓冲/整型/结构/说明）。

## 4. 双口径（与收尾 addenda 同一来源）

- **608 = 场表行×站点口径**：A179 + B78 + C226 + D125（D125 = D-1 88 + D-2 37）。
- **598 = 去重唯一表达式**：`rigid_com_position` 内核同时登记于 kinematics 行（A-2，11 站点）与
  com_position 行（A-3，14 站点），按 (kernel,kind,index) 去行前缀有 **10 个 tail 重复**
  （acc:0/1/2、int:0–3、out:0–2）→ 608 − 10 = 598。
- 报告中不得把 608 读成 608 个唯一表达式；两口径并列见 `evidence.json.dual_count_note`。

## 5. 输入与哈希

- sites.json `e0d4a95bf5c0acf13510b20f51e92ece1ec63ab7c7ac9e37be38cdd0841bac3c`/261,483（工作材料）。
- 词表 addendum `453c5c9157341fa28ecb009c50f2ff560b6f43e15f593ed9b7a75f27dd5b1b26`/6,771（工作材料）。
- upstream revision `71da24d956378a87a703b6e1442b13aec0c4ac29`；tree.head `2d5a6610…`（本卡主提交父提交）。
- D-1 原 evidence `2d509468f20202a632eba85890fb21efc13a67d92d9a78ea5d0dacc55a5d7c6c`/115,577（未改）。

## 6. 非声明

不宣称 NP01 通过/E 指标完成/上游等价/双平台/位级保证；不改判据/参考/容差/相机/recipe；
零 GPU、零窗；`identical` 仅为 source 级表达式/类型/次序对应。
