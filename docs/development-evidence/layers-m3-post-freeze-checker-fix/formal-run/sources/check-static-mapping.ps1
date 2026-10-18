$ErrorActionPreference = 'Stop'
$scriptPath = 'docs/development-evidence/layers-m3-inventory-derived-correction/check-post-freeze-blobs.ps1'
$baseRevision = 'da10228b0e4776fc68532503521fc676c00ca7fc'
$oldFailureRoot = 'target/normify-evidence/2026-10-19-layers-m3-inventory-derived-correction/check-14-post-freeze-commit'

$tokens = $null
$parseErrors = $null
$ast = [Management.Automation.Language.Parser]::ParseFile(
    (Resolve-Path -LiteralPath $scriptPath),
    [ref]$tokens,
    [ref]$parseErrors
)
if ($parseErrors.Count -ne 0) { throw '修正脚本存在PowerShell语法错误' }
$functions = @($ast.FindAll({
    param($node)
    $node -is [Management.Automation.Language.FunctionDefinitionAst] -and
        $node.Name -ceq 'Join-RunArchivePath'
}, $true))
if ($functions.Count -ne 1) { throw '归档路径函数不唯一' }
Invoke-Expression $functions[0].Extent.Text

$cases = @(
    @{named='registration.json';empty='registration.json'},
    @{named='started.json';empty='started.json'},
    @{named='run.json';empty='result.json'},
    @{named='stdout.log';empty='stdout.log'},
    @{named='stderr.log';empty='stderr.log'}
)
$emptyExpected = @(
    'root/registration.json',
    'root/started.json',
    'root/result.json',
    'root/stdout.log',
    'root/stderr.log'
)
$namedExpected = @(
    'root/sample.registration.json',
    'root/sample.started.json',
    'root/sample.run.json',
    'root/sample.stdout.log',
    'root/sample.stderr.log'
)
$emptyActual = @($cases | ForEach-Object { Join-RunArchivePath 'root' '' $_.named $_.empty })
$namedActual = @($cases | ForEach-Object { Join-RunArchivePath 'root' 'sample' $_.named $_.empty })
if ((Compare-Object $emptyExpected $emptyActual -SyncWindow 0).Count -ne 0) {
    throw '空名称归档路径错误'
}
if ((Compare-Object $namedExpected $namedActual -SyncWindow 0).Count -ne 0) {
    throw '普通名称归档路径回归'
}

$current = Get-Content -LiteralPath $scriptPath -Raw
$correctCall = 'Add-RunMappings ''post-freeze-index-retry-01'' '''' "$postRoot/index-extra"'
$wrongCall = 'Add-RunMappings ''post-freeze-index'' '''' "$postRoot/index-extra"'
$resultMapping = "(Join-RunArchivePath `$DestinationRoot `$Name 'run.json' 'result.json')"
if (-not $current.Contains($correctCall) -or $current.Contains($wrongCall)) {
    throw 'index-extra运行映射错误'
}
if (-not $current.Contains($resultMapping)) {
    throw '实际运行结果映射没有保留普通run后缀'
}

$old = @(& git show "$baseRevision`:$scriptPath") -join "`n"
if ($LASTEXITCODE -ne 0) { throw '无法读取原提交脚本对象' }
if (-not $old.Contains($wrongCall) -or $old.Contains($correctCall)) {
    throw '原提交缺陷身份不符'
}
$oldResult = Get-Content -LiteralPath "$oldFailureRoot/result.json" -Raw | ConvertFrom-Json
$oldError = Get-Content -LiteralPath "$oldFailureRoot/stderr.log" -Raw
if ($oldResult.exit_code -ne 1 -or -not $oldError.Contains('/.registration.json')) {
    throw '原失败证据不符'
}

[ordered]@{
    schema = 1
    empty_name_cases = $emptyActual.Count
    named_cases = $namedActual.Count
    index_extra_run = 'post-freeze-index-retry-01'
    old_commit = $baseRevision
    old_failure_exit_code = $oldResult.exit_code
    temporary_fix_was_reverted_before_this_task = $true
    result = 'pass'
} | ConvertTo-Json
