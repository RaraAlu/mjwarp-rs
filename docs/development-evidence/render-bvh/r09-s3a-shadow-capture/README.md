# R09-S3a 阴影参考 authoring 捕获窗

任务：`2026-10-28-render-bvh-r09-s3a-shadow-capture`（P8 渲染与 BVH；p61；owner Backend）
判据树 HEAD（Chief 签名时）：`348866b070f62341a93c60e4a71e6d9def18f52a`
性质：**authoring-only 捕获**（pinned upstream 环境，不进产品路径、不引用产品 Rust 代码）；本片只产出参考，**不产任何产品侧数值结论**（数值验收在 R09-S3b 窗）
产物位置：`fixtures/render-bvh/scene-shadow/`（新目录，只新增）；本目录为窗口产物副本与登记

## 1. 窗口事实（唯一的有效轮 = attempt1）

- 弹窗前自采监测：`2026-10-28T18:53:45.253+08:00 → 19:09:00.934`，**181 样本**（5 s 间隔，`attempt1/monitor.csv`）；**最长连续 60 样本满足 `util<40 且 free≥10240` = 5 分钟达标**（`required=60`、`clean_ready=True`）后才弹窗；监测期 `util 1–91`（外部 burst 明显）、`free_min=10593 MiB`
- 窗口：`window_open 2026-10-28T19:09:01.816+08:00` → `window_close 19:09:23.703+08:00`，**body ≈ 21.9 s**
- baseline（3 点）：`2571,13491,2,32 / 2571,13491,2,32 / 2571,13491,2,32`（used MiB / free MiB / util% / contexts）
- `baseline_free_mib = 13491` → **floor = min(10240, baseline_free−3072) = 10240 MiB**（公式与值由窗口脚本按当次 baseline 重算并写入 `attempt1/facts.txt`）
- post（3 点）：`2571,13491,2,33 / 2571,13491,1,33 / 2571,13491,1,33` → used/free 零漂移，**delta = 0 MiB**
- guard：`attempt1/guard.log` —— `guard_baseline used=2571 free=13491 util=2 contexts=30`、`preconditions n=1 delta=0`、`guard_end peak=2571 samples=1 aborted=False`
- **零 Rust 编译**：窗内无 `cargo`/`rustc` 进程，未触发任何构建
- `residual_processes=4`：4 个 python 进程的 StartTime 为 `10-27 10:33`（2 个）与 `10-28 17:20`（2 个）→ **外部 IDE/LSP 进程，非本次捕获**（捕获进程已退出，窗口 exit 0；如实记录，未做处置）
- 轮次：**attempt1 = 有效轮 1**（第一次弹窗即进窗，两组捕获均 exit 0）；本窗无 precheck abort、无非有效轮、无重试
- 窗口脚本：`target/r09-s3a-window/run-capture-window.ps1`（7 972 B / sha256 `5c7e136fecee2e1c8d481b7e7e5f0d7f0085d720f81c28eecf317908c34bbe95`，`target/` 不入库，仅登记）；签名记录 `target/r09-s3a-window/signature-record.md`（2 887 B / sha256 `0a6fe6b359e851a6182e8edd8ba3fd8713a62f41d0cea05cd6c5961ce68bcd34`，仅登记）

## 2. 采样口径偏离与 Chief 裁决①（接受并如实记）

- 事实：**window body 21.9 s（6 个点：baseline 3 + post 3），未起 0.25 s 连续采样器**
- **经 Chief 裁决接受**，随附补偿事实：窗前后 15.25 分钟 181 样本 5 s 监测、最长连续 60 样本=5 分钟才弹窗、baseline/post 各 3 点零漂移、delta 0、guard preconditions 通过、无计算型负载、零 Rust 编译、4 个 residual python 经 StartTime 核为外部 IDE/LSP
- **该例外只适用 authoring-only 捕获；设备验证窗仍必须 0.25 s 采样器**（口径不变）

## 3. 捕获结果（两组，各 A/B 两次独立捕获）

