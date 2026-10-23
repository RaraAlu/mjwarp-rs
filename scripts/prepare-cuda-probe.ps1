[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$destination = Join-Path $root 'target\toolchains\cuda-12.8.1'
$downloads = Join-Path $destination 'downloads'
New-Item -ItemType Directory -Path $downloads -Force | Out-Null

# 只安装项目私有文件。
# 脚本不改全局PATH，不启动Python。
$base = 'https://developer.download.nvidia.com/compute/cuda/redist/'
$manifests = @{}
foreach ($release in @('12.8.1', '12.9.1')) {
    $manifests[$release] = Invoke-RestMethod -Uri ($base + 'redistrib_' + $release + '.json')
    if ($manifests[$release].release_label -ne $release) {
        throw "CUDA清单版本不匹配：$release"
    }
}
# W13：nvJitLink 取 12.9.1 清单的 12.9.86。
# 它与冻结 Warp 1.15.0 内置链接器同版。
# 12.8 链接器不能读取 12.9 生成的 LTOIR。
# 其余组件仍属 12.8.1，现有内核不变。
$components = @(
    @{ name = 'cuda_nvrtc'; release = '12.8.1'; version = $null },
    @{ name = 'cuda_cudart'; release = '12.8.1'; version = $null },
    @{ name = 'cuda_cccl'; release = '12.8.1'; version = $null },
    @{ name = 'cuda_nvcc'; release = '12.8.1'; version = $null },
    @{ name = 'libnvjitlink'; release = '12.9.1'; version = '12.9.86' }
)
$records = @()
foreach ($entry in $components) {
    $name = $entry.name
    $component = $manifests[$entry.release].$name
    if ($entry.version -and $component.version -ne $entry.version) {
        throw "CUDA组件版本不符：$name"
    }
    $archive = $component.'windows-x86_64'
    if (-not $archive -or $archive.sha256 -notmatch '^[a-f0-9]{64}$' -or
        $archive.relative_path -notmatch ('^' + $name + '/windows-x86_64/[a-zA-Z0-9_.-]+\.zip$')) {
        throw "CUDA组件清单错误：$name"
    }
    $file = Join-Path $downloads ([IO.Path]::GetFileName($archive.relative_path))
    if (-not (Test-Path -LiteralPath $file)) {
        Write-Host "下载$name"
        Invoke-WebRequest -Uri ($base + $archive.relative_path) -OutFile $file
    }
    $hash = (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($hash -ne $archive.sha256) {
        throw "CUDA组件校验失败：$name"
    }
    # 跨清单组件按版本解包，避免复用旧版目录。
    $folderName = if ($entry.version) { $name + '-' + $entry.version } else { $name }
    $unpacked = Join-Path $destination $folderName
    if (-not (Test-Path -LiteralPath $unpacked)) {
        Expand-Archive -LiteralPath $file -DestinationPath $unpacked
    }
    $package = @(Get-ChildItem -LiteralPath $unpacked -Directory)
    if ($package.Count -ne 1) {
        throw "CUDA组件目录错误：$name"
    }
    foreach ($folder in @('bin', 'include', 'nvvm')) {
        $source = Join-Path $package[0].FullName $folder
        if (Test-Path -LiteralPath $source) {
            $target = Join-Path $destination $folder
            New-Item -ItemType Directory -Path $target -Force | Out-Null
            Get-ChildItem -LiteralPath $source | Copy-Item -Destination $target -Recurse -Force
        }
    }
    # 保留组件随附许可原文，供通知核对。
    $licenseText = Join-Path $package[0].FullName 'LICENSE'
    $licenseHash = $null
    if (Test-Path -LiteralPath $licenseText) {
        $licenses = Join-Path $destination 'licenses'
        New-Item -ItemType Directory -Path $licenses -Force | Out-Null
        Copy-Item -LiteralPath $licenseText -Destination (Join-Path $licenses ($name + '-LICENSE.txt')) -Force
        $licenseHash = (Get-FileHash -LiteralPath $licenseText -Algorithm SHA256).Hash.ToLowerInvariant()
    }
    $records += [ordered]@{ component = $name; release = $entry.release; version = $component.version; license = $component.license; license_sha256 = $licenseHash; url = $base + $archive.relative_path; sha256 = $hash }
}
$records | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $destination 'components.json') -Encoding utf8

# MathDx：libmathdx 0.3.1（用户 2026-10-24 裁决 A）。
# 它是 Warp 1.15.0 冻结依赖的同一 Windows cuda12 包。
# 它不属 CUDA redistrib 清单，按固定 URL、大小与摘要下载。
# 二进制与静态库只落地 target/ 忽略目录。
$mathdxVersion = '0.3.1'
$mathdxUrl = 'https://developer.nvidia.com/downloads/compute/cublasdx/redist/cublasdx/cuda12/libmathdx-win32-x86_64-0.3.1-cuda12.0.zip'
$mathdxArchiveBytes = 30920469
$mathdxArchiveSha = '64488decd3bf6e09a717cec966ad96b10fe2e9db7935185ecb491cfd6c2ec514'
$mathdxDllBytes = 14854656
$mathdxDllSha = '5ce7528e8a78b4863a423a9c2e2d1c8e3a62b39e593a001d44d2339ea9433f39'
$mathdxDestination = Join-Path $root 'target\toolchains\libmathdx-0.3.1'
$mathdxDownloads = Join-Path $mathdxDestination 'downloads'
New-Item -ItemType Directory -Path $mathdxDownloads -Force | Out-Null
$mathdxFile = Join-Path $mathdxDownloads 'libmathdx-win32-x86_64-0.3.1-cuda12.0.zip'
if (-not (Test-Path -LiteralPath $mathdxFile)) {
    Write-Host "下载libmathdx $mathdxVersion"
    Invoke-WebRequest -Uri $mathdxUrl -OutFile $mathdxFile
}
if ((Get-Item -LiteralPath $mathdxFile).Length -ne $mathdxArchiveBytes) {
    throw 'libmathdx压缩包大小不符'
}
$mathdxHash = (Get-FileHash -LiteralPath $mathdxFile -Algorithm SHA256).Hash.ToLowerInvariant()
if ($mathdxHash -ne $mathdxArchiveSha) {
    throw 'libmathdx压缩包校验失败'
}
$mathdxDll = Join-Path $mathdxDestination 'bin\mathdx64_0.dll'
if (-not (Test-Path -LiteralPath $mathdxDll)) {
    Expand-Archive -LiteralPath $mathdxFile -DestinationPath $mathdxDestination
}
$mathdxDllItem = Get-Item -LiteralPath $mathdxDll
$mathdxDllHash = (Get-FileHash -LiteralPath $mathdxDll -Algorithm SHA256).Hash.ToLowerInvariant()
if ($mathdxDllItem.Length -ne $mathdxDllBytes -or $mathdxDllHash -ne $mathdxDllSha) {
    throw 'libmathdx DLL大小或摘要不符'
}
foreach ($required in @('LICENSE.txt', 'include\libcommondx.h', 'include\libcusolverdx.h', 'lib\x64\mathdx.lib')) {
    if (-not (Test-Path -LiteralPath (Join-Path $mathdxDestination $required))) {
        throw "libmathdx包内容缺失：$required"
    }
}
Write-Host "MATHdx_ROOT=$mathdxDestination"
Write-Host "CUDA_PATH=$destination"
Write-Host '运行探针时再设置进程PATH。'
