# QA 复核：z2vec 片 2 / 记录面修正（`9155b9c1`，卡 A）

- 任务：`2026-11-11-render-flex-z2vec-record-errata-check`（owner QA）。复核对象：提交 **`9155b9c1`** + 卡 `2026-11-11-render-flex-z2vec-record-errata`。零 GPU、只读复核；结论均自算/自测。
- 基线：`04b981d6`（卡 A 登记/start）；复核期间 Backend 持锁继续做卡 B/C，本复核引用的都是 `9155b9c1` 的内容。

## 结论

**复核通过（0 阻断，1 低 + 1 观察）。** 8 项更正（C1–C8）+ backlog 全部落地且可验证；零 `.cu`、零行为变化；两处历史 README 为**纯追加**（旧字节逐字节保全）；attempt4 目录零改动。

## 一、逐项核（派工要点 ①–⑥）

| 派工要点 | 落点 | 我的核验 | 结论 |
| --- | --- | --- | --- |
| ① Arch 观察（镜像 `warp_length3` 非位级 oracle） | `tests/render_flex_z2vec.rs` 文件头 + `warp_length3` 文档注释；`flex-z2vec-pin/README.md` §7 | 注释明写「数值对应、非位级 oracle（Rust 左结合 vs `.cu` 显式 `fmaf` 链，1 ulp 级，断言只用 1e-6 容差、不承载位级等价）」；并说明 #1 的 `assert_eq!` 只针对近平行常量四元数、不经该求和链——与测试实际一致 | ✅ |
| ② L1 措辞更正 | 同上 + README §7 | 更正为「`(0,1,0,0)` 在 `render_flex.cu`/全 `src/` **无字面**；实际未动的是同向初值 `(0,0,0,1)`」；我实测 `render_flex.cu` 初始化确在 **:36-39**（`quat[0..3]=0,0,0,1`）、分支在 :51-56、同向路径 `return` 早退原样——引用行号正确 | ✅ |
| ③ errata r2 G1（C1 provenance） | `gl2-errata-batch/README.md` §G + 新目录 `artifacts/c1-diag7-post-provenance.txt` + target 现场文件 | §G 明写「`gpu_post_actual` 为 **errata 时段复测**（mtime 2026-11-11 08:06:18）、非 run 时采样」；错误引用「post recorded in run-summary.txt」已从 target 文件删除并改为「post NOT recorded at run end → errata-time re-query」；`run-summary.txt` 仍为 `gpu_after=$gpu` 占位的事实写清；BEFORE/AFTER 逐字留证；我实测 target 文件现 **986 B / mtime 12:03:30**，与 artifact 声明一致 | ✅ |
| ④ r2 低 4 项 | `gl2-errata-batch/README.md` §G | `b-field-change.diff`：我核 artifact 确只有 2 个 hunk（测试搬迁，`diag.rs` −49 / `diag_ext.rs` +51）、old index `25affdc8`；`84ed67d2^` 的 `diag.rs` blob 我复核为 **`02533c7d`** —— 说明成立。receipt 改为「结构重排 + addenda，除 `addenda[]` 外逐键相同」。物理行数 **484 / 237**（我重测：484/18,384 B、237/9,437 B，与 artifact 一致；466/231 的 18/6 差额解释成立）。`task_id` 补 successor 关系、历史 task_id 不改写 | ✅ |
| ⑤ PTX NUL 口径 + 身份重冻 backlog | README §7 + `artifacts/ptx-length-dual-accounting.txt` + receipt `kernel_identity`/`backlog` | 双口径写清（348,459 去 NUL / 348,460 raw `nvrtcGetPTXSize`）；我沿链路核过：repo `src/runtime/launcher/artifact/nvrtc.rs:104-105` 为 `compile_ptx_with_opts(...).map(\|ptx\| ptx.to_src())`；cudarc 0.18.2 `src/nvrtc/result.rs:114-122` 用 `nvrtcGetPTXSize` 定 buffer（含终止 NUL）、`src/nvrtc/safe.rs:41-46` 的 `Ptx::to_src()` 用 `CStr::from_ptr` 在 NUL 处截止 —— **说明链正确**；身份 logs `ptx_bytes=348459/348399` 实读一致；我此前独立复算的 raw 348,460/348,399+1 也吻合。backlog「megakernel `cf356b72…`/`b63a51dd…` ⇒ 后续渲染 GPU 窗口必须重新冻结」在 README §7 + 卡 note + receipt 三处 | ✅ |
| ⑥ 零 `.cu`/行为未变/门禁/manifest | 全提交 | `git diff 04b981d6..9155b9c1 -- '*.cu'` **空**、`src/`/`fixtures/`/`scripts/` 零改动；`tests/render_flex_z2vec.rs` **+15 全为注释行、0 删除**（9 个断言与镜像逻辑零改动）；门禁 fmt/clippy/clippy-cuda 通过、unit **1,403/0/402**；三目录 manifest 重算 **11/11、8/8、5/5**（bad=0/missing=0/extra=0、不自列）；改动/新增文件全 UTF-8/LF、无 BOM/CR/NUL；attempt4 目录 **零改动**（历史字节未被触碰，可读副本口径未变） | ✅ |

## 二、证据自核（我另算）

- 新目录 `perworld-l-v2-flex-z2vec-record-errata/`：README 4,314 B、receipt 5,714 B、manifest 708 B + 3 artifacts（2,942 / 1,008 / 2,061 B），manifest **5/5**。
- `perworld-l-v2-flex-z2vec-pin/README.md` 5,256 → 7,436 B，**纯追加**（前缀 == 旧长度 5,256），新增 §7；manifest 11/11。
- `perworld-l-v2-gl2-errata-batch/README.md` 5,719 → 8,633 B，**纯追加**（前缀 == 旧长度），新增 §G；manifest 8/8。
- 卡 A 状态 verified；`.normify/runs/...record-errata/` 四门禁 exit 0。

## 三、低项 / 观察

- **低 L1（可读性）**：`gl2-errata-batch/README.md` §G 的编号沿用了「L1 = b-field-change.diff」等 QA 编号，但其 §A/§B 原文里同样有 A/B 编号语境；建议后续在 §G 表头注明「本节编号沿用 QA 复核 `fd72cf0b` 的 G1/L1–L4，与本文件 §A/§B 的条目编号无关」，避免读者串号。（纯记录面，不影响任何结论。）
- **观察 O1**：append-only 策略下，`gl2-errata-batch/README.md` §C 的原文（C1 仍按「补跑后快照」语境书写）保持不改，更正只在 §G 与 target 现场文件；只读 §C 的读者可能误读。§G 已声明「原文字节不改」，属既定策略；建议后续若做索引/读取工具时把 §G 视为覆盖层。（不阻断。）
- 其余：cudarc 链路行号、blob index、物理行数、target mtime/size、身份 logs 均逐项复核一致。

## 四、口径（非声明）

不宣称设备端渲染变化/修复生效/上游等价；CPU 镜像断言不等价于设备端验收；本卡不重冻、不占 GPU 窗口；不改判据/参考/容差；零 GPU。
