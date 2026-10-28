# R05-S1 hfield 静态参数独立复核（语义重算 / 裁决 A 等价 / 单份 / ABI 红线 / 证据）

任务：`2026-10-28-render-bvh-r05-s1-hfield-params-check`（P8 渲染与 BVH；scope 仅本目录）
被复核对象：`2026-10-28-render-bvh-r05-s1-hfield-params`（实现+证据 `c2a098aa`、收口 `50f088c4`；判据树 HEAD `4723fd44`）
上游锚点：冻结 MJWarp 3.15 `render.py`（`71da24d`）L332 / L462-464 / L682-685 / L623 / L858
复核方式：只读 + 四门禁独立复跑 + **临时对抗性探针**（运行后删除、不入库）；无 GPU 窗口、无渲染数值结论。

## 0. 结论

**verdict：pass；阻断 0；观察 6（O1/O3 建议修，O2 建议，O4–O6 记录接受）。**

## 1. 独立复跑

| 项 | 命令 | 退出码 | 结果 |
| --- | --- | --- | --- |
| fmt | `cargo fmt --check` | 0 | 通过（1.6 s） |
| clippy | `cargo clippy --all-targets --locked -- -D warnings` | 0 | 通过（0.4 s） |
| clippy-cuda | `... --features cuda -- -D warnings` | 0 | 通过（0.4 s） |
| unit | `cargo test --locked` | 0 | **1060 passed / 0 failed / 387 ignored**（131 条 result 行求和） |
| 架构树 | `normify validate --fingerprints` | 0 | 错误 0（警告 5，既有项） |

- unit 计数与实现片 gate 记录一致（`.normify/runs/<impl>/unit.json` 1060/0/387，at `13:28:34Z`）；比基线 1050 增加 10 条即本片新测。
- 实现片 `normify check` exit 0、四门禁记录 exit 0；实现文件在收口后**零改动**（`50f088c4..HEAD` 对新 4 文件 diff 空、工作树无未提交改动）。

## 2. 语义独立重算（我的对抗性探针，7 条全过 + 1 条构造观察）

实现片 10 条单测覆盖合法/全缺/部分缺/size 形状/长度不一致/adr 越界/非正行列/极端乘积/空集/单份结构性断言；
我另写临时探针（`tests/_qa_r05_hfield_probe.rs`，运行后删除）覆盖实现证据**未覆盖**的边界，7/7 通过：

1. **overlap adr 总和**：两段各自在界内（`0..4`、`2..6`，len=6）但 `Σspan=8 > 6` → `LengthMismatch{field:"hfield_data",expected:8,actual:6}`（卡面 `data.len()>=Σ` 口径成立）。
2. **空/不匹配长度优先级**：`nrow=[] ncol=[1]` → `hfield_ncol(0,1)`；`adr` 长 → `hfield_adr`；`size` 非空而 `nhfield=0` → `hfield_size(expected 0, actual 4)`（长度检查先于 `nhfield==0` 返回 `None`）。
3. **缺字段优先级矩阵**：分别缺 `size/nrow/ncol/adr/data` → `MissingField` 依次点名，五种均验；`Display` 非空。
4. **data 尾部多余合法**：`data.len() > Σspan` 为 `Ok`，`data_of` 只返回 `span` 段、不吞尾部，越界下标 → `None`。
5. **精确边界**：`adr + span == data.len()` → `Ok`。
6. **panic-freedom**：`(i32::MAX, i32::MAX)`、`adr=i32::MAX/i32::MIN`、`nrow=i32::MIN`、空 data 等 6 组极端输入 `catch_unwind` 均不 panic 且给出可读错误。
7. **单份/借用**：`params.data.as_ptr() == data.as_ptr()`（未复制）、视图数 == `nhfield`、二次解析逐字段相同。

另做构造性探针（`_qa_r05_hfield_probe2.rs`，运行后删除）：**外部直接构造非法 `HfieldParams`**（`adr=usize::MAX, span=2`）→ `data_of(0)` 在 debug 下 **panic**（`hfield.rs:119`，`adr + span` 未 checked）；外部构造空 `hfields` → `is_empty()==true`（与 doc「恒为 false」不符）。该路径不经过 `resolve_hfield`（其产出不变量成立），记为观察 O3。

## 3. 裁决 A 语义等价核验（自己读源码，不抄清单）

- `RENDER_FIELDS: [FieldSpec; 116]`，实测 **116 项且无任何 hfield 字段**；`src/model/render.rs` 500 行；`src/io/render.rs` **无 hfield**（grep 0）；
  `RenderFields::new` 仅包装 `Vec<FieldData>`、`field()` 经 `index_of` 只在 `RENDER_FIELDS` 里查名 → `field("hfield_*")` 恒 `None`、无法注入。→ 原卡面 `resolve_hfield(&RenderFields)` 不可执行，裁决 A 前提成立。
- 卡面 acceptance 与实现逐条对应（我逐条比对）：五字段同时提供/缺任一 `MissingField`/类型错配（强类型编译期 + `size.len()!=4nhfield` 的 `LengthMismatch`）/`size` 长度/`nrow|ncol|adr` 长度/`data.len()>=Σ`/`nhfield=0→None`/非正行列/乘积 `checked_mul`/`adr` 越界可读错误/不 panic/单份——均有实现与测试对应。
- **未夸大覆盖**：README §6 明确「渲染侧 hfield 数据通道未接入（由 `…-r05-hfield-channel` 承担，S2 前置）、S2 数据源适配未做、设备侧 BVH 未实现、无上游 2D 数值/等价」；证据中无窗口/精度/等价正面结论。→ 与卡面「本片只做宿主校验」一致。

