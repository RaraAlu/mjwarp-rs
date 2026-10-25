# Faithful-FMA impact measurement: CPU-only two-mode matrix.
# Two modes: repo-default GNU (no FMA) vs GNU + RUSTFLAGS=-C target-feature=+fma.
# Worktree pinned at 5e000cb9; every mode gets a brand-new CARGO_TARGET_DIR.
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
$OutputEncoding = [Text.UTF8Encoding]::new($false)
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$repo = 'D:\Rust\mjwarp-rs'
$tree = Join-Path $repo 'target\worktrees\faithful-5e000cb9'
$out = Join-Path $repo 'target\faithful'
$evidence = Join-Path $repo 'docs\development-evidence\faithful-impact-measurement'
New-Item -ItemType Directory -Force -Path $out | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $evidence 'logs') | Out-Null

$modes = @(
    @{ id = 'gnu-default'; rustflags = '' },
    @{ id = 'gnu-fma'; rustflags = '-C target-feature=+fma' },
    @{ id = 'gnu-fma-ldc'; rustflags = '-C target-feature=+fma -C link-dead-code=yes' }
)

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

function Invoke-Cargo([string[]]$tail, [string]$log) {
    & cargo @tail *> $log
    return $LASTEXITCODE
}

$summary = @()
foreach ($mode in $modes) {
    $id = $mode.id
    $td = Join-Path $out $id
    if (Test-Path $td) { Remove-Item -Recurse -Force $td }
    $envBefore = Clear-CargoEnv (Join-Path $evidence ("logs/env-{0}.txt" -f $id))
    $env:CARGO_TARGET_DIR = $td
    if ($mode.rustflags -ne '') { $env:RUSTFLAGS = $mode.rustflags } else { Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue }
    Push-Location $tree
    $rustcV = (& rustc -vV) -join "`n"
    $cargoV = (& cargo -V) -join "`n"
    "mode=$id`r`nRUSTFLAGS=$($mode.rustflags)`r`nCARGO_TARGET_DIR=$td`r`ngit_head=$(& git rev-parse HEAD)`r`n$rustcV`r`n$cargoV" | Out-File -Encoding utf8 (Join-Path $evidence ("logs/toolchain-{0}.txt" -f $id))

    # verbose build (captures the rustc command line for mjwarp_rs => RUSTFLAGS/config interaction)
    $buildLog = Join-Path $evidence ("logs/build-{0}.log" -f $id)
    $exitBuild = Invoke-Cargo @('test', '--locked', '--test', 'g10_p5_sdf_relmat', '--no-run', '-v') $buildLog
    $rustcLine = (Select-String -Path $buildLog -Pattern '--crate-name mjwarp_rs' -ErrorAction SilentlyContinue | Select-Object -Last 1).Line

    $sdfLog = Join-Path $evidence ("logs/sdf-relmat-{0}.log" -f $id)
    $exitSdf = Invoke-Cargo @('test', '--locked', '--test', 'g10_p5_sdf_relmat', '--', '--nocapture') $sdfLog
    $sdfHostLog = Join-Path $evidence ("logs/sdf-host-{0}.log" -f $id)
    $exitSdfHost = Invoke-Cargo @('test', '--locked', '--test', 'g10_p5_sdf_host') $sdfHostLog

    $cePrintLog = Join-Path $evidence ("logs/case-evidence-print-{0}.log" -f $id)
    $exitCePrint = Invoke-Cargo @('test', '--locked', '--lib', 'physics::collision::gjk::case_evidence::our_contacts_match_upstream_f32_within_4ulp', '--', '--exact', '--nocapture') $cePrintLog
    $ceAssertLog = Join-Path $evidence ("logs/case-evidence-assert-{0}.log" -f $id)
    $exitCeAssert = Invoke-Cargo @('test', '--locked', '--lib', 'physics::collision::gjk::case_evidence::upstream_f32_within_4ulp', '--', '--exact') $ceAssertLog

    Pop-Location
    $depsDir = Join-Path $td 'debug\deps'
    $deps = 0
    if (Test-Path $depsDir) { $deps = @(Get-ChildItem $depsDir -File).Count }
    $sdfExe = @(Get-ChildItem (Join-Path $td 'debug\deps') -Filter 'g10_p5_sdf_relmat-*.exe' -ErrorAction SilentlyContinue | Select-Object -First 1)
    $libExe = @(Get-ChildItem (Join-Path $td 'debug\deps') -Filter 'mjwarp_rs-*.exe' -ErrorAction SilentlyContinue | Sort-Object Length -Descending | Select-Object -First 1)
    $modeRow = [ordered]@{
        mode = $id
        rustflags = $mode.rustflags
        cargo_rust_env_before_count = $envBefore
        deps_files = $deps
        build_exit = $exitBuild
        sdf_relmat_exit = $exitSdf
        sdf_host_exit = $exitSdfHost
        case_evidence_print_exit = $exitCePrint
        case_evidence_assert_exit = $exitCeAssert
        rustc_line_has_link_dead_code = ($rustcLine -match 'link-dead-code')
        rustc_line_has_fma = ($rustcLine -match 'target-feature=\+fma')
        rustc_line = $rustcLine
        sdf_relmat_exe_bytes = if ($sdfExe.Count -gt 0) { $sdfExe[0].Length } else { 0 }
        lib_test_exe_bytes = if ($libExe.Count -gt 0) { $libExe[0].Length } else { 0 }
        sdf_output = @(Select-String -Path $sdfLog -Pattern 'SDF-rot' -ErrorAction SilentlyContinue | ForEach-Object { $_.Line.Trim() })
        case_evidence_frame = @(Select-String -Path $cePrintLog -Pattern 'ellipsoid-box' -ErrorAction SilentlyContinue | ForEach-Object { $_.Line.Trim() })
        case_evidence_assert_result = @(Select-String -Path $ceAssertLog -Pattern 'test result:' -ErrorAction SilentlyContinue | ForEach-Object { $_.Line.Trim() })
        case_evidence_panic = @(Select-String -Path $ceAssertLog -Pattern 'ellipsoid-box frame' -ErrorAction SilentlyContinue | ForEach-Object { $_.Line.Trim() })
    }
    $summary += $modeRow
    Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
    Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue
    Write-Host ("{0}: deps={1} sdf_exit={2} ce_assert_exit={3} ldc={4} fma={5}" -f $id, $deps, $exitSdf, $exitCeAssert, $modeRow.rustc_line_has_link_dead_code, $modeRow.rustc_line_has_fma)
}
$summary | ConvertTo-Json -Depth 5 | Out-File -Encoding utf8 (Join-Path $evidence 'cpu-matrix.json')
Write-Host 'CPU_MATRIX_DONE'
