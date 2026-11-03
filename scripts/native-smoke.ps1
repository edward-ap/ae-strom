$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$executable = Join-Path $projectRoot 'target\release\ae-strom.exe'
if (-not (Test-Path -LiteralPath $executable)) { throw 'Build the native candidate first.' }
$runName = 'native-smoke-' + (Get-Date).ToUniversalTime().ToString('yyyyMMdd-HHmmss-fff')
$dataDirectory = Join-Path $projectRoot ('local-data\' + $runName)
New-Item -ItemType Directory -Path $dataDirectory -Force | Out-Null
$arguments = @('--fake', '--smoke', '--data-dir', ('"' + $dataDirectory + '"'))
$process = Start-Process -FilePath $executable -ArgumentList $arguments -WorkingDirectory $projectRoot -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $dataDirectory 'stdout.log') -RedirectStandardError (Join-Path $dataDirectory 'stderr.log')
Write-Output "Started isolated fake smoke PID $($process.Id)"
if (-not $process.WaitForExit(45000)) { throw "Smoke is still running (PID $($process.Id)); inspect this launch before retrying. Data: $dataDirectory" }
$resultPath = Join-Path $dataDirectory 'native-ui.json'
if (-not (Test-Path -LiteralPath $resultPath)) { Get-Content -LiteralPath (Join-Path $dataDirectory 'stderr.log'); throw 'Native UI did not produce evidence.' }
$result = Get-Content -LiteralPath $resultPath -Encoding utf8 -Raw | ConvertFrom-Json
if ($process.ExitCode -ne 0 -or -not $result.ui.pass) { throw "Native smoke failed. Evidence: $resultPath" }
$expectedDisplayName = '{0} Str{1}m' -f [char]0x00C6, [char]0x00F8
if ($result.host.packageName -ne $expectedDisplayName -or $result.host.productName -ne $expectedDisplayName -or $result.host.identifier -ne 'local.edwardap.ae-strom' -or $result.host.windowTitle -ne $expectedDisplayName -or -not $result.host.dataIsAbsolute -or $result.host.profileName -ne $runName -or $result.host.audioAllowed) { throw 'Native app identity or synthetic data isolation mismatch.' }
Write-Output "PASS: $($result.ui.checks.Count) programmatic native checks; audio=$($result.ui.audio), OS input=$($result.ui.userInput)"
Write-Output $resultPath
