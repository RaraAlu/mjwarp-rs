# QA independent product-level frame matrix (case_evidence frame diagnosis review).
# Every cell: brand-new CARGO_TARGET_DIR (deleted first), CARGO*/RUST* cleared
# and recorded, rustc -vV + cargo -V + deps count recorded; raw cargo output kept.
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
$OutputEncoding = [Text.UTF8Encoding]::new($false)
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$repo = 'D:\Rust\mjwarp-rs'
$out = Join-Path $repo 'target\qa-frame-diag\frame'
$evidence = Join-Path $repo 'docs\development-evidence\case-evidence-frame-diagnosis-independent-check'
New-Item -ItemType Directory -Force -Path $out | Out-Null
$tree = Join-Path $repo 'target\qa-frame-diag\tree'
$treeNoConfig = Join-Path $repo 'target\qa-frame-diag\tree-noconfig'

$cells = @(
    @{ id = 'gnu-default'; dir = $tree; toolchain = ''; rustflags = ''; release = $false },
    @{ id = 'msvc-default'; dir = $tree; toolchain = '+1.99.0-x86_64-pc-windows-msvc'; rustflags = ''; release = $false },
    @{ id = 'gnu-fma'; dir = $tree; toolchain = ''; rustflags = '-C target-feature=+fma'; release = $false },
    @{ id = 'msvc-fma'; dir = $tree; toolchain = '+1.99.0-x86_64-pc-windows-msvc'; rustflags = '-C target-feature=+fma'; release = $false },
    @{ id = 'gnu-release'; dir = $tree; toolchain = ''; rustflags = ''; release = $true },
    @{ id = 'msvc-release'; dir = $tree; toolchain = '+1.99.0-x86_64-pc-windows-msvc'; rustflags = ''; release = $true },
    @{ id = 'msvc-plus-ldc'; dir = $tree; toolchain = '+1.99.0-x86_64-pc-windows-msvc'; rustflags = '-C link-dead-code=yes'; release = $false },
    @{ id = 'gnu-no-config'; dir = $treeNoConfig; toolchain = ''; rustflags = ''; release = $false }
)

$testFilter = 'physics::collision::gjk::case_evidence::upstream_f32_within_4ulp'
$printFilter = 'physics::collision::gjk::case_evidence::our_contacts_match_upstream_f32_within_4ulp'

function Clear-CargoEnv([string]$snapshotPath) {
    $before = @(Get-ChildItem Env: | Where-Object { $_.Name -match '^(CARGO|RUST)' } | Sort-Object Name)
    $lines = @('snapshot=before_clear')
    foreach ($entry in $before) { $lines += "env:$($entry.Name)=$($entry.Value)" }
    $lines += "count=$($before.Count)"
    $lines | Out-File -Encoding utf8 $snapshotPath
    foreach ($entry in $before) { Remove-Item "Env:$($entry.Name)" -ErrorAction SilentlyContinue }
    $after = @(Get-ChildItem Env: | Where-Object { $_.Name -match '^(CARGO|RUST)' })
    "snapshot=after_clear count=$($after.Count)" | Out-File -Encoding utf8 -Append $snapshotPath
    return $before.Count
}

function Invoke-Cargo([string]$toolchain, [string[]]$tail, [string]$log) {
    $argv = $tail
    if ($toolchain -ne '') { $argv = @($toolchain) + $argv }
    & cargo @argv *> $log
    return $LASTEXITCODE
}

function Get-Line([string]$log, [string]$pattern) {
    $match = Select-String -Path $log -Pattern $pattern -ErrorAction SilentlyContinue
    if ($match) { return ($match | Select-Object -Last 1).Line.Trim() }
    return ''
}

