# Pre-gate worktree EOL normalization (plan A: zero blob changes).
#
# Why: normify 0.7's `checks.unchanged` compares WORKTREE raw bytes against the
# HEAD blob. This machine's worktree carries ~428 historical CRLF/mixed-EOL files
# under docs/evidence, docs/development-evidence and fixtures (git status stays
# clean), so any card whose `unchanged` covers those paths randomly reports
# "protected path modified". This script rewrites ONLY files whose worktree bytes
# differ from HEAD solely by line endings, restoring the HEAD bytes. It never
# changes a git blob, never touches the index, and prints counts.
#
# Fast path: HEAD blob shas+sizes come from one `git ls-tree` + one
# `git cat-file --batch-check`; only size-mismatched files are read and checked
# (an EOL-only difference always changes the byte count). A clean tracked tree
# (enforced below) guarantees that any raw difference is EOL-only, so a
# size-equal file cannot carry a real edit.
#
# After writing, the script runs a chunked `git add` ONLY to refresh the index
# stat cache: with core.autocrlf=true the index remembers the CRLF size, so an
# LF-rewritten file is content-identical to its HEAD blob but still shows ` M`.
# The blob itself never changes (post-check re-hashes every rewritten file
# against HEAD and `git ls-files -s` stays byte-identical).
#
# Run it after any checkout/reset that touches these paths, and always before a
# gate that carries a `checks.unchanged` entry over docs/ or fixtures/.
#
# Usage (run before gates):
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate/normalize-evidence-eol.ps1
#   powershell ... -DryRun
#   powershell ... -Force     # allow when tracked files are dirty (dev only)
#
# Exit codes: 0 = ok (nothing to do or normalized), 1 = real content differences
#             found (listed, NOT modified), 2 = refused because worktree is dirty.
#
# ASCII-only on purpose: Windows PowerShell 5.1 reads non-BOM UTF-8 as ANSI.

[CmdletBinding()]
param(
    [string]$RepoRoot = '',
    [string[]]$Paths = @('docs/evidence', 'docs/development-evidence', 'fixtures'),
    [switch]$DryRun,
    [switch]$Force,
    [int]$ListLimit = 20
)

$ErrorActionPreference = 'Stop'
$Latin1 = [System.Text.Encoding]::GetEncoding(28591)

# $PSScriptRoot can be empty in Windows PowerShell 5.1 when the script is invoked
# through a relative path, so resolve the repo root defensively.
if ([string]::IsNullOrEmpty($RepoRoot)) {
    if (-not [string]::IsNullOrEmpty($PSScriptRoot)) {
        $RepoRoot = (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
    }
    else {
        $RepoRoot = (Get-Location).Path
    }
}
if (-not (Test-Path -LiteralPath (Join-Path $RepoRoot '.git'))) {
    throw ("repo root not found (no .git): " + $RepoRoot + "; pass -RepoRoot explicitly")
}
$RepoRoot = (Resolve-Path -LiteralPath $RepoRoot).Path

function Invoke-Git {
    param([string[]]$GitArgs)
    $out = & git @GitArgs 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw ("git " + ($GitArgs -join ' ') + " failed: " + ($out -join ' '))
    }
    return $out
}

