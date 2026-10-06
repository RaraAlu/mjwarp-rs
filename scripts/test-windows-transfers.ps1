[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
if (-not $IsWindows) { throw '此脚本只验证Windows' }
$root = Split-Path -Parent $PSScriptRoot
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
$vs = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $vs) { throw '需要MSVC与Windows SDK' }
$devcmd = Join-Path $vs 'Common7\Tools\VsDevCmd.bat'
$environment = & $env:ComSpec /d /c "call `"$devcmd`" -no_logo -arch=x64 -host_arch=x64 && set"
if ($LASTEXITCODE -ne 0) { throw 'MSVC环境初始化失败' }
foreach ($line in $environment) {
    if ($line -match '^([^=]+)=(.*)$') {
        [Environment]::SetEnvironmentVariable($Matches[1], $Matches[2], 'Process')
    }
}

# 每轮保留独立日志与进程记录。
$stage = Join-Path $root ('target\transfer-probe\' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $stage | Out-Null
$executable = Join-Path $stage 'batch-transfer.exe'
$report = [ordered]@{
    passed = $false
    platform = [System.Environment]::OSVersion.VersionString
    toolchain = 'stable-x86_64-pc-windows-msvc'
    features = 'cuda-probe'
    restrictedPath = "$env:SystemRoot\System32;$env:SystemRoot"
    executable = $executable
    nvrtcInvisible = $false
    gpu = $null
    tests = $null
    exitCode = $null
}
Push-Location $root
try {
    $lines = & cargo +stable-x86_64-pc-windows-msvc test --locked --features cuda-probe --test batch_transfer --no-run --message-format=json
    if ($LASTEXITCODE -ne 0) { throw '传输测试构建失败' }
    $lines | Set-Content -LiteralPath (Join-Path $stage 'build.jsonl') -Encoding utf8
    $messages = @($lines | ForEach-Object { $_ | ConvertFrom-Json })
    $artifacts = @($messages | Where-Object { $_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'batch_transfer' -and $_.executable })
    if ($artifacts.Count -ne 1) { throw '测试执行文件不唯一' }
    Copy-Item -LiteralPath $artifacts[0].executable -Destination $executable
    $report.gpu = (& nvidia-smi --query-gpu=name,driver_version --format=csv,noheader) -join '; '
    if ($LASTEXITCODE -ne 0) { throw 'GPU信息读取失败' }

    $start = [System.Diagnostics.ProcessStartInfo]::new()
    $start.FileName = $executable
    $start.WorkingDirectory = $stage
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $start.StandardOutputEncoding = [System.Text.Encoding]::UTF8
    $start.StandardErrorEncoding = [System.Text.Encoding]::UTF8
    foreach ($argument in @('--ignored', '--test-threads=1', '--nocapture')) { $start.ArgumentList.Add($argument) }
    $start.Environment['PATH'] = $report.restrictedPath
    foreach ($name in @($start.Environment.Keys)) {
        if ($name -match '^(CUDA_|CUDAToolkit|LLVM_|TRACEL_)') {
            $start.Environment.Remove($name) | Out-Null
        }
    }
    $start.Environment['MJWARP_TRANSFER_DRIVER_ONLY'] = '1'
    $process = [System.Diagnostics.Process]::Start($start)
    try {
        $stdout = $process.StandardOutput.ReadToEndAsync()
        $stderr = $process.StandardError.ReadToEndAsync()
        $process.WaitForExit()
        $output = $stdout.GetAwaiter().GetResult()
        $errorOutput = $stderr.GetAwaiter().GetResult()
        $report.exitCode = $process.ExitCode
    } finally {
        $process.Dispose()
    }
    $output | Set-Content -LiteralPath (Join-Path $stage 'stdout.log') -Encoding utf8
    $errorOutput | Set-Content -LiteralPath (Join-Path $stage 'stderr.log') -Encoding utf8
    Write-Output $output
    if ($report.exitCode -ne 0) { throw "仅驱动传输失败：$($report.exitCode)" }
    $report.nvrtcInvisible = $output.Contains('driver_only: NVRTC invisible')
    if (-not $report.nvrtcInvisible) { throw '缺少NVRTC不可见证据' }
    if ($output -notmatch 'test result: ok\. (\d+) passed; 0 failed; 0 ignored;') { throw '缺少完整GPU通过记录' }
    $report.tests = [int]$Matches[1]
    if ($report.tests -eq 0) { throw '零用例不构成验收' }
    $report.passed = $true
} finally {
    $report | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $stage 'report.json') -Encoding utf8
    Pop-Location
    Write-Output "证据：$stage"
}
