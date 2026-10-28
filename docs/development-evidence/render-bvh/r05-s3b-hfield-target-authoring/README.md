# R05-S3b hfield 目标实体 · 离线 authoring（GPU 实体未执行）

任务：`2026-10-28-render-bvh-r05-s3b-hfield-target-authoring`（P8 渲染与 BVH；owner Backend；in_progress）
阶段：**离线 authoring 完成；GPU 实体未执行、无窗口、零渲染数值结论**。

本片在 S3b 验证窗之前把目标实体与回归实体材料全部离线写好，窗内只跑实体+比对，不再写代码：

- 新目标实体：`tests/render_hfield_reference.rs`（host 结构自检 + 两个 `#[ignore]` GPU 实体）；
- 共享辅助：`tests/support/render_bvh/hfield.rs`（读 S3a 捕获参考并校验内部自洽）、
  `tests/support/render_bvh/hfield_scene.rs`（产品场景装配 + 判据断言）；
- 新增产品模型 dump（3.12）：`tests/support/render_bvh/hfield/model-312.json`、`model-mixed-312.json`；
- `tests/support/render_bvh/shadow.rs` 仅放宽 8 个 npz 解析函数的可见性（`pub(super)`，零行为/零断言改动）；
- 窗口材料骨架：`window/entity-table.json`、`window/guards-skeleton.json`（只骨架，不实跑）。

参考数据已由 `2026-10-28-render-bvh-r05-s3a-hfield-capture` 采集窗口入库
（`fixtures/render-bvh/scene-hfield/capture-hfield/{hfield-only,hfield-primitive}/` 8 文件 / 5 784 264 B）。

## 1. 文件与行数（≤500 行）

| 文件 | 状态 | 字节 | 行数 | sha256 |
| --- | --- | --- | --- | --- |
| `tests/render_hfield_reference.rs` | 新增 | 14 568 | 393 | `57dc06252db831be2297748afc45be00b9326942a641dc33ec04c2f677278620` |
| `tests/support/render_bvh/hfield.rs` | 新增 | 12 879 | 355 | `961817f1a495b29c3fd048e594ee3598b5aa884c748f950704a8a18ef3a3da7e` |
| `tests/support/render_bvh/hfield_scene.rs` | 新增 | 17 389 | 500 | `a935d039f953b49a94a742b20151b0d62d03c17cf0924ce229638394daf0ff44` |
| `tests/support/render_bvh/hfield/model-312.json` | 新增（生成数据） | 52 527 | 145 | `bc53747492121d76580d0f19b2062ff40676443292d8907c7964f115798ac321` |
| `tests/support/render_bvh/hfield/model-mixed-312.json` | 新增（生成数据） | 53 207 | 145 | `c9a886ae8714b5a977a6bcb570960396b4cad2aa9e4c947dc26eef7e1ecac801` |
| `tests/support/render_bvh/shadow.rs` | 修改（+8/−8 行，仅可见性） | 16 593 | 423 | `c47aa4b05b6976a4ff1015be1d4dffd787a1affb4f40b8dafb02aff335a2d2d9` |

行数口径：表内为文本行数（含末尾空行；与 Arch 终稿实测一致）；`wc -l`（LF 数）为 392 / 354 / 499。
`hfield_scene.rs` 实测 **500 行**（≤500 上限），**高于本片预核 300–340 行**（场景装配 + 判据断言 +
哈希/打印点全部实装所致），如实记录、不上调上限；终稿记录后不再加行。

`shadow.rs` 的 8 处改动仅把 `quantize_channel`/`sha256_hex`/`read_npz`/`parse_npy`/`f32_member`/
`i32_member`/`f32_member_rgb`/`member` 由私有改为 `pub(super)`，供 `hfield.rs` 复用最小 npz 解析器；
**零行为、零断言、零容差改动**。

## 2. host 自检（无 GPU；普通 `cargo test` 可跑）

1. **三角化结构交叉核对**（`hfield_fixture_triangulation_matches_s3a_reverse_calc`）：用 R05-S3a
   证据 `materials/reverse-calc.txt` [1] 的 16 个 raw f32 位型（该文件 sha256
   `3f404550bbc6acd5d3aaf8720dfa2490699e62c1fd865e2c619a816cbadc7ad5`）作输入，经
   `build_hfield_mesh` 得到 **13 顶点 / 36 索引 / 12 三角形**、`half=[1.5, 1.0, 0.125]`；
   `size=[1.5,1.0,0.25,0.1]` 口径与 `data` 位型抽查一致。**这是结构交叉核对，不是渲染验收结论。**
   实测输出：`r05-s3b[host] hfield mesh: vertices=13 indices=36 triangles=12 half=[1.5, 1.0, 0.125]`
2. **模型 dump 语义核对**（`model_dumps_match_signed_scene_semantics`）：两组 dump 版本 `3.12.0`；
   geom 类型/数量（hfield-only 1 个 hfield；mixed 再加 1 个 box）、hfield `geom_dataid=0`、
   box `geom_dataid=-1` 与半尺寸 0.14；相机 2、灯光 2 且 `castshadow=[1,1]`/`active=[1,1]`；
   内置 checker 纹理 `tex_data` 12 288 B；`resolve_context` 主机解析得到 2 相机、分辨率
   `[320,240]`/`[240,180]`，且不隐式注入 hfield（注入只走 `set_hfield`）。
3. **参考解析与自洽**（`capture_references_parse_and_cross_checks_hold`）：读两组 S3a 捕获参考，
   断言 npz sha 与 `capture.json.files_sha256` 一致、task/group/captured_at、A/B 位等、
   旧参考未动、recipe 场景/脚本 sha、fast-math/shadows 4 处口径、相机两路分辨率与 npz 形状自洽；
   捕获参考缺失时打印并跳过（普通 `cargo test` 不 panic，且不产生数值结论）。
   实测：两组 `pixels=120000 depth_zero_pixels=66826 captured_at=2026-10-29T00:00:00+08:00`。

