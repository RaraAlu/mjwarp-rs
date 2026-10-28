# R05 地形 hfield 渲染家族 · 整片汇总

任务：`2026-10-29-render-bvh-r05-hfield-closeout`（owner Arch）
状态：R05 实现 / 采集 / 两次产品修复 / 验证窗全部 verified；本文只做汇总，不改代码、判据、容差、参考或夹具。
窗口：R05-S3a 采集窗 attempt=1（authoring-only 例外 util<60）；R05-S3b 验证窗 attempt=2（validation-only 例外 util<60，实际生效）。

## 1. 卡片清单（逐张）

| 卡片 | 状态 | 证据 |
| --- | --- | --- |
| `2026-10-28-render-bvh-r05-s1-hfield-params` | verified | `r05-s1-hfield-params/` |
| `2026-10-28-render-bvh-r05-s1-hfield-params-check` | verified | `r05-s1-hfield-params-check/` |
| `2026-10-28-render-bvh-r05-s1-hfield-hardening` | verified | `r05-s1-hfield-hardening/` |
| `2026-10-28-render-bvh-r05-hfield-channel` | verified | `r05-hfield-channel/` |
| `2026-10-28-render-bvh-r05-hfield-channel-check` | verified | `r05-hfield-channel-check/` |
| `2026-10-28-render-bvh-r05-split-megakernel` | verified | `r05-split-megakernel/` |
| `2026-10-28-render-bvh-r05-split-megakernel-check` | verified | `r05-split-megakernel-check/` |
| `2026-10-28-render-bvh-r05-split-cast` | verified | `r05-split-cast/` |
| `2026-10-28-render-bvh-r05-split-cast-check` | verified | `r05-split-cast-check/` |
| `2026-10-28-render-bvh-r05-s2-hfield-intersect-r2` | verified | `r05-s2-hfield-intersect/` |
| `2026-10-28-render-bvh-r05-s2-hfield-intersect-check` | verified | `r05-s2-hfield-intersect-check/` |
| `2026-10-28-render-bvh-r05-s2-o1-fix-check` | verified | `r05-s2-o1-fix-check/` |
| `2026-10-28-render-bvh-r05-s3a-hfield-capture` | verified | `r05-s3a-hfield-capture/` |
| `2026-10-28-render-bvh-r05-s3a-hfield-capture-check` | verified | `r05-s3a-hfield-capture-check/` |
| `2026-10-28-render-bvh-r05-s3b-hfield-target-authoring` | verified | `r05-s3b-hfield-target-authoring/` |
| `2026-10-28-render-bvh-r05-s3b-hfield-target-authoring-check` | verified | `r05-s3b-hfield-target-authoring-check/` |
| `2026-10-28-render-bvh-r05-s3b-hfield-empty-mesh-fix` | verified | `r05-s3b-hfield-empty-mesh-fix/` |
| `2026-10-28-render-bvh-r05-s3b-hfield-cam1-depth-rca` | verified | `r05-s3b-hfield-cam1-depth-rca/` |
| `2026-10-28-render-bvh-camera-pose-warp-fma-align` | verified | `r05-s3b-camera-pose-warp-fma-align/` |
| `2026-10-28-render-bvh-camera-pose-warp-fma-align-check` | verified | `r05-s3b-camera-pose-warp-fma-align-check/` |
| `2026-10-28-render-bvh-r05-s3b-hfield-validate` | verified | `r05-s3b-hfield-validate/` |
| `2026-10-28-render-bvh-r05-s3b-hfield-validate-check` | verified | `r05-s3b-hfield-validate-check/` |

放弃 / 未入库（按 normify 规则保持工作树未跟踪，板面可读）：

| 卡片 | 状态 | 说明 |
| --- | --- | --- |
| `2026-10-28-render-bvh-r05-s1-hfield-host` | abandoned | 早期 S1 卡，因签名口径变更改为 params/hardening |
| `2026-10-28-render-bvh-r05-s2-hfield-intersect` | abandoned | 运行中 scope 冻结，同片重登记为 `-r2`；其 `.normify/runs/` 亦未跟踪 |
| `2026-10-28-render-bvh-r05-s3b-hfield-empty-mesh-fix-check` | abandoned | 见 §7 放弃说明，**不写成「已复核」** |
| `2026-10-28-render-bvh-r09-s2b-e2e-negative-control` | abandoned | R09 派生卡，未启动 |
| `2026-10-28-arch-volume-module-sources` | backlog | `src/render/kernel/` 6 个 .rs 未登记模块源（治理项） |
| `2026-10-29-render-bvh-host-quat-derivation-audit` | backlog | 其它 host 侧 quat→matrix 派生审计 |
| `2026-10-29-normify-archived-card-ingest` | backlog | 历史卡正规归档入库途径 |

