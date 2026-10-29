# R08-RGB 逐相机通道掩码离线片 · QA 独立复核

任务卡：`2026-10-29-render-r08-rgb-hetero-check`（priority 51，Owner QA）
被核对象：`2026-10-29-render-r08-rgb-hetero`（**verified**）
被核提交：基线 `73738357` → 工作 `883ee521`（host 三通道 + latent 负偏移修复）→ 收口 `01d1cb0b`
复核时 HEAD：`01d1cb0b`。**只读**：未开 GPU、未跑 `--ignored` 设备用例、未改代码/证据；允许的门禁构建我自跑（`logs/`）。

---

## 1. 结论

**verdict：pass / 阻断 0 / 观察 6（建议 2、记录 4）。**

`megakernel.rs` 的三条通道数组确实由「全局布尔填满 nrender」改成 `channel_flags(&resolved.plan.render_rgb/depth/seg)`（逐激活相机、与 `cam_res`/`cam_id_map` 同源同序），`ImageOutput` import 与 `is_enabled(ImageOutput::…)` 驱动路径已移除；**81 槽 ABI 与内核源零改动**（我现算 6 个 `.cu` 拼接 sha = `e68e3412…`，81 槽 51/52/53 = `A(Bool)`，内核签名同位置正好是 `render_rgb/render_depth/render_seg`）；latent 负偏移写的成因与闭合我按内核代码复核成立。行数 5 项全在上界内；我自跑 fmt/clippy/clippy-cuda/unit 全 exit 0，unit **1106 / 0 / 391（136 result 行）**，与证据一致。本片不含设备侧结论（GPU 用例只编译不执行，归 S3 窗口）。

---

## 2. host 填充与长度/同序不变量（复核点 1）

`src/render/kernel/megakernel.rs` 的 diff 只有三件事：删 `use crate::render::ImageOutput;`、删三行全局布尔、换成

```rust
let render_rgb = channel_flags(&resolved.plan.render_rgb);
let render_depth = channel_flags(&resolved.plan.render_depth);
let render_seg = channel_flags(&resolved.plan.render_seg);
```

- **纯函数**：`megakernel_arrays.rs` 的 `channel_flags(&[bool]) -> Vec<i32>` 就是逐项 `i32::from`，长度恒等于入参；`bool_words` 再按 4 bool/u32 打包（我核 `megakernel_arrays.rs:108-116`：`1u32 << (8*(i%4))`，与 `bool`=1 byte 的 CUDA 数组布局一致）。
- **长度不变量**：`resolve_targets`（`camera.rs:413-442`）按激活相机逐项解析，长度 ≠ ncam 时返回 `CameraError::RenderFlagLength`；`ResolvedContext::nrender()` = `plan.cameras.len()`；三处实参形状仍是 `&[nrender]`（`megakernel.rs:315-317`）。⇒ 三条数组长度恒等于 nrender，无长度失配风险。
- **同序不变量**：`cam_res`（`plan.resolutions`）、`cam_id_map`（`plan.cameras`）、`plan.render_*` 都由同一个 `CameraPlan` 按激活顺序给出；内核 `render_megakernel.cu:110-133` 用 `cam_res` 累加求 `camid`，随后读 `render_*.data[camid]`，写入用 `*_adr.data[camid]` ⇒ host 侧第 i 项与内核 camid=i 一一对应。`CameraActive::Indices` 反序时同样成立（plan 三个向量一起反序）。
- **缺口归因（我加查，不是抄说明）**：兄弟设备路径本就没有这个问题——`cast.rs:390-393` 按 `position` push `plan.render_depth/seg`，S7b1 `renderer.rs:142-143` 上传的就是这两个逐相机数组；`megakernel_uploads.rs:324-326` 只做 `bool_words` 打包。我扫 `src/render/**` 无 `vec![i32::from(` 残留 ⇒「全局折叠只在 megakernel host 三行」这一归因成立。
- **负偏移写成因/闭合**：旧代码用 `outputs().is_enabled(通道)`（=该通道「有任一相机开启」）填满 nrender；只要异构，关闭通道的相机也 `do_*=true`，而它的 `*_adr=-1`，射线仍会发射（`nrender`/`total_rays` 含该相机像素）⇒ 写入偏移 `worldid*pitch + (-1) + rayid_local`：`worldid=0` 时出现下标 `-1`（缓冲前越界），`worldid≥1` 时落到上一 world 的最后一像素。逐相机填充后该类相机 `do_*=false`，内核 `render_megakernel.cu:128-133` 守卫直接跳过 ⇒ 闭合。

