# QA independent review of 2026-10-25-t4-source-hash-repin-13-r2
# Build-environment matrix. Self-built CARGO_TARGET_DIRs only; never reuses
# Backend targets verify-clean-001/002 or their logs.
# Clean-criteria items recorded: dedicated target dir, CARGO_*/RUST* cleared
# (before/after counts), deps file count + rustc -vV + target triple.
$ErrorActionPreference = 'Stop'
$repo = 'D:\Rust\mjwarp-rs'
Set-Location $repo
$out = Join-Path $repo 'target\qa-repin13'
New-Item -ItemType Directory -Force -Path $out | Out-Null

function Write-Lines([string]$path, [string[]]$lines) {
    $lines | Out-File -Encoding utf8 -Path $path
}

function Env-Snapshot([string]$label) {
    $envs = @(Get-ChildItem Env: | Where-Object { $_.Name -match '^(CARGO|RUST)' } | Sort-Object Name)
    $lines = @()
    $lines += "snapshot=$label"
    $lines += "utc=$((Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ'))"
    $lines += "cwd=$((Get-Location).Path)"
    $lines += "CARGO_RUST_env_count=$($envs.Count)"
    foreach ($e in $envs) { $lines += "env:$($e.Name)=$($e.Value)" }
    return $lines
}

# --- HEAD / worktree facts -------------------------------------------------
$head = (& git rev-parse HEAD).Trim()
$status = & git status --porcelain -- src tests fixtures Cargo.toml Cargo.lock scripts native include build.rs
Write-Lines (Join-Path $out 'repo-facts.txt') @(
    "head=$head",
    "utc=$((Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ'))",
    "product_worktree_changes_count=$(@($status).Count)",
    "product_worktree_changes=$($status -join ' | ')"
)

# --- Clean criterion 2: clear CARGO_*/RUST* env (before/after recorded) ----
Write-Lines (Join-Path $out 'env-before.txt') (Env-Snapshot 'before_clear')
$removed = @()
foreach ($e in @(Get-ChildItem Env: | Where-Object { $_.Name -match '^(CARGO|RUST)' })) {
    $removed += $e.Name
    Remove-Item "Env:$($e.Name)" -ErrorAction SilentlyContinue
}
Write-Lines (Join-Path $out 'env-after-clear.txt') (Env-Snapshot 'after_clear')
Write-Lines (Join-Path $out 'env-cleared.txt') @("removed_env_names=$($removed -join ',')", "removed_env_count=$($removed.Count)")

# --- Clean criterion 3a: toolchains ----------------------------------------
$rustcV = (& rustc -vV) -join "`n"
$cargoV = (& cargo -V) -join "`n"
Write-Lines (Join-Path $out 'toolchain-default.txt') @("== rustc -vV ==", $rustcV, "== cargo -V ==", $cargoV)
$msvcRustc = (& rustc +1.99.0-x86_64-pc-windows-msvc -vV) -join "`n"
$msvcCargo = (& cargo +1.99.0-x86_64-pc-windows-msvc -V) -join "`n"
Write-Lines (Join-Path $out 'toolchain-msvc.txt') @("== rustc +msvc -vV ==", $msvcRustc, "== cargo +msvc -V ==", $msvcCargo)

function Count-Files([string]$dir) {
    if (Test-Path $dir) { return @(Get-ChildItem $dir -File -Recurse -ErrorAction SilentlyContinue).Count }
    return 0
}

