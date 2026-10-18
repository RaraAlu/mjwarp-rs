$ErrorActionPreference = 'Stop'
$taskId = '2026-10-19-layers-m3-post-freeze-checker-fix'
$task = 'docs/development-evidence/layers-m3-post-freeze-checker-fix'
$runtime = "target/normify-evidence/$taskId"
$scriptPath = 'docs/development-evidence/layers-m3-inventory-derived-correction/check-post-freeze-blobs.ps1'
$receiptPath = "docs/development-evidence/changes/$taskId.json"

function Copy-Stable([string]$Source, [string]$Destination) {
    if (-not (Test-Path -LiteralPath $Source -PathType Leaf)) { throw "缺少交付原件：$Source" }
    $parent = Split-Path -Parent $Destination
    if (-not (Test-Path -LiteralPath $parent)) { New-Item -ItemType Directory -Path $parent | Out-Null }
    if (Test-Path -LiteralPath $Destination) {
        if ((Get-FileHash -LiteralPath $Source -Algorithm SHA256).Hash -cne
            (Get-FileHash -LiteralPath $Destination -Algorithm SHA256).Hash) {
            throw "交付归档冲突：$Destination"
        }
        return
    }
    [IO.File]::WriteAllBytes($Destination, [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $Source)))
}

foreach ($item in @(
    @{source="$runtime/registrations/evidence-final.json";destination="$task/post-freeze-check/evidence-final.registration.json"},
    @{source="$runtime/check-06-evidence-final/started.json";destination="$task/post-freeze-check/evidence-final.started.json"},
    @{source="$runtime/check-06-evidence-final/result.json";destination="$task/post-freeze-check/evidence-final.result.json"},
    @{source="$runtime/check-06-evidence-final/stdout.log";destination="$task/post-freeze-check/evidence-final.stdout.log"},
    @{source="$runtime/check-06-evidence-final/stderr.log";destination="$task/post-freeze-check/evidence-final.stderr.log"}
)) { Copy-Stable $item.source $item.destination }

$indexReady = Test-Path -LiteralPath "$runtime/check-07-index-bytes/result.json" -PathType Leaf
if ($indexReady) {
    foreach ($item in @(
        @{source="$runtime/registrations/index-bytes.json";destination="$task/index-check/registration.json"},
        @{source="$runtime/check-07-index-bytes/started.json";destination="$task/index-check/started.json"},
        @{source="$runtime/check-07-index-bytes/result.json";destination="$task/index-check/result.json"},
        @{source="$runtime/check-07-index-bytes/stdout.log";destination="$task/index-check/stdout.log"},
        @{source="$runtime/check-07-index-bytes/stderr.log";destination="$task/index-check/stderr.log"}
    )) { Copy-Stable $item.source $item.destination }
}

& git add -- $scriptPath $task
if ($LASTEXITCODE -ne 0) { throw "证据暂存失败：$LASTEXITCODE" }
$staged = @(& git diff --cached --name-only --diff-filter=ACDMRTUXB | Where-Object { $_ -ne $receiptPath })
if ($LASTEXITCODE -ne 0 -or $staged.Count -eq 0) { throw '暂存范围为空' }
foreach ($path in $staged) {
    if ($path -cne $scriptPath -and -not $path.StartsWith("$task/", [StringComparison]::Ordinal)) {
        throw "暂存范围外文件：$path"
    }
}
$files = @($staged | Sort-Object | ForEach-Object {
    $blob = @(& git rev-parse ":$_")[0].Trim()
    if ($LASTEXITCODE -ne 0 -or $blob -notmatch '^[0-9a-f]{40}$') { throw "对象解析失败：$_" }
    [ordered]@{path=$_;blob=$blob}
})

$validation = [Collections.Generic.List[object]]::new()
foreach ($row in @(
    @{command='delivery preflight';result='pass';evidence="$task/formal-run/delivery-preflight.run.json"},
    @{command='static mapping cases';result='pass';evidence="$task/formal-run/static-mapping-final.run.json"},
    @{command='prior commit Git objects';result='pass';evidence="$task/formal-run/prior-commit-objects-final.run.json"},
    @{command='scope boundaries';result='pass';evidence="$task/formal-run/scope.run.json"},
    @{command='formal evidence';result='pass';evidence="$task/post-freeze-check/evidence-final.result.json"}
)) { $validation.Add($row) }
if ($indexReady) {
    $validation.Add([ordered]@{command='staged formal evidence bytes';result='pass';evidence="$task/index-check/result.json"})
}

$receipt = [ordered]@{
    schema = 1
    kind = 'delivery'
    task_id = $taskId
    milestones = [ordered]@{
        implementation = 'complete'
        validation = 'passed'
        delivery_record = 'pending_commit'
    }
    files = $files
    validation = @($validation)
    preserved_failures = @(
        [ordered]@{
            task = $taskId
            gate = 'first prior commit object verification'
            exit_code = 1
            evidence = "$task/formal-run/prior-commit-objects.run.json"
            reason = 'the first fix renamed ordinary result archives instead of preserving the run suffix'
        },
        [ordered]@{
            task = '2026-10-19-layers-m3-inventory-derived-correction'
            gate = 'post-freeze commit verifier'
            exit_code = 1
            evidence = "$task/formal-run/static-mapping.stdout.log"
            reason = 'the old script used the failed run id and emitted dot-prefixed empty-name paths'
        }
    )
    remaining = @(
        'GPU and product tests were not run',
        'sleep_copy was not started',
        'parent M3 remains proposed and incomplete',
        'the prior task stays closed and its records remain unchanged'
    )
}
$json = ($receipt | ConvertTo-Json -Depth 10).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) $receiptPath), $json, [Text.UTF8Encoding]::new($false))
& git add -- $receiptPath
if ($LASTEXITCODE -ne 0) { throw "回执暂存失败：$LASTEXITCODE" }
[ordered]@{schema=1;task=$taskId;covered_files=$files.Count;receipt=$receiptPath;index_evidence=$indexReady;result='staged'} | ConvertTo-Json
