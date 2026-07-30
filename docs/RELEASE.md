# Release
Current tag target: `v0.2.2-alpha.1`. CI must pass first. Release builds target Linux AppImage/deb, Windows NSIS/MSI, and packaged Chromium/Firefox extensions. Every artifact group includes SHA-256 checksums.

Push a tag or run **Release bundles** manually in GitHub Actions. The Windows runner uploads NSIS `.exe`, WiX `.msi`, and Windows checksums. The Linux runner uploads AppImage/deb. A separate job publishes both browser extension ZIPs. Windows installers cannot be validated from a Linux workstation.
