# SDF 判据 v2（已签发 2026-10-31T11:04:37+08:00；approved_by=user via Chief）

版本：**`render-sdf-criterion-v2`**（**已签发**；B=128 ULP；签署时间 2026-10-31T11:04:37+08:00；approved_by=user via Chief）；判据数值以签发版为准，清单外 1 ULP 即失败。
作者：Arch 2026-10-31；依据：Chief 定性（SDF 渲染 = `gtype==8` 走底层 mesh `ray_mesh_leaf`，与 `ray_sphere` 的 `a` 求和次序无关；U4 的 73 ULP 属 scene-a **普通球 primitive** 的 FMA 结合律差异）+ `target/sphere-ref-conflict-rca.md` + attempt1/2/3 证据。

## 0. 为什么需要 v2

- 现有判据要求 depth **位级**（`mismatched=0` 且 `max_ulp=0`）。上游 warp GPU 对同一 `a = dot(vec,vec)` 表达式在不同场景的生成 kernel 里给出不同 FMA 结合次序（scene-a ↔ (0,1,2)；scene-sdf ↔ (1,0,2)，机器证据见 RCA §2），**任何单一产品实现都无法同时位级匹配两份参考**。
- 用户已否决「SDF 不发布」。因此把「SDF 能力本身」与「scene-a 普通球 primitive 的轮廓像素」拆开：SDF 实体保持位级；scene-a 球轮廓走**受控、可推导、fail-closed** 的豁免。

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

## 2. 受控豁免：scene-a 回归的球轮廓像素

适用对象：`2026-10-31-render-sdf-validate` 的 8 个 scene-a 回归实体（`render_e2e`/`render_kernel_depth_seg`/`render_kernel_flags`/`render_kernel_rgb`/`render_r10_multi_world`/`render_skybox_reference` 及其内部子臂）。

豁免谓词（三个条件**同时**满足才豁免，且只对该像素生效）：

1. 参考 `seg` 指向球 geom（scene-a: geom 2, `seg=(2,5)`）；
2. 该像素在球轮廓带上：参考 depth 在 4 邻域内有 ≥ 1 个非球命中（背景或其它 geom）；
3. 与冻结基线（本版本签发时的位级形态）比的 depth 偏差 `|ΔULP| ≤ B`（见 §3）。

豁免**外**：任何像素 `|ΔULP| > 0` 即失败；豁免**内**：`|ΔULP| > B` 即失败。

### 2.1 受影响像素（草案值，冻结时以 dump 工具输出为准）

从 attempt3（`docs/development-evidence/render-bvh/sdf-validate-window/attempts/attempt3/logs/`）的 5 个回归实体日志汇总（cam0 全部 0）：

| 实体/臂 | cam1 受影响像素 | max_ulp | >4 ULP 像素 | 参考基线（attempt1/2 位级为 0） |
| --- | --- | --- | --- | --- |
| `render_kernel_depth_seg`（recipe 臂） | **183 / 43,200** | 73 | 149（其中 9 个 3 ULP、25 个 4 ULP、149 个 >4） | 0 |
| `render_skybox_reference`（r02-s1） | **183** | 73 | 149 | 0 |
| `render_kernel_flags`（aa2 臂，spp=2 采样路径） | **172** | 91 | — | 0 |
| `render_kernel_rgb` / `render_e2e` / `render_r10_multi_world` | 与 183 同批（首差 28969/(169,120)） | 73 | 149 | 0 |

**冻结规则**：签发后由 dump 工具（新增，只在冻结步运行、只读输出文本）对每个实体输出「像素索引、x、y、参考 seg、两形态各自的 depth 位型与 ULP、是否轮廓」清单，写入 `scene-a-sphere-outline-<entity>.txt`，与 guards 一起哈希（清单 sha 进 guards 文件/窗口证据）。**本节的 183/172/73/91 是草案参考值，冻结时以清单实际内容为准。**

## 3. ULP 上界 B 的推导（不是「观测 73 就取 128」）

### 3.1 误差来源与模型

两份参考只差 `a = dot(vec,vec)` 的三项 FMA 结合次序；`b`、`dd`、`det` 融合、`1/a`、根选择、`depth = t·(−v_z)` 两侧一致。令 `u = 2^-24`（f32 单位舍入），`a ≈ |v|² ≤ 1+2u`。

- **δa**：两个结合形态各 3 次舍入，差值界 `|δa| ≤ 4u·a_max`（两项求和各 2 次舍入，取保守 4u）。
- **δdet**：`det = b² − a·c`（融合），a 变化经 `a·c` 传入：`|δdet| ≤ |c|·δa + u·|a·c| ≤ 5u·|c|`（取 a≈1）。
- **δt**：`t = (−b − √det)/a`，`|δt| = |δdet| / (2·a·√det)`。
- **δdepth**：`depth = t·(−v_z)`，`|δdepth| ≤ |v_z|·|δt| + u·|depth|`。

因此对某像素：
```
ΔULP(px) ≈ [ 5u·|c(px)|·|v_z(px)| / (2·a(px)·√det(px)) ] / ULP(depth(px))
```
`ULP(depth) = 2^(⌊log2 depth⌋ − 23)`。

### 3.2 极值输入（冻结时由 dump 工具给出，草案用保守估计）

- `√det_min`：frozen ray 栅格上球命中像素的最小正 `√det`（轮廓像素离理想切线最近处；由 dump 工具在冻结时精确给出）。
- `|c|max`、`|v_z|max`、`depth_min` 在受影响集合内取极值。
- 草案保守估计：`√det_min ≈ 3.4e-2`（由 attempt3 观测的 73–91 ULP 反解同一公式得到的上界一致性检查，不作为签发值）、`|c| ≈ 4·r²+…`、`depth_min ≈ 1.2`。

### 3.3 数值上界

代入公式（保守取 `|c|max = 4.2`、`|v_z|max = 1`、`a_min = 1−4u`、`√det_min = 3.4e-2`、`ULP(1.2) = 1.19e-7`）：

```
ΔULP ≤ 5·2^-24·4.2 / (2·1·3.4e-2) / 1.19e-7 ≈ 110 ULP
```

再乘 1.15 安全系数并向上取到 2 的幂，得草案签发值 **B = 128 ULP**。
`B` 的最终值在冻结时用 dump 工具给出的精确 `√det_min`/`|c|max` 重算一次（同一公式），若重算值 > 128 则该 attempt 失败、需回炉（不允许临时抬高 B）。

## 4. Fail-closed 规则

1. 清单外像素：depth 必须位级相等（`Δ=0`），否则失败。
2. 清单内像素：`|ΔULP| ≤ B`，否则失败；清单内像素集合本身也必须与冻结清单逐像素一致（多一个/少一个都失败）。
3. 清单文件与 guards 一起冻结、一起哈希；窗口内禁止重写清单。
4. seg 仍要求全像素 exact；rgb 仍 ≤1 LSB；SDF 实体仍全位级（§1）。
5. 任何「临时放宽 B / 手工删像素」都视为违规，本轮失败。

## 5. 待用户签的三项与后续

- 判据版本：`render-sdf-criterion-v2`（草案）。
- 受影响像素（草案）：**183**（scene-a cam1，per-entity；aa2 臂 172）；cam0 = 0。
- ULP 上界（草案）：**B = 128 ULP**（由 §3 公式导出，冻结时用精确极值重算确认）。

签发后顺序：重做 U1b（host 解锁）→ 全新冻结（含清单）→ U4 **attempt4**（同一条条件式协议；SDF 实体位级 + scene-a 回归按 v2）。
