# Build and verify the T4 historical-capture archive.
# Copies the four gitignored runtime capture directories under target/ into one
# tar.gz under this evidence directory and writes a per-file sha256 manifest.
# The source directories are only read; they are never modified.
[CmdletBinding()]
param([switch]$Verify)
$ErrorActionPreference = 'Stop'
$evidence = $PSScriptRoot
$repo = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $evidence))
$utf8 = New-Object System.Text.UTF8Encoding($false)
$archiveName = 't4-historical-captures.tar.gz'
$archive = Join-Path $evidence $archiveName
$manifestPath = Join-Path $evidence 'manifest.json'
$sources = @('t4-g03-retest', 't4-face-operators', 't4-final-face-compile', 't4-rigid-recapture')
$targetRoot = Join-Path $repo 'target'

function Fail([string]$message) { throw "t4-historical-capture-archive: $message" }

function Invoke-GitQuiet([string[]]$arguments) {
    $previous = $ErrorActionPreference
    try {
        $ErrorActionPreference = 'Continue'
        $null = & git @arguments 2>&1
        return $LASTEXITCODE
    } finally { $ErrorActionPreference = $previous }
}

function Get-Sha256([string]$path) {
    return (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Get-TextSha256([string[]]$lines) {
    $text = (($lines | Sort-Object) -join "`n") + "`n"
    $sha = [System.Security.Cryptography.SHA256]::Create()
    try { return ([BitConverter]::ToString($sha.ComputeHash($utf8.GetBytes($text)))).Replace('-', '').ToLowerInvariant() }
    finally { $sha.Dispose() }
}

function Write-Json([string]$path, $value) {
    [System.IO.File]::WriteAllText($path, (($value | ConvertTo-Json -Depth 8) + "`n"), $utf8)
}

function Read-Json([string]$path) { return Get-Content -LiteralPath $path -Raw | ConvertFrom-Json }

function Get-Manifest {
    if (-not (Test-Path -LiteralPath $archive)) { Fail "archive missing: $archiveName (run without -Verify first)" }
    $sourceRows = @()
    $fileRows = @()
    $totalFiles = 0
    $totalBytes = [long]0
    foreach ($name in $sources) {
        $directory = Join-Path $targetRoot $name
        if (-not (Test-Path -LiteralPath $directory)) { Fail "source capture directory missing: target/$name" }
        $files = @(Get-ChildItem -LiteralPath $directory -Recurse -File)
        $treeLines = @()
        $directoryBytes = [long]0
        $latest = [DateTime]::MinValue
        foreach ($file in $files) {
            $relative = $file.FullName.Substring($directory.Length + 1).Replace('\', '/')
            $sha = Get-Sha256 $file.FullName
            $mtime = $file.LastWriteTimeUtc.ToString('o')
            $directoryBytes += $file.Length
            if ($file.LastWriteTimeUtc -gt $latest) { $latest = $file.LastWriteTimeUtc }
            $treeLines += "$relative|$sha|$($file.Length)|$mtime"
            $fileRows += [ordered]@{
                dir = $name
                path = "$name/$relative"
                size = [long]$file.Length
                mtimeUtc = $mtime
                sha256 = $sha
            }
        }
        $sourceRows += [ordered]@{
            path = "target/$name"
            fileCount = $files.Count
            bytes = $directoryBytes
            latestMtimeUtc = $latest.ToString('o')
            treeSha256 = Get-TextSha256 $treeLines
        }
        $totalFiles += $files.Count
        $totalBytes += $directoryBytes
    }
    $archiveInfo = [ordered]@{
        path = $archiveName
        bytes = [long](Get-Item -LiteralPath $archive).Length
        sha256 = Get-Sha256 $archive
        format = 'tar.gz (bsdtar/libarchive)'
        contentsRoots = $sources
    }
    return [ordered]@{
        schema = 1
        kind = 't4-historical-capture-archive'
        task_id = '2026-10-23-t4-historical-capture-archive'
        generatedAtUtc = ([DateTime]::UtcNow.ToString('o'))
        host = $env:COMPUTERNAME
        algorithm = 'sha256'
        sourceRoot = 'target/ (gitignored runtime captures; read-only input)'
        archive = $archiveInfo
        totals = [ordered]@{
            sourceDirectories = $sources.Count
            files = $totalFiles
            bytes = $totalBytes
        }
        sources = $sourceRows
        files = $fileRows
        notes = @(
            'Manifest hashes are computed from the source directories before archiving; verification extracts the archive and recomputes them.'
            'mtimeUtc is informational; verification compares sha256 and size.'
            'Source directories are gitignored runtime captures; the archive is the retained copy.'
        )
    }
}

function Invoke-Verify {
    if (-not (Test-Path -LiteralPath $manifestPath)) { Fail 'manifest.json missing' }
    $manifest = Read-Json $manifestPath
    if (-not (Test-Path -LiteralPath $archive)) { Fail "archive missing: $archiveName" }
    $archiveSize = [long](Get-Item -LiteralPath $archive).Length
    $archiveSha = Get-Sha256 $archive
    if ($archiveSize -ne [long]$manifest.archive.bytes) { Fail "archive size mismatch: $archiveSize vs $($manifest.archive.bytes)" }
    if ($archiveSha -ne $manifest.archive.sha256) { Fail "archive sha256 mismatch: $archiveSha vs $($manifest.archive.sha256)" }
    $temporary = Join-Path $repo 'target/t4-hca-verify'
    if (Test-Path -LiteralPath $temporary) { Remove-Item -LiteralPath $temporary -Recurse -Force }
    New-Item -ItemType Directory -Force $temporary | Out-Null
    try {
        & tar -xzf $archive -C $temporary
        if ($LASTEXITCODE -ne 0) { Fail 'tar extraction failed' }
        $manifestMap = @{}
        foreach ($row in $manifest.files) { $manifestMap[[string]$row.path] = $row }
        $seen = 0
        $mismatches = @()
        foreach ($file in (Get-ChildItem -LiteralPath $temporary -Recurse -File)) {
            $relative = $file.FullName.Substring($temporary.Length + 1).Replace('\', '/')
            $seen++
            if (-not $manifestMap.ContainsKey($relative)) { $mismatches += "unexpected file: $relative"; continue }
            $row = $manifestMap[$relative]
            if ((Get-Sha256 $file.FullName) -ne $row.sha256) { $mismatches += "sha256 mismatch: $relative" }
            if ([long]$file.Length -ne [long]$row.size) { $mismatches += "size mismatch: $relative" }
        }
        if ($seen -ne [int]$manifest.totals.files) { $mismatches += "file count mismatch: extracted $seen vs manifest $($manifest.totals.files)" }
        if ($mismatches.Count -gt 0) {
            $mismatches | Select-Object -First 20 | ForEach-Object { Write-Host $_ }
            Fail ("verification failed: {0} mismatches" -f $mismatches.Count)
        }
        Write-Host ("verified: {0} files, archive {1} bytes, sha256 {2}" -f $seen, $archiveSize, $archiveSha)
    } finally {
        if (Test-Path -LiteralPath $temporary) { Remove-Item -LiteralPath $temporary -Recurse -Force }
    }
}

if ($Verify) {
    Invoke-Verify
    exit 0
}

foreach ($name in $sources) {
    if (-not (Test-Path -LiteralPath (Join-Path $targetRoot $name))) { Fail "source capture directory missing: target/$name" }
}
if (Test-Path -LiteralPath $archive) { Remove-Item -LiteralPath $archive -Force }
$gitBefore = (Get-ChildItem -LiteralPath (Join-Path $repo '.git') -Recurse -File | Measure-Object -Property Length -Sum).Sum
$tarArguments = @('-czf', $archive, '-C', $targetRoot) + $sources
& tar @tarArguments
if ($LASTEXITCODE -ne 0) { Fail 'tar archive creation failed' }

# Measure the repository object growth this evidence adds (archive + manifest).
if ((Invoke-GitQuiet @('-C', $repo, 'hash-object', '-w', '--', $archive)) -ne 0) {
    Fail 'git hash-object for the archive failed'
}
$manifest = Get-Manifest
Write-Json $manifestPath $manifest
if ((Invoke-GitQuiet @('-C', $repo, 'hash-object', '-w', '--', $manifestPath)) -ne 0) {
    Fail 'git hash-object for the manifest failed'
}
$gitAfter = (Get-ChildItem -LiteralPath (Join-Path $repo '.git') -Recurse -File | Measure-Object -Property Length -Sum).Sum
$growth = [long]($gitAfter - $gitBefore)

Invoke-Verify
Write-Host ("manifest: {0} files, {1} bytes, {2} source directories" -f $manifest.totals.files, $manifest.totals.bytes, $manifest.totals.sourceDirectories)
Write-Host ("archive: {0} bytes, sha256 {1}" -f $manifest.archive.bytes, $manifest.archive.sha256)
Write-Host ("git object growth (archive+manifest): {0} bytes ({1} MiB)" -f $growth, [math]::Round($growth / 1MB, 2))
