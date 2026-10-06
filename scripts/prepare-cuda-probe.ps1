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
$manifestUrl = $base + 'redistrib_12.8.1.json'
$manifest = Invoke-RestMethod -Uri $manifestUrl
if ($manifest.release_label -ne '12.8.1') {
    throw 'CUDA清单版本不匹配'
}
$records = @()
foreach ($name in @('cuda_nvrtc', 'cuda_cudart', 'cuda_cccl', 'cuda_nvcc')) {
    $component = $manifest.$name
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
    $unpacked = Join-Path $destination $name
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
    $records += [ordered]@{ component = $name; version = $component.version; license = $component.license; url = $base + $archive.relative_path; sha256 = $hash }
}
$records | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $destination 'components.json') -Encoding utf8
Write-Host "CUDA_PATH=$destination"
Write-Host '运行探针时再设置进程PATH。'