| 组 | 文件 | bytes | sha256 |
| --- | --- | --- | --- |
| shadow-on | `capture/shadow-on/capture.json` | 8 892 | `3867f6e1c1b38eb0cf3e015a2e613d726d85c9e6bcf8ac5271977740e7e28d27` |
| shadow-on | `capture/shadow-on/depth.npz` | 480 486 | `0fa5582170a2b2a117e46868930fa2b4270dd993b0f5d86dc603981e0a2fd7d3` |
| shadow-on | `capture/shadow-on/rgb.npz` | 1 440 478 | `65596761f77a099a6c47ed885143ba3a8b50c0ca128606ae52b373142533ebc7` |
| shadow-on | `capture/shadow-on/seg.npz` | 960 478 | `2153fa728d634c032cf38847761803c4faa806aa554e108a2e0bd7b8ac8e6a97` |
| shadow-off | `capture/shadow-off/capture.json` | 8 900 | `6c1e9d74532ed980a7ae3cca4f6d9ed4727c08c0ecda1ccc9f5a8825dada7b0f` |
| shadow-off | `capture/shadow-off/depth.npz` | 480 486 | `0fa5582170a2b2a117e46868930fa2b4270dd993b0f5d86dc603981e0a2fd7d3` |
| shadow-off | `capture/shadow-off/rgb.npz` | 1 440 478 | `9379f5d3b39e2d5d0ce8ef21a28b365e3608d20f6a992c39dff2d5df3ef90c0c` |
| shadow-off | `capture/shadow-off/seg.npz` | 960 478 | `2153fa728d634c032cf38847761803c4faa806aa554e108a2e0bd7b8ac8e6a97` |

- 合计 **8 文件 / 5 768 076 B（≈5.51 MiB）**
- **两组各自 A/B 两次独立捕获逐字节一致**：`two_captures_bitwise_identical = true`（两组 capture.json 均记录；同进程两次独立 render，数组逐元素与落盘字节均相同）
- 场景：`fixtures/render-bvh/scene-shadow/scene.xml`（4 757 B / sha256 `42f34ae4496042a0d64576bee18e1fc3cf30fdadac3ad374989b06d1af12060c`）；派生自 scene-a，**唯一语义改动**为点光 `lamp` 的 `castshadow` false→true；三灯（`sun` directional / `lamp` point / `spot`）`castshadow=1,1,1`、`active=1,1,1`
- 两组 `capture.at` 相同：`MJWARP_CAPTURE_AT = 2026-10-28T19:09:01+08:00`；`repeat=2`、`seed=20261028`、`total_rays=120000`、`shadow_light_fraction=0.3`、`resolved.use_shadows` = true / false
- 捕获 wall：shadow-on **19 659 ms**（首次含 warp 内核编译；其 render run1 2.660994 s / run2 0.00332 s）、shadow-off **2 085 ms**（run1 0.027629 s / run2 0.000186 s）
- **管线交叉校验信号**（仅作 sanity，不作任何数值或等价结论）：
  - (a) 两组 `depth.npz` 与 `seg.npz` 逐字节相同（阴影只影响光照，不影响 depth/seg 内核输出）
  - (b) shadow-off 组三通道 sha 与既有 S0b scene-a 参考一致（同几何同相机、`use_shadows=false`）

## 4. 判据与口径

- 本片**只产出 authoring 参考**；判据/容差/参考/夹具**零改动**；旧参考（S0b/S7b/S8 等）只增不改
- 两组分别记录两次捕获位等；shadow-on 为主参考，shadow-off 为同 state 对照（文件名区分，判据仍以 shadow-on 为准）
- 不写任何窗口结论、覆盖声明、精度结论或等价结论（含上游与双平台）；不基于本窗数据做任何产品侧结论
- 加密/共享：不适用；本窗不产生密钥、不写日志敏感信息

## 5. 旧参考未动（Chief 裁决②：以逐文件清单为准）