$rows = @()
foreach ($cell in $cells) {
    $id = $cell.id
    $td = Join-Path $repo ("target\qa-frame-diag\frame\{0}" -f $id)
    if (Test-Path $td) { Remove-Item -Recurse -Force $td }
    $envBefore = Clear-CargoEnv (Join-Path $out ("env-{0}.txt" -f $id))
    $env:CARGO_TARGET_DIR = $td
    if ($cell.rustflags -ne '') { $env:RUSTFLAGS = $cell.rustflags } else { Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue }
    Push-Location $cell.dir
    $rustcArgs = @('-vV')
    if ($cell.toolchain -ne '') { $rustcArgs = @($cell.toolchain) + $rustcArgs }
    $rustcV = (& rustc @rustcArgs) -join "`n"
    $cargoArgs = @('-V')
    if ($cell.toolchain -ne '') { $cargoArgs = @($cell.toolchain) + $cargoArgs }
    $cargoV = (& cargo @cargoArgs) -join "`n"
    "toolchain=$($cell.toolchain)`r`nrustflags=$($cell.rustflags)`r`nrelease=$($cell.release)`r`ntree=$($cell.dir)`r`ntarget=$td`r`n$rustcV`r`n$cargoV" | Out-File -Encoding utf8 (Join-Path $out ("toolchain-{0}.txt" -f $id))

    $releaseArgs = @()
    if ($cell.release) { $releaseArgs = @('--release') }
    $log1 = Join-Path $out ("{0}-assert.log" -f $id)
    $log2 = Join-Path $out ("{0}-print.log" -f $id)
    $sw = [Diagnostics.Stopwatch]::StartNew()
    $argv1 = @('test', '--locked') + $releaseArgs + @('--lib', $testFilter, '--', '--exact')
    $argv2 = @('test', '--locked') + $releaseArgs + @('--lib', $printFilter, '--', '--exact', '--nocapture')
    $exit1 = Invoke-Cargo $cell.toolchain $argv1 $log1
    $exit2 = Invoke-Cargo $cell.toolchain $argv2 $log2
    $sw.Stop()
    $profileDir = 'debug\deps'
    if ($cell.release) { $profileDir = 'release\deps' }
    $depsDir = Join-Path $td $profileDir
    $deps = 0
    if (Test-Path $depsDir) { $deps = @(Get-ChildItem $depsDir -File).Count }
    $frameLine = Get-Line $log2 'ellipsoid-box'
    $frame = ''
    if ($frameLine -match 'max abs ([0-9.eE+-]+)') { $frame = $matches[1] }
    $rows += [ordered]@{
        id = $id
        tree = ($cell.dir -replace [regex]::Escape($repo), '.')
        target_dir = ("target\qa-frame-diag\frame\{0}" -f $id)
        command_assert = 'cargo ' + (($argv1) -join ' ')
        command_print = 'cargo ' + (($argv2) -join ' ')
        toolchain = $cell.toolchain
        rustc_host = ($rustcV -split "`n" | Where-Object { $_ -like 'host:*' } | Select-Object -First 1)
        rustflags = $cell.rustflags
        release = $cell.release
        cargo_rust_env_before_count = $envBefore
        exit_assert_test = $exit1
        assert_result_line = (Get-Line $log1 'test result:')
        assert_panic_frame = (Get-Line $log1 'ellipsoid-box frame')
        exit_print_test = $exit2
        print_frame_line = $frameLine
        frame_value = $frame
        deps_files = $deps
        wall_s = [math]::Round($sw.Elapsed.TotalSeconds, 1)
    }
    Pop-Location
    Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
    Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue
    Write-Host "${id}: assert_exit=$exit1 print_exit=$exit2 frame=$frame deps=$deps"
}

# env-var exclusion: same gnu-default binary re-run with G2_DUMP_NATIVE_ERROR=1
$env:CARGO_TARGET_DIR = Join-Path $repo 'target\qa-frame-diag\frame\gnu-default'
$env:G2_DUMP_NATIVE_ERROR = '1'
Push-Location $tree
$log3 = Join-Path $out 'gnu-default-envset.log'
$exit3 = Invoke-Cargo '' @('test', '--locked', '--lib', $testFilter, '--', '--exact') $log3
Pop-Location
Remove-Item Env:G2_DUMP_NATIVE_ERROR -ErrorAction SilentlyContinue
Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
$rows += [ordered]@{
    id = 'gnu-default-envset'
    note = 'same binary as gnu-default, G2_DUMP_NATIVE_ERROR=1'
    exit_assert_test = $exit3
    assert_result_line = (Get-Line $log3 'test result:')
}

$rows | ConvertTo-Json -Depth 5 | Out-File -Encoding utf8 (Join-Path $evidence 'frame-matrix.json')
Write-Host 'FRAME_MATRIX_DONE'
