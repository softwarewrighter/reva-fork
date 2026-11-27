# Check an installed Reva package app on Windows, without Git Bash: the
# core of tests/check-installed.sh (which has the full list).  For a run by
# hand after installing a setup.exe; CI runs it too (installer-lifecycle.sh).
#
#   powershell -ExecutionPolicy Bypass -File tests\check-installed.ps1 `
#       -App "$env:LOCALAPPDATA\Reva Demo\reva-demo.exe" [-PackageVersion 1.0.1] [-RevaVersion 2026.1126.0]
#
# The user's folder goes under a throwaway REVA_DOCUMENTS (the app reads
# it), so a check never touches the user's real Documents.
param(
    [Parameter(Mandatory = $true)][string]$App,
    [string]$PackageVersion = "",
    [string]$RevaVersion = ""
)
# not "Stop": Windows PowerShell 5.1 makes a native program's stderr (WebView2
# writes log lines there) a terminating error; the checks report failures
$ErrorActionPreference = "Continue"
if (-not $RevaVersion) {
    $v = Join-Path $PSScriptRoot "..\VERSION"
    if (Test-Path $v) { $RevaVersion = (Get-Content $v -Raw).Trim() }
}
$docs = Join-Path ([System.IO.Path]::GetTempPath()) ("reva-check-" + [guid]::NewGuid())
New-Item -ItemType Directory $docs | Out-Null
$env:REVA_DOCUMENTS = $docs
$script:fail = 0

function Run([string[]]$Lines) {
    $argv = @("--print-output")
    foreach ($l in $Lines) { $argv += @("-e", $l) }
    $argv += @("-e", "bye")
    (& $App @argv 2>$null | Out-String)
}
function Check([string]$Name, [string]$Want, [string]$Got) {
    if ($Got.Contains($Want)) { "ok   $Name" }
    else { "FAIL ${Name}: expected '$Want' in:"; $Got; $script:fail = 1 }
}
function Yes([string]$Name, [bool]$Ok) {
    if ($Ok) { "ok   $Name" } else { "FAIL $Name"; $script:fail = 1 }
}

$pkg = Join-Path (Split-Path $App) "package"
Yes "the package is installed ($pkg)" (Test-Path (Join-Path $pkg "reva-package.json"))
Yes "sqlite3.dll is installed next to the app" (Test-Path (Join-Path (Split-Path $App) "sqlite3.dll"))

$about = (& $App --about 2>$null | Out-String)
$about
Check "--about: reva-fork $RevaVersion" "vm: reva-fork $RevaVersion" $about
$pkgLine = ($about -split "`r?`n" | Where-Object { $_ -like "package: *" }) -join ""
if ($PackageVersion) { Check "--about: the package's version $PackageVersion" $PackageVersion $pkgLine }
Check "--about: the package is the installed one" "(installed)" $about
$userLine = ($about -split "`r?`n" | Where-Object { $_ -like "user folder: *" }) -join ""
$userPath = $userLine -replace "^user folder: ", ""

$out = Run @("1 2 + .", "version")
Check "starts and evaluates" "3 " $out
Check "version: reva-fork and its commit" "Commit:" $out

$user = if ($userPath -and (Test-Path $userPath)) { Get-Item $userPath } else { $null }
Yes "made the user's folder ($userPath)" ($null -ne $user)
if ($user) {
    Run @('" /check-written.txt" creat value h  " written" h write  h close') | Out-Null
    $f = Join-Path $user.FullName "check-written.txt"
    Yes "writes into the user's folder" ((Test-Path $f) -and ((Get-Content $f -Raw) -eq "written"))
    Yes "never into the install" (-not (Test-Path (Join-Path $pkg "check-written.txt")))

    Run @(': check-saved ." from the saved image" ;', "save check.img") | Out-Null
    $img = Join-Path $user.FullName "check.img"
    $out = (& $App --print-output --image $img -e check-saved -e bye 2>$null | Out-String)
    Check "save, then restart from the image" "from the saved image" $out
}

$out = Run @("needs turtle/fractals 4 koch-demo")
Check "turtle graphics draw in their own window" "[canvas: 768 lines]" $out

Remove-Item -Recurse -Force $docs -ErrorAction SilentlyContinue
exit $script:fail
