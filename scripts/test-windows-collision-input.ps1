#Requires -Version 7.0
[CmdletBinding()]
param([string]$OutputDirectory = 'target/g08-native-capture/attempt-1')
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = Split-Path -Parent $PSScriptRoot
$root = [IO.Path]::GetFullPath($root)
$output = [IO.Path]::GetFullPath((Join-Path $root $OutputDirectory))
$target = [IO.Path]::GetFullPath((Join-Path $root 'target')).TrimEnd([IO.Path]::DirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
if (-not $output.StartsWith($target, [StringComparison]::OrdinalIgnoreCase)) { throw 'Output must remain below target' }
if (Test-Path -LiteralPath $output) { throw 'Refuse to overwrite an existing capture test run' }
New-Item -ItemType Directory -Path $output | Out-Null
function Sha([string]$Path) { (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant() }
$package = Join-Path $root 'target/toolchains/mujoco-3.12.0/package'
$library = Join-Path $package 'bin/mujoco.dll'
if ((Sha $library) -ne '79b61d22b4d230a00bd31930fc6943f8ea8c6174a1bf4a6bf90f2459dbf8df1b') { throw 'Native DLL identity mismatch' }
if ((Sha (Join-Path $root 'fixtures/collision-reference-input/plane-sphere.mjb')) -ne '80b7328563e91fcd39ae04888e9e636615fd113ef01e4debea50d20a245d2123') { throw 'Frozen MJB identity mismatch' }
# 旧桥接身份保持冻结；不调用旧满额验收工具。
$inventory = Get-Content (Join-Path $root 'fixtures/precision/g01-inputs.json') -Raw | ConvertFrom-Json
foreach ($path in @('native/model_probe.cpp', 'native/g01_fields.inc', 'include/mjwarp_native_probe.h')) {
    $expected = @($inventory.product_sources | Where-Object path -eq $path)
    $text = [IO.File]::ReadAllText((Join-Path $root $path)).Replace("`r`n", "`n")
    $actual = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($text))).ToLowerInvariant()
    if ($expected.Count -ne 1 -or $actual -ne $expected[0].sha256) { throw 'Original native ABI source changed' }
}
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
$vs = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if ($LASTEXITCODE -ne 0 -or -not $vs) { throw 'MSVC toolchain unavailable' }
$devcmd = Join-Path $vs 'Common7/Tools/VsDevCmd.bat'
$lines = & $env:ComSpec /d /c "call `"$devcmd`" -no_logo -arch=x64 -host_arch=x64 && set"
if ($LASTEXITCODE -ne 0) { throw 'MSVC environment initialization failed' }
foreach ($line in $lines) {
    $i = $line.IndexOf('=')
    if ($i -gt 0) { [Environment]::SetEnvironmentVariable($line.Substring(0, $i), $line.Substring($i + 1), 'Process') }
}
$cases = @(
    @{Name='valid'; Define=@()}, @{Name='empty'; Define=@('/DCOLLISION_COUNT=0')},
    @{Name='negative'; Define=@('/DCOLLISION_COUNT=-1')},
    @{Name='oversized'; Define=@('/DCOLLISION_COUNT=INT64_MAX')},
    @{Name='wrong-version'; Define=@('/DCOLLISION_VERSION=1234')}
)
$cases += @(0..12 | ForEach-Object { @{Name="missing-$_"; Define=@("/DCOLLISION_MISSING=$_")} })
$watch = [Diagnostics.Stopwatch]::StartNew()
foreach ($case in $cases) {
    $dll = Join-Path $output "$($case.Name).dll"
    $log = Join-Path $output "$($case.Name)-compile.log"
    & cl.exe /nologo /LD /MD /std:c++17 /utf-8 /W4 /WX "/I$(Join-Path $package 'include')" @($case.Define) `
        (Join-Path $root 'tests/native/mock_collision.cpp') "/Fe:$dll" "/Fo:$(Join-Path $output "$($case.Name).obj")" *> $log
    if ($LASTEXITCODE -ne 0) { Get-Content $log; throw "Reader double compilation failed: $($case.Name)" }
}
$watch.Stop()
$compileMs = $watch.ElapsedMilliseconds
$env:MJWARP_MUJOCO_ROOT = $package
$env:MJWARP_MUJOCO_DLL = $library
$env:MJWARP_COLLISION_MOCKS = $output
$env:MJWARP_COLLISION_LIFETIME_LOG = Join-Path $output 'lifetime.log'
$args = @('+1.99.0-x86_64-pc-windows-msvc', 'test', '--locked', '--features', 'native-model-probe', '--test', 'native_collision_input', '--', '--ignored', '--test-threads=1', '--nocapture')
$watch.Restart()
Push-Location $root
try {
    & cargo @args *> (Join-Path $output 'tests.log')
    $exit = $LASTEXITCODE
} finally { Pop-Location }
$watch.Stop()
$testLog = [IO.File]::ReadAllText((Join-Path $output 'tests.log'))
Write-Host $testLog
if ($exit -ne 0 -or $testLog -notmatch 'test result: ok\. 4 passed; 0 failed; 0 ignored;') { throw 'New native capture tests failed' }
$receipt = [ordered]@{
    schema=1; command='cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features native-model-probe --test native_collision_input -- --ignored --test-threads=1 --nocapture'
    passed=4; failed=0; ignored=0; mock_compilations=$cases.Count
    mock_compile_ms=$compileMs; cargo_test_wall_ms=$watch.ElapsedMilliseconds
    test_log_sha256=Sha (Join-Path $output 'tests.log'); lifetime_log_sha256=Sha $env:MJWARP_COLLISION_LIFETIME_LOG
    native_dll_sha256=Sha $library; cpu_physics_calls=0; gpu_sessions=0; python_executions=0
    old_exhausted_target_calls=0; full_p2_accepted=$false; frozen_upstream_gpu_reference_available=$false
}
[IO.File]::WriteAllText((Join-Path $output 'receipt.json'), ($receipt | ConvertTo-Json -Depth 5) + "`n")
Write-Host "New native capture: 4 passed; 18 isolated doubles; no physics/GPU"
