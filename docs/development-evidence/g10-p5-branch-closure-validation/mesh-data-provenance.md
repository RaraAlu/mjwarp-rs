# S4 · mesh 拓扑常量（U6/U7/U8）可复现材料

- 关联任务：`2026-10-27-unc3-branch-closure-s4-window`（S4 验证窗执行与结果回填；开窗前材料）
- 目标常量文件：`tests/support/branch_closure_mesh_data.rs`（418 行 / 17023 B /
  sha256 `2072f4acb9defdd8c69bbf52c327d131ddc514c89b868f0a9778e7aa773ab4e2`，与 `b3e56a5f` 提交一致、**本轮未改动**）
- 生成脚本（已入库）：`scripts/authoring/g10-p5/author_branch_closure_mesh_data.py`
  （279 行 / 12712 B / sha256 `7a4be54d4889e77111b9e000a5305447c1f8abb78e899cd7eb99762178bf319c`）
- 本轮重跑（2026-10-27 开窗前，未跑 cargo/rustc）：**7/7 输入 sha 匹配、42/42 数组、`FULL_FILE_MATCH=True`、`RESULT=PASS`、exit 0**
- 原始重跑输出：`logs/mesh-data-check.txt`（6722 B / sha256 `bf02a73fb32270ca0e96e55841335864ba8ea99a4e50c0e266b9407d18234566`）

本材料只说明「常量如何从只读 fixture 重建」，不改变任何产品路径、判据、容差、参考或期望值；
产品测试仍然**零 Python**（常量已编译进测试 target）。

## 1. 需要 pin 的环境

| 项 | 值 | 说明 |
| --- | --- | --- |
| Python | 3.12.13 | `target/mjwarp-parallel/venv/Scripts/python.exe` |
| mujoco | **3.15.0** | 数组来自 `mjModel` 的**编译结果**（hull / polygon 划分与顺序由 MuJoCo 编译器产生），换版本可能改变顺序；脚本对版本不符直接 FAIL（`--allow-mujoco-version-mismatch` 仅供非 pin 环境排查，会 WARN） |
| numpy | 2.5.4 | 仅用于 f32/i32 的位精确处理与最短十进制输出 |

## 2. 生成命令

在仓库根目录（`D:\Rust\mjwarp-rs`）执行：

```text
$PY = target/mjwarp-parallel/venv/Scripts/python.exe

# 复核（默认动作）：校验 7 份输入 sha -> 重建整份 Rust 源 -> 与入库文件逐字节比较 -> 打印数组表
& $PY scripts/authoring/g10-p5/author_branch_closure_mesh_data.py --check --table

# 重生成（会覆盖目标文件；重生成后应与入库文件逐字节一致）
& $PY scripts/authoring/g10-p5/author_branch_closure_mesh_data.py --emit tests/support/branch_closure_mesh_data.rs

# 只打印 §4 的数组 sha 表
& $PY scripts/authoring/g10-p5/author_branch_closure_mesh_data.py --table
```

`--check` 的 PASS 判据：7 份输入 sha 全部命中 **且** 生成文本与 `tests/support/branch_closure_mesh_data.rs`
逐字节相同（含头注释与 `MeshData` 挂接段），退出码 0。

## 3. 输入 fixture（7 份，只读，S3 落盘，本片未新造）

| 路径 | bytes | sha256 | 角色 |
| --- | --- | --- | --- |
| `fixtures/g10-p5-mesh-multicontact/mesh-mesh-two-assets.xml` | 609 | `1d71151364f4ba910308c80a42817f7a9b7fe843e42afb8396a3b5277f1b3bfb` | U6 模型（cube + tetra 两个 mesh） |
| `fixtures/g10-p5-mesh-multicontact/cube.obj` | 570 | `90a8a990ae17a9265c28ada66788d2f523e15653a1419a28b8074c6edf17603d` | U6 mesh 资产 |
| `fixtures/g10-p5-primitive/tetra.obj` | 155 | `051e75316a32b22094343d249d1b4c5a08af1659f537ab40d9f263fe79a3fe0a` | U6 mesh 资产（XML 内相对路径 `../g10-p5-primitive/tetra.obj`） |
| `fixtures/g10-p5-mesh-multicontact/mesh-limits-cone-box.xml` | 569 | `6ce3552f2cab48fdbbb66a6d397790d18748dcb0a5a87b91186b5742e297f731` | U7 模型（fan-cone-24） |
| `fixtures/g10-p5-mesh-multicontact/fan-cone-24.obj` | 1042 | `4bc34a51a60d76469a68c64e6132cfd1f5713072488ea8deefd44a4ca517f73b` | U7 mesh 资产 |
| `fixtures/g10-p5-mesh-multicontact/mesh-limits-prism-box.xml` | 567 | `1bee35cfa3378d96d18317ff75bbf4af0561f7bd71915574e2ea596bc8d4f3b5` | U8 模型（prism-20） |
| `fixtures/g10-p5-mesh-multicontact/prism-20.obj` | 1819 | `a4477d3b080479e7356ac14bcd67c33272b2771eadd40b14fe3f40360e92cee4` | U8 mesh 资产 |

