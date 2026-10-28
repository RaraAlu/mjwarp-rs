# R02-S1 天空盒离线实现 · QA 独立复核

任务卡：`2026-10-29-render-bvh-r02-skybox-s1-check`（priority 57，Owner QA）
被核对象：`2026-10-29-render-bvh-r02-skybox-s1`（**verified**）
被核提交：`8aa33e00`（代码+测试+模块源登记+卡）→ `b6b2a679`（证据）→ `299459c0`（收口）
复核基线 HEAD：`db6ca47d`。**只读**：未开 GPU 窗口、未跑任何设备数值、未改代码/证据。

---

## 1. 结论

**verdict：pass / 阻断 0 / 观察 8。**

实现与 pinned 上游逐行对齐、非天空盒路径的「零改」静态论证成立且**未越界宣称实测**、注册表子集语义与两处门同步、ABI/flag 只加不改、新内核基线三值我自跑复现、行数棘轮全部成立、证据与范围自检通过。

我**自己重算/重跑**得到的关键数字：

| 项 | 我的复算 |
| --- | --- |
| pinned `render.py` | 51 362 B / 1 604 行 / `16bfe7ed…` ✓；`sample_skybox` L494–556 全部语义点逐条命中（六面 sc/tc 表、ma 链、`safe_div`、`t` clamp、`v=(face+t)*(1/6)`、`texture_sample(vec4)`） |
| 上游**生成代码**（warp 1.15.0 缓存的 `_render_megakernel` .cu） | `1/6` 常量声明为 `const wp::float32 = 0.16666666666666666`（f32 位型 `0x3e2aaaab`）；`wp::clamp = min(max(a,x),b)`；`texture_sample<vec4(tex,uv,lod)`；调用点条件 = `static(render_skybox) && render_rgb[camid]` |
| 实现 `sample_skybox` | 六面 sc/tc 表与上游**逐项相同**（我从两侧源码各自解析后比对）；旋转 `rx=x, ry=z, rz=-y`；`arx>=ary&&arx>=arz` → `else if (ary>=arz)` → else 链一致；`fminf(fmaxf(t,t_min),1-t_min)` ≡ `clamp(t,t_min,1-t_min)`；`1.0f/6.0f` = `0x3e2aaaab` |
| 内核门（我自跑 NVRTC 纯编译） | `source_sha256=e68e3412…` / `ptx_sha256=0b947492…` / `ptx_bytes=268275`（与证据一致）；`test result: ok. 1 passed` |
| PTX 语义增量（我数现盘 dump） | `tex.level.2d` 1→2；`and.b32 …,512` 0→1；`max.f32` 39→40；`min.f32` 9→10；`sqrt` 51→51；另 `and.b32 …,256` 2→3（见 O1） |
| 旧基线未被覆盖 | `baseline.ptx` = 264 015 B / `97467d83…`（= 本片前基线），`current.ptx` = 268 275 B / `0b947492…` |
| 行数棘轮（我按 `\n` 独立数） | shade.cu 457/470、megakernel.cu 368/420、megakernel.rs 410/412、arrays 332/340、uploads 465/480、context.rs 455/470、lifecycle 498/499、texture_inputs 157/160、render_skybox 260/260；红线 texture.rs 486、cast.rs 481、render.cu 499、launch.rs 500、model/render.rs 500 全部零改动 |
| 门禁（我自跑） | fmt 0 / clippy（`--all-targets --locked --features cuda -D warnings`）0 / `cargo test --locked` **1103 passed / 0 failed / 389 ignored（135 result 行）** / `--no-run` 0 / 内核门 0 / `validate --fingerprints` 错误 0 警告 5 |
| 证据 | manifest 5 行 = **5/5 committed blob + 5/5 现盘**、不自引用、无未列文件；禁词 0；`git diff 8aa33e00^..HEAD` 只含卡 scope 文件 + `.normify` |

---

## 2. 上游逐行对齐（复核点 1）

pinned 文件（我直接读盘并自算）：`target/render-bvh-precheck/venv312/Lib/site-packages/mujoco_warp/_src/render.py`
= **51 362 B / 1 604 行 / sha256 `16bfe7edfa73033ef88fb4bbc46364867935f0c527ca53a23ff65c5c40fb3536`**。

### 2.1 `sample_skybox`（L494–556）

我从上游源码**解析**出 face→(sc,tc) 表（不是照抄证据），与实现 `render_shade.cu::sample_skybox` 的同类解析逐一比对：

