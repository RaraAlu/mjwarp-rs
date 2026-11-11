# QA 复核 W2/A2-1 mesh multicontact 容量 32/32（`2ee0540d`，源码 `5689e0ef`）

- 复核对象：卡 `2026-11-12-w2-a2-1-mesh-capacity-32`；源码 `5689e0ef` → 证据 `c4c9c41e` → 卡收口 **`2ee0540d`**；
  证据目录 `docs/development-evidence/g10-p5-mesh-cap32/`（manifest `dd863899…`/1,130）。
- 性质：只读复核（零 GPU、零窗）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，0 一般 + 1 低 + 2 观察）**。

## 1. 容量 32/32 与 Arch `a2-slices.json` 逐项一致

- **设备宏 ✅**：`multicontact_geom.cu` 实测 `G5_MC_MESH_DEG_MAX 32`、`G5_MC_MESH_FACE_MAX 32`（原 10/20）。
- **scratch ✅**：`contact.cu` 实测 `polygon/clipped[64]`、`pnormal/pdist[32]`、`idx1/idx2/n1/n2/endvert[32]`、
  `face1/face2[32]`——与 Arch 目标表 11 项逐项一致。
- **宿主镜像 ✅**：`clip.rs` 实测 `pn/pd [32]`、`polygon/clipped vec![..; 64]`；**新增前置检查
  `if nface1 > 32 || nface2 > 32 { return result; }`**（确定性空结果，注释明写与设备 guard/`pnormal[32]` 对齐）。
- **常量/措辞 ✅**：`types.rs` 实测 `FACE_MAX=32`、`DEGREE_MAX=32`；`error.rs` 含「A2 起 32/32」口径；
  `multicontact.cu`（64/32/32）、`multicontact_clip.cu`（32/64）、`multicontact_mesh.cu`（32/32）注释同步。

## 2. 33 顶点容量门语义

- **确定性、不 panic ✅**：`clip.rs` 对 `nface1>32 || nface2>32` 直接返回零接触 `ClipResult`（非 panic）；
  `mod.rs::polygon_clip_rejects_thirty_three_vertex_face` 断言 `ncontact == 0`（我核了断言体）。
- **产品路径容量错误 ✅**：`inputs_from_snapshot_tests.rs::product_entry_rejects_face_33/degree_33` 断言
  `violations == [(Face, 33, MESH_MULTICONTACT_FACE_MAX)]` / `[(Degree, 33, DEGREE_MAX)]`，且 Display 断言含
  `actual=33`、**`limit=32`**、拒绝路径——`limit=32` 口径明确。
- **不写 over-32 等价 ✅**：README §6 与 receipt 非声明明确「不宣称与上游一致（over-32）/位级等价」；
  over-32 产品路径=宿主确定性错误、设备直调=record-only（A2-3 口径），未写成等价。

## 3. guards / ABI / 无新增 .cu

- **guards 保留 ✅**：`multicontact_mesh.cu` 实测 L120/L148 `if (v1_num > G5_MC_MESH_DEG_MAX)`、L180
  `if (nvert > G5_MC_MESH_FACE_MAX)` —— defense-in-depth 在位。
- **ABI 零改动 ✅**：`abi_tests.rs` 仍断言 `size_of::<CollisionArgs>() == 504`（unit 1409 内通过）；
  `5689e0ef` 的 12 个文件**不含** `abi.rs`/`product.cu`/`Cargo.*`/`fixtures`/`scripts`；`source.rs` 声明
  `DEVICE_SOURCES: [(&str,&str); 28]`（28 段）；**无新增 `.cu`**（5 个 `.cu` 均为 M）。

## 4. 8 条边界单测（按断言内容，不是跑通）

- **clip 3 条 ✅**：`accepts_thirty_two_vertex_face1`（`ncontact ∈ [1,4]`）、`face2`（64 中间缓冲，`∈[1,4]`）、
  `rejects_thirty_three_vertex_face`（`ncontact==0`）；A1 的 20 顶点 face1/face2 **保留**且 PASS。
- **宿主 4 条 ✅**：`product_entry_accepts_face_32_degree_32_boundary`（33 顶点、32 边形+32 三角 → `is_ok()`）、
  `rejects_face_33`/`rejects_degree_33`（精确 violations + `limit=32`）、`host_limits_match_device_macros`
  （解析 `.cu` 宏并与宿主常量逐一相等）。
- **我重跑 ✅**：`cargo test --locked --lib physics::collision::gjk::multicontact` → **5 passed / 0 failed**
  （929 filtered）；`... runtime::collision_gpu::inputs_from_snapshot` → **6 passed / 0 failed**（928 filtered）。

## 5. 门禁 / 棘轮 / 证据 / 口径

- **unit ✅**：`.normify/runs/.../unit.json` = **1409 passed / 0 failed / 402 ignored**，与 `logs/gate.txt` 一致
  （基线 1406 + 3 条新 clip 测试）。
- **500 行棘轮 ✅**：实测 `inputs_from_snapshot.rs` **233 行**、新测试文件 **258 行**、`mod.rs` **408 行**、
  `multicontact_geom.cu` 412 —— 与 `logs/line-counts.txt` 逐行一致且全部 <500。
- **manifest ✅**：`dd863899…`/1,130、**10/10 行全对**（README/receipt/4 logs/4 materials）。
- **口径 ✅**：U7/U8 设备数值判定明确留待 **A2-3 窗口**（README §6 明写 A2-2/A2-3 不在本提交）；
  全文无「mesh 容量缺口已清 / P5 通过 / 等价」正面声明（仅出现在「不宣称」列表）。
- **过程事件 ✅**：README §8 记录 01:33 QA 并发提交事件；我核 `5689e0ef` 12 文件**无** QA/Codex 路径，
  与「本提交不含 QA/Codex 文件」一致。

## 6. 发现

### 低
- **L1 policy_scan 1 命中**：README §6 非声明列表中的「位级」「等价」两词组合未带否定词（列表形式），
  实测 `policy_scan: files=2 hits=1 RESULT: FAIL`。语义是「不宣称」，但按团队证据卫生应 0 命中：
  建议改为「不宣称位级等价」邻近否定式（与其它卡同一修法）。

### 观察
- **O1 「8 条边界单测」计数口径**：证据实际为 **7 组 / 9 条**（clip 3 + 宿主 4 + A1 保留 2；另有 2 条既有用例
  保留：fan-cone-24、without-sidecar）。建议证据写清「7 组 9 条（含 A1 2 条保留）」，避免与派工口径的「8 条」
  对不上产生审计歧义。
- **O2 旧用例重命名未点明**：旧 `product_entry_rejects_fan_cone_24_on_both_dimensions` 随容量口径改写为
  `product_entry_accepts_fan_cone_24_within_capacity_32`（24 > 旧 20/10、≤ 新 32，语义正确）；README §3 未点明
  该重命名，建议注明「旧 rejects 用例随容量提升改判为 accept」，避免被误读为删除旧断言。

## 7. 非声明

本复核只对 A2-1 的容量改动/测试断言/证据卫生负责；**不宣称** U7/U8 数值判定通过（属 A2-3 窗）、
mesh 容量缺口已清、G10@P5 整族通过、准入/发布、上游等价；不宣称位级等价；零 GPU、零窗、不改判据/参考/容差。
