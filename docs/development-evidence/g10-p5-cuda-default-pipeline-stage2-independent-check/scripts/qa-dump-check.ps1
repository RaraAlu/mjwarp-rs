# QA independent check: dump run1/run2 normalized determinism + record B recompute (ASCII only for PS5.1)
$ErrorActionPreference = 'Stop'
$root = 'D:\Rust\mjwarp-rs'
$window = Join-Path $root 'target\stage2-impl\window'
$dumpDir = Join-Path $window 'product'
$bins = Join-Path $root 'docs\development-evidence\g10-p5-gpu-kernels\gpu-window\cases'
$outDir = Join-Path $root 'target\qa-stage2\qa-dump-check'
New-Item -ItemType Directory -Force $outDir | Out-Null
$FOUR_ULP = 4.0 * (1.0 / 8388608.0)

function F32([double]$value) { return [single]$value }
function Bits([single]$value) { return [BitConverter]::ToInt32([BitConverter]::GetBytes($value), 0) }
function SortedBits([single]$value) {
    $b = Bits $value
    if ($b -lt 0) { return -[int64]($b -band 0x7fffffff) }
    return [int64]$b
}
function UlpDist([single]$a, [single]$b) {
    return [math]::Abs((SortedBits $a) - (SortedBits $b))
}
function UlpAt([single]$value) {
    $bits = Bits $value
    $next = [BitConverter]::ToSingle([BitConverter]::GetBytes([int]($bits + 1)), 0)
    return [single]($next - $value)
}
function VecScale([single[]]$values) {
    $scale = [single]0
    foreach ($v in $values) { $a = [math]::Abs($v); if ($a -gt $scale) { $scale = [single]$a } }
    if ($scale -eq 0) { return [single]1.0 }
    return $scale
}
function ReadF32([string]$path) {
    $bytes = [System.IO.File]::ReadAllBytes($path)
    $n = [int]($bytes.Length / 4)
    $out = New-Object 'single[]' $n
    for ($i = 0; $i -lt $n; $i++) { $out[$i] = [BitConverter]::ToSingle($bytes, $i * 4) }
    return $out
}
function ReadI32([string]$path) {
    $bytes = [System.IO.File]::ReadAllBytes($path)
    $n = [int]($bytes.Length / 4)
    $out = New-Object 'int[]' $n
    for ($i = 0; $i -lt $n; $i++) { $out[$i] = [BitConverter]::ToInt32($bytes, $i * 4) }
    return $out
}
function RowsFromDump($run) {
    $n = [int]$run.nacon
    $dist = $run.dist; $pos = $run.pos; $frame = $run.frame; $wid = $run.worldid
    $rows = New-Object System.Collections.ArrayList
    for ($i = 0; $i -lt $n; $i++) {
        $p = @((F32 $pos[$i * 3]), (F32 $pos[$i * 3 + 1]), (F32 $pos[$i * 3 + 2]))
        $f = @()
        for ($c = 0; $c -lt 9; $c++) { $f += (F32 $frame[$i * 9 + $c]) }
        $null = $rows.Add([pscustomobject]@{ worldid = [int]$wid[$i]; geom = 0; dist = (F32 $dist[$i]); pos = $p; frame = $f })
    }
    return [pscustomobject]@{ nacon = $n; nccd = [int]$run.nccd; overflow = @($run.overflow | ForEach-Object { [uint32]$_ }); rows = $rows }
}
function RowsFromOut1([string]$dir) {
    $nacon = (ReadI32 (Join-Path $dir 'nacon.bin'))[0]
    $nccd = (ReadI32 (Join-Path $dir 'nccd.bin'))[0]
    $n = [math]::Max(0, $nacon)
    $dist = ReadF32 (Join-Path $dir 'dist.bin')
    $pos = ReadF32 (Join-Path $dir 'pos.bin')
    $frame = ReadF32 (Join-Path $dir 'frame.bin')
    $wid = ReadI32 (Join-Path $dir 'worldid.bin')
    $ov = @(ReadI32 (Join-Path $dir 'overflow.bin') | ForEach-Object { [uint32]$_ })
    $rows = New-Object System.Collections.ArrayList
    for ($i = 0; $i -lt $n; $i++) {
        $p = @($pos[$i * 3], $pos[$i * 3 + 1], $pos[$i * 3 + 2])
        $f = @()
        for ($c = 0; $c -lt 9; $c++) { $f += $frame[$i * 9 + $c] }
        $null = $rows.Add([pscustomobject]@{ worldid = [int]$wid[$i]; geom = 0; dist = [single]$dist[$i]; pos = $p; frame = $f })
    }
    return [pscustomobject]@{ nacon = $nacon; nccd = $nccd; overflow = $ov; rows = $rows }
}
function SortRows($rows) {
    $arr = @($rows)
    $arr = $arr | Sort-Object -Property @{ Expression = { [double]$_.worldid } }, @{ Expression = { [double]$_.dist } }, @{ Expression = { [double]$_.pos[0] } }, @{ Expression = { [double]$_.pos[1] } }, @{ Expression = { [double]$_.pos[2] } }
    return , @($arr)
}
function CompareRows($dev, $ref, [string]$label) {
    $out = [ordered]@{ label = $label; total = 0; bit_equal = 0; max_dist_ulp = 0; max_pos_ulp = 0.0; max_frame_abs = 0.0; failures = @() }
    $fail = New-Object System.Collections.ArrayList
    if ($dev.nacon -ne $dev.rows.Count) { $null = $fail.Add("device nacon=$($dev.nacon) != rows $($dev.rows.Count)") }
    if ($ref.overflow.Count -gt 0) {
        if ($dev.overflow.Count -ne $ref.overflow.Count) { $null = $fail.Add("overflow len dev=$($dev.overflow.Count) ref=$($ref.overflow.Count)") }
        else { for ($w = 0; $w -lt $dev.overflow.Count; $w++) { if ($dev.overflow[$w] -ne $ref.overflow[$w]) { $null = $fail.Add("world $w overflow dev=$($dev.overflow[$w]) ref=$($ref.overflow[$w])") } } }
    }
    $devSorted = SortRows $dev.rows
    $refSorted = SortRows $ref.rows
    if ($devSorted.Count -ne $refSorted.Count) { $null = $fail.Add("row count dev=$($devSorted.Count) ref=$($refSorted.Count)") }
    $pairs = [math]::Min($devSorted.Count, $refSorted.Count)
    for ($i = 0; $i -lt $pairs; $i++) {
        $d = $devSorted[$i]; $r = $refSorted[$i]
        if ($d.worldid -ne $r.worldid) { $null = $fail.Add("sorted row $i worldid $($d.worldid) vs $($r.worldid)") }
        $out.total += 1
        if ((Bits $d.dist) -eq (Bits $r.dist)) { $out.bit_equal += 1 }
        else {
            $ulp = UlpDist $d.dist $r.dist
            if ($ulp -gt $out.max_dist_ulp) { $out.max_dist_ulp = $ulp }
            $ok = if ([double]$r.dist -eq 0) { [math]::Abs([double]$d.dist - [double]$r.dist) -le [double]$FOUR_ULP } else { $ulp -le 4 }
            if (-not $ok) { $null = $fail.Add("world $($r.worldid) dist over 4 ULP: dev=$($d.dist) ref=$($r.dist) ulp=$ulp") }
        }
        $scale = VecScale $r.pos
        $bound = 4.0 * [double](UlpAt $scale)
        for ($a = 0; $a -lt 3; $a++) {
            $out.total += 1
            if ((Bits $d.pos[$a]) -eq (Bits $r.pos[$a])) { $out.bit_equal += 1 }
            else {
                $diff = [math]::Abs([double]$d.pos[$a] - [double]$r.pos[$a])
                $ratio = $diff / $bound
                if ($ratio -gt $out.max_pos_ulp) { $out.max_pos_ulp = $ratio }
                if ($diff -gt $bound) { $null = $fail.Add("world $($r.worldid) pos[$a] |dev-ref|=$diff > 4ULP(scale=$scale)=$bound") }
            }
        }
        for ($c = 0; $c -lt 9; $c++) {
            $out.total += 1
            if ((Bits $d.frame[$c]) -eq (Bits $r.frame[$c])) { $out.bit_equal += 1 }
            else {
                $diff = [math]::Abs([double]$d.frame[$c] - [double]$r.frame[$c])
                if ($diff -gt $out.max_frame_abs) { $out.max_frame_abs = $diff }
                if ($diff -gt [double]$FOUR_ULP) { $null = $fail.Add("world $($r.worldid) frame[$c] |dev-ref|=$diff > 4ULP@1.0=$FOUR_ULP") }
            }
        }
    }
    $out.failures = @($fail)
    return $out
}

