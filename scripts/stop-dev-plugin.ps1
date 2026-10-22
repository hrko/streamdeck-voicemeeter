param([Parameter(Mandatory)][string]$PluginDirectory)
$ErrorActionPreference = 'Stop'
$expected = [IO.Path]::GetFullPath((Join-Path $PluginDirectory 'bin/streamdeck-voicemeeter.exe'))
# The official stop command can return while the native child is still alive.
# Match the exact installed path; never stop another plugin or checkout.
Get-Process -Name streamdeck-voicemeeter -ErrorAction SilentlyContinue |
    Where-Object { $_.Path -and [IO.Path]::GetFullPath($_.Path) -eq $expected } |
    ForEach-Object {
        if (-not $_.WaitForExit(3000)) {
            $_ | Stop-Process -Force
            $_.WaitForExit()
        }
    }
