param([Parameter(Mandatory=$true)][ValidateRange(1,3)][int]$Attempt)
$ErrorActionPreference='Stop'
$root=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
Set-Location $root
$dir=Join-Path $root "target/small-mass-prefix-reference/attempt-$Attempt"
if(Test-Path -LiteralPath $dir){throw 'Attempt already exists'}
[void](New-Item -ItemType Directory -Path $dir)
$contract=Get-Content fixtures/small-mass-prefix/contract.json -Raw|ConvertFrom-Json
foreach($p in $contract.files){if((Get-FileHash $p.path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $p.sha256){throw 'Frozen source mismatch'}}
Copy-Item -LiteralPath fixtures/small-mass-prefix/contract.json -Destination (Join-Path $dir 'contract.json')
$blocks=@();$start=0;$address=0
foreach($n in 2..6){$blocks+=@([ordered]@{size=$n;relative_start=$start;address=$address});$start+=$n;$address+=$n*$n}
$cases=@()
foreach($sample in 0..1){
 $csr=@();$upper=@()
 foreach($block in $blocks){
  $n=$block.size;$u=[double[]]::new($n*$n)
  for($i=0;$i -lt $n;$i++){for($j=$i;$j -lt $n;$j++){
   $u[$i*$n+$j]=if($i -eq $j){if($sample -eq 0){[Math]::Pow(2,$i+1)}else{2*$i+3}}else{(1-2*(($i+$j)%2))*($i+$j+1)/$(if($sample -eq 0){4.0}else{8.0})}
  }}
  for($i=0;$i -lt $n;$i++){for($j=0;$j -le $i;$j++){
   [double]$value=0
   for($k=0;$k -le $j;$k++){$value+=$u[$k*$n+$i]*$u[$k*$n+$j]}
   if([double][single]$value -ne $value){throw 'Gram matrix is not exactly f32 representable'}
   $csr+=@($value)
  }}
  $upper+=@($u)
 }
 $cases+=@([ordered]@{csr=$csr;upper=$upper})
}
$result=[ordered]@{schema=1;scope='known analytic Small U; M=transpose(U)*U only, no factor algorithm';blocks=$blocks;small_nv=20;small_nc=55;prefix=90;cases=$cases}
$json=($result|ConvertTo-Json -Depth 15).Replace("`r`n","`n")+"`n"
[IO.File]::WriteAllText((Join-Path $dir 'reference.json'),$json,[Text.UTF8Encoding]::new($false))
$output=Join-Path $root 'fixtures/small-mass-prefix/reference.json'
if(Test-Path -LiteralPath $output){throw 'Frozen reference already exists'}
Copy-Item -LiteralPath (Join-Path $dir 'reference.json') -Destination $output
$manifest=[ordered]@{schema=1;attempt=$Attempt;frozen=$true;author='offline independent analytic U and exact dyadic Gram input; no MuJoCo/factorM/Python/product';contract_sha256=(Get-FileHash fixtures/small-mass-prefix/contract.json -Algorithm SHA256).Hash.ToLowerInvariant();output_sha256=(Get-FileHash $output -Algorithm SHA256).Hash.ToLowerInvariant();states=2;small_sizes=@(2,3,4,5,6);prefix=90;old_authors_called=0}
[IO.File]::WriteAllText((Join-Path $root 'fixtures/small-mass-prefix/manifest.json'),(($manifest|ConvertTo-Json -Depth 10).Replace("`r`n","`n")+"`n"),[Text.UTF8Encoding]::new($false))
'Analytic fixture frozen: two states, Small2..6, 55 CSR slots, 90 U slots'
