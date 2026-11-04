$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
$exe=Join-Path $project 'target/release/ae-strom.exe'
$root=Join-Path $project ('local-data/single exe smoke '+(Get-Date).ToUniversalTime().ToString('yyyyMMdd-HHmmss-fff'))
New-Item -ItemType Directory -Path $root | Out-Null
Copy-Item -LiteralPath $exe -Destination (Join-Path $root 'ae-strom.exe')
$profile=$root
$previousProfile=Join-Path $root 'local-data/player'
New-Item -ItemType Directory -Path $previousProfile -Force | Out-Null
New-Item -ItemType Directory -Path $profile -Force | Out-Null
$preset=@{name='ProbeCurve';preamp=1.0;bands=@(1.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,-1.0)}
$config=@{currentUrl='';volume=17;muted=$true;lastPreset=0;lastBank=25;windowW=725;windowH=36;windowX=0;windowY=0;windowPosValid=$false;apiEndpoint='';customEqPresets=@($preset);jsonPollIntervalSec=10;stationBanks=@(
 @{bank='A';slots=@(@{key='1';name='Synthetic A';url='https://synthetic.invalid/a';eq='Off';metadataType='';metadataUrl=''})},
 @{bank='Z';slots=@(@{key='1';name='Synthetic Z';url='https://synthetic.invalid/z';eq='ProbeCurve';metadataType='';metadataUrl=''})}
)}
[IO.File]::WriteAllText((Join-Path $previousProfile 'config.json'),($config | ConvertTo-Json -Depth 20),(New-Object Text.UTF8Encoding($false)))
[IO.File]::WriteAllText((Join-Path $profile 'startup-probe.fixture'),"ae-strom synthetic startup fixture`n",(New-Object Text.UTF8Encoding($false)))
$previousHash=(Get-FileHash -LiteralPath (Join-Path $previousProfile 'config.json') -Algorithm SHA256).Hash
$results=@()
foreach($run in @(1,2)){
 $process=Start-Process -FilePath (Join-Path $root 'ae-strom.exe') -ArgumentList '--startup-smoke' -WorkingDirectory $env:TEMP -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $root "stdout-$run.log") -RedirectStandardError (Join-Path $root "stderr-$run.log")
 if(-not $process.WaitForExit(45000)){throw "Own single-EXE probe still running PID $($process.Id); inspect before retry: $root"}
 $evidence=Join-Path $profile 'native-ui.json'
 if(-not(Test-Path -LiteralPath $evidence)){throw "Startup probe has no evidence: $root"}
 $record=Get-Content -LiteralPath $evidence -Raw -Encoding utf8 | ConvertFrom-Json
 if($process.ExitCode -ne 0 -or -not $record.ui.pass -or $record.host.mode -ne 'live' -or $record.host.profileName -ne (Split-Path -Leaf $root)){throw "Single-EXE default startup failed: $root"}
 if($record.ui.initialVolume -ne (16+$run) -or $record.ui.savedVolume -ne (17+$run)){throw 'Settings did not survive restart'}
 if($record.host.engineDiagnostics.active -eq $true){throw 'Unexpected audio session during stopped-startup probe'}
 $saved=Get-Content -LiteralPath (Join-Path $profile 'config.json') -Raw -Encoding utf8 | ConvertFrom-Json
 if($saved.volume -ne (17+$run) -or $saved.customEqPresets[0].name -ne 'ProbeCurve' -or $saved.stationBanks.Count -ne 2){throw 'Persisted profile lost stations/EQ/settings'}
 Copy-Item -LiteralPath $evidence -Destination (Join-Path $root "native-$run.json")
 $results+=@{run=$run;pass=$true;checks=$record.ui.checks.Count;initialVolume=$record.ui.initialVolume;savedVolume=$record.ui.savedVolume;mode=$record.host.mode;audio=$false}
 Write-Output "PASS native single EXE run $run : default live/stopped, portable profile, save/reload, embedded assets"
}
if((Get-FileHash -LiteralPath (Join-Path $previousProfile 'config.json') -Algorithm SHA256).Hash -ne $previousHash){throw 'Previous profile was modified'}
if(Test-Path -LiteralPath (Join-Path $root 'webview')){throw 'Browser data leaked beside executable'}
if(-not(Test-Path -LiteralPath (Join-Path $record.host.webviewDirectory 'EBWebView'))){throw 'AppData WebView profile not created'}
if(@(Get-ChildItem -LiteralPath $root -File -Filter '*.exe').Count -ne 1){throw 'Probe distribution must contain only one executable'}
@{pass=$true;runs=$results;singleExecutable=$true;sourceOrWebFoldersRequired=$false;cwdIndependent=$true;userProfileOpened=$false;audio=$false;physicalInput=$false} | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $root 'result.json') -Encoding utf8
Write-Output (Join-Path $root 'result.json')
