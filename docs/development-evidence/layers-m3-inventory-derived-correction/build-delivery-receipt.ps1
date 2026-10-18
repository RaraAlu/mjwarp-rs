$ErrorActionPreference = 'Stop'
$task = '2026-10-19-layers-m3-inventory-derived-correction'
$evidence = 'docs/development-evidence/layers-m3-inventory-derived-correction'
$runtime = "target/normify-evidence/$task"
$receiptPath = "docs/development-evidence/changes/$task.json"

function Copy-New([string]$Source, [string]$Destination) {
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

$postRoot = "$evidence/post-freeze-check"
foreach ($item in @(
    @{source="$runtime/check-12-evidence-final/result.json";destination="$postRoot/evidence-final.run.json"},
    @{source="$runtime/check-12-evidence-final/stdout.log";destination="$postRoot/evidence-final.stdout.log"},
    @{source="$runtime/check-12-evidence-final/stderr.log";destination="$postRoot/evidence-final.stderr.log"}
)) {
    Copy-New $item.source $item.destination
}
foreach ($set in @(
    @{id='delivery-preflight';name='preflight-first'},
    @{id='delivery-preflight-retry-01';name='preflight-final'},
    @{id='receipt';name='receipt-first'},
    @{id='receipt-retry-01';name='receipt-retry-first'},
    @{id='post-freeze-index';name='post-freeze-index-first'}
)) {
    $registration = "$runtime/registrations/$($set.id).json"
    $record = Get-Content -LiteralPath $registration -Raw | ConvertFrom-Json
    $runRoot = [string]$record.output_root
    foreach ($item in @(
        @{source=$registration;destination="$postRoot/delivery/$($set.name).registration.json"},
        @{source="$runRoot/started.json";destination="$postRoot/delivery/$($set.name).started.json"},
        @{source="$runRoot/result.json";destination="$postRoot/delivery/$($set.name).run.json"},
        @{source="$runRoot/stdout.log";destination="$postRoot/delivery/$($set.name).stdout.log"},
        @{source="$runRoot/stderr.log";destination="$postRoot/delivery/$($set.name).stderr.log"}
    )) {
        Copy-New $item.source $item.destination
    }
}
$indexReady = Test-Path -LiteralPath "$runtime/check-13-index-bytes/result.json" -PathType Leaf
if ($indexReady) {
    $registration = "$runtime/registrations/index-bytes.json"
    foreach ($item in @(
        @{source=$registration;destination="$evidence/index-check/registration.json"},
        @{source="$runtime/check-13-index-bytes/started.json";destination="$evidence/index-check/started.json"},
        @{source="$runtime/check-13-index-bytes/result.json";destination="$evidence/index-check/result.json"},
        @{source="$runtime/check-13-index-bytes/stdout.log";destination="$evidence/index-check/stdout.log"},
        @{source="$runtime/check-13-index-bytes/stderr.log";destination="$evidence/index-check/stderr.log"}
    )) {
        Copy-New $item.source $item.destination
    }
}
$postIndexReady = Test-Path -LiteralPath "$runtime/check-13-post-freeze-index-retry-01/result.json" -PathType Leaf
if ($postIndexReady) {
    $registration = "$runtime/registrations/post-freeze-index-retry-01.json"
    foreach ($item in @(
        @{source=$registration;destination="$postRoot/index-extra/registration.json"},
        @{source="$runtime/check-13-post-freeze-index-retry-01/started.json";destination="$postRoot/index-extra/started.json"},
        @{source="$runtime/check-13-post-freeze-index-retry-01/result.json";destination="$postRoot/index-extra/result.json"},
        @{source="$runtime/check-13-post-freeze-index-retry-01/stdout.log";destination="$postRoot/index-extra/stdout.log"},
        @{source="$runtime/check-13-post-freeze-index-retry-01/stderr.log";destination="$postRoot/index-extra/stderr.log"}
    )) { Copy-New $item.source $item.destination }
}
$postMembers = @(Get-ChildItem -LiteralPath $postRoot -Recurse -File |
    Where-Object Name -ne 'manifest.json' | Sort-Object FullName | ForEach-Object {
        [ordered]@{
            path = $_.FullName.Substring((Resolve-Path -LiteralPath $postRoot).Path.Length + 1).Replace('\', '/')
            bytes = $_.Length
            sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        }
    })
$postManifest = [ordered]@{schema=1;root=$postRoot;self_excluded=$true;member_count=$postMembers.Count;members=$postMembers}
$postJson = ($postManifest | ConvertTo-Json -Depth 6).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path $postRoot 'manifest.json'), $postJson, [Text.UTF8Encoding]::new($false))

& git add -- $evidence
if ($LASTEXITCODE -ne 0) { throw "git add退出码：$LASTEXITCODE" }
$staged = @(& git diff --cached --name-only --diff-filter=ACDMRTUXB | Where-Object { $_ -ne $receiptPath })
if ($LASTEXITCODE -ne 0) { throw "git diff退出码：$LASTEXITCODE" }
if ($staged.Count -eq 0) { throw '暂存区为空' }
foreach ($path in $staged) {
    if (-not $path.StartsWith("$evidence/")) { throw "暂存范围外文件：$path" }
}
$files = @($staged | Sort-Object | ForEach-Object {
    $blob = @(& git rev-parse ":$_")[0]
    if ($LASTEXITCODE -ne 0 -or $blob -notmatch '^[0-9a-f]{40}$') { throw "对象解析失败：$_" }
    [ordered]@{path=$_;blob=$blob}
})

