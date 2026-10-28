# 行数实测（LF 换行口径，权威）

口径：`\n` 字节计数（.NET / Python 读原始字节，不用 `Get-Content`；本环境 `Get-Content` 少计）。

| 文件 | 父提交 HEAD（LF） | 本次（LF） | 上限 | 说明 |
| --- | --- | --- | --- | --- |
| `src/render/kernel/cast.rs` | 411 | **481** | 500（单文件棘轮） | +72/−2（`git diff --stat`）；新增 warp 收缩序 `warp_quat_to_mat`（含文档注释）+ 1 条四案例位型单测 |

实测命令（仓库根目录）：

```powershell
cmd /c "git cat-file blob HEAD:src/render/kernel/cast.rs > $env:TEMP\cast_head.rs"
python -c "import pathlib; cur=pathlib.Path('src/render/kernel/cast.rs').read_bytes(); head=pathlib.Path(r'$env:TEMP\cast_head.rs').read_bytes(); print(cur.count(b'\n'), head.count(b'\n'))"
```

结果：`481 411`（新增 70 行净增；文件内无 CRLF：`crlf count = 0`）。

说明：卡面预核为 411→477（+68/−2）；实际落地 481 行，差值来自两点：
1. 收缩链改用项目精确 FMA `crate::math::fma::fma_f32`（见 `fma-toolchain-note.md`），文档注释多 3 行；
2. 单测把 4 条位型案例（恒等 / hfield cam-side / scene-a cam-persp / 随机抽样反例）收进同一 `#[test]`。

481 ≤ 500，满足棘轮；已按 Arch 要求对 rustfmt 后最终行数负责。
