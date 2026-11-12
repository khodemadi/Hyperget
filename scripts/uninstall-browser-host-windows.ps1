foreach ($key in @(
    'HKCU:\Software\Google\Chrome\NativeMessagingHosts\io.github.hyper_get',
    'HKCU:\Software\Microsoft\Edge\NativeMessagingHosts\io.github.hyper_get',
    'HKCU:\Software\BraveSoftware\Brave-Browser\NativeMessagingHosts\io.github.hyper_get',
    'HKCU:\Software\Mozilla\NativeMessagingHosts\io.github.hyper_get'
)) {
    Remove-Item $key -Recurse -Force -ErrorAction SilentlyContinue
}
$dir = Join-Path $env:LOCALAPPDATA 'HyperGet\NativeMessaging'
Remove-Item (Join-Path $dir 'io.github.hyper_get.json') -Force -ErrorAction SilentlyContinue
Remove-Item (Join-Path $dir 'io.github.hyper_get.firefox.json') -Force -ErrorAction SilentlyContinue
