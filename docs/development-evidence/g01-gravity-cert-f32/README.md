# G01 重力位证书：f32 逐步口径重生成（W-D4 口径第 1 项）

任务：`2026-10-22-g01-gravity-cert-f32-regen`
基线：`da7a840`
裁决来源：用户 2026-10-22 裁决 W-D4 口径第 1 项——按上游同样的 f32 逐步舍入口径重生成证书。
前置：`2026-10-22-g01-numerics-audit`（2 槽各 4 ulp 来自 f32 输入/中间舍入，非归约顺序/设备问题）。

## 独立性

- 生成器 `gen.rs` 只读取测试输入（`multi_body_model()` 拓扑/位置、gravity/body_mass/body_gravcomp 参数批）与上游公式；
  **不读取 GPU 实际输出、不回读产品快照作为期望**。
- 自检：生成器同时用精确有理算术复算旧证书，28/28 槽逐位复现旧精确证书（证明公式与输入解析正确），再输出 f32 逐步证书。
- 旧证书不删除：归档 `legacy-exact-certificate.json`，并在测试注释与回执中写明口径变更依据。

## 上游公式与常量来源（71da24d956378a87a703b6e1442b13aec0c4ac29）

| 步骤 | 来源 |
| --- | --- |
| `_subtree_com_init/_subtree_com_acc/_subtree_div`（子树 COM：init 乘质量、树序累加、除子树质量） | `target/mjwarp-parallel/upstream/mujoco_warp/_src/smooth.py`（冻结上游） |
| `_cdof`（offset = subtree_com − xanchor；hinge 线性分量 = cross(axis, offset)） | 同上 |
| `gravity_compensation_compute`（force、jacobian、点积、atomicAdd） | `src/physics/gravity_compensation/cuda.rs`（来源注释 passive.py/support.py）；`_src/passive.py`/`_src/support.py` 基线同一提交 |

## f32 语义与舍入点

输入全部是精确可表示的 f32（体位置/`body_ipos` 整数，mass/gravcomp 为 2 的负幂或小整数，gravity 为 0/±0.5/±1）。
逐步舍入点（`certificate.json` 每个 world 均有 exact 有理值与 f32 bits）：

1. `xipos`：父体位置 + body_pos + body_ipos，f32 加法。
2. 子树 COM：`xipos * body_mass`（f32 乘）、子体向父体树序累加（f32 加）、除以子树质量（f32 除）。
   本模型的输入值在除法前均可精确表示；舍入只发生在最后除法，例如 `17/7 = 2.428571…` → `0x401b6db7`。
3. `cdof`：`offset = subtree_com − xanchor`（f32 减，唯一舍入点，例如 `−25/7` → `0xc0649249`）；
   hinge 线性分量 `cross(axis, offset)` 的乘减均精确（axis=z，offset 只有 x）。
4. 每个贡献体：`force = ((−gravity)*mass)*gravcomp`（f32 乘，逐步）；
   `offset = xipos − subtree_com`（f32 减，例如 `32/7` → `0x40924924`）；
   `jacobian = cdof + (cdof[2]*ox − cdof[0]*oz)` 等（f32 乘/减/加，例如 `0.99999976… = 0x3f7ffffc`）；
   `product = jx*fx + jy*fy + jz*fz`（f32 乘与左结合加）。
5. `atomicAdd`：单贡献者等价一次写入；多贡献者按 body 升序模拟，并在 GPU 测试中逐位复核（生成器不读取该结果）。
6. 该 kernel PTX 无 `fma.rn.f32`（审计已统计），因此按独立 mul/add/sub/div 建模。

## 新旧差异

| 槽 | 旧精确证书 | f32 逐步证书 | 差异 |
| --- | --- | --- | --- |
| world 1 dof 2 | `0x3f000000`（0.5） | `0x3efffffc`（0.49999988079071045） | 4 ulp |
| world 4 dof 2 | `0x3f000000`（0.5） | `0x3efffffc`（0.49999988079071045） | 4 ulp |
| 其余 26 槽 | — | — | 逐位一致 |

`max_abs = 1.1920928955078125e-7 ≤ 2.0e-6`（原容差），差异根因与审计结论一致：
`subtree_com 17/7`、`cdof −25/7`、力臂 `32/7` 的 f32 表示与精确值差约 3e-8，力臂相减后放大到 −2.7e-7，最终雅可比乘力得到 4 ulp 低值。

## 生成与复现

```powershell
rustc -O docs/development-evidence/g01-gravity-cert-f32/gen.rs -o target/g01-cert/gen.exe
& target/g01-cert/gen.exe docs/development-evidence/g01-gravity-cert-f32
```

输出：

- `certificate.json`：28 槽期望 bits、逐槽 contributors（force/offset/jacobian/product 的 exact 与 f32 bits）、子树 COM/cdof 中间量、legacy 对照与 differences。
- `legacy-exact-certificate.json`：旧精确证书归档与差异依据。
- `CERTIFICATE.md`：新旧 bits 对照表。

测试 `src/physics/gravity_compensation/tests.rs` 直接 `include_str!` 该 `certificate.json` 并断言
`semantics == "upstream-formula + stepwise f32 rounding; independent of GPU outputs"` 与 2 处固定差异；
零位差断言、2.0e-6 容差与内核算子均未修改。

## 验证

| 项 | 结果 |
| --- | --- |
| 生成器自检（精确有理复算旧证书） | 28/28 槽逐位一致 |
| `cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --lib -- --ignored gravity_compensation::tests::gpu_matches --nocapture --test-threads=1` | 1 过 0 败，`bit_differences=0`、`max_absolute=0`、`max_ulp=0` |
| 报告 | `gpu-verification-report.json` |
| all-probes 复跑 | 见 `docs/three-layer-acceptance.md` §4.1（该项 0 红） |

## 边界

- 只改证书期望值的生成口径（测试常量改读证书 JSON）；未改容差、未删除断言、未改内核/PTX 生成策略/fixtures。
- W-D4 的结论仅覆盖此诊断证书；不宣称 NP01–NP04、精度或性能验收。
- 未跑 release 与 Linux。
