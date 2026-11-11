# A2-2：collision 内核身份重采 + 10 exe 重冻 + fixture 13→15 + NP02 重采（0 窗、CPU-only）

- 任务：`2026-11-12-w2-a2-2-identity-refreeze`（owner Backend；W2-A2 第 2 片，0 窗、CPU-only、零 GPU）。
- 日期：2026-11-12。基线 HEAD `83ccaa13`（A2-1 verified `2ee0540d` + QA 复核 `ea2d1402` + U7/U8 fix `83ccaa13`）。
- 依据：Chief 2026-11-12 放行；Arch `u7-u8-promotion/a2-slices.json` A2-2（identity/refreeze/NP02 范围）。
- 边界：不改产品源码（`src/` 零改动，片内 `unchanged src/` 机检）；不改判据/容差/参考；零 GPU、零窗。

## 1. 一句话结论

A2 容量 32/32 后的 collision 内核身份已重采并入库（source `fb03c5b1…`/254,434 B、PTX `78da9dea…`/3,883,643 B/6 入口），
A1 身份（`12d50d68…`/254,263 B、3,883,769 B）以 append-only addendum 声明作废；10 个 S4 冻结 exe 已用 A2 源码重建并
stash 到 `target/a2-freeze/`（10/10 sha 记录、copy_match 全 true）；mesh fixture 13→15（新增 33-gon over-32 边界
`fan-cone-33.obj`/`prism-33.obj`）；NP02 6 行 collision 回执已 CPU-only 重采（6/6 gate pass，PTX `78da9dea…`）
并完成索引追加与 inventory 身份换指（计数不变）。同时完成 A2-1 证据低 L1 修正（policy_scan 0 命中、manifest 10→11）。

## 2. 内核身份（新旧对照）

| 项 | A1（旧，作废） | **A2（现行）** |
| --- | --- | --- |
| 编译单元 | 28 片段装配源 + `product.cu` | 同左（`DEVICE_SOURCES=28` 不变，无新增 `.cu`） |
| source sha256 | `12d50d68d3e7f155e688f9dab2f69d6d30720a6b07804c7a0a026dd38dbd34b7` | **`fb03c5b10956685408ef720769e69f14560db472aa3516c98b30e59dc8956d81`** |
| source bytes | 254,263 | **254,434** |
| PTX sha256 | `54ab59e3abad7616e219848fda12dc8f6aeba9d0bf3bbe4c09e603d53369a40b` | **`78da9deafb9c8b787120d1cd1ca7d2c560271d2e67b39b45f21852cc6d1ef59d`** |
| PTX bytes | 3,883,769 | **3,883,643** |
| 入口 | 6 | 6（`mjwarp_collision_{boxbox,contact,contact_multi,hfield,primitive,sdf}`） |
| 测量口径 | NVRTC 12.8.93、LF worktree、CPU-only（`nvrtc_smoke`） | 同左（本片复测） |

- 探针：`cargo test --locked --features cuda-probe --lib nvrtc_smoke -- --ignored --nocapture`
  → `nvrtc_smoke_ok ptx_bytes=3883643 entries=6 source_bytes=254434 source_sha256=fb03c5b1…`（`logs/nvrtc-identity.txt`）。
- **A1 身份 supersession**：`docs/development-evidence/g10-p5-mesh-cap20/addenda-a2-identity.md`（append-only，追加；
  A1 正文与历史字节不改，其 manifest 追加 1 条 10→11）。后续任何 GPU 窗口必须用 A2 新冻结，旧 guards/exe 一律不复用。

## 3. 10 exe 重冻（逐项落地）

- 工具链：`+1.99.0-x86_64-pc-windows-msvc`；`MJWARP_MUJOCO_ROOT=target/toolchains/mujoco-3.12.0/package`；
  `cargo test --locked --all-features --no-run --test <target>`；复制到 `target/a2-freeze/` 后重哈希。
- 逐项 sha256/bytes/copy_match：`refreeze-10-exe.json`（机器可读，含源码树 digest `266bbf94…`、29 个 `.cu`）
  与 `refreeze-10-exe.tsv`；**10/10 copy_match=true**。
- 文件名沿用 cargo 稳定后缀（与 S4 同名不同内容）：如 `g10_p5_branch_closure_cuda-1b98cdc445ce5abc.exe`（A2 bytes 41,631,232）。
- 旧 stash（`target/s4-freeze/`、`target/m3-freeze/`）**作废**（只历史参照）；A2-3 窗口用本目录记录的新冻结。


## 4. NP02 6 行 collision 回执重采（CPU-only）

- 新波次目录：`docs/development-evidence/np02-evidence/a2-collision-2026-11/`
  （6×`<entry>-<ptxSha>.{cu,ptx,json}` + `capture-report.json` + README + manifest；NVRTC 12.8.93、不建上下文）。