已知仓库状态限制：上述 abandoned / backlog 卡与 abandoned 卡的 runs 目录**无法按现有 hook 入库**（`E change/transition`、`E commit/status`、`E commit/scope`，且 `normify` 无 archive 子命令），按规则保持未跟踪，不入库、板面可读。唯一必须入库的是 `…-hfield-validate-check`，由 QA 收口时随其提交入库。

## 2. 交付数字

### 2.1 验证窗 attempt=2（正式验收轮，`target/render-bvh/r05-s3b-window/attempt2/`）
- 9 实体 **9/9 exit 0**；`WINDOW_RUN_DONE`；body_wall 51.2 s
- `sha_guards=73/73 (9 exes + 64 read-only inputs), mismatches=0`
- `post_window_exe_recheck=9/9 mismatches=0`；`zero_rust_compile=True`（deps 前后均 1836、max write 04:08:28）
- 采样：samples=171、util 11–43%、`util_streak_ge_80=0`、free_min=13 035 MiB、used_peak 3 027、residual_processes=0
- 基线/floor：baseline_free 13 305 → `floor=min(10240, 13305−3072)=10 233`，未破
- 两组 hfield 判据（每组 2 相机 ×120 000 像素，全 0 差异）：
  - hfield-only：cam0(76 800)/cam1(43 200) RGB `mismatched=0 / max_byte_diff=0 / beyond_1_lsb=0`；depth `mismatch=0 / max_ulp=0 / beyond_ulp=0`；seg `mismatch=0`；depth_zero 29 776 / 37 050
  - hfield-primitive：同上全 0（depth_zero 29 776 / 37 050）
  - 两次运行三 sha 相等：hfield-only `c58b9f17…`/`87fb8b3a…`/`727bcf2b…`；hfield-primitive `0d494b5a…`/`e8ca22fe…`/`343c8978…`

### 2.2 采集（R05-S3a，attempt=1）
- 8 文件 **5 784 264 B**（两组各 4：rgb/depth/seg npz + capture.json）
- 场景：`scene.xml` `b3b9a6b4…`/3 590 B、`scene-mixed.xml` `bf9bb966…`/3 180 B；脚本 `capture_render_hfield.py` `cb1808f8…`/20 314 B/425 行（`--use-fast-math` 默认 false）
- 旧参考零读写：38 文件 / 25 320 515 B / 聚合 `b5cb50af…` 前后一致

### 2.3 两次产品修复
- 零 mesh 发射修复 `dce78b1f`（+`e262e682`/`7c70a44f`）：`megakernel_uploads.rs` 405→**446 LF**、`megakernel.rs` 404→**412 LF**；内核 `source 72bc364c…` / `ptx 97467d83…` / 264 015 B **逐位不变**；unit 1092
- 相机世界位姿 FMA 对齐 `6f246b46`（+`d42a465a`/`f98d74a5`）：`cast.rs` 411→**481 LF**（+72/−2）；cam_xmat **9/9 两相机位等**；world dir cam1 **42 449→0**、cam0 **0/76 800**；1003 条 ×9=**9027 项 0 差异**（候选空间 1 259 712、≤16 条收敛、plain 公式同批差 2614 项/947 条）；scene-a `cam-persp` **9/9**（修复前 2/9）；内核三 sha 不变
- 唯一性依据：`sm89.ptx` 的 `quat_to_mat` 切片 **`fma.rn` 计数 0**（`mul_quat` 片 6 条）⇒ 收缩在 ptxas→SASS，故以 1003 条 pinned GPU 参考反推

## 3. 例外与无效轮（不得写成常规通过）