---

## 3. ABI 与内核身份（复核点 2）

| 项 | 我现算 | 证据声明 | 结论 |
| --- | --- | --- | --- |
| `MEGAKERNEL_SLOTS` 长度 | 81 | 81 | ✓ |
| 槽 51/52/53（0-based） | `A(Bool)` ×3 | `Array(Bool)` ×3 | ✓ |
| `.cu` 签名参数数 | 81 | 81 | ✓ |
| 签名 params[51..54] | `render_rgb`, `render_depth`, `render_seg` | 同 | ✓ |
| 6 个 `.cu` 拼接 `source_sha256` | `e68e3412c1e1809e71100d9e842d5b0f5b886ffdec88d8401bb01c76e812904c`（88 190 B） | 同 | ✓ |
| 每个 `.cu` blob：基线/收口/现盘 | 6/6 三者相同（无 CRLF） | 零设备源改动 | ✓ |
| `aa_resolve.cu` source sha | `0bf8af5a68f9cae50d0cb9cc4da2a01da2607066cf1fe3072c8345e6e104bc76`（2 364 B） | 同 | ✓ |
| `src/render/kernel/cuda/**` diff | 空 | 空 | ✓ |

补充：`MEGAKERNEL_SOURCE` 的第 1 个 include 是 `src/render/bvh/cuda/scene.cu`，**不在 `kernel/cuda/**` 下**——我单独核了它的 blob 也是基线==收口==现盘，所以「`.cu` 目录 diff 为空」不足以覆盖的那一个文件没有漏。
PTX 三值（`0b947492…` / 268 275 B、`aa_resolve` 27 536 B）是 NVRTC 编译记录，我**没有重跑**（本卡只读、不跑 `--ignored`，见 §9 limits）。

---

## 4. 测试覆盖（复核点 3，只读代码不执行）

- **单测**（我自跑全过）：`channel_flags_maps_each_camera_independent`（`[true,false,true]→[1,0,1]`、`[false,true]`、`[true,true]`）、`channel_flags_handles_all_false_and_empty`（全 false、空）。
- **GPU 新用例** `r08_gpu::heterogeneous_rgb_channels`（`tests/render_r08_cameras.rs` 的 `#[ignore]` 入口）：三态 `cam0_on [true,false]` / `cam1_on [false,true]` / `all_off [false,false]`；每态断言 `rgb_adr` 期望 Layout、启用相机 `rgb_packed` 对 S7b2 same-recipe 参考逐位、未启用相机读取拒绝、depth/seg 对 S7b1 参考位等/全等、**两次运行缓冲 SHA-256 相等**。
- **既有用例追加** `heterogeneous_depth_seg_channels`：同一交叉掩码（cam0 仅 depth、cam1 仅 seg）再经 `MegakernelRenderer` 发射并逐相机对参考、未启用通道拒绝——这是 latent 负偏移写的回归位。
- **回归强度我的判断**：旧全局填充下 cam1 的写偏移是 `-1`，会落到 cam0 的通道区间（且 worldid=0 时越界），因此「启用相机的逐位比较」能捕获该缺陷；但它是**间接检测**（不读缓冲前置内存、不校验越界的显式报错），属 O3 记录。证据文件里 `materials/` 没有逐条断言审计——本卡按只读代码审查完成，无需补。
- **覆盖缺口**：现有用例没有「某相机三通道全关」的异构态（例如 rgb/depth/seg 均 `[true,false]`），此时内核 `render_megakernel.cu:131` 的 early-return 分支不会被覆盖（O1，建议）。

