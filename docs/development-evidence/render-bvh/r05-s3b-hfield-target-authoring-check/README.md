# R05-S3b hfield 目标实体 authoring 独立复核

任务：`2026-10-28-render-bvh-r05-s3b-hfield-target-authoring-check`（P8 渲染与 BVH；p57；owner QA）
被复核：`2c56cc1a`（实现：测试 + 3.12 模型 dump）+ `d77b728e`（证据）+ `a6bcca80`（收口）
派工消息 HEAD：`a6bcca80`；复核时 HEAD `4bb9cd15`（另有我的 S3a 复核 2 提交；均未触被核路径，见观察 O-4）
范围：只读复核 + 本证据目录；**不开 GPU、不跑 `--ignored`、零渲染数值结论**。

## 1. 结论

**verdict：pass / 阻断 0 / 观察 4**（O-1/O-2 建议级、O-3/O-4 记录级，不阻断）。

重点「在烧窗口前打弱断言」全部成立：两个 `#[ignore]` 实体的判据与 R09-S3b `assert_group_criteria` **逐断言相同**（我按函数体归一化比对，仅类型名不同），覆盖 cam0/cam1 × rgb/depth/seg 六组 + 两次运行 sha256；没有「只 print 不 assert」「只查存在/长度」「容忍差异」的弱化。

| 复核点 | 结果 |
| --- | --- |
| 断言强度 | 与 R09-S3b 版本**归一化后完全相同**（6 条 assert：RGB `beyond_one_lsb_pixels==0`；depth `mismatched==0` + `max_ulp<=MAX_DEPTH_ULP(=4)` + `beyond_ulp==0`；seg `mismatched==0`）；GPU 实体先 print 比较规模/计数/max diff/ULP，再 `assert_group_criteria`，然后 3 个 sha 断言 |
| 六组 + 确定性 | 参考 2 相机（320×240 / 240×180）、120000 像素；循环覆盖 rgb×2 + depth×2 + seg×2 = 6 组；两次渲染 rgb/depth/seg sha256 各有 assert |
| O5 half 同源 | `let half = mesh.half();`（来自 `build_hfield_mesh`）→ `scene_state(..., half)` 中 hfield leaf `(half, [0,0,0])`；box 用 `geom_size`；注入走 `resolved.set_hfield`；host 无 half 重算/硬编码 |
| host 自检（我独立复算） | 输入 16 个 raw f32 位型 = S3a `reverse-calc.txt` [1]（**逐位相同**，文件 sha `3f404550…`）；我自跑探针得 **13 顶点 / 36 索引 / 12 三角形**、half 位型 `3fc00000/3f800000/3e000000`（=1.5/1.0/0.125），并用独立上游复刻（f32 口径）预测同值 |
| 不提前出结论 | host 只做结构/模型/参考解析核对；`#[ignore]` 2 个、普通 `cargo test` 不执行（我实测 3 passed / 0 failed / 2 ignored）；参考缺失时 host 用例打印跳过不 panic |
| dump provenance | 两个 dump sha/bytes 我重算：`bc537474…`/52 527、`c9a886ae…`/53 207；dumper `cd91e099…`/161 280 B（target 内在）；README 命令与两个签字 XML sha（`b3b9a6b4…`/`bf9bb966…`）齐备；dump 语义我独立解析：3.12.0、geom 1/2（`[1]`/`[1,6]`）、cams=2、lights=2、tex_data=12 288 B |
| 行数/改动面 | 权威口径 = committed blob 的 LF 计数：**392 / 354 / 499**（= `git numstat`/`wc -l`；authoring 表的 393/355/500 是 `split('\n')` 文本行口径，差 1 = 末尾换行后空片段）；sha/bytes 全对；`hfield_scene.rs` LF **499** 贴 500 上限（余量 1 行，见 O-3） |
| `shadow.rs` 改动面 | 仅 8 个 npz 解析函数可见性 `fn` → `pub(super) fn`（我核 diff：被删行全为 `fn` 签名，新增行仅可见性/续行，**断言零改**）；实测 numstat **+14/−8**（非文档写的 +8/−8，因 rustfmt 把 `i32_member` 签名折行，见 O-2） |
| 静态门（我自跑） | `cargo fmt --all --check` exit 0；`cargo clippy --all-targets --locked -- -D warnings` exit 0；`cargo test --locked --no-run` exit 0；`cargo test --locked --test render_hfield_reference` **3 passed / 0 failed / 2 ignored** |
| 窗口材料 / 证据 | entity-table 9 实体 = 1 new + 8 regression，与 R09-S3b 窗口 guards.json 的 `source=regression` 清单一致；guards-skeleton `freeze_status=not_frozen`、9 条哈希全空；证据 manifest 12 行（6 仓库相对 + 6 证据目录相对）全部对 committed blob 相符、无自引用、无 BOM/CRLF、禁词 0；receipt `numeric_conclusion=false`、`gpu_window=false`、`gpu_entities_executed=false` |

