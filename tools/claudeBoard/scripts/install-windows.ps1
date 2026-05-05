$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$targetDir = Join-Path $HOME '.claude-board'
$targetScript = Join-Path $targetDir 'hook-dispatch.ps1'

New-Item -ItemType Directory -Force -Path $targetDir | Out-Null
Copy-Item -Path (Join-Path $scriptDir 'hook-dispatch.ps1') -Destination $targetScript -Force

Write-Output "Installed claudeBoard hook wrapper at $targetScript"
Write-Output "Use this command in Claude Code hook settings: $targetScript"
