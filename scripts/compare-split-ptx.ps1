# Split PTX identity: capture every product-recipe PTX at a pre-split baseline
# and at the split-closure commit, then compare byte by byte.
#
# Read-only over repository history: commits are exported with `git archive`
# (no `git worktree`, no checkout, no `.git/worktrees` pollution). Per-commit
# build products live in their own `CARGO_TARGET_DIR`; the script only writes
# under `target/split-ptx/` and the evidence directory.
#
# Capture convention (existing product capture layer, `record_physics_ptx`):
#   MJWARP_PHYSICS_PTX_DIR=<empty capture root> and every compiled f32-physics
#   recipe is written as `<entry>-<ptxSha256>.{cu,ptx,json}`.
#
# The ignored suite is run with `--no-fail-fast`: known-failing GPU tests are
# recorded, not hidden, and cannot stop the capture. Recipe identity is decided
# from the captured files only.
[CmdletBinding()]
param(
    [string]$Baseline = '360718f521a2728f50e1c207c9d7a91d9a5a0eca',
    [string]$Current = 'e9ae451b60218cf904e9dcae2f69f16b94cfed5a',
    [string]$Features = 'cuda-probe',
    [string]$Toolchain = '+1.99.0-x86_64-pc-windows-msvc',
    [string]$WorkRoot = 'target/split-ptx',
    [string]$EvidenceDir = 'docs/development-evidence/split-ptx-identity',
    [switch]$ReuseRuns
)
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
Set-Location $repo
$toolchainName = $Toolchain.TrimStart('+')
$utf8 = New-Object System.Text.UTF8Encoding($false)
$cuda = Join-Path $repo 'target/toolchains/cuda-12.8.1'

function Fail([string]$message) { throw "compare-split-ptx: $message" }

function Write-JsonFile([string]$path, $value) {
    $directory = Split-Path -Parent $path
    if ($directory) { New-Item -ItemType Directory -Force $directory | Out-Null }
    [System.IO.File]::WriteAllText($path, (($value | ConvertTo-Json -Depth 8) + "`n"), $utf8)
}

function Read-JsonFile([string]$path) { return Get-Content -LiteralPath $path -Raw | ConvertFrom-Json }

