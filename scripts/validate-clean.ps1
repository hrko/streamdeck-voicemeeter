param([switch]$Update)
$ErrorActionPreference = 'Stop'
$repositoryRoot = Split-Path -Parent $PSScriptRoot
$validationRoot = Join-Path $repositoryRoot ('target/cv-' + [Guid]::NewGuid().ToString('N').Substring(0,8))
$sourceRoot = Join-Path $validationRoot 'source'
New-Item -ItemType Directory -Path $sourceRoot -Force | Out-Null
$archivePath = Join-Path $validationRoot 'source.zip'
git -C $repositoryRoot archive --format=zip --output=$archivePath HEAD
if ($LASTEXITCODE -ne 0) { throw 'Could not archive HEAD' }
Expand-Archive -LiteralPath $archivePath -DestinationPath $sourceRoot
$variables = @{
    MISE_DATA_DIR = (Join-Path $validationRoot 'mise-data')
    MISE_CACHE_DIR = (Join-Path $validationRoot 'mise-cache')
    MISE_STATE_DIR = (Join-Path $validationRoot 'mise-state')
    MISE_GLOBAL_CONFIG_FILE = (Join-Path $validationRoot 'empty-global.toml')
    MISE_CONFIG_DIR = (Join-Path $validationRoot 'mise-config')
    MISE_RUSTUP_HOME = (Join-Path $validationRoot 'rustup')
    MISE_CARGO_HOME = (Join-Path $validationRoot 'cargo')
    RUSTUP_HOME = (Join-Path $validationRoot 'rustup')
    CARGO_HOME = (Join-Path $validationRoot 'cargo')
    pnpm_config_store_dir = (Join-Path $validationRoot 'pnpm-store')
    MISE_TRUSTED_CONFIG_PATHS = $validationRoot
}
Set-Content -LiteralPath $variables.MISE_GLOBAL_CONFIG_FILE -Value ''
$previous = @{}
foreach ($name in $variables.Keys) { $previous[$name] = [Environment]::GetEnvironmentVariable($name, 'Process'); [Environment]::SetEnvironmentVariable($name, $variables[$name], 'Process') }
$result = @{ sourceCommit = (git -C $repositoryRoot rev-parse HEAD); path = $validationRoot; startedAt = [DateTimeOffset]::UtcNow.ToString('o'); updated = [bool]$Update; status = 'running' }
Push-Location $sourceRoot
try {
    $locks = @{}
    foreach ($name in @('mise.lock','pnpm-lock.yaml','Cargo.lock')) { $locks[$name] = (Get-FileHash -LiteralPath $name -Algorithm SHA256).Hash }
    mise run setup
    if ($LASTEXITCODE -ne 0) { throw 'Clean setup failed' }
    foreach ($name in $locks.Keys) { if ((Get-FileHash -LiteralPath $name -Algorithm SHA256).Hash -ne $locks[$name]) { throw "Setup changed $name" } }
    if ($Update) { mise run deps:update; if ($LASTEXITCODE -ne 0) { throw 'Isolated dependency update failed' } }
    foreach ($task in @('check','test','package')) { mise run $task; if ($LASTEXITCODE -ne 0) { throw "Clean $task failed" } }
    if (!$Update) { foreach ($name in $locks.Keys) { if ((Get-FileHash -LiteralPath $name -Algorithm SHA256).Hash -ne $locks[$name]) { throw "Validation changed $name" } } }
    $result.status = 'passed'
} catch {
    $result.status = 'failed'
    $result.error = $_.Exception.Message
    throw
} finally {
    $result.finishedAt = [DateTimeOffset]::UtcNow.ToString('o')
    $result | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $validationRoot 'result.json')
    Pop-Location
    foreach ($name in $previous.Keys) { [Environment]::SetEnvironmentVariable($name, $previous[$name], 'Process') }
    Write-Output "Validation record: $validationRoot/result.json"
}