## 4. 单份核验

- `HfieldInputs` 无 `nworld`；`resolve_hfield` 签名不含 world 参数；代码无 `nworld` 读取、无 per-world 复制循环；
  `HfieldParams.data` 借用输入切片（指针相等实测）、`hfields` 数恒等于 `nhfield`。
- 实现片「单份」单测属**结构性断言**（输入无 world 维度，无法观察 per-world 复制）；我以上述指针/循环检查补充该口径（O4）。

## 5. ABI / 红线（零改动）

- `MEGAKERNEL_SLOTS` = **81**（`args.rs`）；`hfield_bvh_id` 仍为占位 `upload(&[0u32, 0])` + 原槽位 `args.push(array(&hfield_bvh_id, RenderElem::U64, &[1]))`（`megakernel.rs` 未改）。
- `render.cu` 499 / `launch.rs` 500 / `cast.rs` 494 / `model/render.rs` 500 / `camera_output.rs` 499 / `megakernel.rs` 490：**实测行数一致且零改动**（`4723fd44..HEAD` 对这些路径 diff 空）；`DEVICE_SOURCES=28`、入口 6、`CollisionArgs=504 B`、`fixtures/precision` 未动；本片未触碰任何 `*.cu`。
- 新文件行数：`hfield.rs` **332**、`tests/render_hfield_params.rs` **284**、`params.rs` 193、`mod.rs` 384——全部 ≤500。

## 6. 证据完整性（实现片）

- 目录 **7 文件**（README + receipt + manifest + 4 logs），`manifest.txt` **6 条**逐条对 **committed blob** 相符、**不自引用**、LF、无 BOM、禁词 0。
- 实现片 manifest 仅登记证据自身；代码文件未列入（可接受，提交链可定位）。
- 证据含 `logs/lines.log`（行数/bytes 与我的实测一致）、`logs/test.log`（10 passed）、空 `fmt.log`（通过）。

## 7. 观察（O1/O3 建议修；O2 建议；O4–O6 记录接受）

- **O1（一般｜建议修）模块源未登记**：`src/render/context/hfield.rs` 是新模块源，但 `.normify/modules/mjwarp/render.md` 的 `[[source]]` 列表**没有登记**它；
  收口 `50f088c4` 只刷新了 `render.md` 的指纹（未加 source）。对照 R09-S1 先例（`context/lighting.rs` 已登记于 render.md L116）。建议补
  `[[source]] path = "src/render/context/hfield.rs"` 并重刷指纹；否则架构树对 `mjwarp.render` 的模块源不完整（`normify validate` 不会报错，属静默缺口）。
- **O2（建议｜接受或修）`HfieldError::FieldKind` 无构造点**：该变体仅在枚举定义与 `Display` 出现（实测构造 0 处），runtime 不可达。
  与证据「保留与 `ContextError` 同构的错误面」的说法一致，但建议在 mapping 里注明「保留变体、当前不可达」或后续删除，避免误读为可被返回。
- **O3（建议｜修）`HfieldParams`/`HfieldView` 公开字段可绕过不变量**：字段均为 `pub`，外部可构造非法视图；`HfieldParams::data_of` 用
  `view.adr + view.span`（未 `checked_add`），debug 下 `adr=usize::MAX, span=2` **实测 panic**（`hfield.rs:119`）；且 `is_empty` 文档
  「恒为 false」对任意构造不成立（外部构造空 `hfields` → `true`）。`resolve_hfield` 自身路径不受影响（其产出不变量成立），但作为公共 API 建议：
  字段私有化 + getter（或 `data_of` 用 `checked_add`），并把 `is_empty` 文档改为「`resolve_hfield` 返回值恒为非空」。
- **O4（记录｜接受）「单份」单测强度**：单测只能做结构性断言（`len==nhfield`、`Σspan=37`、两次解析相等），无法观察 per-world 复制；
  代码层已核无 `nworld`/无复制，我另用指针相等补充。建议测试注释写明该口径。
- **O5（记录｜接受）`#[path = "hfield.rs"]` 挂模块**：与既有 `lighting.rs` 同构（R09-S1 O2 已记录的结构债），本片沿用、无新增债务类型。
- **O6（记录｜接受）裁决 A 留痕充分**：原卡签名差异、不可执行理由、语义等价清单、后续卡归属（channel/S2 适配）均在证据 README §2 与 receipt `ruling_a`；`
  本片未宣称渲染侧 hfield 通道具备。

## 8. 边界（我没有做的）

- 无 GPU、无窗口、无渲染数值结论；上游 HFIELD 求交/数值等价不在本片（S2/channel 卡）。
- 未评测设备侧 hfield BVH（尚未实现）；未做上游 2D 等价。
- 我的对抗性探针为临时测试文件（运行后删除、未入库），用例与结果见 `logs/probe-attack.log`、`logs/probe-construction.log`；
  `probe-construction.log` 含一条预期内的 panic 观察（O3），是 debug 下外部构造非法值的记录，不是实现单测失败。

## 9. 复算入口

```text
<repo>/target/render-bvh-precheck/venv312/Scripts/python.exe docs/development-evidence/render-bvh/r05-s1-hfield-params-check/recompute.py
```

`recompute.json` 含：实现证据完整性、行数/红线、裁决 A 证据（116 项/无通道/new 包装/查名）、语义面静态核对（变体/Display/checked/无 nworld/无 unwrap）、
ABI（81 槽/占位/DEVICE_SOURCES/504 B）、收口后零改动、门禁记录（1060/0）、实现单测日志、我的探针对照——`problems = []`。