- 6/6 精度门禁 `f32-physics` pass；PTX `78da9dea…`/3,883,643 B；source `fb03c5b1…`/254,434 B。
- 补采索引 `np02-evidence/receipt.json` 追加 6 条（25 → **33**，幂等）；旧回执保留在 `new-kernels-2026-11/receipts/` 不改写。
- `inventory.json` 6 行 collision 的 `ptx_sha256`/`evidence_paths` 改指新回执（identity-only swap）：
  `receipts_total=715`、`receipt_ptx_entities_verified=711` 等**计数不变**；其余 96 行零改动。
- **inventory 全量重算未在本片执行**：本机 scanner 基线（100 路径 / 33 缺口）与仓内清单（102 / 32）已存在
  既存差异（NP5-03 后的 patched 状态），重算会回退 102→100 等口径；该口径更新归 NP5-03 面，本片只做身份换指并如实记录。
- 映射表：`np02-identity-mapping.json`（旧 `54ab59e3…` → 新 `78da9dea…`，逐 entry）。

## 5. fixture 13→15（33-gon over-32 边界）

- 新增：`fixtures/g10-p5-mesh-multicontact/fan-cone-33.obj`（`58908058…`/1,425 B）、`prism-33.obj`（`bb1b05c2…`/3,382 B）；
  33 边形/33 度扇，用于 over-32 边界（host guard 33 → 确定容量错误；设备 >32 回退为 record-only）。
- `tests/support/branch_closure_case.rs::FIXTURE_SHA256` 13→**15**（新增两条守卫）；CPU 签名测试
  `fixtures_match_s3_signature` 通过（`cargo test --locked --features cuda-probe --test g10_p5_branch_closure_cuda`
  → 3 passed / 2 ignored，`logs/fixture-guard-test.log`）。
- 边界：over-32 **不进 pass/fail**；设备 over-32 case 是否入 A2-3 窗待 Chief/Arch 决定（见 A2-3 计划 §5）。

## 6. A2-1 证据低 L1 修正 + 观察记录（第一节）

- `docs/development-evidence/g10-p5-mesh-cap32/README.md` §6：非声明列表的连写短语改为否定式
  「不作位级等价声明、不提供位级等价保证」；`addenda.md` 记录 L1（原文留档于闭标记引用块并注明已作废）、
  O1（「8 条」实为 **7 组 9 条** = clip 3 + 宿主 4 + A1 2）、O2（`rejects_fan_cone_24…` 改名为
  `accepts_fan_cone_24_within_capacity_32` 的说明）。
- 该证据 manifest **10→11** 重算；`policy_scan` 对 A2-1 证据 12 个文件 **0 命中**（`logs/policy-scan-a2-1-evidence.log`）；
  `policy_scan --self-test` **34/34 PASS**。
- A2-1 结论/数字/验收状态不变（仍 verified）。

## 7. render guards 不受影响（写明）

- G-L2 render 冻结 guards（如 `guards-perworld-l2-light.json`，`94478b3d…`/43,729 B 与 93 输入清单）**不含**
  collision 设备源；A2 只改 collision 容量与 fixtures/tests 支撑，**渲染数值行为不变**。
- 但 collision 源变更会改变**链接同一库的 Rust 测试 exe** 的字节：任何重建 Rust exe 的窗口（含渲染 suite）
  exe 身份都会变，**若将来重跑仍须重冻**；既有 render 窗口证据不重写、不混用。不得把 A2 身份混入 render guards。

## 8. A2-3 窗口计划（只计划、不排窗）

见同目录 `a2-3-window-plan.md`（10 exe 清单、M2/M3/S4+U7/U8 改判+A1 回归+A2 边界+NP02 七行矩阵、
monitor/三点基线/floor 取严/fail-stop/零 Rust 编译）。由 Chief 按四前提开窗，开窗前群里留痕。

## 9. 边界与非声明

- 不改任何产品源码（`src/` 零改动）；不改判据/容差/参考；**不宣称** U7/U8 通过、mesh 容量缺口已清、
  与上游等价（over-32）、G10@P5 整族通过、NP01/NP02 门禁通过、性能结论。
- 零 GPU、零窗；CPU-only 构建与 NVRTC 编译。

## 10. 证据文件与 manifest

- `README.md`（本文件）、`receipt.json`（机器可读）
- `refreeze-10-exe.json` / `refreeze-10-exe.tsv`（10 exe 逐项）
- `np02-identity-mapping.json`（NP02 旧→新身份映射）
- `a2-3-window-plan.md`（窗口计划）
- `logs/nvrtc-identity.txt`、`logs/np02-capture.log`、`logs/policy-scan-a2-1-evidence.log`、
  `logs/fixture-guard-test.log`、`logs/refreeze-build.log`
- `manifest.txt`（逐文件 sha256+bytes 自核，不列自身）
