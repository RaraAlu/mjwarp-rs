[CmdletBinding()]
param(
    [string]$RunDirectory = 'target\u038-3d\debt2\final',
    [string]$FixtureDirectory = 'fixtures\gravity-upstream-abi\three-d-kinematics',
    [string]$OutputPath = 'docs\development-evidence\u038-3d\debt-2\first-diff.json'
)
# U038-3D 首差离线对照：把产品八字段/重力原始位与冻结上游原始位逐项对齐，
# 按 world/body/joint/dof/component 映射每个位差的位置与首个差异点。
# 只读既有证据文件，不重跑 GPU、不改产品源码/测试/fixtures/容差/断言。
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
Push-Location $root
try {
    $product = Get-Content -LiteralPath (Join-Path $RunDirectory 'product-fields-raw.json') -Raw -Encoding utf8 | ConvertFrom-Json
    $productGravity = Get-Content -LiteralPath (Join-Path $RunDirectory 'product-gravity-raw.json') -Raw -Encoding utf8 | ConvertFrom-Json
    $upstream = Get-Content -LiteralPath (Join-Path $FixtureDirectory 'raw-bits.json') -Raw -Encoding utf8 | ConvertFrom-Json
    $upstreamGravity = Get-Content -LiteralPath (Join-Path $RunDirectory 'upstream-gravity-raw.json') -Raw -Encoding utf8 | ConvertFrom-Json
    $case = Get-Content -LiteralPath (Join-Path $FixtureDirectory 'input.json') -Raw -Encoding utf8 | ConvertFrom-Json

    # MuJoCo 关节类型 -> 名称（0 free / 1 ball / 2 slide / 3 hinge）。
    $jointNames = @{ 0 = 'free'; 1 = 'ball'; 2 = 'slide'; 3 = 'hinge' }
    $joints = @()
    for ($j = 0; $j -lt $case.njnt; $j++) {
        $joints += [ordered]@{
            joint = $j
            type = $jointNames[[int]$case.jnt_type[$j]]
            body = [int]$case.jnt_bodyid[$j]
        }
    }
    $dofOwners = @('free:body1(0-5)', 'free:body1(0-5)', 'free:body1(0-5)', 'free:body1(0-5)', 'free:body1(0-5)', 'free:body1(0-5)',
        'ball:body2(6-8)', 'ball:body2(6-8)', 'ball:body2(6-8)', 'hinge:body3(9)', 'slide:body4(10)')

    function Get-UlpDistance([uint32]$A, [uint32]$B) {
        # 浮点位映射为单调整数（正数 +0x80000000，负数 0x80000000-尾数），再取差。
        $orderA = if (($A -band 0x80000000) -ne 0) { [int64]0x80000000 - [int64]($A -band 0x7FFFFFFF) } else { [int64]0x80000000 + [int64]$A }
        $orderB = if (($B -band 0x80000000) -ne 0) { [int64]0x80000000 - [int64]($B -band 0x7FFFFFFF) } else { [int64]0x80000000 + [int64]$B }
        return [int64][Math]::Abs($orderA - $orderB)
    }

    function Get-FieldLocation([string]$Field, [int]$Index) {
        switch ($Field) {
            'xpos' { $per = 15; $world = [int][Math]::Floor($Index / $per); $rem = $Index % $per; $body = [int][Math]::Floor($rem / 3); $comp = $rem % 3; $name = @('x', 'y', 'z')[$comp]; return [ordered]@{ world = $world; body = $body; component = $name } }
            'xquat_wxyz' { $per = 20; $world = [int][Math]::Floor($Index / $per); $rem = $Index % $per; $body = [int][Math]::Floor($rem / 4); $comp = $rem % 4; $name = @('w', 'x', 'y', 'z')[$comp]; return [ordered]@{ world = $world; body = $body; component = $name } }
            'xmat' { $per = 45; $world = [int][Math]::Floor($Index / $per); $rem = $Index % $per; $body = [int][Math]::Floor($rem / 9); $comp = $rem % 9; return [ordered]@{ world = $world; body = $body; row = [int][Math]::Floor($comp / 3); column = $comp % 3 } }
            'xipos' { $per = 15; $world = [int][Math]::Floor($Index / $per); $rem = $Index % $per; $body = [int][Math]::Floor($rem / 3); $comp = $rem % 3; $name = @('x', 'y', 'z')[$comp]; return [ordered]@{ world = $world; body = $body; component = $name } }
            'xanchor' { $per = 12; $world = [int][Math]::Floor($Index / $per); $rem = $Index % $per; $joint = [int][Math]::Floor($rem / 3); $comp = $rem % 3; $name = @('x', 'y', 'z')[$comp]; return [ordered]@{ world = $world; joint = $joint; joint_type = $joints[$joint].type; body = $joints[$joint].body; component = $name } }
            'xaxis' { $per = 12; $world = [int][Math]::Floor($Index / $per); $rem = $Index % $per; $joint = [int][Math]::Floor($rem / 3); $comp = $rem % 3; $name = @('x', 'y', 'z')[$comp]; return [ordered]@{ world = $world; joint = $joint; joint_type = $joints[$joint].type; body = $joints[$joint].body; component = $name } }
            'subtree_com' { $per = 15; $world = [int][Math]::Floor($Index / $per); $rem = $Index % $per; $body = [int][Math]::Floor($rem / 3); $comp = $rem % 3; $name = @('x', 'y', 'z')[$comp]; return [ordered]@{ world = $world; body = $body; component = $name } }
            'cdof' { $per = 66; $world = [int][Math]::Floor($Index / $per); $rem = $Index % $per; $dof = [int][Math]::Floor($rem / 6); $comp = $rem % 6; $kind = if ($comp -lt 3) { 'angular' } else { 'linear' }; $name = if ($comp -lt 3) { @('rx', 'ry', 'rz')[$comp] } else { @('x', 'y', 'z')[$comp - 3] }; return [ordered]@{ world = $world; dof = $dof; dof_owner = $dofOwners[$dof]; component = $name; kind = $kind } }
            'qfrc_gravcomp' { $per = 11; $world = [int][Math]::Floor($Index / $per); $dof = $Index % $per; return [ordered]@{ world = $world; dof = $dof; dof_owner = $dofOwners[$dof] } }
            default { throw "未知字段：$Field" }
        }
    }

    function Convert-HexBits([string[]]$HexBits) {
        $result = New-Object 'System.Collections.Generic.List[uint32]'
        foreach ($hex in $HexBits) { $result.Add([Convert]::ToUInt32($hex, 16)) }
        return , $result.ToArray()
    }

    function Compare-Field([string]$Field, [uint32[]]$ProductBits, [uint32[]]$UpstreamBits) {
        if ($ProductBits.Length -ne $UpstreamBits.Length) { throw "$Field 长度不一致" }
        $differences = New-Object 'System.Collections.Generic.List[object]'
        $first = $null
        for ($i = 0; $i -lt $ProductBits.Length; $i++) {
            $a = $ProductBits[$i]; $b = $UpstreamBits[$i]
            if ($a -eq $b) { continue }
            $productValue = [BitConverter]::ToSingle([BitConverter]::GetBytes($a), 0)
            $upstreamValue = [BitConverter]::ToSingle([BitConverter]::GetBytes($b), 0)
            $entry = [ordered]@{
                index = $i
                location = (Get-FieldLocation $Field $i)
                product_bits = ('{0:x8}' -f $a)
                upstream_bits = ('{0:x8}' -f $b)
                product = $productValue
                upstream = $upstreamValue
                abs = [Math]::Abs([double]$productValue - [double]$upstreamValue)
                ulp = (Get-UlpDistance $a $b)
            }
            $differences.Add($entry)
            if ($null -eq $first) { $first = $entry }
        }
        return [ordered]@{
            field = $Field
            values = $ProductBits.Length
            bit_differences = $differences.Count
            first_difference = $first
            differences = $differences.ToArray()
        }
    }

    $productFields = @{}
    foreach ($field in $product.fields) { $productFields[$field.field] = Convert-HexBits $field.bits }
    $upstreamFields = @{}
    foreach ($field in $upstream.fields) { $upstreamFields[$field.field] = Convert-HexBits $field.bits }

    $fields = New-Object 'System.Collections.Generic.List[object]'
    foreach ($name in @('xpos', 'xquat_wxyz', 'xmat', 'xipos', 'xanchor', 'xaxis', 'subtree_com', 'cdof')) {
        $fields.Add((Compare-Field $name $productFields[$name] $upstreamFields[$name]))
    }

    $productGravityBits = Convert-HexBits $productGravity.field_bits[0]
    $upstreamGravityBits = Convert-HexBits $upstreamGravity.field_bits[0]
    $gravity = Compare-Field 'qfrc_gravcomp' $productGravityBits $upstreamGravityBits
    $perDof = New-Object 'System.Collections.Generic.List[object]'
    foreach ($difference in $gravity.differences) {
        $perDof.Add([ordered]@{
            world = $difference.location.world
            dof = $difference.location.dof
            dof_owner = $difference.location.dof_owner
            product = $difference.product
            upstream = $difference.upstream
            abs = $difference.abs
            ulp = $difference.ulp
            outside_2e_6 = ($difference.abs -gt 2e-6)
        })
    }
    $gravity.outside_2e_6 = @($perDof | Where-Object { $_.outside_2e_6 }).Count
    $gravity.differences_by_dof = $perDof.ToArray()

    # 因果序核对：world2 是首个可观测差异世界，逐字段列出是否位差。
    $world2 = [ordered]@{}
    foreach ($field in $fields) {
        $inWorld = @($field.differences | Where-Object { $_.location.world -eq 2 })
        $world2[$field.field] = [ordered]@{
            bit_differences = $inWorld.Count
            first_index = if ($inWorld.Count -gt 0) { $inWorld[0].index } else { $null }
        }
    }

    # 逐 world 位差矩阵：用于核对「重力输入字段差异 world」与「qfrc 差异 world」是否同域。
    $worlds = 7
    $worldMatrix = @()
    for ($w = 0; $w -lt $worlds; $w++) {
        $row = [ordered]@{ world = $w }
        foreach ($field in $fields) {
            $row[$field.field] = @($field.differences | Where-Object { $_.location.world -eq $w }).Count
        }
        $row['qfrc_gravcomp'] = @($perDof | Where-Object { $_.world -eq $w }).Count
        $row['qfrc_outside_2e_6'] = @($perDof | Where-Object { $_.world -eq $w -and $_.outside_2e_6 }).Count
        $worldMatrix += $row
    }

    # 重力灵敏度：每体 |f| = mass*gravcomp*|g|（qfrc 差异 ≈ Δj·f 的量级来源）。
    $modelPackage = Get-Content -LiteralPath 'fixtures\gravity-upstream-abi\three-d-model-package.json' -Raw -Encoding utf8 | ConvertFrom-Json
    $gravityVector = @([double]$modelPackage.model.opt_gravity[0], [double]$modelPackage.model.opt_gravity[1], [double]$modelPackage.model.opt_gravity[2])
    $gravityMagnitude = [Math]::Sqrt(($gravityVector | ForEach-Object { $_ * $_ } | Measure-Object -Sum).Sum)
    $forceMagnitudes = @()
    for ($b = 1; $b -lt $modelPackage.model.body_mass.Count; $b++) {
        $mass = [double]$modelPackage.model.body_mass[$b]
        $gravcomp = [double]$modelPackage.model.body_gravcomp[$b]
        $forceMagnitudes += [ordered]@{ body = $b; mass = $mass; gravcomp = $gravcomp; force = ($mass * $gravcomp * $gravityMagnitude) }
    }

    $report = [ordered]@{
        schema = 1
        kind = 'u038-3d-first-difference-offline-comparison'
        task = '2026-10-23-u038-3d-gravity-propagation-debt-2'
        provenance = [ordered]@{
            product_fields = (Join-Path $RunDirectory 'product-fields-raw.json')
            product_gravity = (Join-Path $RunDirectory 'product-gravity-raw.json')
            upstream_fields = (Join-Path $FixtureDirectory 'raw-bits.json')
            upstream_gravity = (Join-Path $RunDirectory 'upstream-gravity-raw.json')
            kinematics_input = (Join-Path $FixtureDirectory 'input.json')
            joints = $joints
            absolute_tolerance = 2e-6
            note = '按产品与冻结上游原始 f32 位逐项比较；world2 因果序核对看首个可观测差异世界的字段次序'
        }
        fields = $fields.ToArray()
        gravity = $gravity
        world2_field_order = $world2
        world_diff_matrix = $worldMatrix
        gravity_force_magnitudes = [ordered]@{ gravity_magnitude = $gravityMagnitude; bodies = $forceMagnitudes }
        total_bit_differences = (($fields | ForEach-Object { $_.bit_differences } | Measure-Object -Sum).Sum + $gravity.bit_differences)
        total_values = (($fields | ForEach-Object { $_.values } | Measure-Object -Sum).Sum + $gravity.values)
    }
    $directory = Split-Path -Parent $OutputPath
    if ($directory -and -not (Test-Path -LiteralPath $directory)) { New-Item -ItemType Directory -Force -Path $directory | Out-Null }
    $report | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $OutputPath -Encoding utf8
    Write-Host ("首差对照完成：{0}（{1} 位差 / {2} 项，重力超差 {3}）" -f $OutputPath, $report.total_bit_differences, $report.total_values, $gravity.outside_2e_6)
    foreach ($field in $fields) {
        if ($field.bit_differences -gt 0) {
            $first = $field.first_difference
            $pairs = for ($k = 0; $k -lt $first.location.Count; $k++) { "$($first.location.Keys[$k])=$($first.location.Values[$k])" }
            Write-Host ("  {0}: {1} 位差，首个 #{2} [{3}]（{4}->{5}，{6} ulp）" -f $field.field, $field.bit_differences, $first.index, ($pairs -join ','), $first.product_bits, $first.upstream_bits, $first.ulp)
        }
    }
}
finally {
    Pop-Location
}
