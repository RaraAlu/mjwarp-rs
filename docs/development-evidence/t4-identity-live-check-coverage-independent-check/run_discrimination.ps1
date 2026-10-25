# QA reverse-discrimination runner for the T4 identity live check.
# Runs the product test target against self-built temporary fixture copies
# (target/qa-t4-check/, repo fixture untouched) via the test-only seam
# T4_IDENTITY_LIVE_FIXTURE, then verifies the diagnostics carry path/expected/actual.
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
$OutputEncoding = [Text.UTF8Encoding]::new($false)
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$repo = 'D:\Rust\mjwarp-rs'
$evidence = Join-Path $repo 'docs\development-evidence\t4-identity-live-check-coverage-independent-check'
$logs = Join-Path $evidence 'logs'
$tmp = Join-Path $repo 'target\qa-t4-check'
New-Item -ItemType Directory -Force -Path $logs, $tmp | Out-Null
Remove-Item Env:T4_IDENTITY_LIVE_FIXTURE -ErrorAction SilentlyContinue

$results = [ordered]@{}
function Run-Case {
    param([string]$Name, [string]$FixturePath)
    if ($FixturePath -ne '') {
        $env:T4_IDENTITY_LIVE_FIXTURE = $FixturePath
    } else {
        Remove-Item Env:T4_IDENTITY_LIVE_FIXTURE -ErrorAction SilentlyContinue
    }
    $log = Join-Path $logs ("{0}.log" -f $Name)
    & cargo test --locked --test t4_identity_live_check -- --test-threads=1 *> $log
    $exit = $LASTEXITCODE
    $text = Get-Content $log -Raw
    $entry = [ordered]@{
        fixture = $(if ($FixturePath -eq '') { '(repo fixture; seam unset)' } else { $FixturePath })
        exit = $exit
        passed = [bool]($text -match 'test result: ok')
        checks = [ordered]@{
            mentions_t4_identity = [bool]($text -match 'T4 身份')
            has_expected_actual = ($text -match 'expected=') -and ($text -match 'actual=')
            has_path_set_incomplete = [bool]($text -match '路径集合不完整')
            has_count_mismatch = [bool]($text -match '条数应为')
            has_missing_or_unexpected = ($text -match '缺少') -or ($text -match '多出')
            mentions_tampered_sha = [bool]($text -match '18cc1fd3')
            mentions_sleep_renamed = [bool]($text -match 'sleep_renamed')
        }
        log = "logs/$Name.log"
    }
    $results[$Name] = $entry
    Remove-Item Env:T4_IDENTITY_LIVE_FIXTURE -ErrorAction SilentlyContinue
    Write-Host ("{0}: exit={1} passed={2}" -f $Name, $exit, $entry.passed)
}

# 0) seam-free control + test listing
Run-Case -Name 'normal-repo-fixture' -FixturePath ''
& cargo test --locked --test t4_identity_live_check -- --list *> (Join-Path $logs 'list.log')
Write-Host "list exit=$LASTEXITCODE"
$listText = Get-Content (Join-Path $logs 'list.log') -Raw
$results['list'] = [ordered]@{
    exit = $LASTEXITCODE
    contains_live_product_sources = [bool]($listText -match 'live_product_sources_match_fixture')
    contains_sha_selftest = [bool]($listText -match 'sha256_and_lf_normalization_match_known_vectors')
    log = 'logs/list.log'
}

# 1) tampered expected sha (must fail, diagnostic path/expected/actual)
Run-Case -Name 'tamper-sha' -FixturePath (Join-Path $tmp 'tampered-sha.json')
# 2) tampered path set (rename -> missing/unexpected; must fail)
Run-Case -Name 'tamper-path-rename' -FixturePath (Join-Path $tmp 'tampered-path-rename.json')
# 3) tampered path set (remove -> count 19; must fail)
Run-Case -Name 'tamper-path-remove' -FixturePath (Join-Path $tmp 'tampered-path-remove.json')
# 4) control copy through the seam (must pass: proves seam wiring is the only difference)
Run-Case -Name 'normal-copy' -FixturePath (Join-Path $tmp 'normal-copy.json')
# 5) restored control (must pass again)
Run-Case -Name 'normal-restored' -FixturePath ''

$repoFixtureStatus = @(& git -C $repo status --porcelain -- fixtures/precision/t4-current-kernels.json)
$results['repo_fixture_modified'] = [bool]($repoFixtureStatus.Count -gt 0)
$results['summary'] = [ordered]@{
    expected = [ordered]@{
        'normal-repo-fixture' = 'exit 0 (2 passed)'
        'tamper-sha' = 'exit 101 (1 failed); path/expected/actual in diagnostic'
        'tamper-path-rename' = 'exit 101; missing/unexpected path in diagnostic'
        'tamper-path-remove' = 'exit 101; count assertion'
        'normal-copy' = 'exit 0 (2 passed)'
        'normal-restored' = 'exit 0 (2 passed)'
    }
}
$results | ConvertTo-Json -Depth 8 | Out-File -Encoding utf8 (Join-Path $evidence 'qa-discrimination.json')
Write-Host 'DISCRIMINATION_RUNNER_DONE'
