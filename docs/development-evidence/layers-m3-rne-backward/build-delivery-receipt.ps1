$ErrorActionPreference = 'Stop'
$task = '2026-10-19-layers-m3-rne-backward'
$evidence = 'docs/development-evidence/layers-m3-rne-backward'
$runtime = "target/normify-evidence/$task"
$source = 'src/physics/rne.rs'
$currentSource = "$evidence/frozen/current-rne.rs"
$receiptPath = "docs/development-evidence/changes/$task.json"

function Copy-New([string]$Source, [string]$Destination) {
    if (-not (Test-Path -LiteralPath $Source -PathType Leaf)) { throw "缺少交付原件：$Source" }
    $parent = Split-Path -Parent $Destination
    if (-not (Test-Path -LiteralPath $parent)) { New-Item -ItemType Directory -Path $parent | Out-Null }
    $bytes = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $Source))
    $stream = [IO.FileStream]::new(
        (Join-Path (Get-Location) $Destination),
        [IO.FileMode]::CreateNew,
        [IO.FileAccess]::Write,
        [IO.FileShare]::Read
    )
    try { $stream.Write($bytes, 0, $bytes.Length) } finally { $stream.Dispose() }
}

$postRoot = "$evidence/post-freeze-check"
foreach ($item in @(
    @{source="$runtime/check-13-formal-evidence/result.json";destination="$postRoot/formal-evidence.run.json"},
    @{source="$runtime/check-13-formal-evidence/stdout.log";destination="$postRoot/formal-evidence.stdout.log"},
    @{source="$runtime/check-13-formal-evidence/stderr.log";destination="$postRoot/formal-evidence.stderr.log"}
)) {
    Copy-New $item.source $item.destination
}
$postMembers = @(Get-ChildItem -LiteralPath $postRoot -Recurse -File |
    Where-Object Name -ne 'manifest.json' | Sort-Object FullName | ForEach-Object {
        [ordered]@{
            path=[IO.Path]::GetRelativePath((Resolve-Path -LiteralPath $postRoot), $_.FullName).Replace('\', '/')
            bytes=$_.Length
            sha256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        }
    })
$postManifest = [ordered]@{
    schema=1
    root=$postRoot
    self_excluded=$true
    member_count=$postMembers.Count
    members=$postMembers
}
$postJson = ($postManifest | ConvertTo-Json -Depth 6).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path $postRoot 'manifest.json'), $postJson, [Text.UTF8Encoding]::new($false))

if ((Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash -cne
    (Get-FileHash -LiteralPath $currentSource -Algorithm SHA256).Hash) {
    throw '当前源码与冻结原件不一致'
}
& git add -- $source $evidence
if ($LASTEXITCODE -ne 0) { throw "git add退出码：$LASTEXITCODE" }
$sourceBlob = @(& git rev-parse ":$source")[0].Trim()
$currentBlob = @(& git rev-parse ":$currentSource")[0].Trim()
if ($LASTEXITCODE -ne 0 -or $sourceBlob -cne $currentBlob) {
    throw '暂存源码与冻结原件不一致'
}
$staged = @(& git diff --cached --name-only --diff-filter=ACDMRTUXB | Where-Object { $_ -ne $receiptPath })
if ($LASTEXITCODE -ne 0 -or $staged.Count -eq 0) { throw '暂存区读取失败或为空' }
foreach ($path in $staged) {
    if ($path -ne $source -and -not $path.StartsWith("$evidence/")) {
        throw "暂存范围外文件：$path"
    }
}
$files = @($staged | Sort-Object | ForEach-Object {
    $blob = @(& git rev-parse ":$_")[0]
    if ($LASTEXITCODE -ne 0 -or $blob -notmatch '^[0-9a-f]{40}$') { throw "对象解析失败：$_" }
    [ordered]@{path=$_;blob=$blob}
})
$inventory = Get-Content -LiteralPath "$evidence/unmigrated-launch-inventory-v2.json" -Raw | ConvertFrom-Json
$receipt = [ordered]@{
    schema=1
    kind='delivery'
    task_id=$task
    milestones=[ordered]@{
        implementation='complete'
        validation='passed'
        delivery_record='pending_commit'
    }
    files=$files
    validation=@(
        [ordered]@{command='cargo test --locked --lib rne_backward_plan -- --test-threads=1';result='pass: 1 passed';evidence="$evidence/formal-run/runs/host-wrapper.run.json"},
        [ordered]@{command='cargo fmt --check';result='pass';evidence="$evidence/formal-run/runs/format-after-gpu-code.run.json"},
        [ordered]@{command='cargo clippy --locked --all-targets --features cuda-probe -- -D warnings';result='pass';evidence="$evidence/formal-run/runs/clippy-all-targets.run.json"},
        [ordered]@{command='cargo test --locked --features cuda-probe --lib physics::rne::tests::runs_rne_backward_wrapper_in_reverse_levels_and_recovers -- --ignored --exact --test-threads=1';result='pass on local NVIDIA GeForce RTX 4070 Ti SUPER: 1 passed, 43 worlds, 258 work items';evidence="$evidence/formal-run/runs/gpu-wrapper-captured.run.json"},
        [ordered]@{command='pwsh -NoProfile -File docs/development-evidence/layers-m3-rne-backward/check-evidence.ps1';result='pass: authoritative v2';evidence="$evidence/formal-run/runs/evidence-final.run.json"},
        [ordered]@{command='pwsh -NoProfile -File docs/development-evidence/layers-m3-rne-backward/check-formal-evidence.ps1';result='pass: 162 members';evidence="$evidence/post-freeze-check/formal-evidence.run.json"}
    )
    preserved_findings=@(
        [ordered]@{
            finding='initial inventory validated but did not apply the rne_bias overlay'
            historical="$evidence/unmigrated-launch-inventory-v1.json"
            authoritative="$evidence/unmigrated-launch-inventory-v2.json"
            correction="$evidence/inventory-correction.json"
        },
        [ordered]@{
            finding='early registration command display rendered the argument array as System.Object[]'
            authoritative_fields='executable, arguments, environment'
            correction="$evidence/registration-command-errata.json"
            historical_registrations_unchanged=$true
        }
    )
    remaining=@(
        "$($inventory.counts.unmigrated_actual_entry_kinds) actual CUDA entry kinds remain unmigrated",
        "$($inventory.counts.unmigrated_low_level_call_sites) product low-level call sites remain unmigrated",
        'general sibling atomic tolerance was not revalidated',
        'historical flg_acc=false still has 1042 failed blocks',
        'NP01 through NP04 remain unaccepted',
        'performance remains unaccepted',
        'release and Linux were not run',
        'task GPU memory usage remains unknown',
        'parent M3 remains incomplete'
    )
}
$json = ($receipt | ConvertTo-Json -Depth 12).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) $receiptPath), $json, [Text.UTF8Encoding]::new($false))
& git add -- $receiptPath
if ($LASTEXITCODE -ne 0) { throw "回执暂存失败：$LASTEXITCODE" }
[ordered]@{
    schema=1
    task=$task
    covered_files=$files.Count
    receipt=$receiptPath
    result='staged'
} | ConvertTo-Json
