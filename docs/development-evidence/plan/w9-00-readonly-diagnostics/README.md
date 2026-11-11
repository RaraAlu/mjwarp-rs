# W9-00 只读诊断（零代码、零 GPU、零窗）

- 任务：`2026-11-11-w9-00-readonly-diagnostics`（owner Backend）；性质：**只读诊断**，不改代码/判据/历史报告，不排期 deferred 项。

## 1. hfield/sdf/shadow 采集 refit 同型风险

- 方法：复刻 `scripts/np02-evidence/np02-capture-new-kernels.py` 的 source 装配（collision `DEVICE_SOURCES`
  concat + `product.cu`；render `concat!` 7/3 文件），与 receipt/inventory 记录的 `sourceSha256` 比对；
  并统计被采源码 refit 词频。

| 单元 | 记录 sourceSha | 当前装配 sha | bytes | 结论 |
| --- | --- | --- | --- | --- |
| mjwarp_collision_hfield | `12d50d68d3e7f155…` | `12d50d68d3e7f155…` | 254263 | ✅ 匹配 |
| mjwarp_collision_sdf | `12d50d68d3e7f155…` | `12d50d68d3e7f155…` | 254263 | ✅ 匹配 |
| render_megakernel | `cf356b728c563f7d…` | `cf356b728c563f7d…` | 110879 | ✅ 匹配 |
| render_depth_seg | `1e3f273c88a77235…` | `1e3f273c88a77235…` | 53552 | ✅ 匹配 |

- **known drift 参照**：`scene_bvh_ray` 记录 `8dad4d61235a8686…` → 当前 scene.cu `caac8c86a4e4f1e7…`（不匹配；`d35cab68` 所致，属 NP5-04 待裁第 10 条）。
- **refit 词频**：collision product.cu / scene.cu / render_shadow.cu / render_shade.cu 均为 **0**；`refit_bvh`/`refit_splat_bvh`
  是 host 侧 API，不在被采内核源码中 → refit 语义变化不会静默改变采集回执。
- **风险结论**：同型机制存在（回执按 sourceSha 绑定），但 **hfield/sdf/shadow 当前无漂移**；唯一实际漂移为已知 scene_bvh_ray（第 10 条待裁）。

## 2. P0-1 计时含 refit 文档

- 位置：`docs/replication-plan.md` §2026-10-22 P0-01 复核记录。
- 发现：记录含验证命令/平台/通过/失败/未测/误差与容量/剩余缺口，但**没有**「计时是否包含 BVH refit」的显式说明；
  其他性能文档只写编译/传输/依赖/JIT 等计时边界，均未提 refit。
- 建议（W9 文档项，不本片执行）：补一句计时边界是否含 `refit_calls`；当前 P0-01 记录无性能数字，故不影响既有数字，
  但若后续性能对比引用 P0-01，须显式声明口径。

## 3. R06 recipe params（7 vs 28）

- 卡：`2026-11-05-render-r06-flex-multiframe-recipe-params-backlog`（status=backlog，卡面明示不排期/不 start、无机器检查项）。
- 差异：实体 A `assert_params_match_capture` 比对 **7 项**；单帧验收 `tests/render_flex_validate.rs` 覆盖 **28 项**。
- 不修理由（在案）：多帧与单帧参考同 pinned 环境/同 recipe 采集；`capture.json.provenance` 已锚定 spec sha + 三脚本 sha +
  `refit_calls`/`actual_sequence`；帧内判据已覆盖三相机输出位级；缺口仅在「其余 21 项若漂移，实体不会显式失败」。
- 补齐触发与路径（若触发）：验证窗或后续审计暴露 recipe 漂移风险 → CPU-only 零 GPU 自证，把 7 对齐到 28 并逐项列差异；
  不改已签 spec/判据/参考、不改 G-2 夹具。

## 4. 结论

3 项只读诊断完成：①hfield/sdf/shadow 采集 source 身份与回执全匹配（无漂移；refit 不在被采源码、唯一实际漂移为已知 scene_bvh_ray 第 10 条）；②P0-01 记录缺「计时含 refit」显式说明 → 记为文档缺口（不改历史报告）；③R06 recipe params 7 vs 28 维持 backlog 不修 + 理由在案，补齐触发条件与 CPU-only 路径已写明。零代码、零 GPU、零窗。

## 5. 验证

- selfcheck（`logs/selfcheck.txt`）：3 项诊断在案、`readonly/code_changes_made=false/zero_code=true`、manifest 4 文件自核、
  policy_scan 0 命中、编码 UTF-8/LF 无 BOM、边界仅本目录 + `.normify`。
- 四门禁：fmt/clippy/clippy-cuda/unit。

## 6. 边界与非声明

- 零代码、零 GPU、零窗；只写本目录 + `.normify/`；不改代码/判据/参考/历史报告，不排期 deferred 项。
- 不宣称任何门禁或阶段准入通过、不宣称等价/双平台/发布。
