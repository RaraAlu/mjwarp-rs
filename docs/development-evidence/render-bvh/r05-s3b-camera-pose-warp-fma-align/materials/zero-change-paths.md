# 零改动路径核对（HEAD a7beafae + 本片工作树）

```text
$ git diff --stat HEAD -- src/render/kernel/cast.rs
 src/render/kernel/cast.rs | 74 +++++++++++++++++++++++++++++++++++++++++++++--
 1 file changed, 72 insertions(+), 2 deletions(-)

$ git status --short -- src/math src/render/kernel/cuda tests fixtures scripts src/render/bvh
# （空输出 = 零改动）
# <empty>

$ git diff --stat HEAD -- src/math src/render/kernel/cuda tests fixtures scripts src/render/bvh
# <empty>
```

- `src/math/mod.rs`、`src/math/fma.rs`：零改动（`fma_f32` 只被调用）。
- `.cu` 设备源：零改动；`DEVICE_SOURCES=28`、81 槽、`CollisionArgs=504 B` 不变（见 kernel-gate-after.log 同源）。
- `tests/`、`fixtures/`、`tests/support/render_bvh/hfield_scene.rs`：零改动。
- 判据/容差/参考/fixture：未改。
- 说明：工作树 `src/render/kernel/cuda/render_megakernel.cu` 曾因环境原因呈 CRLF（git blob 仍 LF，`git status` 曾误报 M）；已按 committed blob 归一化回 LF 后 git 无 diff，内核门 source_sha256 回到 72bc364c…（详见 kernel-gate-after.log）。