$validation = [Collections.Generic.List[object]]::new()
foreach ($row in @(
    @{command='first build corrected inventory';result='failed and preserved';evidence="$evidence/formal-run/build-first.run.json"},
    @{command='preserve failed corrected inventory';result='pass';evidence="$evidence/formal-run/preserve-build-failure.run.json"},
    @{command='verify private summary repair';result='pass';evidence="$evidence/formal-run/repair-private-sum.run.json"},
    @{command='check corrected inventory';result='pass';evidence="$evidence/formal-run/normal.run.json"},
    @{command='duplicate entry negative case';result='pass: rejected';evidence="$evidence/formal-run/negative-duplicate.run.json"},
    @{command='unknown classification negative case';result='pass: rejected';evidence="$evidence/formal-run/negative-unknown.run.json"},
    @{command='derived drift negative case';result='pass: rejected';evidence="$evidence/formal-run/negative-drift.run.json"},
    @{command='empty candidate negative case';result='pass: rejected';evidence="$evidence/formal-run/negative-empty.run.json"},
    @{command='cross classification negative case';result='pass: rejected';evidence="$evidence/formal-run/negative-cross-class.run.json"},
    @{command='cross set negative case';result='pass: rejected';evidence="$evidence/formal-run/negative-cross-set.run.json"},
    @{command='private summary drift negative case';result='pass: rejected';evidence="$evidence/formal-run/negative-private-summary.run.json"},
    @{command='check source and evidence boundaries';result='pass';evidence="$evidence/formal-run/evidence.run.json"},
    @{command='check formal evidence';result='pass';evidence="$evidence/post-freeze-check/result.json"},
    @{command='delivery preflight after receipt binding';result='pass';evidence="$evidence/post-freeze-check/delivery/preflight-final.run.json"}
)) { $validation.Add($row) }
if ($indexReady) {
    $validation.Add([ordered]@{command='check staged evidence bytes';result='pass';evidence="$evidence/index-check/result.json"})
}
if ($postIndexReady) {
    $validation.Add([ordered]@{command='check staged post-freeze bytes';result='pass';evidence="$evidence/post-freeze-check/index-extra/result.json"})
}

$inventory = Get-Content -LiteralPath "$evidence/corrected-inventory.json" -Raw | ConvertFrom-Json
$receipt = [ordered]@{
    schema = 1
    kind = 'delivery'
    task_id = $task
    milestones = [ordered]@{
        implementation = 'complete'
        validation = 'passed'
        delivery_record = 'pending_commit'
    }
    process_compliance = [ordered]@{
        current_task_run_registration = 'complete'
        current_task_failed_runs_preserved = $true
        predecessor_missing_registrations = 6
        predecessor_gap_closed = $false
        parent_m3_passed = $false
    }
    files = $files
    validation = @($validation)
    preserved_failures = @(
        [ordered]@{
            gate = 'build corrected inventory'
            evidence = "$evidence/formal-run/build-first.run.json"
            exit_code = 1
            reason = 'PowerShell array property aggregation did not produce scalar totals'
        },
        [ordered]@{
            gate = 'delivery preflight'
            evidence = "$evidence/post-freeze-check/delivery/preflight-first.run.json"
            exit_code = 1
            reason = 'Normify task note did not yet bind the delivery receipt path'
        },
        [ordered]@{
            gate = 'delivery receipt'
            evidence = "$evidence/post-freeze-check/delivery/receipt-first.run.json"
            exit_code = 1
            reason = 'receipt builder used the task id instead of the evidence directory name'
        },
        [ordered]@{
            gate = 'delivery receipt scope'
            evidence = "$evidence/post-freeze-check/delivery/receipt-retry-first.run.json"
            exit_code = 1
            reason = 'misplaced evidence from the first receipt attempt remained staged'
        },
        [ordered]@{
            gate = 'post-freeze index bytes'
            evidence = "$evidence/post-freeze-check/delivery/post-freeze-index-first.run.json"
            exit_code = 1
            reason = 'PowerShell method chaining crossed a line boundary'
        }
    )
    remaining = @(
        '64 actual CUDA entry kinds remain unmigrated',
        '79 product low-level call sites remain unmigrated',
        "sleep_copy is read-only candidate $($inventory.next_candidate.entry -ceq 'sleep_copy') and no task was started",
        'GPU and product tests were not run',
        'parent M3 remains proposed and incomplete',
        'the predecessor retains six missing preregistrations'
    )
}
$json = ($receipt | ConvertTo-Json -Depth 10).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) $receiptPath), $json, [Text.UTF8Encoding]::new($false))
& git add -- $receiptPath
if ($LASTEXITCODE -ne 0) { throw "回执暂存失败：$LASTEXITCODE" }
[ordered]@{schema=1;task=$task;covered_files=$files.Count;receipt=$receiptPath;index_evidence=$indexReady;result='staged'} | ConvertTo-Json
