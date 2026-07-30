param([Parameter(Mandatory=$true)][string]$HostPath,[Parameter(Mandatory=$true)][string]$ExtensionId)
$dir=Join-Path $env:LOCALAPPDATA 'HyperGet\NativeMessaging';New-Item -ItemType Directory -Force -Path $dir|Out-Null
$manifest=Join-Path $dir 'io.github.hyper_get.json';@{name='io.github.hyper_get';description='Hyper Get native messaging bridge';path=$HostPath;type='stdio';allowed_origins=@("chrome-extension://$ExtensionId/")}|ConvertTo-Json|Set-Content -Encoding UTF8 $manifest
New-Item -Path 'HKCU:\Software\Google\Chrome\NativeMessagingHosts\io.github.hyper_get' -Force|Set-ItemProperty -Name '(default)' -Value $manifest
New-Item -Path 'HKCU:\Software\Microsoft\Edge\NativeMessagingHosts\io.github.hyper_get' -Force|Set-ItemProperty -Name '(default)' -Value $manifest
