# perworld L v2 残余像素 P9：spot 灯 cos_theta 的 dot 次序 pin（修复卡证据；零 GPU）

- 任务：`2026-11-11-render-perworld-l-v2-cos-theta-dot-pin`（owner Backend；p85；P8-渲染与BVH）。
- 依据：Arch 站点审计 `target/render-bvh/gl2-ptx-diff/site-conformance-audit.md` §B（参考真值=`reference_li_128.ptx` 指令 8715-8720；pinned upstream `mujoco_warp/_src/render.py:907` = `wp.dot(-L, lightdir)`）。参考 = **3×plain `mul.f32` + 1 `neg` + 2 `sub.f32`（无 fma）**，数值 = `-(rnd(rnd(Lx·dx + Ly·dy) + Lz·dz))`；产品 `render_shade.cu:426` 原为 `dot3_ba(-L.x,-L.y,-L.z, lightdir...)`（1 mul + 2 fma 收缩）。
- 改动只涉及 `render_shade.cu` spot 分支一行 + 注释；**不改判据/参考/容差/相机/recipe**；零 GPU、不采集、不跑诊断；未动冻结件/guards。
- 非声明：本卡不宣称修复生效（需 Chief 诊断轮 #6 token 验证残余像素是否归零）；不改变 G-L2 判定（判据 A FAIL 维持）；内核身份变化属实现修复新基线。

## 1. 改动（1 个文件，1 行 + 4 行注释；diff `logs/src.diff`）

### 1.1 变量来源与参考映射核对

| 仓库量（`render_shade.cu` spot 分支） | 参考（`render.py:907` / PTX 8715-8720） |
| --- | --- |
| `L` = `delta/norm` 三次除法（P4 落地；PTX `div.rn.f32` ×3，loc 1881） | `wp.dot(-L, lightdir)` 的 `-L` |
| `lightdir`（`light_xdir_in` 逐灯传入；PTX `ld.global.f32` ×3，loc 2289） | `lightdir` |
| 原实现 `dot3_ba(-L.x,-L.y,-L.z, lightdir...)` = `fma(-Lz,dz, fma(-Lx,dx, mul(-Ly,dy)))` | 参考 = `mul;mul;neg;sub;mul;sub`（无 fma） |

`dot3_ba` 本体、`cos_cutoff = cosf(lightcutoff_rad)` 比较、`attenuation * powf(fmaxf(cos_theta,0),lightexp)` 组合均**未动**；spot 分支的 `L` 归一化（P4/P6）未动。

### 1.2 代码改动

```diff
         if (lighttype == 0) {
-            float cos_theta = dot3_ba(-L.x, -L.y, -L.z, lightdir.x, lightdir.y, lightdir.z);
+            // P9：参考真值 `wp.dot(-L, lightdir)` 为 3×plain `mul.f32` + 1 `neg` + 2 `sub.f32`（**无 fma**），
+            // 数值 = `-(rnd(rnd(Lx·dx + Ly·dy) + Lz·dz))`；`neg` 精确，可外提。
+            float cos_theta = -__fadd_rn(
+                __fadd_rn(__fmul_rn(L.x, lightdir.x), __fmul_rn(L.y, lightdir.y)),
+                __fmul_rn(L.z, lightdir.z));
```

### 1.3 文件 sha/bytes（old → new）

| 文件 | old sha256 / bytes | new sha256 / bytes |
| --- | --- | --- |
| `render_shade.cu` | `bdf6c52778732b83a6f721900123565f2551e27380c702968a2bec63e07c3f27` / 21268 | `7c4ee441004ab88b16c560d11f51476ef2f99f2322cb33cece30d74bb0a0719f` / 21562 |

改动统计（实际 `git diff --numstat`）：1 个文件，**+5 / −1**（1 行代码 + 4 行注释；文件 468 行 ≤500）。其余 `.cu` 全部零改动。

## 2. 内核身份新基线（`logs/kernel-identity-old-new.txt`；NVRTC 12.8.93 纯编译，无 GPU 会话）

