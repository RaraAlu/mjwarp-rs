# 本地调试事实（**本地调试、非验收**；数字只作调试信息）

命令与窗口 runner 同环境（`PATH` 前置 `target/toolchains/cuda-12.8.1/bin`）：

```text
cargo test --locked --features cuda-probe --test render_hfield_reference -- --ignored --test-threads=1 --nocapture
cargo test --locked --features cuda-probe --test <8 条回归> -- --ignored --test-threads=1
```

## 1. hfield 两组判据（修复后，两组均全绿）

| 组 | cam | RGB | seg | depth |
| --- | --- | --- | --- | --- |
| hfield-only | cam0 (76800 px) | mismatched=0、max_byte_diff=0、beyond_1_lsb=0 | mismatch=0 | **mismatch=0、max_ulp=0、beyond=0**（zero=29776） |
| hfield-only | cam1 (43200 px) | mismatched=0、max_byte_diff=0、beyond_1_lsb=0 | mismatch=0 | **mismatch=0、max_ulp=0、beyond=0**（zero=37050） |
| hfield-primitive | cam0 (76800 px) | 同上 0 | mismatch=0 | **mismatch=0、max_ulp=0、beyond=0** |
| hfield-primitive | cam1 (43200 px) | 同上 0 | mismatch=0 | **mismatch=0、max_ulp=0、beyond=0** |

修复前（RCA 基线，`docs/development-evidence/render-bvh/r05-s3b-hfield-cam1-depth-rca/`）：
hfield-only cam1 depth **mismatch=4423、max_ulp=7、beyond=20**；hfield-primitive **4389/7/20**。
本片修复后两组 4 个相机全部 **0/0/0**。

两组 `test result: ok`；`render_hfield_reference` 实体 exit 0（原始输出
`materials/hfield-final.log`）。运行 sha256：hfield-only `rgb=c58b9f17… depth=87fb8b3a… seg=727bcf2b…`；
hfield-primitive `rgb=0d494b5a… depth=e8ca22fe… seg=343c8978…`。

## 2. 8 条回归（修复后）

`render_camera_rays` / `render_context_lifecycle` / `render_e2e` / `render_kernel_depth_seg` /
`render_kernel_flags` / `render_kernel_rgb` / `render_r08_cameras` / `render_r10_multi_world`
= **8/8 exit 0**（含 R08/R09 相机与 e2e 的位级/容差断言原样通过）。

## 3. world dir 位差（本片决定性指标，host 同口径复算）

口径与 RCA `kernel_vs_upstream.py` 一致：同一份产品 local dirs（与上游 120000 条位级全等），
分别用「我们的 cam_xmat」与「上游 warp cam_xmat」做行点积（同一 plain f32 顺序），比较 3 分量位型。

| 相机 | 修复前 | 修复后 |
| --- | --- | --- |
| hfield cam1 | **42449 / 43200** | **0 / 43200** |
| hfield cam0 | 0 / 76800 | **0 / 76800** |

（`materials/world-dir-counts.json`；cam0 修复前即 0，因为恒等四元数两边都得精确单位矩阵）

## 4. scene-a `cam-persp` 重放

新 host 公式 vs 已 verified 的 `camera_data_check[0].xmat_bits`：**9/9 位级相等**
（修复前 2/9 差 1 ULP：index 0/7）。见 `materials/cam-xmat-bits.json`。

## 5. 三条硬门（GPU 参考）

| 门 | 结果 |
| --- | --- |
| 硬门 1：hfield cam0/cam1 `cam_xmat` | **9/9、9/9** 位级相等（对 pinned chain GPU 参考 + capture 位型） |
| 硬门 2：≥1000 随机四元数 | 1003 条 × 9 元素 = **9027 项 0 差异**（seed 20261029，可复现） |
| 硬门 3：scene-a `cam-persp` | **9/9** 位级相等（对已 verified 参考） |

反例对照：同一 1003 条上，未收缩代数展开版差 **2614 项 / 947 条**。

## 6. 数字性质

以上全部为**本地调试、非验收**数字；不构成窗口证据，不代表 attempt=2 结果。
