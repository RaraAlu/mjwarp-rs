$ErrorActionPreference = 'Stop'
$root = 'D:\Rust\mjwarp-rs'
$target = Join-Path $root 'docs\development-evidence\layers-m3-split-resident'
$reportPath = Join-Path $target 'report.json'
$limit = 500
New-Item -ItemType Directory -Force $target | Out-Null
if (Test-Path -LiteralPath $reportPath) { throw "报告已存在：$reportPath" }

$files = New-Object System.Collections.Generic.List[object]
$rootFile = Join-Path $root 'src\physics\resident.rs'
$files.Add([pscustomobject]@{ path = 'src/physics/resident.rs'; lines = ([IO.File]::ReadAllLines($rootFile)).Length })
$children = Get-ChildItem (Join-Path $root 'src\physics\resident') -Recurse -File -Filter '*.rs' | Sort-Object FullName
foreach ($file in $children) {
    $relative = $file.FullName.Substring($root.Length + 1).Replace('\', '/')
    $files.Add([pscustomobject]@{ path = $relative; lines = ([IO.File]::ReadAllLines($file.FullName)).Length })
}
$over = @($files | Where-Object { [int]$_.lines -gt $limit })
$total = 0
foreach ($file in $files) { $total += [int]$file.lines }
$max = 0
foreach ($file in $files) { if ([int]$file.lines -gt $max) { $max = [int]$file.lines } }
$report = [ordered]@{
    schema = 1
    kind = 'size-report'
    task_id = '2026-10-21-refactor-split-resident'
    source_root = 'src/physics/resident.rs'
    source_root_lines_before = 6878
    limit = $limit
    file_count = $files.Count
    total_lines = $total
    max_lines = $max
    over_limit_files = $over.Count
    over_limit = @($over | ForEach-Object { [ordered]@{ path = $_.path; lines = $_.lines } })
    files = @($files | ForEach-Object { [ordered]@{ path = $_.path; lines = $_.lines } })
    generated_by = 'build-report.ps1'
}
$json = ($report | ConvertTo-Json -Depth 6).Replace("`r`n", "`n") + "`n"
$bytes = [Text.UTF8Encoding]::new($false).GetBytes($json)
$stream = [IO.File]::Open($reportPath, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::Read)
try { $stream.Write($bytes, 0, $bytes.Length) } finally { $stream.Dispose() }
[ordered]@{
    report = $reportPath
    file_count = $report.file_count
    total_lines = $report.total_lines
    max_lines = $report.max_lines
    over_limit_files = $report.over_limit_files
    sha256 = (Get-FileHash -LiteralPath $reportPath -Algorithm SHA256).Hash.ToLowerInvariant()
} | ConvertTo-Json -Depth 3
if ($report.over_limit_files -ne 0) { throw "存在超过 $limit 行的文件" }