| artifact | 值 | old | new |
| --- | --- | --- | --- |
| megakernel | source sha256 | `eb59dd78…bb051ee` | **`02160146…e260a1`** |
| megakernel | PTX sha256 / bytes | `e91b7d85…e17fac` / 344,175 | **`7fd86b85…995c7e`** / **344,181** |
| aa_resolve | 三值 | `0bf8af5a…` / `b28e155c…` / 27,536 | 未变 |
| depth_seg | 三值 | `b546ef08…` / `e8d84963…` / 138,540 | 未变（`render_shade.cu` 不在 DEPTH_SEG_SOURCE 拼接中） |

- 采集/复算：两个 identity 用例各 1 passed；按 `mod.rs` `include_str!` 顺序独立拼接复算均与打印值逐位一致（`logs/old-source-recompute.txt`、`logs/new-source-recompute.txt`）。

## 3. 离线 NVRTC PTX 证明（零 GPU；`logs/p9-site-proof.txt`）

用离线 NVRTC harness（product 选项 + `--generate-line-info`）编译 old/new 拼接源码，站点指令窗口：

```text
old (dot3_ba 收缩，loc 1891 = render_shade.cu cos_theta 行):
   neg.f32    %f3326, %f5157            ; -L.y
   neg.f32    %f3328, %f5156            ; -L.x
   neg.f32    %f3330, %f5158            ; -L.z
   mul.rn.f32 %f3327, %f3326, %f912     ; mul(-L.y, dy)
   fma.rn.f32 %f3329, %f3328, %f911, %f3327   ; fma(-L.x, dx, ·)
   fma.rn.f32 %f936,  %f3330, %f913, %f3329   ; fma(-L.z, dz, ·)

new (P9 显式 intrinsics，loc 1893-1896 = 本行/操作数行):
   mul.rn.f32 %f3326, %f5157, %f912     ; L.y * dy
   mul.rn.f32 %f3327, %f5156, %f911     ; L.x * dx
   add.rn.f32 %f3328, %f3327, %f3326
   mul.rn.f32 %f3329, %f5158, %f913     ; L.z * dz
   add.rn.f32 %f3330, %f3328, %f3329
   neg.f32    %f936,  %f3330
```

寄存器来源（lineinfo 实证）：`L.x/.y/.z = %f5156/%f5157/%f5158`（loc 1881 三次 `div.rn.f32`，spot 分支 `L = delta/norm`）；`lightdir.x/.y/.z = %f911/%f912/%f913`（loc 2289 `ld.global.f32`，`light_xdir_in`）。

数值等价：参考 `mul;mul;neg;sub;mul;sub` = `-(rnd(rnd(m1+m2)+m3))`（m1/m2/m3 各自舍入；`neg` 精确，`a-b` 与 `a+(-b)` 舍入同幅）；new = `mul.rn×3 + add.rn×2 + neg` 同样在每对加法处各舍入一次 ⇒ **与参考逐位等价**；old 把两次乘融进 fma（少一次舍入）⇒ 可能差 1–2 ULP。

## 4. 构建与静态验证（零 GPU）

| 项 | 结果 |
| --- | --- |
| `cargo build --locked --features cuda-probe` | exit 0（`logs/build-cuda-probe.log`） |
| `cargo test --locked --features cuda-probe --no-run` | exit 0（`logs/no-run-cuda-probe.log`） |
| 四门禁 fmt / clippy / clippy-cuda / unit | 全过；unit **1,387 过 / 0 败 / 402 ignored** |
| 内核身份用例（NVRTC 纯编译） | 2 个用例各 1 passed / 0 failed |
| 离线 PTX 站点窗口 | PASS（`logs/p9-site-proof.txt`，old/new 序列与寄存器映射自洽） |

## 5. 边界与非声明

- 边界：只改 `src/render/kernel/cuda/render_shade.cu` 一行 + 注释并新增本证据目录；`unchanged fixtures/`、`unchanged tests/`、`unchanged scripts/`、其余 `.cu` 文件与既有证据目录只读；零 GPU、不采集、不跑诊断、未改 guards/冻结 stash、未重编译冻结 exe 证据。
- 非声明：不宣称修复生效（需 Chief 诊断轮 #6 token 验证）；不改 G-L2 判定（判据 A FAIL 维持）；不改判据/阈值/参考/容差；诊断轮/重冻/G-L2 attempt 3 由 Chief 另发 token。