| 轮次 | 结果 | 说明 |
| --- | --- | --- |
| 采集 attempt=1 | 达标（60/60） | **authoring-only 例外 util<60**；前三次常规 util<40 监测失败（最长 24 / 34 / 39 of 60）原样保留 |
| 验证 attempt=1 | **INVALID_ROUND** | 零 mesh 场景发射 panic（产品缺口），记录保留、目录未覆盖 |
| 验证 attempt=2 常规监测 | **STOP** | 354 样本、最长连续干净 45/60、util 1–49、ge40=33；未开窗、`window_commands_executed=0` |
| 验证 attempt=2 例外监测 | 达标（60/60） | **validation-only 例外 util<60**：util 36–41、ge40=5、ge60=0、free_min 13 271 |
| 验证 attempt=2 窗口 | 绿 | 以 `-OpenUtilLimit 60` 执行；**例外生效的机器证据**：例外监测 CSV 中 5 个 `util≥40` 样本（04:50:55=40 / 04:52:32=41 / 04:54:24=41 / 04:54:55=40 / 04:55:45=40）被记 `clean=True` 且 streak 连续增至 59→60 ⇒ 生效门槛为 **60**。窗口 `guard.log` 的 precheck 三样本是 **37/37/37**，`baseline.txt` 三点是 **37/40/40**（两者是不同采样，`run-window.ps1` L111-129 vs L131-135），**不能**用 precheck 证明例外生效。`not_a_normal_threshold_pass=true`、`authoring_exception_used=false` |

理由（写进证据）：桌面常驻图形外载（`filtered compute-apps=0`、build=0、free 充足），窗口判据为位级 + 两次运行 sha 决定性校验，图形外载不改变算术结果。**判据/容差/参考/夹具本次全程零改动。**

## 4. S2 附录更新
S2 证据 §9 原有「S2 hfield 设备通路未验证」为历史记录，**保留不改**；其后追加（`06cba487`）：「该通路已由 R05-S3b attempt=2 验证窗验证」。`receipt.limits` 第 9 条同义追加。此更新不改判据/容差/参考/夹具。

## 5. 每次暴露都留痕的两条根因链
1. **零 mesh 发射**：`nmesh=0` ⇒ 6 个 mesh 家族槽零长度 ⇒ `TransferSession::upload` 不分配（allocation=None）⇒ `launcher_span` 报 `launcher_buffer`；另 `launcher/render/args.rs` 拒零维 shape ⇒ 必须「1 元素占位 + shape≥1」。此前所有场景恰有 1 个 `<mesh>`，故从未暴露。
2. **相机世界位姿 1–2 ULP**：上游 `quat_to_mat` 在 GPU 上以 `fuse_fp=true` 收缩、我们 host 不收缩 ⇒ 非平凡四元数差 1–2 ULP ⇒ world dir 差 ⇒ 掠射 depth ≤7 ULP。cam0 四元数为精确恒等故此前全等；旧位等实体走参考 `camera_data` 位姿直传，从未让产品公式进入位级比较（scene-a 重放 2/9 ULP 为证）。

## 6. 门禁与冻结面最终值
- guards：`scripts/validate/render-bvh/guards.json` = **`e2444a96…`/27 670 B/attempt=2**/aggregate **`7d24f036…`**/`generated_at 2026-10-29T04:10:37.668+08:00`；旧 guards `210f8208…`/27 670 B 标 superseded
- 9 实体 exe：`render_hfield_reference a48f9cc4…/100 378 534`、`camera_rays 45254172…/90 221 740`、`context_lifecycle b7516685…/86 701 423`、`e2e 44fecfb2…/95 166 724`、`kernel_depth_seg 48764f97…/91 134 761`、`kernel_flags 64c050fa…/96 591 181`、`kernel_rgb 202365aa…/94 638 769`、`r08_cameras 6ebecc2f…/94 971 054`、`r10_multi_world db1baa39…/94 732 459`
- 脚本：`freeze-guards.ps1` `b01abff2…`/7 803 B（`-Attempt` 参数）、`run-window.ps1` `31ac28bf…`/18 187 B、`monitor-until-clean.ps1` `788d3505…`/2 509 B
- 64 只读输入与 attempt1 归档 guards **逐项 64/64 全等**；capture 8/5 784 264 B、scene-shadow 16/11 561 363 B、legacy 38/25 320 515 B
- 不变量：`DEVICE_SOURCES`=28、入口 6、81 槽、`CollisionArgs`=504 B、内核 `source 72bc364c…`/`ptx 97467d83…`/264 015 B 全程未变；unit 1092 passed / 0 failed / 389 ignored
- 行数棘轮未破：`render.cu` 499、`launch.rs` 500、`model/render.rs` 500、`camera_output.rs` 499、`cast.rs` 481、`megakernel_uploads.rs` 446、`megakernel.rs` 412、`hfield_scene.rs` 499