- 范围：`fixtures/render-bvh/` 除 `scene-shadow/` 外，**38 文件 / 25 320 515 B**
- **权威证据 = 逐文件 sha256 清单跑前/跑后一致 + count/bytes 不变**：`attempt1/legacy-manifest-after.txt`（38 行；跑前基线由窗口脚本按同一算法采集，facts 内 `legacy_before_*` 与 `legacy_after_*` 全等、`legacy_untouched=True`）
- 三种聚合值（**各自定义不同，均不得标为 canonical**）：
  - `b5cb50afcc608a67ca9a7b83222029402719f2770a3510ab423df13601e50d8b` —— 定义：38 个文件 sha256 十六进制串**排序后直接拼接**再 sha256（Chief 签窗口径）
  - `983617f16b534f9cda743ff2b84bfed08e0154e88c369419b885bacb841cc260` —— 定义：`sha256<空格>bytes<空格>path` **排序行拼接**再 sha256（`attempt1/facts.txt` 与 `legacy-manifest-after.txt` 口径）
  - `b21b8b02…` —— 定义：38 个文件**字节直接拼接**再 sha256（仅记前缀，完整值见窗口 facts 记录；本目录不复制该中间产物）
- **教训（写入流程）**：以后聚合 sha 必须在签窗材料里写清定义，否则不可复核

## 6. 环境与命令

- pinned python：`D:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Scripts\python.exe`（47 104 B / sha256 `feeec36120097a03201a96ae17a0e2c1fba70dd6241cb4bfdd9e7c2a00bada77`）
- 版本五连：**python 3.12.13 / warp 1.15.0 / mujoco 3.15.0 / mujoco_warp 3.15.0 / numpy 2.5.4**
- GPU：**RTX 4070 Ti SUPER / arch 89 / driver 596.36**；pinned warp 自带 NVRTC：**CUDA Toolkit 12.9 / Driver 13.2**（见 `attempt1/capture-shadow-on.log` 头部；窗口 facts 内 `env_cuda_bin=target/toolchains/cuda-12.8.1/bin` 仅为守卫脚本环境变量，不参与本捕获渲染）
- upstream：`71da24d956378a87a703b6e1442b13aec0c4ac29` clean
- 窗内命令（各一次，同一 `MJWARP_CAPTURE_AT`）：
  - `…\venv\Scripts\python.exe -B scripts/capture_render_shadow.py --use-shadows true --out-dir fixtures/render-bvh/scene-shadow/capture/shadow-on`
  - `…\venv\Scripts\python.exe -B scripts/capture_render_shadow.py --use-shadows false --out-dir fixtures/render-bvh/scene-shadow/capture/shadow-off`
- 窗内只跑 pinned python + `nvidia-smi` 采样 + `scripts/gpu-window-guard.ps1`；**零 Rust 编译**
- 捕获脚本：`scripts/capture_render_shadow.py`（18 828 B / sha256 `8453318da03897993ea2d6b6269347ceb60c8c9a5b5e2f6ece29e02a3a231f57`，Chief 签名后**零编辑**）

## 7. 入库物与登记

- 入库（本提交）：
  - `scripts/capture_render_shadow.py`
  - `fixtures/render-bvh/scene-shadow/scene.xml`
  - `fixtures/render-bvh/scene-shadow/capture/shadow-on/{capture.json,depth.npz,rgb.npz,seg.npz}`
  - `fixtures/render-bvh/scene-shadow/capture/shadow-off/{capture.json,depth.npz,rgb.npz,seg.npz}`
  - 本目录：`README.md` / `receipt.json` / `manifest.txt` / `attempt1/`（6 个窗口产物副本）
- `target/` 只登记不入库：见 `manifest.txt` 尾部「target/ 登记」区块（含原始 CRLF 产物 sha 与路径）
- 副本口径：`attempt1/` 下 6 个文本副本已按 **LF 规范化**（盘上 = committed blob，便于 manifest 自洽）；target 原件（含 CRLF 的 `guard.log` 与两份 `capture-shadow-*.log`）sha 见登记区块

## 8. 限制（必须随证据保留）

- **采样例外**：见 §2，只适用 authoring-only 捕获
- **`capture.json` EOL 属性**：`fixtures/render-bvh/scene-shadow/capture/*/capture.json` 未声明 `text eol=lf`；`core.autocrlf=true` 下 fresh clone 可能被 checkout 为 CRLF。若后续 S3b 需逐字节校验 `capture.json`，需另行补 `.gitattributes` 规则（不属本卡 scope，本卡未改）
- **本窗只有 1 次有效轮**，无失败重试记录；不推断其他窗口行为
- 本片**不产产品数值结论**，也不对上游做数值等价结论；数值验收在 R09-S3b 窗
