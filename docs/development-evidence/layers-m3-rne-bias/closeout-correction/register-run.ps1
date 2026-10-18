param(
    [Parameter(Mandatory = $true)]
    [string]$Id
)

$ErrorActionPreference = 'Stop'
$taskId = '2026-10-19-layers-m3-rne-bias-record-correction'
$planPath = 'docs/development-evidence/layers-m3-rne-bias/closeout-correction/execution-plan.json'
$runtimeRoot = "target/normify-evidence/$taskId"
$registryRoot = "$runtimeRoot/registrations"

if ($Id -notmatch '^[a-z0-9][a-z0-9-]*$') { throw "运行标识非法：$Id" }
$plan = Get-Content -LiteralPath $planPath -Raw | ConvertFrom-Json
$entries = @($plan.registered_validations | Where-Object id -CEQ $Id)
if ($entries.Count -ne 1) { throw "运行计划不唯一：$Id" }
$entry = $entries[0]
if (-not $entry.registered_before_execution) { throw "运行未声明事前登记：$Id" }
if (-not $entry.output_root.StartsWith("$runtimeRoot/", [StringComparison]::Ordinal)) {
    throw "输出目录越界：$($entry.output_root)"
}
if (Test-Path -LiteralPath $entry.output_root) { throw "输出目录已存在：$($entry.output_root)" }

New-Item -ItemType Directory -Force -Path $registryRoot | Out-Null
$registrationPath = "$registryRoot/$Id.json"
if (Test-Path -LiteralPath $registrationPath) { throw "运行登记已存在：$registrationPath" }
$record = [ordered]@{
    schema = 1
    task_id = $taskId
    id = $Id
    registered_before_execution = $true
    registered_at_utc = [DateTime]::UtcNow.ToString('o')
    purpose = $entry.purpose
    executable = $entry.executable
    arguments = @($entry.arguments)
    command = (($entry.executable, @($entry.arguments)) -join ' ')
    environment = $entry.environment
    output_root = $entry.output_root
    stdout = "$($entry.output_root)/stdout.log"
    stderr = "$($entry.output_root)/stderr.log"
}
$json = ($record | ConvertTo-Json -Depth 10).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText($registrationPath, $json, [Text.UTF8Encoding]::new($false))
[ordered]@{
    schema = 1
    registration = $registrationPath
    registration_sha256 = (Get-FileHash -LiteralPath $registrationPath -Algorithm SHA256).Hash.ToLowerInvariant()
    output_root = $entry.output_root
    result = 'registered'
} | ConvertTo-Json
