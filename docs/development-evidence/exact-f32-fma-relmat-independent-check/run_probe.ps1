# QA independent probe runner for the exact f32 FMA helper check.
# Builds the self-written probe for 3 builds, runs it with and without the
# test-only seam env, and emits the product snapshot assembly to check vfmadd.
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
$OutputEncoding = [Text.UTF8Encoding]::new($false)
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$repo = 'D:\Rust\mjwarp-rs'
$evidence = Join-Path $repo 'docs\development-evidence\exact-f32-fma-relmat-independent-check'
$probeDir = Join-Path $evidence 'probe'
$out = Join-Path $repo 'target\qa-fma-check\probe'
$logs = Join-Path $evidence 'probe-logs'
New-Item -ItemType Directory -Force -Path $out, $logs | Out-Null
$probeSrc = Join-Path $probeDir 'qa_fma_probe.rs'
$asmSrc = Join-Path $probeDir 'qa_fma_asm.rs'

$builds = @(
    @{ tag = 'gnu-default'; toolchain = ''; extra = @() },
    @{ tag = 'gnu-fma'; toolchain = ''; extra = @('-C', 'target-feature=+fma') },
    @{ tag = 'msvc-default'; toolchain = '+1.99.0-x86_64-pc-windows-msvc'; extra = @() }
)

$buildInfo = @('# probe build info', ('utc=' + (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ')))
foreach ($build in $builds) {
    $rustcArgs = @('--edition', '2024', '--test', '-C', 'opt-level=0', '-C', 'debug-assertions=on') + $build.extra
    $argv = $rustcArgs
    if ($build.toolchain -ne '') { $argv = @($build.toolchain) + $argv }
    $exe = Join-Path $out ("probe-{0}.exe" -f $build.tag)
    & rustc @argv @('-o', $exe, $probeSrc)
    if ($LASTEXITCODE -ne 0) { throw "probe compile failed: $($build.tag)" }
    $rustcV = (& rustc $(if ($build.toolchain) { @($build.toolchain) } else { @() }) -vV) -join "`n"
    $buildInfo += "## $($build.tag): rustc $($build.toolchain) $($rustcArgs -join ' ')"
    $buildInfo += $rustcV
    $logPlain = Join-Path $logs ("probe-{0}.log" -f $build.tag)
    if (Test-Path $logPlain) { Remove-Item $logPlain -Force }
    & $exe --nocapture --test-threads=1 *> $logPlain
    Write-Host "$($build.tag): plain exit=$LASTEXITCODE"
    $logSeam = Join-Path $logs ("probe-{0}-seam.log" -f $build.tag)
    $env:MJWARP_FORCE_SOFT_FMA = '1'
    & $exe --nocapture --test-threads=1 *> $logSeam
    $seamExit = $LASTEXITCODE
    Remove-Item Env:MJWARP_FORCE_SOFT_FMA -ErrorAction SilentlyContinue
    Write-Host "$($build.tag): seam exit=$seamExit"
}

# assembly check: product snapshot (non-test build) must emit vfmadd231ss
$asmTags = @(
    @{ tag = 'gnu'; toolchain = '' },
    @{ tag = 'msvc'; toolchain = '+1.99.0-x86_64-pc-windows-msvc' }
)
foreach ($asm in $asmTags) {
    $asmFile = Join-Path $out ("asm-{0}.s" -f $asm.tag)
    $argv = @('--edition', '2024', '-C', 'opt-level=0', '--emit=asm', '-o', $asmFile)
    if ($asm.toolchain -ne '') { $argv = @($asm.toolchain) + $argv }
    & rustc @argv $asmSrc
    if ($LASTEXITCODE -ne 0) { throw "asm compile failed: $($asm.tag)" }
    $text = Get-Content $asmFile
    $hits = @()
    for ($i = 0; $i -lt $text.Count; $i++) {
        if ($text[$i] -match 'vfmadd231ss') { $hits += $i }
    }
    $report = @(
        "# assembly check for product helper snapshot (non-test build)",
        "tag=$($asm.tag)",
        "asm_file=$asmFile",
        "size_bytes=$((Get-Item $asmFile).Length)",
        "vfmadd231ss_matches=$($hits.Count)"
    )
    if ($hits.Count -gt 0) {
        $first = $hits[0]
        $report += "first_hit_line=$($first + 1)"
        for ($j = [Math]::Max(0, $first - 4); $j -le [Math]::Min($text.Count - 1, $first + 4); $j++) {
            $report += "$($j + 1): $($text[$j])"
        }
    }
    $report | Out-File -Encoding utf8 (Join-Path $evidence ("asm-vfmadd-{0}.txt" -f $asm.tag))
    Write-Host "asm $($asm.tag): vfmadd231ss matches=$($hits.Count)"
}
$buildInfo | Out-File -Encoding utf8 (Join-Path $evidence 'probe-build-info.txt')
Write-Host 'PROBE_RUNNER_DONE'
