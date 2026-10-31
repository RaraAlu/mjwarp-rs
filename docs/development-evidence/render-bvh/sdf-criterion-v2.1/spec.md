# SDF 判据 v2.1（已签发 2026-11-01T02:42:29+08:00；approved_by=user via Chief）

版本：**`render-sdf-criterion-v2.1`**（已签发；B=128 ULP；签署时间 2026-11-01T02:42:29+08:00；
approved_by=user via Chief）。取代 `render-sdf-criterion-v2`（2026-10-31T11:04:37+08:00）。
判据数值以签发版为准：清单外 1 ULP 即失败，清单内 >128 ULP 即失败，冻结时精确 max >128 即该 attempt 失败且禁止抬高 B。
依据：`render-sdf-criterion-v2` + `docs/development-evidence/render-bvh/sdf-validate-window-material-v2/v2.1-proposal.md`
（QA 复核 pass 2026-10-31，证据在 `prototype-lists/` 可独立复跑）+ `outline-replay-findings.txt`（v2 冻结两阻塞事实）+
attempt1/2/3 证据。

## 0. 三问头（对齐 `docs/development-evidence/criteria-purpose-audit/standard.md`）

- **裁定目标**：决定 SDF（`gtype==8`）能否发布。
- **服务对象**：产品发布（SDF 可达性）；scene-a 的 8 个回归实体只是内部回归，不是发布对象。
- **被测物本质**：SDF = `gtype==8` 按底层 mesh 走 `ray_mesh_leaf`（`render_common.cu:366` / 上游
  `render.py:738-761`）；scene-a 球轮廓 = 普通 sphere primitive 的 `a = dot(vec,vec)` FMA 结合律算术差，
  与 SDF 求交路径无关。
- **前提**：位级判据只在同编译形态下成立（两份上游参考对 `a` 的结合次序不同，机器证据见
  `reference-conflict-rca.md`），因此 scene-a 球轮廓不适用位级判据，按 §2 的精确差集清单受控豁免。

## 1. SDF 实体判据（不放宽，位级）

实体 `render_sdf_reference`（scene-sdf 参考，`fixtures/render-bvh/scene-sdf/capture-sdf/{sdf-mesh,sdf-occluder}`）逐条：

| 项 | 要求 |
| --- | --- |
| RGB | 每通道 `max_byte_diff = 0`（位级） |
| depth | `mismatched = 0` 且 `max_ulp = 0`（位级） |
| seg | `mismatched = 0`（逐像素相等） |
| 两跑确定性 | 同一 context 两次渲染 rgb/depth/seg sha256 相等 |
| shadow 开/关一致性臂 | depth/seg 位级相等；rgb `shadow_off ≥ shadow_on`（逆序超 1 LSB=0）；差异像素 > 0；两臂各自两跑 sha 相等 |
| DepthSeg 跨路径 | `render_depth_seg` 与 megakernel 的 depth/seg 位级相等 |

依据：a-order fix 下 scene-sdf 诊断 44/44 位级复现；该实体与 scene-a 参考互相独立，不受本草案豁免影响。

## 2. scene-a 回归：精确差集清单（取代 v2 的「三条件谓词」）

### 2.1 清单定义

对每个臂，`list(arm) = { p : depth_bits_form_A(p) != depth_bits_form_B(p) }`，取该参考 camera 1
的**全部球面像素**（参考 `seg` 指向球 geom）。其中：

- form A = `a` 结合次序 (0,1,2)（scene-a 参考形态）；
- form B = `a` 结合次序 (1,0,2)（a-order refix 后的产品形态）。

清单**不是几何启发式**，而是两形态在冻结参考上的**完整差异集**：产品（form B）与参考（form A）只可能
在清单像素上不同，清单外必须位级相等。

冻结时实测（与 attempt3 逐项一致）：

| 臂 | 球面像素 | 清单像素 | 首差像素 | 首差 xy | 精确 max |ΔULP| |
| --- | --- | --- | --- | --- | --- |
| recipe（spp=1） | 747 | **183** | 28969 | (169,120) | **73** |
| `render_kernel_flags` aa2 臂（spp=2，子样本 0） | 745 | **172** | 29448 | (168,122) | **91** |
| camera 0（两参考） | 0 | — | — | — | — |

### 2.2 清单文件与哈希

