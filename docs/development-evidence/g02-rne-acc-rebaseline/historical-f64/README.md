# 历史 native MuJoCo 3.12 f64 参考（已归档，不再作为期望）

本目录保存 `fixtures/rne-acceleration` 重基线前的原始内容，仅作历史证据与差异复核用途，**不是任何测试的期望值**，产品构建、脚本与测试都不读取本目录。

## 来源

- 作者工具：`fixtures/rne-acceleration/reference.cpp`（离线 C++ 工具，`mj_rne` 原生调用；产品链从不执行）。
- 原生库：MuJoCo 3.12.0，DLL SHA256 `79b61d22b4d230a00bd31930fc6943f8ea8c6174a1bf4a6bf90f2459dbf8df1b`。
- 作者可执行 SHA256：`722a17b6e1120f32fd7b05d0678ee6753d3a79c9f9b498f709fc3c7bd8109660`。
- Manifest 版本：`native_version=3012000`、`absolute_tolerance=relative_tolerance=2e-5`（这些口径现仍保留在 `fixtures/rne-acceleration/manifest.json`，未改动）。

## 为什么归档

用户 2026-10-23 裁决 A：RNE 加速度 521 块 native-tolerance 失败经离线 authoring 定位后，确认是「冻结原生 f64 参考 vs MJWarp f32 累加」的参考口径差异（失败块产品 qfrc 位串与冻结上游 MJWarp `71da24d…` 逐位相同，上游自身对同一原生参考也超差 1/240 条）。因此期望值改为冻结上游 MJWarp f32 参考（`docs/development-evidence/g02-rne-acc-rebaseline/`），容差、断言、产品代码一律不动；本目录的旧值保留以便复核差异，不得再被用作准入期望。

## 文件与 SHA256

| 文件 | SHA256（重基线前原始值） |
| --- | --- |
| `mixed-joints.json` | `0bb8e995a81bc9fbe9814aceb9b63ef803a2219ad1506229faf1d119c91adf2f` |
| `inertial-tree.json` | `f359787ee824ff490c0c1169796145c64cb408c98acb5a0842061498120d649e` |
| `rotated-tree.json` | `66abdf7a21ac0ef3915bd551b235014d8d9d30c86c6d03aa3de3a40280f4b3dc` |
| `zero-dof.json` | `92de890f4c4a8258189c1ac56ea48820d7c5192063baa3ff9e15c38065b239bd` |
| `massless-tree.json` | `c6bb013ab5059fb1e725579bec3553a9f5bb5f0eea86425e0fdb17396190fb4e` |

注：`zero-dof` 与 `massless-tree` 的 `nv=0`，qfrc 全为空数组，重基线前后字节一致，仅 `mixed-joints`、`inertial-tree`、`rotated-tree` 三个文件的 qfrc 期望值有变化。

## 复核方式

```powershell
# 新期望（冻结上游 MJWarp f32 位串）→ 见 fixtures/rne-acceleration/*.json
# 旧期望（本目录）→ 逐 case 对比 qfrc；差异应只出现在 qfrc 字段，输入与容差不变
```

重基线生成与差异清单见 `docs/development-evidence/g02-rne-acc-rebaseline/rebaseline.json` 与同目录 `README.md`。
