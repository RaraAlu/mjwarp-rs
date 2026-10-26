# G10@P5 M0a 修正：mesh polygon 数组形状语义（校验层 / 注释 / 单测 / 证据补注）

任务：`2026-10-26-mesh-multicontact-m0a-fix-mesh-poly-shapes`（in_progress，owner Backend；纯 CPU）。
性质：**修正 M0a 已 verified 的 CPU 校验层缺陷**；ABI / device 源 / fixtures / 判据**零改动**。
发现来源：M0b 第一块预检（结构阻塞，Chief/Arch 裁定单列本小修片、不并入 M0b）。

## 1. 根因（pinned 证据）

M0a 的 `validate_mesh_poly`（`src/runtime/collision_gpu/abi.rs`）把 6 个逐 entity 数组
（`polynum/polyadr/polyvertadr/polyvertnum/polymapadr/polymapnum`）统一按「长度都等于
`nmesh`」校验。但 pinned `mujoco_warp/_src/types.py`（上游 rev
`71da24d956378a87a703b6e1442b13aec0c4ac29`）明确分三段：

| 数组 | pinned dim（types.py 行号） | 归属 |
| --- | --- | --- |
| `mesh_polynum` / `mesh_polyadr` | `(nmesh,)`，L1329-1330 / L1864-1865 | 逐 mesh |
| `mesh_polynormal` | `(nmeshpoly, 3)`，L1331 / L1866 | 逐 polygon |
| `mesh_polyvertadr` / `mesh_polyvertnum` | `(nmeshpoly,)`，L1332-1333 / L1867-1868 | 逐 polygon |
| `mesh_polyvert` | `(nmeshpolyvert,)`，L1334 / L1869 | 逐 polygon（扁平） |
| `mesh_polymapadr` / `mesh_polymapnum` | `(nmeshvert,)`，L1335-1336 / L1870-1871 | 逐 vertex |
| `mesh_polymap` | `(nmeshpolymap,)`，L1337 / L1872 | 逐 vertex（扁平） |

设备侧用法一致（pinned `collision_gjk.py` L2245-2262/2278-2310）：`geom.mesh_polyadr` 是
逐 mesh 标量起点，`polyvertadr/polyvertnum` 按 polygon 索引，`polymapadr/polymapnum` 按
vertex 索引。

**真实 fixture（cube）实测长度**（pinned 模型，CPU probe，与 M1 `precheck-cpu.json` 一致）：
`nmesh=1`、`nmeshpoly=6`、`nmeshvert=8`、`nmeshpolyvert=24`、`nmeshpolymap=24` →
9 数组长度 `(1, 1, 6, 6, 6, 24, 8, 8, 24)`。旧规则在第 2 条长度检查即判
`InvalidArgument("mesh_poly 逐 mesh 数组长度须一致…")` → M0b 合法 bins 与 M2 都会被卡死。

## 2. 规则审计表（Chief 追加要求 2：逐条对 pinned `dim` 标注）