| face | 上游 sc/tc | 实现 sc/tc | 一致 |
| --- | --- | --- | --- |
| 0 (+X) | `-rz` / `-ry` | `-rz` / `-ry` | ✓ |
| 1 (−X) | `rz` / `-ry` | `rz` / `-ry` | ✓ |
| 2 (+Y) | `rx` / `rz` | `rx` / `rz` | ✓ |
| 3 (−Y) | `rx` / `-rz` | `rx` / `-rz` | ✓ |
| 4 (+Z) | `rx` / `-ry` | `rx` / `-ry` | ✓ |
| 5 (−Z) | `-rx` / `-ry` | `-rx` / `-ry` | ✓ |

其余逐条：旋转 `rx=x, ry=z, rz=-y`（GL3）✓；`arx/ary/arz = abs` ✓；`face=0, sc=0, tc=0, ma=1.0` 初值 ✓；最大轴链 `arx>=ary and arx>=arz` → `elif ary>=arz` → else ✓；`ma` 三处置值 ✓；`s/t=(safe_div(x,ma)+1)*0.5` ✓；`t_min=0.5*face_width_inv` 且 `clamp(t,t_min,1-t_min)` ✓；`v=(face+t)*(1/6)` ✓；`texture_sample(vec2(s,v), vec4)` ✓；面顺序注释 `+X,-X,+Y,-Y,+Z,-Z` ✓。

### 2.2 常量与「生成语义」的额外独立验证（比只看 Python 更硬）

warp 1.15.0 的**生成 CUDA**（`target/render-bvh/s3a-shadow-cache/1.15.0/…render_megakernel…cu`，含该函数与调用点）给出：

```text
const wp::float32 var_100 = 0.16666666666666666;   // 1/6 以 float32 常量落地 → f32 位型 0x3e2aaaab
var_97 = wp::clamp(var_92, var_94, var_96);        // clamp = min(max(a,x),b)（warp/native/builtin.h）
var_103 = wp::texture_sample<wp::vec_t<4, wp::float32>>(var_skybox_tex, var_102, var_104);
// 调用点：if (static(render_skybox) && render_rgb[camid]) { sample_skybox(...); store_pixel(...); } elif render_rgb[camid] { ... }
```

⇒ 实现写 `(float)face + t) * (1.0f/6.0f)`（f32 除法同样正确舍入到 `0x3e2aaaab`）、`fminf(fmaxf(...))`、`texture_sample<wp::vec4>(…, 0.0f)` 与生成语义**逐点一致**；`safe_div`（`math.py` L318–319，`MJ_MINVAL=1e-15`）与文件内既有 `safe_div_render` 同义。

### 2.3 调用点与 `store_pixel`

上游 L1259–1268：`if wp.static(rc_static["render_skybox"]) and render_rgb[camid]:` → 取 `skybox_tex_id[worldid % shape[0]]`、`1.0/float(skybox_face_width[...])`、`sample_skybox(...)`、`store_pixel(...)`；`elif render_rgb[camid]:` 才是原背景分支。
实现：`if (do_rgb) { if (bit9) {…sample_skybox…; aa_store/pack} else if (aa) {原背景} else {原背景} }` —— 等价（`bit9 == resolved.render_skybox`，`do_rgb == render_rgb[camid]`）；`store_pixel` 语义（AA `+= color*255`、非 AA `pack_rgba_to_uint32(r*255,g*255,b*255,255)`）由**同一个** `aa_store`/`pack_rgba_to_uint32` 承担 ✓。

---

## 3. 非天空盒路径「零改」（复核点 2）

**静态论证成立且完整**（我逐条核代码 + diff）：

1. 控制流：新增分支只存在于 `if (hit.geom_id == -1)` 早退块内、且第一层条件 `(flag_bits & MK_FLAG_SKYBOX) != 0u`；`bit9` 来自 `resolved.render_skybox`，而 `context.rs:321` 里 `render_skybox: skybox.is_some()` ⇒ 无天空盒 = `false` ⇒ 落入原 `else if (aa) / else` 两支。
2. 既有语句未被改写：`git diff 8aa33e00^..8aa33e00 -- src/render/kernel/cuda/` 只新增 `#define`、新增函数与 `if (aa)` → `} else if (aa)` 的接线；原两支语句逐字节保留（我比对 `aa_store(aa_accum_out, rgb_offset, background, ray_base == 0)` 与 `rgb_out.data[rgb_offset] = packed_background;` 均在原位）。
3. 输入面不变：无天空盒时 `texture_array = None`（uploads 门 `use_textures || render_skybox` 为假）、两槽 `([0],[0])`（`None => ([0i32],[0i32])`）、注册表空（`texture_registry` 在 `use_textures=false && skybox=None` 返回 `TextureRegistry::default()`）、`lifecycle` 的 `TextureResources` 为 `None`。
4. flags 打包方式不变：`(background_color_packed << 32) | flag_bits` 原样；bit0–8 逐位未动（详见 §4）。
5. **措辞纪律**：证据明确写「本片无 GPU 执行、零泄漏的位级回归证据留 S3」「no zero-leak *measured* claim (static argument only)」（README §6.4、receipt `zero_leak_static_argument`/`non_claims`），**没有**把静态论证说成设备实测 ⇒ 未触碰阻断条款。