## 2. 判据等价（与 R09-S3b 逐断言对照）

- 我抽取 `hfield_scene::assert_group_criteria` 与 `shadow_scene::assert_group_criteria` 函数体，把 `HfieldReference` 归一为 `ShadowReference` 后**字符串完全相同**（含 6 个 `assert`/`assert_eq!` 与错误消息）；`MAX_DEPTH_ULP` 取同一常量 = **4**。
- 覆盖：`for camera in reference.rgb`（2 相机 RGB，`beyond_one_lsb_pixels == 0`）；`for camera in reference.depth_seg`（depth `mismatched==0`、`max_ulp<=4`、`beyond_ulp==0`；seg `mismatched==0`）→ 2×3 = 6 组。
- 未发现弱化：两个 GPU 实体都调用 `compare_group` + `assert_group_criteria` + 两次运行 sha 三断言；无「只 print」「只查文件存在/长度」「忽略差异」的分支。
- 超判据打印（同样执行）：每相机 `compared/mismatched/max_byte_diff/beyond_1_lsb`；depth `pixels/depth_mismatch/max_ulp/beyond_ulp/seg_mismatch/depth_zero_pixels`；两次运行 `rgb/depth/seg` sha256。

## 3. O5：`SceneLeaf.size` 来自 `build_hfield_mesh().half()`

- `HfieldScene::new`：`let half = mesh.half();` → `resolved.set_hfield(&fixture.inputs())` → `scene_state(&fields, &resolved, group, half)`。
- `scene_state` 内 hfield 分支 `(half, [0.0, 0.0, 0.0])` 直接作为 `SceneLeaf.size`；box 分支用 `geom_size`。无 host 侧重算/硬编码 half。
- host 自检另断言 `mesh.half() == [1.5, 1.0, 0.125]`（S3a 反算记录口径），并打印 `vertices=13 indices=36 triangles=12`。

## 4. host 自检独立复算（我自己跑）

- 输入位型：`tests/support/render_bvh/hfield_scene.rs` 的 `HFIELD_DATA_BITS`（16 个）与 S3a `reverse-calc.txt` [1] **逐位相同**（我解析比对）；`HFIELD_SIZE=[1.5,1.0,0.25,0.1]` 与签字 `scene.xml` 的 `hfield size="1.5 1.0 0.25 0.1"` 一致。
- 我用独立上游复刻（numpy 2.5.4/NEP 50 的 f32 口径，纯 stdlib 舍入）对同一输入预测：**13 顶点 / 36 索引 / 12 三角形**、`half` 位型 `0x3FC00000/0x3F800000/0x3E000000`。
- 我另跑临时探针（`cargo test --test _qa_s3b_auth_probe`，运行后删除）调 `build_hfield_mesh`：**V=13 I=36 T=12**、`HALF 3fc00000 3f800000 3e000000` —— 与上游复刻、host 自检、证据三方一致。
- 参考统计独立复算：两组 depth npz 各 `depth_zero_pixels=66826`（cam0 29776 + cam1 37050），与 host 打印一致；这是参考数据统计，**不是渲染数值比较**。

