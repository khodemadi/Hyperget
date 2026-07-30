Remove-Item 'HKCU:\Software\Google\Chrome\NativeMessagingHosts\io.github.hyper_get' -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item 'HKCU:\Software\Microsoft\Edge\NativeMessagingHosts\io.github.hyper_get' -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item (Join-Path $env:LOCALAPPDATA 'HyperGet\NativeMessaging\io.github.hyper_get.json') -Force -ErrorAction SilentlyContinue