其中 5 份同时被窗口 runner 的 13 个 fixture sha 守卫覆盖（3 个 XML + `fan-cone-24.obj` + `prism-20.obj`）；
`cube.obj` / `tetra.obj` 只在编译 fixture 时被 MuJoCo 通过相对路径间接读取，不进入设备 target 的
运行时输入，因此由本脚本守卫、不在 runner 守卫内。

## 4. 输出：42 组数组 sha256（f32/i32 小端原始值）

编码定义：`f32` 取 IEEE-754 单精度 4 字节小端原始位；`i32` 取 4 字节小端；每组 `sha256` 只覆盖原始字节。
组合摘要 = `sha256("NAME|type|count|" + raw ...)`，本轮为
`22ac2c3e816e98baf50b912bac3487a582687d608e16b4eafceb521f9e130e76`；整份生成文件 sha256 =
`2072f4acb9defdd8c69bbf52c327d131ddc514c89b868f0a9778e7aa773ab4e2`（17023 B）。

| group | const | count | bytes | sha256 (LE raw values) |
| --- | --- | --- | --- | --- |
| U6 | `U6_VERT` | 36 | 144 | `3fa734040c10d9bc32f5f84e4d7a85b6762750eb4782aad15153349c7d740b0a` |
| U6 | `U6_GRAPH` | 136 | 544 | `b1cbf9839eea3d9b21fb469dd086556f1d0013e8fb6c04fa60c99da3a2326e7b` |
| U6 | `U6_VERTADR` | 2 | 8 | `7b742c398b1a841a160d67298c4e11857acc1db71c3f90509722ca74733cb814` |
| U6 | `U6_VERTNUM` | 2 | 8 | `cc64e39e1d38c7bf38b8fccf98e0f9c424fd65bd40a7d3926b20ca79a9239df3` |
| U6 | `U6_GRAPHADR` | 2 | 8 | `1b7a98355e788b3d4e39b09ce2c38ffd930fb5abbaa1e5440eb981b19c58518b` |
| U6 | `U6_POLYNUM` | 2 | 8 | `4cf89882b9b9f07505dd83f21a06dfd15c4e0b85fae7a4bc2b8f31d50e95d05a` |
| U6 | `U6_POLYADR` | 2 | 8 | `20aef0af2bb62267e2f5c157abdc51d1ce6506e42478ae204f1fab5494907df1` |
| U6 | `U6_POLYNORMAL` | 30 | 120 | `35f0277c9e64bcaad221e8f62008429491623155fa020237a13f2bca0a18d05a` |
| U6 | `U6_POLYVERTADR` | 10 | 40 | `50e1e830fae52b4274d239c9b116b93c8c2bc0854e2bbcb7a22a8423c7ec1a43` |
| U6 | `U6_POLYVERTNUM` | 10 | 40 | `ff89dc117a1237461580d16f37bb4297d5af8059aea1927e9196707afd890be5` |
| U6 | `U6_POLYVERT` | 36 | 144 | `7ed5515a7ef4bfbf9074dc9ff24e7ea18f5a29866ed59504843bc688ffbcbfe5` |
| U6 | `U6_POLYMAPADR` | 12 | 48 | `090fa035e07deccdee6bbe47a516d7e2be3993bf64d889656b7176f106677e29` |
| U6 | `U6_POLYMAPNUM` | 12 | 48 | `b886a876a05d91284ccc843786c3e1d18cac3eb22d804e3d6ed8d4f055e2b769` |
| U6 | `U6_POLYMAP` | 36 | 144 | `1748f363111899a66b2441234f311126cccdb3f3118914b1a6496b94145d36e1` |
| U7 | `U7_VERT` | 75 | 300 | `898ab780b62d238a284951f912c2ea553160cdecca1e823ed64f618f6b821026` |
| U7 | `U7_GRAPH` | 353 | 1412 | `6c7a10531a0a3746b3479211000033f052af8b273ab9e7570a745a3c03759b36` |
| U7 | `U7_VERTADR` | 1 | 4 | `df3f619804a92fdb4057192dc43dd748ea778adc52bc498ce80524c014b81119` |
| U7 | `U7_VERTNUM` | 1 | 4 | `0623ccb9b1619bd388284a438034d8cb6431964ba727d8b1c450303105735488` |
| U7 | `U7_GRAPHADR` | 1 | 4 | `df3f619804a92fdb4057192dc43dd748ea778adc52bc498ce80524c014b81119` |
| U7 | `U7_POLYNUM` | 1 | 4 | `0623ccb9b1619bd388284a438034d8cb6431964ba727d8b1c450303105735488` |
| U7 | `U7_POLYADR` | 1 | 4 | `df3f619804a92fdb4057192dc43dd748ea778adc52bc498ce80524c014b81119` |
| U7 | `U7_POLYNORMAL` | 75 | 300 | `a3034267089f77539916ddba47e0a4bca4b0ef3c25cee2cb940265ace9f960d2` |
| U7 | `U7_POLYVERTADR` | 25 | 100 | `26a50ee47e19d79a9d9466d32ae6661401155fd34f0195da11f7b95d14fdeb38` |
| U7 | `U7_POLYVERTNUM` | 25 | 100 | `e2b3811cc59427b53de137cfde821da44a2eb4063307ef8d5c5defd5dd4fcc64` |
| U7 | `U7_POLYVERT` | 96 | 384 | `af7fdf1374d5b245b79efcea3e4d729ab9e7a60f0bd9a065a6b2d056e5ffcf97` |
| U7 | `U7_POLYMAPADR` | 25 | 100 | `26a50ee47e19d79a9d9466d32ae6661401155fd34f0195da11f7b95d14fdeb38` |
| U7 | `U7_POLYMAPNUM` | 25 | 100 | `e2b3811cc59427b53de137cfde821da44a2eb4063307ef8d5c5defd5dd4fcc64` |
| U7 | `U7_POLYMAP` | 96 | 384 | `0f943dbc8daeff7c9cc7a3685db2a2c2e7afcc00a4250d99504659f616dae63f` |
| U8 | `U8_VERT` | 120 | 480 | `8fe8e86a32061460cabd9c5163d35b58f04ca2ca347cf2f99dc9fedb9744b648` |
| U8 | `U8_GRAPH` | 578 | 2312 | `caebf6722a6d35f7792ac1b52523c8b615ab531d7668c8401895739f0d87293a` |
| U8 | `U8_VERTADR` | 1 | 4 | `df3f619804a92fdb4057192dc43dd748ea778adc52bc498ce80524c014b81119` |
| U8 | `U8_VERTNUM` | 1 | 4 | `1a00f51f3029fa4b7e61b9bf7aa9de5a64798857872981f7e056e4f437171955` |
| U8 | `U8_GRAPHADR` | 1 | 4 | `df3f619804a92fdb4057192dc43dd748ea778adc52bc498ce80524c014b81119` |
| U8 | `U8_POLYNUM` | 1 | 4 | `f59d756696b2089fca5969fdfde2674e333765e1dad0a51c8ebbc63f351c4ca5` |
| U8 | `U8_POLYADR` | 1 | 4 | `df3f619804a92fdb4057192dc43dd748ea778adc52bc498ce80524c014b81119` |
| U8 | `U8_POLYNORMAL` | 66 | 264 | `99429d66d34e3d271d208aa9321e5bb86a42ac85a7a3527d4daa86f7f52d4ffc` |
| U8 | `U8_POLYVERTADR` | 22 | 88 | `a7212e634e64792a22ac382a0cf08e9058a8895858febe4d03dd060169ab5b24` |
| U8 | `U8_POLYVERTNUM` | 22 | 88 | `59306de5d7a4830c6ece1f8e1f841dde102bb4bd7bbcbe21182cca556cdee289` |
| U8 | `U8_POLYVERT` | 120 | 480 | `9b25f35dde6de077f9a82b8e32a998c18822fa5a0e50121f7a616d5589177332` |
| U8 | `U8_POLYMAPADR` | 40 | 160 | `c48b18000df0ecf700797ae7cc9b0857cf6c58f6850e6b8c0d0e4e1fd8309d4c` |
| U8 | `U8_POLYMAPNUM` | 40 | 160 | `a16b6e3eff2b38fa5f9f8f8384bfe035debc35e1d056ccf335c87f8a252dd58c` |
| U8 | `U8_POLYMAP` | 120 | 480 | `af00dca76ea4460b77f49703d256ef3b998f7036d690858d0180c77021106aa8` |

