# 逐 world 渲染实现 S1（方案 L v1）

任务：`2026-10-30-render-perworld-impl-s1`（实现片；数值验收在验证窗内由 GPU `--ignored` 用例执行）。

## 结论与边界

- 本片完成方案 L 的 **host 逐 world 循环 + 单 world 发射** 实现：per-world geom 位姿容器、AA 守卫、
  逐 world 发射编排、最小接线与 CPU 单测均已落地；四门禁通过。
- **窗口尚未执行**：逐 world 数值比对用例已就绪（`tests/render_per_world.rs::gpu_per_world_matches_captured_reference`，
  `#[ignore]`），等 18:00 后空闲块内由用户双签的验证窗运行。本证据不预判窗口结果。
- v1 只覆盖 **per-world geom 位姿**（`geom_xpos`/`geom_xmat` → `WorldScenePoses` → `update_poses` → `refit_bvh`）；
  灯数组仍模型级单份（v1 不注入 per-world 灯）。per-world 灯与 AA 组合留 L v2/K。
- 性能不做声明；不改 `.cu`/ABI/判据/参考/夹具。

## 实现机制（证据点 a、b）

1. **输出定位用 adr 位移，不做 sub-buffer 视图**：
   - `kernel/megakernel_uploads.rs` 的 `adr` 闭包把每相机 `rgb/depth/seg adr` 平移到目标 world 区段
     （`world × pitch + adr`；`adr < 0` 的禁用通道保持 -1），上传到既有 81 槽。
   - 单 world 发射时内核 `worldid` 恒为 0，因此内核写 `worldid × pitch + adr + rayid_local`
     = 该 world 的全局区段；输出数组形状仍是 `[nworld, pixels]`，未改槽形状、未改内核输出下标公式。
2. **为什么内核索引点无需改动（b）**：单 world 发射下 `worldid == 0`；相机数组按 `[1, ncam]` 只喂目标
   world 行；场景 BVH 在发射前用目标 world 的位姿 refit（同拓扑），`cast_ray` 的 `roots[worldid]` 命中目标
   world 树；各槽只含该 world 数据，因此内核里 `cam_slot`、`rgb_offset`、`worldid` 相关表达式逐字不变。
3. **逐 world 编排**：`kernel/per_world.rs::render_per_world`
   - 守卫：`reject_multi_sample(nsamples)`：`1 → Ok`，其余返回 `LauncherError::Unsupported`（不静默按单样本渲染）。
   - 每 world：把目标 world 的几何位姿铺到全部 world 组 → `context.update_poses` → `context.refit_bvh()`
     → `MegakernelRenderer::render_world`（`launch_worlds=1`）。
   - 结束后把 context 场景状态同步为调用方给的 per-world 位姿；失败时尽力恢复调用前状态。
   - leaf 槽位映射校验：各 world 的 leaf 数一致、同槽位 geom id 次序一致，否则 `InvalidArray` 可诊断返回。
4. **已知边界（c）**：场景 BVH 使用同拓扑逐 world refit（与上游全局单拓扑 + 逐 world bounds 的命中结果一致，
   树形不同，只影响性能）；单 world 发射为 per-world 顺序循环，不改变单 world 语义。
5. **L-2 / L-3 不采用（e）**：L-2（每 world 另建 context + D2D 拷贝）违背零 D2D/零第二套 context 前提；
   L-3（抽 `megakernel_args.rs`）超出本片写入面并会改动 500 行顶格文件的结构。

## 行数死线与零改动面（实测，UTF-8 LF；2026-11-03 订正：tests 行数原误填上限 500，实测 453）

| 文件 | 行数 | 限制 |
| --- | --- | --- |
| `src/render/kernel/megakernel.rs` | 450 | ≤450（0 余量） |
| `src/render/kernel/megakernel_uploads.rs` | 480 | ≤480（0 余量） |
| `src/render/kernel/per_world.rs` | 238 | ≤260 |
| `src/render/kernel/mod.rs` | 476 | ≤500 |
| `tests/render_per_world.rs` | 453 | ≤500 |

零改动面（逐文件未触碰）：`kernel/cast.rs`(481)、`context/lifecycle.rs`(498)、
`src/runtime/launcher/render/args.rs`(494)、`context/output.rs`(151)、`kernel/megakernel_aa.rs`(93)、
`kernel/launch.rs`(500)、`context/camera_output.rs`(499)；`git diff --stat -- src/render/kernel/cuda` 为空；
`MEGAKERNEL_SLOTS` 81 槽与既有形状规则不变。

## 验证

- 四门禁：`normify gate 2026-10-30-render-perworld-impl-s1` → fmt / clippy / clippy-cuda 通过，
  unit **1240 过 0 败**（默认特性）。
- lib 单测（cuda-probe）：`cargo test --locked --features cuda-probe --lib` → **741 过 0 败 137 ignored**，
  含新增 `scene_poses` 3 项、`per_world` 3 项（守卫 1→Ok、2/4/0→Unsupported；leaf 槽位/geom 次序校验；
  目标 world 铺位姿与调用方映射回填）。
- 集成测试（cuda-probe）：`cargo test --locked --features cuda-probe --test render_per_world` →
  CPU **3 过**（容器/守卫 + **参考解码自查**：4 个 npz 的 sha256 绑定、逐 world 成员布局、state f32 位型、
  world 间位姿差异）、GPU 用例 **1 ignored**（窗口执行）。

## 窗口待执行（GPU 用例，就绪）

```text
cargo test --locked --features cuda-probe --test render_per_world -- --ignored
```

用例覆盖（`fixtures/render-bvh/scene-a/perworld/nw3-hinge02040/`，nworld=3、spp=1、单相机 320×240）：
参考文件 sha256 绑定 `capture.json.files_sha256`；逐 world rgb（mismatched=0 / max_byte_diff=0 / >1LSB=0）、
depth（逐位不匹配=0 且 max_ulp≤4）、seg（全等）；两次运行 SHA 相同；world 隔离（仅改 world1 位姿后
world0/world2 输出逐位不变，且 world1 输出确实变化）。window 结果回填本目录。

## 不宣称

不宣称逐 world 数值通过、G01/G03 全量、NP01–NP04 精度、Linux、release 或性能；窗口执行前不给出任何
逐 world 数值结论。
