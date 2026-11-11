# NP5-01a D-1：gjk_host 45 + epa_host 43 host 路径 source 级对照（88 站点；零 GPU）

- 任务：`2026-11-11-np5-01a-d1-gjk-epa-host-source-level`（owner Backend）。
- 依据：`np5-01a-sites.json` 的 gjk_host 45 + epa_host 43；词表 `np5-01a-vocab-addendum.md`
  （`453c5c91…`/6,771，六类词表 + not-decidable 六原因）；上游 = 冻结 `71da24d9` 的
  `target/mjwarp-parallel/upstream/mujoco_warp/_src/collision_gjk.py`。
- 边界：source-level、零 GPU、零窗；不改 `src/`/`tests/`/`fixtures/`/`scripts/`/判据/参考/容差。
- 累计：NP5-01a **483 + 88 = 571**（D-1 为 D 批 125 站点的主体 88；D-2 37 + 2 附录随后）。

## 1. A-3 复核 L1/O1/O2 处置（第一节）

- **L1（40 条补 `kind`/`label`）**：`np-gate/np5-01a-a3/rows-fields-v1.json`（`ce9f37d6…`/8,078，
  `row_count: 40`）按 evidence 顺序补齐 `id`/`kind`/`label`/`classification`，`kind`/`label` 与
  `sites.json` 三行并集逐条一致（构建时校验）；`addenda.md` 说明补齐方式；**A-3 原 evidence 字节不改**
  （`c9a9676f…`/53,532），A-3 manifest 重算为 6 文件。
- **O1（双口径记录）**：`rigid_com_position` 内核同时登记于 kinematics 行（A-2，11 站点）与 com_position 行
  （A-3，14 站点），按 (kernel,kind,index) 去行前缀有 **10 个 tail 重复**
  （acc:0/1/2、int:0–3、out:0–2）；故 **608 = 场表行×站点口径**（A179+B78+C226+D125）而
  **去重唯一表达式 = 598（608−10）**。收尾 addenda 将并列两口径并注明来源与去重方法；本片不改变任何计数。
- **O2（engine-semantic 沿用 A-2 先例）**：已在 `np5-01a-a3/addenda.md` §2 记录口径来源，不改变分类。

## 2. 站点与分类计数（88 = 45 + 43；累计 571）

| 组 | 站点 | 切分（intermediate / accumulation / output / rounding-note） |
| --- | --- | --- |
| `gjk_host.gjk_core.rs` | 11 | 5 / 3 / 3 / 0 |
| `gjk_host.gjk_support_core.rs` | 5 | 3 / 1 / 1 / 0 |
| `gjk_host.subdistance/s1d/s2d/s3d/linear_combine` | 8 | 4 / 3 / 1 / 0 |
| `gjk_host.support_support.rs` | 8 | 4 / 2 / 2 / 0 |
| `gjk_host.ccd_pair_contact.rs` | 9 | 3 / 2 / 4 / 0 |
| `gjk_host.rounding-note` | 4 | 0 / 0 / 0 / 4 |
| `epa_host.epa_epa.rs` | 12 | 5 / 3 / 4 / 0 |
| `epa_host.attach_face_polytope.rs` | 8 | 3 / 2 / 3 / 0 |
| `epa_host.epa_witness_polytope.rs` | 8 | 3 / 3 / 2 / 0 |
| `epa_host.polytope2/3/4` | 7 | 3 / 2 / 2 / 0 |
| `epa_host.add_edge/get_edge/delete_face/invalidate_face` | 4 | 2 / 0 / 2 / 0 |
| `epa_host.rounding-note` | 4 | 0 / 0 / 0 / 4 |

分类计数：**`identical` 37 / `n/a` 51**（无 engine-semantic/not-decidable 站点；无覆盖率/通过率表述）。

- `identical` 37：host 实现按上游 f32 表达式逐条复刻（`dot3` 非融合左结合、`linear_combine` i 升序、
  `s2d/s3d` 固定顺序、`support` 各类型分支与 MESH hillclimb 同构、EPA 上下界/`face_norm2` 显式两次舍入、
  `attach_face` 取反判定、`epa_witness` 重心加权、ccd 装配 margin/中点/帧同式）——**source 级表达式/类型/
  求值次序对应，不是位级结论**。
- `n/a` 51：局部缓冲/中间量、整型索引与状态位、结构输出（`GjkResult`/`Polytope`/计数/溢出位）、
  迁移基准（origin）、8 条 rounding-note（host/device recipe 说明）。
- **D 批 device 级**：host 路径无 PTX，88 站点整体记 not-decidable 原因 `host-path-no-ptx`（不逐站点重复）；
  本片只做 source 级对照，不声明 device/位级结论。

## 3. 方法与判据

- 产品锚点（repo 行号）：`src/physics/collision/gjk/` 的 `core.rs`（gjk/gjk_support）、`simplex.rs`
  （det3/same_sign/project_origin_*/s1d/s2d/s3d/linear_combine/subdistance）、`support.rs`（support/
  support_mesh）、`math.rs`（dot3/make_frame 等）、`contact.rs`（ccd_pair 装配）、`epa.rs`（epa/add_edge/
  get_edge）、`polytope.rs`（attach_face/epa_witness/状态位）、`polytope_build.rs`（polytope2/3/4）。
- 参考锚点（`collision_gjk.py`，冻结 71da24d9）：`support` 120–245、`_gjk_support` 618–656、
  `gjk` 657–867、`_linear_combine` 292–302、`_subdistance` 303–368、`_S3D` 369–444、`_S2D` 445–589、
  `_S1D` 590–617、`_attach_face` 249–274、`_epa` 1341–1952、`_epa_witness` 969–1042、
  `_polytope2/3/4` 1043–1340、`_add_edge` 947–968、`eval_ccd_write_contact` 769–1026、
  `multicontact` 2216–2454（逐站点 file:line 见 `evidence.json`）。
- **host recipe 口径**：host 站点刻意非融合（独立乘、独立加、两次舍入；不用 `mul_add`、不引 f64）；
  device 侧 pinned ccd PTX 为 `fma.rn` 融合语义（另一 recipe）——本片只做 source 级判定，
  不声明 device 等价或位级保证。
- `identical` 的 source 级含义：产品与上游在同一表达式上写法/类型/求值次序可对应（f32 全链、无升精度）；
  `n/a`：非算术点（缓冲/整型/结构/说明）。

## 4. 输入与哈希

- sites.json `e0d4a95bf5c0acf13510b20f51e92ece1ec63ab7c7ac9e37be38cdd0841bac3c`/261,483（工作材料）。
- 词表 addendum `453c5c9157341fa28ecb009c50f2ff560b6f43e15f593ed9b7a75f27dd5b1b26`/6,771（工作材料）。
- upstream revision `71da24d956378a87a703b6e1442b13aec0c4ac29`；tree.head `dbffa8dd7460bdf31f647b3c1811073f19751e29`
  （本卡主提交父提交）。
- A-3 原 evidence `c9a9676f42092cbd53d82b31c34e5f2c0c27f512d1750ee6eff1bc065be00d36`/53,532（未改）。

## 5. 非声明

不宣称 NP01 通过/E 指标完成/上游等价/双平台/位级保证；不改判据/参考/容差/相机/recipe；
零 GPU、零窗；`identical` 仅为 source 级表达式/类型/次序对应；608/598 双口径见收尾 addenda。