---

## 4. 注册表子集 / 门 / ABI（复核点 3、4）

- `texture_inputs.rs::texture_registry` 对照上游 `render_util.py` L514–535：`use_textures=true` → 全量（与既有 `TextureRegistry::from_fields` 同）；`false && skybox=Some` → 仅天空盒 id 真实实体化，其余 id 为 **1×1×4 全零** 占位，**下标仍等于模型纹理 id**（不压缩）；两者都无 → 空注册表。上游的 `has_skybox = render_skybox and size>0`、无天空盒时 `render_skybox=False` 也已由既有 `model.rs::skybox()` 对应（本片未改该文件）。
- 两处门同步：`lifecycle.rs:330` `wants_textures = resolved.use_textures || resolved.skybox.is_some()`；`megakernel_uploads.rs:183` `Some(resources) if resolved.use_textures || resolved.render_skybox`。因 `render_skybox == skybox.is_some()`，两式等价 ✓；uploads 另有 `render_skybox && texture_array.is_none()` 的发射前拒绝（防御）✓。
- ABI/flag：`src/runtime/launcher/render/args.rs`（`MEGAKERNEL_SLOTS` 定义处）**本片零改动**（我 diff 该文件为空），`declared_len = 81`；测试仍断言 81 = 1 bounds + 7 scalars；`MK_FLAG_*` bit0–8 逐条不变、bit9 `0x200u` 追加（Rust 侧 `FLAG_SKYBOX_BIT = 1 << 9`，与内核同值）；`DEVICE_SOURCES` 仍 28、`CollisionArgs` 仍 504 B（均未改，我 grep 确认）。

---

## 5. 新内核基线（复核点 5）

我自跑 `cargo test --locked --features cuda-probe --test render_kernel_rgb -- --ignored megakernel_source_compiles_under_render_gate --test-threads=1 --nocapture`（**NVRTC 纯编译、不占 GPU**）：

```text
source_sha256=e68e3412c1e1809e71100d9e842d5b0f5b886ffdec88d8401bb01c76e812904c
ptx_sha256=0b9474928ab8a7add27e5da7ac0869ee955a75405cd976cd04839f2312e88f3e
ptx_bytes=268275
test result: ok. 1 passed; 0 failed; 0 ignored
```

与证据三值逐位一致；旧值 `72bc364c…` / `97467d83…` / 264 015 在 `ptx-diff-summary.txt` 中明确标为「baseline（本片前）」而非被覆盖 ✓；现盘 `target/r02-ptx/baseline.ptx` 的 sha 也得 `97467d83…`，说明基线重建忠实。
语义增量我按现盘 PTX 逐项数（见 §1 表）：`tex.level.2d` +1、`and.b32 …,512` +1、`max.f32` +1、`min.f32` +1、`sqrt` 不变 ✓；另有 `and.b32 …,256` +1（O1）。

---

## 6. 证据、范围与纪律（复核点 6–8）

- 行数棘轮：我按 `\n` 独立数，全部与实现方一致且在界内（§1 表）；`texture.rs` 486 行零改动。
- 范围：`git diff 8aa33e00^..HEAD` = 11 个实现/测试/模块文档文件 + 证据 6 文件 + `.normify` 卡/runs；`src/render/bvh/**`、`src/math/**`、`fixtures/**`、npz、capture 脚本、判据阈值**零改动**。
- 纪律：本片无窗口产物（`target/render-bvh/` 下 06:00 之后无新文件）；`materials/gates.txt` 只记录四门禁 + `--no-run` + NVRTC 门；证据多处声明「无设备数值执行」，与 `receipt.limits` 一致。
- 证据自检：manifest 5 行 5/5 committed + 5/5 现盘、无自引用、无未列文件；禁词 0；README/receipt 均无「窗口/精度/等价/验收通过」类结论。
- 模块源登记：`mjwarp.render` 源码 39 → **40**（新增 `src/render/context/texture_inputs.rs`），module doc `[[source]]` 计数 40、`[[apis]]` 仍 11；`normify impact mjwarp.render` 输出 `源码40，API11` ✓。

---

