param(
    [Parameter(Mandatory=$true)][string]$HostPath,
    # Defaults match the pinned "key" in extensions/browser/chromium/manifest.json and the Firefox gecko id.
    [string]$ExtensionId = 'cjlafmbgkfhgahjdndnaakkljodbncei',
    [string]$FirefoxId = 'hyper-get@example.invalid'
)
$ErrorActionPreference = 'Stop'

if (-not [System.IO.Path]::IsPathRooted($HostPath) -or -not (Test-Path -LiteralPath $HostPath -PathType Leaf)) {
    throw "Host binary not found (use an absolute path): $HostPath"
}

$dir = Join-Path $env:LOCALAPPDATA 'HyperGet\NativeMessaging'
New-Item -ItemType Directory -Force -Path $dir | Out-Null
$utf8NoBom = New-Object System.Text.UTF8Encoding $false

$chromiumManifest = Join-Path $dir 'io.github.hyper_get.json'
$chromiumJson = @{
    name = 'io.github.hyper_get'
    description = 'Hyper Get native messaging bridge'
    path = $HostPath
    type = 'stdio'
    allowed_origins = @("chrome-extension://$ExtensionId/")
} | ConvertTo-Json -Depth 5
[System.IO.File]::WriteAllText($chromiumManifest, $chromiumJson, $utf8NoBom)

# Firefox needs its own manifest: "allowed_extensions" instead of "allowed_origins".
$firefoxManifest = Join-Path $dir 'io.github.hyper_get.firefox.json'
$firefoxJson = @{
    name = 'io.github.hyper_get'
    description = 'Hyper Get native messaging bridge'
    path = $HostPath
    type = 'stdio'
    allowed_extensions = @($FirefoxId)
} | ConvertTo-Json -Depth 5
[System.IO.File]::WriteAllText($firefoxManifest, $firefoxJson, $utf8NoBom)

function Register-Host([string]$key, [string]$manifest) {
    New-Item -Path $key -Force | Out-Null
    Set-ItemProperty -Path $key -Name '(default)' -Value $manifest
}

Register-Host 'HKCU:\Software\Google\Chrome\NativeMessagingHosts\io.github.hyper_get' $chromiumManifest
Register-Host 'HKCU:\Software\Microsoft\Edge\NativeMessagingHosts\io.github.hyper_get' $chromiumManifest
Register-Host 'HKCU:\Software\BraveSoftware\Brave-Browser\NativeMessagingHosts\io.github.hyper_get' $chromiumManifest
Register-Host 'HKCU:\Software\Mozilla\NativeMessagingHosts\io.github.hyper_get' $firefoxManifest

Write-Host "Registered native messaging host for Chrome, Edge, Brave and Firefox."
Write-Host "Chromium extension ID: $ExtensionId"
Write-Host "Firefox extension ID:  $FirefoxId"
