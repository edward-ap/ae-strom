param([string]$ExecutablePath)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$executable = if ($ExecutablePath) { (Resolve-Path -LiteralPath $ExecutablePath).Path } else { Join-Path $projectRoot 'target\release\ae-strom.exe' }
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

$compactSeen = $false
$expandSeen = $false
foreach ($layout in $result.host.layouts) {
    if ($null -eq $layout.outerBefore -or $null -eq $layout.outerAfter) { throw 'Missing native resize geometry evidence.' }
    $before = $layout.outerBefore
    $after = $layout.outerAfter
    if ($layout.stripYBefore -ne $layout.stripYAfter) { throw 'Native resize moved the player strip vertically.' }
    if ($layout.horizontalAnchor -eq 'right' -and ($before.position.x + $before.size.width) -ne ($after.position.x + $after.size.width)) { throw 'Native resize lost its selected right anchor.' }
    if ($layout.horizontalAnchor -eq 'left' -and $before.position.x -ne $after.position.x) { throw 'Native resize lost its selected left anchor.' }
    if ($after.size.width -le $layout.workArea.size.width -and ($after.position.x -lt $layout.workArea.position.x -or ($after.position.x + $after.size.width) -gt ($layout.workArea.position.x + $layout.workArea.size.width))) { throw 'Native resized window exceeds its monitor work area.' }
    if ($layout.compact -and $before.size.width -gt $after.size.width) { $compactSeen = $true }
    if (-not $layout.compact -and $before.size.width -lt $after.size.width) { $expandSeen = $true }
}
if (-not $compactSeen -or -not $expandSeen) { throw 'Missing compact/expand native transitions.' }
Write-Output 'PASS: native compact/expand stay in the work area with the selected edge and player strip anchored.'