$cases = @('cylinder-box', 'ellipsoid-box', 'mesh-mesh', 'convex-mesh', 'degenerate-mesh', 'sphere-box-shallow', 'sphere-box-deep', 'capsule-box', 'sdf-box')
$report = [ordered]@{ generated_at = (Get-Date -Format o); dump_two_run = @(); record_b = @() }

Write-Host '=== dump run1 vs run2 (normalized by worldid + G5 sort key)'
foreach ($case in $cases) {
    $kernels = if ($case -eq 'sdf-box') { @('Sdf') } else { @('Contact', 'ContactMulti') }
    foreach ($kernel in $kernels) {
        $path = Join-Path $dumpDir "$case.$kernel.json"
        $j = Get-Content -LiteralPath $path -Raw -Encoding UTF8 | ConvertFrom-Json
        $r1 = RowsFromDump $j.run1; $r2 = RowsFromDump $j.run2
        $a = SortRows $r1.rows; $b = SortRows $r2.rows
        $equal = $true; $why = ''
        if ($r1.nacon -ne $r2.nacon -or $r1.nccd -ne $r2.nccd) { $equal = $false; $why = 'nacon/nccd' }
        elseif ((@($r1.overflow) -join ',') -ne (@($r2.overflow) -join ',')) { $equal = $false; $why = 'overflow' }
        elseif ($a.Count -ne $b.Count) { $equal = $false; $why = 'rows count' }
        else {
            for ($i = 0; $i -lt $a.Count; $i++) {
                if ($a[$i].worldid -ne $b[$i].worldid -or (Bits $a[$i].dist) -ne (Bits $b[$i].dist)) { $equal = $false; $why = "row $i dist/world"; break }
                for ($x = 0; $x -lt 3; $x++) { if ((Bits $a[$i].pos[$x]) -ne (Bits $b[$i].pos[$x])) { $equal = $false; $why = "row $i pos$x"; break } }
                if (-not $equal) { break }
                for ($x = 0; $x -lt 9; $x++) { if ((Bits $a[$i].frame[$x]) -ne (Bits $b[$i].frame[$x])) { $equal = $false; $why = "row $i frame$x"; break } }
                if (-not $equal) { break }
            }
        }
        Write-Host ("{0,-18} {1,-12} run1_eq_run2_normalized={2} {3}" -f $case, $kernel, $equal, $why)
        $report.dump_two_run += [ordered]@{ case = $case; kernel = $kernel; equal_normalized = $equal; detail = $why; run1_worldid = @($j.run1.worldid); run2_worldid = @($j.run2.worldid) }
    }
}

