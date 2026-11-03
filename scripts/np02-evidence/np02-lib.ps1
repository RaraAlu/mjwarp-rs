# NP02 只读扫描库：路径/哈希工具、回执实体复核、产品源扫描、编译点收集。
# 由 np02-evidence.ps1 点源加载；所有函数显式接收 $ProductRoot，无全局状态。

function Get-RepoPath([string]$ProductRoot, [string]$rel) {
    $p = Join-Path $ProductRoot $rel
    if (-not (Test-Path -LiteralPath $p)) { throw "缺少输入：$rel（$p）" }
    return (Resolve-Path -LiteralPath $p).Path
}
function Get-RepoRel([string]$ProductRoot, [string]$full) {
    return ($full.Replace($ProductRoot + '\', '')).Replace('\', '/')
}
function Get-Sha256Bytes([byte[]]$bytes) {
    $sha = [System.Security.Cryptography.SHA256]::Create()
    return (($sha.ComputeHash($bytes)) | ForEach-Object { $_.ToString('x2') }) -join ''
}
function Get-Sha256Text([string]$text) { return (Get-Sha256Bytes ([System.Text.Encoding]::UTF8.GetBytes($text))) }
function Get-Sha256File([string]$path) { return (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLower() }

# 逐份复核回执文件哈希，并在仓内核对 <entry>-<ptxSha256>.ptx 实体哈希。
function Invoke-ReceiptAudit([string]$ProductRoot, $ReceiptsIndex) {
    $rows = @()
    $verified = 0
    $failed = @()
    foreach ($r in $ReceiptsIndex.receipts) {
        $full = Join-Path $ProductRoot $r.path
        if (-not (Test-Path -LiteralPath $full)) { $failed += "$($r.path)（缺失）"; continue }
        $actual = Get-Sha256File $full
        $ok = ($actual -eq $r.fileSha256)
        if ($ok) { $verified++ } else { $failed += "$($r.path)（哈希不符）" }
        $ptxPath = Join-Path (Split-Path -Parent $full) ("{0}-{1}.ptx" -f $r.entry, $r.ptxSha256)
        $ptxOk = $false
        $ptxPathRel = ''
        if (Test-Path -LiteralPath $ptxPath) {
            $ptxHash = Get-Sha256File $ptxPath
            $ptxOk = ($ptxHash -eq $r.ptxSha256)
            $ptxPathRel = Get-RepoRel $ProductRoot $ptxPath
        }
        $cuPath = Join-Path (Split-Path -Parent $full) ("{0}-{1}.cu" -f $r.entry, $r.ptxSha256)
        $cuPathRel = if (Test-Path -LiteralPath $cuPath) { Get-RepoRel $ProductRoot $cuPath } else { '' }
        $cuHashOk = if ($cuPathRel) { (Get-Sha256File $cuPath) -eq $r.sourceSha256 } else { $false }
        $rows += [pscustomobject]@{
            path = $r.path
            entry = $r.entry
            ptxSha256 = $r.ptxSha256
            sourceSha256 = $r.sourceSha256
            options = @($r.options)
            fmad = $r.fmad
            useFastMath = $r.useFastMath
            nvrtcVersion = @($r.nvrtcVersion)
            precisionGate = $r.precisionGate
            fileHashOk = $ok
            ptxEntity = $ptxPathRel
            ptxEntityOk = $ptxOk
            cuEntity = $cuPathRel
            cuEntityHashOk = $cuHashOk
        }
    }
    return [pscustomobject]@{ Rows = $rows; Verified = $verified; Failed = $failed }
}

# 扫描 src/**/*.rs 的 CUDA 源常量（内联 r#"..."# 与 include_str!("*.cu")），提取内核名与源文本哈希。
function Get-SourceUnits([string]$ProductRoot, [hashtable]$ExcludedFiles) {
    $constPattern = [regex]'(?s)\bconst\s+(?<name>[A-Z0-9_]+)\s*:\s*&str\s*=\s*(?:r#"(?<body>.*?)"#\s*;|include_str!\(\s*"(?<inc>[^"]+)"\s*\)\s*;)'
    $kernelPattern = [regex]'__global__\s+void\s+(?<name>\w+)'
    $units = @()
    $files = Get-ChildItem -LiteralPath (Join-Path $ProductRoot 'src') -Recurse -File -Filter *.rs |
        Where-Object { $_.FullName -notmatch '\\tests\\' -and $_.Name -ne 'tests.rs' }
    foreach ($file in $files) {
        $relFile = Get-RepoRel $ProductRoot $file.FullName
        if ($ExcludedFiles.ContainsKey($relFile)) { continue }
        $text = [System.IO.File]::ReadAllText($file.FullName, [System.Text.Encoding]::UTF8)
        foreach ($m in $constPattern.Matches($text)) {
            $body = $m.Groups['body'].Value
            $inc = $m.Groups['inc'].Value
            $sourceText = ''
            $sourcePath = $relFile
            if ($body.Length -gt 0) {
                $sourceText = $body
            } elseif ($inc.Length -gt 0) {
                $incFull = Join-Path $file.DirectoryName $inc
                if (-not (Test-Path -LiteralPath $incFull)) { continue }
                $sourceText = [System.IO.File]::ReadAllText($incFull, [System.Text.Encoding]::UTF8)
                $sourcePath = Get-RepoRel $ProductRoot $incFull
            }
            if ([string]::IsNullOrEmpty($sourceText)) { continue }
            # rustc 读取源码时把 CRLF 规范化为 LF（字符串字面量同样生效），捕获源哈希按 LF 口径；
            # 工作区可能因 core.autocrlf 检出为 CRLF，这里统一按 LF 规范化后再比对。
            $normalized = $sourceText -replace "`r`n", "`n"
            $kernels = @($kernelPattern.Matches($normalized) | ForEach-Object { $_.Groups['name'].Value })
            if ($kernels.Count -eq 0) { continue }
            $units += [pscustomobject]@{
                relFile = $relFile
                path = $sourcePath
                const = $m.Groups['name'].Value
                sha256 = Get-Sha256Text $normalized
                text = $normalized
                kernels = $kernels
            }
        }
    }
    return $units
}

# 收集产品编译点（SynchronousKernel::compile 的入口名），用于补充派生/动态源内核。
function Get-CompileNames([string]$ProductRoot, [hashtable]$ExcludedFiles, [string[]]$ExcludedNames) {
    $names = @()
    $files = Get-ChildItem -LiteralPath (Join-Path $ProductRoot 'src') -Recurse -File -Filter *.rs |
        Where-Object { $_.FullName -notmatch '\\tests\\' -and $_.Name -ne 'tests.rs' -and -not $ExcludedFiles.ContainsKey((Get-RepoRel $ProductRoot $_.FullName)) }
    foreach ($file in $files) {
        $text = [System.IO.File]::ReadAllText($file.FullName, [System.Text.Encoding]::UTF8)
        foreach ($m in [regex]::Matches($text, '(?s)SynchronousKernel::compile\((?<args>.*?)\)\s*(\?|\.map_err|;|\.unwrap)')) {
            $lits = [regex]::Matches($m.Groups['args'].Value, '"(?<n>[A-Za-z0-9_]+)"')
            if ($lits.Count -gt 0) { $names += $lits[$lits.Count - 1].Groups['n'].Value }
        }
    }
    return @($names | Sort-Object -Unique | Where-Object { $_ -notin $ExcludedNames })
}
