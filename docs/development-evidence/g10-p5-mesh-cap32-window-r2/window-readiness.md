# W2/U7U8 全条件断言片 —— 窗口就绪材料（窗口前）

- 任务：`2026-11-12-w2-u7u8-full-condition-assert`（owner Backend；0 窗片）。
- 基线 HEAD：`f9708d14`（执行期以实际为准）；锁 owner=backend。
- 本文件为**窗口前**材料：不构成窗口记录、不构成通过声明。
- **窗口前 U7/U8 全条件结果未知，不宣称通过**；窗口由 Chief 按四前提开。

## 1. 本片改了什么（0 窗）

- `tests/g10_p5_branch_closure_cuda.rs`：limits 用例从 record-only 升级为 pass/fail——调用现有
  `tests/support/mesh_multicontact_compare_support.rs::compare_case_with_geom_order`（M1/M2/M3 同口径，
  geom 序 `[1,0]`；comparator 495/500 行，本片**不编辑**）；保留原 count/dist/pos 数值打印；
  聚合断言 failures 为空。判据/容差/参考/期望值零改动。
- `tests/support/branch_closure_case.rs`：仅注释口径更新（limits 记录组 → 判据组/U7/U8 pass/fail）。
- 产品 `src/` 零改动；`fixtures/`、`scripts/`、旧证据目录字节不动。

## 2. 内核身份（机检，不变）

| 项 | 值 |
| --- | --- |
| source sha256 / bytes | `fb03c5b10956685408ef720769e69f14560db472aa3516c98b30e59dc8956d81` / 254,434 |
| PTX sha256 / bytes / 入口 | `78da9deafb9c8b787120d1cd1ca7d2c560271d2e67b39b45f21852cc6d1ef59d` / 3,883,643 / 6 |
| NVRTC | 12.8.93；零 GPU 上下文（`nvrtc_smoke`） |
| 29 `.cu` combined digest | `266bbf94fd92cce3202159df196d84824dacbaa6d22f4b848eddd83ac57fcb44`（与 A2-2 冻结一致） |


## 2b. 内核身份机检（本片实跑）

- `logs/nvrtc-identity-r2.txt`：`cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features cuda-probe
  --lib nvrtc_smoke -- --ignored --nocapture`（CPU-only、无 device context）→
  `ptx_bytes=3,883,643 entries=6 source_bytes=254,434 source_sha256=fb03c5b1…`；
  与 A2-2 身份一致（PTX sha256 `78da9dea…` 由 NP02 回执记录，本探针不输出 PTX 哈希）。
- 29 个 `.cu` combined digest `266bbf94…` 由 `refreeze-10-exe-r2.json` 的 `source_tree_digest` 机检记录。

## 3. r2 冻结轮（只重建 branch exe）

- 工具链 `+1.99.0-x86_64-pc-windows-msvc`；env `MJWARP_MUJOCO_ROOT=target/toolchains/mujoco-3.12.0/package`；
  构建命令 `cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --no-run --test g10_p5_branch_closure_cuda`
  （取 `Executable` 行；**不取** gate 变体二进制）。
- 冻结目录 `target/a2-freeze-r2/`；旧 `target/a2-freeze/` 原样保留（A2-3 窗证据引用的旧 branch sha 仍可复核）。
- 逐项 sha/bytes：`refreeze-10-exe-r2.json` / `.tsv`；计数：{"rows": 10, "rebuilt": 1, "copied_unchanged": 9, "old_freeze_match": 9, "copy_match_total": 10}。
- 新 branch exe：`g10_p5_branch_closure_cuda-1b98cdc445ce5abc.exe` / **41771008 B** /
  `6d2b39fcad182e43c85a9faf64ab48a1369614889a60d00befc723efe0373677`；旧的 A2-3 branch exe：`9fe3192bd35f7a68cf5c1d2943e95a69fab8f6abd91ad6d00ccfed4b418cac7a` / 41631232 B（superseded，仅用于历史复核）。
- 9 个未重建 exe：sha/bytes 与旧冻结 **9/9 全等**（manifest `counts.old_freeze_match=9`）。

## 4. 最小窗守卫表（18 = 1 exe + 15 fixture + 2 参考）

逐项 sha 见 `guards-r2.tsv`（runner 同源生成）；夹具与参考 sha 与 A2-3 窗口完全一致。

## 5. 下一窗（Chief 四前提 + GO 后执行）

- 实体：`target/a2-freeze-r2/g10_p5_branch_closure_cuda-1b98cdc445ce5abc.exe --ignored --nocapture --test-threads=1`（只此 1 个）。
- 预计：窗内 45–60 s；窗墙钟 ~70–90 s；上限 300 s/实体、900 s 总。
- fail-stop 8 条沿用（A2-3 就绪包 §5）；第 6 条具体化：U7/U8 comparator failures≠0 即停。
- 浮点判据（零放宽）：U7 `mesh-limits-cone-box` count=1、dist 4 ULP 内、pos 4·ULP(scale)；
  U8 `mesh-limits-prism-box` count=4、dist 4 ULP 内、pos 4·ULP(scale)；结构键排序配对、11 零容差字段、
  overflow==0、efc_address==-1、contact.geom 序 `[1,0]`。
- **窗内 comparator failures≠0 → 即停、留档、不重试、不放宽，报 Chief；U7/U8 继续不宣称通过。**

## 6. 四前提勾表（Chief 填）

| # | 前提 | 判据 | 状态 |
| --- | --- | --- | --- |
| 1 | fresh freeze | `src/` 自冻结零 diff；`git diff 2ee0540d..HEAD -- src/`=0 且 `git diff HEAD -- src/`=0；29 `.cu` digest `266bbf94fd92cce3…`；r2 10 exe sha 与 `refreeze-10-exe-r2.json` 全等；旧 branch exe 未被重建（r2 另目录） | ☐ |
| 2 | monitor 60/60 | `scripts/validate/render-bvh/monitor-until-clean.ps1` 连续 60 干净样本、过滤后 compute 0、构建进程 0；CSV 落 `target/render-bvh/gl2-ptx-diff/w2-a2-precheck/monitor-pre-window-r2.csv` | ☐ |
| 3 | 三点基线 | pre ×3 util/used/free/temp/driver（596.36）记录并贴群 | ☐ |
| 4 | floor 取严 | free ≥ 10,240 MiB；`floor=min(10240, baseline_free−3072)` 取更严；pre/post 复核 | ☐ |
| 附 | 零 Rust 编译 | 起跑前无 cargo/rustc/cl/nvcc；窗内不构建；窗后复核 PASS | ☐ |
| 附 | 单窗互斥 | 全队同一时间仅本窗；不写仓、不发群消息 | ☐ |

## 7. 非声明

- 窗口前 U7/U8 全条件结果未知，不宣称通过；不宣称 mesh 容量缺口已清、与上游等价（over-32）、
  G10@P5 整族通过、NP01/NP02 门禁通过、性能结论。
- 本文件不是窗口记录；窗后结果另落 README/receipt/logs/manifest。