Write-Host ''
Write-Host '=== record B self-compute (device dump run1 vs frozen out1, G5 criterion)'
foreach ($case in $cases) {
    if ($case -eq 'convex-mesh' -or $case -eq 'degenerate-mesh') { Write-Host "$case : no frozen reference (skip)"; continue }
    $kernel = if ($case -eq 'sdf-box') { 'Sdf' } else { 'Contact' }
    $j = Get-Content -LiteralPath (Join-Path $dumpDir "$case.$kernel.json") -Raw -Encoding UTF8 | ConvertFrom-Json
    $dev = RowsFromDump $j.run1
    $ref = RowsFromOut1 (Join-Path $bins "$case\out1")
    $cmp = CompareRows $dev $ref "$case $kernel"
    Write-Host ("{0,-18} total={1} bit_equal={2} max_dist_ulp={3} max_pos_ulp={4:F3} max_frame_abs={5:E3} failures={6}" -f $case, $cmp.total, $cmp.bit_equal, $cmp.max_dist_ulp, $cmp.max_pos_ulp, $cmp.max_frame_abs, $cmp.failures.Count)
    foreach ($fl in $cmp.failures) { Write-Host "   FAIL $fl" }
    $report.record_b += $cmp
}
$report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $outDir 'qa-dump-check.json') -Encoding utf8
Write-Host ''
Write-Host "report -> $(Join-Path $outDir 'qa-dump-check.json')"


