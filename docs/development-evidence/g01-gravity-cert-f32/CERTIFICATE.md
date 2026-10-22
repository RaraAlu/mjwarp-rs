# G01 重力位证书（f32 逐步口径）

任务：`2026-10-22-g01-gravity-cert-f32-regen`。
由 `gen.rs` 独立生成；输入只用测试模型与参数批，不读取 GPU 输出。

| world | f32 逐步期望 bits | 旧精确证书 bits | 对比 |
| --- | --- | --- | --- |
| 0 | 0x420c0000 0x41b00000 0x40800000 0x41000000 | 0x420c0000 0x41b00000 0x40800000 0x41000000 | 一致 |
| 1 | 0x40b00000 0x40400000 0x3efffffc 0x3f000000 | 0x40b00000 0x40400000 0x3f000000 0x3f000000 | 口径差异 |
| 2 | 0x41280000 0x40900000 0x3f000000 0x40000000 | 0x41280000 0x40900000 0x3f000000 0x40000000 | 一致 |
| 3 | 0x41040000 0x40a80000 0x3f800000 0x40800000 | 0x41040000 0x40a80000 0x3f800000 0x40800000 | 一致 |
| 4 | 0x41200000 0x40900000 0x3efffffc 0x40000000 | 0x41200000 0x40900000 0x3f000000 0x40000000 | 口径差异 |
| 5 | 0x41540000 0x40e00000 0x3f800000 0x3f000000 | 0x41540000 0x40e00000 0x3f800000 0x3f000000 | 一致 |
| 6 | 0x427a0000 0x42280000 0x41000000 0x40000000 | 0x427a0000 0x42280000 0x41000000 0x40000000 | 一致 |

差异槽：2 处（见 certificate.json 的 differences）。

生成命令：

```powershell
rustc -O docs/development-evidence/g01-gravity-cert-f32/gen.rs -o target/g01-cert/gen.exe
& target/g01-cert/gen.exe docs/development-evidence/g01-gravity-cert-f32
```

逐步舍入点见 `certificate.json`：`subtree_com`（init 乘法、树序累加、除以子树质量）、`cdof`（com−anchor 减法、cross 乘减）、每个贡献的 force/offset/jacobian/product、以及累加；同时给出精确有理值与 f32 bits。