- 每个臂一份 ASCII/LF 清单：`scene-a-sphere-outline-<arm>.txt`，表头（criterion 版本、参考路径 +
  sha256、camera/raster/spp、列定义）+ 每个清单像素一行，按 pixel 升序：
  `pixel x y ref_seg_geom ref_seg_type ref_depth_bits formA_bits formB_bits ulp_ab`。
- 清单 sha256 与行数写入 guards 文件（`outline_lists[]`），并入冻结聚合（见 §2.4）。
- 清单在窗口内**不可改**：冻结后重写/删除/新增清单即违规；窗口 runner 在 body 前后各校验一次 sha。
- 可用原型（可独立复跑，`prototype-lists/`）：recipe 183 行/10,099 B/`cfd1038a…`、
  aa2 172 行/9,509 B/`14893fc6…`。最终值以冻结时工具输出为准（应与原型逐字节一致，否则停手报 Chief）。

### 2.3 逐像素规则（fail-closed）

1. **清单外**：depth 必须位级相等（Δ=0，1 ULP 即失败）。
2. **清单内**：`|ΔULP| <= B`（B 见 §3），超过即失败。
3. **清单成员本身**必须与冻结清单逐像素一致：多一个、少一个、换一个都失败。
4. seg 仍要求全像素 exact；rgb 仍 ≤1 LSB；SDF 实体仍全位级（§1）。
5. 任何「临时放宽 / 手工删像素 / 改清单 / 抬 B」都视为违规，本轮失败。

### 2.4 冻结计数

guards 条目 = **9 个实体 exe + 83 个只读输入 + 2 份清单 = 94 条**；`counts.total = 94`；
聚合 sha256 口径为「94 条按 repo-relative path 排序后的 sha256 hex 串无分隔拼接再 sha256」。
`-GuardsOnly` 必须 94/94 通过，并额外校验清单 sha、行数、精确 max。

## 3. 精确可算 B（B = 128 ULP，数值与 v2 相同）

### 3.1 定义

B 不再由解析误差界推导，而是由**精确可算量**定义：

```
B = 2 的幂 >= max over list pixels | depth_bits_form_A(px) - depth_bits_form_B(px) |
```

冻结时实测精确 max = recipe 73 ULP、aa2 91 ULP → 两臂均 **B = 128 ULP**（next pow2(73)=128、
next pow2(91)=128）。B 保持 v2 已签数值，不抬高、不收紧。

### 3.2 冻结时重算与失败规则

- 冻结步用 authoring 工具对 pinned fixtures 重算 exact max，并写进 guards：
  `outline_lists[].exact_max_ulp`，同时记录 `B = 128`、`criterion_version`。
- **exact max > 128 ⇒ 该 attempt 失败**；禁止抬高 B；禁止同一 attempt 重冻。
- 清单成员/清单 sha 与冻结值不符 ⇒ 失败。
- 清单内某像素 > 128 ⇒ 失败；清单外任一像素 Δ≠0 ⇒ 失败。

### 3.3 解析界（仅记录，不用于定 B）

v2 §3 解析公式（每像素自己的 `|c|`、`a`、`sqrt(det)`、`depth`、`ULP(depth)`）在本清单上的实测：

| 臂 | bound_raw min/median/max | margin(=1.15×raw) min/median/max | max obs/bound_raw | max obs/margin |
| --- | --- | --- | --- | --- |
| recipe | 25.39 / 32.13 / 360.49 | 29.20 / 36.95 / 414.57 | 0.4005 | 0.3483 |
| aa2 | 25.35 / 32.40 / 430.02 | 29.16 / 37.26 / 494.52 | 0.4203 | 0.3655 |

- 观察值均 ≤ 自身解析界（无像素 ratio ≥ 1），模型成立；但解析界比精确 max 宽 **4.94×/4.73×**
  （raw）或 **5.68×/5.43×**（margin）。
- 对可精确计算的量使用解析上界会把 B 抬到 512 ULP，属过度保守而非安全需要，因此解析界
  **仅作宽松上界记录**，明确不用于定 B；`B_rigorous = 512` 只作为对照保留在提案里。

## 4. 前提（A 类 + 显式前提）

精确值的前提必须同时成立，任一不成立则本判据不适用（且冻结步 fail-closed，不做解释性放宽）：

1. **同冻结参考形态**：`s7b/depth_seg_fast_math_false.json`（sha `d6ab6dde…`）与
   `s7b/flags/aa2_depth_seg.json`（sha `79a275ef…`）为 scene-a 参考；aa2 深度按产品规则取子样本 0
   （双倍栅格，`render_megakernel.cu`「Depth and seg are single-sample outputs: only the first pass writes them」）。