| # | 修正后规则（写死） | pinned 依据 | 归属 / dim | M0a 旧校验 | 结论 |
| --- | --- | --- | --- | --- | --- |
| 1 | 9 数组全空 → 不构造 sidecar、`mesh_poly_args=null` | M0a 语义层 | — | 相同 | 保持 |
| 2 | 部分空 → `InvalidArgument`（不静默禁用） | M0a 语义层 | — | 相同 | 保持 |
| 3 | `polynum.len() == polyadr.len()` | types.py L1329-1330 / L1864-1865 | 逐 mesh / `(nmesh,)` | 与其余 4 数组一起要求 `== nmesh` | **收敛**（删错误约束） |
| 4 | `polynormal.len() % 3 == 0` → `nmeshpoly` | L1331 / L1866 | 逐 polygon / `(nmeshpoly,3)` | 相同 | 保持 |
| 5 | `polyvertadr.len() == polyvertnum.len() == nmeshpoly` | L1332-1333 / L1867-1868 | 逐 polygon / `(nmeshpoly,)` | 要求 `== nmesh`（真值 6 vs 1） | **修正** |
| 6 | `polymapadr.len() == polymapnum.len()`（即 `nmeshvert`） | L1335-1336 / L1870-1871 | 逐 vertex / `(nmeshvert,)` | 要求 `== nmesh`（真值 8 vs 1） | **修正** |
| 7 | `sum(polynum) == nmeshpoly` | L1329-1331 推导 | 逐 mesh → 逐 polygon | 相同 | 保持 |
| 8 | 逐 mesh：`polyadr[i]+polynum[i] <= nmeshpoly` | L1329-1331 | 逐 mesh | 相同 | 保持 |
| 9 | 逐 polygon：`polyvertadr[i]+polyvertnum[i] <= polyvert.len()` | L1332-1334 | 逐 polygon / `(nmeshpolyvert,)` | 循环按 `0..nmesh`（只查 1/6 个 polygon） | **修正循环域** |
| 10 | 逐 vertex：`polymapadr[i]+polymapnum[i] <= polymap.len()` | L1335-1337 | 逐 vertex / `(nmeshpolymap,)` | 循环按 `0..nmesh`（只查 1/8 个 vertex） | **修正循环域** |
| 11 | 负值拒绝（adr/num ≥ 0） | int 数组 | 三段各自 | 有，但 polyvert/polymap 仅在 index 0 生效 | 保持 + 修正域 |
| 12 | 强一致 `sum(polyvertnum) == polyvert.len()` | L1333-1334 | 逐 polygon | 无 | **新增** |
| 13 | 强一致 `sum(polymapnum) == polymap.len()` | L1336-1337 | 逐 vertex | 无 | **新增** |

结论：13 条规则中 6 条（#5/#6/#9/#10/#11 的域、#3 的收敛）与 pinned 形状不符或覆盖不足，
其余 7 条（含全空/部分空语义）保持。修正后不再有「逐 mesh 等长」这类错误表述。

## 3. 红 / 绿对照（Chief 追加要求 1）

同一测试 `validate_tests::accepts_real_cube_mesh_poly_shapes`，输入为真实 cube 形状
`(1,1,6,6,6,24,8,8,24)`：

```text
# 红：把 abi.rs 经 git stash 临时退回 HEAD（旧规则）后重跑
cargo test --locked --lib runtime::collision_gpu::validate_tests::accepts_real_cube_mesh_poly_shapes -- --nocapture
→ exit 101
  left: Err(InvalidArgument("mesh_poly 逐 mesh 数组长度须一致（polynum/polyadr/polyvertadr/polyvertnum/polymapadr/polymapnum）"))
 right: Ok(())
  test result: FAILED. 0 passed; 1 failed
日志：logs/red-old-rule.log

# 绿：修正后同一套 validate_tests
cargo test --locked --lib runtime::collision_gpu::validate_tests -- --nocapture
→ exit 0，7 passed / 0 failed
日志：logs/green-new-rule.log
```

## 4. 负例覆盖（逐条有单测，全部命中指定错误消息）

① `polynormal % 3 != 0`；② `polynum/polyadr` 长度不匹配；③ `polyvertadr` 长度 != `nmeshpoly`；
④ `polymapadr/polymapnum` 长度不匹配；⑤ 逐 mesh 区间越界（`polyadr+polynum > nmeshpoly`）；
⑥ 逐 polygon 区间越界（`mesh_polyvert`）；⑦ 逐 vertex 区间越界（`mesh_polymap`）；
⑧ 负值（`polynum=-1`）；⑨ `sum(polynum)!=nmeshpoly`；⑩ `sum(polyvertnum)!=len(mesh_polyvert)`；
⑪ `sum(polymapnum)!=len(mesh_polymap)`；⑫ 部分空（9 项不齐）；⑬ 全空 → 通过（null sidecar）。

## 5. 改动清单（仅 CPU 校验层 + 注释 + 单测）

| 文件 | 改动 | 行数（`Measure-Object -Line`） |
| --- | --- | --- |
| `src/runtime/collision_gpu/abi.rs` | 重写 `validate_mesh_poly`（三段形状 + 强一致）；修正模块头与 sidecar 注释的形状语义；`MjwarpMeshPolyArgs` 9 指针逐字段注明 dim/归属 | 378 |
| `src/runtime/collision_gpu/types.rs` | `CollisionGpuInputs` 32 slice 说明 + 9 个 mesh_poly 字段注释按真实形状更正 | 94 |
| `src/runtime/collision_gpu/validate_tests.rs` | 新增真实 cube 形状正例、重建负例（含 2 条新负例：区间越界/ sum 不一致都已扩到 9 类）；`CubeMeshPoly` 用真实 `(1,1,6,6,6,24,8,8,24)` 数据 | 374 |

