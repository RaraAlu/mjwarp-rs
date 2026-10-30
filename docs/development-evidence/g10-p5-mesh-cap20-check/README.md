# mesh 容量 A1（16→20 / 32→40）— 独立复核

任务：`2026-10-31-g10-p5-mesh-cap20-check`（QA；只读）
被核：`2026-10-31-g10-p5-mesh-cap20`（Arch；P8；离线、零 GPU、零冻结）
pin：`307219fe`；start 时 HEAD（`revision.before`）：`a01b088141c57089f974243476d695303fcdf6b9`
复核时间：2026-10-31 04:0x–04:2x（+08:00）；**未跑 GPU/窗口/冻结，未改任何产品代码**

## 1. 结论

**pass / 阻断 0 / 需修 2 / 观察 7**

- **改动范围干净**：`307219fe` 的 `src` 只动 7 个文件（5 个 `.cu` + `clip.rs` + `mod.rs` 仅追加 `#[cfg(test)]` 测试模块）；
  无 `src/runtime`、`src/physics/mod.rs`、`tests/`、`fixtures/`、`Cargo*` 改动；无历史证据目录改写。
- **设备/宿主容量**逐项复核：`contact.cu` `polygon[40]`/`clipped[40]`/`pnormal[20]`/`pdist[20]`/`face1[20]`/`face2[20]`；
  `multicontact_geom.cu` `G5_MC_MESH_FACE_MAX 20`、`G5_MC_MESH_DEG_MAX` 仍 **10**；`clip.rs` `pn/pd[20]`、`polygon/clipped vec![..;40]`。
- **注释契约 5 处**：`contact.cu` / `multicontact_geom.cu` / `multicontact.cu` / `multicontact_clip.cu` / `multicontact_mesh.cu`
  均统一为 20/40/face 20，并写明 **U7（fan-cone-24）未覆盖、属 A2**。
- **2 条 CPU 单测**真实存在且我独立跑过：`polygon_clip_accepts_twenty_vertex_face1`、`..._face2`，
  断言 `ncontact 1..=4`；`cargo test --locked --lib physics::collision::gjk::multicontact` ⇒ **2 passed / 0 failed**。
  unit 门禁 `1113/0/397`（1111 + 2 条新测），`--no-run` EXIT=0，fmt/clippy/clippy-cuda passed。
- **ABI/判据零改动**：无 `CollisionArgs`/launch 签名/JSON 字段变化（render 的 81 槽 ABI 天然未触碰）；判据/容差/参考/fixtures 零改。
- **U8/U7 边界**：U8（prism-20）只接线到 clip 路径，设备接触数 1→4 是**上游参考**（sha `b6abb8fa…`）的期望，留待用户签窗后断言；
  U7 仍单接触回退；committed 文档未写「U8 通过 / U7 覆盖 / mesh 容量缺口已清」（这些短语只出现在 `not_claimed` 与「不得写」规则里）。

需修 2 条（证据可复核性，不重跑、不改代码）：
- **P1 内核 source 身份不可复现（EOL 污染）**：证据记 source `255,840/28385801…` → `254,743/f32f0af2…`；但我从 committed LF blob 独立重算
  `assemble_device_source()` = `254,005/9b7435ab…`（前）→ `254,263/12d50d68…`（后），现盘探针（worktree 仅 `primitive_plane_shapes.cu` 带 CRLF 169 对）= `254,431/af1791be…`。
  三者互相矛盾、差值正是 CRLF/LF 字节量 ⇒ **记录值随测量时工作树 EOL 漂移**。PTX 部分可复现：`ptx_bytes=3,883,769 / entries=6` ✓；
  「身份已变 ⇒ 后续窗口必须全新冻结」的结论在任意一致 EOL 下都成立（source sha 与 PTX bytes 均变）。修法：等 `evidence-eol-hygiene` 钉完 `*.cu text eol=lf` 后从干净 LF checkout 重跑探针、改记 LF 值（或注明测量时 EOL 状态并保留 LF 值为准）。
- **P2 证据 manifest 与 committed blob 不符 4/10**：`logs/no-run.txt`、`logs/nvrtc-identity.txt`、`materials/src-diff.patch`、`materials/validate.txt`
  列的是 CRLF 工作树字节（如 `no-run.txt` 175B vs blob 171B）。修法：并入在途 `evidence-eol-hygiene` 的 P3 范围，从 blob 重算后核 10/10。

## 2. 观察

- O1 现盘 worktree 已有 `primitive_plane_shapes.cu` 的 CRLF（169 对，git status 因 autocrlf 看不出来），单这一项就让探针 +168 B —— 正是上次 P4 的 EOL 问题现形。
- O2 门禁转写：fmt/clippy-cuda 与 runs 同一次；clippy、unit 快照来自更早一次（wall_ms/log_sha 不同）；README §4 已声明「转写快照、权威以 runs/ 为准」，属来源说明而非缺陷。
- O3 `no-run.txt` 是截断转写（Finished + 可执行行数 + EXIT=0），非原文。
- O4 `validate.txt` 快照 0 err/6 warn（含 fingerprint drift，done 刷新前的历史态）；复核时另一次运行的 drift 来自并发在途改动，与本片无关。
- O5 预核 `target/mesh-a1-precheck.md` 的几处行数预测与终稿差 1（contact.cu 271 而非 ~272、multicontact.cu 235、geom 411）；预核是 target-only 参考，实际文件远低于 500 行 ratchet。
- O6 注释契约逐条一致（见 §1）。
- O7 U8 的 1→4 只是上游参考期望，本片零设备执行；U7 单接触回退如实记录为未覆盖（A2）。

## 3. 边界 / 不宣称

只读 + 两次定向 CPU 构建（NVRTC 纯编译探针、模块单测），未跑 GPU、未开窗、未冻结。
不宣称渲染数值验收通过 / 上游等价 / 精度达标；不宣称 U8 已通过或 U7 已覆盖。

## 4. 复现

```
git diff --name-only 307219fe^..307219fe -- src/           # 7 个文件
git show 307219fe:src/physics/collision/cuda/contact.cu | findstr /C:"polygon[40]" /C:"pnormal[20]" /C:"face1[20]"
cargo test --locked --lib physics::collision::gjk::multicontact -- --nocapture          # 2 passed
PATH=target\toolchains\cuda-12.8.1\bin;%PATH% cargo test --locked --features cuda-probe --lib nvrtc_smoke -- --ignored --nocapture
# => ptx_bytes=3883769 entries=6 source_bytes=254431 sha=af1791be..（当前 EOL 状态；LF blob 复算见 materials/selfcheck-source.txt）
```

`materials/selfcheck-*.txt` 是本次重算输出（scope / source / gates）。
