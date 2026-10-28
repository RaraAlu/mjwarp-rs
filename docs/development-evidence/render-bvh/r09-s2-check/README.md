# R09-S2 独立复核（内核光照与阴影实现 / 方案 A 复用 first-hit `cast_ray`；无 GPU 窗口）

任务：`2026-10-28-render-bvh-r09-s2-check`（P8 渲染与 BVH）
判据树 HEAD：`1ad2f268`；实现提交：`6b5d6dc6`（20 文件，`git show --shortstat` = +574/−31）
复核对象证据：`docs/development-evidence/render-bvh/r09-s2-lighting-kernel/`（README 57 / receipt 113 / manifest 14 条）
上游锚点：冻结 MJWarp 3.15 `render.py`（`71da24d`）L585/L830/L850/L866/L880/L919-961/L998-1000/L1457-1459

## 0. 结论

**verdict：pass；阻断 0；观察 8 条（1 严重 / 2 一般 / 5 建议）。Arch 2026-10-28 18:4x 已逐条裁决，见 §7 处置列。**

其中 **O1 为「严重但不阻断本片验收」**：R09-S2 卡面契约是「内核实现 + 单测 + 零数值结论」，而 host `use_shadows=true` 前置拒绝是 S7b3 既有的**产品策略**（未验证路径刻意不解锁）。本片交付的是「**分支实现就位、产品路径待解锁**」，**不是**「阴影已可用」；该前置限制必须随证据保留，解锁动作归后续片 `2026-10-28-render-bvh-r09-s2-host-guard-unlock`（R09-S3b 窗口前必须完成）。

本片只实现、不写任何渲染数值结论（数值验收在 S3b 窗）。我未重开门、未重跑 GPU 实体、未开 GPU 计算；除四门禁与「内核编译门」（纯 NVRTC 编译、不发射）外全为只读核对。
下文所有数字均为我自己重算/重跑（`recompute.py` → `recompute.json`，`logs/` 五份），未引用 Backend/Arch 报告值。

## 1. 独立复跑（HEAD `1ad2f268`，工作树仅任务卡/证据未跟踪）

| 门 | 命令（`.normify/gates.toml`） | 退出码 | 我测 wall | 结果 |
| --- | --- | --- | --- | --- |
| fmt | `cargo fmt --check` | 0 | 1.6 s | 无输出（通过） |
| clippy | `cargo clippy --all-targets --locked -- -D warnings` | 0 | 6.1 s | 通过 |
| clippy-cuda | 同上 `--features cuda` | 0 | 0.4 s | 通过 |
| unit | `cargo test --locked` | 0 | 72.2 s（含编译） | **1045 passed / 0 failed / 384 ignored**（129 条 result 行求和） |
| 内核编译门 | `--test render_kernel_rgb -- --ignored megakernel_source_compiles_under_render_gate` | 0 | 5.3 s | 1 passed；`source_sha256=d38d5067…`、`ptx_sha256=1e37c50f…`、`ptx_bytes=259107` |
| 架构树 | `normify validate --fingerprints` | 0 | — | 错误 0；警告 5（既有项，与本片无关） |

- unit 中 `tests/render_lighting_kernel.rs` 二进制 5 passed（本片新增 5 条断言）；相对 R10-S2 的 1040/0/384 增加 5、ignored 不变 → 与 Backend 报的 1045 一致。
- 内核编译门默认 `#[ignore]`（「需要 NVRTC 与 W15 头树」），我显式 `--ignored` 复跑，需 pin 的 `target/toolchains/cuda-12.8.1`（等价 `scripts/g01-fast.ps1` 的 `CUDA_PATH`），**纯编译、不启动内核、不占 GPU 计算**；PTX 哈希与实现证据逐字符相同（我另算 `MEGAKERNEL_SOURCE` 六段拼接连结 sha256 = `d38d5067…`，80305 B，与 receipt 一致）。
- `check_render_ptx`（`src/render/kernel/mod.rs:239`）只放行 2 条 f64 常量与 sin/cos 归约行 → 本片新增代码未引入 f64 或新的乘加表达式，f32 纯度门通过；本片无非数值断言可回归。

## 2. 证据自身复核（实现片 `r09-s2-lighting-kernel`）

