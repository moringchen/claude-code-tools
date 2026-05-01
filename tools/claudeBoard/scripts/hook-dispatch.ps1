$payload = [Console]::In.ReadToEnd()
$url = 'http://127.0.0.1:46123/events'
$bufferPath = if ($env:CLAUDE_BOARD_BUFFER_PATH) { $env:CLAUDE_BOARD_BUFFER_PATH } else { Join-Path $HOME '.claude-board/events.jsonl' }
$debugEnabled = -not [string]::IsNullOrEmpty($env:CLAUDE_BOARD_DEBUG)

try {
    Invoke-RestMethod -Uri $url -Method Post -ContentType 'application/json' -Body $payload | Out-Null
    exit 0
} catch {
    if ($debugEnabled) {
        Write-Error ("claudeBoard dispatch failed: {0}" -f $_.Exception.Message)
    }
}

$directory = Split-Path -Parent $bufferPath
if ($directory) {
    New-Item -ItemType Directory -Force -Path $directory | Out-Null
}
Add-Content -Path $bufferPath -Value $payload
