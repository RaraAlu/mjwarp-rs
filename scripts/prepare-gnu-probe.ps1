[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$destination = Join-Path $root 'target\toolchains\mingw'
New-Item -ItemType Directory -Path $destination -Force | Out-Null

# 下载官方工具包，保留许可文件。
# 工具仅参与构建，不链接引擎。
$packages = @(
    @('binutils-2.47-3', '827363748ce3320683319d860ee4fcfcdbb36baf66aac6dab0ec3822be2d4ff7'),
    @('gettext-runtime-1.0-1', 'be68d7f260633284b910c588c6d82ee304a81c8817a686d2cd9df83f872c27af'),
    @('libwinpthread-14.0.0.r426.g4564ee4b5-1', '543017ce2731292b215bf1d36fd70a86d8a8d5ed0afba9d3db9fff89804cda71'),
    @('zlib-1.3.2-2', '9e75842a070ba648e986e12424e1c92c9d7d77200e85f6a34eeb600819f2e694'),
    @('zstd-1.5.7-2', '1add6705b344664f6aca108c85f79ab5bdd9e1162662bb06a4cf40a34f6e0907'),
    @('libiconv-1.19-1', '21e334d0911f25de75d3e18e0697648bcecfa9658256d600cad0827d719c2f35')
)
foreach ($package in $packages) {
    $name = 'mingw-w64-x86_64-' + $package[0] + '-any.pkg.tar.zst'
    $file = Join-Path $destination $name
    if (-not (Test-Path -LiteralPath $file)) {
        Invoke-WebRequest -Uri ('https://repo.msys2.org/mingw/mingw64/' + $name) -OutFile $file
    }
    if ((Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLowerInvariant() -ne $package[1]) {
        throw "GNU工具包校验失败：$name"
    }
    & "$env:SystemRoot\System32\tar.exe" -xf $file -C $destination
    if ($LASTEXITCODE -ne 0) {
        throw "GNU工具包解压失败：$name"
    }
}
$bin = Join-Path $destination 'mingw64\bin'
& (Join-Path $bin 'as.exe') --version
if ($LASTEXITCODE -ne 0) {
    throw 'GNU汇编器启动失败'
}
& (Join-Path $bin 'dlltool.exe') --version
if ($LASTEXITCODE -ne 0) {
    throw 'GNU导入库工具启动失败'
}
Write-Host "GNU_BIN=$bin"