- 7 个文件（README/receipt/manifest + 4 份 logs）**CR=0、BOM=0**；manifest 14 条 sha256/bytes 逐条重算 **14/14 相符**，无自引用；`target/r09-s2-prep/scale-constants-check-2.txt`（1896 B）也在盘上且哈希相符。
- 行数（LF 实数）与声称值：`render_shadow.cu` 30、`render_shade.cu` 359、`render_megakernel.cu` 340、`megakernel.rs` 492、`megakernel_arrays.rs` 241、`kernel/mod.rs` 474 全部相符；**`tests/render_lighting_kernel.rs` 实测 144 行，而 README §3 / receipt `changes[].added` / 提交信息均写 145**（观察 O2）。
- 模块源：`mjwarp.render` source 条目 **33→34**（新增 `src/render/kernel/cuda/render_shadow.cu`），`normify validate --fingerprints` 0 错误；说明注册与提交树一致。
- 禁词：任务卡 3 词在证据/改动源码中 0 命中；全局词表只在实现 README 的一句否定式清单中作为反面表述出现（与既往证据同格式，非正面结论）。

## 3. 语义逐项对照（上游 → 产品；只做代码级核对）

| 项 | 上游 | 产品（`render_shadow.cu` / `render_shade.cu` / `render_megakernel.cu`） | 判定 |
| --- | --- | --- | --- |
| 阴影总开关 | L850 `use_shadows` | `use_shadows` 由 host scalar 槽传入（原硬编码 false） | 接线正确 |
| 单灯开关 | L866 `lightcastshadow` | `light_castshadow.data[light]` 逐灯传入 | 正确（单份数组，见 §5 限制） |
| 发射条件 | L919 `if use_shadows and lightcastshadow` | `if (!use_shadows || !cast_shadow) return 1.0f;` | 正确 |
| 原点微移 | L922 `hitpoint + normal*1e-4` | `add(hitpoint, scale(normal, 1.0e-4f))` | 正确 |
| 距离上限 | L924-926 非方向光 `dist_to_light−1e-3`；方向光 `1e8` | `(lighttype == 1) ? 1.0e8f : (dist_to_light - 1.0e-3f)` | 正确 |
| 命中判据 | L960-961 `shadow_geom_id != -1` → `visible = shadow_light_fraction` | `(hit.geom_id != -1) ? shadow_light_fraction : 1.0f` | 正确 |
| 权重合成 | L963 `weight = attenuation * visible` | `weight = attenuation * shadow_visibility(...)` | 正确（false 分支 ×1.0f 精确） |
| headlight | L1457-1459 传 `True, 1, False`（不投影） | `true, 1, …, false` | 正确 |
| 逐灯参数顺序 | L1403-1422 | 调用实参逐项对齐签名（lightactive/type/pos/dir/attenuation/cutoff/exponent/diffuse/specular/normal/hitpoint/view_dir/mat_spec/mat_shin_exp/enable_specular/castshadow/fraction） | 正确，无串位 |
| ambient / per-light ambient | L970 起（不在本片改动面） | megakernel L275-288 未改 | 未受影响 |
| diffuse/specular 分支、cutoff/exponent | L897-969 | `compute_lighting` 主体除 `weight` 一行外零改动 | 未受影响（diff 可证） |

**实现路径差异（如实记录，不写结论）**：上游阴影用的是 `cast_ray_first_hit = _make_cast_ray(..., first_hit=True)`（L999-1000），该变体是 any-hit、命中即返回（L808-811）；产品复用主射线的 closest-hit `cast_ray`（`render_common.cu:304`，非 any-hit、无新增 API）。布尔遮挡目标相同；但复用 closest-hit 带来其「接近 `max_t` 的容差并列」语义：`RAY_TOL_REL=1e-6`/`RAY_TOL_ABS=1e-9`（`render_math.cu:28-29`），命中判定在 `max_t` 外允许 `1e-6*max(max_t,1)` 的带（上游 any-hit 为严格 `d < max_t`）。该差异属容差级、且本片不写数值结论 → 仅记录，数值影响留 S3b 判定。

## 4. 槽位与 81 槽签名（零改动核查）

