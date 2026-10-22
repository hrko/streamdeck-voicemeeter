$ErrorActionPreference = 'Stop'
$expected = [IO.Path]::GetFullPath((Join-Path $env:APPDATA 'Elgato/StreamDeck/Plugins/jp.hrko.streamdeck.voicemeeter.rust.sdPlugin/bin/streamdeck-voicemeeter.exe'))
for ($attempt = 0; $attempt -lt 50; $attempt++) {
    $running = @(Get-Process -Name streamdeck-voicemeeter -ErrorAction SilentlyContinue |
        Where-Object { $_.Path -and [IO.Path]::GetFullPath($_.Path) -eq $expected })
    if ($running.Count -eq 1) {
        Write-Output "Host launched plugin process $($running[0].Id)"
        exit 0
    }
    if ($running.Count -gt 1) { throw 'Multiple installed plugin processes found' }
    Start-Sleep -Milliseconds 200
}
throw 'Stream Deck did not launch the installed plugin within 10 seconds'
