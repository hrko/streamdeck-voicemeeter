param([Parameter(Mandatory)][string]$PluginDirectory)
$ErrorActionPreference = 'Stop'
$expected = [IO.Path]::GetFullPath((Join-Path $PluginDirectory 'bin/streamdeck-voicemeeter.exe'))
# Signal the worker to release held buttons and log out before process exit.
# Match the exact installed path; never stop another plugin or checkout.
Get-Process -Name streamdeck-voicemeeter -ErrorAction SilentlyContinue |
    Where-Object { $_.Path -and [IO.Path]::GetFullPath($_.Path) -eq $expected } |
    ForEach-Object {
        $signal = [Threading.EventWaitHandle]::OpenExisting("Local\streamdeck-voicemeeter-stop-$($_.Id)")
        try { $null = $signal.Set() } finally { $signal.Dispose() }
        if (-not $_.WaitForExit(10000)) { throw 'Plugin did not shut down gracefully; refusing to force termination.' }
    }