结构说明：14 个数组的名字与顺序与 `MeshData` 字段顺序一致（`vert/graph/vertadr/vertnum/graphadr/
polynum/polyadr/polynormal/polyvertadr/polyvertnum/polyvert/polymapadr/polymapnum/polymap`）。
U6 = 2 个 mesh（cube 8 verts / 6 polys + tetra 4 verts / 4 polys），`vertadr`…`polyadr` 等 nmesh 级数组长度 2；
U7/U8 各 1 个 mesh（fan-cone-24：25 verts / 25 polys；prism-20：40 verts / 22 polys，其中最后 2 个
polygon 为 20 顶点面——U8 的 device `FACE_MAX=16` 回退语义见 README §1.1，属 record-only）。

## 5. 他人独立自查

1. 用 §1 的 pinned 环境在仓库根目录执行 §2 第一条命令（`--check --table`）。
   若工作树来自 `core.autocrlf=true` 的 Windows 克隆（文本文件为 CRLF），脚本会打印
   `CHECK normalized=CRLF-to-LF` 并仍然 PASS；**常量字节本身不做任何归一化**。
2. 期望关键行：`INPUT OK` ×7；`ARRAYS groups=3 fields_per_group=14 total=42
   combined_sha256=22ac2c3e816e98baf50b912bac3487a582687d608e16b4eafceb521f9e130e76`；
   `FULL_FILE_MATCH=True`；`RESULT=PASS`；退出码 0。