## 7. 观察（8 条，均不阻断）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | 记录 | `materials/ptx-diff-summary.txt` 的语义表只列 `and.b32 …,512` +1；我实测**总** `and.b32` 43→45，另 +1 是 `and.b32 …,256`（0x100 = AA 位）由 2→3，来源是新分支内 `if (aa)` 的位复检。声明本身正确，但读者可能把「+1」误当总数 | 在语义表补一行「总 and.b32 +2（其中 512 掩码 +1、256 掩码 +1）」 |
| O2 | 记录 | 新增 `pub fn texture_registry` 并经 `pub use` 导出（测试用），属新的公开 Rust 项；module `[[apis]]` 仍 11（该清单是 planned rpc 合同口径，不含 Rust pub 项） | 「API 11 不变」成立；建议 README 注明该口径，避免后续误读 |
| O3 | 记录 | dummy 纹理实现差异：上游对所有非天空盒 id 共享**同一个** `empty_tex` 实例；本实现为每个 id 各建一个 1×1×4 全零纹理（内容与采样语义等价，仅对象数不同） | 记录即可；如需与上游一致可共享实例 |
| O4 | 记录 | `tests/render_skybox.rs` = **260 行，正好等于**卡面上界（≤260），零余量 | 后续任何追加即破棘轮；如需扩展先拆文件/调整上界并报批 |
| O5 | 记录 | 设备侧 `sample_skybox` 与新增分支**没有任何执行级验证**（本片无 GPU；README §3/§10 与 receipt limits 已如实声明） | 建议 S2/S3 首窗把「六轴方向 + face 边界 + 零方向」纳入 capture 检查点，否则 face 映射只能靠整体位级比对兜底 |
| O6 | 记录 | 证据 `gates.txt` 记 `validate --fingerprints`「6 条警告，含 1 条本片引起的 `mjwarp.render` 指纹过期，closing 时由 done 刷新」；我关闭后复跑为 **错误 0 / 警告 5**（指纹已刷新） | 与证据自述自洽；无需改 |
| O7 | 记录 | `src/model/render.rs` 工作树为 CRLF（CR=500），本片零改动；若将来把它纳入 LF 行数口径需先归一 | 仅记录 |
| O8 | 记录 | `texture_registry` 比上游多两项 host 校验（`face_width == tex_width[skybox_id]`、`skybox_id` 在范围内）→ 对合法模型不可触发，属 fail-fast 防御 | 记录；若将来上游允许「声明 skybox 但无纹理 id」需重新评估（当前 `model.rs` 已保证一致） |

---

## 8. 边界

1. 未开 GPU 窗口、未跑任何设备数值/`--ignored` 设备用例；设备侧 `sample_skybox` 与新增分支的像素级行为**未验证**，本卡不出任何渲染数值结论。
2. 我自跑的门禁只有 fmt / clippy（默认特性含 cuda）/ `cargo test --locked` / `--no-run` / NVRTC 内核门 / `validate --fingerprints`；未重跑实现方的 PTX 导出 scratch（直接读现盘 `target/r02-ptx/*.ptx` 并校验 sha 与门禁输出一致）。
3. 「生成语义」以 warp 1.15.0 缓存的生成 CUDA 为准（来自 R05 capture 的 megakernel 实例化，含 skybox 分支）；Python 源我逐行核过，二者一致。
4. 未改实现代码/证据/卡；未跑 GPU 窗口脚本。
5. 第一次并发调用门禁时曾遇 `os error 32`（两个 cargo 抢同一 exe），属我自己并发操作的环境噪声，串行重跑后全部通过（见 `logs/gates.txt`），不作为结论依据。

---

## 9. 复现命令（仓库根目录）

```text
target\render-bvh-precheck\venv312\Scripts\python.exe -B docs\development-evidence\render-bvh\r02-skybox-s1-check\recompute.py
cargo fmt --all --check
cargo clippy --all-targets --locked --features cuda -- -D warnings
cargo test --locked
cargo test --locked --no-run
$env:PATH = 'D:\Rust\mjwarp-rs\target\toolchains\cuda-12.8.1\bin;' + $env:PATH
$env:CUDA_PATH = 'D:\Rust\mjwarp-rs\target\toolchains\cuda-12.8.1'
cargo test --locked --features cuda-probe --test render_kernel_rgb -- --ignored megakernel_source_compiles_under_render_gate --test-threads=1 --nocapture
normify validate --fingerprints
```

（脚本的 `problems` 为空，见 `recompute.json`；门禁与内核门原始输出在 `logs/`。）

---

## 10. 收口

- 本卡证据提交：本目录（README/receipt/manifest/recompute.py+json/logs/materials）。
- 被核对象三段提交 `8aa33e00` + `b6b2a679` + `299459c0` 保持不动（本卡只读）。
- 本片为离线实现片：设备数值验证属 S2/S3 窗口，本卡结论仅覆盖「离线实现 + 静态零改论证 + 主机侧语义 + 新内核基线」。
