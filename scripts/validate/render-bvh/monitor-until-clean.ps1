param([string]$OutCsv='target/s3b-validation-window/monitor-pre-window.csv',[int]$Required=60,[int]$MaxMinutes=12)
$ErrorActionPreference = 'Stop'
Set-Location 'D:\Rust\mjwarp-rs'
$utf8 = New-Object System.Text.UTF8Encoding($false)
$rows = New-Object System.Collections.Generic.List[object]
$pattern = 'python|g10_p5_|product_collision|stage2_equivalence|cargo|rustc|nvcc|cl\.exe|render_|mjwarp'
$consec = 0; $ready = $false; $sw = [Diagnostics.Stopwatch]::StartNew()
while ($sw.Elapsed.TotalSeconds -lt ($MaxMinutes * 60)) {
    $line = & nvidia-smi --query-gpu=memory.used,memory.free,utilization.gpu,temperature.gpu --format=csv,noheader
    if (-not $line) { throw 'nvidia-smi returned nothing' }
    $p = $line -split ',\s*'
    $used = [int]($p[0] -replace ' MiB',''); $free = [int]($p[1] -replace ' MiB','')
    $util = [int]($p[2] -replace ' %',''); $temp = [int]$p[3]
    $raw = @(& nvidia-smi --query-compute-apps=pid,process_name,used_memory --format=csv,noheader)
    $filtered = @($raw | Where-Object { $_ -and ($_ -match $pattern) })
    $build = @(Get-Process cargo,rustc,cl,nvcc,link -ErrorAction SilentlyContinue)
    $ok = ($free -ge 10240) -and ($util -lt 40) -and ($filtered.Count -eq 0) -and ($build.Count -eq 0)
    if ($ok) { $consec++ } else { $consec = 0 }
    $rows.Add([pscustomobject]@{
        ts = (Get-Date).ToString('yyyy-MM-ddTHH:mm:ss.fffK'); used = $used; free = $free; util = $util; temp = $temp
        cuda_raw = $raw.Count; cuda_filtered = $filtered.Count; build = $build.Count; clean = $ok; streak = $consec
    })
    if ($consec -ge $Required) { $ready = $true; break }
    Start-Sleep -Seconds 5
}
$lines = @('ts,used_mib,free_mib,util_percent,temp_c,compute_raw,compute_filtered,build,clean,streak') +
    @($rows | ForEach-Object { "$($_.ts),$($_.used),$($_.free),$($_.util),$($_.temp),$($_.cuda_raw),$($_.cuda_filtered),$($_.build),$($_.clean),$($_.streak)" })
$path = Join-Path (Get-Location).Path $OutCsv
[IO.File]::WriteAllText($path, (($lines -join "`n") + "`n"), $utf8)
$best = 0; $cur = 0
foreach ($r in $rows) { if ($r.clean) { $cur++; if ($cur -gt $best) { $best = $cur } } else { $cur = 0 } }
"monitor samples=$($rows.Count) longest_clean=$best required=$Required ready=$ready start=$($rows[0].ts) end=$($rows[-1].ts) util_min=$(($rows | Measure-Object util -Minimum).Minimum) util_max=$(($rows | Measure-Object util -Maximum).Maximum) free_min=$(($rows | Measure-Object free -Minimum).Minimum)"
if (-not $ready) { exit 20 }
exit 0