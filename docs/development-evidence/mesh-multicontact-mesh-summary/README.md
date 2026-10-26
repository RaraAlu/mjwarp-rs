# mesh multicontact 整片汇总（G10@P5，M0a → M3）

**任务**：`2026-10-27-mesh-multicontact-m3-closure-and-mesh-summary`（owner Arch）。
**范围**：把 M0a/M1/M0b/M2/M3 五片（+ 各自 QA 复核）的交付事实、门禁数字、未覆盖与「不宣称」清单收在一处，
供后续片与对外说明引用。**本文件只做汇总与指针，不改任何已 verified 数字、判据、容差、参考或期望值。**

## 1. 交付链（严格串行）

| 片 | 内容 | 实现提交 | QA 复核 | 结论 |
| --- | --- | --- | --- | --- |
| M0a | ABI sidecar `CollisionArgs` 496→504、`mesh_poly_args` offset 184、`MjwarpMeshPolyArgs` 72 B、9 指针 | `097c54b9`（形状修正 `b50d93a6`） | `91a9fdea` pass / 0 阻断 / 观察 7 | 已收口 |
| M1 | mesh-box / mesh-cylinder 上游 GPU 捕获参考 | `35192320` | `fe8956f4` pass / 0 阻断 / 观察 5 | 已收口 |
| M0b | 2 case × 32 数组 bins（含 9 个 mesh polygon slice） | `ac01d86d` | `f191fdba` pass / 0 阻断 / 观察 6 | 已收口（观察收口 `c377241e`） |
| M2 | convex-vs-mesh 设备实现（`multicontact_mesh.cu`，`DEVICE_SOURCES` 27→28）+ 测试入口 | `16214a22` / `85cce21b` | `271fe926` pass / 0 阻断 / 观察 8 | 已收口（观察收口 `c89d84c2`） |
| M3 | mesh-mesh 多接触验证（**0 行新设备代码**）+ O7 值域校验 + 32 数组 bins | `f1e5cda1` | `ab586682` pass / 0 阻断 / 观察 7 | 已收口（观察收口本任务） |

## 2. ABI 与设备侧事实

- `CollisionArgs` **504 B** / align 8 / 无隐式填充；`mesh_poly_args` @184；标量块 192–340；`out`@344（136 B）；
  尾 `nacon/nccd/overflow`=480/488/496。
- 9 个 mesh polygon 数组的 slice 形状（修正后真值）：`(polynum, polyadr, polynormal, polyvertadr, polyvertnum,
  polyvert, polymapadr, polymapnum, polymap)`；M3 fixture（nmesh=2）实测 **`[2,2,36,12,12,48,16,16,48]`**。
- **设备源与入口**：M2 后 `DEVICE_SOURCES` 28、入口 6；M3 **0 行新设备代码**（`src/physics/collision/cuda/` 零 diff、
  `src/physics/mod.rs` 零 diff、`DEVICE_SOURCES` 28→28、入口 6→6）。
- **NVRTC 冒烟（M2 后 = M3 后，因设备源未变）**：`source_bytes=254485` /
  `source_sha256=00810ddb5355564eed422e9d992a6843baf1f5e3541714230463df45a16ee253` /
  `ptx_bytes=3883740` / `entries=6`。
- 空/缺失 sidecar 语义不变：9 slice 全空 → `mesh_poly_args=null`、mesh 多接触不启用、行为与改动前一致；
  部分空/长度不自洽 → `InvalidArgument`。

## 3. 判据与参考

- 判据口径：位级优先；非位级按既有 **4·ULP@scale**（`FOUR_ULP_AT_ONE`，与既有片完全相同）；结构键
  （`worldid + geomcollisionid`）配对；行序按结构键归一化；零容差字段集未放宽。
- M1 参考：`docs/development-evidence/mesh-multicontact-m1/upstream-mesh-multicontact-gpu.json`
  sha `c83baffe6de695fdefda6852c574d5d0d7312ebb591c1b487dfc59ec126c206a`（5703 B，CPU 后端采集、非融合）。
- M3 参考：`docs/development-evidence/mesh-multicontact-m3/upstream-mesh-multicontact-mesh-mesh-gpu.json`
  sha `96e7895d13c3ba23475a013fcaa586485e93f95cced5351e2874973917e149a1`（3676 B，上游 GPU 融合参考，
  count=4、`geom=[[0,1]]×4`、dist 全 `-0.009999997913837433`）。
- 两份参考窗口前后均未变；旧 G5 bins/参考与 fixture 零改动。

## 4. 覆盖结果（判据内）

| case | 结果 |
| --- | --- |
| mesh-box / mesh-cylinder（M2） | 各 nacon 4/4、**bit_equal 4/4**、ULP 0、`contact.geom=[1,0]`、failures 0（M3 验证窗复跑同结果） |
| mesh-mesh（M3） | nacon 4/4、**bit_equal 4/4**、dist/pos/frame ULP 0/0/0、`contact.geom=[0,1]`、failures 0 |
| G5 multicontact | cylinder-box 314/314 bit；box-box-margin 391/392（pos 1 ULP）；G5 mesh-mesh（单接触路径）80/80 |
| launcher / primitive / plane / sphere_capsule / branches | 全 failures 0（B3 6 分支 + 4 swap 含 plane-mesh-graph swap） |
| stage2（9 synth） | dump 全完成、两次运行一致；窗外 CPU 比较按「记录 C 只记录」：17 dump / 16 条 case-kernel 比较行 / 11 条有失败（已知 recipe 分岔，非本程序回归） |

