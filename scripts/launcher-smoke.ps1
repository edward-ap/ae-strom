$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
$launcher=Join-Path $project 'target/release/ae-strom-player.exe'
if(-not(Test-Path -LiteralPath $launcher)){throw 'Build the live launcher first'}
$root=Join-Path $project ('local-data/launcher probe '+(Get-Date).ToUniversalTime().ToString('yyyyMMdd-HHmmss-fff'))
New-Item -ItemType Directory -Path $root | Out-Null
$source=Join-Path $root 'probe.rs'
$code=@'
#![windows_subsystem = "windows"]
fn main(){let dir=std::env::current_exe().unwrap().parent().unwrap().to_path_buf();let args=std::env::args().skip(1).collect::<Vec<_>>().join("\n");std::fs::write(dir.join("probe-args.txt"),args).unwrap();}
'@
[IO.File]::WriteAllText($source,$code,(New-Object Text.UTF8Encoding($false)))
& rustc --edition=2021 --crate-name ae_strom_launcher_probe $source -o (Join-Path $root 'ae-strom.exe') *> (Join-Path $root 'compile.log')
if($LASTEXITCODE -ne 0){throw 'Synthetic probe compilation failed'}
Copy-Item -LiteralPath $launcher -Destination (Join-Path $root 'ae-strom-player.exe')
$process=Start-Process -FilePath (Join-Path $root 'ae-strom-player.exe') -WindowStyle Hidden -PassThru
if(-not $process.WaitForExit(10000)){throw "Own launcher still running PID $($process.Id); inspect before retry"}
if($process.ExitCode -ne 0){throw 'Native launcher returned an error'}
$timer=[Diagnostics.Stopwatch]::StartNew();$argsFile=Join-Path $root 'probe-args.txt'
while(-not(Test-Path -LiteralPath $argsFile)){
 if($timer.ElapsedMilliseconds -gt 3000){throw 'Synthetic child did not record its arguments'}
 Start-Sleep -Milliseconds 20
}
$actual=@(Get-Content -LiteralPath $argsFile -Encoding utf8)
$expected=@('--enable-live-audio','--data-dir',(Join-Path $root 'local-data/player'))
if($actual.Count -ne 3 -or $actual[0] -ne $expected[0] -or $actual[1] -ne $expected[1] -or [IO.Path]::GetFullPath($actual[2]) -ne [IO.Path]::GetFullPath($expected[2])){throw 'Native launcher mode/profile/space quoting mismatch'}
@{pass=$true;nativeProcessLaunch=$true;childWasSyntheticProbe=$true;normalMode='live';absoluteProfile=$true;spaceQuoting=$true;audio=$false;userProfileOpened=$false} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $root 'result.json') -Encoding utf8
Write-Output 'PASS: native launcher forwards explicit live mode and absolute isolated profile through a path with spaces; only a synthetic child ran.'
Write-Output (Join-Path $root 'result.json')
