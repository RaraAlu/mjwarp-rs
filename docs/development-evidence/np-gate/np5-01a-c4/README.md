# NP5-01a C-4：C 批 pose/batch/sleep 63 站点 source 级逐站点对照（零 GPU）

- 任务：`2026-11-11-np5-01a-c4-pose-batch-sleep-source-level`（owner Backend；p95；P8-渲染与BVH）。
- 依据：`np5-01a-SLICE-PLAN.md` C-4 口径；站点 = `np5-01a/sites.json` 的 4 个 pose/batch/sleep 行（63 站点）；
  词表 `np5-01a-vocab-addendum.md`（`453c5c91…`/6771 B）。
- 本卡第一节处置 C-3 复核的 1 条低项 + 1 条观察（§4）。
- 边界：source-level、零 GPU、零窗；不改 `src/`/`tests/`/`fixtures/`/判据/参考/容差。

## 1. 站点与分类计数（63 = 4 组；NP5-01a 累计 380 + 63 = 443）

| 组 | 站点 | 构成 | 说明 |
| --- | --- | --- | --- |
| `camlight` | 17 | intermediate 6 / output 8 / rounding-note 3 | 相机/灯光位姿（rigid + COM 两段，look_at 与矩阵行展开） |
| `field_batch` | 17 | accumulation 2 / intermediate 7 / output 5 / rounding-note 3 | 参数批描述符取数（设备助手）+ 宿主打包（拓扑/肌腱专段/参数槽） |
| `sleep_wake` | 15 | accumulation 1 / intermediate 7 / output 4 / rounding-note 3 | 睡眠状态位型拷贝 + 肌腱唤醒（树链写 wakeval、长度边界比较） |
| `attached` | 14 | intermediate 4 / output 7 / rounding-note 3 | geom/site 附着位姿（device 库被两个 __global__ 共用，MOCAP 初始化分支） |

分类计数：**{'identical': 21, 'n/a': 42}**
（逐站点明细见 `evidence.json`；不写覆盖类/通过类统计口径）。

## 2. 方法与判据

- 产品锚点：`src/physics/camlight.rs`（`camlight_rigid`/`camlight_com`）、
  `src/physics/resident/pack_batches.rs`（设备取数助手 `field_parameter` + Rust 宿主打包）、
  `src/physics/sleep.rs`（`sleep_copy`/`tendon_wake`）、`src/physics/attached.rs`（device 库 `attached_frames`
  被 `attached_kinematics`/`initialize_attached` 共用）；`product_anchor` 为 repo 行号 + 表达式原文。
- 参考锚点（全部带 file:line）：`smooth.py` `_cam_local_to_global`（831）/
  `_light_local_to_global`（896）/`_geom_local_to_global`（180）/`_site_local_to_global`（210），
  `sleep.py` `_wake_tendon_kernel`（417）/`_tendon_limit_active`（58）/`_wake_tree`（236），
  `io.py` `put_model`（288，参数批/pulley scale 打包）。
- 分类六类词表：`identical` **21**（source 级表达式形态与次序对应；不声明位级）、
  `n/a` **42**（指针/描述符/整型游标/状态码/位型拷贝/布局说明/局部量/prose）、
  `engine-semantic` 0、`recipe-difference` 0、`not-decidable` 0、`pin-needed` 0。
- 核心对照：camlight 的 `xpos + rot_vec_quat(local, xquat)`、look_at 的 z/x/y 构造与矩阵行展开、COM 段
  `subtree_com + offset` 与 `normalize(target - lx)`；attached 的同一 site/geom 位姿式与
  `quat_to_mat(mul_quat(...))`、MOCAP 初始化静态跳过；sleep 的 `(length-low)`/`(high-length)`
  两边界比较与 i32 位型拷贝；field_batch 的描述符取数与宿主打包（整型游标、memcpy、索引映射）。
- **唯一 f64 接触点（记录）**：`field_batch` 的 wrap-pulley scale `1.0/wrap_prm` 在 Rust f64 计算后写
  f32 参数槽；上游 `io.py put_model` 同样在 numpy float64 计算后存 f32 warp 数组——**float64→f32 路径对称**，
  不是产品单方面升精度（`double_in_product:true` + `double_precision_parity` 字段说明；本片 1 条）。