---

## 5. 行数棘轮（复核点 4）

before 取基线 blob、after 取现盘，均按 LF 计（我自算，非抄表）：

| 文件 | before | after | 上界 | |
| --- | --- | --- | --- | --- |
| `src/render/kernel/megakernel.rs` | 410 | **409** | 420 | ✓ |
| `src/render/kernel/megakernel_arrays.rs` | 332 | **357** | 365 | ✓ |
| `tests/render_r08_cameras.rs` | 320 | **326** | 380 | ✓ |
| `tests/support/render_bvh/r08_gpu.rs` | 321 | **403** | 420 | ✓ |
| `tests/support/render_bvh/r08.rs` | 366 | 366（未改） | 380 | ✓ |

---

## 6. 零外溢（复核点 5/6）

- `git diff 73738357..01d1cb0b --name-status` 我逐条对账：**21 条** = 代码 4（`megakernel.rs`、`megakernel_arrays.rs`、`r08_cameras.rs`、`r08_gpu.rs`）+ 证据 8（README/receipt/6 logs）+ `.normify` 9（3 张新卡 + gap 卡 note + 模块指纹 + 4 份 gate runs）。无多余、无遗漏。
- 零改动清单 6 项 diff 全空：`src/render/kernel/cuda/**`、`megakernel_uploads.rs`、`camera_output.rs`、`context.rs`、`context/params.rs`、`camera.rs`。
- 判据/容差/参考/夹具：`fixtures/`、`scripts/validate/` 零文件变更 ⇒ 未改。
- committed == 现盘：`git diff --quiet 01d1cb0b -- <主卡证据目录>` 与 `git diff --quiet HEAD -- <同>` 均 exit 0。
- `normify status`：**已验收 568 / 运行 1（本复核卡）/ 错误 0 / 警告 0**；`normify validate --fingerprints` 我重跑 **0 错误 / 5 警告**（均为既有条目：2 个 leaf-empty、leaf-too-coarse 34、2 个 layout/order-incomplete），说明 `modules/mjwarp/render.md` 的 fingerprint 更新与新源码一致、无漂移。

---

## 7. 证据与规划（复核点 5）

- 主卡证据 8 文件全 **LF、无 BOM**；`receipt.json` `/task_id` = `2026-10-29-render-r08-rgb-hetero` ✓；禁词 0（我用码点构造的 3 个词扫描，0 命中）。
- 证据里 `logs/line-counts.txt`、`logs/source-sha.txt`、`logs/kernel-gate.txt` 的数字我全部复算/对账一致；`logs/scope-diff.txt` 的改动清单与 §6 一致。
- 规划：缺口卡 `2026-10-28-render-per-camera-rgb-gap` note 已指向本实现卡；主卡 `status=verified` 且有 `closed_at`；S3 窗口卡 `2026-10-29-render-r08-rgb-hetero-validate` 存在且 `status=proposed`（**未 start**，板上列在就绪）✓。

---

## 8. 门禁（我自跑，`logs/`）

| 门禁 | 命令 | 结果 |
| --- | --- | --- |
| fmt | `cargo fmt --check` | exit 0（`logs/fmt.log` 空） |
| clippy | `cargo clippy --all-targets --locked -- -D warnings` | exit 0（`logs/clippy.log`） |
| clippy-cuda | `cargo clippy --all-targets --locked --features cuda -- -D warnings` | exit 0（`logs/clippy-cuda.log`） |
| unit | `cargo test --locked` | exit 0；**136 result 行 / 1106 passed / 0 failed / 391 ignored**（`logs/unit.log`，我逐行求和） |
| 新单测 | 同上 | 两条均 `... ok`；GPU 两个入口均编译并记 `ignored`（本片不执行） |
| `--no-run` | — | 未单跑（`cargo test --locked` 已编译全部 target）；证据自带 `logs/no-run.txt` 为截断尾部（O4） |
| 内核 NVRTC 编译门 | — | 未跑（只读边界）；改用 §3 的 sha 现算 + blob 对账 |
| GPU 设备用例 | — | 未执行 ✓（与本片声明一致） |