3. 逐组数组可与 §4 表比对；整份生成文件 sha256 应为 `2072f4ac…`。
4. QA 预窗独立重建（Arch 于 2026-10-27 转述）：用 pinned mujoco 独立重建 42 组数组并逐位比对，
   0 不符；本材料保留命令与 sha 以便第三方复算。

## 6. 边界与口径（不宣称项）

- **authoring-only**：不进产品 crate、不进测试 target 的编译/运行路径；产品测试仍**零 Python**。
- 不改 `tests/support/branch_closure_mesh_data.rs`（本轮 sha 与 `b3e56a5f` 一致）、不改产品代码 / 设备源 /
  fixtures / 判据 / 容差 / 参考 / 期望值。
- 验证窗冻结期**未跑 cargo/rustc**；本脚本只用 CPU + pinned mujoco（不占用 GPU 窗口、不触碰冻结 exe）。
- 常量文件头注释保留 `b3e56a5f` 时的措辞（其中一句写「生成脚本未入库」）：本窗口任务 `files` 不含
  `tests/`，故本轮**未回写该行**；脚本以原头注释重生成，`FULL_FILE_MATCH=True` 即以此为条件。
  若后续要改该注释，需另开/扩任务后重跑本脚本。
- U7/U8 仍为 **record-only**，不进 pass/fail 分母；措辞口径见 README §1.1（U8「行为差异 / 限制存在」，
  U7「与 degree>10 限制语义一致」）。本材料只覆盖 mesh 拓扑常量的来源，不声称任何分支「已覆盖」。

## 7. 落盘记录

| 项 | 值 |
| --- | --- |
| 首次落盘 | 2026-10-27（S4 验证窗开窗前；窗口任务 `…-s4-window`） |
| 脚本 sha256 | `7a4be54d4889e77111b9e000a5305447c1f8abb78e899cd7eb99762178bf319c`（12712 B / 279 行） |
| 目标常量文件 sha256 | `2072f4acb9defdd8c69bbf52c327d131ddc514c89b868f0a9778e7aa773ab4e2`（17023 B / 418 行） |
| 重跑原始输出 | `logs/mesh-data-check.txt`（6722 B / sha256 `bf02a73fb32270ca0e96e55841335864ba8ea99a4e50c0e266b9407d18234566`） |
| 组合数组摘要 | `22ac2c3e816e98baf50b912bac3487a582687d608e16b4eafceb521f9e130e76` |