# --- One measurement step --------------------------------------------------
function Step {
    param(
        [string]$Name,
        [string]$Exe,
        [string[]]$CmdArgs,
        [string]$TargetDir,
        [hashtable]$EnvSet = @{}
    )
    $td = Join-Path $repo $TargetDir
    if (Test-Path $td) { Remove-Item -Recurse -Force $td }
    $depsBefore = 0
    if (Test-Path (Join-Path $td 'debug\deps')) { $depsBefore = @(Get-ChildItem (Join-Path $td 'debug\deps') -File).Count }
    foreach ($k in $EnvSet.Keys) { Set-Item -Path "Env:$k" -Value $EnvSet[$k] }
    $env:CARGO_TARGET_DIR = $td
    $log = Join-Path $out "$Name.log"
    $started = (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ')
    $sw = [Diagnostics.Stopwatch]::StartNew()
    & $Exe @CmdArgs *> $log
    $code = $LASTEXITCODE
    $sw.Stop()
    $ended = (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ')
    $depsAfter = Count-Files (Join-Path $td 'debug\deps')
    $size = 0
    if (Test-Path $td) { $size = [math]::Round((((Get-ChildItem $td -Recurse -File -ErrorAction SilentlyContinue) | Measure-Object Length -Sum).Sum) / 1MB, 1) }
    $rec = [ordered]@{
        name = $Name
        cmd = (@($Exe) + $CmdArgs) -join ' '
        target_dir = $TargetDir
        target_triple = if ($EnvSet.ContainsKey('QA_TARGET_TRIPLE')) { $EnvSet['QA_TARGET_TRIPLE'] } else { '' }
        exit = $code
        wall_s = [math]::Round($sw.Elapsed.TotalSeconds, 1)
        started_utc = $started
        ended_utc = $ended
        deps_files_before = $depsBefore
        deps_files_after = $depsAfter
        target_mib = $size
        log = $log
    }
    ($rec | ConvertTo-Json -Compress) | Out-File -Encoding utf8 -Append (Join-Path $out 'steps.jsonl')
    Remove-Item Env:CARGO_TARGET_DIR
    foreach ($k in $EnvSet.Keys) { Remove-Item "Env:$k" -ErrorAction SilentlyContinue }
    return $rec
}

# --- Step 1: repo-default (GNU active) full suite, no-fail-fast ------------
$s1 = Step -Name 'gnu-default-full' -Exe 'cargo' -CmdArgs @('test', '--locked', '--no-fail-fast') -TargetDir 'target\verify-clean-qa'

# --- Step 2: repo-default single case_evidence test (minimal repro) --------
$s2 = Step -Name 'gnu-default-single' -Exe 'cargo' -CmdArgs @('test', '--locked', '--lib', 'physics::collision::gjk::case_evidence::upstream_f32_within_4ulp', '--', '--exact') -TargetDir 'target\verify-clean-qa'

# --- Step 3: repo-default cuda-probe lib -----------------------------------
$s3 = Step -Name 'gnu-cuda-lib' -Exe 'cargo' -CmdArgs @('test', '--locked', '--features', 'cuda-probe', '--lib') -TargetDir 'target\verify-clean-qa-cuda'

# --- Step 4: explicit MSVC toolchain, native-model-probe + cuda-probe lib --
$g01 = @{
    'MJWARP_MUJOCO_ROOT' = (Join-Path $repo 'target\toolchains\mujoco-3.12.0\package')
    'MJWARP_MUJOCO_DLL' = (Join-Path $repo 'target\toolchains\mujoco-3.12.0\package\bin\mujoco.dll')
    'CUDA_PATH' = (Join-Path $repo 'target\toolchains\cuda-12.8.1')
    'PATH' = ((Join-Path $repo 'target\toolchains\cuda-12.8.1\bin') + ';' + $env:PATH)
}
$s4 = Step -Name 'msvc-cuda-lib' -Exe 'cargo' -CmdArgs @('+1.99.0-x86_64-pc-windows-msvc', 'test', '--locked', '--features', 'native-model-probe,cuda-probe', '--lib') -TargetDir 'target\verify-clean-qa-msvc' -EnvSet $g01

# --- Summary ---------------------------------------------------------------
$summary = @(
    "head=$head",
    "steps=gnu-default-full,gnu-default-single,gnu-cuda-lib,msvc-cuda-lib",
    "gnu_default_full_exit=$($s1.exit)",
    "gnu_default_single_exit=$($s2.exit)",
    "gnu_cuda_lib_exit=$($s3.exit)",
    "msvc_cuda_lib_exit=$($s4.exit)"
) + $s1.Keys
Write-Lines (Join-Path $out 'matrix-summary.txt') $summary
