$ErrorActionPreference = 'Stop'
$task = '2026-10-19-layers-m3-rigid-mass-solve'
$evidence = 'docs/development-evidence/layers-m3-rigid-mass-solve'
$runtime = "target/normify-evidence/$task"
$source = 'src/physics/mass_solve.rs'
$currentSource = "$evidence/frozen-v3/current-mass-solve.rs"
$receiptPath = "docs/development-evidence/changes/$task.json"

function Copy-New([string]$Source, [string]$Destination) {
    if (-not (Test-Path -LiteralPath $Source -PathType Leaf)) { throw "缺少交付原件：$Source" }
    $parent = Split-Path -Parent $Destination
    if (-not (Test-Path -LiteralPath $parent)) { New-Item -ItemType Directory -Path $parent | Out-Null }
    $bytes = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $Source))
    $stream = [IO.FileStream]::new((Join-Path (Get-Location) $Destination), [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::Read)
    try { $stream.Write($bytes, 0, $bytes.Length) } finally { $stream.Dispose() }
}

$postRoot = "$evidence/post-freeze-check-v3"
foreach ($item in @(
    @{source="$runtime/check-13-formal-evidence-retry-02/result.json";destination="$postRoot/formal-evidence.run.json"},
    @{source="$runtime/check-13-formal-evidence-retry-02/stdout.log";destination="$postRoot/formal-evidence.stdout.log"},
    @{source="$runtime/check-13-formal-evidence-retry-02/stderr.log";destination="$postRoot/formal-evidence.stderr.log"}
)) { Copy-New $item.source $item.destination }
$postMembers = @(Get-ChildItem -LiteralPath $postRoot -Recurse -File | Where-Object Name -ne 'manifest.json' | Sort-Object FullName | ForEach-Object {
    [ordered]@{
        path=[IO.Path]::GetRelativePath((Resolve-Path -LiteralPath $postRoot), $_.FullName).Replace('\', '/')
        bytes=$_.Length
        sha256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    }
})
$postManifest = [ordered]@{schema=1;root=$postRoot;self_excluded=$true;member_count=$postMembers.Count;members=$postMembers}
$postJson = ($postManifest | ConvertTo-Json -Depth 6).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path $postRoot 'manifest.json'), $postJson, [Text.UTF8Encoding]::new($false))

if ((Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash -cne (Get-FileHash -LiteralPath $currentSource -Algorithm SHA256).Hash) {
    throw '当前源码与冻结原件不一致'
}
& git add -- $source $evidence
if ($LASTEXITCODE -ne 0) { throw "git add退出码：$LASTEXITCODE" }
$sourceBlob = @(& git rev-parse ":$source")[0].Trim()
$currentBlob = @(& git rev-parse ":$currentSource")[0].Trim()
if ($LASTEXITCODE -ne 0 -or $sourceBlob -cne $currentBlob) { throw '暂存源码与冻结原件不一致' }
$staged = @(& git diff --cached --name-only --diff-filter=ACDMRTUXB | Where-Object { $_ -ne $receiptPath })
if ($LASTEXITCODE -ne 0 -or $staged.Count -eq 0) { throw '暂存区读取失败或为空' }
foreach ($path in $staged) {
    if ($path -ne $source -and -not $path.StartsWith("$evidence/")) { throw "暂存范围外文件：$path" }
}
$files = @($staged | Sort-Object | ForEach-Object {
    $blob = @(& git rev-parse ":$_")[0]
    if ($LASTEXITCODE -ne 0 -or $blob -notmatch '^[0-9a-f]{40}$') { throw "对象解析失败：$_" }
    [ordered]@{path=$_;blob=$blob}
})
$inventory = Get-Content -LiteralPath "$evidence/unmigrated-launch-inventory-v5.json" -Raw | ConvertFrom-Json
$formal = Get-Content -LiteralPath "$evidence/post-freeze-check/result.json" -Raw | ConvertFrom-Json
$receipt = [ordered]@{
    schema=1
    kind='delivery'
    task_id=$task
    milestones=[ordered]@{implementation='complete';validation='passed';delivery_record='pending_commit'}
    files=$files
    validation=@(
        [ordered]@{command='cargo test --locked --lib physics::mass_solve::tests::rigid_mass_solve_plan -- --test-threads=1';result='pass: 2 passed';evidence="$evidence/formal-run-v3/runs/host-wrapper-current.run.json"},
        [ordered]@{command='cargo fmt --check';result='pass';evidence="$evidence/formal-run-v3/runs/format-retry-03.run.json"},
        [ordered]@{command='cargo clippy --locked --all-targets --features cuda-probe -- -D warnings';result='pass';evidence="$evidence/formal-run-v3/runs/clippy-all-targets-retry-03.run.json"},
        [ordered]@{command='cargo test --locked --features cuda-probe --lib physics::mass_solve::tests::runs_rigid_mass_solve_wrapper_paths_and_recovers -- --ignored --exact --test-threads=1';result='pass on local NVIDIA GeForce RTX 4070 Ti SUPER: 1 passed, 257 worlds';evidence="$evidence/formal-run-v3/runs/gpu-wrapper-captured-retry-01.run.json"},
        [ordered]@{command='pwsh -NoProfile -File docs/development-evidence/layers-m3-rigid-mass-solve/check-evidence.ps1';result='pass: authoritative v5 inventory and frozen PTX registry';evidence="$evidence/formal-run-v3/runs/evidence-retry-04.run.json"},
        [ordered]@{command='pwsh -NoProfile -File docs/development-evidence/layers-m3-rigid-mass-solve/check-formal-evidence.ps1';result="pass: $($formal.verified_members) members";evidence="$evidence/post-freeze-check/formal-evidence.run.json"}
    )
    preserved_findings=@(
        [ordered]@{finding='the initial two-world GPU run preceded three supervisor fixes and has no independent Rust snapshot';authoritative_run='gpu-wrapper-captured-retry-01';historical_run='gpu-wrapper-captured'},
        [ordered]@{finding='inventory v3 retained one null-entry raw mass_solve row';historical="$evidence/unmigrated-launch-inventory-v3.json";corrected="$evidence/unmigrated-launch-inventory-v5.json";historical_preserved=$true},
        [ordered]@{finding='one host selector passed with zero tests';historical_run='host-layout-boundary';authoritative_run='host-wrapper-current'}
    )
    remaining=@(
        "$($inventory.counts.unmigrated_actual_entry_kinds) actual CUDA entry kinds remain unmigrated",
        "$($inventory.counts.unmigrated_low_level_call_sites) product low-level call sites remain unmigrated",
        'historical flg_acc=false still has 1042 failed blocks',
        'NP01 through NP04 remain unaccepted',
        'performance remains unaccepted',
        'release and Linux were not run',
        'task GPU memory usage remains unknown',
        'parent M3 remains incomplete',
        'rigid_kinematics is only the read-only next candidate and was not started'
    )
}
$json = ($receipt | ConvertTo-Json -Depth 12).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) $receiptPath), $json, [Text.UTF8Encoding]::new($false))
& git add -- $receiptPath
if ($LASTEXITCODE -ne 0) { throw "回执暂存失败：$LASTEXITCODE" }
[ordered]@{schema=1;task=$task;covered_files=$files.Count;receipt=$receiptPath;result='staged'} | ConvertTo-Json