- `MEGAKERNEL_SLOTS` 长度 **81**（`src/runtime/launcher/render/args.rs:405`）；`use_shadows` 位于 0 基索引 40（`S(SB)`），**槽类型与位置未变**，仅取值由字面 `false` 改为 `resolved.use_shadows`（`megakernel.rs:407`）。
- `shadow_light_fraction` 复用 `splat_scale.data[0][2]`：我扫全部 `.cu`，`splat_scale` 读取点恰为 `[0][1]`（zfar）与新增 `[0][2]` 各 1 处，无重复/越界；Rust 侧 `scale_constants` 仍为 `Vec3 × [1]`（形状未变，故 81 槽签名无需改）。
- `render.cu`(499) / `launch.rs`(500) / `cast.rs`(494) 一字未动（见 §6 diff）；`MEGAKERNEL_SOURCE` 拼接顺序 common→shadow→shade→megakernel，`tests/render_lighting_kernel.rs::shadow_source_precedes_shading_in_concat_order` 有独立断言，我按 `find` 语义复核成立。

## 5. `use_shadows=false` 路径（只做代码级等价论证，不做数值结论）

- `shadow_visibility` 在 `use_shadows=false` 时立即返回 `1.0f`，`weight = attenuation * 1.0f` 对 f32 精确（含 ±0/次正规），随后表达式与旧实现逐项同形 → 代码级等价可证。
- 被删掉的内核早退分支（`if (use_shadows) { return; }`）只在 `use_shadows=true` 时可达；而 host 目前仍拒绝该配置（见观察 O1），故本片不改变任何可达路径的结果。
- **本片未做任何渲染数值验证**；`use_shadows=false` 的逐位回归与阴影像素验收均留 S3b 窗（本行与实现证据 limitations 一致）。

## 6. 红线 / 计数 / 夹具（独立 diff）

`git diff 6b5d6dc6^..1ad2f268` 对下列路径 **输出为空**：`src/render/kernel/cuda/render.cu`、`kernel/launch.rs`、`kernel/cast.rs`、`src/model/render.rs`、`src/render/context/camera_output.rs`、`src/render/bvh/**`、`fixtures/**`、`Cargo.toml`、`Cargo.lock`、`src/runtime/**`。
→ `DEVICE_SOURCES` 仍 28（`src/runtime/collision_gpu/source.rs:51`）、产品入口 6（`abi_tests.rs` 既有断言）、`CollisionArgs` 504 B、`fixtures/precision` 与判据/参考/容差零改动。
行数：`model/render.rs` 500、`camera_output.rs` 499、`render.cu` 499、`launch.rs` 500、`cast.rs` 494 均未超限；本片新文件 30 行、测试 144 行，改动文件最大 492 行。

## 7. 观察清单（严重级倒序；含 Arch 裁决与处置）

- **O1（严重｜裁决：不阻断 + 必须留痕）** 新增的内核阴影分支在 host 路径上**暂不可达**：`MegakernelRenderer::render` 仍在写前拒绝 `resolved.use_shadows == true`（`src/render/kernel/megakernel.rs:97-99`，S7b3 引入、本片未改），故第 40 槽恒为 false；两条既有被 `#[ignore]` 的 GPU 测试仍把该拒绝当契约（`tests/render_kernel_flags.rs:380` 内 L412-435；`tests/render_kernel_rgb.rs:209-224`）。实现证据 README §2 / receipt `slot_reuse.use_shadows` 只说了「host 从硬编码 false 改为 `resolved.use_shadows`」，未说明这一前置拒绝。
  **裁决理由（Arch）**：本片契约是「内核实现 + 单测 + 零数值结论」，前置拒绝是既有产品策略，因此不阻断本片验收；但必须留痕，不得表述为端到端接通。
  **处置**：Arch 在实现片证据补 addendum（`megakernel.rs:97-99` 仍拒、第 40 槽恒 false、两条 `#[ignore]` 断言仍以拒绝为契约、本片不解锁）；登记后续片 `2026-10-28-render-bvh-r09-s2-host-guard-unlock`（**R09-S3b 窗口前必须完成**）：移除 guard + 同步改两条断言 + 头注释更正 + 接线单测。**本复核证据按此保留前置限制，不写「已解锁/已可用」类表述。**
