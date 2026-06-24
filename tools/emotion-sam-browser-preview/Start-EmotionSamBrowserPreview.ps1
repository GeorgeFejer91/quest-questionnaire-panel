param(
    [switch]$NoOpen
)

$ErrorActionPreference = "Stop"

$previewRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$indexPath = Join-Path $previewRoot "index.html"

if (-not (Test-Path -LiteralPath $indexPath)) {
    throw "Preview entrypoint not found: $indexPath"
}

Write-Host "Emotion SAM browser preview:"
Write-Host "  $indexPath"
Write-Host ""
Write-Host "Reference fixtures:"
Write-Host "  $(Join-Path $previewRoot 'fixtures\default-state.json')"
Write-Host "  $(Join-Path $previewRoot 'fixtures\edge-cases.json')"

if (-not $NoOpen) {
    Start-Process -FilePath $indexPath
}
