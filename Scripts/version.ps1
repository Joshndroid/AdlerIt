# Shared version, checksum, and optional-signing helpers for AdlerIt Windows tooling.

function Get-AdlerItVersion {
    param([string]$RootDir)
    $cargo = Join-Path $RootDir "Cargo.toml"
    foreach ($line in Get-Content $cargo) {
        if ($line -match '^\s*version\s*=\s*"([^"]+)"') {
            return $Matches[1]
        }
    }
    throw "Could not read version from $cargo"
}

function Write-AdlerItChecksum {
    param([string]$Path)
    $hash = (Get-FileHash -Algorithm SHA256 -Path $Path).Hash.ToLower()
    $name = [IO.Path]::GetFileName($Path)
    $checksumPath = "$Path.sha256"
    "$hash  $name" | Set-Content -Path $checksumPath -Encoding ascii
    Write-Host "Wrote $checksumPath"
}

# Optionally Authenticode-sign a Windows binary when a code-signing certificate
# is configured via secrets. No-op (with a notice) when none is present, so the
# build still succeeds for unsigned/dev builds.
function Invoke-AdlerItOptionalSigning {
    param([string]$Path)

    $pfxBase64 = $env:WINDOWS_CERTIFICATE_PFX_BASE64
    $pfxPassword = $env:WINDOWS_CERTIFICATE_PASSWORD

    if ([string]::IsNullOrWhiteSpace($pfxBase64)) {
        Write-Host "No Windows code-signing certificate configured; leaving $Path unsigned."
        return
    }

    $signtool = Get-Command signtool.exe -ErrorAction SilentlyContinue
    if ($null -eq $signtool) {
        $candidate = Get-ChildItem "C:\Program Files (x86)\Windows Kits\10\bin" -Recurse -Filter signtool.exe -ErrorAction SilentlyContinue |
            Where-Object { $_.FullName -match 'x64' } | Select-Object -First 1
        if ($null -eq $candidate) { throw "signtool.exe not found but a signing certificate was provided." }
        $signtool = $candidate
    }

    $pfxPath = Join-Path $env:RUNNER_TEMP "adlerit-codesign.pfx"
    [IO.File]::WriteAllBytes($pfxPath, [Convert]::FromBase64String($pfxBase64))
    try {
        $args = @("sign", "/fd", "SHA256", "/tr", "http://timestamp.digicert.com", "/td", "SHA256", "/f", $pfxPath)
        if (-not [string]::IsNullOrWhiteSpace($pfxPassword)) { $args += @("/p", $pfxPassword) }
        $args += $Path
        & $signtool.Source @args
        if ($LASTEXITCODE -ne 0) { throw "signtool failed ($LASTEXITCODE)" }
        & $signtool.Source "verify" "/pa" $Path | Out-Null
        Write-Host "Signed $Path"
    }
    finally {
        Remove-Item $pfxPath -Force -ErrorAction SilentlyContinue
    }
}

# Offline-safety verification, ported from TypeText's offline-portable checks.
# AdlerIt is fully offline by design (no update checks, URL opening, networking,
# or startup-registry writes). This asserts that remains true by:
#   1. checking the dependency graph contains no networking/registry crates, and
#   2. scanning the built binary for capability markers it must never contain.
# Any match fails the build, catching a regression before release.
function Assert-AdlerItOfflineSafety {
    param(
        [string]$ExePath,
        [string]$Target = "x86_64-pc-windows-msvc",
        [string]$RootDir
    )

    if ([string]::IsNullOrWhiteSpace($RootDir)) {
        $RootDir = Split-Path -Parent $PSScriptRoot
    }

    Write-Host "Running offline-safety verification on $ExePath"

    # 1. Dependency graph must not include networking or registry crates.
    $deny = @(
        "reqwest", "hyper", "ureq", "curl", "isahc", "attohttpc", "surf",
        "native-tls", "openssl", "rustls", "winreg", "windows-registry"
    )
    Push-Location $RootDir
    try {
        $tree = cargo tree --target $Target --edges normal --prefix none --locked 2>$null
        if ($LASTEXITCODE -ne 0) { throw "cargo tree failed during offline-safety verification." }
    }
    finally {
        Pop-Location
    }
    $crates = $tree |
        ForEach-Object { ($_ -split '\s+')[0] } |
        Where-Object { $_ } |
        Sort-Object -Unique
    foreach ($crate in $deny) {
        if ($crates -contains $crate) {
            throw "Offline-safety verification failed: dependency graph includes '$crate'."
        }
    }

    # 2. Built binary must not contain network / URL / startup-registry markers.
    $text = [Text.Encoding]::ASCII.GetString([IO.File]::ReadAllBytes($ExePath))
    $forbidden = @(
        'Software\Microsoft\Windows\CurrentVersion\Run',
        'url.dll,FileProtocolHandler',
        'Invoke-WebRequest',
        'api.github.com'
    )
    foreach ($marker in $forbidden) {
        if ($text.Contains($marker)) {
            throw "Offline-safety verification failed: binary contains capability marker '$marker'."
        }
    }

    Write-Host "Offline-safety verification passed."
}