Push-Location $RepoRoot
try {
    # Ready-check: dirty tracked tree means real content changes are in flight.
    $dirty = @(& git status --porcelain --untracked-files=no)
    if ($LASTEXITCODE -ne 0) { throw 'git status failed' }
    if ($dirty.Count -gt 0 -and -not $Force) {
        Write-Host ("REFUSE: tracked worktree has " + $dirty.Count + " dirty entries; rerun with -Force only if you know the diff is only EOL.")
        $dirty | Select-Object -First $ListLimit | ForEach-Object { Write-Host ("  dirty: " + $_) }
        exit 2
    }

    # 1. One pass over HEAD: repo-relative path -> @{ Sha; Size }.
    $blobs = @{}
    foreach ($p in $Paths) {
        $listed = @(& git ls-tree -r -z HEAD -- $p)
        if ($LASTEXITCODE -ne 0) { throw ("git ls-tree failed for " + $p) }
        foreach ($entry in $listed) {
            foreach ($rec in ($entry -split "`0")) {
                if ([string]::IsNullOrWhiteSpace($rec)) { continue }
                $tab = $rec.IndexOf("`t")
                if ($tab -lt 0) { continue }
                $meta = $rec.Substring(0, $tab) -split ' '
                if ($meta.Count -lt 3 -or $meta[1] -ne 'blob') { continue }
                $rel = $rec.Substring($tab + 1)
                $blobs[$rel] = @{ Sha = $meta[2]; Size = -1 }
            }
        }
    }
    Write-Host ("scanned: " + $blobs.Count + " tracked files under " + (($Paths -join ', ')))
    if ($blobs.Count -eq 0) { exit 0 }

    # 2. Blob sizes in one batch call (avoid reading equal-size files at all).
    $shaList = @($blobs.Values | ForEach-Object { $_.Sha } | Sort-Object -Unique)
    $sizeLines = @($shaList | & git cat-file --batch-check)
    if ($LASTEXITCODE -ne 0) { throw 'git cat-file --batch-check failed' }
    $sizes = @{}
    foreach ($line in $sizeLines) {
        $f = ($line -split ' ')
        if ($f.Count -ge 3) { $sizes[$f[0]] = [int64]$f[2] }
    }
    foreach ($rel in @($blobs.Keys)) { $blobs[$rel].Size = $sizes[$blobs[$rel].Sha] }

    $tmp = [System.IO.Path]::GetTempFileName()
    $stale = 0
    $normalized = 0
    $realDiff = 0
    $sameSize = 0
    $missing = 0
    $staleList = @()
    $realDiffs = @()

    foreach ($rel in ($blobs.Keys | Sort-Object)) {
        $info = $blobs[$rel]
        $full = Join-Path $RepoRoot $rel
        if (-not (Test-Path -LiteralPath $full)) { $missing++; continue }
        $len = (Get-Item -LiteralPath $full).Length
        if ($len -eq $info.Size) { $sameSize++; continue }   # EOL-only diffs always change size

        $bytes = [System.IO.File]::ReadAllBytes($full)
        $text = $Latin1.GetString($bytes)
        $stripped = $Latin1.GetBytes($text.Replace("`r", ''))
        [System.IO.File]::WriteAllBytes($tmp, $stripped)
        $normHash = (& git hash-object -- $tmp)
        if ($LASTEXITCODE -ne 0) { continue }
        if ($normHash -eq $info.Sha) {
            $stale++
            $staleList += $rel
            if (-not $DryRun) {
                # normHash == blob sha proves the CR-stripped bytes ARE the HEAD blob.
                [System.IO.File]::WriteAllBytes($full, $stripped)
                $normalized++
            }
        }
        else {
            $realDiff++
            if ($realDiffs.Count -lt $ListLimit) { $realDiffs += $rel }
        }
    }
    Remove-Item -LiteralPath $tmp -ErrorAction SilentlyContinue

    Write-Host ("same-size (identical by guard): " + $sameSize)
    Write-Host ("stale-eol: " + $stale)
    Write-Host ("normalized: " + $normalized + $(if ($DryRun) { " (dry-run: not written)" } else { "" }))
    Write-Host ("real-content-diff: " + $realDiff)
    Write-Host ("missing-in-worktree: " + $missing)
    foreach ($s in ($staleList | Select-Object -First $ListLimit)) { Write-Host ("  stale sample: " + $s) }
    foreach ($s in $realDiffs) { Write-Host ("  real diff (left untouched): " + $s) }

    # 3. Index stat refresh. core.autocrlf=true makes the index remember the
    #    CRLF ("smudged") size, so a rewritten LF file is content-identical to its
    #    HEAD blob yet shows up as ` M` until git re-stats it. `git add` only
    #    refreshes the cached stat/oid: the clean filter maps LF to LF, so the
    #    blob is unchanged (verified by the post-check below and by an identical
    #    `git ls-files -s` fingerprint).
    if (-not $DryRun -and $normalized -gt 0) {
        $i = 0
        while ($i -lt $staleList.Count) {
            $end = [Math]::Min($i + 199, $staleList.Count - 1)
            $chunk = @($staleList[$i..$end])
            & git add -- $chunk 2>$null
            if ($LASTEXITCODE -ne 0) { throw 'git add (stat refresh) failed' }
            $i = $end + 1
        }
    }

    # 4. Post-check (only when written): every stale file must hash back to its HEAD blob
    #    and the tracked tree must still be clean.
    if (-not $DryRun -and $normalized -gt 0) {
        $bad = 0
        foreach ($rel in $staleList) {
            $h = (& git hash-object -- (Join-Path $RepoRoot $rel))
            if ($h -ne $blobs[$rel].Sha) { $bad++; Write-Host ("  POST-CHECK FAIL: " + $rel) }
        }
        $dirtyAfter = @(& git status --porcelain --untracked-files=no)
        Write-Host ("post-check: " + ($staleList.Count - $bad) + "/" + $staleList.Count + " hash back to HEAD; dirty after: " + $dirtyAfter.Count)
        if ($bad -gt 0 -or $dirtyAfter.Count -gt 0) { exit 1 }
    }

    if ($realDiff -gt 0) { exit 1 }
    exit 0
}
finally {
    Pop-Location
}
