# Release
Tag `v0.1.0-alpha.1`. CI must pass first. Release builds target Linux AppImage/deb and Windows NSIS/MSI; generate SHA-256 files for every artifact. Packaging success is platform-dependent and must be verified per runner.

Push a tag or run **Release installers** manually in GitHub Actions. The Windows runner uploads an artifact containing the NSIS `.exe`, WiX `.msi`, and `SHA256SUMS-windows.txt`. Windows installers cannot be validated from a Linux workstation.
