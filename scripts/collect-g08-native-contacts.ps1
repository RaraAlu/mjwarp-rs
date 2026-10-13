#Requires -Version 7.0
[CmdletBinding()]
param(
    [string]$RepositoryRoot = (Split-Path -Parent $PSScriptRoot),
    [string]$OutputDirectory = 'target/d04-g08-native-contact/native-v1',
    [switch]$SelfTest
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = [IO.Path]::GetFullPath($RepositoryRoot)
function Sha([string]$Path) { return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant() }
function Assert-Hash([string]$Path, [string]$Expected) {
    if ((Sha $Path) -ne $Expected) { throw "Pinned file mismatch: $Path" }
}
function Run([string]$Executable, [string[]]$Arguments, [string]$Name) {
    $start = [Diagnostics.ProcessStartInfo]::new($Executable)
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $start.WorkingDirectory = $output
    foreach ($argument in $Arguments) { $start.ArgumentList.Add($argument) }
    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $start
    try {
        $watch = [Diagnostics.Stopwatch]::StartNew()
        if (-not $process.Start()) { throw "Process start failed: $Name" }
        $stdout = $process.StandardOutput.ReadToEndAsync()
        $stderr = $process.StandardError.ReadToEndAsync()
        $process.WaitForExit()
        $watch.Stop()
        $result = [ordered]@{
            name = $Name; executable = $Executable; arguments = $Arguments
            exit_code = $process.ExitCode; elapsed_ms = $watch.ElapsedMilliseconds
            stdout = $stdout.GetAwaiter().GetResult(); stderr = $stderr.GetAwaiter().GetResult()
        }
        [IO.File]::WriteAllText((Join-Path $output "$Name.json"), ($result | ConvertTo-Json -Depth 8) + "`n")
        return $result
    } finally { $process.Dispose() }
}

$fixture = Join-Path $root 'fixtures/collision-reference-input'
$model = Join-Path $fixture 'plane-sphere.mjb'
$statesPath = Join-Path $fixture 'states-f32.json'
$inputReceipt = Join-Path $fixture 'native-input-receipt.json'
$source = Join-Path $fixture 'native-contacts.cpp'
Assert-Hash $model '80b7328563e91fcd39ae04888e9e636615fd113ef01e4debea50d20a245d2123'
Assert-Hash $statesPath '3406647d311f2779af2e1b18097da48d4261e82385a7e7c5a10cad45f48552de'
Assert-Hash $inputReceipt 'c927ccf5cdaf63407434c4fcb807b01ce067c29b74b22de0fd53b56aa75c840b'
$packageRoot = Join-Path $root 'target/toolchains/mujoco-3.12.0'
$archive = Join-Path $packageRoot 'mujoco-3.12.0-windows-x86_64.zip'
$package = Join-Path $packageRoot 'package'
$library = Join-Path $package 'bin/mujoco.dll'
Assert-Hash $archive 'ffe071c2747dd9513a1c59e7d2428bb678d887f9edec9eb9674b4288a248a8e9'
Assert-Hash $library '79b61d22b4d230a00bd31930fc6943f8ea8c6174a1bf4a6bf90f2459dbf8df1b'
# 核对编译所用头文件；原生布局必须匹配冻结库。
Add-Type -AssemblyName System.IO.Compression.FileSystem
$zip = [IO.Compression.ZipFile]::OpenRead($archive)
$headers = [ordered]@{}
try {
    foreach ($entry in $zip.Entries) {
        $name = $entry.FullName.Replace('\', '/')
        if ($name -notmatch '^include/mujoco/[^/]+\.h$') { continue }
        $stream = $entry.Open()
        try { $hash = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($stream)).ToLowerInvariant() }
        finally { $stream.Dispose() }
        Assert-Hash (Join-Path $package $name) $hash
        $headers[$name] = $hash
    }
} finally { $zip.Dispose() }
if ($headers.Count -eq 0) { throw 'No verified native headers' }

$states = @(Get-Content -LiteralPath $statesPath -Raw | ConvertFrom-Json)
if ($states.Count -ne 7) { throw 'State coverage mismatch' }
for ($i = 0; $i -lt 7; $i++) {
    $state = $states[$i]
    if ($state.id -cne "z$i" -or $state.qpos_bits.Count -ne 7 -or $state.qvel_bits.Count -ne 6) { throw 'State identity/width mismatch' }
    foreach ($bits in @($state.qpos_bits) + @($state.qvel_bits)) {
        if (-not [single]::IsFinite([BitConverter]::UInt32BitsToSingle([uint32]$bits))) { throw 'Nonfinite input bits' }
    }
}
$targetBase = [IO.Path]::GetFullPath((Join-Path $root 'target')).TrimEnd([char[]]@('/', '\')) + [IO.Path]::DirectorySeparatorChar
$output = [IO.Path]::GetFullPath((Join-Path $root $OutputDirectory))
if (-not $output.StartsWith($targetBase, [StringComparison]::OrdinalIgnoreCase)) { throw 'Output must remain below target' }
if (Test-Path -LiteralPath $output) { throw 'Refuse to overwrite author output' }
New-Item -ItemType Directory -Path $output | Out-Null
$bitFile = Join-Path $output 'states-bits.bin'
$writer = [IO.BinaryWriter]::new([IO.File]::Create($bitFile))
try {
    $writer.Write([Text.Encoding]::ASCII.GetBytes("G08IN01`0"))
    foreach ($count in @(7, 7, 6)) { $writer.Write([uint32]$count) }
    foreach ($state in $states) {
        foreach ($bits in @($state.qpos_bits) + @($state.qvel_bits)) { $writer.Write([uint32]$bits) }
    }
} finally { $writer.Dispose() }
if ((Get-Item -LiteralPath $bitFile).Length -ne 384) { throw 'State transport width mismatch' }

$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
$vs = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if ($LASTEXITCODE -ne 0 -or -not $vs) { throw 'MSVC BuildTools unavailable' }
$devcmd = Join-Path $vs 'Common7/Tools/VsDevCmd.bat'
# 此处仅导入工具链环境；文件操作继续使用原生接口。
$lines = & $env:ComSpec /d /c "call `"$devcmd`" -no_logo -arch=x64 -host_arch=x64 && set"
if ($LASTEXITCODE -ne 0) { throw 'MSVC environment initialization failed' }
foreach ($line in $lines) {
    $separator = $line.IndexOf('=')
    if ($separator -gt 0) { [Environment]::SetEnvironmentVariable($line.Substring(0, $separator), $line.Substring($separator + 1), 'Process') }
}
$compiler = Join-Path $env:VCToolsInstallDir 'bin/Hostx64/x64/cl.exe'
$exe = Join-Path $output 'native-contacts.exe'
$compile = Run $compiler @('/nologo', '/std:c++17', '/EHsc', '/MD', '/W4', '/WX', '/utf-8', "/I$(Join-Path $package 'include')", $source, "/Fe:$exe", "/Fo:$(Join-Path $output 'native-contacts.obj')") 'compile'
if ($compile.exit_code -ne 0) { throw "Native diagnostic compilation failed: $($compile.stdout) $($compile.stderr)" }
$contacts = Join-Path $output 'native-contacts.json'
$positive = Run $exe @($library, $model, $bitFile, $contacts) 'collect'
if ($positive.exit_code -ne 0) { throw "Native contact collection failed: $($positive.stderr)" }
$result = Get-Content -LiteralPath $contacts -Raw | ConvertFrom-Json
if ($result.schema -ne 1 -or $result.native_version -ne 3012000 -or $result.result_scalar -ne 'f64' -or
    $result.upstream_gpu -ne $false -or $result.physical_acceptance -ne $false -or $result.rows.Count -ne 7) { throw 'Diagnostic provenance/coverage mismatch' }
for ($i = 0; $i -lt 7; $i++) {
    $row = $result.rows[$i]
    if ($row.state_index -ne $i -or $row.ncon -ne $row.contacts.Count -or $row.ncon -lt 0 -or $row.ncon -gt 1) { throw 'Contact row mismatch' }
    foreach ($contact in $row.contacts) {
        foreach ($pair in @(@('dist', 1), @('pos', 3), @('frame', 9), @('geom', 2), @('includemargin', 1))) {
            $values = @($contact.($pair[0]))
            if ($values.Count -ne $pair[1]) { throw 'Contact field width mismatch' }
            foreach ($value in $values) { if (-not [double]::IsFinite([double]$value)) { throw 'Nonfinite contact output' } }
        }
    }
}
$rejections = [Collections.Generic.List[object]]::new()
if ($SelfTest) {
    foreach ($case in @('bad-header', 'nonfinite', 'read-denied')) {
        $bad = Join-Path $output "$case.bin"
        $bytes = [IO.File]::ReadAllBytes($bitFile)
        $pattern = switch ($case) {
            'bad-header' { $bytes[0] = 0; 'state header mismatch' }
            'nonfinite' { [BitConverter]::GetBytes([uint32]2143289344).CopyTo($bytes, 20); 'nonfinite state' }
            'read-denied' { 'state read failed' }
        }
        [IO.File]::WriteAllBytes($bad, $bytes)
        $badOutput = Join-Path $output "$case-output.json"
        $lock = $null
        try {
            if ($case -eq 'read-denied') { $lock = [IO.File]::Open($bad, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::None) }
            $negative = Run $exe @($library, $model, $bad, $badOutput) $case
        } finally { if ($null -ne $lock) { $lock.Dispose() } }
        if ($negative.exit_code -ne 1 -or $negative.stderr -notmatch $pattern -or (Test-Path -LiteralPath $badOutput)) { throw "Expected rejection failed: $case" }
        $rejections.Add([ordered]@{ case = $case; exit_code = $negative.exit_code; expected_error = $pattern; before_native_physics = $true })
    }
}
$receipt = [ordered]@{
    schema = 1; source_kind = 'native CPU diagnostic author'; native_version = '3.12.0'
    frozen_upstream = '71da24d956378a87a703b6e1442b13aec0c4ac29'
    state_scalar = 'f32 bits converted to native f64'; result_scalar = 'f64'
    model_sha256 = Sha $model; states_sha256 = Sha $statesPath; input_receipt_sha256 = Sha $inputReceipt
    state_transport_sha256 = Sha $bitFile; source_sha256 = Sha $source; executable_sha256 = Sha $exe
    library_sha256 = Sha $library; compiler = $compiler; compiler_sha256 = Sha $compiler; verified_headers = $headers
    contacts_sha256 = Sha $contacts; rows = 7; contacts_per_row = @($result.rows | ForEach-Object { $_.ncon })
    native_compilations = 1; native_executions = 1 + $rejections.Count; cpu_forward_calls = 7
    rejections = @($rejections.ToArray()); elapsed_ms = [ordered]@{ compile = $compile.elapsed_ms; collect = $positive.elapsed_ms }
    upstream_gpu = $false; product_backend = $false; physical_acceptance = $false; d04_complete = $false
    product_physics_changes = $false; gpu_executions = 0; python_executions = 0
}
[IO.File]::WriteAllText((Join-Path $output 'native-contact-receipt.json'), ($receipt | ConvertTo-Json -Depth 12) + "`n")
Write-Host "G08 native CPU diagnostic: 7 states, $($rejections.Count) rejections; GPU reference remains missing"
