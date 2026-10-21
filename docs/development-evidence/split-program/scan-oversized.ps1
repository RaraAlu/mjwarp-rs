# Scan repository code files for the 500-line limit.
# Used by task 2026-10-21-split-program-reconcile to produce report.json.
# Inspected extensions: rs, cu, c, h, cpp, sh, ps1.
# Excluded directories: evidence (docs/) and sample (fixtures/) trees plus VCS and build output.
$ErrorActionPreference = 'Stop'

$repoRoot = (git -C $PSScriptRoot rev-parse --show-toplevel).Trim()
$limit = 500
$exts = @('.rs', '.cu', '.c', '.h', '.cpp', '.sh', '.ps1')
$excludedDirs = @('docs', 'fixtures', 'target', '.git')

Push-Location $repoRoot
try {
    $files = Get-ChildItem -Path . -Recurse -File -Force | Where-Object {
        $exts -contains $_.Extension.ToLower()
    } | ForEach-Object {
        $rel = $_.FullName.Substring($repoRoot.Length + 1).Replace('\', '/')
        $top = ($rel -split '/')[0]
        if ($excludedDirs -contains $top) { return }
        [pscustomobject]@{
            path = $rel
            lines = (Get-Content -LiteralPath $_.FullName -Encoding utf8 | Measure-Object -Line).Lines
        }
    }

    $sorted = @($files | Sort-Object -Property @{ Expression = 'lines'; Descending = $true }, @{ Expression = 'path'; Descending = $false })
    $over = @($sorted | Where-Object { $_.lines -gt $limit })
    $totalLines = 0
    foreach ($f in $sorted) { $totalLines += $f.lines }

    $report = [pscustomobject]@{
        schema_version = 1
        task = '2026-10-21-split-program-reconcile'
        generated_at = (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ')
        generated_by = 'docs/development-evidence/split-program/scan-oversized.ps1'
        head = (git rev-parse HEAD).Trim()
        limit_lines = $limit
        inspected_extensions = $exts
        excluded_dirs = $excludedDirs
        file_count = $sorted.Count
        total_lines = $totalLines
        over_limit_files = $over.Count
        max_lines = $sorted[0].lines
        max_lines_path = $sorted[0].path
        over_limit = $over
        files = $sorted
    }

    $outPath = Join-Path $PSScriptRoot 'report.json'
    $json = $report | ConvertTo-Json -Depth 6
    $utf8 = New-Object System.Text.UTF8Encoding($false)
    [System.IO.File]::WriteAllText($outPath, $json + "`n", $utf8)
    Write-Output ("report: {0}" -f $outPath)
    Write-Output ("files: {0}" -f $report.file_count)
    Write-Output ("total_lines: {0}" -f $report.total_lines)
    Write-Output ("max_lines: {0} ({1})" -f $report.max_lines, $report.max_lines_path)
    Write-Output ("over_limit_files: {0}" -f $report.over_limit_files)
}
finally {
    Pop-Location
}
