# GPU guard sampler loop (child process, launched by `Start-GuardSampler`).
#
# Contract: append one CSV row per sample with an immediate flush (AppendAllText closes the handle),
# stop when the stop-file appears. Columns: seq,timestamp,used_mib,free_mib,util_percent,temperature_c
param(
    [Parameter(Mandatory = $true)][string]$Csv,
    [Parameter(Mandatory = $true)][string]$Stop,
    [int]$IntervalMs = 250
)

$seq = 0
while (-not (Test-Path -LiteralPath $Stop)) {
    $line = (& nvidia-smi --query-gpu=memory.used,memory.free,utilization.gpu,temperature.gpu --format=csv,noheader,nounits 2>$null | Select-Object -First 1)
    if ($line) {
        $cells = @(($line -split ',') | ForEach-Object { $_.Trim() })
        if ($cells.Count -ge 4 -and $cells[0] -match '^\d+$' -and $cells[1] -match '^\d+$' -and $cells[2] -match '^\d+$' -and $cells[3] -match '^\d+$') {
            $seq++
            $row = "{0},{1},{2},{3},{4},{5}" -f $seq, (Get-Date -Format 'o'), $cells[0], $cells[1], $cells[2], $cells[3]
            [System.IO.File]::AppendAllText($Csv, $row + [Environment]::NewLine, (New-Object System.Text.ASCIIEncoding))
        }
    }
    Start-Sleep -Milliseconds $IntervalMs
}
