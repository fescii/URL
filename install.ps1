$ErrorActionPreference = "Stop"

$Repo = "fescii/URL"
$Binary = "urls.exe"
$Asset = "urls-windows-x86_64.zip"
$DownloadUrl = "https://github.com/$Repo/releases/latest/download/$Asset"

$InstallDir = Join-Path $HOME ".urls\bin"
$ZipPath = Join-Path $env:TEMP $Asset

Write-Host "==> Downloading $Binary from $DownloadUrl..." -ForegroundColor Cyan
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
Invoke-WebRequest -Uri $DownloadUrl -OutFile $ZipPath -UseBasicParsing

Write-Host "==> Extracting $Asset..." -ForegroundColor Cyan
if (!(Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
}
Expand-Archive -Path $ZipPath -DestinationPath $InstallDir -Force
Remove-Item $ZipPath -Force

Write-Host "==> Configured installation directory: $InstallDir" -ForegroundColor Green

# Add to user PATH if not already present
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    Write-Host "==> Adding $InstallDir to user PATH environment variable..." -ForegroundColor Yellow
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    $env:Path += ";$InstallDir"
}

Write-Host "==> Successfully installed URLs!" -ForegroundColor Green
Write-Host "Run 'urls --help' in a new PowerShell window to get started." -ForegroundColor Cyan
