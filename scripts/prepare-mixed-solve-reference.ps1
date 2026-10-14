param([Parameter(Mandatory=$true)][ValidateRange(1,3)][int]$Attempt)
$ErrorActionPreference='Stop'
$root=(Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Set-Location $root
$dir=Join-Path $root "target/mixed-solve-reference/attempt-$Attempt"
if(Test-Path -LiteralPath $dir){throw 'Attempt already exists'}
[void](New-Item -ItemType Directory -Path $dir)
$contract=Get-Content fixtures/mixed-mass-solve/contract.json -Raw|ConvertFrom-Json
foreach($pin in $contract.files){if((Get-FileHash $pin.path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $pin.sha256){throw "Frozen pin differs: $($pin.path)"}}
Copy-Item fixtures/mixed-mass-solve/contract.json (Join-Path $dir 'contract.json')
$native=(Get-Content fixtures/sparse-mass-factor/native.json -Raw|ConvertFrom-Json).models[0]
$lu=(Get-Content fixtures/sparse-mass-solve/reference.json -Raw|ConvertFrom-Json).models[0]
$small=Get-Content fixtures/small-mass-prefix/reference.json -Raw|ConvertFrom-Json
if($native.name -ne 'chain65' -or $lu.name -ne 'chain65'){throw 'Frozen model order'}
function NewFields {
 $f=[ordered]@{}
 foreach($key in @('dof_parentid','m_rownnz','m_rowadr','m_colind','tree_dofadr','tree_dofnum')){$f[$key]=[Collections.Generic.List[int]]::new()}
 return $f
}
function AddTree($f,[int]$n){
 $start=$f.dof_parentid.Count
 $f.tree_dofadr.Add($start);$f.tree_dofnum.Add($n)
 for($i=0;$i -lt $n;$i++){
  $f.dof_parentid.Add($(if($i -eq 0){-1}else{$start+$i-1}))
  $f.m_rowadr.Add($f.m_colind.Count);$f.m_rownnz.Add($i+1)
  for($j=0;$j -le $i;$j++){$f.m_colind.Add($start+$j)}
 }
}
function FieldsJson($f){
 $out=[ordered]@{}
 foreach($key in $f.Keys){$out[$key]=$f[$key].ToArray()}
 return $out
}
function Known([int]$n,[int]$column){
 $z=[double[]]::new($n);[uint32]$seed=9701
 for($i=0;$i -lt $n;$i++){
  if($column -eq 1){$z[$i]=$(if($i%2){-1.0}else{1.0})*($i+1)*0.125}
  if($column -eq 2){$seed=[uint32](([long]1664525*$seed+1013904223) -band 0xffffffffL);$z[$i]=([int]($seed -shr 24)-128)/64.0}
 }
 return ,$z
}
function Multiply($f,[single[]]$m,[double[]]$z,[int]$first){
 $b=[double[]]::new($z.Length)
 for($i=$first;$i -lt $z.Length;$i++){
  $start=$f.m_rowadr[$i];$end=$start+$f.m_rownnz[$i]
  for($j=$start;$j -lt $end;$j++){
   $k=$f.m_colind[$j];if($k -lt $first){throw 'cross-tree reference row'}
   $b[$i]+=[double]$m[$j]*$z[$k]
   if($k -ne $i){$b[$k]+=[double]$m[$j]*$z[$i]}
  }
 }
 return ,$b
}
$models=[Collections.Generic.List[object]]::new()
$maxResidual=0.0
foreach($name in @('mixed86','crowded390')){
 $f=NewFields
 if($name -eq 'mixed86'){
  foreach($key in $f.Keys){foreach($v in $native.$key){$f[$key].Add([int]$v)}}
  foreach($n in 2..6){AddTree $f $n}
 } else {AddTree $f 131;for($block=0;$block -lt 129;$block++){AddTree $f 2}}
 AddTree $f 1
 $n=$f.dof_parentid.Count;$cases=[Collections.Generic.List[object]]::new()
 foreach($sample in 0..1){
  $m=[Collections.Generic.List[single]]::new()
  if($name -eq 'mixed86'){
   foreach($v in $native.cases[$sample].matrix){$m.Add([single]$v)}
   foreach($v in $small.cases[$sample].csr){$m.Add([single]$v)}
  } else {
   for($i=0;$i -lt 131;$i++){for($j=0;$j -le $i;$j++){
    $d=$(if($sample -eq 0){1 -shl ($i%5)}else{2*($i%5)+3})
    $m.Add([single]$(if($i -eq $j){$d+0.0625}else{0.0625}))
   }}
   for($block=0;$block -lt 129;$block++){foreach($v in $small.cases[$sample].csr[0..2]){$m.Add([single]$v)}}
  }
  $m.Add([single]4.0)
  if($m.Count -ne $f.m_colind.Count){throw 'matrix/CSR extent'}
  $matrix=$m.ToArray();$rhs=[Collections.Generic.List[single]]::new();$solution=[Collections.Generic.List[double]]::new()
  for($column=0;$column -lt 3;$column++){
   $z=Known $n $column;$first=$(if($name -eq 'mixed86'){65}else{0})
   $b=Multiply $f $matrix $z $first
   for($i=0;$i -lt $first;$i++){$b[$i]=[single]$lu.cases[$sample].rhs[$column*65+$i];$z[$i]=[double]$lu.cases[$sample].solution[$column*65+$i]}
   foreach($v in $b){if([double][single]$v -ne $v){throw 'manufactured RHS not exact f32'};$rhs.Add([single]$v)}
   foreach($v in $z){$solution.Add([double]$v)}
   $az=Multiply $f $matrix $z 0
   $rowNorm=[double[]]::new($n)
   for($i=0;$i -lt $n;$i++){for($j=$f.m_rowadr[$i];$j -lt $f.m_rowadr[$i]+$f.m_rownnz[$i];$j++){
    $k=$f.m_colind[$j];$rowNorm[$i]+=[Math]::Abs([double]$matrix[$j]);if($k -ne $i){$rowNorm[$k]+=[Math]::Abs([double]$matrix[$j])}
   }}
   $an=0.0;$xn=0.0;$bn=0.0;$rn=0.0
   for($i=0;$i -lt $n;$i++){$an=[Math]::Max($an,$rowNorm[$i]);$xn=[Math]::Max($xn,[Math]::Abs($z[$i]));$bn=[Math]::Max($bn,[Math]::Abs($b[$i]));$rn=[Math]::Max($rn,[Math]::Abs($az[$i]-$b[$i]))}
   $residual=$rn/[Math]::Max(1e-30,$an*$xn+$bn)
   if($residual -gt 1e-12){throw 'reference residual'}
   $maxResidual=[Math]::Max($maxResidual,$residual)
  }
  $cases.Add([ordered]@{matrix=$matrix;rhs=$rhs.ToArray();solution=$solution.ToArray()})
 }
 $models.Add([ordered]@{name=$name;nv=$n;nc=$f.m_colind.Count;fields=(FieldsJson $f);cases=$cases.ToArray()})
}
$output='fixtures/mixed-mass-solve/reference.json'
if(Test-Path -LiteralPath $output){throw 'Frozen reference already exists'}
$reference=[ordered]@{schema=1;purpose='reuse frozen sparse LU and independently known solutions; only exact matrix-vector products, no solve/factorization';rhs_count=3;max_reference_residual=$maxResidual;models=$models.ToArray()}
$json=($reference|ConvertTo-Json -Depth 14).Replace("`r`n","`n")+"`n"
[IO.File]::WriteAllText((Join-Path $dir 'reference.json'),$json,[Text.UTF8Encoding]::new($false))
Copy-Item -LiteralPath (Join-Path $dir 'reference.json') -Destination $output
$manifest=[ordered]@{schema=1;attempt=$Attempt;frozen=$true;models=2;cases=4;columns=12;max_reference_residual=$maxResidual;contract_sha256=(Get-FileHash fixtures/mixed-mass-solve/contract.json -Algorithm SHA256).Hash.ToLowerInvariant();output_sha256=(Get-FileHash $output -Algorithm SHA256).Hash.ToLowerInvariant();old_authors_called=0;Python_called=$false;candidate_called=$false}
[IO.File]::WriteAllText((Join-Path $root 'fixtures/mixed-mass-solve/manifest.json'),(($manifest|ConvertTo-Json -Depth 8).Replace("`r`n","`n")+"`n"),[Text.UTF8Encoding]::new($false))
Write-Output "assembly: 2 models, 4 cases, 12 columns; max reference residual=$maxResidual"
