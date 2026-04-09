param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$MoonArgs
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$moonExe = Join-Path $repoRoot "target\debug\moon.exe"

if (-not (Test-Path -LiteralPath $moonExe)) {
    Push-Location $repoRoot
    try {
        cargo build
    } finally {
        Pop-Location
    }
}

if (-not (Test-Path -LiteralPath $moonExe)) {
    throw "moon executable was not found at $moonExe"
}

for ($day = 1; $day -le 30; $day++) {
    $date = "2026-04-{0:D2}" -f $day
    Write-Host ""
    Write-Host "=== $date ==="
    & $moonExe @MoonArgs $date
}