- `not-decidable` 未使用：63 站点均有 source 锚点或明确的非表达式/指针/整型点；设备级映射留给 RE-3。

## 3. 自检

- 63/63 站点与 `sites.json` 的 id/kind/label 逐条一致；4 组计数 = 17/17/15/14。
- `pending_manual=0`；分类计数与逐条目重算一致；`not_decidable_reasons` 词表未被使用（0 条）。
- `policy_scan`（`scripts/np-gate/policy_scan.py`，`21a895b6…`/16532 B）
  对本目录 `evidence.json`/`README.md`/`receipt.json` 命中 0；工具 `--self-test` **34/34 PASS**
  （含 veto 块否定窗口禁用用例）。
- 编码 UTF-8 / LF / 无 BOM；manifest 逐文件 sha256+bytes 自核；详见 `logs/selfcheck.txt`。

## 4. 第一节：C-3 复核低项/观察处置

- **L1（veto 与 18 字符否定窗口交互）**：块级 veto 生效后，块内断言若紧邻否定声明仍会被否定窗口豁免
  （`withdrawn: never supersede` + 紧邻断言曾 0 命中）。已修：**被 veto 的闭合引用块内禁用 18 字符否定
  窗口**——只有块级肯定声明能清除块内断言；否定窗口只用于块外普通文本。新增 3 条构造自测（紧邻断言必中、
  肯定声明块+普通否定句不误伤、veto 块内否定句亦命中=保守方向），自测 31→**34/34 PASS**；
  工具新版 `21a895b6…`/16532 B，docstring 已写明该交互。
- **观察（`__pycache__` 污染）**：QA 复核用 importlib 导入工具时产生 `scripts/np-gate/__pycache__/`（QA 已清理并
  补提交 `cf10ed6f`，仓文件无改动）。根 `.gitignore` 现加入 `__pycache__/` 与 `*.pyc`，后续运行
  `scripts/np-gate/*.py` 不再污染工作区（本卡允许的小范围仓库卫生改动，仓内文件零改动）。
- **C-2 O2（errata-0001）**：保持转发状态（forwarded to closing addenda errata list；不改 field-table 历史）。

## 5. 输入与哈希

- 站点清单 `target/render-bvh/gl2-ptx-diff/np5-01a/sites.json` = `e0d4a95bf5c0acf13510b20f51e92ece1ec63ab7c7ac9e37be38cdd0841bac3c` / 261483 B。
- 场表（4 行，`docs/development-evidence/np01-type-mapping/field-tables/`）：
  - `camlight.json` `e4e25da5c60dda6d502a219e35323bdb33716911b3673b65a46cfea7f8a4626f`
  - `field_batch.json` `5d2b7d164203c7d5c7c9890e5f055cedd7cb8c25d9ad909bb8b36ee089d656f7`
  - `sleep_wake.json` `59fdf33b1da9451df652bb979513f5287ab5ba756d8931f5e4084a20e3a26422`
  - `attached.json` `fe32eacb274a1cfd3f7e99090eec9c789ff9d698a7a557d0d061117edc2252f8`
- spec `target/render-bvh/gl2-ptx-diff/np5-01a-exec-spec.md` = `5321f31258ca9ce02f344dd38c1d1d32ad2005fc1cd66b27f21fe1c8b86c5389` / 12402 B（工作材料，非证据）。
- 词表 `target/render-bvh/gl2-ptx-diff/np5-01a-vocab-addendum.md` = `453c5c9157341fa28ecb009c50f2ff560b6f43e15f593ed9b7a75f27dd5b1b26` / 6771 B（工作材料，非证据）。
- `.gitignore` = `5013cb14ee0345704d9703013be02bb55fb0b7d7f9d2e01a5bd38295d596b4e4` / 269 B（新增 `__pycache__/`、`*.pyc`）。
- upstream revision `71da24d956378a87a703b6e1442b13aec0c4ac29`；`tree.head` `cf10ed6f5f46e9e1cb591a72cd2023d1dafc1ecf`（本卡主提交父提交）。

## 6. 非声明

不宣称 NP01 通过/E 指标完成/上游等价/双平台/位级保证；不改判据/参考/容差；零 GPU、零窗；
pose/batch/sleep 设备级映射不在本片范围（RE-3 后续）。