## 5. 两次验证窗（口径不同，不追溯）

| 项 | M2 attempt4 | M3 attempt1 |
| --- | --- | --- |
| 窗口内容 | 8 实体（M2 新 case + 9 G5 设备 case + B1/B2/B3 + stage2） | 9 实体（M3 新 case + M2 + launcher/primitive/plane/sphere_capsule/branches + stage2） |
| 结果 | **数值全绿 + guard 因 free floor 环境边界 abort，经 Chief 2026-10-26 裁定接受为有效轮** | **数值全绿 + guard 无 abort**（9/9 exit 0、body wall 202.1 s） |
| guard | 固定 floor 10240；attempt1–3 作废（env/free floor） | `floor=min(10240, baseline_free−3072)=10240`；open 门仍 `free≥10240`；732 样本 / delta 278-283-283 / free_min 13557 / util 60（streak 0）/ residual 0 |
| 编译 | 窗口内零 Rust 编译 | 同（只跑冻结 exe + nvidia-smi + guard 采样） |

M2 按当时固定 floor 执行、**不追溯改数据**；M3 起用派生公式。两个窗口均未写「窗口通过」。

## 6. 未覆盖（如实列出，不宣称）

1. **一般 mesh**：nmesh>1 的通用多 mesh 情形只测了 2 mesh fixture；`degree>10` / `polygon>16` 超出 scratch
   固定容量 → 走「返回 0 → 单接触回退」，**未覆盖**。
2. **mesh 分支层未逐条标注**：face 逆序、边法线、swap 几何序由 case 命中，但未像 B3 那样逐分支做覆盖台账。
3. **`hfield-capsule` 设备接触回归**：ABI 504 后无设备证据（补偿：launcher `hfield-sphere` 位级回归 +
   ABI 偏移静态断言）。
4. **`efc_address` 缺口**：上游捕获未记录该字段；设备侧断言全 `-1`，不构成对 efc 视图的等价声明。
5. **UNC-6 深穿透**：本程序 fixture 穿透 0.01，未触发深穿透分支（继续列未覆盖）。
6. **`contact.geom` 顺序**：multicontact 入口不做类型序归一化，顺序是调用方职责（`[1,0]`/`[0,1]` 由调用方传）。
7. **stage2 只按「记录 C」**：CPU 侧比较只记录不判 pass/fail，已知 recipe 分岔不计入本程序结论。
8. **性能、跨平台、Linux**：未做性能测试；Linux 按用户 2026-10-24 裁决集中回测，本程序不含任何跨平台结论。
9. **可复算性边界**：M3 拆分前的 `990/0/374` 只有 addenda 文字记录（拆前 WIP 未入 git），不能从仓内独立复算。

## 7. 债务

- **环 0→1（model→CollisionInputs）产品内无接线**：32 数组现由调用方/测试 support 组装；后备片
  `2026-10-26-product-collision-inputs-wiring`（backlog，不 start）。
- **mesh 分支层台账**：可仿照 UNC-3 做法补一张「face/edge/swap/退化」覆盖表。
- **M3 验证窗 runner**：`facts.txt` 的 `guard_aggregate=…OrderedDictionary` 是 runner 把字典对象字符串化的
  无效行；复用该 runner 前必须修正（修 runner 会破签名 → 重新报签）。本轮结果有效、无需重跑。

## 8. 不宣称清单（引用时逐条遵守）

- 不宣称 mesh 全类型 / 全分支 / 一般 mesh 已支持；
- 不宣称 G10@P5 整体通过、精度目标已达成、跨平台或双平台通过、上游完全等价、原生放行；
- 不宣称 `hfield-capsule` 设备路径已验证、`efc_address` 已等价；
- 不宣称性能达标（本程序未做性能测试）；
- `contact.geom` 顺序结论只对「调用方按 pinned 类型序传参」成立。

## 9. 证据指针

- M0a `docs/development-evidence/mesh-multicontact-m0a/`（+ `-m0a-fix-mesh-poly-shapes/`、`-m0a-independent-check/`）
- M1 `docs/development-evidence/mesh-multicontact-m1/`（+ `-m1-independent-check/`）
- M0b `docs/development-evidence/mesh-multicontact-m0b/`（+ `-m0b-independent-check/`、`-m0b-observation-governance/`）
- M2 `docs/development-evidence/mesh-multicontact-m2/`（+ `-m2-independent-check/`、`-m2-observation-governance/`）
- M3 `docs/development-evidence/mesh-multicontact-m3/`（+ `-m3-independent-check/`、`-m3-observation-governance/`）
- 台账 `docs/development-evidence/g10-p5-acceptance-map/`（README §7/§7.2、`map.json` 的 UNC-1）
- 治理清单 `docs/development-evidence/scheduling-registration-checklist.md`（§6/§7/§7.1/§8/§9）
