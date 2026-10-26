# QA 默认配置 clean 构建/测试（重跑，前一次 summary 缺 default 段）
$ErrorActionPreference = 'Stop'
$root = 'D:\Rust\mjwarp-rs'
Set-Location $root
$base = Join-Path $root 'target\qa-stage2'
$logDir = Join-Path $base 'logs'
New-Item -ItemType Directory -Force $logDir | Out-Null
$target = Join-Path $base 't-default'
$env:CARGO_TARGET_DIR = $target
if (Test-Path -LiteralPath $target) { Remove-Item -LiteralPath $target -Recurse -Force }
function Invoke-Capture {
    param([scriptblock]$Block)
    $prev = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try {
        $out = & $Block *>&1 | ForEach-Object { if ($_ -is [System.Management.Automation.ErrorRecord]) { $_.Exception.Message } else { [string]$_ } }
        $code = $LASTEXITCODE
    } finally {
        $ErrorActionPreference = $prev
    }
    return @{ out = @($out); code = $code }
}
@(
    "# default config header",
    "time=$(Get-Date -Format o)",
    "cargo=$( (& cargo -vV *>&1 | Out-String).Trim() )",
    "rustc=$( (& rustc -vV *>&1 | Out-String).Trim() )",
    "CARGO_TARGET_DIR=$env:CARGO_TARGET_DIR",
    "PATH=$env:PATH"
) | Set-Content -LiteralPath (Join-Path $logDir 'default2-env.txt') -Encoding utf8
$sw = [System.Diagnostics.Stopwatch]::StartNew()
$build = Invoke-Capture { cargo build --locked }
$sw.Stop()
$build.out | Set-Content -LiteralPath (Join-Path $logDir 'default2-build.log') -Encoding utf8
$exe = Join-Path $target 'debug\mjwarp-rs.exe'
$exeMiB = if (Test-Path -LiteralPath $exe) { [math]::Round((Get-Item -LiteralPath $exe).Length / 1MB, 2) } else { 0 }
Write-Host "RESULT default build=$($build.code) build_ms=$($sw.ElapsedMilliseconds) exe_mib=$exeMiB"
$sw2 = [System.Diagnostics.Stopwatch]::StartNew()
$test = Invoke-Capture { cargo test --locked }
$sw2.Stop()
$test.out | Set-Content -LiteralPath (Join-Path $logDir 'default2-test.log') -Encoding utf8
$p = 0; $f = 0; $ig = 0; $b = 0; $lib = ''
foreach ($m in [regex]::Matches(($test.out -join "`n"), 'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;')) {
    $b++; $p += [int]$m.Groups[1].Value; $f += [int]$m.Groups[2].Value; $ig += [int]$m.Groups[3].Value
}
$libLine = ($test.out | Select-Object -First 1)
$s = (Get-ChildItem -LiteralPath $target -Recurse -File | Measure-Object -Property Length -Sum).Sum
$targetMiB = [math]::Round($s / 1MB, 1)
Write-Host "RESULT default test=$($test.code) test_ms=$($sw2.ElapsedMilliseconds) passed=$p failed=$f ignored=$ig binaries=$b target_mib=$targetMiB"
Write-Host "RESULT default first_result_line=$libLine"
Write-Host 'DEFAULT DONE'
