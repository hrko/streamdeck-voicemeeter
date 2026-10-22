param([Parameter(Mandatory)][ValidateSet('Stop', 'Start')][string]$Mode)
$ErrorActionPreference = 'Stop'
$app = Join-Path $env:ProgramFiles 'Elgato/StreamDeck/StreamDeck.exe'
if (-not (Test-Path -LiteralPath $app)) { throw "Stream Deck not found at $app" }
$expected = [IO.Path]::GetFullPath($app)
$running = @(Get-Process -Name StreamDeck -ErrorAction SilentlyContinue |
    Where-Object { $_.Path -and [IO.Path]::GetFullPath($_.Path) -eq $expected })
if ($Mode -eq 'Start') {
    if ($running.Count -eq 0) { Start-Process -FilePath $app -WindowStyle Hidden }
} else {
    foreach ($process in $running) {
        $null = $process.CloseMainWindow()
        if (-not $process.WaitForExit(3000)) {
            $process | Stop-Process -Force
            $process.WaitForExit()
        }
    }
}
