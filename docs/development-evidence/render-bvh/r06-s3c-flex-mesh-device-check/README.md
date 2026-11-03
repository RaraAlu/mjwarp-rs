# R06-S3c QA 复核：2D/3D flex 设备接线（`181eaa7a` + 证据补丁 `e9e198cd`）

- 任务：`2026-11-04-render-r06-s3c-flex-mesh-device-check`（QA；离线、零 GPU、零窗口、零参考）
- 被复核对象：`2026-11-04-render-r06-s3c-flex-mesh-device` 提交 `181eaa7a`（23 文件 +1142/−70）与其证据补丁 `e9e198cd`（README §3.1，只改证据）
- 复核基线：`e9e198cd`（工作树 clean）
- 结论：**PASS-with-observations**（0 阻断 / 0 严重；2 条建议级、3 条观察级）

## 1. 复核点逐条

| 复核点 | 结果 | 依据（复算见 `qa-recheck.txt`） |
| --- | --- | --- |
| `None` 路径等价 | 成立 | `flex_mesh=None` 且 `flex=None` 时仍调用 `SceneDevice::pack(context.scene_bvh(), context.scene(), …)`，实参与父提交逐项一致；设备 flex 分支由数据门控（无 flex 叶 ⇒ `leaf_geom >= 0` ⇒ 不进 flex 分支）。未跑 GPU，像素/深度数值归 S3b。 |
| `append_to` 绝对条目号 | 成立 | `mesh_device` 内 `bvh_ids[flex] = device.entries.len()`（**段内相对**，不改写）；`FlexMeshDevice::append_to` 先记 `base = offsets.len()`、`(meta_base, params_base)`，`extend_from_slice` 之后逐条目 push `[entry[0]+meta_base, entry[1]+params_base]`；`append_to_mesh_device` 对每 flex 输出 `base + id` 并保留 `u64::MAX`（1D/无 mesh）。§3.1 四条可复核点 ①–④ 全部成立。 |
| 449 行纯移动 | 成立（token 级） | 父提交 40 行 match 臂 → 新 `pack_flex_scene`：去注释、忽略花括号/分号后 token 流**只差 2 个 token**——一个 `&`（参数已是 `&ModelRenderArrays`）与尾部 `?`（改为直接返回 `Result`）。行数：`megakernel.rs` **449**（≤450 死线）。 |
| NVRTC sha / PTX | 成立（独立重编译） | 重跑 `tools/nvrtc_precheck.py`：`source_concat_sha256=d48fc74e…`、LF 文本 **343,993 B / `9118d1ea…`**；W15 两条常量各 1 次、sin/cos 归约三元组 5（`.f64` token 18 = 2 + 5×3）、f16/f16x2/bf16/bf16x2/tf32 **全 0**、`.visible .entry render_megakernel` 在。 |
| 零 diff 三类 | 成立 | 对象 23 文件中 `fixtures/` **0**、`docs/evidence/` **0**；`src/render/bvh/` 仅 3 个新 `flex_scene*` + `mod.rs` +4 注册行（无既有刚性路径改动）；`.cu` 改动全部在 flex 路径（新增 `flex_mesh_leaf_hit`、`if (leaf_geom < 0)` 内 flex 分派、cull 条件追加 `|| flex_mesh_path`——该标志仅 flex 叶置真、刚性叶恒假、`flex.bvh_id` 赋值），刚性几何语义不变。 |
| 行数 | 成立 | `megakernel.rs` 449/450；`flex_scene.rs` 470、`device.rs` 186、`tests.rs` 259（≤500）。 |
| 通道与错误路径 | 成立，无 panic | `prepare` 先校验 `worlds==nworld`、`flex_dim.len()==nflex`；`build` 校验 `topology.nflex==nflex` 后**先 `channel.topology.validate()` 再按 flex 索引**；`dim∉1..=3` → `UnsupportedDimension`，非有限半径 → `NonFinite`；2D/3D 但无 mesh 通道 → `UnsupportedMeshPrimitive` → 上层 `flex-scene-leaves: …` Driver 错误；新文件非测试代码无 `unwrap()/expect()/panic!()`。 |
| 调用顺序联动 | 成立（不适用即满足） | `FlexMeshChannel` 不含 `FlexRenderFields`，S3c **不调用** `validate_topology_against_render`（该函数只在 `src/io/flex/topology.rs` 及其测试出现）；S3c 的 host 入口用 `topology.validate()`，与 S2c 修卡（`b4a7e827` 已把 `render.validate()` 放到该函数入口）互不冲突，后续正式接线若复用该函数则自动先校验 render 侧。 |
| 上游证据一致性 | 成立 | S3c 证据 `manifest.txt` 6 条 sha+bytes **6/6** 通过；4 门禁记录 fmt/clippy/clippy-cuda passed、unit **1302/0/398**（S3a 1298 + 4 条新单测）。 |

## 2. 观察与建议（不阻塞）

