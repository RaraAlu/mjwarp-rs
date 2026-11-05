# R06 多帧渲染参考（脚本帧循环产物）

- 生成命令：`D:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Scripts\python.exe -B D:\Rust\mjwarp-rs\scripts\authoring\capture_render_flex.py --frames 3 --frame-dz 0.1 --out-dir D:/Rust/mjwarp-rs/fixtures/render-bvh/scene-flex/multiframe --repeat 2`
- 冻结 spec：`2026-11-05-render-r06-flex-multiframe-criterion-spec`（sha256 `1d9fa5cef1ebc829747d63f648d3bab1ec935d8df8e3403bc52c2ac2aa7c40b6` / 10506 B）
- 帧集（spec §2 冻值）：frame-000 / frame-001 / frame-002；锚点 `body_pos.z` 每帧 `+0.10 m`
  （锚点 body：flex-anchor0=id1, flex-anchor1=id2, flex-anchor2=id3，
  base z = 0.000000, 0.000000, 0.000000）
- 每帧产物：`frame-{000,001,002}/{rgb,depth,seg,state}.npz`；顶层 `capture.json` / `cam-poses.json` /
  本 README。`state.npz` 字段同 base：qpos / geom_xpos / geom_xmat / light_xpos / light_xdir / flexvert_xpos。
- 两次采集：`--repeat 2`，逐帧逐文件 sha256 一致（`capture.json.two_captures_bitwise_identical = true`）。
- 每帧渲染前强制 `mjw.refit_bvh`（两遍采集逐帧都做，`capture.json.refit_calls = 6`）。
- 采集路径三条硬自查（任一失败 exit 3）：frame-000 与 reference/base 位级全等、附着顶点位移 = k×0.100 m、
  两段变化像素均 ≥576；provenance 见 `capture.json.provenance`（spec sha / 脚本 sha / refit 标记）。
- 变化像素自检（spec §5）：结论见 `capture.json.delta_selfcheck.overall`；非 `pass-margin` 时脚本
  **exit 3** 并打印「不得落库」警告（`--check-deltas` 独立模式同语义）。
- 本目录只是离线 authoring 产物记录；帧内/跨帧判定口径见冻结 spec，本 README 不作结论。
