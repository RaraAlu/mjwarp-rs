# G10@P5 mesh multicontact 容量 A1（16→20）

任务：`2026-10-31-g10-p5-mesh-cap20`（Arch；P8；离线，零 GPU 窗口、零冻结）
用户拍板：2026-10-30（via Chief）mesh 选 **A1（16→20，实现）**；Chief 2026-10-31 批准离线实现，GPU 验证窗排在用户签窗口后。
预核：`target/mesh-a1-precheck.md`（只读，不入库）。

## 1. 范围与预期

- **A1 只修 U8**（prism-20：单 polygon 顶点 20）：把 mesh multicontact 分支的固定 scratch 上限 16→20、
  多边形裁剪缓冲 32→40，让 prism-20 走完整 polygon clip；预计设备接触数 1 → **4**（上游参考
  `docs/development-evidence/g10-p5-branch-closure-capture/upstream-mesh-limits-gpu.json`，sha `b6abb8fa…`）。
- **U7（fan-cone-24：顶点 24 / degree 24）A1 修不了**：`G5_MC_MESH_DEG_MAX` 仍为 10、单个 polygon 上限 20，
  fan-cone-24 仍返回 0 → 回退单接触，属 A2（动态容量）范围。**证据不得写「mesh 容量缺口已清」。**
- **零 ABI / 零槽位改动**：全部是 per-thread 局部数组与宿主 `Vec` 容量；81 槽、`CollisionArgs`、launch 签名、
  JSON 字段全不动。判据/容差/参考零改动；U8/U7 维持 **record-only**（是否把 U8 提升为 pass/fail 等用户窗前后确认）。

## 2. 改动（7 个文件）

### 2.1 设备（`src/physics/collision/cuda/`）

| # | 文件:位置 | 现在 | A1 后 |
| --- | --- | --- | --- |
| 1 | `contact.cu`（multicontact 分支 scratch） | `v3 polygon[32];` | `v3 polygon[40];` |
| 2 | 同上 | `v3 clipped[32];` | `v3 clipped[40];` |
| 3 | 同上 | `v3 pnormal[16];` | `v3 pnormal[20];` |
| 4 | 同上 | `float pdist[16];` | `float pdist[20];` |
| 5 | 同上 | `v3 face1[16];` | `v3 face1[20];` |
| 6 | 同上 | `v3 face2[16];` | `v3 face2[20];` |
| 7 | `multicontact_geom.cu` | `#define G5_MC_MESH_FACE_MAX 16` | `#define G5_MC_MESH_FACE_MAX 20` |
| — | `multicontact_geom.cu` | `#define G5_MC_MESH_DEG_MAX 10` | **不动**（U7 未覆盖，A2） |

### 2.2 宿主镜像（`src/physics/collision/gjk/multicontact/`）

| # | 文件 | 现在 | A1 后 |
| --- | --- | --- | --- |
| 8 | `clip.rs` | `pn [[f32;3];16]` / `pd [f32;16]` | `[20]` / `[20]`（与设备 `pnormal/pdist[20]` 对齐，否则 20 边形 CPU 参考路径越界） |
| 9 | `clip.rs` | `polygon/clipped vec![..;32]` | `vec![..;40]`（与设备缓冲对齐） |

### 2.3 注释契约 5 处

`contact.cu`（npolygonmax=20）、`multicontact_geom.cu`（FACE_MAX 20 + degree 仍 10 / U7 未覆盖）、
`multicontact.cu` 头注释容量行、`multicontact_clip.cu` 容量契约、`multicontact_mesh.cu` 容量边界段，
统一改为 20 / 40 / face 20 并写明 U7 未覆盖（A2）。完整 diff 见 `materials/src-diff.patch`。

### 2.4 新增 CPU 单测（2 条，`gjk/multicontact/mod.rs`）

- `polygon_clip_accepts_twenty_vertex_face1`：20 边形作 face1 会写 `pn/pd[19]`，A1 之前是越界点；
  断言裁剪出接触且 `ncontact ≤ 4`。
- `polygon_clip_accepts_twenty_vertex_face2`：20 顶点 face2 需要 40 格中间缓冲；断言裁剪出接触且 `ncontact ≤ 4`。
- 两条都只跑 CPU（`cargo test --lib`），设备行为在验证窗内跑。

## 3. 内核身份（collision 设备源）

`nvrtc_smoke` 探针（`cargo test --locked --features cuda-probe --lib nvrtc_smoke -- --ignored --nocapture`）：

| 项 | before A1 | after A1 |
| --- | --- | --- |
| assembled source | 255,840 B / `28385801…` | **254,743 B / `f32f0af2…`** |
| PTX | 3,883,740 B / 6 入口 | **3,883,769 B / 6 入口** |

⇒ **collision 内核身份已变**：任何后续 GPU 窗口必须**新冻结**，旧 guards 一律不复用。
（源字节变小是因为容量注释重写；PTX 同 6 入口、+29 B。）

## 4. 门禁与构建

| 项 | 结果 |
| --- | --- |
| fmt / clippy / clippy-cuda | 通过 |
| unit | 通过，**1113 过 / 0 败 / 397 ignored**（1111 + 新增 2 条 A1 单测） |
| `cargo test --locked --no-run` | **EXIT=0** 含 `Finished`（`logs/no-run.txt`） |
| `normify validate --fingerprints` | 错误 0（done 刷新 `mjwarp.physics.collision` 指纹） |

`logs/gate-*.txt` 是某次门禁的转写快照；权威结果见 `.normify/runs/<task>/`。

## 5. 边界与后续

- 不改判据/容差/参考/fixtures；不写「U8 已成为 pass/fail」；不改写 g10 既有窗口证据与 render-bvh 证据。
- 零 GPU、零冻结；U7 仍回退单接触，A2（动态容量）另议。
- 后续（窗口，等用户签）：新 suite（如 `g10-p5-mesh-cap20`）+ 全新冻结（collision 身份已变）→ U8 断言用例
  + M1/M2/M3/S4 回归重放；窗口内如实记录 U7 仍为单接触。
- 已知项：证据目录的 EOL 保护与 `.cu text eol=lf` 钉版由后续 `2026-10-31-evidence-eol-hygiene` 卡统一处理。