独立重算脚本：`recompute.py` → `recompute.json`（**18 项 check、problems=0**，`logs/recompute.log`）。

---

## 9. limits

- **PTX 身份未独立复跑**：`ptx_sha256`/`ptx_bytes`（含 `aa_resolve`）来自实现方 NVRTC 记录。我的替代证据是：6 个 `.cu` 的 blob 基线==收口==现盘、`MEGAKERNEL_SOURCE` 拼接 sha 现算一致、编译配方相关文件零改动（`src/runtime/launcher/`、`src/render/kernel/mod.rs` 均不在 diff 内）。若要把「PTX 逐位不变」升为 QA 亲测项，需要卡面明确放行 NVRTC 纯编译门（O6）。
- **设备侧行为未验证**：即逐相机掩码在 GPU 上的位级结果归 S3 窗口 `2026-10-29-render-r08-rgb-hetero-validate`；本片只审代码与编译。
- 本复核只覆盖 `73738357..01d1cb0b`；Arch 之后若继续提交 S2/S3 卡不在范围内。

---

## 10. 观察（建议 2、记录 4）

| # | 级别 | 观察 | 建议 |
| --- | --- | --- | --- |
| O1 | 建议 | GPU 用例没有「某相机三通道全关」异构态；内核 `render_megakernel.cu:131` 的 all-off early-return 分支无覆盖（本片未改内核，不影响本片判定） | S3 窗口可顺手加一态（rgb/depth/seg 均 `[true,false]`），成本一行；不阻塞本片 |
| O2 | 建议 | 证据 `logs/no-run.txt` 只是 `--no-run` 输出尾部 4 行 `Executable` 片段，无 exit 码、无 `Finished` 行，单凭它不能证明「全目标编译通过」 | 下次留 exit 码 + 尾部 `Finished` 行；本片已由我 `cargo test --locked` exit 0 覆盖 |
| O3 | 记录 | 负偏移回归是**间接检测**：靠「启用通道逐位比较」捕获污染，没有读缓冲前置内存/越界显式断言 | 可接受（比只看不 panic 强）；若想直接证明，可在窗口内加显存 guard 页或对 worldid=0 读回前后快照 |
| O4 | 记录 | 主卡 acceptance 写「单测…长度」，实现用整 Vec 相等 + 空断言隐式覆盖长度，无单独 `.len()` 断言 | 接受（`channel_flags` 结构上不可能变长），后续措辞可写「等值断言隐含长度」 |
| O5 | 记录 | `ContextParams::use_fast_math` 默认 `true`（`params.rs:115`），但 render 路径无消费者、kernel 编译配方固定 `fast_math=false`；README 的「same-recipe」成立，但读者可能误以为 context 该参数参与了口径 | 接受；建议 README 一句话点明（`resolved.use_fast_math` 仅记录，不参与本内核配方） |
| O6 | 记录 | 本轮 QA 对「kernel 编译门」采用「source sha 现算 + blob 对账」而非重跑；口径来自派工「只读、不跑 GPU/--ignored」 | 接受（结论不变）；建议把「QA 是否可跑 NVRTC 纯编译门」写成卡面固定口径，减少每次解释 |

---

## 11. 本卡文件

| 文件 | 内容 |
| --- | --- |
| `recompute.py` / `recompute.json` | 18 项独立重算（host 填充/归因、ABI 两处、内核 sha、范围、行数、证据、规划、测试静态、门禁日志） |
| `logs/recompute.log` | 上述脚本 stdout |
| `logs/fmt.log` / `logs/clippy.log` / `logs/clippy-cuda.log` / `logs/unit.log` | QA 自跑门禁原始输出 |
| `logs/gates.txt` | 门禁命令与 exit 码汇总 |
| `manifest.txt` | 本卡文件 sha256/字节/路径（不含自身） |
