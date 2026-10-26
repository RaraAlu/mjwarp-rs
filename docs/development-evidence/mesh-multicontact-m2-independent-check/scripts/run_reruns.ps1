# QA M2 independent reruns: NVRTC smoke -> default unit -> all-features.
# Logs are raw UTF-16LE (PowerShell *>); they are converted to UTF-8 in the evidence build step.
$ErrorActionPreference = 'Continue'
$root = 'D:\Rust\mjwarp-rs'
Set-Location $root
$env:PATH = "$root\target\toolchains\cuda-12.8.1\bin;$env:PATH"

$smoke = 'D:\Rust\mjwarp-rs\target\qa-m2\smoke-raw.log'
cargo test --locked --features cuda-probe --lib runtime::collision_gpu::nvrtc_smoke -- --include-ignored --nocapture *> $smoke
"SMOKE_EXIT=$LASTEXITCODE"

$unit = 'D:\Rust\mjwarp-rs\target\qa-m2\unit-raw.log'
cargo test --locked --no-fail-fast *> $unit
"UNIT_EXIT=$LASTEXITCODE"

$env:MJWARP_MUJOCO_ROOT = 'target/toolchains/mujoco-3.12.0/package'
$af = 'D:\Rust\mjwarp-rs\target\qa-m2\all-features-raw.log'
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --no-fail-fast *> $af
"ALL_FEATURES_EXIT=$LASTEXITCODE"

$cuda = 'D:\Rust\mjwarp-rs\target\qa-m2\collision-gpu-subset-raw.log'
cargo test --locked --features cuda-probe --lib runtime::collision_gpu -- --include-ignored *> $cuda
"COLLISION_SUBSET_EXIT=$LASTEXITCODE"

"HEAD_AFTER=$(git log --oneline -1)"
