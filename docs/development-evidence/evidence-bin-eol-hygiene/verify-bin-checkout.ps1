<#
证据二进制 .bin 在 core.autocrlf=true 下的 checkout 安全验证（diagnostic / reproducible evidence）。

用途：证明 .gitattributes 的 `*.bin -text -diff` 规则能让全新 clone 的 1130 个已跟踪 .bin
逐字节还原，并保留「修复前 clone 会被改写」的复现对照。

流程：
  1. 记录仓库状态、core.autocrlf、已跟踪 .bin 清单与属性判定（check-attr 计数）。
  2. clone-before：从当前 HEAD 全新 clone（带 core.autocrlf=true），与工作树逐文件 sha256
     对比，复现修复前被 CRLF 改写的文件清单。
  3. clone-after：全新 clone 后仅在 clone 内复制 worktree 的 .gitattributes 并做一次性本地
     验证提交（该提交只存在于临时 clone，不进入主仓库），再强制全量重建工作树，逐文件
     sha256 对比，要求 1130/1130 一致。
  4. 在 clone-after 中对 5 个最小 .bin 做 `checkout -f` 往返 sha256 对照。

注意：本次任务提交后 HEAD 已包含修复规则，重跑时 clone-before 预期不再复现差异
（0 不一致），这是预期行为而非回归；clone-after 恒应为 1130/1130。

只读仓库对象，不修改主仓库工作树，不重写历史。
#>
param(
    [string]$RepoRoot = '',
    [string]$CloneRoot = '',
    [string]$OutputDir = '',
    [switch]$SkipBefore
)
$ErrorActionPreference = 'Stop'

if (-not $RepoRoot) {
    # <repo>/docs/development-evidence/evidence-bin-eol-hygiene -> <repo>
    $RepoRoot = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
}
if (-not $CloneRoot) { $CloneRoot = Join-Path $RepoRoot 'target/bin-eol-verify' }
if (-not $OutputDir) { $OutputDir = $PSScriptRoot }

$RepoRoot = (Resolve-Path -LiteralPath $RepoRoot).Path
New-Item -ItemType Directory -Force -Path $CloneRoot | Out-Null
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
$sha256 = [System.Security.Cryptography.SHA256]::Create()

function Write-Utf8NoBom([string]$path, [string]$text) {
    [System.IO.File]::WriteAllText($path, $text, $utf8NoBom)
}

function Get-Sha256([string]$path) {
    $fs = [System.IO.File]::OpenRead($path)
    try { $hash = $sha256.ComputeHash($fs) } finally { $fs.Dispose() }
    return ([System.BitConverter]::ToString($hash) -replace '-', '').ToLowerInvariant()
}

function Invoke-Git([string[]]$gitArgs) {
    $out = & git -C $RepoRoot @gitArgs 2>&1
    if ($LASTEXITCODE -ne 0) { throw "git $($gitArgs -join ' ') 失败：$out" }
    return @($out | ForEach-Object { [string]$_ })
}