## 7. 独立复核（QA verdict）
- R05-S3a 采集复核 `462e48c1`/`cbb04758`：**pass / 阻断 0 / 观察 4**
- R05-S3b 目标实体 authoring 复核 `36fa461e`/`0f35d5c2`：**pass / 阻断 0 / 观察 4**
- 相机世界位姿 FMA 对齐复核 `043927a2`/`d1bd6b0d`：**pass / 阻断 0 / 观察 9**（QA 独立用精确有理数 f32 fma 复现唯一性、自建候选族 1 259 712、9 元素各唯一候选、9027 项 0 差、scene-a 9/9、内核三 sha 不变）
- R05-S3b 验证窗复核 `2026-10-28-render-bvh-r05-s3b-hfield-validate-check`：**pass / 阻断 0 / 观察 10**（QA `5100732f` + closing）。独立复核关键数字：9 实体 exit 0、`sha_guards=73/73`、`post_window_exe_recheck=9/9`、`zero_rust_compile=True`、floor 10 233 未破；两组 hfield cam0/cam1 的 RGB/depth/seg 全部 0 差异且两次运行 sha 相等；例外合法性：常规 util<40 失败记录原样保留、例外记录五要素齐全、**例外门槛 60 的机器证据为例外监测 CSV 中 5 个 util≥40 样本被记 clean=True 且 streak 增至 59→60**（注意：precheck 采样为 37/37/37、baseline 为 37/40/40，二者是不同采样，**不能**用 precheck 证明例外生效）；attempt1 目录原样未覆盖。**O1（precheck/baseline 口径误述）与 O2（monitor sha 与例外命令留痕）必须修，已由 validate 证据修正提交 `9a44f0ae` 全量修掉（O1–O6/O8/O10；O7/O9 仅记录）。**
- **放弃说明**：`2026-10-28-render-bvh-r05-s3b-hfield-empty-mesh-fix-check` 已被 abandon，**未做独立复核**；理由=该修复已 verified 且被 attempt2 窗口全绿实证、同类判据（位级断言/零外溢/内核 sha 不变）已由相机位姿修复的 QA 复核覆盖。**不得表述为「已复核」。**

## 8. 遗留与 backlog
- **SDF `gtype==8` cull 缺口**（`2026-10-28-render-shadow-sdf-gtype-cull-gap`，backlog）
- **host quat→matrix 派生审计**（`2026-10-29-render-bvh-host-quat-derivation-audit`，backlog）：其它 host 派生位姿（geom/light/body）是否同口径差
- **历史卡归档入库途径**（`2026-10-29-normify-archived-card-ingest`，backlog）
- **模块源治理**（`2026-10-28-arch-volume-module-sources`，backlog）：`src/render/kernel/` 下 6 个 .rs 未登记
- **驱动/toolchain 依赖**：相机位姿收缩序来自本栈 ptxas/driver（596.36）对 pinned 上游代码的 SASS 级收缩，驱动或工具链变化可能需重推（脚本+种子已入库）
- **仅固定相机**：`CameraPose::from_fields` 只覆盖 world 体恒等四元数的固定相机；运动体相机乘 `xquat_world` 未覆盖
- **后续家族**（R06/R07/R02 等）与 mesh multicontact 方向等用户拍板

## 9. 非声明
本汇总只做证据索引与数字汇总，**不作验收性结论**：不声明渲染全量验收完成、不声明精度达到目标值、不声明与上游结果一致、不声明产品渲染验收已完成。所有数字来自对应 verified 任务的证据与窗口留档；R05 的验收结论以 `r05-s3b-hfield-validate` 及其 QA 复核卡为准。
