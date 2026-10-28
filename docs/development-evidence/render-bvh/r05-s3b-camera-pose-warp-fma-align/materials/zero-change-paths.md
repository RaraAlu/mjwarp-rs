# 零改动路径核对（OTA 观察 O4 后：只写终态可核事实，不写不可核的过程叙述）

> 说明：本文件不再保留任何「曾经如何」的过程叙述（例如某次 `source_sha256` 中间值）；只列可从
> 当前仓库直接复核的终态事实与复核命令。

## 1. 设备源 `.cu`：工作树与 HEAD blob 逐字节相同

复核命令（仓库根目录）：

```powershell
cmd /c "git cat-file blob HEAD:src/render/kernel/cuda/render_megakernel.cu > $env:TEMP\meg_head.cu"
python -c "import hashlib,pathlib; wt=pathlib.Path('src/render/kernel/cuda/render_megakernel.cu').read_bytes(); bl=pathlib.Path(r'$env:TEMP\meg_head.cu').read_bytes(); print('equal', wt==bl, 'bytes', len(wt), 'sha256', hashlib.sha256(wt).hexdigest(), 'LF', wt.count(b'\n'), 'CR', wt.count(b'\r\n'))"
git diff --quiet -- src/render/kernel/cuda/render_megakernel.cu; echo diff_quiet_exit=$LASTEXITCODE
git status --porcelain -- src tests fixtures scripts
```

实测（当前 HEAD）：

```text
equal True  bytes 17814  sha256 62f0aa8af45b947feb2d1b7eade07fe0ebe817885126267a66826e57b7213376
LF 347  CR 0
diff_quiet_exit=0
# git status --porcelain -- src tests fixtures scripts ：空输出
```

⇒ 设备源与 HEAD 完全一致：**本片未改 `.cu`**。

## 2. 本片 diff 只含一个生产文件

```text
$ git diff --stat a7beafae..HEAD -- src
 src/render/kernel/cast.rs | 74 ++++++-----...     （411 → 481 行，+72/−2）
# 其余 src/ 路径：空
$ git status --short -- src/math src/render/kernel/cuda src/render/bvh tests fixtures scripts
# 空输出
```

- `src/math/mod.rs`、`src/math/fma.rs`：零改动（`fma_f32` 只被调用）。
- `tests/`、`fixtures/`、`tests/support/render_bvh/hfield_scene.rs`：零改动。
- 判据/容差/参考/fixture：未改。

## 3. 内核门三 sha 与父提交一致（设备源未变的行为证据）

`cargo test --locked --features cuda-probe --test render_kernel_rgb -- --ignored
megakernel_source_compiles_under_render_gate --test-threads=1 --nocapture` 输出：

```text
s7b2 ptx: source_sha256=72bc364c4b732e278396d2aebd09625872f6dcecba469a98a3e40aba04851746
          ptx_sha256=97467d835e27107a4a0803678e194f4868fb552707543a544e270cbde8a9bd13
          ptx_bytes=264015
```

（原始输出 `materials/kernel-gate-after.log`，UTF-8 LF 无 BOM）

- `DEVICE_SOURCES=28`、81 槽、`CollisionArgs=504 B` 不变。
- 结论：只改 host Rust，内核源/PTX 逐字节不变。

## 4. 全树状态

```text
$ git status --porcelain -- src tests fixtures scripts
# 空输出（本文件提交时；B 的 guards.json 改动已按 -text 属性原字节提交）
```