function Get-RelativeToRepo([string]$path) {
    $prefix = $RepoRoot.TrimEnd('\') + '\'
    if ($path.StartsWith($prefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        return $path.Substring($prefix.Length).Replace('\', '/')
    }
    return $path
}

function Get-GitConfigValue([string]$dir, [string]$key) {
    $value = & git -C $dir config --get $key 2>$null
    if ($LASTEXITCODE -ne 0) { return $null }
    return [string]$value
}

function Remove-IfExists([string]$path) {
    if (-not (Test-Path -LiteralPath $path)) { return }
    # 仓库含 >260 字符路径，Windows PowerShell 5.1 的 Remove-Item -Recurse 会失败；
    # 先用 robocopy 空目录镜像（支持长路径），再用 rd 清掉空壳。
    $empty = Join-Path $CloneRoot '.empty-for-delete'
    New-Item -ItemType Directory -Force -Path $empty | Out-Null
    & robocopy $empty $path /MIR /NFL /NDL /NJH /NJS /NC /NS /R:1 /W:1 | Out-Null
    & cmd /c ('rd /s /q "{0}"' -f $path) | Out-Null
    if (Test-Path -LiteralPath $path) { throw "无法清理临时目录：$path" }
}

# 逐文件 sha256 对比；返回可直接序列化进证据的对象。
function Compare-BinSets([string]$referenceRoot, [string]$candidateRoot, [string[]]$paths) {
    $missing = @()
    $mismatched = @()
    $matched = 0
    foreach ($rel in $paths) {
        $refPath = Join-Path $referenceRoot $rel
        $candPath = Join-Path $candidateRoot $rel
        if (-not (Test-Path -LiteralPath $candPath)) { $missing += [string]$rel; continue }
        $refHash = Get-Sha256 $refPath
        $candHash = Get-Sha256 $candPath
        if ($refHash -eq $candHash) { $matched++ }
        else {
            $mismatched += [pscustomobject]@{
                path        = [string]$rel
                ref_sha256  = $refHash
                dest_sha256 = $candHash
                ref_size    = (Get-Item -LiteralPath $refPath).Length
                dest_size   = (Get-Item -LiteralPath $candPath).Length
            }
        }
    }
    return [pscustomobject]@{
        total            = $paths.Count
        matched          = $matched
        missing          = $missing.Count
        mismatched       = $mismatched.Count
        missing_files    = @($missing)
        mismatched_files = @($mismatched)
    }
}

function Get-PorcelainLines([string]$dir) {
    $out = & git -C $dir status --porcelain 2>&1
    if ($LASTEXITCODE -ne 0) { throw "git status 失败于 $dir" }
    return @($out | Where-Object { ([string]$_).Length -gt 0 } | ForEach-Object { [string]$_ })
}

# 全新 clone：显式 core.autocrlf=true（复现问题）与 core.longpaths=true（本仓库存在 >260 字符路径，
# 属既有环境前提，与 .bin 规则无关）；输出重定向到日志，避免进度条污染管道。
function Invoke-Clone([string]$destination, [string]$logName) {
    Remove-IfExists $destination
    $logOut = Join-Path $CloneRoot ($logName + '.out.log')
    $logErr = Join-Path $CloneRoot ($logName + '.err.log')
    $proc = Start-Process -FilePath 'git' -ArgumentList @(
        'clone', '--no-hardlinks',
        '--config', 'core.autocrlf=true',
        '--config', 'core.longpaths=true',
        $RepoRoot, $destination
    ) -RedirectStandardOutput $logOut -RedirectStandardError $logErr -NoNewWindow -Wait -PassThru
    if ($proc.ExitCode -ne 0) {
        throw ("clone 失败：{0}（stderr 日志 {1}）" -f $destination, (Get-RelativeToRepo $logErr))
    }
    return [pscustomobject]@{
        out_log = (Get-RelativeToRepo $logOut)
        err_log = (Get-RelativeToRepo $logErr)
    }
}

# ---------- 1. 仓库与属性基线 ----------
# @() 强制成数组：单行输出时 PowerShell 会把「函数返回的单元素数组」解包成字符串。
$repoHead = @(Invoke-Git @('rev-parse', 'HEAD'))[0]
$gitVersion = (& git --version) -join ''
$binPaths = Invoke-Git @('ls-files', '*.bin')
$allTracked = Invoke-Git @('ls-files')

$attrInput = Join-Path $CloneRoot 'bin-paths-lf.txt'
$attrOutput = Join-Path $CloneRoot 'check-attr-bins.txt'
Write-Utf8NoBom $attrInput (($binPaths -join "`n") + "`n")
# 用文件重定向喂 stdin/stdout：1130 条清单 + 2260 行输出直接用管道会死锁。
$attrProcess = Start-Process -FilePath 'git' -ArgumentList @('-C', $RepoRoot, 'check-attr', '--stdin', 'text', 'diff') `
    -RedirectStandardInput $attrInput -RedirectStandardOutput $attrOutput -NoNewWindow -Wait -PassThru
if ($attrProcess.ExitCode -ne 0) { throw 'git check-attr --stdin 失败' }
$attrLines = @([System.IO.File]::ReadAllLines($attrOutput) | Where-Object { $_.Length -gt 0 })
$textUnset = @($attrLines | Where-Object { $_ -match ': text: unset$' }).Count
$diffUnset = @($attrLines | Where-Object { $_ -match ': diff: unset$' }).Count
$attrUnexpected = @($attrLines | Where-Object { $_ -notmatch ': (text|diff): unset$' })

$attrSummary = [pscustomobject]@{
    repo_head              = $repoHead
    git_version            = $gitVersion
    core_autocrlf          = (Get-GitConfigValue $RepoRoot 'core.autocrlf')
    tracked_all            = $allTracked.Count
    tracked_bin            = $binPaths.Count
    check_attr_lines       = $attrLines.Count
    text_unset             = $textUnset
    diff_unset             = $diffUnset
    unexpected_attr_lines  = $attrUnexpected.Count
    gitattributes_sha256   = Get-Sha256 (Join-Path $RepoRoot '.gitattributes')
}
Write-Utf8NoBom (Join-Path $OutputDir 'attr-summary.json') ($attrSummary | ConvertTo-Json -Depth 5)
Write-Output ("attr: tracked_bin={0} text_unset={1} diff_unset={2} unexpected={3}" -f `
        $binPaths.Count, $textUnset, $diffUnset, $attrUnexpected.Count)

# ---------- 2. clone-before：修复前 HEAD 的全新 checkout ----------
$cloneBefore = Join-Path $CloneRoot 'clone-before'
$beforeResult = $null
if (-not $SkipBefore) {
    Write-Output 'step: clone-before 全新 clone ...'
    $beforeCloneLogs = Invoke-Clone $cloneBefore 'clone-before'
    $beforeCompare = Compare-BinSets -referenceRoot $RepoRoot -candidateRoot $cloneBefore -paths $binPaths
    $beforeResult = [pscustomobject]@{
        clone_path             = (Get-RelativeToRepo $cloneBefore)
        clone_head             = (& git -C $cloneBefore rev-parse HEAD)
        clone_autocrlf         = (Get-GitConfigValue $cloneBefore 'core.autocrlf')
        clone_longpaths        = (Get-GitConfigValue $cloneBefore 'core.longpaths')
        clone_logs             = $beforeCloneLogs
        gitattributes_sha256   = Get-Sha256 (Join-Path $cloneBefore '.gitattributes')
        clone_status_porcelain = @(Get-PorcelainLines $cloneBefore)
        compare                = $beforeCompare
    }
    Write-Utf8NoBom (Join-Path $OutputDir 'clone-before-compare.json') ($beforeResult | ConvertTo-Json -Depth 6)
    Write-Output ("clone-before: total={0} matched={1} mismatched={2} status_lines={3}" -f `
            $beforeCompare.total, $beforeCompare.matched, $beforeCompare.mismatched, $beforeResult.clone_status_porcelain.Count)
}

# ---------- 3. clone-after：修复规则落地后的全新 checkout ----------
$cloneAfter = Join-Path $CloneRoot 'clone-after'
Write-Output 'step: clone-after 全新 clone ...'
$afterCloneLogs = Invoke-Clone $cloneAfter 'clone-after'
# 修复尚未在主仓库提交：只在临时 clone 内复制 worktree 的 .gitattributes 并做一次性提交。
Copy-Item -LiteralPath (Join-Path $RepoRoot '.gitattributes') -Destination (Join-Path $cloneAfter '.gitattributes') -Force
& git -C $cloneAfter add .gitattributes | Out-Null
& git -C $cloneAfter -c user.name=bin-eol-verify -c user.email=verify@local `
    commit -m 'verify-only: .bin -text -diff（一次性验证提交，仅存在于临时 clone）' | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'clone-after 验证提交失败' }
$verifyCommit = (& git -C $cloneAfter rev-parse HEAD)
# 强制全量重建工作树：清空索引后 reset --hard，模拟「修复提交后的全新 checkout」。
& git -C $cloneAfter rm -r -q --cached . | Out-Null
& git -C $cloneAfter reset --hard HEAD | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'clone-after 全量重建失败' }

$afterCompare = Compare-BinSets -referenceRoot $RepoRoot -candidateRoot $cloneAfter -paths $binPaths
$afterResult = [pscustomobject]@{
    clone_path             = (Get-RelativeToRepo $cloneAfter)
    clone_head_before_ver  = (& git -C $cloneAfter rev-parse 'HEAD~1')
    verify_commit          = $verifyCommit
    verify_commit_subject  = (& git -C $cloneAfter log -1 --format=%s)
    clone_autocrlf         = (Get-GitConfigValue $cloneAfter 'core.autocrlf')
    clone_longpaths        = (Get-GitConfigValue $cloneAfter 'core.longpaths')
    clone_logs             = $afterCloneLogs
    gitattributes_sha256   = Get-Sha256 (Join-Path $cloneAfter '.gitattributes')
    clone_status_porcelain = @(Get-PorcelainLines $cloneAfter)
    compare                = $afterCompare
}
Write-Utf8NoBom (Join-Path $OutputDir 'clone-after-compare.json') ($afterResult | ConvertTo-Json -Depth 6)
Write-Output ("clone-after: total={0} matched={1} mismatched={2} status_lines={3}" -f `
        $afterCompare.total, $afterCompare.matched, $afterCompare.mismatched, $afterResult.clone_status_porcelain.Count)

# ---------- 4. 小 .bin 的 checkout -f 往返 ----------
# 选取：clone-before 中已复现差异的非空样本（最多 3 个）+ 未受影响的最小非空文件（补足 5 个），
# 同时覆盖「最坏样本」与「普通样本」，且不硬编码具体路径。
$sized = @()
foreach ($rel in $binPaths) {
    $sized += [pscustomobject]@{ path = [string]$rel; size = (Get-Item -LiteralPath (Join-Path $RepoRoot $rel)).Length }
}
$corruptedPaths = @()
if ($beforeResult) {
    $corruptedPaths = @($beforeResult.compare.mismatched_files | ForEach-Object { [string]$_.path })
}
$spot = @()
if ($corruptedPaths.Count -gt 0) {
    $spot += @($sized | Where-Object { $corruptedPaths -contains $_.path } | Sort-Object size, path | Select-Object -First 3)
}
$spot += @($sized | Where-Object {
        $_.size -gt 0 -and -not ($corruptedPaths -contains $_.path)
    } | Sort-Object size, path | Select-Object -First (5 - $spot.Count))
$spotPaths = @($spot | ForEach-Object { $_.path })
$spotBefore = @{}
foreach ($p in $spotPaths) { $spotBefore[$p] = Get-Sha256 (Join-Path $cloneAfter $p) }
& git -C $cloneAfter checkout -f -- @spotPaths | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'clone-after checkout -f 失败' }
$spotRows = @()
foreach ($p in $spotPaths) {
    $afterCheckout = Get-Sha256 (Join-Path $cloneAfter $p)
    $cloneBeforeSha = $null
    if ($beforeResult) { $cloneBeforeSha = Get-Sha256 (Join-Path $cloneBefore $p) }
    $spotRows += [pscustomobject]@{
        path                  = $p
        size                  = (Get-Item -LiteralPath (Join-Path $RepoRoot $p)).Length
        repo_worktree_sha256  = Get-Sha256 (Join-Path $RepoRoot $p)
        clone_before_sha256   = $cloneBeforeSha
        clone_after_sha256    = $spotBefore[$p]
        after_checkout_sha256 = $afterCheckout
        roundtrip_identical   = ($spotBefore[$p] -eq $afterCheckout)
    }
}
$spotResult = [pscustomobject]@{
    clone_after_path = (Get-RelativeToRepo $cloneAfter)
    spot_count       = $spotRows.Count
    spot_corrupted_from_clone_before = @($spotRows | Where-Object { $_.clone_before_sha256 -ne $_.repo_worktree_sha256 }).Count
    roundtrip_all_identical = (@($spotRows | Where-Object { -not $_.roundtrip_identical }).Count -eq 0)
    rows             = $spotRows
}
Write-Utf8NoBom (Join-Path $OutputDir 'spot-checkout-roundtrip.json') ($spotResult | ConvertTo-Json -Depth 6)

# ---------- 5. 汇总 ----------
$beforeCompareForSummary = $null
if ($beforeResult) { $beforeCompareForSummary = $beforeResult.compare }
$summary = [pscustomobject]@{
    generated_at        = (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ')
    repo_head           = $repoHead
    core_autocrlf       = $attrSummary.core_autocrlf
    tracked_bin         = $binPaths.Count
    attr_text_unset     = $textUnset
    attr_diff_unset     = $diffUnset
    clone_before        = $beforeCompareForSummary
    clone_after         = $afterResult.compare
    spot_roundtrip_ok   = $spotResult.roundtrip_all_identical
}
Write-Utf8NoBom (Join-Path $OutputDir 'summary.json') ($summary | ConvertTo-Json -Depth 6)

Write-Output ('summary: bins={0} attr(text_unset={1}, diff_unset={2}) before_mismatch={3} after_mismatch={4} spot_ok={5}' -f `
        $binPaths.Count, $textUnset, $diffUnset,
    $(if ($beforeResult) { $beforeResult.compare.mismatched } else { 'skipped' }),
    $afterCompare.mismatched, $spotResult.roundtrip_all_identical)

if ($afterCompare.mismatched -ne 0 -or $afterCompare.missing -ne 0) { exit 2 }
if (-not $spotResult.roundtrip_all_identical) { exit 3 }
exit 0
