# W15/W14 交付独立复核（QA）

任务：`2026-10-23-w15-w14-independent-check`。
对象：W15 `b1de2db`（Warp 头文件原样复制）、W14 `f98dc34`（tile 共享内存声明与发射约束）。
方式：只读比对 + 独立跑测试；不占 GPU、不改 Backend 的 `include/warp-headers`、`LICENSES`、`src/runtime/launcher`、`tests/launcher-*.rs`、`THIRD_PARTY_NOTICES.md`。

## 结论

**两份交付与各自声明基本一致，未发现阻断/严重问题；复核通过（QC 口径），另有 1 条一般差异与 3 条建议需后续处理。**

- W15：36 个头文件与 `headers.json` 逐字节一致（36/36 bytes 与 SHA-256），集合无多/少，总字节 `1,520,709`；36/36 与 clone tag `ffa038b1` 的 `warp/native` 文件逐字节一致，36/36 与 wheel 安装头不同——**来源唯一性成立（未混用 wheel）**；`LICENSES/warp-1.15.0/LICENSE.md` 与 clone `LICENSE.md` 逐字节一致（sha256 `6d1d968f…`）；`THIRD_PARTY_NOTICES.md` 来源/许可/清单路径齐全；NVRTC 头文件编译测试通过（compile-only，出 PTX）。
- W14：块大小与 Tile 规则不符、非法 Tile 线程数、未 opt-in 动态共享内存、超默认上限、超 opt-in 上限、静态共享超限、静态与动态互斥、非法 `SHARED_BYTES`（4 字节对齐）均有实现与对应测试；`BlockRule::Tile` 的 32 倍数/≤1024、`SharedMemoryLimits` 默认 48KiB、`validate_rule` 严格等块大小与 M7-tile 冻结约束（`docs/development-evidence/layers-w14.md`）一致；全 `src` 扫描 60 处产品 `LaunchDims::new` 的共享内存实参**全部为 0**；NVRTC tile 测试只编译 PTX（断言入口名与 `.shared`），不启动 GPU 内核；lib 447 过 0 败（127 ignored）、artifact 单测 14/14、fmt/clippy-cuda 通过。

## 差异清单（按严重级别）

| ID | 级别 | 结论 | 说明 |
| --- | --- | --- | --- |
| W14-F1 | 一般 | 冻结约束未完全接线 | 冻结要求「产品按设备属性核对共享内存上限」；实现是 `SharedMemoryLimits::new(per_block, opt_in)` 由调用方注入 + 默认保守 48KiB，全仓**无** `cudaDeviceGetAttribute`/`MaxSharedMemoryPerBlock` 查询。当前产品内核 `shared_bytes=0`，无现网影响；tile 内核上线前须接线设备查询，否则 >48KiB 的合法静态/opt-in 上限可能与实际设备不符。`w14 README` 该行表述偏强，建议改写为「可注入设备属性 + 保守默认」。**2026-10-23 Backend 已修复**：新增 `DeviceSharedMemoryLimits::query`（cudarc 查询 MAX_SHARED_MEMORY_PER_BLOCK/OPTIN）、`SharedMemoryLimits::intersect/for_device`（查询失败回退保守默认），模拟+真实设备测试通过；W14 README 表述已更新。证据 `docs/development-evidence/w14-shared-memory-device-limit/`。 |
| REPRO-F1 | 建议 | 复现命令缺环境前置 | W14/W15 README 的 ignored 测试命令只设 `MJWARP_CUDA_INCLUDE`；实测仅设该变量时报 `NVRTC不可用`。把 `target/toolchains/cuda-12.8.1/bin` 加入 `PATH` 后两项 ignored 测试均通过（见 `test-runs.json`）。属环境前置登记缺失，非实现缺陷。**2026-10-23 已补**：W14/W15 README 的复现命令已加 `target/toolchains/cuda-12.8.1/bin` 入 `PATH` 说明。 |
| W15-F1 | 建议 | 清单样本性 | `headers.json.wheel_vs_clone` 只列 7 个代表文件；独立比对显示 36 个文件**全部**与 wheel 不同（36/36 与 clone 一致）。结论被加强而非推翻，建议说明样本性质或补全。 |
| W15-F2 | 建议 | SDK 头摘要进键的调用点条件 | `cuda_fp16.h` 内容摘要进 `ArtifactKey` 只在 W15 NVRTC probe 测试登记（`with_external_header`）；W14 tile 编译测试只传 include 路径，键中只有路径字符串。当前产品路径未用 Warp 头树，无现网影响；若后续产品走上游源码路线，需在调用点登记 SDK 头摘要，`THIRD_PARTY_NOTICES` 的表述才完全成立。**2026-10-23 已补**：W14 tile NVRTC 测试登记 `cuda_fp16.h` 摘要进键并断言键变化。 |

## 测试与核对结果（本次实跑）

| 项 | 命令 | 结果 |
| --- | --- | --- |
| W15 头文件清单 | `cargo test --locked --features cuda-probe --test launcher-warp-headers` | 2 过 0 败 1 ignored |
| W15 NVRTC | 同上 `-- --ignored`（`PATH` + `MJWARP_CUDA_INCLUDE`） | 1 过 0 败（出 PTX，缓存命中/换键断言） |
| W14 拒绝面 | `cargo test --locked --features cuda-probe --test launcher-tile` | 3 过 0 败 1 ignored |
| W14 NVRTC tile | 同上 `-- --ignored` | 1 过 0 败（PTX 含入口名与 `.shared`） |
| artifact 单测 | `cargo test --locked --features cuda-probe --lib runtime::launcher::artifact` | 14 过 0 败 |
| launcher 全 lib | `cargo test --locked --features cuda-probe --lib` | 447 过 0 败 127 ignored |
| 门禁 | `cargo fmt --check`、`cargo clippy --all-targets --features cuda-probe -- -D warnings` | 均通过（exit 0） |

环境：`rustc 1.99.0`、Windows `10.0.26200`、CUDA 12.8.1 工具链（仅 NVRTC 编译，**未启动 GPU 内核**）。

## 文件

- `asset-check.json`：W15 逐文件比对（manifest/clone/wheel）、许可与声明、W14 代码事实与测试覆盖、产品 `LaunchDims` 扫描、差异清单。
- `test-runs.json`：8 项命令的退出码/摘要/耗时与环境记录（`run-tests.ps1` 生成）。
- `asset-check.mjs`、`run-tests.ps1`：只读复核脚本，可复跑。

## 复现

```powershell
node docs/development-evidence/w15-w14-independent-check/asset-check.mjs
powershell -NoProfile -File docs/development-evidence/w15-w14-independent-check/run-tests.ps1
```

## 边界

- 未做 GPU 内核执行、tile 装卸/清零与失败路径「半个 tile」验证——W14 已在边界中声明，本轮不补 GPU 证据，也不据此判定通过。
- 未做 CUDA Toolkit 许可/再分发法律结论，只核对仓内许可与来源声明是否一致。
- 未重跑完整 G01/T4 验收，未改任何产品代码、测试、容差、fixtures 或 Backend 文件。
- 本复核不改变 W15/W14 的验收状态；差异如需修复由对应 Backend 任务处理。
