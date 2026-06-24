# Build the AdlerIt Windows installer with Inno Setup (fixed release name).
#   dist/AdlerIt-Windows-x64-Setup.exe
[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$RootDir = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot "version.ps1")

$version = Get-AdlerItVersion -RootDir $RootDir
$target = $env:ADLERIT_WINDOWS_TARGET
if ([string]::IsNullOrWhiteSpace($target)) { $target = "x86_64-pc-windows-msvc" }

$distDir = Join-Path $RootDir "dist"
$exePath = Join-Path $RootDir "target\$target\release\adlerit.exe"
$issPath = Join-Path $RootDir "packaging\windows\AdlerIt.iss"

Set-Location $RootDir
Write-Host "Version: $version"

cargo build --release --target $target --locked
if ($LASTEXITCODE -ne 0) { throw "cargo build failed ($LASTEXITCODE)" }
if (-not (Test-Path $exePath)) { throw "Expected build output not found: $exePath" }

# Verify the binary stays fully offline (no network/registry/update capability).
Assert-AdlerItOfflineSafety -ExePath $exePath -Target $target -RootDir $RootDir

Invoke-AdlerItOptionalSigning -Path $exePath
New-Item -ItemType Directory -Force -Path $distDir | Out-Null

$iscc = "C:\Program Files (x86)\Inno Setup 6\ISCC.exe"
if (-not (Test-Path $iscc)) { throw "Inno Setup compiler not found at $iscc" }

& $iscc "/DMyAppVersion=$version" "/DSourceExe=$exePath" "/DOutputDir=$distDir" $issPath
if ($LASTEXITCODE -ne 0) { throw "Inno Setup compilation failed ($LASTEXITCODE)" }

$out = Join-Path $distDir "AdlerIt-Windows-x64-Setup.exe"
if (-not (Test-Path $out)) { throw "Installer was not produced: $out" }
Invoke-AdlerItOptionalSigning -Path $out
Write-AdlerItChecksum -Path $out
Write-Host "Built $out"