`git diff --stat -- src/runtime/collision_gpu/`：3 files changed, 253 insertions(+), 64 deletions(-)。

## 6. 不变量核对（Chief 追加要求 4）

| 不变量 | 核对方式 | 结果 |
| --- | --- | --- |
| `CollisionArgs` = 504 B | 单测 `abi_collision_args_matches_device_layout_504`（unit 套件内） | 通过 |
| sidecar = 72 B / 9 指针顺序 | 单测 `abi_mesh_poly_args_matches_device_sidecar_layout` | 通过 |
| entries = 6 | 单测 `all_six_entries_take_args_by_value` + 冒烟 `entries=6` | 通过 |
| 冒烟指纹 | `238969 / ec2e7acddc71eec2b8cd6b07cf70ba4a4a3e7eab258982815f2199349e1a047a / 3786520 / 6` | 与 M0a **完全一致** |
| `DEVICE_SOURCES` 27 段 | 未改 `src/runtime/collision_gpu/source.rs`、未改 device 源；`src/physics/mod.rs` 零 diff | 不变 |
| device 源 / fixtures / 判据 / 容差 / 参考 | 本片 `git status` 无 `product.cu`、`fixtures/`、`tests/` 改动 | 零改动 |

## 7. 门禁数字

见 `logs/gates-summary.txt`（含命令、exit code 与原始日志 sha）：

- fmt `cargo fmt --all -- --check` → exit 0
- clippy `cargo clippy --locked --all-targets -- -D warnings` → exit 0
- clippy-cuda `cargo clippy --locked --all-targets --features cuda-probe -- -D warnings` → exit 0
- unit `cargo test --locked` → **982 / 0 / 372**（基线 980/0/372，**+2** 新测试）
- all-features（`MJWARP_MUJOCO_ROOT` + msvc）→ **1031 / 0 / 430**（基线 1029/0/430，**+2**）
- 冒烟 → 见 §6

## 8. M0a 证据补注（Chief 追加要求 3）

- `docs/development-evidence/mesh-multicontact-m0a/README.md` §4：
  「逐 mesh 数组等长」表述已替换为真实三段形状，并加 blockquote
  **「已被修正片 `2026-10-26-mesh-multicontact-m0a-fix-mesh-poly-shapes` 更新」**，
  写明旧表述作废、真实 dim/行号、影响面（仅 CPU 校验层，ABI/sidecar/冒烟不变）与修正片后行数。
- `docs/development-evidence/mesh-multicontact-m0a/receipt.json::empty_sidecar_semantics`：
  新增 `shape_semantics`（真实三段形状 + 旧表述作废声明）、`shape_correction_task`
  （task_id / evidence / impact），并把 `unit_tests` 更新为包含新增 2 条测试；
  `validation` 指针去掉过期的 `:241` 行号。
- 不做静默改写：原「逐 mesh 等长」只在 §4 被显式标注作废。

## 9. Chief 2026-10-26 追加要求落实

1. 对照证据加强 → §3（红 exit 101 / 绿 7 passed，原始日志入 `logs/`）；负例覆盖 §4（13 类）。
2. 一次性规则审计 → §2 逐条表（pinned `dim` + 行号 + 归属 + 旧/新结论）。
3. M0a 证据补注 → §8（README §4 + receipt，均标注「已被修正片更新」）。
4. 零改动确认 → §6（ABI/sidecar/entries/冒烟/DEVICE_SOURCES/device 源/fixtures 全核对）。

## 10. 影响面 / 未覆盖

- 影响面：**仅 CPU 输入校验层 + 注释 + 单测**；不改 ABI、不改 device 内核、不改上传链、
  不改 fixtures、不改判据/容差/参考。
- 未覆盖：GPU 发射级行为（null / 非 null sidecar）仍留待 M2 验证窗；本片不跑 GPU。
- 本片不宣称 mesh 已支持；M0b 需在本片 pass 后继续（bins 产出将使用修正后的校验）。