以上只做结构/自洽/模型语义核对，**不含任何 rgb/depth/seg 数值比较**。

## 3. 两个 GPU 实体（`#[ignore]`，只在 S3b 验证窗内跑）

| 实体 | 组 | 命令 |
| --- | --- | --- |
| `hfield_only_matches_captured_reference` | hfield-only | `cargo test --locked --features cuda-probe --test render_hfield_reference -- --ignored --test-threads=1` |
| `hfield_primitive_matches_captured_reference` | hfield-primitive | 同上（同一 target） |

- 场景装配：`resolve_context`（spp=1、`use_fast_math=false`、`use_shadows=true`、几何组 0..3、
  三输出）→ `ResolvedContext::set_hfield` → `SceneState`（hfield leaf `size = build_hfield_mesh().half()`，
  S2 O5；mixed 组 box leaf 用 `geom_size`）→ `RenderContext::create` → `RayBuilder` 预计算射线。
- 判据（沿用 R09-S3b，不新增/不放宽）：RGB 每通道 ≤1 LSB（超判据像素数须 0）、depth 位级优先
  （`mismatched==0`）且 `max_ulp<=4`、`beyond_ulp==0`、seg 逐像素全等、两次渲染 sha256 相同。
- 打印点（超判据时同样打印）：每相机 RGB `compared/mismatched/max_byte_diff/beyond_1_lsb`；
  每相机 depth `pixels/depth_mismatch/max_ulp/beyond_ulp/seg_mismatch/depth_zero_pixels`；
  两次运行 `rgb/depth/seg` sha256。
- 停手条件（不改容差/参考/fixture）：任一相机任一产物超判据、`leaf_size` 差异导致的可疑 tie-break、
  出现镜像/转置形 hfield 差异 → 立即停手报 Arch。

## 4. 产品模型 dump provenance（3.12）

| 项 | 值 |
| --- | --- |
| 工具 | `target/render-bvh/s0a/compile_scene_312.exe`（S0a 已验收 dumper），sha256 `cd91e099659ebda3f51a84d5392f7d726c0c61378bfa2487215ddd00b2de2970`，161 280 B |
| 输入 1 | `fixtures/render-bvh/scene-hfield/scene.xml` sha `b3b9a6b4…`（签字值） |
| 输入 2 | `fixtures/render-bvh/scene-hfield/scene-mixed.xml` sha `bf9bb966…`（签字值） |
| 输出 1 | `tests/support/render_bvh/hfield/model-312.json` 52 527 B / `bc537474…` |
| 输出 2 | `tests/support/render_bvh/hfield/model-mixed-312.json` 53 207 B / `c9a886ae…` |
| 命令 1 | `target/render-bvh/s0a/compile_scene_312.exe fixtures/render-bvh/scene-hfield/scene.xml target/render-bvh/s3b-model-probe/scene-hfield.mjb tests/support/render_bvh/hfield/model-312.json` |
| 命令 2 | `target/render-bvh/s0a/compile_scene_312.exe fixtures/render-bvh/scene-hfield/scene-mixed.xml target/render-bvh/s3b-model-probe/scene-hfield-mixed.mjb tests/support/render_bvh/hfield/model-mixed-312.json` |
| 确定性 | 同输入两次 dump 的 `arrays`/`counts` 完全相等（dump 内 `model_file` 字段记录输出路径，仅该字段随路径变化） |
| 落点说明 | 放 `tests/support/render_bvh/hfield/`（authoring 卡 files 覆盖），不动 `fixtures/`（S3a 采集卡运行期间该前缀有写入范围冲突，Arch 裁决 A） |

dump 的字段表不含 `hfield_*` 五数组（S0a dumper 字段表即如此）；hfield 注入数据取自 S3a 反算证据的
16 个 raw f32 位型（见 §2.1），不构成对捕获参考的改写。

## 5. 窗口材料骨架（只骨架、不实跑）

- `window/entity-table.json`：9 实体 = 1 新目标实体 + 8 个 R09-S3b 回归实体
  （`render_camera_rays`/`render_context_lifecycle`/`render_e2e`/`render_kernel_depth_seg`/
  `render_kernel_flags`/`render_kernel_rgb`/`render_r08_cameras`/`render_r10_multi_world`），
  与 R09-S3b `guards.json` 的 `frozen_exes` 中 `source=regression` 清单一致（authoring 期已核对）。
- `window/guards-skeleton.json`：冻结时填 `path/bytes/sha256` 的骨架；本片不实跑、不填实际哈希。

## 6. 本地门禁（本轮实测）

- `cargo fmt --all --check`：通过
- `cargo clippy --all-targets --locked -- -D warnings`：通过
- `cargo test --locked --no-run`：通过（全 target 编译）
- `cargo test --locked --test render_hfield_reference`：**3 passed / 0 failed / 2 ignored**
- 未跑任何 GPU 用例、未跑 `--ignored`、未产生渲染数值结论

## 7. 边界与遗留

- 本片零渲染数值结论；RGB/depth/seg 判据数字全部留 S3b 验证窗。
- S3b 验证窗（`2026-10-28-render-bvh-r05-s3b-hfield-validate`）仍需自己的窗口材料与双签；
  本片只提供实体与骨架。
- S2 O5 已落实：hfield leaf `size` 来自 `build_hfield_mesh().half()`；mixed 组 box 世界位姿按签字
  `scene-mixed.xml` L55 的静态 body 常量构造（3.12 render 字段表无 `body_pos`，无可读取来源）。