- **O2（一般｜裁决：fix）** 证据数字：`tests/render_lighting_kernel.rs` 实测 **144 行**，实现 README §3 表、receipt `changes[].added`、提交信息均写 **145**。处置：实现片治理提交改 145→144；本复核证据按实测 144 记录（`recompute.json` 的 problems 只含这一条，属实现片文字修正，不阻断）。
- **O3（一般｜裁决：accept 延后）** 5 条单测偏「结构/字符串」：第 1 条用源码子串断言（含一条依赖缩进的 `!contains("if (use_shadows) {\n        return;")`，脆弱）、第 3 条只断言 `ResolvedContext` 字段；**真正的接线**——`shading_words()` 的 `scale_constants[2] == shadow_light_fraction as f32`、以及第 40 槽实参取值——无任何 CPU 测试覆盖（`megakernel_arrays.rs` 无 `#[cfg(test)]`）。处置：5 条结构断言保留不改；「in-crate 单测锁接线」并入 `host-guard-unlock` 片，**不算本片履约缺口**。
- **O4（建议｜裁决：accept 延后）** `render_megakernel.cu:15` 文件头注释仍写 `splat_scale = [znear, zfar, unused]`，而 `[0][2]` 已被改作 `shadow_light_fraction` → 注释过期。处置：随 `host-guard-unlock` 片改；**本片不动 `.cu`**，因为改注释会变 `MEGAKERNEL_SOURCE` 与 PTX sha，与已冻证据面冲突。
- **O5（建议｜裁决：accept）** 实现片 `manifest.txt` 混用相对基准：证据文件相对证据目录、源码/`target/` 留痕相对仓库根，无分隔说明。我按「先仓库根、再证据目录」解析后 14/14 相符（可信）；处置：规则一句话在实现片 manifest 头补，本片接受。
- **O6（建议｜裁决：accept）** 实现片 `logs/tests-and-lint.log` 开头保留了 rustfmt **修复前**的 diff（历史中间态），与最终树 fmt 通过不矛盾（我已复跑 fmt 0），但容易被读成「fmt 曾失败」。处置：本复核证据注明该段为修前历史日志即可，实现片不动。
- **O7（建议｜裁决：accept）** 任务卡 acceptance#1 写「拆分核验」，但本片**没有文件拆分/移动**（新增影子源文件 + 追加逻辑），该项无对象；实现证据也未说明「本片无拆分」。处置：实现片证据补一句「本片无拆分对象、贴线文件零改动」，不改卡。
- **O8（建议｜裁决：accept）** 实现片 receipt `head` = `5cc6a3d5`（证据写作时的 HEAD，提交前），与判据树 `1ad2f268` / 实现 `6b5d6dc6` 不同。属「证据时点」而非「最终树绑定」的字段口径；处置：实现片补字段口径说明，本片接受。

## 8. 边界（我**没有**验证的）

- 未开 GPU 窗口、未发射任何渲染内核：世界内无阴影像素、阴影可见度、`use_shadows=false` 逐位回归、`dist_to_light` 截止行为、容差带影响等**全部未做数值验证**，按计划留 S3b。
- 内核编译门只证明「源码可编译 + PTX 形状/hash 可复现」，不证明运行期正确性。
- 未检查上游 GPU 后端与产品在阴影叠加下的差异（S3b 面）；未做实机 GPU 兼容矩阵。
- 本复核不含任何正面验收结论；证据中不写窗口/覆盖/精度/等价类表述。

## 9. 复算入口

```text
python target/.../venv312/Scripts/python.exe docs/development-evidence/render-bvh/r09-s2-check/recompute.py
```

`recompute.json` 内含：证据 EOL/BOM、manifest 逐条、行数对照、拼接 sha256、禁词命中、槽位/读取点/守卫、红线 diff、改动清单、problems 列表（当前 problems = 1，即 O2 的行数不符）。

`manifest.txt` 规则：证据目录内文件按工作树复核；仓库内被复核文件按 **committed blob**（`git show HEAD:<path>`，LF 归一）记录/复核——本仓库既有 EOL 状态下 `megakernel_arrays.rs` 工作树为 CRLF、blob 为 LF（10168 B），按 blob 记录才能对提交内容成立。