2. **同产品内核身份**：megakernel source `10070226…` / PTX `7782aeb1…` / 268,589 B（a-order refix 形态）；
   内核身份变化即需重算清单与 exact max。
3. **replay 保真**（已对冻结数据位级验证）：camera-space 射线对 S5 夹具 **256/256** 位级；
   form A 对冻结参考深度位**747/747（recipe）、745/745（aa2）**位级；form B 对 form A 给出
   **183 px/73 ULP** 与 **172 px/91 ULP**，与 attempt3 观测集一致；cam0 球面像素 0。
4. **冻结时重算**：清单与 exact max 由冻结步重算并写 guards；漂移即失败。

## 5. 签署块（已填写）

```
criterion_version : render-sdf-criterion-v2.1
based_on          : render-sdf-criterion-v2 (signed 2026-10-31T11:04:37+08:00, B=128 ULP)
bound_ulp         : 128            # 精确可算 B，保持 v2 数值
scope             : SDF 实体位级（§1）+ scene-a 相机 1 精确差集清单受控豁免（§2/§3）
exact_max_ulp     : recipe 73 / aa2 91（冻结时重算；>128 即失败）
analytic_bound    : recipe 414.57 / aa2 494.52（margin 口径，仅记录，不用于定 B）
signed_at         : 2026-11-01T02:42:29+08:00
approved_by       : user via Chief
signature_sentence: "SDF 实体保持位级；scene-a 球轮廓仅在冻结清单内、精确 max <= 128 ULP 时豁免；
                     清单外任何像素 depth 偏差 1 ULP 即失败，清单内超 128 ULP 即失败；
                     冻结时精确 max > 128 即该 attempt 失败且禁止抬高 B。"
```

## 6. 边界与不声明

- 只对 camera 1 的 scene-a 球轮廓像素生效（cam0 = 0）；SDF 实体、scene-sdf 参考、其它判据不变。
- 豁免是签名级判据变更，不得写成「常规通过」「SDF 已验证」「精度达标」「上游等价」。
- **窗口外不强制**：本判据的清单与精确可算 B 只在冻结窗口内强制（由窗口 runner 设置
  `MJWARP_RENDER_OUTLINE_LIST_DIR` + `MJWARP_REQUIRE_OUTLINE_CRITERION=1` 时才启用）；窗口外的普通
  `cargo test` / CI 不使用清单、行为与既有判据一致，也不得据本判据宣称任何通过。
- U4 在窗口按本判据完成 attempt4 之前仍为未通过；本轮窗口在计划与基线双签后才会打开。

## 7. errata（2026-11-01，记录型；已签数值不变）

本节只记录 v2.1 签发后发现的表述/证据修正，不改变任何已签数值或口径：

1. **清单格式 = 已签提案原型（9 列）+ bounds sidecar**：§2.2 的清单文件为
   `sdf-validate-window-material-v2/prototype-lists/` 的原型 9 列格式（`pixel x y ref_seg_geom
   ref_seg_type ref_depth_bits formA_bits formB_bits ulp_ab`，header 含 reference、reference sha256、
   camera/raster/spp、列定义），工具输出与原型**逐字节一致**：recipe `cfd1038a1cc50326de51a25495db68cd8b25c0983121dea88a2fb89ca83724ee`
   / 10,099 B / 183 行；aa2 `14893fc6a2479e8e8f5e9cc07cabe72fd9f37aea638dda25fc26839d59ec8eab` / 9,509 B /
   172 行。解析界 `bound_raw`/`margin` 移到 `outline-bounds-<arm>.txt` sidecar（仅记录，不用于定 B）。
2. **§1 恢复 v2 字面原文**：v2.1 入库时 §1 与已签 v2 §1 存在措辞差异（`>=`、`1 LSB = 0`、依据句、
   小节标题等），现已按 v2 `spec.md` 的 §1 逐字恢复，使「§1 与 v2 完全一致」成立。
3. **已签数值与签署信息不变**：B = 128 ULP；清单 183/172；exact max 73/91；签署时间
   2026-11-01T02:42:29+08:00；approved_by = user via Chief；§2 豁免规则、§3 精确可算 B 定义、
   §4 前提、§5 签署块均未改动。
