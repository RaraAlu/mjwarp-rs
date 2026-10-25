# Faithful-FMA impact measurement: GPU test binaries built OUTSIDE the window.
# Two modes; each gets its own fresh CARGO_TARGET_DIR (clean-build criterion).
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
$OutputEncoding = [Text.UTF8Encoding]::new($false)
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$repo = 'D:\Rust\mjwarp-rs'
$tree = Join-Path $repo 'target\worktrees\faithful-5e000cb9'
$out = Join-Path $repo 'target\faithful'
$evidence = Join-Path $repo 'docs\development-evidence\faithful-impact-measurement'
$targets = @(
    'g10_p5_nvrtc_launcher', 'g10_p5_primitive_cuda', 'g10_p5_multicontact_cuda',
    'render_kernel_depth_seg', 'render_kernel_rgb', 'render_kernel_flags', 'render_e2e'
)
$modes = @(
    @{ id = 'gnu-default'; rustflags = '' },
    @{ id = 'gnu-fma'; rustflags = '-C target-feature=+fma' }
)

foreach ($mode in $modes) {
    $id = $mode.id
    $td = Join-Path $out ("{0}-cuda" -f $id)
    if (Test-Path $td) { Remove-Item -Recurse -Force $td }
    $env:CARGO_TARGET_DIR = $td
    if ($mode.rustflags -ne '') { $env:RUSTFLAGS = $mode.rustflags } else { Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue }
    $env:CUDA_PATH = (Join-Path $repo 'target\toolchains\cuda-12.8.1')
    $env:PATH = (Join-Path $env:CUDA_PATH 'bin') + ';' + $env:PATH
    Push-Location $tree
    $log = Join-Path $evidence ("logs/gpu-build-{0}.log" -f $id)
    $tail = @('test', '--locked', '--features', 'cuda-probe') + ($targets | ForEach-Object { @('--test', $_) }) -join ' '
    $argv = @('test', '--locked', '--features', 'cuda-probe')
    foreach ($t in $targets) { $argv += @('--test', $t) }
    $argv += @('--no-run')
    & cargo @argv *> $log
    $exit = $LASTEXITCODE
    Pop-Location
    $deps = 0
    $depsDir = Join-Path $td 'debug\deps'
    if (Test-Path $depsDir) { $deps = @(Get-ChildItem $depsDir -File).Count }
    $exes = @(Get-ChildItem $depsDir -Filter '*.exe' -ErrorAction SilentlyContinue | Where-Object { $_.Name -match '^(g10_p5_|render_)' } | Measure-Object)
    "mode=$id exit=$exit deps=$deps test_exes=$($exes.Count)" | Out-File -Encoding utf8 (Join-Path $evidence ("logs/gpu-build-{0}-summary.txt" -f $id))
    Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
    Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue
    Write-Host ("{0}: build_exit={1} deps={2} test_exes={3}" -f $id, $exit, $deps, $exes.Count)
}
Write-Host 'GPU_PREPARE_DONE'
