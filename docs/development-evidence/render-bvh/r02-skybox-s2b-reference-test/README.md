# R02-S2b：天空盒位级比较测试目标 `render_skybox_reference`（离线实现，窗口硬前置）

任务：`2026-10-29-render-bvh-r02-skybox-s2b-reference-test`（P8-渲染与BVH，owner Backend，Arch 登记）
本片性质：**离线实现片**。为 r02-skybox 验证窗准备第 9 个实体（冻结器需要该 exe 才能生成 77 条目
guards）。本片**不跑任何 GPU**（不执行 `--ignored` 设备用例）、不 capture、不冻结、不开窗、不出渲染
数值结论；设备数值只在窗口内的该实体执行时产生。

## 1. 交付物

| 交付物 | 路径 | 说明 |
| --- | --- | --- |
| 窗口实体测试目标 | `tests/render_skybox_reference.rs`（138 LF / 4 993 B / sha256 `7ecb972a…`） | 实体名 `render_skybox_reference`；host 自检用例默认执行、GPU 用例 `#[ignore]` |
| 参考读取 + 产品路径渲染 + 位级比较 | `tests/support/render_bvh/skybox.rs`（479 LF / 16 563 B / sha256 `e6a2e6fd…`） | 只读 stage A 参考；npz/sha 复用 `shadow.rs`，RGB diff 复用 `s7b2::diff_rgb`，depth/seg 复用 `kernel::output::diff_depth/diff_seg` |
| 测试身份与门禁 | `materials/test-identity.txt` | 文件 sha/bytes/LF、用例名、门禁与 unit 数 |
| 窗口就绪补充（双签包用） | `materials/window-readiness.txt` | GPU 快照（3 点）、预计时长、`run-window.ps1` 默认行为不变证明、冻结器断言、失败/回滚口径 |

## 2. 产品路径（无旁路）

`resolve_context(render_skybox=true)` → `context::texture_registry(use_textures || skybox)` →
`RenderContext::create` → `MegakernelRenderer::render`。相机位姿与预计算射线复用 `s7b2::camera_poses` /
`s7b2::precomputed_rays`（与 scene-a 既有实体同配方）。**不直接喂设备数组、不绕过产品入口。**

## 3. 判据（与 R05 完全一致，不新增/不放宽）

* RGB：逐通道 ≤1 LSB（`beyond_one_lsb_pixels == 0`，并打印 `mismatched_pixels/max_byte_diff`）；
* depth：位级优先（`mismatched == 0`）且 `max_ulp <= 4` 且 `beyond_ulp == 0`（打印 `depth_zero_pixels`）；
* seg：逐像素全等（`mismatched == 0`）；
* 确定性：同一 context 第二次发射后 rgb/depth/seg sha256 相等（并打印三 sha）。
任何超判据均以 assert 失败 + 差异首点（camera/pixel 坐标）结束，不静默接受。

## 4. 参考绑定与缺失行为

* 参考目录 `fixtures/render-bvh/scene-a/reference-skybox/`（stage A 产出）。加载时校验：
  `schema=1`、`kind=r02-s1-skybox-reference`、`params28.render_skybox=true`、`expected_pixels=120000`、
  `skybox.skybox_tex_id=2`、`face_width=64`、`skybox_tex_height=384`（竖排 6×64）、`determinism.sha_equal=true`、
  两相机分辨率 320×240 / 240×180、三份 npz 的 sha256 与 `capture.json.files` 记录一致。
* stage A 未跑时 host 自检**打印并跳过**（不 panic），因此普通 `cargo test` 在采集前也全绿；GPU 实体在
  参考缺失时会直接失败并提示先跑 `capture_skybox.py`。

## 5. 门禁（最终树）

fmt 0 / clippy 0 / clippy-cuda 0 / unit 0（**136 result 行，passed 1104 / failed 0 / ignored 390**，
较 S2 基线 +1 host 用例、+1 ignored GPU 用例）/ `--no-run` 0 / `validate --fingerprints` 错误 0。
详见 `materials/test-identity.txt`。

## 6. 未执行 / 非声明

* **零 GPU 执行**：`skybox_matches_captured_reference` 未运行（`#[ignore]`），窗口未开、未 capture、未冻结。
* 未改判据/容差/参考/fixture；未动 `guards.json`；未动 R05 证据。
* 本片不产任何渲染数值结论；设备数值与两次运行 sha 只会在窗口内的该实体执行时产生。