## 5. dump provenance

- 输出：`model-312.json` 52 527 B / `bc537474…`、`model-mixed-312.json` 53 207 B / `c9a886ae…`（我重算，与 README/receipt 一致）。
- 工具：`target/render-bvh/s0a/compile_scene_312.exe` 161 280 B / `cd91e099…`（现盘存在，我重算一致）；两条命令与两个输入 XML 签字 sha（`b3b9a6b4…`/`bf9bb966…`）都在 README §4。
- 我独立解析 dump：`mujoco_version=3.12.0`；geom 数 1 / 2（mixed 的 `geom_type=[1,6]`）；`cam_mode` 2；`light_type` 2；`tex_data` 12 288 B —— 与 host 打印一致。

## 6. 行数与改动面（权威口径说明）

- **仓库惯例 = committed blob 的 LF（0x0A）计数**（等价 `git numstat` / `wc -l`）：`render_hfield_reference.rs` **392**、`hfield.rs` **354**、`hfield_scene.rs` **499**、两个 dump 各 **144**、`shadow.rs` **422**。
- authoring 表用 `split('\n')` 的「文本行」：**393 / 355 / 500 / 145 / 145 / 423**，比 LF 计数 **+1**（末尾换行后的空片段）；两种口径我都实测，bytes/sha 与声明一致。
- 因此 Arch 的 393/355/500 与 Chief 的 392/354/499 差异 = 计数口径，不是文件被改；按仓库口径 `hfield_scene.rs` = **499**（≤500，余量 1 行，见 O-3）。

## 7. 观察（4 条，均不阻断）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| **O-1** | 建议 | 行数口径未标权威：README §1 给了两种计数（LF 392/354/499、文本 393/355/500），但未说明哪个是仓库口径；`hfield_scene.rs` 按文本口径正好 500（贴上限） | **修**：README §1 加一句「仓库权威口径 = LF 计数（392/354/499）」 |
| **O-2** | 建议 | `shadow.rs` 改动面文档写「+8/−8」，实测 `git numstat` = **+14/−8**：8 处可见性 + rustfmt 把 `i32_member` 签名折成多行（+3 续行 + 1 处格式）；断言零改 | **修**：措辞改「8 处可见性 + 1 处 fmt 折行（+14/−8），零行为/零断言」 |
| **O-3** | 记录 | `hfield_scene.rs` LF **499**（文本 500）贴 500 上限：本片已声明「终稿记录后不再加行」；S3b 验证窗期间不应再改此文件 | **接受**（窗口内禁改） |
| **O-4** | 记录 | 复核期 HEAD 从 `a6bcca80` 到 `4bb9cd15`（S3b 验证窗 authoring 两提交 + 我的 S3a 复核两提交）；我核过 `git diff a6bcca80..HEAD` **未触被核路径** | **记录**（后续派工写提交号） |

## 8. 边界与未覆盖

- **零渲染数值结论**：两个 GPU 实体未执行（本卡不跑 `--ignored`、不开 GPU）；RGB/depth/seg 判据数字全部留 S3b 验证窗。
- host 打印中的 `depth_zero_pixels=66826` 是**参考数据统计**（我独立复算一致），不是产品 vs 参考的比较结论。
- 未重跑 dump（dumper provenance 按现盘 sha + README 命令核对；未复跑 S0a dumper）。
- `target/` 下 dumper/S3b 窗口 guards 属瞬态：本报告记录的 target 证据（dumper sha、R09 guards 清单）已在响应中注明来源；关键结论另有 `git` 侧证据（commit/sha/numstat）。
- 未核 S3b 验证窗脚本 `scripts/validate/render-bvh/*`（属后续 validate 任务，不在本卡 files）。

## 9. 收口状态

- `normify gate 2026-10-28-render-bvh-r05-s3b-hfield-target-authoring-check` → exit 1「任务没有引用任何门禁」属预期（receipt `gate_note`）。
- 两段提交：证据（本目录 + 任务卡）→ `normify done` → verified 记录单独提交；显式路径，无 `git add -A`。
