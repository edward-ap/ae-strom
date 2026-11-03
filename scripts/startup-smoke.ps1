$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
$exe=Join-Path $project 'target\release\ae-strom.exe'
$root=Join-Path $project ('local-data\followup\startup-matrix-'+(Get-Date).ToUniversalTime().ToString('yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Path $root | Out-Null
$results=@()
function Run-Case($name,$config,$reason){
 $dir=Join-Path $root $name
 New-Item -ItemType Directory -Path $dir | Out-Null
 if($null -ne $config){[IO.File]::WriteAllText((Join-Path $dir 'config.json'),($config | ConvertTo-Json -Depth 30),(New-Object Text.UTF8Encoding($false)))}
 $proc=Start-Process -FilePath $exe -ArgumentList @('--fake','--smoke','--data-dir',('"'+$dir+'"')) -WorkingDirectory $project -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $dir 'stdout.log') -RedirectStandardError (Join-Path $dir 'stderr.log')
 if(-not $proc.WaitForExit(45000)){throw "Own synthetic launch still running PID $($proc.Id); inspect before retry: $dir"}
 $record=Get-Content -LiteralPath (Join-Path $dir 'native-ui.json') -Raw -Encoding utf8 | ConvertFrom-Json
 $s=$record.host.startup
 if($proc.ExitCode -ne 0 -or -not $record.ui.pass -or $record.ui.checks.Count -ne 33){throw "Native checks failed: $dir"}
 if($s.visibleBefore -or -not $s.visibleAfter -or $s.arbitraryDelay -or $s.planned.reason -ne $reason -or -not $s.ui.fontLoaded){throw "Startup ordering failed: $dir"}
 foreach($key in @('x','y')){if($s.positionBeforeShow.$key -ne $s.planned.$key -or $s.positionAfterShow.$key -ne $s.planned.$key){throw "Position mismatch: $dir"}}
 foreach($key in @('width','height')){if($s.sizeBeforeShow.$key -ne $s.sizeAfterShow.$key){throw "Visible geometry changed: $dir"}}
 Write-Host "$name PASS: 33 checks, $reason, hidden before geometry/show"
 return $record
}
$first=Run-Case 'absent' $null 'centered-default'
$results+=@{case='absent';evidence='absent/native-ui.json';checks=33;pass=$true}
$primary=@($first.host.startup.monitors | Where-Object primary)[0]
if($null -eq $primary){$primary=$first.host.startup.monitors[0]}
# Only the freshly generated fake profile is read; no existing user profile.
$template=Get-Content -LiteralPath (Join-Path $root 'absent\config.json') -Raw -Encoding utf8 | ConvertFrom-Json
$cases=@(
 @{name='restored';x=($primary.x+100);y=($primary.y+100);reason='restored'},
 @{name='offscreen';x=1000000;y=1000000;reason='centered-offscreen'},
 @{name='clamped';x=($primary.x+$primary.width-20);y=($primary.y+100);reason='clamped'}
)
foreach($other in @($first.host.startup.monitors | Where-Object { -not $_.primary })){
 $cases+=@{name=('monitor-'+$cases.Count);x=($other.x+50);y=($other.y+50);reason='restored'}
}
foreach($case in $cases){
 $copy=$template | ConvertTo-Json -Depth 30 | ConvertFrom-Json
 $copy.windowPosValid=$true;$copy.windowX=[int]$case.x;$copy.windowY=[int]$case.y
 $record=Run-Case $case.name $copy $case.reason
 $results+=@{case=$case.name;evidence=($case.name+'/native-ui.json');checks=$record.ui.checks.Count;pass=$true}
}
@{pass=$true;cases=$results;audio=$false;physicalInput=$false;firstCompositorFrameObserved=$false;monitorCount=$first.host.startup.monitors.Count} | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $root 'summary.json') -Encoding utf8
Write-Output "Matrix evidence: $root"
