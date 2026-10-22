$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.IO.Compression.FileSystem
$repositoryRoot = Split-Path -Parent $PSScriptRoot
$pluginId = 'jp.hrko.streamdeck.voicemeeter.rust'
$archivePath = Join-Path $repositoryRoot "dist/$pluginId.streamDeckPlugin"
$stagingPath = Join-Path $repositoryRoot "dist/$pluginId.sdPlugin"
$archive = [IO.Compression.ZipFile]::OpenRead($archivePath)
try {
    $count = 0
    foreach ($file in Get-ChildItem -LiteralPath $stagingPath -File -Recurse) {
        $relative = [IO.Path]::GetRelativePath($stagingPath, $file.FullName).Replace('\', '/')
        $entry = $archive.GetEntry("$pluginId.sdPlugin/$relative")
        if ($null -eq $entry) { throw "Archive missing $relative" }
        $stream = $entry.Open()
        try {
            $sha = [Security.Cryptography.SHA256]::Create()
            try { $actual = [Convert]::ToHexString($sha.ComputeHash($stream)) }
            finally { $sha.Dispose() }
        } finally { $stream.Dispose() }
        if ($actual -ne (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash) { throw "Archive content differs: $relative" }
        $count++
    }
    Write-Output "Verified $count archived files against staging."
} finally { $archive.Dispose() }