function Get-Sha256([string]$path) {
    return (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Get-TextSha256([string[]]$lines) {
    $text = (($lines | Sort-Object) -join "`n") + "`n"
    $sha = [System.Security.Cryptography.SHA256]::Create()
    try { return ([BitConverter]::ToString($sha.ComputeHash($utf8.GetBytes($text)))).Replace('-', '').ToLowerInvariant() }
    finally { $sha.Dispose() }
}

function Invoke-Git([string[]]$arguments) {
    $output = & git @arguments 2>&1
    if ($LASTEXITCODE -ne 0) { Fail ("git {0} failed: {1}" -f ($arguments -join ' '), ($output -join ' ')) }
    return $output
}

function Get-CommitMeta([string]$revision) {
    $full = ((Invoke-Git @('rev-parse', '--verify', "$revision^{commit}")) | Select-Object -First 1).Trim()
    return [ordered]@{
        revision = $full
        short = $full.Substring(0, 7)
        subject = ((Invoke-Git @('log', '-1', '--format=%s', $full)) -join ' ').Trim()
        committedAt = ((Invoke-Git @('log', '-1', '--format=%cI', $full)) -join ' ').Trim()
        tree = ((Invoke-Git @('rev-parse', "$full^{tree}")) -join ' ').Trim()
    }
}

# Export one commit into <work>/<short>/src using git archive + tar.
function Expand-Commit([string]$fullSha, [string]$directory) {
    $archiveDirectory = Join-Path (Join-Path $repo $WorkRoot) 'archives'
    New-Item -ItemType Directory -Force $archiveDirectory | Out-Null
    $archive = Join-Path $archiveDirectory "$fullSha.tar"
    $source = Join-Path $directory 'src'
    if (-not (Test-Path -LiteralPath $archive)) {
        Invoke-Git @('archive', '--format=tar', "--output=$archive", $fullSha) | Out-Null
    }
    if (-not (Test-Path -LiteralPath (Join-Path $source 'Cargo.toml'))) {
        if (Test-Path -LiteralPath $source) { Remove-Item -LiteralPath $source -Recurse -Force }
        New-Item -ItemType Directory -Force $source | Out-Null
        & tar -xf $archive -C $source
        if ($LASTEXITCODE -ne 0) { Fail "tar extraction failed for $fullSha" }
    }
    return $source
}

function Get-TestSummary([string]$logPath) {
    if (-not (Test-Path -LiteralPath $logPath)) { return [ordered]@{ suites = 0; passed = 0; failed = 0; ignored = 0; compileError = $false } }
    $text = Get-Content -LiteralPath $logPath -Raw
    $passed = 0; $failed = 0; $ignored = 0; $suites = 0
    foreach ($match in [regex]::Matches($text, 'test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored;')) {
        $passed += [int]$match.Groups[1].Value
        $failed += [int]$match.Groups[2].Value
        $ignored += [int]$match.Groups[3].Value
        $suites++
    }
    return [ordered]@{
        suites = $suites
        passed = $passed
        failed = $failed
        ignored = $ignored
        compileError = [bool]($text -match 'error\[E\d+\]|error: could not compile|could not compile `mjwarp-rs`')
    }
}

# Run the capture suite for one commit. Build products stay in the per-commit
# CARGO_TARGET_DIR; the capture root is a fresh empty directory.
function Invoke-Capture([hashtable]$meta, [string]$directory) {
    $source = Join-Path $directory 'src'
    $target = Join-Path $directory 'target'
    $capture = Join-Path $directory 'ptx'
    $stdout = Join-Path $directory 'run.stdout.log'
    $stderr = Join-Path $directory 'run.stderr.log'
    $log = Join-Path $directory 'run.log'
    $metaPath = Join-Path $directory 'run-meta.json'
    $arguments = "$Toolchain test --locked --features $Features --no-fail-fast -- --ignored --test-threads=1 --nocapture"

    if ($ReuseRuns) {
        if (-not (Test-Path -LiteralPath $metaPath)) { Fail "reuse requested but $metaPath is missing" }
        if (-not (Test-Path -LiteralPath $capture)) { Fail 'reuse requested but capture directory is missing' }
        return [ordered]@{ meta = (Read-JsonFile $metaPath); capture = $capture; log = $log; reused = $true; command = $arguments }
    }

    if (Test-Path -LiteralPath $capture) { Remove-Item -LiteralPath $capture -Recurse -Force }
    New-Item -ItemType Directory -Force $capture | Out-Null
    foreach ($path in @($stdout, $stderr)) { if (Test-Path -LiteralPath $path) { Remove-Item -LiteralPath $path -Force } }
    $previous = @{}
    foreach ($key in @('CUDA_PATH', 'CARGO_TARGET_DIR', 'MJWARP_PHYSICS_PTX_DIR')) {
        $previous[$key] = [Environment]::GetEnvironmentVariable($key)
    }
    $previousPath = $env:PATH
    $env:CUDA_PATH = $cuda
    $env:PATH = (Join-Path $cuda 'bin') + ';' + $previousPath
    $env:CARGO_TARGET_DIR = $target
    $env:MJWARP_PHYSICS_PTX_DIR = $capture
    $startedAt = [DateTime]::UtcNow
    $timer = [System.Diagnostics.Stopwatch]::StartNew()
    $peakMiB = 0
    try {
        $process = Start-Process -FilePath 'cargo' -ArgumentList $arguments -WorkingDirectory $source `
            -RedirectStandardOutput $stdout -RedirectStandardError $stderr -PassThru -NoNewWindow
        while (-not $process.HasExited) {
            $used = & nvidia-smi --query-gpu=memory.used --format=csv,noheader,nounits 2>$null
            if ($LASTEXITCODE -eq 0 -and $used) {
                $first = @($used)[0].Trim()
                if ($first -match '^\d+$' -and [int]$first -gt $peakMiB) { $peakMiB = [int]$first }
            }
            Start-Sleep -Seconds 2
        }
        $process.WaitForExit()
        $timer.Stop()
        $exitCode = $process.ExitCode
        $combined = @()
        foreach ($part in @($stdout, $stderr)) {
            if ((Test-Path -LiteralPath $part) -and (Get-Item -LiteralPath $part).Length -gt 0) {
                $combined += (Get-Content -LiteralPath $part -Raw)
            }
        }
        [System.IO.File]::WriteAllText($log, ($combined -join "`n"), $utf8)
        $runMeta = [ordered]@{
            exitCode = $exitCode
            elapsedSeconds = [math]::Round($timer.Elapsed.TotalSeconds, 2)
            peakGpuMemoryMiB = $peakMiB
            startedAtUtc = $startedAt.ToString('o')
            command = $arguments
            cudaPath = $cuda
            sourceDirectory = $source.Substring($repo.Length + 1)
            cargoTargetDir = $target.Substring($repo.Length + 1)
            captureDirectory = $capture.Substring($repo.Length + 1)
            captureEnv = 'MJWARP_PHYSICS_PTX_DIR'
            log = $log.Substring($repo.Length + 1)
        }
        Write-JsonFile $metaPath $runMeta
        return [ordered]@{ meta = $runMeta; capture = $capture; log = $log; reused = $false; command = $arguments }
    } finally {
        $timer.Stop()
        foreach ($key in $previous.Keys) {
            if ($null -eq $previous[$key]) { Remove-Item -Path "env:$key" -ErrorAction SilentlyContinue }
            else { Set-Item -Path "env:$key" -Value $previous[$key] }
        }
        $env:PATH = $previousPath
    }
}

# One captured .ptx = one compiled recipe variant; the file name carries
# `<entry>-<ptx sha256>` and the companion JSON must agree.
function Measure-Capture([string]$captureRoot) {
    $records = @()
    $anomalies = @()
    $nvrtc = @{}
    $entries = @{}
    $seen = @{}
    $files = Get-ChildItem -LiteralPath $captureRoot -Recurse -File
    foreach ($file in $files) {
        if ($file.Extension -ne '.ptx') { continue }
        $relative = $file.FullName.Substring($captureRoot.Length + 1)
        $prefix = [System.IO.Path]::GetFileNameWithoutExtension($file.Name)
        $jsonPath = Join-Path $file.DirectoryName ($prefix + '.json')
        $cuPath = Join-Path $file.DirectoryName ($prefix + '.cu')
        if (-not (Test-Path -LiteralPath $jsonPath)) { $anomalies += "missing companion JSON: $relative"; continue }
        $record = Read-JsonFile $jsonPath
        if ($prefix -ne "$($record.entry)-$($record.ptxSha256)") { $anomalies += "name/JSON prefix mismatch: $relative" }
        $ptxSha = Get-Sha256 $file.FullName
        if ($ptxSha -ne $record.ptxSha256) { $anomalies += "PTX sha mismatch: $relative" }
        if ($record.precisionGate -ne 'f32-physics') { $anomalies += "unexpected precision gate: $relative" }
        if ($record.fmad -ne $true -or $record.useFastMath -ne $false) { $anomalies += "unexpected recipe flags: $relative" }
        $cuSha = $null
        if (Test-Path -LiteralPath $cuPath) {
            $cuSha = Get-Sha256 $cuPath
            if ($cuSha -ne $record.sourceSha256) { $anomalies += "CU source sha mismatch: $relative" }
        }
        $version = (@($record.nvrtcVersion) | ForEach-Object { "$_" }) -join '.'
        $nvrtc[$version] = $true
        $entries[[string]$record.entry] = $true
        if ($seen.ContainsKey($relative)) { $anomalies += "duplicate capture: $relative" }
        $seen[$relative] = $true
        $records += [ordered]@{
            name = $relative
            entry = [string]$record.entry
            ptxSha256 = $ptxSha
            cuSha256 = $cuSha
            nvrtcVersion = $version
            passed = [bool]$record.passed
            bytes = $file.Length
        }
    }
    $recipeNames = @($entries.Keys | Sort-Object)
    return [ordered]@{
        captureDirectory = $captureRoot.Substring($repo.Length + 1)
        recipeCount = $recipeNames.Count
        recipes = $recipeNames
        ptxFileCount = $records.Count
        ptxBytes = [long](($records | ForEach-Object { $_.bytes } | Measure-Object -Sum).Sum)
        digestSha256 = Get-TextSha256 @($records | ForEach-Object { "$($_.name)|$($_.ptxSha256)" })
        sourceDigestSha256 = Get-TextSha256 @($records | Where-Object { $_.cuSha256 } | ForEach-Object { "$($_.name)|$($_.cuSha256)" })
        nvrtcVersions = @($nvrtc.Keys | Sort-Object)
        precisionGatePassed = @($records | Where-Object { $_.passed }).Count
        precisionGateFailed = @($records | Where-Object { -not $_.passed }).Count
        anomalies = $anomalies
        records = $records
        recipesList = $recipeNames
    }
}

function Compare-Capture($baseline, $current) {
    $baseMap = @{}
    foreach ($record in $baseline.records) { $baseMap[$record.name] = $record }
    $currentMap = @{}
    foreach ($record in $current.records) { $currentMap[$record.name] = $record }
    $missing = @($baseMap.Keys | Where-Object { -not $currentMap.ContainsKey($_) } | Sort-Object)
    $extra = @($currentMap.Keys | Where-Object { -not $baseMap.ContainsKey($_) } | Sort-Object)
    $changed = @()
    $changedSources = @()
    foreach ($name in ($baseMap.Keys | Where-Object { $currentMap.ContainsKey($_) } | Sort-Object)) {
        if ($baseMap[$name].ptxSha256 -ne $currentMap[$name].ptxSha256) {
            $changed += [ordered]@{ name = $name; baseline = $baseMap[$name].ptxSha256; current = $currentMap[$name].ptxSha256 }
        }
        if ($baseMap[$name].cuSha256 -ne $currentMap[$name].cuSha256) {
            $changedSources += [ordered]@{ name = $name; baseline = $baseMap[$name].cuSha256; current = $currentMap[$name].cuSha256 }
        }
    }
    $recipeDiff = @(Compare-Object -ReferenceObject @($baseline.recipes) -DifferenceObject @($current.recipes))
    $identical = ($baseline.anomalies.Count -eq 0) -and ($current.anomalies.Count -eq 0) -and
        ($missing.Count -eq 0) -and ($extra.Count -eq 0) -and ($changed.Count -eq 0) -and
        ($recipeDiff.Count -eq 0) -and ($baseline.digestSha256 -eq $current.digestSha256)
    return [ordered]@{
        identical = [bool]$identical
        recipeCountsEqual = ($baseline.recipeCount -eq $current.recipeCount)
        ptxFileCountsEqual = ($baseline.ptxFileCount -eq $current.ptxFileCount)
        ptxDigestsEqual = ($baseline.digestSha256 -eq $current.digestSha256)
        sourceDigestsEqual = ($baseline.sourceDigestSha256 -eq $current.sourceDigestSha256)
        missingInCurrent = $missing
        extraInCurrent = $extra
        changedPtx = $changed
        changedSources = $changedSources
    }
}

function Get-EnvironmentInfo {
    $gpu = @(& nvidia-smi --query-gpu=name,driver_version,memory.total --format=csv,noheader 2>$null)
    if ($LASTEXITCODE -ne 0) { Fail 'nvidia-smi failed' }
    $rustc = (& rustc "+$toolchainName" --version 2>&1) -join ' '
    if ($LASTEXITCODE -ne 0) { $rustc = 'unavailable' }
    $nvrtcDll = Join-Path $cuda 'bin/nvrtc64_120_0.dll'
    return [ordered]@{
        host = $env:COMPUTERNAME
        os = [System.Environment]::OSVersion.VersionString
        cpu = (Get-CimInstance Win32_Processor | Select-Object -First 1).Name
        gpu = $gpu
        cudaPath = $cuda.Substring($repo.Length + 1)
        nvrtcDllVersion = $(if (Test-Path $nvrtcDll) { (Get-Item $nvrtcDll).VersionInfo.FileVersion } else { 'missing' })
        rustc = $rustc
        cargo = ((& cargo --version 2>&1) -join ' ')
    }
}

New-Item -ItemType Directory -Force (Join-Path $repo $WorkRoot) | Out-Null
$baselineMeta = Get-CommitMeta $Baseline
$currentMeta = Get-CommitMeta $Current
Invoke-Git @('merge-base', '--is-ancestor', $baselineMeta.revision, $currentMeta.revision) | Out-Null
$baselineDirectory = Join-Path (Join-Path $repo $WorkRoot) $baselineMeta.short
$currentDirectory = Join-Path (Join-Path $repo $WorkRoot) $currentMeta.short
foreach ($directory in @($baselineDirectory, $currentDirectory)) { New-Item -ItemType Directory -Force $directory | Out-Null }
Expand-Commit $baselineMeta.revision $baselineDirectory | Out-Null
Expand-Commit $currentMeta.revision $currentDirectory | Out-Null

$baselineRun = Invoke-Capture $baselineMeta $baselineDirectory
$currentRun = Invoke-Capture $currentMeta $currentDirectory
$baselineCapture = Measure-Capture $baselineRun.capture
$currentCapture = Measure-Capture $currentRun.capture
$comparison = Compare-Capture $baselineCapture $currentCapture
if ($comparison.changedSources.Count -gt 0) {
    $comparison['changedSourcesNote'] = 'CU source bytes differ for these entries while PTX bytes match; test probes may embed a per-run temporary path in a source comment.'
}
$environment = Get-EnvironmentInfo

$report = [ordered]@{
    schema = 1
    kind = 'split-ptx-identity'
    task_id = '2026-10-22-split-ptx-identity'
    generatedAtUtc = ([DateTime]::UtcNow.ToString('o'))
    commits = [ordered]@{ baseline = $baselineMeta; current = $currentMeta }
    command = [ordered]@{
        toolchain = $Toolchain
        cargo = $baselineRun.command
        features = $Features
        captureRule = 'MJWARP_PHYSICS_PTX_DIR is a fresh empty capture root; product record_physics_ptx writes <entry>-<ptxSha256>.{cu,ptx,json}'
        readOnlyHistory = 'git archive + tar extraction; no worktree/checkout; per-commit CARGO_TARGET_DIR'
        featureScope = 'product kernels are compiled under cuda-probe; native-model-probe/cubecl probes do not compile f32-physics recipes'
        reuseRuns = [bool]$ReuseRuns
    }
    environment = $environment
    runs = [ordered]@{
        baseline = [ordered]@{
            exitCode = $baselineRun.meta.exitCode
            elapsedSeconds = $baselineRun.meta.elapsedSeconds
            peakGpuMemoryMiB = $baselineRun.meta.peakGpuMemoryMiB
            log = $baselineRun.meta.log
            cargoTargetDir = $baselineRun.meta.cargoTargetDir
            reused = [bool]$baselineRun.reused
        }
        current = [ordered]@{
            exitCode = $currentRun.meta.exitCode
            elapsedSeconds = $currentRun.meta.elapsedSeconds
            peakGpuMemoryMiB = $currentRun.meta.peakGpuMemoryMiB
            log = $currentRun.meta.log
            cargoTargetDir = $currentRun.meta.cargoTargetDir
            reused = [bool]$currentRun.reused
        }
    }
    capture = [ordered]@{
        baseline = [ordered]@{
            captureDirectory = $baselineCapture.captureDirectory
            recipeCount = $baselineCapture.recipeCount
            recipes = $baselineCapture.recipesList
            ptxFileCount = $baselineCapture.ptxFileCount
            ptxBytes = $baselineCapture.ptxBytes
            digestSha256 = $baselineCapture.digestSha256
            sourceDigestSha256 = $baselineCapture.sourceDigestSha256
            nvrtcVersions = $baselineCapture.nvrtcVersions
            precisionGatePassed = $baselineCapture.precisionGatePassed
            precisionGateFailed = $baselineCapture.precisionGateFailed
            anomalies = $baselineCapture.anomalies
            testSummary = (Get-TestSummary $baselineRun.log)
        }
        current = [ordered]@{
            captureDirectory = $currentCapture.captureDirectory
            recipeCount = $currentCapture.recipeCount
            recipes = $currentCapture.recipesList
            ptxFileCount = $currentCapture.ptxFileCount
            ptxBytes = $currentCapture.ptxBytes
            digestSha256 = $currentCapture.digestSha256
            sourceDigestSha256 = $currentCapture.sourceDigestSha256
            nvrtcVersions = $currentCapture.nvrtcVersions
            precisionGatePassed = $currentCapture.precisionGatePassed
            precisionGateFailed = $currentCapture.precisionGateFailed
            anomalies = $currentCapture.anomalies
            testSummary = (Get-TestSummary $currentRun.log)
        }
    }
    comparison = $comparison
    identical = [bool]$comparison.identical
    notes = @(
        'recipeCount = distinct kernel entries; ptxFileCount = <entry>-<ptxSha256>.ptx triples.'
        'identical requires: no capture anomalies, equal PTX file-name sets, equal per-file PTX sha256, equal recipe sets and equal PTX digest.'
        'Known-failing ignored GPU tests are recorded in testSummary only; they decide no recipe identity and are not skipped silently.'
        'Sub-process exit codes are not used for identity: the ignored suite contains known failing tests; identity is decided from captured files only.'
        'Scope is the source split 360718f -> e9ae451 on this host; no Linux/performance/precision-acceptance claim.'
    )
}
$reportPath = Join-Path $repo (Join-Path $EvidenceDir 'report.json')
Write-JsonFile $reportPath $report
Write-Host ("split-ptx-identity: baseline recipes={0} files={1}; current recipes={2} files={3}; identical={4}" -f
    $baselineCapture.recipeCount, $baselineCapture.ptxFileCount, $currentCapture.recipeCount,
    $currentCapture.ptxFileCount, $comparison.identical)
if (-not $comparison.identical) {
    Fail ("PTX identity failed: missing={0} extra={1} changed={2}; see {3}" -f
        $comparison.missingInCurrent.Count, $comparison.extraInCurrent.Count, $comparison.changedPtx.Count, $reportPath)
}
Write-Host "report: $($reportPath.Substring($repo.Length + 1))"
