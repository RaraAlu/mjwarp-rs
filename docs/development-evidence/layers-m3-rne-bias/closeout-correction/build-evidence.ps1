$ErrorActionPreference = 'Stop'
$originalTask = '2026-10-19-layers-m3-rne-bias'
$originalCommit = 'd735b1bfcea2a60668421c197e96a56947c111ec'
$root = 'docs/development-evidence/layers-m3-rne-bias/closeout-correction'
$archive = "$root/archive"
$runtime = "target/normify-evidence/$originalTask"
$oldReceiptPath = 'docs/development-evidence/changes/2026-10-19-layers-m3-rne-bias.json'
$frozenPlanPath = 'docs/development-evidence/layers-m3-rne-bias/formal-run/records/execution-plan.json'
$currentPlanPath = 'docs/development-evidence/layers-m3-rne-bias/execution-plan.json'
$sourceMapPath = 'docs/development-evidence/layers-m3-rne-bias/source-map.json'
$frozenManifestPath = 'docs/development-evidence/layers-m3-rne-bias/formal-run/manifest.json'
$clippyResultPath = 'docs/development-evidence/layers-m3-rne-bias/formal-run/runs/clippy-first.run.json'

function Copy-Exact([string]$Source, [string]$Destination) {
    if (-not (Test-Path -LiteralPath $Source -PathType Leaf)) { throw "缺少原件：$Source" }
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

function Copy-GitBlob([string]$Commit, [string]$Path, [string]$Destination) {
    $parent = Split-Path -Parent $Destination
    if (-not (Test-Path -LiteralPath $parent)) { New-Item -ItemType Directory -Path $parent | Out-Null }
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName = 'git'
    $info.UseShellExecute = $false
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    $info.ArgumentList.Add('cat-file')
    $info.ArgumentList.Add('blob')
    $info.ArgumentList.Add("${Commit}:$Path")
    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $info
    $stream = [IO.FileStream]::new(
        (Join-Path (Get-Location) $Destination),
        [IO.FileMode]::CreateNew,
        [IO.FileAccess]::Write,
        [IO.FileShare]::Read
    )
    try {
        if (-not $process.Start()) { throw '无法启动git cat-file' }
        $process.StandardOutput.BaseStream.CopyTo($stream)
        $process.WaitForExit()
        $errorText = $process.StandardError.ReadToEnd()
        if ($process.ExitCode -ne 0) { throw "git cat-file失败：$errorText" }
    }
    finally {
        $stream.Dispose()
        $process.Dispose()
    }
}

function Git-Blob([string]$Revision, [string]$Path) {
    $value = @(& git rev-parse "${Revision}:$Path")
    if ($LASTEXITCODE -ne 0 -or $value.Count -ne 1 -or $value[0] -notmatch '^[0-9a-f]{40}$') {
        throw "无法解析Git对象：${Revision}:$Path"
    }
    $value[0]
}

if (Test-Path -LiteralPath $archive) { throw "归档目录已存在：$archive" }
New-Item -ItemType Directory -Path $archive | Out-Null
Copy-GitBlob $originalCommit $oldReceiptPath "$archive/original-receipt.json"

$runs = @(
    [ordered]@{id='delivery-stage';directory='check-14-delivery-stage'},
    [ordered]@{id='staged-whitespace';directory='check-15-staged-whitespace'},
    [ordered]@{id='delivery-staged';directory='check-16-delivery-staged'},
    [ordered]@{id='delivery-postcommit';directory='check-17-delivery-postcommit'},
    [ordered]@{id='delivery-recorded';directory='check-18-delivery-recorded'},
    [ordered]@{id='delivery-final';directory='check-19-delivery-final'}
)
foreach ($run in $runs) {
    $destination = "$archive/$($run.id)"
    Copy-Exact "$runtime/registrations/$($run.id).json" "$destination/registration.json"
    foreach ($name in @('started.json', 'result.json', 'stdout.log', 'stderr.log')) {
        Copy-Exact "$runtime/$($run.directory)/$name" "$destination/$name"
    }
}

$clippy = Get-Content -LiteralPath $clippyResultPath -Raw | ConvertFrom-Json
if ($clippy.exit_code -ne 101) { throw 'Clippy原始退出码不是101' }
$appended = @('delivery-stage', 'staged-whitespace', 'delivery-staged', 'delivery-postcommit', 'delivery-recorded', 'delivery-final')
$correction = [ordered]@{
    schema = 1
    kind = 'authoritative-delivery-erratum'
    original_task = $originalTask
    original_commit = $originalCommit
    original_receipt = [ordered]@{
        path = $oldReceiptPath
        blob = Git-Blob $originalCommit $oldReceiptPath
        sha256 = (Get-FileHash -LiteralPath "$archive/original-receipt.json" -Algorithm SHA256).Hash.ToLowerInvariant()
        preserved_value = 1
        policy = 'preserve committed bytes and append this erratum'
    }
    corrected_field = [ordered]@{
        json_path = 'preserved_failures[gate=clippy].exit_code'
        authoritative_value = 101
        original_result_path = $clippyResultPath
        original_result_blob = Git-Blob $originalCommit $clippyResultPath
        original_result_sha256 = (Get-FileHash -LiteralPath $clippyResultPath -Algorithm SHA256).Hash.ToLowerInvariant()
    }
    plans = [ordered]@{
        frozen = [ordered]@{
            path = $frozenPlanPath
            blob = Git-Blob $originalCommit $frozenPlanPath
            sha256 = (Get-FileHash -LiteralPath $frozenPlanPath -Algorithm SHA256).Hash.ToLowerInvariant()
            generated_stage = 'formal evidence freeze before closeout commands were appended'
        }
        closeout = [ordered]@{
            path = $currentPlanPath
            blob = Git-Blob $originalCommit $currentPlanPath
            sha256 = (Get-FileHash -LiteralPath $currentPlanPath -Algorithm SHA256).Hash.ToLowerInvariant()
            generated_stage = 'delivery closeout registration before the original commit'
            appended_commands = $appended
        }
        policy = 'preserve both generations; the old source-map applies only to the frozen snapshot'
    }
    source_map = [ordered]@{
        path = $sourceMapPath
        blob = Git-Blob $originalCommit $sourceMapPath
        scope = 'records/execution-plan.json maps the frozen snapshot only'
    }
    frozen_manifest = [ordered]@{
        path = $frozenManifestPath
        blob = Git-Blob $originalCommit $frozenManifestPath
        sha256 = (Get-FileHash -LiteralPath $frozenManifestPath -Algorithm SHA256).Hash.ToLowerInvariant()
        policy = 'preserved without replacement'
    }
    archived_closeout_runs = @($runs | ForEach-Object id)
    product_tests_rerun = $false
    gpu_rerun = $false
}
$correctionJson = ($correction | ConvertTo-Json -Depth 10).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path $root 'correction.json'), $correctionJson, [Text.UTF8Encoding]::new($false))

$archiveRoot = (Resolve-Path -LiteralPath $archive).Path
$members = @(Get-ChildItem -LiteralPath $archive -Recurse -File | Sort-Object FullName | ForEach-Object {
    [ordered]@{
        path = $_.FullName.Substring($archiveRoot.Length + 1).Replace('\', '/')
        bytes = $_.Length
        sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    }
})
$manifest = [ordered]@{
    schema = 1
    root = $archive
    self_excluded = $true
    member_count = $members.Count
    members = $members
}
$manifestJson = ($manifest | ConvertTo-Json -Depth 6).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path $archive 'manifest.json'), $manifestJson, [Text.UTF8Encoding]::new($false))
[ordered]@{
    schema = 1
    original_commit = $originalCommit
    archived_members = $members.Count
    plan_generations = 2
    corrected_clippy_exit_code = 101
    result = 'built'
} | ConvertTo-Json
