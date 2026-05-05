$targetDir = Join-Path $HOME '.claude-board'
$targetScript = Join-Path $targetDir 'hook-dispatch.ps1'
if (Test-Path $targetScript) {
    Remove-Item -Path $targetScript -Force
}

Write-Output "Removed claudeBoard hook wrapper at $targetScript"
Write-Output "Buffered events in $targetDir were left intact. Remove that directory manually if you want a full purge."
Write-Output 'Remove the Claude Code hook command that referenced hook-dispatch.ps1 if it is still configured.'
