param(
    [Parameter(Mandatory = $true)]
    [string]$Registration
)

$ErrorActionPreference = 'Stop'
$taskId = '2026-10-19-layers-m3-rigid-mass-matrix'
$runtimeRoot = "target/normify-evidence/$taskId"
$registrationRoot = "$runtimeRoot/registrations/"
if (-not $Registration.StartsWith($registrationRoot, [StringComparison]::Ordinal)) {
    throw "运行登记越界：$Registration"
}
$record = Get-Content -LiteralPath $Registration -Raw | ConvertFrom-Json
if ($record.task_id -cne $taskId -or $record.registered_before_execution -ne $true) {
    throw '运行登记无效'
}
$registrationHash = (Get-FileHash -LiteralPath $Registration -Algorithm SHA256).Hash.ToLowerInvariant()
$outputRoot = [string]$record.output_root
if (-not $outputRoot.StartsWith("$runtimeRoot/", [StringComparison]::Ordinal)) {
    throw "输出目录越界：$outputRoot"
}
if (Test-Path -LiteralPath $outputRoot) { throw "输出目录已存在：$outputRoot" }
if ($record.capture_directory -and (Test-Path -LiteralPath $record.capture_directory)) {
    throw "捕获目录已存在：$($record.capture_directory)"
}
if ($record.requires_gpu_preflight) {
    $preflight = Get-Content -LiteralPath $record.gpu_preflight_result -Raw | ConvertFrom-Json
    if ($preflight.exit_code -ne 0) { throw 'GPU预检没有通过' }
    $device = Get-Content -LiteralPath $preflight.stdout -Raw | ConvertFrom-Json
    if ($device.device -notmatch '^NVIDIA GeForce RTX 4070 Ti SUPER,' -or
        $device.cuda_root -cne 'target/toolchains/cuda-12.8.1') {
        throw 'GPU预检身份不符'
    }
}

New-Item -ItemType Directory -Path $outputRoot | Out-Null
$stdout = [string]$record.stdout
$stderr = [string]$record.stderr
$stdoutStream = [IO.File]::Open($stdout, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::Read)
$stdoutStream.Dispose()
$stderrStream = [IO.File]::Open($stderr, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::Read)
$stderrStream.Dispose()
$started = [DateTime]::UtcNow
$startedRecord = [ordered]@{
    schema = 1
    id = $record.id
    registration = $Registration
    registration_sha256 = $registrationHash
    started_at_utc = $started.ToString('o')
    stdout_precreated = Test-Path -LiteralPath $stdout
    stderr_precreated = Test-Path -LiteralPath $stderr
}
$startedJson = ($startedRecord | ConvertTo-Json).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path $outputRoot 'started.json'), $startedJson, [Text.UTF8Encoding]::new($false))

$saved = [ordered]@{}
try {
    foreach ($property in $record.environment.PSObject.Properties) {
        $name = $property.Name
        $saved[$name] = [Environment]::GetEnvironmentVariable($name, 'Process')
        $value = [string]$property.Value
        if ($name -ceq 'PATH_PREPEND') {
            $saved['PATH'] = [Environment]::GetEnvironmentVariable('PATH', 'Process')
            $separator = [IO.Path]::PathSeparator
            [Environment]::SetEnvironmentVariable('PATH', "$value$separator$($saved['PATH'])", 'Process')
        }
        else {
            [Environment]::SetEnvironmentVariable($name, $value, 'Process')
        }
    }
    $stopwatch = [Diagnostics.Stopwatch]::StartNew()
    $exitCode = 0
    try {
        $executable = [string]$record.executable
        $arguments = @($record.arguments)
        & $executable @arguments 1>> $stdout 2>> $stderr
        $exitCode = $LASTEXITCODE
        if ($null -eq $exitCode) { $exitCode = 0 }
    }
    catch {
        $_ | Out-String | Add-Content -LiteralPath $stderr -Encoding utf8
        $exitCode = 9009
    }
    $stopwatch.Stop()
}
finally {
    foreach ($name in $saved.Keys) {
        [Environment]::SetEnvironmentVariable($name, $saved[$name], 'Process')
    }
}
$ended = [DateTime]::UtcNow
if ((Get-FileHash -LiteralPath $Registration -Algorithm SHA256).Hash.ToLowerInvariant() -cne $registrationHash) {
    throw '运行登记在执行期间发生变化'
}
$result = [ordered]@{
    schema = 1
    id = $record.id
    registration = $Registration
    registration_sha256 = $registrationHash
    started_at_utc = $started.ToString('o')
    ended_at_utc = $ended.ToString('o')
    duration_ms = $stopwatch.ElapsedMilliseconds
    exit_code = $exitCode
    stdout = $stdout
    stderr = $stderr
    result = if ($exitCode -eq 0) { 'pass' } else { 'fail' }
}
$resultPath = Join-Path $outputRoot 'result.json'
$resultJson = ($result | ConvertTo-Json).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText($resultPath, $resultJson, [Text.UTF8Encoding]::new($false))
Write-Output '--- stdout ---'
Get-Content -LiteralPath $stdout
Write-Output '--- stderr ---'
Get-Content -LiteralPath $stderr
Write-Output '--- result ---'
Get-Content -LiteralPath $resultPath
exit $exitCode
