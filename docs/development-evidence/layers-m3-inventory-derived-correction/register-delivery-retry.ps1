param(
    [Parameter(Mandatory = $true)]
    [string]$Id
)

$ErrorActionPreference = 'Stop'
$taskId = '2026-10-19-layers-m3-inventory-derived-correction'
$planPath = 'docs/development-evidence/layers-m3-inventory-derived-correction/post-freeze-delivery-plan.json'
$runtimeRoot = "target/normify-evidence/$taskId"
$registryRoot = "$runtimeRoot/registrations"
$plan = Get-Content -LiteralPath $planPath -Raw | ConvertFrom-Json
$entries = @($plan.registered_validations | Where-Object id -CEQ $Id)
if ($entries.Count -ne 1) { throw "交付重试计划不唯一：$Id" }
$entry = $entries[0]
if (-not $entry.registered_before_execution) { throw '交付重试未声明事前登记' }
if (Test-Path -LiteralPath $entry.output_root) { throw "输出目录已存在：$($entry.output_root)" }
New-Item -ItemType Directory -Force -Path $registryRoot | Out-Null
$registrationPath = "$registryRoot/$Id.json"
if (Test-Path -LiteralPath $registrationPath) { throw "运行登记已存在：$registrationPath" }
$arguments = @($entry.arguments | ForEach-Object {
    if ($_ -ceq '{HEAD}') {
        $head = @(& git rev-parse HEAD)
        if ($LASTEXITCODE -ne 0 -or $head.Count -ne 1 -or $head[0] -notmatch '^[0-9a-f]{40}$') {
            throw '无法解析完整HEAD提交号'
        }
        $head[0]
    }
    else { $_ }
})
$record = [ordered]@{
    schema = 1
    task_id = $taskId
    id = $Id
    registered_before_execution = $true
    registered_at_utc = [DateTime]::UtcNow.ToString('o')
    purpose = $entry.purpose
    executable = $entry.executable
    arguments = $arguments
    command = (@($entry.executable) + $arguments) -join ' '
    environment = [ordered]@{}
    output_root = $entry.output_root
    stdout = "$($entry.output_root)/stdout.log"
    stderr = "$($entry.output_root)/stderr.log"
    capture_directory = $null
    requires_gpu_preflight = $false
    gpu_preflight_result = $null
}
$json = ($record | ConvertTo-Json -Depth 10).Replace("`r`n", "`n") + "`n"
$bytes = [Text.UTF8Encoding]::new($false).GetBytes($json)
$stream = [IO.File]::Open($registrationPath, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::Read)
try { $stream.Write($bytes, 0, $bytes.Length) } finally { $stream.Dispose() }
[ordered]@{
    schema = 1
    registration = $registrationPath
    registration_sha256 = (Get-FileHash -LiteralPath $registrationPath -Algorithm SHA256).Hash.ToLowerInvariant()
    output_root = $entry.output_root
    result = 'registered'
} | ConvertTo-Json