1. **【建议】S3c 证据有 3 个文件带 UTF-8 BOM**（`EF BB BF` 开头）：`gates.txt`（329 B）、`manifest.txt`（1059 B）、`unit-tests.txt`（559 B）；同类目录（如 `r06-s2c-flex-shell/manifest.txt`）无 BOM。影响：`manifest.txt` 第 1 行 `# R06-S3c manifest: …` 的 `#` 不在字节 0，按行解析时若不先剥 BOM 会被当成条目（我的复核脚本因此报 `MISSING manifest: sha256 bytes repo-relative path`）。**更正我上一轮口头说法：不是"表头缺 `#`"，是 BOM 让 `#` 不在行首**。建议剥掉这三个文件的 BOM（不改变任何被列文件的哈希，manifest 内容无需重算）。
2. **【建议】precheck 工具 `.ptx` 落盘是 CRLF**：`tools/nvrtc_precheck.py` 用文本模式 `write_text`，磁盘文件 354,740 B / sha `4e397438…`，而记录口径是 LF 文本 343,993 B / `9118d1ea…`。`.ptx` 未入库，不影响本片上库证据；建议改二进制写或写出前归一（Arch 已登记小修卡，不占本复核面）。
3. **【观察】字段文档措辞**：`FlexMeshAssembly.bvh_ids` 的文档写「绝对索引」，但 build 期实际是**段内相对号**，只有 `append_to_mesh_device` 的返回值才是绝对号（证据 README §3.1 描述是准确的）。建议字段文档改为「段内相对号；append 后转绝对」避免误用。
4. **【观察】`render_with_flex` 目前无生产调用点**（证据 README §5 已如实披露，S3b 之后接 harness）。现阶段「设备接线正确」由 4 条单测 + 静态检查 + NVRTC 预检支撑，无 GPU 数值结论。
5. **【观察】1D 等价性靠静态同构论证**：`flex_scene::scene_leaves` 与 S3a `one_dimensional_scene_leaves` 公式/顺序/`Aabb`/`group` 编码一致，但没有单测直接对比两个 builder；`channel=None` 时仍走旧 builder，故不会回归。建议后续补一条等价单测（可选）。

## 3. 复核方式与边界

- 全部复算在本机离线完成（NVRTC 12.8.93，不建 CUDA 上下文、不发射内核）；未跑 GPU、未开窗口、未生成参考。
- 未重跑 S3c 的 cargo 全量门禁（取提交内 `.normify/runs/` 记录）；S2c 修卡 `b4a7e827` 的定向测试由本人在本轮独立跑过（`--features native-model-probe --lib io::flex::topology` → **6 passed / 0 failed / 1 ignored**），该结果归 S2c 复核闭合，不混入本卡结论。
- 不宣称 2D/3D flex 渲染可用；1D/2D/3D 像素/深度/seg 数值结论归 S3b 窗口。
- 本复核只写本目录证据，不改 `src/`、`fixtures/`、`.cu`、判据/容差/参考。

## 5. 观察收口记录（2026-11-04）

| # | 本条复核的观察 | 收口状态 | 核对证据（HEAD） |
| --- | --- | --- | --- |
| 1 | S3c 证据 3 个文件带 UTF-8 BOM（`gates.txt`/`manifest.txt`/`unit-tests.txt`） | **closed**（修复提交 `ddd06292`「R06-S3c 证据去 BOM + `bvh_ids` 文档口径（QA 复核 2716c092 观察）」） | 三文件现均无 BOM：`gates.txt` 首 3 字节 `523036`（326 B）、`manifest.txt` `232052`（1067 B）、`unit-tests.txt` `202020`（556 B）；`manifest.txt` 6 条 sha+bytes 复核 0 条不匹配（0 = 6/6 通过）。 |
| 2 | precheck 工具 `.ptx` 文本模式落盘 CRLF（磁盘 354,740 B/`4e397438…` vs 记录 LF 343,993 B/`9118d1ea…`） | **已转治理卡**（`2026-11-04-governance-nvrtc-precheck-ptx-eol`，`24f1e940`，owner Backend；`.ptx` 未入库，不影响本片上库证据） | 记录口径本就是 LF 文本；工具修复归该卡，不在本复核面。 |
| 3 | `FlexMeshAssembly.bvh_ids` 字段文档写「绝对索引」但 build 期是段内相对号 | **closed**（同 `ddd06292`） | 现文档为「build 期存**段内相对条目号**（1D/无 mesh 为 `u64::MAX`）；[`Self::append_to_mesh_device`] 追加到 `MeshDevice` 后转成 blob 绝对条目号」（核对=True）。 |
| 4 | `render_with_flex` 暂无生产调用点 | 已由 S3b① 采集/harness 卡接管（`0a3058ee`），不在本复核面 | — |
| 5 | `dim==1` 与 S3a「逐字节一致」缺直接对比单测 | 记 S3c 后续待办（Arch 已记，不立卡） | — |

结论维持 **PASS-with-observations**，其中观察 1/3 已关闭、观察 2 转治理卡、观察 4/5 已有归属；无需重开 S3c 复核卡。

## 4. 文件

| 文件 | 内容 |
| --- | --- |
| `README.md` | 本文件：三态结论、复核点逐条、观察清单 |
| `qa-recheck.txt` | 独立复算原始输出（零 diff 口径、行数、纯移动 token 差、NVRTC 重编译 sha/字节、§3.1 四点、manifest、BOM 扫描） |
| `manifest.txt` | 本目录文件 sha256 + bytes（不含自身） |
