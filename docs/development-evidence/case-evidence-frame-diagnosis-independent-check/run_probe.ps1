# QA independent f32::mul_add probe runner (frame diagnosis review).
# Builds the review's own probe source with 4 configurations and runs it at
# 4096 and 1e6 deterministic inputs. Raw outputs go to the evidence directory.
$ErrorActionPreference = 'Stop'
$repo = 'D:\Rust\mjwarp-rs'
$evidence = Join-Path $repo 'docs\development-evidence\case-evidence-frame-diagnosis-independent-check'
$out = Join-Path $repo 'target\qa-frame-diag\probe'
New-Item -ItemType Directory -Force -Path $out | Out-Null
$source = Join-Path $evidence 'qa_fmaf_probe.rs'

$configs = @(
    @{ tag = 'gnu-default'; toolchain = '+1.99.0-x86_64-pc-windows-gnu'; extra = @() },
    @{ tag = 'msvc-default'; toolchain = '+1.99.0-x86_64-pc-windows-msvc'; extra = @() },
    @{ tag = 'gnu-fma'; toolchain = '+1.99.0-x86_64-pc-windows-gnu'; extra = @('-C', 'target-feature=+fma') },
    @{ tag = 'msvc-fma'; toolchain = '+1.99.0-x86_64-pc-windows-msvc'; extra = @('-C', 'target-feature=+fma') }
)

$toolchain_lines = @()
foreach ($cfg in $configs) {
    $toolchain_lines += "== $($cfg.tag) =="
    $toolchain_lines += (& rustc $cfg.toolchain -vV) -join "`n"
}
$toolchain_lines | Out-File -Encoding utf8 (Join-Path $evidence 'probe-toolchains.txt')

foreach ($cfg in $configs) {
    $exe = Join-Path $out ("probe-{0}.exe" -f $cfg.tag)
    $rustcArgs = @($cfg.toolchain, '--edition', '2024', '-C', 'opt-level=0', '-C', 'debug-assertions=on') + $cfg.extra + @('-o', $exe, $source)
    & rustc @rustcArgs
    if ($LASTEXITCODE -ne 0) { throw "compile failed: $($cfg.tag)" }
    $log = Join-Path $evidence ("probe-{0}.log" -f $cfg.tag)
    "## build = {0}`r`n## rustc {1} --edition 2024 -C opt-level=0 -C debug-assertions=on {2}`r`n## source = qa_fmaf_probe.rs (own probe)" -f $cfg.tag, $cfg.toolchain, ($cfg.extra -join ' ') | Out-File -Encoding utf8 $log
    & $exe 4096 2>&1 | Out-File -Encoding utf8 -Append $log
    & $exe 1000000 2>&1 | Out-File -Encoding utf8 -Append $log
    Write-Host "$($cfg.tag): compiled and ran (4096 + 1000000)"
}
Write-Host "PROBE_DONE"
